# Rust writing after planning

Here **planning** means every semantic decision about what will be generated and
how values cross the boundary; it excludes turning those decisions into Rust
syntax. This document proposes an implementation change. It does not redefine
the [binding model](model.md): Flat, crossings, recipes, fragments and site
plans keep their existing meanings. The proposal makes the implementation obey
the [Flat planning boundary](model.md#flat-is-the-planning-boundary): captured
Rust type syntax becomes available only after every generation decision is
complete and validated.

The target is semantic preservation, not textual stability. The public C
application binary interface (ABI), Java Native Interface (JNI) contract and
generated Kotlin API must remain compatible, and existing ownership, cleanup,
error, panic and concurrency behavior must keep working. Generated source and
private helper structure may change. In particular, the migration may reorder,
combine, inline or remove private converter functions when that improves the
result without changing the public contract.

Any change to a public symbol, calling convention, wire layout, Kotlin-visible
signature or documented behavior is separate binding work and must be proposed
and reviewed as such. A code-generation optimization is in scope when only its
private implementation, code size or performance changes.

## Terms used here

The model terms below are defined fully in [the model vocabulary](model.md#the-vocabulary).
Their short descriptions here say how this proposal uses them.

| Term | Meaning in this document |
|---|---|
| **Binding** | The generated foreign-language interface for one annotated Rust source. Its build is described in [where the crates sit](model.md#where-the-crates-sit). |
| **Adapter** | The target-specific generator: currently Cbindgen for C or JniGen for JNI and Kotlin. It decides how model values cross its target boundary. |
| **Flat** | The complete language-independent model parsed from captured source records. Planning asks Flat for facts instead of inspecting Rust syntax. |
| **`TypeRef`** | Flat's opaque description of one Rust type. Its structural reading is available during planning; its captured Rust spelling is not. See [the crossing](model.md#the-crossing). |
| **Declaration** | A build-script statement selecting a function, constant or way for a type to cross. Undeclared source items generate no binding output. |
| **Direction** | The orientation of a recursive shape walk: toward Rust (`Construct`) or toward the boundary (`Deconstruct`). See [parts and wire use different verbs](model.md#parts-and-wire-use-different-verbs). |
| **Crossing** | One `TypeRef` and one direction, used as a query into the recipe table. It does not itself say whether the next step is parts or wire. |
| **Recipe** | One named row owned by the recipe table and containing a shape. `RecipeName` is reusable policy vocabulary local to a crossing; `RecipeKey = (CrossingKey, RecipeName)` identifies the row position throughout the table, including a missing position named by a diagnostic. |
| **Shape** | The value–parts step declared by a recipe row. A shape with parts constructs or deconstructs; `Atomic` ends the shape walk without naming wire types. |
| **Site** | One position where a value crosses, such as a function parameter, return or callback argument. |
| **Fragment** | The adapter-specific result of applying one selected recipe row to one spelled crossing and composing its child fragments. It is the first layer that records wire layout and terminal decoding or encoding. One normalized table row may produce different fragments for `T`, `&T` and `Box<T>`. |
| **Site plan** | The adapter-specific answer for one position in the generated interface, such as a function parameter or return. It selects a fragment for that position. |
| **Wire value** | A Rust value whose type has an exact representation in the target calling convention, such as JNI's 64-bit integer slot `jlong` or C's untyped raw pointer `*mut c_void`. |
| **Converter** | Internal Rust planned from a fragment and emitted later. It composes value–parts operations, representation bridges and child converters; at an `Atomic` terminal it runs the adapter's decode or encode operation. |
| **Wrapper** | The exported generated Rust entry point for a declared function. It converts parameters, calls the source function and converts its result. See [what the foreign side calls](model.md#what-the-foreign-side-ends-up-calling). |
| **Callback** | A callable supplied by the foreign side and invoked later by Rust. Its argument crossings run opposite to the callback crossing. |

This proposal also uses implementation terms that are not part of the binding
model:

- Planning ends only when declarations, recipes, dependencies, fragments,
  sites, generated symbols, ownership and cleanup behavior are validated and
  immutable.
- A **generation plan** is that complete immutable result. Its Rust type is
  adapter-specific around a shared recipe-composition core; C and JNI do not
  need a shared language for their exported wrappers.
- An **artifact plan** completely describes one output item, such as a converter
  function, exported wrapper, helper type or target-language declaration.
- A **fragment identity** is the selected complete `RecipeKey` applied to one
  spelled, directed crossing. The recipe supplies the shape; using only the
  shape variant would incorrectly merge rows that use different constructors
  or accessors.
- An **intermediate type** is the adapter-selected, internal Rust type assigned
  to one fragment identity. It carries that fragment's converted value while
  composed recipes are assembled or taken apart. It is not a source type and
  need not itself be legal in an exported C or JNI signature.
- **Intermediate parts** are the child intermediate values corresponding to a
  shape's source parts. A product has a tuple of intermediate parts, an
  optional has `Option<I>`, and a choice has one selected arm's parts.
- A **representation bridge** is the formal, shape-specific way to pack
  intermediate parts into an intermediate value and unpack that value back
  into its parts. It describes operations; it is not generated Rust syntax.
- An **ABI layout** is the ordered set of wire values occupied by an
  intermediate value at one boundary site, together with the operations that
  flatten into those values or assemble from them. One intermediate value may
  occupy no, one or several wire values.
- A **niche** is a bit pattern available in an intermediate or wire
  representation but excluded from the child value's valid domain. Optional
  and choice bridges may consume niches for discriminants and expose any
  unused niches to enclosing bridges.
- A **converter plan** is the registry-built operation graph relating one
  source value to its intermediate value. It chains model-defined
  construct/deconstruct operations, child converter plans and representation
  bridges; it contains no rendered source type.
- A **callback bridge** is the adapter-specific plan that turns a foreign
  callable into a Rust callable and delivers each later Rust invocation across
  the foreign boundary. It reuses ordinary argument converter plans but also
  owns callable state, thread entry, invocation, error and cleanup policy.
- **Final emission** turns artifact plans into Rust abstract syntax tree (AST)
- A **role** is the implementation's classification of a site as a parameter,
  receiver, return, error, constant, callback argument or recipe part. It lets
  an adapter state the validity that the target accepts at that position.
- A **yield** is the source-value contract of a fragment: normalized type
  identity, owned or borrowed mode, and validity. The registry compares yields
  with the requirements on recursive part edges without inspecting adapter
  representation.
  items and assembles the output file. It makes no generation decisions.
- **`Emit`** is the registry-owned capability that lets code render the captured
  source syntax behind `TypeRef`. It implements Flat's `RustEmitter` protocol.
- **`Compile`** is the current adapter trait for constructing fragments and site
  plans; **`recipe::Compiler`** drives that trait over recipes and sites.
- **`Answer`** is the registry-facing result for one crossing. It records which
  other crossings the adapter's fragment depends on, not the generated Rust.
- A **source feature guard** is an anonymous captured constant that makes a
  generated file fail to compile when generator and source features disagree.
- **`Prebindgen`** is the current shared trait containing item-kind Rust
  emission callbacks and the final whole-item rewrite.
- The **adapter renderer** translates one adapter-specific artifact plan into
  Rust AST items. It is allowed to receive `Emit` because it runs only during
  final emission.
- The **shared Rust writer** drives final emission for every adapter: it owns
  ordering, `Emit`, file assembly, formatting, error handling and destination
  writing.

## Baseline: rendering starts during resolution

[How the file is built today](model.md#how-the-file-is-built-today) describes
the two visible rounds: resolve conversions, then assemble the Rust file. The
implementation mixes Rust rendering into the first round.

The current call path is:

1. [`RegistryBuilder::convert_with`](../prebindgen-registry/src/registry/declare.rs)
   derives the crossing order, constructs an `Emit`, and passes it into the
   adapter's conversion closure.
2. [`recipe::Compiler`](../prebindgen-registry/src/recipe/compile.rs) constructs
   another `Emit` and exposes it through every `Compile` hook's context. C and
   JNI compile hooks use it to spell source-side `TypeRef`s while fragments and
   sites are still being planned.
3. A compiled fragment holds generated syntax, including complete
   `syn::ItemFn` AST nodes where its recipe emits converter functions. Both
   adapters keep the fragment memo, keyed by spelled `TypeKey` plus complete
   `RecipeKey`, in `Rc<RefCell<Compiled<_>>>`, a shared interior-mutable map.
   They repeatedly clone that map into
   `Compiler::resume`, run part of compilation, and store the resulting map
   again so later conversion generation can observe earlier generated code.
4. The [C builder](../prebindgen-c/src/trait_impl.rs) and
   [JNI builder](../prebindgen-jni/src/jni/trait_impl.rs) copy generated
   functions out of the fragment memo into a second `compiled_fns` collection.
   JNI includes a main converter and any pre-stage converter functions; a
   composed-only fragment contributes none.
5. Each adapter passes `compiled_fns`, itself and the resolved registry to the
   [shared writer](../prebindgen-registry/src/write.rs). The writer constructs a
   further `Emit`, appends the already-rendered converters, invokes item-kind
   callbacks, parses each callback's `TokenStream` (an unparsed Rust token
   sequence) back into `syn::Item` AST nodes, appends source feature guards and
   runs `post_process_item` over the complete AST before formatting and writing
   it.

`post_process_item` is the current cross-cutting rewrite that qualifies bare
source names with their module paths. It can inspect and rewrite every generated
item after the earlier rendering steps have finished.

### Finding

The spelling capability is private but not late. Ordinary adapter code cannot
construct `Emit`, but conversion planning explicitly receives it. The comment
on `convert_with` therefore calls its closure an emission callback.

The registry does not need generated Rust to resolve a binding. It needs a
conversion's semantic dependency edges, returned as `Answer::over(...)`.
Recipe composition already knows those edges; the registry never analyzes a
generated function body to discover them. Keeping `syn::ItemFn`s alive during
the dependency walk adds no model information.

The resumable compiler, shared mutable fragment memo, duplicate function list,
token reparsing and whole-file rewrite are consequences of rendering too early.
They are not required by crossings, recipes or recursive dependency ordering.

## Required boundary

The normative rule lives in
[Flat is the planning boundary](model.md#flat-is-the-planning-boundary). In
operational terms:

- Planning may inspect Flat's structural type class (`TypeKind`), normalized map
  identity (`TypeKey`), crossing mode, structural children, declared fields and
  functions, and source locations.
- Planning may store and pass a `TypeRef` as an opaque source-type reference.
- Planning may not receive `Emit`, obtain a source-side `syn::Type` Rust type
  AST, turn a `TypeRef` into tokens or text, or render a function merely to
  infer its dependencies.
- If planning needs a fact available only from captured Rust syntax, Flat is
  incomplete. Add a lossless fact and its recovery test to Flat.
- Adapter-authored wire syntax is allowed during planning. A C adapter can
  state `*mut c_void` and JNI can state `jlong`: those are output vocabulary,
  not captured source syntax.
- Final emission may spell `TypeRef`s stored in frozen plans, but may not select
  recipes, discover dependencies, rebuild site plans, choose fallback
  converters or reject a newly discovered shape.

## Target control flow

Generation has one semantic phase followed by one Rust-syntax phase:

```text
captured records
    -> Flat
    -> declarations, recipes and recipe selection at each site
    -> ordered crossings
    -> adapter intermediate types, bridges and ABI layouts
    -> registry-composed converter plans
    -> adapter-specific site and artifact plans
    -> validate and freeze the generation plan
    -> shared Rust writer + adapter renderer + Emit
    -> assemble, format and write the file
```

The roles are deliberately narrow:

| Component | Owns | Must not do |
|---|---|---|
| Flat and registry | Source facts, crossing demand and order, dependency completeness | Generate Rust syntax |
| Recipe composer | Source construct/deconstruct steps and recursive converter graphs | Choose a target representation or render source types |
| Adapter planner | Intermediate types, representation bridges, ABI layouts, callback bridges and adapter-specific artifacts | Render captured source types or reproduce the recipe walk |
| Generation plan | Immutable fragment, site and artifact decisions for one generation run | Lazily consult or mutate registry state |
| Adapter renderer | Translate one frozen artifact plan into `syn::Item`s | Receive `&Registry` or re-plan from a source signature |
| Shared Rust writer | Drive artifact iteration, mint `Emit`, collect items, append guards, format and write | Make target-language or crossing decisions |

The shared writer remains one pipeline. Cbindgen and JniGen may retain
convenience `write_rust` methods, but those delegate to it; adapters do not
reimplement ordering, assembly, formatting, destination handling or common
errors.

## Restricted recipe conversion

The recipe compiler should stop asking every adapter to render the same
recursive Rust control flow. For every fragment identity, the adapter assigns
exactly one intermediate type and describes how that type represents the
selected shape. The registry then builds the converter plan mechanically.

"Exactly one" means that a fragment cannot acquire different intermediate
types at different sites. It does not require different fragments to have
different physical Rust types. Two fragments may both use `jint`, for example,
but their generated operation and artifact identities remain distinct. Private
converter symbols therefore derive from the fragment identity and operation,
not merely from a pair of rendered Rust types.

Because a crossing includes its direction, this rule does not force construct
and deconstruct fragments to share one Rust type. They may use different input
and output representations when constness, initialization or the target calling
convention requires it; compatibility between them belongs to the ABI plan.

The intermediate type is deliberately separate from the ABI layout:

```text
source Rust value
    <-> construct/deconstruct through model parts
child source values
    <-> child converter plans
child intermediate values
    <-> representation bridge
one intermediate value
    <-> ABI layout at a site
wire values
```

The first relation is defined by the recipe and Flat. The second is already
resolved recursively. The adapter declares the last two relations without
seeing source Rust syntax. This preserves the model rule that wire values only
appear after the recursive shape walk reaches adapter representation: an
intermediate value is a private carrier for the eventual wire values, not a
third kind of source part.

An intermediate type may be:

- an adapter runtime type, such as a JNI primitive or handle wrapper;
- a tuple used only inside generated Rust;
- a generated private struct or enum;
- a transparent newtype used to give one recipe its own niche or bridge
  operations; or
- another adapter-defined aggregate.

Adapter-authored Rust types remain output vocabulary and may be represented
during planning. An intermediate declaration may refer to child intermediate
identities, but it may not embed the rendered spelling of a source `TypeRef`.
If an operation needs a fact about a source type that Flat does not expose, the
operation is rejected during planning and Flat is extended first.

### The representation protocol

Every non-atomic shape needs a bidirectional representation protocol. The
conceptual common operation is:

```rust,ignore
trait ShapeRepresentation {
    type Parts;

    fn pack(parts: Self::Parts) -> Result<Self, RepresentationError>;
    fn unpack(self) -> Result<Self::Parts, RepresentationError>;
}
```

This is a specification, not necessarily a Rust trait that will be emitted.
Generated private free functions or inlined expressions avoid trait coherence
problems, support borrowed intermediates and allow two recipe rows to reuse the
same physical type. A frozen plan names operations by semantic artifact
identity; the renderer assigns Rust identifiers at the end.

The protocol is shape-specific:

| Shape | Intermediate parts | Required representation operations |
|---|---|---|
| `Atomic` | None | Adapter terminal `decode` or `encode`, plus its ABI layout. |
| `Product` | Ordered tuple of child intermediate values | Pack all positions; unpack all positions exactly once. |
| `Optional` | `Option<I>` | Construct absent/present; distinguish absence; extract the present value; state consumed and remaining niches. |
| `Sequence` | A run of `I` | Input builder (`begin`, `push`, `finish`) and output traversal (`begin`, `next`, `finish`), including length, allocation and cleanup policy. |
| `Choice` | A generated logical sum whose arms contain their child intermediate values | Construct each arm; read the active arm; reject invalid tags; state inert-slot and niche policy. |
| `Invoke` | Callable state plus the argument and future result site plans | Construct the Rust callable and define each later invocation; see [callbacks](#callbacks). |

Fixed positional accessors are sufficient only for `Product`. `Optional`,
`Sequence`, `Choice` and `Invoke` need control-flow operations as well. The
formal API therefore uses pack/unpack terminology for fixed shapes and explicit
builder, traversal, arm and invocation operations for the others.

Each operation plan records:

- which intermediate identities it consumes and produces;
- whether each input is moved, shared-borrowed or mutably borrowed, using the
  crossing and part modes defined by the model;
- the validity of borrowed outputs and what must remain alive;
- whether it can fail and its typed error route;
- allocations or resources it acquires;
- cleanup required on success, foreign-call failure and every earlier
  conversion failure; and
- any niche domain it consumes or leaves available.

The registry validates those facts before rendering. A predefined method name
alone would leave ownership, failure and cleanup implicit and would move type
errors to compilation of the generated crate.

### Atomic terminals

`Atomic` is the only shape for which the adapter supplies the value conversion
rather than a representation bridge over child fragments. Its terminal codec
plan states:

- the source `TypeRef`, kept opaque;
- the intermediate type and direction;
- whether the operation decodes or encodes, borrows or moves, and can fail;
- any validity, niche, allocation and cleanup effects; and
- either a known runtime operation or an adapter-specific terminal artifact to
  render later.

The adapter does not return a `syn::ItemFn` or need to return a rendered function
name. The planner assigns a terminal artifact identity from the fragment and
operation. During final emission the adapter renderer may derive a readable
private Rust name from the spelled source and intermediate types, but that name
is an output of rendering rather than the key used for dependency lookup or
deduplication.

This retains room for genuinely target-specific terminal bodies while keeping
all composed control flow in the registry. A built-in scalar codec may render
as an inline cast, an opaque handle may call a runtime helper, and a string may
emit a reusable private function; the frozen terminal plan chooses among them.

### Converter synthesis

The registry builds two mirror converter plans for a bidirectional recipe. It
does not require the adapter to return a complete converter function.

For `Direction::Construct`:

1. The site ABI plan assembles its wire values into the root intermediate.
2. The root representation bridge unpacks child intermediates.
3. Each child construct plan recursively decodes or constructs its source
   value.
4. Flat's selected product constructor, fields, optional constructor, sequence
   collector or choice arm constructs the source value.
5. Any failed step runs cleanup for intermediates already created but not
   transferred.

For `Direction::Deconstruct`:

1. Flat's selected fields, accessors, value form, optional projection, sequence
   traversal or choice arm deconstructs the source value.
2. Each child deconstruct plan recursively deconstructs or encodes its source
   value.
3. The root representation bridge packs the child intermediates.
4. The site ABI plan flattens the root intermediate into its wire values.
5. Ownership transferred to the foreign side is removed from cleanup; every
   non-transferred value is released on an error path.

The plan is a directed acyclic operation graph after the registry applies the
model's existing recursive-cycle rules. Common subgraphs may be emitted as
shared private helpers or inlined, but that choice is frozen before rendering.
The final renderer is not allowed to rediscover the chain from a source type.

### Worked product and optional

For a hypothetical source value:

```rust,ignore
struct Sample {
    buf: Buffer,
    enc: Option<Encoding>,
}
```

assume the selected recipes and adapter representations are:

```text
Sample           <-> Product(Buffer, Option<Encoding>)
Option<Encoding> <-> Optional(Encoding)
Buffer           <-> Atomic
Encoding         <-> Atomic

IBuffer           = adapter aggregate of (pointer, length)
IEncoding         = adapter integer representation
IOptionalEncoding = adapter niche or (present, value) representation
ISample           = adapter product representation
```

The registry derives the construct chain:

```text
wire values
    -> ISample
    -> (IBuffer, IOptionalEncoding)
    -> (IBuffer, Option<IEncoding>)
    -> (Buffer, Option<Encoding>)
    -> Sample
```

and the deconstruct chain in reverse. Its rendered body may be equivalent to:

```rust,ignore
fn decode_sample(value: ISample) -> Result<Sample, Error> {
    let (buf, enc) = unpack_sample(value)?;
    let buf = decode_buffer(buf)?;
    let enc = unpack_optional_encoding(enc)?
        .map(decode_encoding)
        .transpose()?;
    Ok(Sample { buf, enc })
}

fn encode_sample(value: Sample) -> Result<ISample, Error> {
    let Sample { buf, enc } = value;
    let buf = encode_buffer(buf)?;
    let enc = pack_optional_encoding(
        enc.map(encode_encoding).transpose()?,
    )?;
    pack_sample((buf, enc))
}
```

Those source type spellings and field expressions appear only when the frozen
operations are rendered. If the selected recipe names a constructor and
accessors instead, the plan already contains references to those Flat
functions and the renderer emits calls rather than field syntax.

`IBuffer` being one Rust type does not imply one ABI value. C may flatten it to
`(*const u8, usize)`, while JNI may package it in an object or expose several
native arguments. The root site's ABI layout makes that choice independently
of the recursive converter.

### Niches and inactive storage

Niche handling belongs to the representation layer. For example,
`IOptionalEncoding` may be a transparent newtype over `IEncoding`: its absent
constructor writes a reserved integer, its present constructor rejects that
integer for the inner value, and unpack tests the sentinel. An enclosing
optional can reuse another reserved integer only when the child's representation
plan reports it as still available.

When a representation uses separate presence or choice-tag values, inactive
payload storage is not a valid child intermediate. Its ABI plan states whether
the slot is omitted, initialized as `MaybeUninit`, zero-filled only to avoid
disclosing bytes, or populated with another target-safe inert value. Generic
composition must never manufacture a Rust enum, pointer or handle value merely
to fill an inactive ABI slot.

This replaces the current `Niches` payload of early `syn::Expr`s with a frozen
domain description that the final renderer turns into value and predicate
expressions.

### Callbacks

A callback is not an ordinary product. Its outer crossing constructs because
Rust receives a callable from the foreign side, while every callback argument
deconstructs because Rust later owns that argument and sends it out. This is
the direction swap defined by [the model](model.md#the-direction), and the
registry remains the only component that applies it.

The automatic path for a callback parameter is:

```text
exported-wrapper entry
    foreign callable ABI
    -> callback intermediate
    -> Rust callable

each later Rust invocation
    Rust callback arguments
    -> ordinary Deconstruct converter plans
    -> argument intermediates
    -> callback-delivery ABI values
    -> foreign callable
    -> unconditional per-invocation cleanup
```

A callback bridge is selected once per callback fragment and frozen with:

- the foreign callable's ABI values and ownership transfer at wrapper entry;
- the Rust callable mode, lifetime, `Send` and `Sync` requirements read from
  Flat;
- one callback-argument role site plan per argument, bound to the selected
  direction-swapped deconstruct fragment; when it selects the same row as an
  ordinary output, it reuses the same fragment identity;
- the exact callback-delivery layout of every argument, including flattened
  products, option presence, choice tags, sequence iteration and target object
  wrapping;
- setup performed once when the callable is constructed;
- work performed on every invocation;
- success, failure and drop cleanup, including values offered to foreign code
  but not taken;
- the route for conversion and foreign-call errors when the current
  unit-returning callback provides no error result; and
- an optional callback-result site reserved for
  [issue #216](https://github.com/milyin/prebindgen/issues/216). A future result
  runs `Direction::Construct`: foreign result wire values are assembled into an
  intermediate and then converted into the Rust return value.

Callback arguments must not be reclassified or independently lowered by a
callback emitter. Products, optionals, choices and sequences use the same
deconstruct composer and representation protocols as ordinary outputs while
retaining their callback-argument site selection. Only the final delivery
operation differs because the values are arguments of a foreign callable rather
than the result of an exported wrapper.

The C callback bridge additionally freezes:

- the `#[repr(C)]` closure value containing `context`, `call` and `drop`;
- the exact `call` function-pointer signature after every argument layout is
  flattened;
- whether each opaque argument is borrowed, transferred, or takeable through a
  mutable graveyard slot;
- zero-copy pointer-and-length delivery for borrowed slices when its element
  representation proves the cast valid;
- the rule that no argument conversion runs when `call` is null;
- post-call destruction of takeable values that foreign code did not take;
  and
- the current panic route for a fallible conversion during a callback firing,
  because no caller-side error channel remains.

The JNI callback bridge additionally freezes:

- the Java Virtual Machine handle, global reference and typed `run` method
  descriptor captured at callable construction;
- one interface specification shared by Rust invocation, generated Kotlin and
  the JNI descriptor so they cannot drift;
- daemon-thread attachment and a sized local-reference frame on every
  invocation;
- primitive `jvalue` delivery, object wrapping and any sequence-fold helper
  setup selected for each argument;
- owned-handle close-unless-taken behavior and cloning required to turn a
  borrowed Rust argument into a self-sufficient JVM value;
- unconditional local-frame release on success, conversion failure, JNI
  failure and a thrown JVM exception; and
- exception description/clearing plus logging through `__JniErr`, because the
  current callback signature has no error result.

A callback may fire synchronously inside the source call or asynchronously
after the exported wrapper returns. The bridge records that lifetime instead of
assuming either case. A wrapper-scoped lock may therefore overlap a synchronous
callback under the existing wrapper contract, but the generic composer must not
silently extend its scope. Handle protection and temporary borrows acquired
while encoding callback arguments belong to that invocation and end at cleanup
or the documented ownership transfer.

The callback renderer receives the frozen bridge, the already-resolved
argument converter plans and `Emit`. It may spell the callback's source
argument types to emit the closure signature, but it receives neither
`&Registry` nor a conversion lookup capability. This removes the current JNI
callback emitter's registry queries and the C callback path's separate
structural lowering walk.

### Planning API shape

The exact Rust API is implementation work, but the plan needs these semantic
layers rather than complete `syn::ItemFn`s:

```rust,ignore
struct FragmentPlan<R> {
    id: FragmentId,                 // spelled crossing + complete RecipeKey
    source: TypeRef,                // opaque until final emission
    intermediate: R::Intermediate,
    shape: ShapePlan<R>,
    converter: ConverterPlan,
    dependencies: Vec<FragmentId>,
    yields: Yield,
}

enum ShapePlan<R> {
    Atomic(R::TerminalCodec),
    Product(R::ProductBridge),
    Optional(R::OptionalBridge),
    Sequence(R::SequenceBridge),
    Choice(R::ChoiceBridge),
    Invoke(R::CallbackBridge),
}

struct SitePlan<R> {
    fragment: FragmentId,
    abi: R::AbiLayout,
    role: Role,
    cleanup: R::Cleanup,
}
```

Here `R` is adapter-owned representation vocabulary. The registry understands
the shape protocol and child identities but treats target types, ABI values,
target metadata and cleanup operations as typed adapter data. The adapter may
offer declarative standard bridges—tuple product, sentinel optional, tagged
choice—or an external runtime bridge with a fixed semantic symbol. It may not
return arbitrary already-rendered converter bodies.

Before freezing, validation proves:

- every reached fragment identity has exactly one intermediate type and one
  selected shape plan;
- every intermediate part has the same fragment identity, mode and validity as
  the corresponding resolved child;
- pack and unpack arity agree and every product position or choice arm is
  covered exactly once;
- every site ABI value is produced or consumed exactly once and every inactive
  slot has a target-safe policy;
- nested niche domains are disjoint and sufficient for their discriminants;
- every failure edge has cleanup and an error route;
- every callback argument uses the direction-swapped fragment selected for its
  `Role::CallbackArg` site, and Rust, ABI and target-language callback
  signatures have the same ordered leaves; and
- no converter, bridge or callback plan contains rendered source syntax.

## The frozen generation plan

The recipe-composition skeleton is shared while its representation payloads and
wrapper artifacts remain adapter-specific. Each immutable store must contain at
least:

- one fragment plan per fragment identity, including its intermediate type,
  selected shape bridge, semantic dependencies, ownership, validity, failure,
  niche and cleanup operations, and source positions stored as `TypeRef`s;
- one site plan for every crossing position in a declared function or callback,
  with its exact selected fragment, ABI layout and error route;
- one converter operation graph per reached fragment direction, including an
  explicit decision to inline it or emit a stable private artifact;
- one callback bridge per reached `Invoke` fragment, including its argument
  sites, callable setup, per-invocation work, target signature and cleanup;
- one artifact plan for every final top-level item. A private conversion may be
  represented as its own helper, an operation in another artifact or a shared
  helper; the plan need not preserve one node per converter function in the old
  output;
- an explicit artifact order, compatible identities for public symbols and
  deterministic identities for private artifacts, so ordering and duplicate
  detection do not depend on rendered function names;
- all prerequisites needed by the selected artifacts; and
- typed planning and validation errors, produced before an output path is
  opened.

The Rust and Kotlin sides of a JNI binding must read the same frozen plan
instance. They cannot rebuild plans independently or retain a mutable cache that
allows the two writers to observe different decisions.

## The final Rust-writing pipeline

The shared writer performs one fixed sequence:

1. Accept a validated generation plan and its adapter renderer.
2. Mint the only `Emit` that the generation run exposes.
3. Walk the already-ordered artifact plans.
4. Give the renderer one item-specific plan and `&Emit`; collect its
   `Vec<syn::Item>` result.
5. Append the source feature guards represented by the frozen plan.
6. Apply only syntax-level normalization that cannot affect a generation
   decision.
7. Format the full AST and write the destination.

Conceptually, not as a committed public API:

```rust,ignore
struct Generation<P> {
    artifacts: Vec<P>, // complete, validated and already ordered
}

trait RenderRust {
    type ArtifactPlan;

    fn render_artifact(
        &self,
        plan: &Self::ArtifactPlan,
        emit: &Emit,
    ) -> Result<Vec<syn::Item>, RenderError>;
}

fn write_rust<R: RenderRust>(
    generation: &Generation<R::ArtifactPlan>,
    renderer: &R,
    destination: impl AsRef<Path>,
) -> Result<PathBuf, WriteError>;
```

Calling `render_artifact` is the single target-specific boundary in the shared
pipeline. Unlike the current `Prebindgen` protocol, it is not split by source
item kind and it exposes neither the registry nor a global plan store from which
the renderer could select another answer.

The renderer returns `syn::Item`s directly, eliminating the current
`TokenStream -> syn::File -> syn::Item` parse round trip. Source qualification
should happen when the renderer spells each stored `TypeRef`. If a final
syntax-only normalizer remains, it receives no registry and cannot change which
artifacts or conversions exist.

## Optimizations this boundary permits

Requiring byte-identical generated files would preserve several artifacts that
the new boundary makes unnecessary. Semantic preservation allows the planner
and renderer to improve them:

- Emit fully qualified source types directly and delete the whole-file
  `post_process_item` rewrite, even when that changes generated spelling.
- Return `syn::Item`s directly and let ordinary formatting determine whitespace
  and item layout instead of preserving token-reparse artifacts.
- Inline or combine private converters and JNI pre-stage functions when a
  separate generated function provides no reuse or diagnostic value.
- Remove unreachable private helpers instead of retaining them to keep a golden
  file unchanged.
- Deduplicate artifacts by semantic identity before rendering rather than by a
  rendered function name afterwards.
- Use the complete frozen plan to choose reusable helpers versus inlined code,
  reducing generated code size or call overhead without exposing source syntax
  to planning.

An optimization that changes which artifacts exist or how they compose is a
planning decision and must be stored in the frozen plan. The renderer only
realizes that decision; semantic freedom does not permit it to start planning
again.

## What the change removes and preserves

The proposal removes accidental coordination:

- `Emit` from `RegistryBuilder::convert_with`, `recipe::Compiler` and all
  `Compile` contexts;
- generated `syn::ItemFn`s from fragment planning;
- the duplicate `compiled_fns` caches;
- the `Rc<RefCell<Compiled<_>>>` clone/resume/finish exchange used so generation
  can observe its own partial output;
- lazy or later compiler resumes used to construct site plans after conversion
  resolution;
- registry-bearing item-kind emission callbacks;
- adapter copies of product, optional, sequence and choice converter chaining;
- the C callback path's second structural lowering walk and the JNI callback
  renderer's registry lookups;
- generated-token reparsing; and
- name-based converter deduplication after rendering.

It preserves model complexity:

- Flat as the sole description of captured Rust;
- crossing dependency order and the explicit rule for recursive cycles;
- the distinction between reusable recipe fragments and per-site plans;
- adapter choice of intermediate types, representation bridges, ABI layouts,
  target metadata and wrapper artifacts;
- typed planning and validation failures; and
- the callback rule in which argument crossings run opposite to the callback
  crossing, plus each target's callable ownership, invocation, error and cleanup
  contract, as described under [direction](model.md#the-direction).

## Migration

The migration uses semantically preserving vertical slices. A slice changes one
adapter all the way from planning through final emission and deletes the state
it replaces, rather than maintaining parallel semantic and pre-rendered
representations merely to keep generated text identical.

1. **Pin the observable contracts and failure behavior.** Add or identify tests
   for every representation family before changing its planner. Record exported
   C symbols, function-pointer signatures, struct and union layouts; JNI native
   names and descriptors; Kotlin public signatures; ownership and cleanup;
   fallible conversion, panic and exception routes; and concurrency behavior.
   Callback coverage must include zero arguments, scalar and composite
   arguments, optional and sequence arguments, borrowed and owned handles,
   null C call pointers, C takeable arguments, repeated JNI calls from a daemon
   thread, thrown JVM exceptions and callback-object destruction. Generated
   source remains a review aid rather than the specification.
2. **Introduce syntax-free plan vocabulary.** Add `FragmentId`, intermediate
   type and ABI layout descriptors, shape bridge plans, converter operations,
   cleanup operations and semantic artifact identities. Move `Yield`, modes,
   validity and dependency edges onto those plans. Add validation tests with a
   small fake adapter for arity, identity, ownership, niche, failure-route and
   direction errors. This stage emits no new production Rust path.
3. **Build the shared recipe composer.** Implement construct and deconstruct
   operation graphs for `Atomic`, `Product` and `Optional`, including the
   worked `Sample` chain, partial-construction cleanup and nested niche
   propagation. Add `Sequence` builder/traversal and `Choice` arm/tag protocols
   only after the fixed-arity path proves the API. The composer consumes Flat
   operations and child fragment identities, never `Emit` or `syn::Type` from a
   source `TypeRef`.
4. **Migrate ordinary C crossings.** Express the existing scalar, pointer,
   string, data-struct, optional, slice and tagged-union representations as C
   intermediate and ABI plans. Move field construction/deconstruction, optional
   propagation, sequence lowering and choice control flow to the shared
   composer. Preserve `MaybeUninit`, inactive-slot, borrowed-pointer, typed-drop
   and `Result` routing policies as explicit C operations. Render from the
   frozen graph and remove the migrated branches from C's `Compile` hooks,
   `lower_shape` and `encode_value` rather than retaining a second answer.
5. **Migrate C callbacks as a separate vertical slice.** Freeze the closure
   struct and its `call` signature from the callback argument site plans. Make
   callback firing invoke the same deconstruct graphs used by ordinary outputs.
   Encode only inside the non-null `call` guard; retain zero-copy slices,
   takeable graveyard slots, post-call destruction and panic-on-conversion-
   failure. Render the callback struct, Rust closure and header-facing ABI from
   the same bridge. Delete callback-specific structural classification and
   registry queries when this slice lands.
6. **Migrate ordinary JNI crossings.** Represent whole-object and flattened
   input/output forms as intermediate and ABI plans. Fold existing pre-stages
   into the converter operation graph. Make exported Rust wrappers, generated
   Kotlin declarations, native descriptors, handle-lock sets, exception routes
   and cleanup all consume one frozen plan. Migrate products and optionals
   first, then sequences and choices; after each family moves, delete its
   parallel `expand`, `unfold` or `fn_plan` carrier. This deletion coordinates
   with [issue #506](https://github.com/milyin/prebindgen/issues/506).
7. **Migrate JNI callbacks as a separate vertical slice.** Freeze the callback
   interface specification, method descriptor, argument leaves, one-time setup,
   per-invocation local-frame size and cleanup. Reuse the ordinary deconstruct
   graph for each callback argument, including pre-stages, flattened data,
   optional gates, sequence folds and owned-handle delivery. Retain daemon
   attachment, cached method lookup, close-unless-taken handles, unconditional
   local-frame release, exception clearing and asynchronous error logging.
   Generate the Kotlin interface and Rust `jvalue` list from the same ordered
   leaves. Delete `callback_input` registry lookups and its independent output
   chain only after the runtime callback tests cover the new bridge.
8. **Move all source spelling to final emission.** Replace the remaining
   complete functions inside `ConverterImpl` and `Stage` with frozen operations.
   Render converter, wrapper, helper-type and callback artifacts through the
   adapter renderer. Make qualification happen while `Emit` spells each stored
   source reference, and remove the whole-file `post_process_item` rewrite.
9. **Delete and seal the old path.** Remove `Emit` from `convert_with`,
   `recipe::Compiler` and every planning context; remove `compiled_fns`, the
   shared mutable compiler memo and name-based deduplication; and replace the
   item-kind `Prebindgen` callbacks with the single artifact-rendering boundary.
   Compile-fail and call-graph checks then enforce that planning cannot obtain
   source spelling and rendering cannot obtain a registry.

Each numbered stage may need multiple pull requests, but each pull request is
an independently reviewable vertical slice: its description names the migrated
shape and adapter behavior, identifies the old path deleted in the same diff,
and reports exact verification performed. A temporary comparison harness may
run old and new plans in tests, but production generation must have one answer
for every migrated fragment when the pull request merges.

The Flat boundary is complete after both adapters stop rendering during
planning and stage 9 removes the generic escape. Generated output may evolve at
each slice; every externally visible difference still requires an explicit
contract change outside this proposal.

## Relation to existing work

- [Issue #195](https://github.com/milyin/prebindgen/issues/195) requires pure
  emission: renderers consume frozen plans rather than querying registry state.
  This proposal supplies the complementary rule that source Rust spelling is
  unavailable until that pure emission begins.
- [Issue #251](https://github.com/milyin/prebindgen/issues/251) moved crossing
  demand and answers out of the old callback protocol. Its remaining
  emission-out direction is completed here without duplicating the Rust writer.
- [Issue #506](https://github.com/milyin/prebindgen/issues/506) tracks deletion
  of JNI's legacy plan carriers. This proposal uses that deletion during
  migration but does not turn those carriers into the replacement architecture.

## Exit checks

The boundary and behavior are mechanically testable:

- No function reachable from planning or validation receives `Emit`, implements
  `RustEmitter`, or renders a source-side `TypeRef`.
- No function reachable from the final renderer receives `&Registry`, rebuilds
  a fragment or site plan, or classifies a source signature.
- Every source-side type in a generation plan remains a `TypeRef` until final
  emission.
- Validation and every artifact writer observe the same immutable plan store.
- All source-spelling calls are reachable only from the shared Rust writer's
  final-emission boundary.
- Every reached fragment has one intermediate type, and every reached site has
  one ABI layout referring to that fragment rather than a second conversion.
- Product, optional, sequence and choice converter bodies are derived from the
  shared operation graph; adapter planning code contains representation policy
  but no duplicate recursive source-value walk.
- C callback declarations and invocation bodies consume the same ordered
  callback-argument site plans, including takeable and inactive-slot policy.
- JNI callback interfaces, method descriptors and Rust `jvalue` lists consume
  the same ordered callback leaves.
- Callback rendering cannot query the registry; callback runtime setup,
  per-invocation cleanup and asynchronous error routes are frozen before source
  types are spelled.
- Callback-result absence is explicit. Adding callback results uses the frozen
  construct-direction result site instead of adding another callback-only
  converter path.
- Exported C symbols, signatures and layouts remain compatible; JNI native names
  and descriptors remain compatible; generated Kotlin retains its public
  signatures and behavior.
- Generated-source changes are classified as private restructuring,
  qualification, formatting or an explicit optimization. No unexplained public
  delta is accepted.

Every implementation pull request reports the exact subset it ran and why any
item was not applicable. The complete migration gate is:

- `cargo fmt --all --check`;
- `cargo test --workspace`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`;
- `./examples/regen-check.sh`, with every generated-source difference reviewed
  and classified;
- `./examples/smoke-asan.sh` for ownership, partial-construction and callback
  cleanup; and
- `cd examples/covertest-kotlin && ./gradlew run --console=plain` for JNI and
  Kotlin behavior.

The callback migration does not pass on generation snapshots alone. Its C unit
tests and JNI covertest cases must execute callback creation, repeated firing,
failure cleanup and final destruction on the generated boundary.
