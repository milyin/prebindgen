<!-- spec: {"kind": "stage", "stage": "03-requests"} -->

[Project contents](../README.md) · Previous: [Build and inspect the source model](02-flat.md) · Next: [Select conversion relations](04-select.md)

# Record binding requests

The frontends translate user configuration into two things: a `Binding`,
which is data — the
[carriers](05-represent.md#describing-target-values-and-operations) generated
Rust may use, the rules saying how
each value's [conversion](04-select.md#select-conversion-relations) is made,
and each `Declaration` with the form it is exposed in — and their own
`Target`, which writes what the registry feeds it. This chapter first
explains that translation, then describes the request identities and
conversion-sharing rules. Some later
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
configuration**, and `.build()` is where the frontend turns it into a
`Binding`, which is the input the registry actually consumes.
Users never write a `Binding`; frontends do, which is why the two builders above can be as
different as their languages while everything after this stage is shared.

An **output request** asks for a function, type or other public declaration,
and carries two things: the declaration's identity, and the form that says
what this output *is* — the representation of a type's own value, the symbol
and calling convention of an exported function. The engine keeps the form
with the output. Carrying it is what lets a binding declare one entity twice,
since a table the engine looked the form up in by declaration would hold one
of the two.

Every other choice — which way values of a type cross wherever they appear,
what a setting on one parameter of one function overrides — is a
[conversion rule](#conversion-rules), handed to the engine as data beside the
declarations. The engine holds the table of rules and applies the one
precedence among them, so planning is a lookup into data the binding stated
before planning began, and the whole configuration can be checked and
printed.

This stage also fixes the names by which everything is addressed afterwards. A
**declaration** is one requested output, identified by an `OutputId` standing
for the declaration and the form [recorded](#what-a-choice-records) with it —
exposing the same Rust function at two Kotlin placements makes two of them, with
separate [outcomes](07-retain.md#retain-supported-output). A **site** is a position inside such a declaration:
parameter 0 of the exported `stamp_sum`, or its
return. A **part** is a position inside a source value: the `secs` field of
`Stamp`, or the single argument of a `stamp_from_millis` constructor. Sites and
parts together are the **positions** a conversion can be planned at. Overrides
attach to sites and parts, and so do diagnostics, which is why a skipped binding
can later say *which* parameter of *which* exported function was the problem.

Precedence between those rules is the engine's, and there is one: a rule
at the value's own position, then a rule for its type. A position can
identify a nested field, such as `param stamp.field secs`. What a rule
carries is a representation, which also decides whether a plan can be
reused. The two implemented frontends record type rules only, since neither
has a per-site declarator that V2 lowers; the engine's own test target records
rules at positions too.

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

- **The generation operation**, `generate(flat, target, binding,
  source_module)`, is a free function that starts a fresh run.
  The first three arguments are three kinds of thing: the source model, the
  language, and what this binding asks of it. `binding` is the work list —
  what to expose — and the conversion rules; `target` implements the adapter
  interface and answers what only the language knows. Current V2 selects
  atomic conversions or struct-field construction. Constructors, accessors
  and other helper [relations](04-select.md#what-a-relation-is) described by the design are future extensions.
- **The run**, private `Run` state in `plan.rs`, keeps the outputs — each
  declaration with the choice recorded for it — the rules, the registered
  relations, conversion plans, the cache and cycle-detection marks. `generate`
  accumulates declaration outcomes and checks public dependencies. It returns
  a completed **`Generation`** or a generation error.

A binding crate normally builds one configured frontend. The frontend calls `generate` once for all requests. `Flat` supplies source facts; the registry plans conversions using the run's working state.

## Binding requests and target choices

The common [`prebindgen-flat` library](https://github.com/milyin/prebindgen/blob/main/prebindgen-flat/src/lib.rs) supplies Rust source facts through `Flat`. Users call the C or JNI frontend's API to choose the generated interface: for example, exposing `Stamp` as a C data struct or a Kotlin class. The configured frontend pairs each declaration with what it declared it as, and calls the registry.

A declaration or setting recorded by the frontend is a **configuration entry**. A frontend call exposing a function records an output request; an argument override records the requested representation choice. The frontend also preserves naming hooks and unsupported settings. An ignore is not an entry: it tells V1 not to warn that an item is undeclared, and V2 issues no such warning, so the frontend hands the engine no ignores and the ignored item stays in the model like any other undeclared item. Recording a request does not establish that the registry can generate it.

### What a choice records

A **choice** is one target-specific decision the binding recorded — that
`Stamp` crosses as a C struct passed by value, under the name `Stamp`. It is
configuration data. It does not contain a recursive conversion algorithm or a
finished wrapper.

The frontend turns every choice into one of four kinds of declaration before
planning starts, and hands all of them to the registry:

| A choice about | Becomes | Read by |
| --- | --- | --- |
| a Rust type generated code may hold at the boundary | a [carrier](05-represent.md#describing-target-values-and-operations): its Rust type, and the metadata the target's writers need — a C name, a JVM descriptor | the registry, to type the plan; the target, to write |
| how the values in some scope cross | a [conversion rule](#conversion-rules): a scope, and the [representation](05-represent.md#represent-and-compose-values) of the values it covers | the registry |
| how one function is exported | a function form: calling convention, symbol, the extra parameters the convention adds, and a route per failure category | the registry |
| what only the foreign declaration shows | output metadata: a Kotlin name and package | the target's foreign writer |

So the split of concerns is fixed at this stage. The registry decides, from
these declarations alone, which carriers each value uses, which operations
move it, and how every wrapper is put together. The target writes: given what
the registry feeds it — an operation and its already-typed operands, a
carrier and its members — it produces Rust text, and for Kotlin the foreign
declarations. It decides nothing the plan depends on, because by the time it
is called the plan is complete.

Metadata, and the target's own operations, are the two things the registry
holds without reading. It compares them for equality. A carrier's metadata
matters to that comparison, because two carriers with the same Rust type may
differ in it: every JVM object is a `JObject`, and a
`JObject` holding an `example.Stamp` is a different carrier from one holding
an `other.Stamp`. What the metadata says is the writers' business: the JNI
writer needs the class to construct an object going out of Rust, and the
descriptor to call a getter going in.

No choice lists `Stamp`'s fields or explains how to construct it. The registry obtains those facts through a [relation](04-select.md#what-a-relation-is) — a link inside the source domain from a Rust type to the values it is built from or read into, such as its fields or a helper's argument; where the contrast with the target side matters, the chapters call one a *source relation*. A representation names which relation it reads a value through; the registry walks it.

Three concepts stay separate throughout the design:

| Concept | Question it answers | `Stamp` example |
| --- | --- | --- |
| Relation | How can the Rust value be constructed or read? | Construct/read its `secs` and `nanos` fields. |
| Target representation | What values carry it, and how are those values accessed? | One C struct, two JNI integer arguments, or a JVM object. |
| Boundary delivery | Where do the converted values go at an exported call? | The wrapper's return, caller-provided output parameters, or a declared result callback. |

### The registry API called by the frontend

The frontend builds one value, the `Binding`, and hands it to the registry
with its `Target`. The `Binding` is everything planning reads; the `Target`
is only [the writers](04-select.md#what-the-target-writes).

```rust
pub struct Binding<T: Target> { /* private */ }

impl<T: Target> Binding<T> {
    /// A carrier generated Rust may use. Declaring an equal carrier again
    /// returns the same id.
    pub fn carrier(&mut self, carrier: CarrierOf<T>) -> CarrierId;
    /// One way a type crosses. Declaring an equal representation again
    /// returns the same id.
    pub fn representation(&mut self, representation: RepresentationOf<T>) -> ReprId;
    /// The values `scope` covers cross as `representation`.
    pub fn rule(&mut self, scope: Scope, representation: ReprId);
    /// Expose `declaration` in this form. The id is how a rule addresses a
    /// value inside the output.
    pub fn output(&mut self, declaration: Declaration, form: OutputFormOf<T>) -> OutputId;
}

/// One of a target's few wire types — C's `I64`, `Pointer`, `Aggregate`;
/// JNI's `Long`, `Int`, `Handle`, `Object` — stated the same way by every
/// target.
pub trait WireClass: Clone + Eq + Hash + Debug {
    fn all() -> Vec<Self>;          // Every class: what accepting anything accepts.
    fn name(&self) -> &'static str; // `pointer`, in `unsupported.c.member.pointer`.
    fn rust(&self) -> syn::Type;    // `i64`, `jni::sys::jlong`; `_` for a declared name: `*mut _`.
}

pub struct WireType<C, M> {         // `CarrierOf<T>` is `WireType<T::WireClass, T::CarrierMeta>`.
    rust: syn::Type,                // The class's type, read through `rust()`.
    pub class: C,                   // Which of the adapter's few wire types it is.
    pub members: Option<Accepts<C>>, // For an aggregate, the classes its members may be.
    pub meta: M,                    // What the target's writers need to know of it.
}

impl<C: WireClass, M> WireType<C, M> {
    /// A carrier of a class naming one exact type.
    pub fn exact(class: C, members: Option<Accepts<C>>, meta: M) -> Self;
    /// A carrier of a type the target declares: the class's type, `name` for `_`.
    pub fn declared(class: C, name: Ident, members: Option<Accepts<C>>, meta: M) -> Self;
}

pub enum OutputForm<Op, C, O> {    // `OutputFormOf<T>` fills in the target's own types.
    /// One representation of a type, exposed: its carriers' Rust
    /// declarations, and its foreign declaration. It is also the rule at
    /// this output's root.
    Type {
        representation: ReprId,
        release: Option<FunctionForm<Op, C>>, // The wrapper releasing a handed-out value.
        meta: O,
    },
    /// A function, exported through a wrapper of this form.
    Function { form: FunctionForm<Op, C>, meta: O },
    /// A declarator the target does not lower, refused by name.
    Unsupported(Unsupported),
}

pub struct FunctionForm<Op, C> {
    pub abi: String,                   // "C", "system"
    pub symbol: String,                // stamp_sum, Java_example_JNINative_stampSum
    pub context: Vec<ContextParam>,    // Parameters no source parameter feeds: `env: JNIEnv`.
    pub inputs: Vec<syn::Ident>,       // The names of the parameters that carry the inputs.
    pub routes: Vec<FailureRoute<Op>>, // Per failure category: how it is reported, how the call ends.
    pub attrs: Vec<syn::Attribute>,
    pub unsafety: bool,
    pub params: Accepts<C>,            // The classes a wrapper parameter may be,
    pub ret: Accepts<C>,               // and the wrapper's return.
}
```

A carrier's Rust type is its class's, built by the registry from the class
rather than restated by each carrier, so the two cannot disagree: C's
`Pointer` is `*mut _`, and a carrier of it declared as `Ledger` is
`*mut Ledger`. What a refusal calls a class is its `name`.

`Accepts` lists the wire classes a holder may hold: an aggregate's members, a
wrapper's parameters, its return; `Accepts::any()` is every class. The registry checks each placement once the
plan has worked out what is placed there, and refuses the value it would put
anywhere else. The [extensions page](../extensions.md#acceptance) gives the
rule in full, with the limits other targets would state in it.

A representation is one way a type crosses. A type may have several — `Stamp`
as a C struct and as a handle — and each is declared once and referred to by
its `ReprId`, from rules and from outputs alike. It is the answer a target
used to give when asked, stated in advance instead:

```rust
pub enum Representation<Op> {
    /// The whole value, one operation each way: a scalar, a handle, a
    /// fieldless enum. Each direction has a carrier of its own: a C enum
    /// arrives as `MaybeUninit` of itself.
    Terminal {
        into_rust: Option<Codec<Op>>,   // None: never crosses into Rust.
        out_of_rust: Option<Codec<Op>>,
        release: Option<Operation<Op>>, // How the foreign side gives a held value back.
    },
    /// The parts of a relation, carried together in one carrier, into Rust.
    Product {
        via: Via,              // Which relation: Fields.
        carrier: CarrierId,    // The aggregate or object holding the parts.
        read: Operation<Op>,   // One part out of the carrier, applied per part.
    },
    /// A representation the target does not lower, refused by name.
    Unsupported(Unsupported),
}

pub struct Codec<Op> {
    pub carrier: CarrierId,
    pub operation: Operation<Op>,
}

pub struct Operation<Op> {
    pub implementation: Implementation<Op>, // Standard(StandardOp) or Target(Op)
    pub context: Vec<String>,               // Runtime contexts it needs, by name: "jni.env".
    pub failure: Option<Failure>,           // Its failure category and error type, if it can fail.
}
```

An operation states no operand or result types. Its place in the
representation fixes them: a `Terminal`'s `into_rust` codec takes its carrier
and produces the source type, a `Product`'s `read` takes the carrier and
produces the part's carrier, whatever carrier that part resolves to. The
registry works them out when it plans the value and feeds them to the writer.
A standard operation — a member read, a handle taken back or released, a
fieldless enum matched value by value — is one the registry writes itself,
because writing it means naming a source type.

For the C `Stamp`, with the frontend's own `CCarrier` as the carrier metadata:

```rust
let i64_c = binding.carrier(WireType::exact(CClass::I64, None, CCarrier::Builtin));
let unchanged = Codec { carrier: i64_c, operation: Operation::standard(StandardOp::Identity) };
let i64_whole = binding.representation(Representation::Terminal {
    into_rust: Some(unchanged.clone()),
    out_of_rust: Some(unchanged),
    release: None,
});
binding.rule(Scope::Type(key!(i64)), i64_whole);

let stamp_c = binding.carrier(WireType::declared(
    CClass::Aggregate,                          // `_`: a struct the C target declares
    format_ident!("Stamp"),
    Some(Accepts::of([CClass::I64])),           // Not yet another aggregate, a handle or an enum.
    CCarrier::Aggregate { c_name: "Stamp".into() },
));
let stamp_struct = binding.representation(Representation::Product {
    via: Via::Fields,
    carrier: stamp_c,
    read: Operation::standard(StandardOp::ReadMember),
});
binding.rule(Scope::Type(key!(Stamp)), stamp_struct);   // every Stamp value
binding.output(Declaration::Type(key!(Stamp)),          // and the struct, exposed
               OutputForm::Type { representation: stamp_struct, release: None, meta: () });
```

and for the JNI one, with `Jvm { descriptor, kotlin }` as the metadata:

```rust
let jlong = binding.carrier(WireType::exact(
    JniClass::Long,
    None,
    Jvm { descriptor: "J".into(), kotlin: KotlinType::Value("Long".into()) },
));
// i64: a Terminal over `jlong`, as C's is over `i64`.
let stamp_obj = binding.carrier(WireType::exact(
    JniClass::Object,                             // every JVM object is a `JObject`
    Some(Accepts::of([JniClass::Long])),          // What a getter returning a `long` reads.
    Jvm {
        descriptor: "Lexample/Stamp;".into(),
        kotlin: KotlinType::Value("example.Stamp".into()),
    },
));
let stamp_class = binding.representation(Representation::Product {
    via: Via::Fields,
    carrier: stamp_obj,
    read: Operation::target(JniOp::Getter)
        .context("jni.env")
        .fails(FailureCategory::Runtime, parse_quote!(jni::errors::Error)),
});
binding.rule(Scope::Type(key!(Stamp)), stamp_class);
```

The C declarations use only standard operations, so C's `Target::Op` has no
values at all. A scalar kind is one `Type` rule: each frontend records the
scalars its target carries — `i64`, for both so far — before the binding's
own declarations. There is no default for a type no rule covers: a
representation names a carrier, and one carrier cannot fit every type. A
value no rule covers is refused as `unsupported.conversion.no_rule`, naming
the type. How a generic instance such as `Vec<Stamp>` would cross without a
rule of its own is [designed](../extensions.md#containers), and not built.

### Conversion rules

A **conversion rule** names the representation of the values in a scope: every
value of one type, or one value at one position inside one output.

```rust
pub enum Scope {
    /// Every value of this type, wherever it turns up.
    Type(TypeKey),
    /// The one value at this path inside this output. The empty path is the
    /// output's own root: a type declaration's value, in both directions.
    At(OutputId, ValuePath),
}

/// From an output's root to one value inside it.
pub struct ValuePath(pub Vec<Step>);

pub enum Step {
    /// A source function's parameter, by the name the source gives it.
    Param(String),
    /// A source function's result.
    Return,
    /// A field of the struct the current value is read through: its name,
    /// or its position for a tuple field.
    Field(String),
    /// An argument of the callback the current value is, by position.
    Arg(usize),
}
```

A **value path** is that `ValuePath`, printed with its steps joined by dots:
`param stamp`, `return`, `param stamp.field secs`, `param each.arg 0`. It is
the public
vocabulary for a [site](#a-values-position-in-an-exported-function) — its
first step — and for the
[parts](#fields-constructor-arguments-and-enum-variants) below it; a
diagnostic prints the same path. Steps name what the source names, so a rule
reads the way a build script author thinks of the value, and a renamed
parameter makes the rule fail validation instead of silently applying
nowhere.

When the registry plans a value it uses the rule at the value's own position
if there is one, and the rule for its type otherwise, whole. That is the only
precedence. Nothing is merged between the two: a rule at a position replaces
the type's rule for that value, and says nothing about the value's children,
which are looked up again at their own positions.

Suppose the user configures the JNI frontend to accept `Stamp` as two integer
arguments by default, then overrides the `Stamp` parameter of function `f` to
accept a JVM object. The frontend records a `Type(Stamp)` rule for the
default and an `At(f, param stamp)` rule for the override, and the registry
applies the second at `f`'s parameter and the first everywhere else. The two
representations differ, so the two plans stay apart. A rule for a particular
field is recorded the same way, one step deeper.

### What the registry checks before planning

The registry checks the binding against the model before it plans anything,
and fails the build with invalid input when:

- two rules have the same scope, which is the binding saying two things about
  one value — a rule at a type output's own root among them, since the
  output's representation already is that rule;
- a function form names a symbol that is not a Rust identifier, or restates
  the linkage the writer owns with `#[no_mangle]` or `#[export_name]`;
- a rule at `At(output, path)` names a position the output does not have: a
  parameter the function does not take, a `return` on a function returning
  nothing, a field of a value whose representation is not read through
  `Fields`, or a field the struct does not have.

The last check resolves each path the way planning will: each step's value
has its rule looked up in the same order, and a `Field` step needs that
rule's representation to be a `Product` through `Fields`. So
`param stamp.field secs` is valid while `Stamp` crosses through its fields,
and becomes invalid if a rule makes it an opaque handle — a rule that would
otherwise sit unused under a value nothing reads into.

Whether each wire value may sit where the plan puts it — a member in an
aggregate, a parameter in a wrapper signature — depends on what the children
resolve to, so the registry checks that during planning instead, and a
failure there is a refusal, not invalid input: see
[acceptance](../extensions.md#acceptance).

A `Type` rule no planned value used is not an error: a frontend's scalar
rules cover kinds a binding may never mention, and a binding may declare a
type no function uses. Such rules are listed in `Generation::unused_rules`,
which a build script may print.

The binding is also printable, one line per carrier, representation, rule
and output, a function form's signature, acceptance and routes on the lines
under its output, wire classes by their names, and metadata in its `Debug`
form:

```text
carrier  c0  i64  i64  no members  Builtin
carrier  c1  Stamp  aggregate  members [i64]  Aggregate { c_name: "Stamp" }
repr     r0  terminal  in: c0 Standard(Identity)  out: c0 Standard(Identity)
repr     r1  product  c1  Fields  read: Standard(ReadMember)
rule     type i64  r0
rule     type Stamp  r1
output   type:Stamp  type r1  ()
output   fn:stamp_sum  function  ()
         form  extern "C" stamp_sum  context []  inputs [stamp]
         form  params [i64, pointer, aggregate, enum, enum_bits, closure]  ret [i64, pointer, aggregate, enum, enum_bits, closure]
         form  route binding: abort
```

The text prints every field planning reads, which makes it the diagnostic to
diff when two builds of one binding generate differently. It does not prove
two bindings equal: the target's classes, metadata and operations print only
as much as their `Debug` form tells apart.

### How the frontends build a binding

Nothing here is generic. C and JNI hand over the same kind of `Binding`, and
differ in its metadata and operations and in the `Target` they pass beside it
— which is also where the language's name comes from, as `Target::NAME`, since
an adapter knows what it is. The generated file carries it.

The engine's one entry point (signature only):

```rust
pub fn generate<T: Target>(
    flat: Flat,
    target: &T,
    binding: Binding<T>,
    source_module: syn::Path,
) -> Result<Generation<T>, EngineError>;
```

The function takes the model, borrows the target's writers, and consumes the binding. Planning reads `flat` and `binding` only; the target is first called when the registry writes. The returned `Generation` owns the model, the retained plans and the written Rust. Unsupported requests are [outcomes](07-retain.md#retain-supported-output) of the run; a declaration that must name a captured item and does not, a binding the checks above reject, invalid input or an invariant failure return `EngineError`. Rendering and I/O follow planning.

Inside the C frontend's build implementation after selecting v2 — the whole
chain from capture to planning, in internal pseudocode rather than user
`build.rs` code:

```rust
let source_model = self.sources.clone().build()?;   // stage 2: the snapshot

let binding = self.binding(&source_model);           // this stage
let generation = generate(source_model, &CTarget, binding, source_module)?;
                                                     // stages 4 to 6
```

`CbindgenBuilder::binding()` reads the builder's storage once, sorted so that
a run over unchanged input emits the same file. It records the scalar table,
then for each declared type a carrier, a representation, the `Type` rule
and the output naming that representation — `data_type!` a `Product`
through `Fields`, `ptr_type!` a `Terminal` over a `*mut` carrier with a
release — for each `callback!` signature a closure-struct carrier, a
`Callback` representation, its `Type` rule and a `Declaration::Callback`
output, and for each function an output with its
function form. The JNI frontend does the same with its declarations, and
states a callback for every `impl Fn(..)` an exported function takes, since
its build scripts declare none. A check
that needs the source model runs here too: a C enum numbering a value
outside `i32` is recorded as `Representation::Unsupported`, with the reason,
rather than refused later. Neither target holds anything: every choice it
writes from arrives with what it is asked to write. What JNI knows of the whole
binding — the package prefix, the harness object's name — is its Kotlin
writer's, which reads the finished generation.

A declarator the target has no lowering for — a tagged union, a declared
conversion — still becomes an output. A type's is exposed as, and ruled by, a
`Representation::Unsupported`, so a value of it is refused too; anything
else's is recorded as `OutputForm::Unsupported`. It is refused by the
declarator's name before anything under it is planned, and the skip carries
the capability it waits for.

The frontend and registry can both inspect
[source items](01-source.md#capture-source-items) through `prebindgen-flat`
directly. The frontend translates user declarations; the engine validates
their requested source names and kinds, discovers required fields and plans
conversions. Proposed helper relations will also need argument validation and
planning. The registry supplies no separate source-inspection API to the frontend.

Request construction must lose no recorded frontend choice. Today it carries
the choices this increment lowers — names, the class a type is declared as —
beside the declaration they were recorded for, or in the target it builds for
the ones that are about a type's values rather than about one output, and
turns the settings it does not lower into a **refusal** of what they apply to: a per-function `expand_param`/`expand_return`/
`split_on_param` refuses the function; a type-level boundary declaration refuses
every function with a parameter or result of that type, declared class or bare
scalar alike, and leaves the class itself; a declarator the target does not
lower refuses by that declarator's name. Emitting the default interface in place
of the one a setting asked for is not honoring the declaration; the skip names
the setting the declaration waits on. Settings that have no effect within this
increment are carried without refusing: a C function's
`abort_on_conversion_error` says what a fallible input does, and no conversion
here can fail. A helper the binding defines with `fun!(crate::x).sig(..)`, and
an opaque type it declares although the source never exported it, enter the
model as entities of their own before anything is planned — see
[what an entity is](02-flat.md#build-and-inspect-the-source-model) — so a
declaration names them as it names a captured item. A constant the binding
computes on the foreign side and a declared conversion are refused
(`unsupported.const.computed`, `unsupported.conversion.not_implemented`).
Naming closures remain owned configuration objects, applied where the requests
are built; nothing is serialized.

## Identifying requests, value positions and reusable conversions

Names ending in `Id` identify particular records, but they do not all have the
same lifetime or construction rules. `RelationId` and `NodeId` identify entries
used within a generation run, and so does the `CarrierId` a binding's carrier
is registered under. A `Declaration` is instead a
stable value naming the kind the target gets and the Rust item, printed as
`fn:stamp_sum`; an `OutputId` stands for one of those together with the form
recorded with it, and is what the run is keyed by and what a rule's position
is rooted at. (`DeclarationId` in later chapters' sketches is that role.) A
value path describes a position within an output. Keeping these
identities separate prevents a field position from being confused with a public
function or a reusable conversion.

### Where planning starts

To generate a wrapper for the source function `normalize(stamp: Stamp) -> Stamp`, the registry needs an input conversion, the call to `normalize` itself and an output conversion. The request to expose `normalize` is the starting point, called a **root**. The conversions required to implement that request are its **dependencies**. A request to expose a public type is also a root, even if no function uses that type.

An **entity** is one real item of the API: a type, a function or a constant,
with its whole description. The model holds more than that — a guard, an
unsupported item — and does not rank what it holds; that a binding can name
exactly these three kinds, and nothing else, is the registry's judgment, and
the three variants of `Declaration` that name an entity are where it states
it. Where an entity came from is not a kind: a captured item and one the
binding stated are the same entity, and differ in origin alone.

A declaration is that identity and nothing else. What the declaration *is* —
its symbol, its placement, the declarator it came from — is the form
recorded beside it. What it *depends on* follows from the plan, and the
registry works it out without asking: a wrapper taking a `Stamp` aggregate is
unusable unless the `Stamp` carrier is declared too, and
[retention](07-retain.md#retain-supported-output) finds the type output whose
representation uses that carrier. Requiring a type does not export it: a type nothing
requested is not emitted because something needed it, and whatever needed it is
skipped instead.

An `OutputId` stands for one output the frontend recorded: a
`Declaration` and the form recorded with it. The `Declaration` says which of
the kinds the target gets and which entity, if any, the engine plans it from:

```rust
enum Declaration {
    Function(Ident),        // exported through a wrapper that calls it
    Const(Ident),           // exposed as a foreign constant
    Type(TypeKey),          // given a foreign representation
    Conversion(TypeKey),    // a wire mapping the binding defines for a type
    Callback(TypeKey),      // a callback signature the binding exports: an `impl Fn(..)`
    ComputedConst(String),  // a constant the binding computes on the foreign side
}
```

Where an entity came from is not part of this. A `#[prebindgen]` function and
a helper the binding defines are both a `Function`, because the model holds
both and only an entity's origin tells them apart; the same goes for a
captured type and an opaque one the binding declares over a type the source
never exported. Nor is how the target shows the entity: a Kotlin `val` read
through a nullary function, `constant!(X).fun(fun!(f))`, is `Function(f)`, and
the `val` is the target's choice recorded under that declaration. A callback
and a computed constant name no entity at all: a callback is named by its
signature, the type of the `impl Fn(..)` parameters that take one, and prints
as `callback:impl Fn(i64)+Send+Sync+'static`. `generate` matches on the
whole, so each planner is reached by the variants it can plan and is handed
the entity they name.

One entity may be declared more than once — `stamp_sum` as a Kotlin `fun` in
one package and as the `val` a `constant!` reads through it, `Stamp` as a data
class and as a handle. Each is an output of its own, planned and accounted for
on its own, and what tells them apart is the choice recorded with it: the
engine compares choices for equality and never reads one. The declaration is
unchanged by any of this, which is why a binding declaring each entity once is
described exactly as it always was; declaring one entity twice as the same
thing is the binding saying one thing twice, and is refused.

The declaration is all the engine can say about an output afterwards, so two
outputs of one entity are indistinguishable in what a run leaves out:
`Generation::skipped` returns the same `Declaration` twice, once per output it
could not generate. What separates them is the target's own vocabulary — a
Kotlin placement, a C symbol — which the engine does not speak, so a binding
that needs its diagnostics to tell them apart says so in its own terms, from
the choices it recorded.

Two declarations of one type are two representations of it, each exposed.
A type output names the representation it exposes, which is the rule at its
root, `At(output, [])`, and outranks the type's rule there and nowhere else.
So `Stamp` declared as a C struct and as a handle is two representations,
two outputs each naming one, and at most one `Type` rule saying which of the
two a `Stamp` parameter gets:

```rust
let stamp_struct = binding.representation(/* Product over the `Stamp` carrier */);
let stamp_handle = binding.representation(/* Terminal over `*mut stamp_t`, with a release */);
binding.rule(Scope::Type(key!(Stamp)), stamp_struct);
binding.output(Declaration::Type(key!(Stamp)), OutputForm::Type { representation: stamp_struct, meta: () });
binding.output(Declaration::Type(key!(Stamp)), OutputForm::Type { representation: stamp_handle, meta: () });
```

The two outputs differ by the representation they name, so they are two
outputs rather than one stated twice; exposing one representation twice is
the duplicate. A binding that records two `Type` rules
for one type fails the
[checks](#what-the-registry-checks-before-planning) instead of having one
silently win. Which declaration a *value* requires is then settled at
[retention](07-retain.md#retain-supported-output).

### A value's position in an exported function

The registry needs to locate the parameter affected by a per-function override. A **site** is such a position: for example, the `stamp` parameter of the requested `normalize` binding, or its result. It is the first step of a [value path](#conversion-rules), `Step::Param("stamp")` or `Step::Return`, under the output that exports the function, and it is how the registry applies an override there and reports problems there. A callback's argument is one more step below the parameter that receives the callback: `param each.arg 0`, `Step::Arg(0)`, valid where that parameter is represented as a callback.

### Fields, constructor arguments and enum variants

`Stamp` can be built from its `secs` and `nanos` fields or by calling `stamp_from_millis(millis: i64) -> Stamp`. Both are relations of `Stamp` — parallel links out of the same type, leading to different values — so each has a separate `RelationId`, and a conversion says which one it took. Constructor parameters need not match the fields in name, type or number: the registry converts `millis` and calls the helper; the helper computes the fields.

A **part** is a field or argument converted within that relation. `Stamp.fields` (a descriptive label, not Rust syntax) has two parts; the constructor relation has one, `millis`. A step of a value path identifies which part a conversion rule applies to: `Step::Field("secs")` for a field; a constructor's argument would be a step of its own kind.

For an enum such as `enum Event { At(Stamp), Count(u32) }`, the variant is also needed to identify a part, because its parts are not all converted together the way a struct's are: one arm's parts are live at a time, and the others are not reached at all. An **arm** is one alternative, and `ArmId` identifies it: here, `At` or `Count`. Each variant has a field at position 0, but those fields belong to different arms. A declared choice between constructors can also use arm IDs. Ordinary struct fields and a single constructor have no alternatives, so their arm is `None`.

For example, `param event.arm At.field 0` would identify `At`'s payload: the arm is a step of the path, between the value and its field.

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

A reusable conversion plan is a [node](05-represent.md#represent-and-compose-values). The registry finds nodes using a private `NodeKey`, derived internally from the accepted `Crossing` and the representation the applicable rule recorded, which names the relation too. The key includes the children, so it identifies a whole subgraph rather than a single value: equal subgraphs become one node, which is what makes the result a graph with sharing instead of a tree of repeated plans. No frontend/adapter conversion-planning API accepts `TypeKey` or `NodeKey`, or a caller-supplied type/key pair.

```rust
// Private to the registry's conversion cache module; not a public request type.
struct NodeKey {
    source: TypeKey,       // Derived internally from crossing.source.key().
    direction: Direction, // Copied from that crossing.
    representation: ReprId, // Carrier, operations and relation: equal ids are equal content.
    children: Vec<NodeId>,// The conversions its parts resolved to.
}
```

The private cache operation accepts the validated crossing and selection, derives the key, and retains the same `TypeView` in the plan. Key construction and cache mutation are private; plan descriptors expose typed readings. Callers cannot submit mismatched type/key pairs, or ask for a conversion by handing in a key they built themselves.

The existing structural reading `TypeRef` has no `Eq`/`Hash`; its `key()` returns `prebindgen_flat::TypeKey`, which supplies both. `key()` preserves references/mutability, wrappers, generic arguments, array extents and lifetime spelling. Flat normalizes parentheses and known equivalent paths, such as `std::vec::Vec<T>` and `Vec<T>`, without equating arbitrary aliases. `stripped_key()` removes outer `Box`/`Cow` wrappers for declaration lookup: `Box<Stamp>` finds the `Stamp` declaration. The proposed `TypeView::key()` delegates to its retained reading. The conversion cache uses that key to retain wrappers. Plans retain the view for model-aware inspection and emission; key text cannot recreate a view.

`representation` identifies how the whole value crosses. `children` identifies
the conversions selected for its fields or other parts. Both affect reuse. If
two functions accept `Stamp` but one applies a different conversion to `secs`,
their struct conversions must differ too. Omitting the child identities from the
cache key would incorrectly reuse the first function's field behavior.

For example, two owned `Stamp` inputs with the same two-integer JNI representation and field construction can share a node. An object-input override records a different representation; a rule on one of their `secs` fields changes that child, and therefore the struct's conversion; a return conversion changes direction. `Stamp`, `&Stamp` and `Option<&Stamp>` remain distinct.

Model membership follows the [snapshot contract](02-flat.md#private-storage-and-model-consistency). Flat publishes immutable source data after helper registration; its views preserve that snapshot through field and parameter navigation. Registry operations that accept a view check it against their own model before planning, rather than relying on the caller to check first. Flat owns these checks and private view construction. A valid view from another snapshot is rejected even when its key text matches. The registry accepts no detached reading or independently supplied model/type pair as a substitute for a view.

Keys are local to one `Flat` model; Flat owns normalization. `NodeId`
identifies a retained plan, and registry-issued node references must be
validated within their generation context. Function sites retain separate
overrides and diagnostic paths.

Two values share a node exactly when their crossings, representations and
children are equal, and the registry can check every part of that itself:
representations are compared by id, and the binding gives two equal
representations one id — comparing their carriers by id, and operations and
metadata by value. Nothing rests
on a promise from the target that two opaque keys mean the same thing.
Representations are recorded before planning, so equal settings are equal
data. A naming closure runs when the frontend records a carrier or a rule,
and what lands in the binding is the name it produced.

Outcomes are keyed by source and configuration identities that do not vary
between runs over unchanged inputs, so two builds of the same crate decide the
same things.

## Elements at this stage

- [Function taking an owned struct][fn_requests] · [C][fn_requests_c] · [Kotlin/JNI][fn_requests_jni]
- [Struct with scalar fields][struct_requests] · [C][struct_requests_c] · [Kotlin/JNI][struct_requests_jni]
- [Type alias declaring an opaque handle][typedef_requests] · [C][typedef_requests_c] · [Kotlin/JNI][typedef_requests_jni]
- [Function taking a callback][fn_callback_requests] · [C][fn_callback_requests_c] · [Kotlin/JNI][fn_callback_requests_jni]

[fn_requests]: ../examples/fn/03-requests.md
[fn_requests_c]: ../examples/fn/03-requests.c.md
[fn_requests_jni]: ../examples/fn/03-requests.jni.md
[struct_requests]: ../examples/struct/03-requests.md
[struct_requests_c]: ../examples/struct/03-requests.c.md
[struct_requests_jni]: ../examples/struct/03-requests.jni.md
[typedef_requests]: ../examples/typedef/03-requests.md
[typedef_requests_c]: ../examples/typedef/03-requests.c.md
[typedef_requests_jni]: ../examples/typedef/03-requests.jni.md
[fn_callback_requests]: ../examples/fn_callback/03-requests.md
[fn_callback_requests_c]: ../examples/fn_callback/03-requests.c.md
[fn_callback_requests_jni]: ../examples/fn_callback/03-requests.jni.md
