<!-- spec: {"kind": "implementation"} -->

[Project contents](README.md) · Previous: [Emit bindings](stages/08-emit.md)

# Implementation and acceptance

The stage chapters explain the architecture, including contracts for features
that are not implemented. This page separates that design from the evidence we
have today and explains what an implementation must demonstrate next.

The current increment generates scalar and owned-struct input bindings through
the real C and JNI frontends. The C entry point is executed in tests; generated
JNI Rust is compiled and Kotlin text is checked, but that fixture does not yet
execute a JVM call. Owned source-model views, optional [conversions](stages/04-select.md#select-conversion-relations), resource
contracts and skip-based test selection remain future work. The sections
below explain the planned sequence, the completed increment and its limits.

## Flat implementation sequence and acceptance

Implement the [source-model API](stages/02-flat.md) incrementally in
`prebindgen-flat`. A V2 module can coexist with the current API during migration;
`Flat` in these documents denotes that V2 model. This does not require a new
standalone crate or an immediate breaking rewrite of V1 consumers.

1. Add builder-to-snapshot ownership, helper registration checks and direct
   function lookup/enumeration. Preserve unsupported items and locations.
2. Add parameter/result `TypeView`s, exact struct navigation and field views.
   Implement the independent inspection example using existing captured inputs.
3. Add snapshot checks and checked type composition. Have V2 registry requests
   retain views, and derive cache keys privately from those views.
4. Extend constants, enums and other type forms using the same conventions when
   their consumers need them. Existing inputs remain retained and reportable
   throughout; unsupported structural access is not a reason to lose an item.

Required validation for implementation:

- A standalone Flat consumer navigates function → parameter → struct → field
  without depending on the registry.
- Lookup and enumeration agree; views remain usable after the original `Flat`
  value is dropped, and cloned views refer to the same snapshot.
- A view from another snapshot is rejected even when both snapshots contain a
  type with the same name and key.
- External callers cannot forge views or mutate their retained records.
- Local helpers are validated before publication, including duplicate names and
  source-module qualification; existing opaque helper types remain representable.
- Reference, optional and fallible navigation preserves the enclosing types and exact child
  types. Modeled lifetime arguments survive field navigation and emission.
- Opaque items, unsupported items, guards and locations survive migration.
- Derived type views preserve model association without changing source-item
  enumeration, and type/key consistency holds by construction.
- Flat tests validate source inspection and emission. The registry project's
  C/JNI tests separately validate [conversion](stages/04-select.md#select-conversion-relations) behavior and ownership.

## Integration and first implementation steps

### Switching existing examples

Keep v1 and v2 as parallel engines behind the existing frontend. Independent v2 registry and C/JNI implementation crates can share binding-configuration data modules and the source model. They must not depend on v1 conversion plans, recursive generators or emitters. Engine selection happens before v1 generation starts.

Engine selection and separate output paths began in
[#721](https://github.com/milyin/prebindgen/pull/721) and
[#722](https://github.com/milyin/prebindgen/pull/722). V2 now also plans and emits
the scalar/struct subset described under [the built increment](#the-first-increment-as-built).
Requests outside that subset remain visible as reported skips. Those skips are
scaffolding for the transition, not the specified answer to an unsupported
request: closing the coverage gap includes turning them back into build
failures, and until that lands no consumer arrangement should rely on a skipped
declaration.

The switching contract from [#719](https://github.com/milyin/prebindgen/issues/719) is:

- The frontend crate's Cargo feature `v2` — `prebindgen-c` or `prebindgen-jni` — makes the optional engine available. It does not select it by itself.
- `.build()` reads `PREBINDGEN_PIPELINE=v1|v2`; an unset variable selects v1, including under `--all-features`.
- `build_with(Pipeline::...)` selects an engine explicitly and bypasses the environment.
- Selecting v2 without its feature produces a clear error.
- Example scripts forward the selection and enable the feature as needed. Build scripts track environment changes so Cargo regenerates the selected output.

For example:

```sh
PREBINDGEN_PIPELINE=v2 cargo build -p covertest-kotlin --features v2
```

Both engines receive the complete current configuration through their respective paths. V2 generates its supported subset without falling back to v1 for individual items. Output paths and publication must support v1 → v2 → v1 without manual cleanup or stale generated declarations.

Existing Kotlin tests can directly refer to classes absent from v2 output. Select complete supported test sections before Kotlin compilation; runtime guards cannot hide missing symbols from the compiler. C tests need equivalent selection. The selection must still require meaningful supported tests to execute, so skipping everything cannot pass a milestone.

### First executable increment

The initial implementation should demonstrate the architecture with both existing C and Kotlin examples:

1. Construct the declaration list from all recorded frontend choices; implement identities, diagnostic causes and request accounting. Preserve unsupported configuration entries and settings from the start.
2. Implement one scalar function through target descriptors, registry conversion/function plans, frozen output and the normal output path: common Rust emission followed by C header generation or Kotlin emission. Execute it through both language boundaries.
3. Add named-field structs with registry-owned field traversal, construction and decomposition. Demonstrate a C aggregate and a JNI [representation](stages/05-represent.md#represent-and-compose-values) using the same source [relation](stages/04-select.md#what-a-relation-is) algorithm.
4. Add plain optional representations and the temporary/borrow operations required by selected existing examples. Test present/absent behavior and temporary lifetime requirements.
5. Verify dependency-based skipping, existing test-section selection, and repeated switching between engines. Each preceding executable increment also produces complete generated outputs and accounts for what it left out.

Steps 2 and 3 are exactly the first two element paths specified in this
document: the [function path][fn] is the scalar function and its owned struct
argument, and the [struct path][struct] is the named-field struct behind it.
The [handle path][typedef] is the first piece of the resource-bearing work the
list below defers, taken in its narrowest form: an owned handle, out and back,
with its release.

Do not implement every proposed enum variant before the scalar case runs. Constructor/projector conversions, `Result`, resource-bearing handles, sequences and callbacks can be added incrementally through the same descriptions and registry algorithms. Full inputs remain accepted throughout that work.

### Cases that expose architectural mistakes

The cases below come from the repository's coverage tests and performance
examples. In `perftest-flat` and `perftest-kotlin`, `large_flat_input_sum` and
`large_object_input_sum` both take a Rust struct by borrow. Their JNI
configurations differ: one flattens the fields into separate
[wrapper](stages/06-boundary.md#assemble-the-wrapper-boundary) arguments, while the
other passes a whole JVM object. That combination tests both representation
choice and the temporary lifetime needed for a borrow.

| Case | Expected behavior |
| --- | --- |
| `Stamp` under C aggregate and JNI JVM-object input, with separate JNI arguments as a future extension | Source field discovery and Rust construction stay in the registry; target operations differ. An unimplemented requested representation is reported. |
| Two functions using the same `Stamp` representation | Share the conversion while retaining different parameter names and diagnostic paths. |
| A whole opaque representation of a type with unsupported private fields | Do not traverse the unused fields. |
| Accessor/value-form helper returning a compound value | Call it once, keep its exact result type, and let the registry process its selected children. |
| Existing `large_flat_input_sum(&ObjectBoundary64)` | Support requires an owned temporary and call-scoped borrow, beyond owned struct conversion. Preserve the signature; skip with a borrow reason until implemented. |
| The existing JVM-object-input sibling of that function | Its independently selected representation may have a different support [outcome](stages/07-retain.md#retain-supported-output). |
| Source `Result` with configured handler/builder | Preserve both branches and existing delivery conventions; skip if a required handler or destination is unimplemented. |
| Nested optional values | Preserve distinct states and never decode inactive payloads. |
| Unsupported required field or promised interface member | Propagate to the complete dependent public contract, while retaining unrelated output. |

A capability involving JNI is complete only when the existing Kotlin covertest exercises both its generated wrapper boundary and Kotlin API. Unit tests for planning are useful, but they do not establish runtime ownership, JNI, or foreign-interface correctness.

## The first increment, as built

The planning half of steps 2 and 3 above is implemented in
`prebindgen-registry-v2`, over the four element paths this document specifies:
all are planned, assembled and emitted, for both targets. What step 2 also asks
for — executing the binding through both language boundaries — is met on the C
side and not on the JNI side, where the evidence is that the generated Rust
compiles against the real `jni` crate and that the Kotlin says what this document
says. A JVM that loads the library and calls the method, which is what
[the JNI completion rule](#cases-that-expose-architectural-mistakes) requires,
comes with the covertest work rather than here. What follows records what
building this settled, so the chapters and the engine describe the same thing.

To find the implementation, start with `generate(flat, &target, binding, source_module)`
in `prebindgen-registry-v2`. Its responsibilities are divided across files:

- `binding.rs` defines what a frontend states before planning:
  [carriers](stages/05-represent.md#describing-target-values-and-operations),
  representations, rules, outputs and function forms.
- `target.rs` defines the writers an adapter implements and the feeds they are
  handed, with the vocabulary both sides share.
- `plan.rs` looks conversions up and combines them, caches reusable plans, assembles
  [wrappers](stages/06-boundary.md#assemble-the-wrapper-boundary) and checks public dependencies.
- `body.rs` defines the instructions stored in those plans; `emit.rs` writes
  the corresponding Rust code.
- `run.rs` holds the completed `Generation`, including what it left out;
  `decl.rs` and `outcome.rs` describe requested declarations and their
  outcomes.

The two targets live in the language frontends, under their `v2` feature:
`prebindgen-c/src/v2/` and `prebindgen-jni/src/v2/`. Each is two things. A
`Target` of two writers — `write_operation` and `write_carrier` — that decides
nothing, walks no type and names no temporary; and a reader of the frontend's
own declaration storage that turns it into the binding — carriers,
representations, rules and outputs, in a stable order — with the frontend's
manglers already applied to every name. The JNI frontend also carries its Kotlin
writer, over the outputs the generation retained. A frontend's
`build()` runs this route when `PREBINDGEN_PIPELINE=v2` selects it — or
`build_with(Pipeline::V2)` states it — and nothing of v1 runs on that route.
The user's `build.rs` is the same under either engine: the one thing v2 adds to
it is that a declaration the engine cannot lower is a skip it can print rather
than a build failure.

`examples/v2check` provides evidence for this increment. It includes
[the specification's fixture](source.md), plus additional test cases,
parses them directly and passes them through the frontends' `.items(...)` API.
It therefore tests generation without exercising proc-macro capture. It selects
V2 explicitly and compiles both generated Rust files. Its tests read the expected
wrappers out of [the emit pages][fn_emit] themselves, item by item, so a chapter
and the engine cannot drift apart quietly, and read the Kotlin in order and
without duplicates. One test calls the generated C entry point, on a function
whose result changes if the two fields arrive in the wrong order — addition
would not notice. Another opens a handle, closes it and reads the total back,
releases a second one unread, and releases a null. Two more pass a C closure
struct in: one counts the calls, their order and the `drop`, and one takes back
the handle and reads the enum a callback receives. The existing examples are further evidence: unchanged, built
with `PREBINDGEN_PIPELINE=v2`, every one of their declarations reaches the engine
and comes back with an outcome — the data classes and functions within this
increment emitted, everything else skipped under the capability it waits for.

The engine's unit tests use a small test adapter to isolate the planner's rules.
They check conversion sharing and rules at positions, temporary-name
collisions, propagation from a value no rule covers to its struct and callers,
and the requirements between outputs. They check the rules themselves: two for
one value, or one at a position the output does not have, fail the build; a
member or parameter of a wire type its holder does not accept is refused where
it sits; an unused type rule is listed; the binding prints as what planning
reads. They also check that a function is skipped when an operation has no
error route or needs a runtime context the form does not supply; that a handle is carried both ways and released under its type's
identity, while a null one arriving where it is consumed needs a route; and
that a handle nobody can release skips the type and what takes it. For
callbacks they check the closure the registry builds, an argument handed out
inside each call, a call's failure taking the callback's own route, the
refusals — an unrouted failure inside a call, a call needing a runtime
context, an argument that cannot leave Rust or that the carrier does not hold,
a callable leaving Rust, a callback representation on another type, a
callback no output declares — and a rule at `param f.arg 0`. Contradictory configuration must instead produce a generation error.
These tests establish planner behavior; C/JNI runtime tests are still needed to
establish the behavior of the resulting foreign interface.

### What the increment settles

1. **The instruction set.** Current conversion bodies are `NodeBody` values;
   function bodies are stored in `FunctionPlan::instrs`. Both use four kinds
   of instruction over value identities: apply a registered operation, construct
   a source struct, call the source function, and build a callback's closure,
   whose own instructions use the same identities. These implement the body roles
   described with the rest of the
   [conversion plans](stages/05-represent.md#the-conversion-plans-the-registry-builds).
   A conversion's body is a template whose
   carrier is its
   input; using it inlines it under the caller's identities. Temporary names are allocated by the writer from
   definition order, never by an adapter.
2. **Standard and target operations.** An operation's implementation is
   `Implementation<Op>`: either a `Standard` operation the registry writes —
   identity, member read, the handle and enum operations — or the target's own
   `Op`, which its writer writes when the registry feeds it. C writes no
   operation at all, which its target states by giving `Op` no values.
3. **A rule applies to one value.** The rule at a value's position, else its
   type's, gives the representation of the value currently being planned. It
   says nothing of that value's children, which are looked up again at their
   own positions. This gives each override one place to be expressed and keeps
   child choices visible in the conversion cache key.

   No writer is told the position it writes for. A conversion is reused
   wherever one of the same identity is needed —
   [crossing](stages/03-requests.md#finding-an-existing-conversion-plan),
   representation, children — so a writer that wrote differently for two
   positions would have its second text silently bypassed by the first one's
   [node](stages/05-represent.md#represent-and-compose-values). Varying by
   position is what a rule at a position is for: it names a different
   representation, which is in the identity.
4. **How a binding declares its types and generated units.** A carrier is a
   `WireType` — the Rust type it is spelled as, which of the target's wire
   classes it is, which classes its members may be, and metadata only the
   writers read — declared once and named by id, as a representation is. A
   generated unit is an `Artifact`: a name and the Rust it contributes, returned
   by a writer beside the text that needs it. The registry keeps one
   [artifact](stages/05-represent.md#individual-target-operations) per name and
   publishes only those a retained output needs.

   These declarations are checked where they meet the plan, which is the
   registry and nowhere else. What is checked today, and nothing beyond it: two
   rules for one value, a rule at a position its output does not have, a form
   naming the wrong number of inputs, a symbol that is not an identifier, and a
   form restating the linkage all fail the build; a member, parameter or return
   of a wire class its holder does not accept is refused where it sits; an
   operation needing a context the form does not supply is refused; and a
   failure route must report the error type the operation raises. Wrapper
   parameters are typed from the carriers their conversions resolved to, so a
   parameter and the conversion reading it cannot disagree.
5. **Feature-assertion guards.** Reading captured source injects a `const _`
   assertion comparing the source crate's features against the set the capture
   was filtered by. Flat retains it as a guard, and the V2 writer emits every
   guard the model holds, ahead of the supporting items and
   [wrappers](stages/06-boundary.md#assemble-the-wrapper-boundary) it planned.
   Guards belong to no declaration, so retention does not decide their fate: a
   run that emits nothing still carries them, which is the case where losing the
   check would matter most. `v2check` feeds parsed items directly and has no
   guard to carry, so an engine test supplies one instead, and building an
   existing example with `PREBINDGEN_PIPELINE=v2` shows it in the generated file.
6. **Runtime contexts.** `ScopeRequirement`'s concrete form is a named operand
   role: an operation declares `Context("jni.env")` where it needs the
   environment, the function form names the wrapper parameter that supplies it, and
   the registry binds the two at assembly. A conversion asking for a context its
   boundary does not supply is a reported skip, not a fragment reaching for a
   variable its caller happens to have.
7. **A consumed handle needs an exchange, not a lock.** Every use of a handle
   on the specified path consumes it, so the Kotlin class hands its address
   out through one `AtomicLong.getAndSet`: of two racing consumers exactly one
   gets the address, the other gets the zero the Rust side refuses, and the
   exchange carries its own happens-before edge and cannot tear. V1 locks
   instead — a sorted pass over every handle a wrapper touches — because v1
   *borrows*, holding an address across the call into Rust, and a lock is what
   keeps it valid for that span. Whoever specifies a borrowed handle needs
   v1's shape; porting it for a consume-only handle would be answering a
   question this path does not ask.
8. **Handles without a resource contract.** An opaque value crosses as an
   address through three more standard operations — `IntoRaw`, `FromRaw`,
   `Release` — which are the registry's because they spell a source type. The
   frontend states the carrier the address is cast to and, on the
   representation, that a release exists; naming one is what tells the
   registry the type is a handle, whether the item behind it is an alias or a
   struct whose fields the target never reads. The registry then requires the
   out-of-Rust direction too and plans the release as a wrapper under the
   type's own identity, from the release form the type output names. A null
   address taken back is a `Binding` failure carrying a `String`, routed like
   any other; a null address released is a no-op. What keeps this sound
   without `ResourceContract` is the shape of the three wrappers, stated in
   [the handle's representation cell][typedef_represent]: nothing acquires a
   resource that a later failing operation could leak.

9. **A fieldless enum is its numbers.** An `enum Op { Add, Mul = 7 }` has no
   parts, so it crosses through the atomic relation like a scalar — but the
   value that crosses is the number Rust assigns each alternative, which the
   model already computes. Two more standard operations spell the conversion,
   `EnumOut` and `EnumIn`, for the reason the handle operations are standard:
   they name the source type, and an adapter cannot spell a source path. What
   the number is carried *as* is the frontend's to state. C declares an enum of the same
   values, and a value crosses as one of them either way, so the header names
   the enum wherever the source does. Out of Rust that is an exhaustive `match`
   that cannot fail. Into Rust the enum arrives as `MaybeUninit` and is matched
   as the C `int` it holds, because C lets an enum variable hold any `int` and a
   Rust enum holding a number none of its values has is undefined behaviour.
   `EnumIn` takes the integer type to read the carrier as for this.
   `MaybeUninit` could also hold storage never initialized, which no match can
   check, so a wrapper that reads a carrier's bits is `unsafe` and documents
   that its caller owes initialized storage. JNI carries a `jint` and a Kotlin
   `enum class`. In both, the direction into Rust can meet a number no value
   names and fails as a `Binding` error. An alternative carrying a field makes a
   sum, which the model calls a variant and which has no lowering.

   Some shapes are refused rather than mirrored, by one check both targets
   share. A number the model could not evaluate — a `const`, arithmetic — is not
   a number to mirror. A value written under a `#[cfg]` breaks the numbering,
   because the model counts every value as present, and would put an entry in
   the mirror for a variant the source crate compiled out. An enum with no
   values has nothing to mirror at all. A number outside `i32` is refused,
   because both carriers are 32 bits: a C `int` and JNI's `Int`. And
   `#[non_exhaustive]` is refused wherever it sits: on the enum, which another
   crate cannot match without an arm for a value it does not know — and going
   out of Rust there is nothing for that arm to produce — or on a value, which
   another crate cannot name at all, a unit value included, whose constructor is
   private outside the crate that declared it.

   What is *not* refused is a fieldless value written `Add()` or `Mul {}`:
   those are not unit variants, so every mention of them carries its
   delimiters, which the model's own speller supplies from the shape it
   captured.

   `examples/v2check` parses a crate it also links, so rustc compiles the
   generated matches across the boundary. That is what the accepted shapes need,
   and what `#[non_exhaustive]` is about in either position, so those are the
   shapes it declares: an enum with explicit and implicit numbers, one whose
   values are constructors, one only ever passed in, and one refusal for each
   place the attribute can sit. A test runs cbindgen over the C binding, because
   the header declares an enum only if an exported signature names it, and the
   input-only one is named by nothing else. The remaining refusals produce
   nothing to compile, so each is checked by a test that asks a frontend for
   such an enum and reads the capability back.

10. **A callback is a closure the registry builds.** An `impl Fn(..)`
    parameter crosses as a `Callback` representation: a carrier the callable
    arrives in, a `capture` applied to it once, an `invoke` applied on every
    call, and the routes a failure inside a call takes. The relation is the
    callback's arguments, planned out of Rust through the rules for their
    types, and the registry composes the closure from their templates and the
    two operations — so neither target walks an argument or writes a closure.
    A failure inside a call cannot reach the wrapper, which may have returned,
    so the callback states its own routes and the wrapper routes only
    `capture`'s. C lets the caller's closure struct be the capture itself; JNI
    captures the JVM, a global reference and the `run` method, and attaches
    each calling thread. A callback is a `callback:` output, required by the
    functions taking it as a type is. C declares one with `callback!`; JNI
    states one for every signature its exported functions take, as v1's
    implicit Kotlin interfaces are. [The callback path][fn_callback] shows
    both. The JNI side was run once in a JVM by hand — values in order, a
    handle and an enum adapted, a call from another thread, a lambda that
    throws — and is otherwise evidenced as the rest of JNI is.

### What it does not settle

The contracts designed for these are on [the extensions page](extensions.md);
this list says what the increment left open and why.

- **The binding's vocabulary** is real for these paths — representations,
  codecs, operations, function forms, acceptance, and the writers' feeds — and
  untested by a third target or a deferred capability.
- **The representations.** A `Product` reads one part per part, in the
  into-Rust direction only. Struct output needs a construction operation that
  is not implemented, and both targets reach the registry's
  `unsupported.struct.out_of_rust` skip. Containers, choices, niches and
  flattening at the boundary describe the wider design.
- **Fallible construction meeting the boundary** is untouched, because
  constructor and projector relations are not implemented: the only relations are
  the struct's fields and the atomic conversion.
- **Optional values** and everything that goes with them — `Layout::Slots`,
  `SlotRole`, `GuardId`, `AbsenceEncoding` — are not implemented.
- **Validity and resource contracts** are absent from `Operation`. The one
  resource-bearing operation set — the owned handle — is safe without them by
  construction on the Rust side, as
  [the extensions page](extensions.md#runtime-resources) argues, and so is a
  callback's capture, which the closure owns; a borrowed handle or an optional
  handle is what has to add them, and cannot be written without them. A handle
  a callback's call hands out before the call fails is not taken back.
- **A wrapper's form** is the writer's, except what `FunctionForm` lets a
  binding state: the convention, the symbol, the parameters the convention
  adds, the names of the inputs' parameters, attributes beyond `#[no_mangle]`,
  and `unsafe`. A target reached other than by an
  exported symbol — a registration table, an attribute macro — has no way to
  say so yet, and no target has asked.
- **Delivery** is a wrapper return or nothing. Out-parameters, `Result` branches
  and a result handed to a callback the caller passes in are deliveries the
  increment does not have.
- **A callback** takes arguments and returns nothing, and is moved into the
  source function. A callback returning a value, one taken by reference, and
  a Rust callable handed out to foreign code are not built; the last is
  refused as `unsupported.callback.out_of_rust`. Nothing inside a call has a
  runtime context, so an argument whose conversion needs one — a JNI `String`,
  when strings are built — refuses the callback.
- **A condition reaches the Rust side only.** V2 carries a `#[cfg]` the capture
  reader could not answer onto everything it generates in Rust for the item that
  carries it — the wrapper, the target's own declaration, and, for a field,
  the mirrored member with the read and the initializer that serve it. No writer
  gates the declaration the *other* language compiles against: the C prototype
  and the Kotlin `external fun` are emitted whatever the condition says, so a
  JNI binding can build and fail on the one call, and a C header can declare a
  member the library's struct does not have — silently, unless `cbindgen` is
  given a `[defines]` entry for the condition. What is done about it instead is
  a `cargo:warning` from the C frontend and the condition named in the Kotlin
  function's documentation.
  [The capture chapter](stages/01-source.md#the-binding-crate-decides) prices
  that.
- **The source model's views.** Stage 2's `FunctionView`/`TypeView` are not
  built: the engine plans over today's borrowed `Flat` API and its frozen result
  owns the model, so a plan cannot outlive it and a view from another snapshot
  cannot be offered. That is enough for one model per run, and it is exactly what
  cross-snapshot planning will break.
- **Node retention** keeps every conversion the run planned rather than only
  those a retained output reaches. Nodes are referenced by nothing after
  inlining, so this costs memory and no correctness; pruning them needs the
  reachability the retention loop does not yet track.
- **The `Unselected` outcome** is not implemented: a captured item nobody
  declared is not accounted for at all, since the engine hears only what the
  binding asked for.
- **A sum** — an enum whose alternatives carry values — has no representation.
  The fieldless enum above is a named set of integers and crosses as one; a
  sum needs a tag and one group of members per alternative, which
  [choices](extensions.md#choices) describe and nothing implements. The engine
  refuses one with `unsupported.type.variant`.

## Acceptance and feasibility evidence

Acceptance criteria:

- [x] The design's boundaries are exercised by scalar and struct bindings — in `examples/v2check`, over the specification's own source crate, and in the existing C/JNI examples built with `PREBINDGEN_PIPELINE=v2`, whose declarations are unchanged.
- [x] Users configure the existing language frontends; frontend internals construct the binding for the registry — carriers, representations, rules and outputs — with every name already mangled. A declarator a frontend does not lower is recorded as unsupported, naming it; the registry reads the binding's structure and never the target's own metadata.
- [x] The registry owns recursive conversion, source calls, dependency resolution, control flow and Rust wrapper assembly.
- [ ] The [source model](stages/02-flat.md) supplies checked source views; the registry validates snapshot association.
- [x] Targets are writers only: every representation, runtime-operation and delivery choice is stated in the binding before planning, and neither target walks a type, decides a plan or names a temporary.
- [x] Complete unsupported inputs produce actionable per-declaration outcomes; malformed configuration and generator defects fail generation.
- [x] One immutable generation result supplies Rust output, optional foreign-writer output and what the run left out; C headers are derived from the retained Rust output by `cbindgen`.
- [ ] Test selection from what a run left out.
- [x] Emitted output preserves logical behavior and declared interfaces without a byte-identity requirement — checked item by item against the emit pages, compiled by rustc, and executed for C.
- [ ] New nested combinations reuse the registry's composition algorithm instead of requiring a new per-language wrapper implementation.
- [ ] Remaining unsupported capabilities and any API refinements discovered during implementation are documented.

Earlier feasibility work inspected registry relations and composition and Flat emission at [e046546](https://github.com/milyin/prebindgen/tree/e04654679e7aa0e7c7d4b9bb4a1268f9943926ec). Type-key inspection at [a429662](https://github.com/milyin/prebindgen/tree/a429662bb450408f401ad8f52ff753c5f5a179d4/prebindgen-flat/src/flat) confirmed `TypeRef::key()` and its equality/hash contract. `cargo test -p prebindgen-flat --lib`: 92 passed. These checks covered existing Flat behavior, not the proposed V2 contracts.

Future resource, recursive and runtime capabilities require implementations and tests; until then, affected requests remain unsupported. Background: [#689](https://github.com/milyin/prebindgen/issues/689) / [#701](https://github.com/milyin/prebindgen/issues/701); earlier plans: [#713](https://github.com/milyin/prebindgen/issues/713) / [#717](https://github.com/milyin/prebindgen/issues/717).

[fn]: examples/fn/README.md
[typedef]: examples/typedef/README.md
[typedef_represent]: examples/typedef/05-represent.md
[fn_emit]: examples/fn/08-emit.md
[struct]: examples/struct/README.md
[fn_callback]: examples/fn_callback/README.md
