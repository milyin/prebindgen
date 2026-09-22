<!-- spec: {"kind": "stage", "stage": "03-requests"} -->

[Project contents](../README.md) · Previous: [Build and inspect the source model](02-flat.md) · Next: [Select conversion relations](04-select.md)

# Record binding requests

The implemented frontends translate user configuration into two things: the
`BindingRequests` naming what to generate, and their own `Target`, which holds
what each of those declarations *is*. This chapter first explains that
translation, then describes the request identities and
[conversion](04-select.md#select-conversion-relations)-sharing rules. Some later
types are design sketches: in particular, owned Flat views and
placement-specific declaration ids are not implemented.

The examples in this chapter use one small source crate — a struct and a function
over it, marked for binding generation:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64;
```

Capturing `Stamp` and `stamp_sum` makes them available for inspection; it does
not automatically publish either in a foreign API. The next decision is yours:
which items should callers see, under what names, and in what form? You make
those decisions in a binding crate's build script. This stage records them as
requests that the engine can attempt to fulfill.

A binding crate is an ordinary Rust crate whose build script configures a
**language frontend** — the public Rust API of `prebindgen-c` or
`prebindgen-jni` — and asks it to generate. Exposing these two items through C is two declarations:

```rust
// build.rs of the C binding crate
Cbindgen::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)
    .source_module(parse_quote!(source_crate))
    .declare(
        decls!()
            .data_type(
                data_type!(Stamp)          // Stamp crosses as a C struct, by value
                    .base_name("Stamp"),   // under this name, not the default `stamp`
            )
            .fun(fun!(stamp_sum)),         // and this function becomes a C entry point
    )
    .build_with(prebindgen_c::pipeline::Pipeline::V2)
    .expect("generate the C binding");
```

In this builder, `.source` locates captured Rust items, and `.source_module`
specifies the Rust path generated code uses to reach their implementation.
`.declare` receives the requested C API. `data_type!(Stamp)` selects a type;
`.base_name("Stamp")` names its C [representation](05-represent.md#represent-and-compose-values); `fun!(stamp_sum)` selects the
function. Finally, `build_with` runs generation with the V2 engine. Enable the
frontend's `v2` Cargo feature for these examples. Plain `.build()` instead
reads `PREBINDGEN_PIPELINE` and defaults to V1 when the variable is unset.
Imports are omitted from the excerpts.

The set is flat, because an exported C function is not a member of anything:
`stamp_sum(Stamp)` is a free function that happens to take a `Stamp`, and the
header declares it beside the type rather than inside it. Each declaration
carries its own options, so the set means the same however it is ordered.

Names come from the frontend's manglers. A function keeps its Rust name by
default, which is why `stamp_sum` is the exported symbol; a type's default base
is the snake_case of its short name, so `Stamp` would reach C as `stamp`, which
is why the declaration above names it. Generator-wide naming hooks do the same
job for every declaration at once, and are configured on the same builder —
before `.declare(...)`, since names are derived when the set is applied.

Through Kotlin it is the same two items with different answers — `Stamp` becomes
a class in a package, and the function a top-level function of that package,
reached through the Java Native Interface (JNI), the mechanism by which JVM code
calls into a compiled library:

```rust
// build.rs of the JNI binding crate
JniGen::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)
    .set_package_prefix("example")
    .package(
        package!()
            .class(data_class!(Stamp))   // Stamp becomes a Kotlin data class
            .fun(prebindgen_registry::fun!(stamp_sum)), // a top-level function
    )
    .build_with(prebindgen_jni::pipeline::Pipeline::V2)
    .expect("generate the JNI binding");
```

The Kotlin configuration also describes placement. `set_package_prefix("example")`
sets the base package, including the location of `JNINative`. `package!()` puts
these public items in that same package. A named `package!("sub")` would add a
subpackage for its public items without moving the harness. `data_class!(Stamp)`
asks for a data class; `.fun(...)` places a top-level function in that package. The
frontend derives names unless configuration overrides them. A name given there
names the Kotlin declaration a caller uses; the
`external` method behind it, and so the `Java_…` symbol, is named from the Rust
identifier through the method-name hook — the harness object has one namespace,
and two packages may each export a `value`. The macros take a type as written
rather than a string — `data_class!(Stamp)`, not `data_class!("Stamp")` — and
record its spelling. A function declaration must name a captured function, and
one that does not is an error when the requests meet the model; a class
declaration need not name a captured type at all — a target may represent
`String` without the source exporting one — so `data_class!(Absent)` is a
reported skip (`unsupported.type.not_a_struct`) rather than an error.

The adapter also chooses how conversion failures reach the caller. For example,
reading a JVM property can fail before the Rust function runs. The adapter
supplies a reporting convention, rather than a per-function setting. The
[wrapper-boundary stage](06-boundary.md) uses that convention when planning the
[wrapper](06-boundary.md#assemble-the-wrapper-boundary)'s error path.

Each such call records a choice. Together they are the **binding
configuration**, and `.build()` is where the frontend turns it into
`BindingRequests` — the input the registry actually consumes. Users never write
that structure; frontends do, which is why the two builders above can be as
different as their languages while everything after this stage is shared.

A request set separates the desired output from choices about its
implementation. An **output request** asks for a function, type or other public
declaration, and that is *all* it carries: the declaration's identity. The
language-specific choices that apply to it — a C struct or a JVM object to
carry a `Stamp`, which header name, which `Java_…` symbol — stay in the
frontend's own storage, which is the same object the registry later asks its
questions of as the `Target`. The engine therefore does not need to understand
any C or JNI configuration option, and has no table of them to keep in step
with the frontend's.

This stage also fixes the names by which everything is addressed afterwards. A
**declaration** is one requested output, identified by a `DeclarationId` —
exposing the same Rust function at two Kotlin placements makes two of them, with
separate [outcomes](07-retain.md#retain-supported-output), which the engine cannot express yet (its identity is the kind
and the Rust origin). A **site** is a position inside such a declaration:
parameter 0 of the exported `stamp_sum`, or its
return. A **part** is a position inside a source value: the `secs` field of
`Stamp`, or the single argument of a `stamp_from_millis` constructor. Sites and
parts together are the **positions** a conversion can be planned at. Overrides
attach to sites and parts, and so do diagnostics, which is why a skipped binding
can later say *which* parameter of *which* exported function was the problem.

Precedence between those choices is the frontend's, not the engine's: a target
looks for an override at the value's path, then a choice for the type, then its
own default. A value path can identify a nested field, such as
`param 0.field secs`, and the engine supplies it as the `Position` handed to
[`Target::select`](04-select.md#how-the-registry-asks-a-target-for-decisions).
What comes back is a **conversion key**, which affects whether a plan can be
reused. The two implemented frontends have no per-site declarator today, so
both answer from the type alone; the engine's own test target exercises the
full precedence. The separate part-rule table shown later is a design
extension.

Recording a request claims nothing about feasibility; whether a well-formed
request can actually be generated is not known until the next stage tries.

## What V2 has to accept

V2 preserves the existing frontend APIs: the same builders and macros, the same
recorded choices, over the same captured Rust inputs. What it changes is the
engine behind `.build()`, which is why a request set can be built from a
configuration written for V1.
[#719](https://github.com/milyin/prebindgen/issues/719) covers introducing it and
switching the existing examples over. The types sketched below are schematic
contracts; capabilities not yet implemented are extension points, not features.

## What the registry does

The registry receives the [source model](02-flat.md) and requests to expose particular types, functions, and constants. For each request, it builds a **plan**: structured data describing the required conversions, calls and their execution order. The common Rust writer renders the wrappers and supporting Rust types. The C build then uses `cbindgen` to derive C headers from that Rust output; the JNI implementation renders Kotlin declarations from the completed plans. Planning happens in the generator; the generated operations execute later when the bindings are used.

Take the struct above and a second function over it — one that also
*returns* a `Stamp`, so that both directions are visible at once:

```rust
fn normalize(stamp: Stamp) -> Stamp;
```

Generating a binding for it requires several decisions and operations:

1. Choose how a foreign caller represents a `Stamp`.
2. Obtain and convert its `secs` and `nanos` values.
3. Construct the Rust `Stamp` and call `normalize`.
4. Read and convert the returned fields.
5. Return the result through the chosen foreign interface.

A C binding might represent `Stamp` as a C struct. A JNI binding might accept two integer arguments produced by the public Kotlin function, or receive a JVM object whose properties must be read. Those [representations](05-represent.md#represent-and-compose-values) need different target operations. The source-side work of discovering two fields, converting them, constructing `Stamp`, invoking the source function, and processing its result is common.

**The registry owns that common work.** When a struct gains another nested struct field, the shared recursive registry algorithm should process it using the representations supplied by the target. Each language should not need another implementation of struct traversal or wrapper assembly.

A language implementation also owns its **foreign writer**: the component that renders the target language's own declarations from the completed plans. JNI's writes Kotlin. C's is the exception — [emission](08-emit.md) explains why it delegates to `cbindgen` instead. The registry library provides the **common Rust writer**, which emits the Rust wrappers and supporting Rust types for both targets.

The source crate and Flat have finished their work by the time requests exist.
These are the roles that act from here on — roles, not crates: the frontend and
the target adapter are the two faces of one language adapter crate, and the
common Rust writer belongs to the engine.

<table>
<thead>
<tr><th>Component</th><th>Responsibility</th><th>Example C</th><th>Example Kotlin</th></tr>
</thead>
<tbody>
<tr>
<td>Language frontend</td>
<td>Provide the public language-specific Rust API; pass the registry what to expose and what to leave alone, and keep the recorded choices, answering the registry's questions about them as its target.</td>
<td>Expose <code>Stamp</code> as a C data struct and <code>normalize</code> as a C function.</td>
<td>Expose <code>Stamp</code> as a Kotlin data class in a specified package and place <code>normalize</code> in the requested API.</td>
</tr>
<tr>
<td>Registry</td>
<td>Plan Rust field access, value construction and helper calls; combine child conversions and track dependencies.</td>
<td colspan="2">Convert both fields, construct the Rust <code>Stamp</code>, call <code>normalize</code> once, and process its result using the target's representation operations.</td>
</tr>
<tr>
<td>Target adapter</td>
<td>Describe representations, runtime operations and boundary conventions.</td>
<td>Represent <code>Stamp</code> as a C struct with member-access operations; select the declared C return/out-parameter and error conventions.</td>
<td>Represent <code>Stamp</code> as separate JNI arguments or a JVM object with property-read operations; select the declared JNI result and error-handler conventions.</td>
</tr>
<tr>
<td>Common Rust writer</td>
<td>Render the registry's completed conversion and function plans.</td>
<td colspan="2">Emit Rust locals, field accesses, source calls, branches and the extern wrapper from the common plans, using the chosen calling convention and target operations.</td>
</tr>
<tr>
<td>Foreign writer</td>
<td>Render the target language's own declarations from the completed plans.</td>
<td>Delegated: the build invokes the external <code>cbindgen</code> tool on the generated Rust, which is where C headers come from.</td>
<td>Emit the Kotlin <code>Stamp</code> class and typed wrappers that call the generated JNI boundary.</td>
</tr>
</tbody>
</table>

The implementation divides the registry's data between two structures:

- **The generation operation**, `generate(flat, target, requests)`, is a free
  function that starts a fresh run. Three arguments, three kinds of thing: the
  source model, the language, and what this binding asks of it. `requests` is
  the work list — what to expose and what to leave alone; `target` implements
  the adapter interface and holds the frontend's choices, which is where every
  question about them goes. Current V2 selects
  atomic conversions or struct-field construction. Constructors, accessors
  and other helper [relations](04-select.md#what-a-relation-is) described by the design are future extensions.
- **The run**, private `Run` state in `plan.rs`, keeps requests, offered
  relations, conversion plans, the cache and cycle-detection marks. `generate`
  accumulates declaration outcomes and checks public dependencies. It returns
  a completed **`Generation`** or a generation error.

A binding crate normally builds one configured frontend. The frontend calls `generate` once for all requests. `Flat` supplies source facts; the registry plans conversions using the run's working state.

## Binding requests and target choices

The common [`prebindgen-flat` library](https://github.com/milyin/prebindgen/blob/main/prebindgen-flat/src/lib.rs) supplies Rust source facts through `Flat`. Users call the C or JNI frontend's API to choose the generated interface: for example, exposing `Stamp` as a C data struct or a Kotlin class. The configured frontend creates `BindingRequests` and calls the registry.

A declaration or setting recorded by the frontend is a **configuration entry**. A frontend call exposing a function records an output request; an argument override records the requested representation choice. The frontend also preserves naming hooks, ignored items and unsupported settings. Recording a request does not establish that the registry can generate it.

### What a choice records

A **choice** is one target-specific decision the binding recorded about a
request or about a particular value conversion — that `Stamp` crosses as a C
struct passed by value, under the name `Stamp`. It is configuration data. It
does not contain a recursive conversion algorithm or a finished wrapper.

It belongs to the frontend that recorded it, and to the target that frontend
builds. The registry holds no table of them and knows no precedence among them: it
asks the target, and the target resolves what applies from its own storage.

Examples include JNI object versus separate-argument input, C struct versus
handle, and function error handling or public placement.

An illustrative portion of what JNI records for one value could be:

```rust
// Illustrative choices, not a replacement for the existing builder/macro API.
enum JniStructInput {
    SeparateArguments, // Kotlin supplies the fields as individual JNI arguments.
    ObjectProperties,  // JNI receives an object and reads its properties.
}

struct JniValueChoice {
    record_input: JniStructInput, // How this struct reaches the wrapper.
}
```

What C records for the same value is a different shape entirely, because the
choices are different — there is no environment, no object, and no property to
read:

```rust
// Illustrative choices, again not a replacement for the existing builder API.
enum CStructShape {
    Aggregate,   // A repr(C) struct, passed and returned by value.
    OpaquePtr,   // A pointer to a Rust-owned value, with a typed drop.
}

struct CValueChoice {
    shape: CStructShape, // How this struct crosses the C boundary.
    c_name: String,      // Its name in the generated header.
}
```

Neither type is known to the registry at all — not as a generic parameter, not
as a table entry. Each frontend stores its own and interprets it in its own
`Target`. That is what lets one engine serve two languages whose choices have
nothing in common, without the engine naming either.

Neither choice lists `Stamp`'s fields or explains how to construct it. The registry obtains those facts through a [relation](04-select.md#what-a-relation-is) — a link inside the source domain from a Rust type to the values it is built from or read into, such as its fields or a helper's argument; where the contrast with the target side matters, the chapters call one a *source relation*. The adapter reads its own storage when describing the target representation and its property/argument operations.

Three concepts stay separate throughout the design:

| Concept | Question it answers | `Stamp` example |
| --- | --- | --- |
| Relation | How can the Rust value be constructed or read? | Construct/read its `secs` and `nanos` fields. |
| Target representation | What values carry it, and how are those values accessed? | One C struct, two JNI integer arguments, or a JVM object. |
| Boundary delivery | Where do the converted values go at an exported call? | The wrapper's return, caller-provided output parameters, or a declared result callback. |

The binding's choices guide the selection of these descriptions. The registry turns the descriptions into an executable plan.

### The registry API called by the frontend

The registry separates what should be generated from how values should be converted. An **output request** asks for one declaration, such as a function, type or constant. A **conversion rule** selects a relation and a way of converting for a particular type, parameter, result or child value. The two live in different places, which is the whole of this chapter's boundary:

```rust
// What the frontend hands the registry.
struct BindingRequests {
    declaring_crate: String,      // The crate generating, for the report.
    source_module: Path,          // How generated Rust reaches the source items.
    requests: Vec<Request>,       // Expose this, or leave that alone; defined below.
}

// What the frontend keeps and answers the registry's questions from —
// `CbindgenBuilder` and the `CTarget` it builds, schematically.
struct FrontendStorage {
    conversion_rules: ConversionRules,     // Per-type, per-position and default choices.
    declared: Map<DeclarationId, CChoice>, // What each requested output is.
    // …plus the naming hooks, which are closures and go nowhere.
}
```

The registry meets a conversion rule one value at a time, as the conversion key
the target returns from `select`; it never sees the table. A setting the
frontend cannot lower needs no list of its own either — it becomes an ordinary
output request that the target then refuses by name, so the report accounts for
it like anything else.

Neither structure is generic. C and JNI use the same `BindingRequests`, and
differ only in the `Target` they pass beside it — which is also where the
report's name for the language comes from, as `Target::NAME`, since an adapter
knows what it is. These sketches explain the responsibilities;
`prebindgen-registry-v2/src/plan.rs` defines the exact current fields.

The registry plans what is to be exposed, reports what is left alone, and asks the adapter for every choice that applies.

Suppose the user configures the JNI frontend to accept `Stamp` as two integer arguments by default, then overrides the `Stamp` parameter of function `f` to accept a JVM object. The JNI target resolves that override when the registry asks it to select a relation for `f`'s parameter 0, and returns a different conversion key than it does for function `g`, which has no override. A choice recorded for a particular field or constructor argument is applied the same way, where that child is converted, following the frontend API's documented override rules.

Identical type, construction and representation choices produce an equal key and can share a converter; the object override produces a different key and needs a different converter.

A setting the frontend cannot honor needs no record of its own: the declaration it applies to is requested like any other, and the target refuses it by name when the registry asks. Identity, location and reason then reach the report through the ordinary [skip](07-retain.md#retain-supported-output), and propagate as one.

The engine's one entry point (signature only):

```rust
pub fn generate<T: Target>(
    flat: Flat,
    target: &T,
    requests: BindingRequests,
) -> Result<Generation<T::Payload>, EngineError>;
```

`T: Target` ties the adapter to its [conversion-key and rendering-payload types](04-select.md#how-the-registry-asks-a-target-for-decisions). The function takes the model, borrows the target — which is also where every choice the binding recorded lives — consumes the requests, and builds private working state. The returned `Generation` owns the model, the retained plans and payloads. Unsupported requests are [outcomes](07-retain.md#retain-supported-output) of the run; a declaration that must name a captured item and does not, invalid input or an invariant failure return `EngineError`. Rendering and I/O follow planning.

Inside the C frontend's build implementation after selecting v2 — the whole
chain from capture to planning, in internal pseudocode rather than user
`build.rs` code:

**Implemented.** This is today's call path: `CbindgenBuilder::build()` under v2
reads its own declaration storage once, turns it into a request set *and* the
C target that holds what each request is, and hands both to the engine. The JNI
frontend does the same with its declarations and the JNI target. Nothing of v1
runs on this route — no `declare_into`, no resolution, no assembly — and the
frontend's part is naming: which C name a type or a symbol gets is its manglers
applied, the same answer v1 gives.

```rust
let source_model = self.sources.clone().build()?;   // stage 2: the snapshot

let (target, requests) = self.binding(source_module);  // this stage
let generation = generate(source_model, &target, requests)?;
                                                     // stages 4 to 6
```

Both halves are built in one pass over the builder's storage, sorted so that a
run over unchanged input emits the same file. Stating them together is what
keeps them in step: a declaration cannot be planned without the target knowing
what it is, and asking for an output the target recorded nothing about is
invalid input rather than a silent default. A declarator the target has no
lowering for — an enum, a tagged union, a callback signature — still becomes a
request,
recorded as the declarator it came from, so the target refuses it by name, and
the skip carries the capability it waits for.

The frontend and registry can both inspect
[source items](01-source.md#capture-source-items) through `prebindgen-flat`
directly. The frontend translates user declarations; the engine validates
their requested source names and kinds, discovers required fields and plans
conversions. Proposed helper relations will also need argument validation and
planning. The registry supplies no separate source-inspection API to the frontend.

Request construction must lose no recorded frontend choice. Today it carries the
choices this increment lowers — names, the class a type is declared as, the
ignore rules — in the target it builds, and turns the settings it does not
lower into a **refusal** of what they apply to: a per-function `expand_param`/`expand_return`/
`split_on_param` refuses the function; a type-level boundary declaration refuses
every function with a parameter or result of that type, declared class or bare
scalar alike, and leaves the class itself; a declarator the target does not
lower refuses by that declarator's name. Emitting the default interface in place
of the one a setting asked for is not honoring the declaration; the skip names
the setting the declaration waits on. Settings that have no effect within this
increment are carried without refusing: a C function's
`abort_on_conversion_error` says what a fallible input does, and no conversion
here can fail. Local helpers and declared conversion operations are
refused the same way (`unsupported.fn.binding_local`,
`unsupported.conversion.not_implemented`) until they are registered as typed
source descriptions. Naming closures remain owned configuration objects,
applied where the requests are built; nothing is serialized.

## Identifying requests, value positions and reusable conversions

Names ending in `Id` identify particular records, but they do not all have the
same lifetime or construction rules. `RelationId` and `NodeId` identify entries
used within a generation run; a **conversion key** plays the same role for the
settings a target applied, except that the target mints it and the registry
only compares it. A `Declaration` is instead its
own identity — a stable value naming the kind the target gets and the Rust
item, printed as `fn:stamp_sum`; reports and tests can use it across runs.
(`DeclarationId` below is this chapter's name for that role.) The proposed `SiteId`
and `PartId` describe positions within a declaration or relation. Keeping these
identities separate prevents a field position from being confused with a public
function or a reusable conversion.

### Where planning starts

To generate a wrapper for the source function `normalize(stamp: Stamp) -> Stamp`, the registry needs an input conversion, the call to `normalize` itself and an output conversion. The request to expose `normalize` is the starting point, called a **root**. The conversions required to implement that request are its **dependencies**. A request to expose a public type is also a root, even if no function uses that type.

```rust
enum Request {
    Expose(DeclarationId), // Plan this and generate it.
    Ignore(CapturedName),  // Plan nothing; the report carries it as a decision.
}

// A captured item by name: the kind the source captures it as, and what it is
// called there. Flat's three kinds, stated once as `Entity<T, F, C>`.
type CapturedName = Entity<TypeKey, Ident, Ident>;
```

An ignore carries less than a declaration, deliberately. It names a captured
item and says nothing about how the target would get it, so a callback, a
conversion or anything the binding coined cannot be ignored — those are not
captured, and there is nothing in the source to leave alone. What a report row
and a duplicate check compare an ignore by is the declaration that would have
exposed the same item.

One list rather than two, because the report has one row per declaration and
both dispositions produce one. An id that appeared under both would be two
rows for one declaration, so saying both about one declaration is refused
where the duplicate check already runs. Existence is a separate question, and
it is asked only of what is to be exposed: an ignore means "if this is here,
leave it alone", which a binding may reasonably say about an item its source
crate compiles out under a feature.

A requested output is that identity and nothing else. What the declaration *is*
— its symbol, its placement, the declarator it came from — the target looks up
under it when the registry asks for a boundary, a public declaration or a
report line. What it *depends on* is the target's answer too, at
[`surface`](04-select.md#how-the-registry-asks-a-target-for-decisions):
`SurfaceSpec.requires` names the public types a declaration is unusable
without — a wrapper taking an aggregate needs the type declared as well — and
[retention](07-retain.md#retain-supported-output) resolves each name to the
declaration covering it. Requiring a type does not export it: a type nothing
requested is not emitted because something needed it, and whatever needed it is
skipped instead.

A `DeclarationId` is a `Declaration`: one value that is its own identity and
says both which of the kinds the target gets and which captured item, if any,
the engine plans it from. The kinds the source captures are Flat's three, and
the two variants that name one carry it as an `Entity`:

```rust
enum Declaration {
    Captured(Entity<TypeKey, Ident, Ident>),  // a captured item, exposed as itself
    ConstFromFunction(Ident),   // a Kotlin `val` read through a captured nullary function
    Local(Entity<TypeKey, String, String>),   // the binding's own type, function or constant
    Callback(String),           // a callback signature the binding exports
    Conversion(TypeKey),        // a wire mapping the binding defines for a type
}
```

`Captured` is the same `CapturedName` an ignore carries, so exposing an item
and ignoring it meet as one identity. `ConstFromFunction` is the one
declaration whose kind on the target side differs from its kind in the source;
a callback and a conversion have no source kind at all. `generate` matches on
the whole, so each planner is reached by the variants it can plan and is
handed the captured item they name. One source item has one declaration, so a
request that names the same declaration twice is refused.

### A value's position in an exported function

The registry needs to locate the parameter affected by a per-function override. A **site** is such a position: for example, parameter 0 of the requested `normalize` binding. `SiteId` identifies that position so the registry can apply its override and report problems there.

```rust
struct SiteId {
    owner: DeclarationId, // The declared function containing the position, e.g. normalize.
    path: SitePath,   // Param(0), Return, or a nested callback argument position.
}
```

### Fields, constructor arguments and enum variants

`Stamp` can be built from its `secs` and `nanos` fields or by calling `stamp_from_millis(millis: i64) -> Stamp`. Both are relations of `Stamp` — parallel links out of the same type, leading to different values — so each has a separate `RelationId`, and a conversion says which one it took. Constructor parameters need not match the fields in name, type or number: the registry converts `millis` and calls the helper; the helper computes the fields.

A **part** is a field or argument converted within that relation. `Stamp.fields` (a descriptive label, not Rust syntax) has two parts; the constructor relation has one, `millis`. `PartId` identifies which part a conversion rule applies to.

For an enum such as `enum Event { At(Stamp), Count(u32) }`, the variant is also needed to identify a part, because its parts are not all converted together the way a struct's are: one arm's parts are live at a time, and the others are not reached at all. An **arm** is one alternative, and `ArmId` identifies it: here, `At` or `Count`. Each variant has a field at position 0, but those fields belong to different arms. A declared choice between constructors can also use arm IDs. Ordinary struct fields and a single constructor have no alternatives, so their arm is `None`.

```rust
struct PartId {
    owner: RelationId,       // Relationship defining the part, e.g. Stamp.fields.
    arm: Option<ArmId>,      // Enum variant/declared alternative; None without alternatives.
    position: PartPosition,  // Field identity, helper argument index, or projector result.
}
```

For example, `(Stamp.fields, None, Field("secs"))` identifies a struct field; `(Event.variants, Some(At), Field(0))` identifies `At`'s payload. These are illustrative IDs. `owner` refers to the containing relation, not Rust memory ownership. Function-specific overrides and diagnostics remain attached to `SiteId` positions.

### Finding an existing conversion plan

Every conversion has a direction: input travels into the source Rust API, while
a return value travels out. A **crossing** pairs that direction with the exact
source type. The proposed interface below uses `TypeView`, a handle that also
retains the source model; current V2 uses `TypeRef` and keeps the model in the
generation result. See [type readings and type views](02-flat.md#type-readings-and-type-views)
for the planned ownership change. Frontends supply type information, while the
registry derives the keys used to find reusable plans:

```rust
enum Direction {
    IntoRust,  // Produce the Rust value expected by a source function.
    OutOfRust, // Encode a Rust result or callback argument for foreign code.
}

struct Crossing {
    source: TypeView,      // Exact type and retained Flat snapshot, including wrappers.
    direction: Direction, // Whether the value enters or leaves the Rust API.
}
```

A reusable conversion plan is a [node](05-represent.md#represent-and-compose-values). The registry finds nodes using a private `NodeKey`, derived internally from the accepted `Crossing`, the selected relation and the conversion key the target returned with it. The key includes the children, so it identifies a whole subgraph rather than a single value: equal subgraphs become one node, which is what makes the result a graph with sharing instead of a tree of repeated plans. No frontend/adapter conversion-planning API accepts `TypeKey` or `NodeKey`, or a caller-supplied type/key pair.

```rust
// Private to the registry's conversion cache module; not a public request type.
struct NodeKey<ConversionKey> {
    source: TypeKey,       // Derived internally from crossing.source.key().
    direction: Direction, // Copied from that crossing.
    relation: RelationId, // Validated selected relation.
    conversion: ConversionKey, // The target's name for the settings it applied.
    children: Vec<NodeId>,// The conversions its parts resolved to.
}
```

The private cache operation accepts the validated crossing and selection, derives the key, and retains the same `TypeView` in the plan. Key construction and cache mutation are private; plan descriptors expose typed readings. Callers cannot submit mismatched type/key pairs, or ask for a conversion by handing in a key they built themselves.

The existing structural reading `TypeRef` has no `Eq`/`Hash`; its `key()` returns `prebindgen_flat::TypeKey`, which supplies both. `key()` preserves references/mutability, wrappers, generic arguments, array extents and lifetime spelling. Flat normalizes parentheses and known equivalent paths, such as `std::vec::Vec<T>` and `Vec<T>`, without equating arbitrary aliases. `stripped_key()` removes outer `Box`/`Cow` wrappers for declaration lookup: `Box<Stamp>` finds the `Stamp` declaration. The proposed `TypeView::key()` delegates to its retained reading. The conversion cache uses that key to retain wrappers. Plans retain the view for model-aware inspection and emission; key text cannot recreate a view.

`conversion` identifies the choices for the whole value. `children` identifies
the conversions selected for its fields or other parts. Both affect reuse. If
two functions accept `Stamp` but one applies a different conversion to `secs`,
their struct conversions must differ too. Omitting the child identities from the
cache key would incorrectly reuse the first function's field behavior.

For example, two owned `Stamp` inputs with the same two-integer JNI representation and field construction can share a node. An object-input override makes the target answer with a different conversion key; a rule on one of their `secs` fields changes that child, and therefore the struct's conversion; a return conversion changes direction. `Stamp`, `&Stamp` and `Option<&Stamp>` remain distinct.

Model membership follows the [snapshot contract](02-flat.md#private-storage-and-model-consistency). Flat publishes immutable source data after helper registration; its views preserve that snapshot through field and parameter navigation. Registry operations that accept a view check it against their own model before planning, rather than relying on the caller to check first. Flat owns these checks and private view construction. A valid view from another snapshot is rejected even when its key text matches. The registry accepts no detached reading or independently supplied model/type pair as a substitute for a view.

Keys are local to one `Flat` model; Flat owns normalization. `NodeId`
identifies a retained plan, and registry-issued node references must be
validated within their generation context. Function sites retain separate
overrides and diagnostic paths.

A **conversion key** is the target's own name for one way of converting a
value, returned from `select` beside the relation and compared — never read —
by the registry. It carries one obligation, and it is the target's:
*equal keys mean interchangeable conversions* — same layout, same operations,
same failures, same release — and settings that would generate differently must
produce different keys. The registry checks neither, because it cannot look
inside the key; what it does with equal keys is share one node, and what it
does with a key already being resolved is refuse the conversion as recursive. A
target that minted a fresh key on every visit would therefore share nothing and
would recurse where it should refuse. Both implemented targets key on plain
data, so two values the binding declared the same way are converted the same
way. A target whose settings held something incomparable — a naming closure,
which cannot be compared to another closure — interns it and keys on the
index.

Outcomes are keyed by source and configuration identities that do not vary
between runs over unchanged inputs, so two builds of the same crate decide the
same things.

## Elements at this stage

- [Function taking an owned struct][fn_requests] · [C][fn_requests_c] · [Kotlin/JNI][fn_requests_jni]
- [Struct with scalar fields][struct_requests] · [C][struct_requests_c] · [Kotlin/JNI][struct_requests_jni]
- [Type alias declaring an opaque handle][typedef_requests] · [C][typedef_requests_c] · [Kotlin/JNI][typedef_requests_jni]

[fn_requests]: ../examples/fn/03-requests.md
[fn_requests_c]: ../examples/fn/03-requests.c.md
[fn_requests_jni]: ../examples/fn/03-requests.jni.md
[struct_requests]: ../examples/struct/03-requests.md
[struct_requests_c]: ../examples/struct/03-requests.c.md
[struct_requests_jni]: ../examples/struct/03-requests.jni.md
[typedef_requests]: ../examples/typedef/03-requests.md
[typedef_requests_c]: ../examples/typedef/03-requests.c.md
[typedef_requests_jni]: ../examples/typedef/03-requests.jni.md
