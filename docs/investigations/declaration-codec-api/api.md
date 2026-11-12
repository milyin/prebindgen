# Proposed operation contracts

These are **proposed signatures**, not implemented APIs. They complete the
[main proposal](README.md). Standard names such as `Type`, `Ident`, `TokenStream`,
`Flat`, `WireNode`, `InputPlan`, `OutputPlan`, `Condition` and `Projection` refer
to the existing Rust/prebindgen types. Request structures are temporary inputs
to generation; their contents are not copied into another binding registry.

## Current baseline for comparison

Today's leaf signatures spread positional inputs over two methods:

```rust,ignore
fn input_leaf(&self, flat: &Flat<Self::Schema>, ty: &Type,
              extra: &Self::Extra, wire: Ident, target: Ident,
              active: &Condition) -> Result<CodecInput<Self>>;
fn output_leaf(&self, flat: &Flat<Self::Schema>, ty: &Type,
               extra: &Self::Extra, wire: Ident, value: TokenStream,
               borrowed: bool) -> Result<CodecOutput<Self>>;
```

In addition to these, `consumes`, and `group_metadata`, a scalar-only codec must
provide seven more bodies. With its `Niche`, `Chain` and `Branch` types all `()`,
the currently required stubs can look like this (signatures abbreviated with
`...` where their arguments do not affect the example):

```rust,ignore
fn optional_metadata(...) -> Result<()> { Err("options unsupported".into()) }
fn alternatives_metadata(...) -> Result<()> { Err("alternatives unsupported".into()) }
fn optional_niche(...) -> Option<NicheReservation<()>> { None }
fn retire_niches(...) -> Result<()> { Ok(()) } // This codec never offers niches.
fn field_group(...) {}
fn fields_group(&self) {}
fn selector(&self, _: &()) -> Type { syn::parse_quote!(i32) } // Cannot return Err.
```

The proposed minimal implementation in the main document replaces the first
four operations with request-based input/output, `output_consumes`, and
`group(kind)`. It needs none of the seven stubs. Unsupported group kinds can now
return an error instead of being forced to invent metadata.

## Core

The six associated types retain their current bounds and roles, except that
`Niche` is removed. `Schema` still fixes the language's converter/metadata types
before parsing. Its converters still accept the codec's compiled plans through
`From<CodecInput<Self>>` / `From<CodecOutput<Self>>` and implement the existing
input/output converter contracts. This proposal does not change those bounds.

```rust,ignore
pub trait Codec {
    type Schema;  // Existing Schema/converter bounds retained.
    type Wire;    // Existing WireKind + Clone + Debug bounds retained.
    type Context; // Existing TemplateContext + 'static bound retained.
    type Extra;
    type Chain: Clone;
    type Branch: Clone;

    fn input_leaf(&self, r: InputLeaf<'_, Self>) -> Result<CodecInput<Self>>;
    fn output_leaf(&self, r: OutputLeaf<'_, Self>) -> Result<CodecOutput<Self>>;
    fn output_consumes(&self, flat: &Flat<Self::Schema>, ty: &Type,
                       extra: &Self::Extra) -> bool;
    fn group(&self, kind: GroupKind<'_>) -> Result<Self::Branch>;

    // Optional operation signatures and defaults are specified below.
}
```

`CodecInput<C>` and `CodecOutput<C>` remain the existing plan aliases. Below,
`Node<C>` abbreviates `WireNode<C::Wire, C::Chain, C::Branch>`, and `Values`
abbreviates `Vec<(Ident, TokenStream)>`. Request generic parameters admit
`C: Codec + ?Sized`; the sketches do not introduce an object-safe trait or a
runtime codec interface. `C::Wire` is the adapter's **wire kind**;
`prebindgen_flat::Wire<C::Wire>` adds a binding name to that kind. Flag/control
values use the named form, while `selector` constructs a kind that the compiler
names and prefixes.

`output_consumes` is the renamed `consumes`. Its answer must agree with the
output leaf's move policy. The compiler needs it before compiling output views
so it can observe fields before moving the owner and reject multiple moves.
An explicit projection still participates in the compiler's existing ownership
analysis; this method is not permission to move out of a shared reference.

```rust,ignore
pub enum GroupKind<'a> {
    Values,                  // Constructor arguments / combined output views.
    Fields,                  // Ordered source fields, including an empty group.
    Field(&'a Ident),        // One named field with its complete subtree.
    NamedOutput(&'a str),    // An explicitly named output view.
}
```

The compiler asks for precisely the former group kind; it does not erase field
or view names. `group` returns an error when a kind is unsupported. Requiring
this operation retains a deliberate answer for wire-tree structure rather than
assuming every metadata type can silently represent every group.

## Leaf requests

```rust,ignore
pub enum Direction { Input, Output }
pub enum Representation { Ordinary, Bulk }
pub enum InputConstruction { EnclosingConversion, GroupedRow }
pub enum InputRole { Parameter, CallbackResult }
pub enum ValueAccess { Owned, Shared }

pub struct InputLeaf<'a, C: Codec + ?Sized> {
    pub flat: &'a Flat<C::Schema>,
    pub ty: &'a Type,
    pub extra: &'a C::Extra,
    pub wire: Ident,
    pub target: Ident,
    pub active: &'a Condition,
    pub representation: Representation,
    pub construction: InputConstruction,
    pub role: InputRole,
}

pub struct OutputLeaf<'a, C: Codec + ?Sized> {
    pub flat: &'a Flat<C::Schema>,
    pub ty: &'a Type,
    pub extra: &'a C::Extra,
    pub wire: Ident,
    pub value: TokenStream,
    pub access: ValueAccess,
    pub representation: Representation,
}
```

`Ordinary` means the adapter's normal representation, including normal fields
inside ordinary collection rows. `Bulk` selects native record fields. These
labels do not claim that C's ordinary representation is a Java-like object.

The exact existing input dispatch becomes:

| Current selected method | Representation | Construction |
| --- | --- | --- |
| `input_leaf` | Ordinary | EnclosingConversion |
| `input_row_leaf` | Ordinary | GroupedRow |
| `input_bulk_leaf` | Bulk | EnclosingConversion |
| `input_bulk_row_leaf` | Bulk | GroupedRow |

The compiler computes this context from the same conditions as today:
`bulk_input_row && grouped_input` selects bulk grouped-row conversion; otherwise
`bulk` selects bulk enclosing conversion; otherwise `grouped_input` selects
ordinary grouped-row conversion. The context is not inferred just from the
source type or from whether a collection appears higher in the recipe.

`role` is propagated unchanged through the conversion being prepared. An input
compiled by `compile_callback_return` has `CallbackResult`; ordinary function
parameters have `Parameter`. This replaces C's private codec-instance flag used
for callback-result ownership handling. Row construction is independent of that
role.

For output, `access` describes the supplied expression: `Owned` supplies a value
of the declared Rust type `T`; `Shared` supplies an observation `&T`. `Owned`
does not grant ownership of a pointee when `T` is itself a Rust reference.
Source-return reference handling keeps its separate hook and original type.

Explicit projections retain their own operation:

```rust,ignore
fn output_projected(&self, r: OutputLeaf<'_, Self>, projection: &Projection)
    -> Result<CodecOutput<Self>>;
```

The shared compiler retains its existing rejection of an `Owned` projection
of a shared occurrence, and normalizes expression/access as today. After that
normalization, `Shared` access must correspond to `ProjectionMode::Borrowed`.
The **default hook** rejects field overrides and an owned projection of a
reference payload, looking through optional layers exactly as the current
trait default does; it then delegates to `output_leaf`.

Concrete overrides retain their existing type-specific checks and behavior.
In particular, the C/JNI overrides currently check direct references rather
than applying the default's optional-layer scan. Moving that stronger scan to
the shared compiler would newly restrict declarations and is **not** proposed.
Adapters may override field handling and explicit copying just as today.
Ordinary borrowed output does not authorize cloning.

This separate operation preserves the default projection checks that would
otherwise disappear inside a required generic output method. The four old
ordinary/bulk leaf/projected methods become **two**, not one.

```rust,ignore
fn supports_bulk(&self, direction: Direction) -> bool { false }
```

The shared compiler calls `supports_bulk(Direction)` before **every** bulk
leaf, sequence transport, or whole-expansion packing request. The default is
false. C keeps false; JNI opts in for its implemented directions. An adapter
that only supports bulk output can opt in only for `Output`. An opt-in does not
promise every source type is supported; unsupported types must still fail.
This gate preserves today's unsupported-by-default behavior even if a simple
leaf implementation never examines `representation`.

`GroupedRow` retains the old ordinary-row fallback: using the ordinary leaf
plan is permitted where it already works. There is no obligation to write a
four-arm match for resource-free primitives. C's receipt handling and JNI's
bulk-buffer/enum storage are documented specializations that must inspect the
context. A request flag alone cannot prove a custom plan's lifetime correctness.

The combined methods must preserve all four existing leaf implementations'
generation behavior. In particular, a bulk/grouped-row conversion cannot
silently delegate to ordinary JNI conversion or discard row-local storage.

## A complete optional encoding

```rust,ignore
fn optional(&self, r: Optional<'_, Self>) -> Result<OptionalEncoding<Self>>;
// Default: Err("optional conversion is unsupported by this codec").

pub struct Optional<'a, C: Codec + ?Sized> {
    pub extra: &'a C::Extra,
    pub payload: &'a mut Node<C>,
    pub presence: Ident,
    pub direction: Direction,
}
pub enum Presence<W> {
    Flag(prebindgen_flat::Wire<W>),
    Niche { input_test: Option<TokenStream> },
}
pub struct OptionalEncoding<C: Codec + ?Sized> {
    pub presence: Presence<C::Wire>,
    pub metadata: C::Chain,
    pub absent_values: Values,
}
```

This operation chooses the physical representation and prepares its metadata;
the shared compiler retains optional branching, lifetime scopes and ownership
commit. A flag-only codec can return `Flag` and inert absent expressions without
defining a niche type or any niche-selection/retirement hooks: it never offers
spare payload representations. Its invariants are:

1. A flag is a single `i32` wire named by the requested `presence` binding, with
   0 meaning absent and 1 present. Its own unused representations may remain
   available to an enclosing option. The compiler retains today's input-tag
   validation policy (`validates_inputs`).
2. A niche adds no control wire. For input, `input_test: Some(test)` is required:
   a boolean expression inspecting the selected payload discriminator, total
   and side-effect-free even for inactive initialized values. It must not
   dereference a foreign pointer. The compiler binds normalized 0/1 presence
   from it. Output uses `input_test: None`; it already branches on Rust's
   `Option`, so it must not emit an unused presence test.
3. `metadata` retains the discriminator and reservation needed by that language's
   readers/validators. It contains neither duplicate wires nor another payload
   subtree. The compiler wraps the supplied payload in one chain node.
4. The operation updates the payload's available niches: remove offers on
   inactive values, preserve only the chosen donor's *remaining* domain, and
   never restore an already reserved value. Nested options must exhaust a domain
   or use a new flag exactly as today.
5. For output, `absent_values` contains one inert expression for every physical
   payload value crossing this native scope, keyed by wire name. The added flag
   is supplied by the compiler, not duplicated here. For a niche encoding, the
   donor's absent expression must produce **the reserved niche value**, so the
   foreign reader observes `None`. Input encoding returns no absent output
   values; input placeholders used elsewhere remain the adapter's responsibility.
6. Candidate/absent-value traversal respects row and bulk boundaries. A child
   record's fields inside a bulk carrier or a collection row are not outer
   parameters. The adapter keeps its existing scope-aware traversal.

### Reusing the niche algorithms

An opt-in utility can retain the existing algorithm without adding provider
traits to every codec. This is the proposed shape, with abbreviated generic
bounds for readability:

```rust,ignore
pub fn prepare_optional_payload<'a, W, N>(
    wires: impl IntoIterator<Item = &'a mut Wire<W>>, // One native scope only.
    direction: Direction,
    available: impl Fn(&W) -> Option<N>,
    set_available: impl Fn(&mut W, Option<N>) -> Result<()>,
    present: impl Fn(&W, &N, TokenStream) -> Result<TokenStream>,
    remaining: impl Fn(&W, &N) -> Option<N>,
    absent: impl Fn(&W, Option<&N>) -> Result<TokenStream>,
) -> Result<PreparedOptionalPayload<N>>; // W: WireKind + 'a, N: Clone.

pub struct PreparedOptionalPayload<N> {
    pub reservation: Option<NicheReservation<N>>,
    pub input_test: Option<TokenStream>,
    pub absent_values: Values,
}
```

The utility chooses the first candidate niche, prepares its input test or absent
output expressions **before retiring the old wire state**, and updates remaining
availability. It creates neither flags nor language metadata. The codec wraps
its result in `OptionalEncoding`, choosing its existing flag if no niche was
found. The returned reservation stores only the selected wire's identifier and
its reserved value, not another wire/payload tree.

The five wire-encoding functions remain real work for a niche-capable codec.
C/JNI already have most of them as wire methods or private optional helpers;
they are supplied here once, not forwarded through `Codec` and its wrappers.
A flag-only codec can implement `optional` directly without this utility or a
niche type. This reduces compulsory extension work; it does not eliminate the
representation knowledge needed for optimized options.

The existing public niche utilities must migrate from `Codec::Niche` to explicit
wire/type arguments. Retirement also remains reusable outside options:
JNI data-class annotation clears field niche offers, and alternatives retire
inactive arm offers. Those callers must use the migrated utility/private wire
helpers rather than a removed trait method. C error-return initialization also
uses the old `absent_value` hook and must call the same C wire helper directly.

All current calls to the trait's `absent_value` in shared composition and C error
emission pass `false` for its input flag. JNI bulk input separately calls the
private `optional::absent(..., true)` to initialize inactive native fields; that
path must remain. Empty `OptionalEncoding.absent_values` for input does not
remove native input placeholder generation.

The shared compiler checks flag naming/count, duplicate/missing output bindings
and direction: input `Niche` must supply `Some(input_test)`, output `Niche` must
supply `None`, and input absent-output expressions must be empty. It rejects a
malformed encoding before installing a slot. The adapter's existing metadata
validation checks that a flag uses its `i32`-compatible selector representation;
the generic compiler cannot infer ABI aliases from an arbitrary wire kind.
These are generation-time checks, not additional runtime validation. Errors during preparation do not install a partially built
converter in Flat. They need not restore discarded temporary plans.

## Alternative input encodings

```rust,ignore
fn selector(&self, extra: &Self::Extra) -> Result<Self::Wire>;
// Default: Err("selector tags are unsupported by this codec").
fn alternatives(&self, r: Alternatives<'_, Self>) -> Result<Self::Branch>;
// Default: Err("input alternatives are unsupported by this codec").

pub struct Alternatives<'a, C: Codec + ?Sized> {
    pub selector: &'a Wire<C::Wire>, // Already prefixed by the compiler.
    pub arms: &'a mut [Node<C>],
}
```

Selector construction remains separate because naming must precede metadata:

1. The compiler creates the `i32` selector using `selector(extra)` and applies
   `prefix_layout(..., Selector)` to that selector alone.
2. It compiles arms under the requested tag's activation conditions and prefixes
   their paths with `Alternative(i)`.
3. It passes the **prefixed selector** and mutable arm trees to `alternatives`.
   The codec validates the selector's **representation at generation time**,
   retires inactive arm niche offers, and returns metadata. The compiler must not prefix the entire resulting branch again.

Tags still select arm indices `0..arms.len()`. The codec must disable unused
arm offers that an enclosing optional cannot safely use, preserving inner
reservations and native row/bulk scopes. This absorbs the alternatives use of
`retire_niches`, while preserving the original selector-prefix ordering.
The compiler retains Rust match construction and its trusted/validated invalid
tag policy; empty alternatives remain errors.

## Sequence transport

```rust,ignore
fn sequence_input(&self, r: SequenceInputRequest<'_, Self>)
    -> Result<SequenceInput<Self::Wire, Self::Chain>>;
fn sequence_output(&self, r: SequenceOutputRequest<'_, Self>)
    -> Result<SequenceOutput<Self::Wire, Self::Chain>>;
// Defaults: precise unsupported generation errors.
```

The requests contain all the existing operation inputs explicitly:

```rust,ignore
pub struct SequenceInputRequest<'a, C: Codec + ?Sized> {
    pub extra: &'a C::Extra,
    pub wire: Ident,
    pub length: Ident,
    pub index: Ident,
    pub element: &'a Node<C>,
    pub active: &'a Condition,
    pub representation: Representation,
}
pub struct SequenceOutputRequest<'a, C: Codec + ?Sized> {
    pub extra: &'a C::Extra,
    pub wire: Ident,
    pub length: Ident,
    pub index: Ident,
    pub element: &'a Node<C>,
    pub values: &'a [(Ident, TokenStream)],
    pub delivery: SequenceDelivery,
    pub representation: Representation,
}
```

Output `Auto` selection is resolved
by the shared compiler as today, before requesting transport. The returned
`SequenceInput`/`SequenceOutput` structures and their lifetime/commit contracts
remain unchanged.

This replaces the separate `sequence_input_bulk` / `sequence_output_bulk`
methods. It does not combine array transport with bulk-packing an individual
expanded record: `bulk_input` / `bulk_output` remain distinct operations.

Whole-collection bulk storage cannot itself use per-row callback delivery.
An ordinary callback sequence **can** have bulk-packed elements: its request
uses `Ordinary`, and the element tree retains its bulk node. This preserves the
existing callback API rather than removing or conflating either boundary.

## One source-error channel plan

```rust,ignore
fn source_error(&self, r: SourceError<'_, Self>) -> Result<SourceErrorPlan<Self>>;
// Default: mapped-String preparation below, or an explicit missing-mapper error.
fn mapped_error(&self, flat: &Flat<Self::Schema>, ty: &Type,
                message: TokenStream, extra: &Self::Extra) -> Result<TokenStream>;
// Default: Err("mapped source errors are unsupported by this codec").

pub struct SourceError<'a, C: Codec + ?Sized> {
    pub flat: &'a Flat<C::Schema>,
    pub ty: &'a Type,          // Original owned error type E.
    pub extra: &'a C::Extra,
    pub value: TokenStream,   // Expression owning the caught E.
    pub message: Option<TokenStream>, // Only an explicitly declared mapper.
}
pub struct SourceErrorPlan<C: Codec + ?Sized> {
    pub payload: Node<C>,
    pub propagate: TokenStream,
}
```

`propagate` retains the existing expression protocol: produce a `String` for
the active fallible boundary, possibly after delivering the declared typed
error and marking it reported. The compiler emits `return Err(propagate)` in
the source `Err` arm. The layout describes exactly that error delivery.

The default `source_error` requires `message: Some(...)`, calls `mapped_error`,
and obtains the payload through the ordinary String output leaf, matching the
current mapped-message fallback. A codec overriding only `mapped_error`
therefore retains that convenience; it does not have to prepare a typed channel.
An adapter with typed errors overrides `source_error` instead. The compiler
checks the returned layout against the declared error wires as today.

The mapper expression is evaluated at most once and may consume `E`; the
implementation must not also consume `value`. An absent mapper permits a typed
error or an unsupported diagnostic; it never requests implicit `to_string()`.
The existing binding-error channel and function-return `DeliveryPolicy` remain
separate. The helper/capture mechanism and Kotlin handler invocation timing are
unchanged by consolidating this preparation operation.

## Complete proposed method inventory

There are **36 methods: four required and 32 with defaults**. Only changed
signatures above gain requests; unmentioned retained methods keep their current
arguments. Source occurrence context should be added by the compiler when
wrapping a default unsupported diagnostic.

| Method | Default / obligation |
| --- | --- |
| `input_leaf` | Required. |
| `output_leaf` | Required. |
| `output_projected` | Existing field/ownership checks, then `output_leaf`. |
| `output_consumes` | Required; agrees with output move policy. |
| `group` | Required; may reject unsupported group kinds. |
| `optional` | Unsupported. |
| `alternatives` | Unsupported. |
| `selector` | Unsupported; returns `Result<Self::Wire>` instead of forcing a dummy tag. |
| `supports_bulk` | `false` for each direction; enforced by the shared compiler. |
| `sequence_input` | Unsupported; representation is explicit. |
| `sequence_output` | Unsupported; representation and delivery are explicit. |
| `bulk_input` | Unsupported. |
| `bulk_output` | Unsupported. |
| `callback_input` | Unsupported. |
| `source_error` | Mapped String layout + `mapped_error`; missing mapper is an error. |
| `mapped_error` | Unsupported. |
| `binding_failure_layout` | `Ok(None)`, no binding-failure channel. |
| `input_record_metadata` | Unsupported foreign-record representation. |
| `output_record_metadata` | Unsupported foreign-record representation. |
| `input_borrowed_sequence` | `Ok(None)`, decline specialization. |
| `output_borrowed_sequence` | `Ok(None)`, decline specialization. |
| `output_return_reference` | `Ok(None)`, retain existing adapter policy. |
| `output_owner` | `Ok(None)`, decline final-allocation specialization. |
| `expand_input` | `Ok(None)`, no language-specific normalization. |
| `expand_output` | `Ok(None)`, no language-specific normalization. |
| `input_collection_is_leaf` | `false`. |
| `output_collection_is_leaf` | `false`. |
| `default_output_sequence_is_bulk` | `false`. |
| `validates_inputs` | `true`; C continues overriding to `false`. |
| `prefix_layout` | No source-path metadata to update. |
| `finish_input` | `Ok(())`; root-only finalization. |
| `finish_output` | `Ok(())`; final delivery remains a root operation. |
| `validate_input` | `Ok(())`; adapters validate their own metadata. |
| `validate_output` | Existing named-value binding validation. |
| `validate_owner_getter` | `Ok(())`. |
| `validate_getter_output` | `Ok(())`. |

`None` above means that a specialization declined the request; it is not an
unsupported representation being silently accepted. The existing general path
must still honor its borrowing/copying and lifetime rules. In particular, C's
borrowed input/output guards and owner-backed getter checks remain implemented.

The proposal retains separate root finalization and validation. Calling a
foreign record a single value, generating a callback, preparing final owner
storage, and validating an escaping getter are also distinct real operations;
they are not folded into a universal request enum merely to lower the count.

## Language codec policy

The C `EnumCodec` adds declared enum decoding, structured errors and callback
preparation to raw `CCodec` operations. JNI's `DataCodec` adds data-class recipe
normalization/annotation, declared enums/errors and a borrowed set selecting
callback ownership policy. Their fallback calls implement a raw/declared policy
boundary. Neither wrapper is the recursive record compiler; that logic already
lives in shared preparation and the class-preparation pass.

Proposed language construction is explicit:

```rust,ignore
let raw = CCodec::raw();               // Same policy as today's public CCodec.
let declared = CCodec::declared();     // Used by Cbindgen's preparation path.
let raw = JniCodec::raw();             // Also the Default policy.
let declared = JniCodec::declared(callback_outputs); // Borrowed optional context.
```

A new `Default` implementation would equal `raw()`, preserving the policy of
manual users of today's unit codec values. Raw mode
retains the base codec's current behavior, including existing opaque-handle and
inline-owner metadata lookups. It does not add the enum/data-class/error recipe
reinterpretation currently supplied by the wrappers. Declaration-aware mode
consults Flat. For undeclared types and C's ordinary mapped-error path, the
existing raw/message fallback is legitimate and must remain. “Missing prepared
metadata” means a representation is positively selected in existing Flat data
but its required prepared information/context is unavailable (for example a
selected JNI error projection); it does not mean every absent `type_info` or
C enum mapping is an error. Preserve current diagnostics and selection rules;
introducing a new declaration-intent/readiness invariant belongs to the separate
#898 readiness task. With no callback-policy set, ordinary classes/enums/error
preparation can proceed; preparing a source callback produces the same missing
callback-preparation diagnostic as `DataCodec::default()` does today.

Private helpers perform raw leaf conversion and declared enum/data/error
conversion inside **one implementation** per language. Do not replace one
forwarding wrapper with two full trait implementations for raw/declared modes.
The current preparation order is retained: class metadata and error bindings
must be ready at the same stages as today. Missing-readiness policy across all
public entry points is a separate #898 task, not permission to change it here.

C's callback-result mode affects `EnumCodec::input_leaf`; row preparation calls
that leaf and applies the existing row postprocessing. Its other structural,
sequence and optional hooks do not inspect that flag. Propagating `InputRole`
through callback-result compilation and all its children preserves this use.
If future transport behavior needs the role it must receive explicit context;
there is no new global mode or thread-local state.

The constructors, `JniCodec` lifetime parameter, trait methods, helper signatures
and removed `Codec::Niche` are breaking low-level API changes. The implementation
must document migration and follow the project's release-version policy. This
includes updating the public `&CCodec` example in `prebindgen-c/src/lib.rs` and
manual `&JniCodec` examples, rather than leaving stale unit-value calls. No
compatibility shim with a second codec pipeline is proposed.
