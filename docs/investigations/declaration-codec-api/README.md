# A smaller declaration codec API

**Proposal for human review; no generator changes are implemented by this PR.**
This addresses the third checkbox in [#898](https://github.com/milyin/prebindgen/pull/898).
The implementation checkbox stays open. The audited source is #898 at
`a2a9c8849c5e997654dc18a51d8d3dcfabd49e02`, after #899–#901 merged.

## Where `Codec` fits

There are two different users of prebindgen:

1. A binding author chooses which Rust functions/types to expose and how to
   expand them, usually in `build.rs` through `Cbindgen` or `JniGen`.
2. A language-adapter author implements how those choices become foreign
   values and generated Rust. `Codec` is principally this author's interface.

This proposal changes the second interface. A binding author continues to use
the existing declarations. For example, a getter expansion still means “observe
these fields of this Rust result”; adding `.bulk()` still means “pack that
expansion in native record storage”.

The current pipeline is:

```text
annotated Rust source
  -> Flat: source items and language declarations
  -> Input/Output recipes: selected expansions and local overrides
  -> shared declaration compiler + language Codec
  -> prepared parameter/return converters stored in Flat
  -> C or JNI emitter
  -> generated Rust and C/Kotlin bindings
```

The declaration compiler handles source structure: it finds constructors,
getters and fields, combines children, and builds the ownership/cleanup code.
The codec supplies the language's representation and conversion decisions.
Later, the emitter assembles the prepared converters around a source function
call. The converter-contract work in #900/#901 improved that **last** boundary;
this proposal improves **preparation**, where converters are created.

Some terms used below:

| Term | Meaning here |
| --- | --- |
| Leaf | One source value converted without further recipe expansion. It can produce multiple foreign values. |
| Wire | One value in a foreign representation: for example a C pointer, JNI `long`, or array carrier. |
| Plan | Prepared Rust fragments plus the wire tree, resources and ownership-commit information. `CodecInput`/`CodecOutput` are aliases for the existing plan types. |
| Group | Wire-tree structure retaining the relationship between constructor arguments, fields, or named output views. |
| Row | The foreign representation of one collection element. Its temporary storage may end after that element is constructed. |
| Bulk | Packing expanded fields in native record storage, rather than passing each through the ordinary foreign representation. |
| Niche | An unused wire value reserved to mean `None`; for example a null handle can encode an absent optional handle. |

The wire tree has `Leaf` nodes containing wires, `Chain` nodes enclosing one
child (an option, sequence or bulk frame), and `Branch` nodes grouping several
children. A parent's `controls` are its own wires, such as a presence flag or
row carrier. Its metadata explains those controls to the adapter; metadata does
not contain another copy of the child tree.

The associated types are also worth separating: `Schema` fixes the converters
and metadata stored in Flat; `Wire` describes one foreign value; `Chain` and
`Branch` describe the adapter's parent-node meanings; `Context` supplies names
such as the JNI environment when Rust fragments are emitted; `Extra` is borrowed
preparation input passed to nested conversions. They do not all represent
runtime data.

A `Condition` / `active` expression says which optional/alternative branch uses
a value. Inactive branches must not acquire its owner or decode its payload.
An **ownership receipt** reports which foreign owners Rust consumed before
success/failure; Kotlin uses it to invalidate their handles. It does not restore
consumed objects. `records_consumption` means an individual acquisition already
updates that report, so the shared compiler must not count it again.

All codec methods execute **while generating bindings**. Their returned Rust
fragments execute later, in the generated library. Changing an extension point
does not itself justify changing those runtime fragments.

## Recommendation

Keep one `Codec` interface, but make its extension points describe complete
conversion operations. Pass ordinary/bulk representation and row-construction
context to a leaf operation instead of giving each combination its own method.
Ask for one complete optional encoding and one complete source-error channel
instead of coordinating several independently overridden pieces.

The concrete proposal has **36 methods, four required**, compared with today's
55 methods, 11 required. It also removes the codec's `Niche` associated type.
The counts describe customization points, not a promised reduction in total
generator source lines. The important changes are:

- A codec implementing only scalar conversions does not implement options,
  selectors, records, collections, callbacks, or errors just to compile.
- C and JNI keep every representation, ownership, cleanup and delivery choice
  they currently need. In particular, row construction remains an explicit
  context with its existing lifetime requirements.
- Each language should have one codec implementation, with explicit raw or
  declaration-aware preparation policy. The migration must fold the private C
  `EnumCodec` and JNI `DataCodec` wrappers into that implementation, retaining
  their behavior in small helpers for enums, data classes and errors.
- Existing `expand_param!`, `expand_return!`, function declarations, prepared
  converter contracts, and the shared call assembler retain their behavior.

The proposed signatures are in [API details](api.md). The [audit](audit.md)
maps **every current method** to its proposed destination. Read the explanation
below first; neither appendix is needed to understand the problem.

## What the current API asks an adapter to implement

[`Codec`](../../../prebindgen-declarations/src/lib.rs#L660) currently has 55
methods and seven associated types. Eleven methods are mandatory. Even a codec
that only passes `i64` to/from Rust must define optional metadata, niche
selection/retirement, alternative metadata, selector construction and field
groups. Some implementations consequently consist of `None`, `Ok(())`, or `()`.
The [before/after signatures](api.md#current-baseline-for-comparison) show the
current leaf arguments and the seven unrelated method bodies a scalar codec
can drop under this proposal.

Most optional features already have sensible unsupported defaults. The proposal
preserves that useful property; it extends it to the unnecessarily mandatory
operations and consolidates several related protocols.

### One conversion appears as several entry points

For an input leaf, the compiler selects one of:

```rust,ignore
input_leaf(...)
input_row_leaf(...)
input_bulk_leaf(...)
input_bulk_row_leaf(...)
```

These distinguish two real facts: which representation the leaf uses and where
its construction must occur. The distinctions are necessary, but separate
methods make an adapter repeat signatures and dispatch logic.

For example, C's row-specific conversion records each completed ownership
transfer when a later field's construction can fail. JNI's bulk row conversion
keeps decoded buffers/enums alive through the row constructor. Replacing all
four methods with an unqualified “convert this value” would lose those rules.
In ordinary preparation, the compiler constructs a child and marks its foreign
resources `registered`, meaning their acquisition/reporting is already embedded
in that child's fragments. Grouped rows postpone that wrapping so fields can be
built together; their move sites still report completed ownership transfers.
This distinction explains the different row handling and must survive the API
change. It does not require a new staging array.

The proposal represents these facts explicitly in one request. A separate
`supports_bulk(direction)` declaration defaults to false; the **shared compiler**
rejects bulk before invoking a codec that has not opted in. A small codec does
not have to remember to reject bulk in every leaf method.

```rust,ignore
fn input_leaf(&self, request: InputLeaf<'_, Self>) -> Result<CodecInput<Self>>;

// Part of request:
representation: Representation,     // Ordinary or Bulk
construction: InputConstruction,    // EnclosingConversion or GroupedRow
role: InputRole,                    // Parameter or CallbackResult
```

`GroupedRow` means that the shared compiler is grouping leaf construction inside
one row. It does not mean “some ancestor happens to be a collection”. Nested
collections can introduce their own construction scopes. [The precise dispatch
mapping](api.md#leaf-requests) preserves today's four cases.

An output-leaf request similarly carries representation and owned/shared access.
**Explicit** output projections retain a separate `output_projected` operation:
its default policy checks and opt-in copying behavior have an independent
purpose. “The source expression is borrowed” and “the author requested a copying
conversion” remain distinct. Returned Rust references and owner-backed getter
views also retain dedicated hooks.

For a concrete example, assume the declaration expands this source record to
its fields:

```rust
pub struct Row { pub x: i64, pub y: i32 }
pub fn accept(row: Row) { /* use the record */ }
```

With ordinary expansion, the compiler asks the codec for the `i64` and `i32`
leaves, then emits construction of `Row` in Rust. With `.bulk()`, it asks for the
same two source leaves in `Bulk` representation, then packs their wire tree
through `bulk_input`. The JNI emitter later exposes the resulting frame and
controls instead of separate scalar arguments. For `Vec<Row>`, the compiler
also chooses a sequence transport and the appropriate row-construction context.
Those are separate decisions: constructing a Rust row, representing its fields,
and carrying multiple rows. The proposed requests preserve all three.

### Optional encoding is one decision spread across ten hooks

The current compiler coordinates:

- Choosing an available niche and testing whether its wire is present.
- Supplying inert values for absent output fields.
- Updating the payload's remaining niche availability.
- Describing the enclosing optional node in language metadata.
- Reading/updating a wire's niche state and producing its absent expression.

These account for ten `Codec` methods. C and JNI add forwarding methods to
shared helpers, while their own wire types already implement the representation
operations. The helpers also need the adapter's correct view of wires visible
in one native scope; blindly traversing every descendant would include fields
inside collection rows or bulk frames.

The proposed `optional(...)` operation returns a **complete encoding**: either a
separate presence flag or an existing payload wire used as a niche, the
language's optional-node metadata, and the absent output expressions. It updates
the payload's remaining availability as part of that operation.

For `Option<&Handle>`, C/JNI can retain the existing null niche: no added flag,
no copy, and no new runtime work. If no niche is available, the adapter supplies
the existing 0/1 flag representation. A codec without optional support uses the
default generation error and implements none of this protocol.

For example, a Rust `u8` uses a JNI `int` whose valid payload range is 0–255.
The adapter can reserve `-1` for `Option<u8>::None`, leaving `-2` available.
For `Option<Option<u8>>`, `-2` can mean outer `None`, `-1` means
`Some(None)`, and 0–255 mean `Some(Some(value))`. “Remaining domain” means the
unused representations still available after those reservations. The choice
must never offer `-1` again for another layer.

The shared compiler still emits the `Some`/`None` branches and coordinates
ownership commit. Adapters only choose the encoding; they do not each acquire a
second implementation of optional control flow.

### Error routing and error layout can drift apart

`result_error_value` chooses how to deliver an owned source `Err(E)`;
`source_error_layout` separately supplies its logical payload. `result_error`
provides a mapped-message fallback. These describe two halves of the **same
source-error channel**.

The proposed `source_error(...)` returns its payload layout and propagation
expression together. A typed JNI error remains typed; a user-declared mapper
can explicitly select a string message. Binding failures, such as a malformed
JNI input, retain their separate `binding_failure_layout` operation. No error
is automatically stringified. Kotlin user result/error handlers must continue
to run after the ordinary native call.

The mapped-message convenience path must remain available: the proposed
`source_error` default obtains the String layout through ordinary output
conversion and calls an optional `mapped_error` hook. An adapter that only wants
mapped errors can implement that hook without assembling a full typed channel.

### The production implementations contain forwarding layers

The base C and JNI codecs each implement 37 methods. The real preparation paths
also use [C's `EnumCodec`](../../../prebindgen-c/src/enums.rs#L32) and
[JNI's `DataCodec`](../../../prebindgen-jni/src/data.rs#L496). These add
declaration-aware behavior and forward much of the same interface to a base
codec. They are part of the extension burden, not a third language to ignore.

The migration should put their behavior into the language's single codec,
reading enum/class/error declarations from Flat. JNI's borrowed callback-policy
set remains a construction input. C's callback-result input mode becomes the
explicit `InputRole` above. These do not become another registry of binding data.

The wrappers currently separate **declaration-aware preparation** from raw leaf
conversion; they also carry callback preparation policy. They do not implement
a separate recursive record compiler. Preserving that distinction is necessary:
manual raw-codec use should keep its old policy. Declaration-aware mode keeps
normal fallback for undeclared types, while preserving errors for a positively
selected representation whose required preparation context is missing. [The migration policy](api.md#language-codec-policy)
spells this out.

## The proposed authoring experience

A minimal codec implements four operations:

| Required operation | Why it is required |
| --- | --- |
| `input_leaf(request)` | Decode an ordinary source value from its wires. |
| `output_leaf(request)` | Prepare its foreign result values. |
| `output_consumes(flat, ty, extra)` | Tell the compiler whether that leaf moves its source, before ordering getter observations and moves. This cannot be inferred after construction. |
| `group(kind)` | Preserve the relationship between combined values/fields in the wire tree, including empty groups. |

The required types stay explicit: `Schema`, `Wire`, `Context`, `Extra`, `Chain`
and `Branch`. These are existing roles; the codec no longer exposes a `Niche`
type because optional encoding owns that detail. `Wire`/metadata remain typed
per language, preserving C/JNI separation.

A default `output_consumes = true` would be conservative but would reject
multiple non-consuming scalar observations unless overridden. Requiring an
explicit answer preserves accurate move analysis. `group` is required because
the shared compiler uses combined-value groups even when preparing simple
outputs; it cannot invent arbitrary `Branch` metadata. Other group kinds can
still return an unsupported error.

Here is the shape of a deliberately small scalar codec. This is **proposed API
pseudocode**, not code available in this branch:

```rust,ignore
impl Codec for RustScalars {
    type Schema = PlanSchema<syn::Type>;
    type Wire = syn::Type;
    type Context = ();
    type Extra = ();
    type Chain = ();
    type Branch = ();

    fn input_leaf(&self, r: InputLeaf<'_, Self>) -> Result<CodecInput<Self>> {
        require_ordinary_scalar_input(&r)?;
        // Same Rust primitive on the wire and in the function.
        scalar_input_plan(r)
    }
    fn output_leaf(&self, r: OutputLeaf<'_, Self>) -> Result<CodecOutput<Self>> {
        require_ordinary_scalar_output(&r)?;
        // output_consumes == false supplies a shared observation. The
        // helper passes Shared to value::output, which dereferences the
        // scalar; it must not return an &i64 wire.
        scalar_output_plan(r)
    }
    fn output_consumes(&self, _: &Flat<Self::Schema>, _: &Type, _: &()) -> bool {
        false // Copying a scalar does not move an owned allocation.
    }
    fn group(&self, kind: GroupKind<'_>) -> Result<()> {
        match kind {
            GroupKind::Values | GroupKind::Fields | GroupKind::Field(_) => Ok(()),
            other => Err(format!("unsupported group: {other:?}")),
        }
    }
}
```

The `scalar_*`/`require_*` functions stand for the codec's own implementation;
they are not additional proposed framework hooks. A real implementation can
use today's `prebindgen_conversions::value` helpers. This example promises only
ordinary scalar leaves and structural groups, including unit returns, at the
same capability as the old minimal stub codec. Foreign-record representations,
options, collections and final foreign-call delivery need additional work.

For a richer adapter, the remaining operations cover option/alternative
encodings, sequences, bulk packing, callbacks, borrowed views, source errors,
normalization and finalization. Their defaults either reject unsupported work,
decline a specialization, or apply the same validation as today. [The complete
36-method inventory](api.md#complete-proposed-method-inventory) states which.

An unsupported operation must identify the feature and source occurrence, for
example: `publish parameter packets: bulk input conversion is unsupported`.
Adding `.bulk()` must not silently fall back to allocating ordinary objects.
Declining a borrowed-sequence specialization retains the adapter's existing
copy/borrow policy; the refactoring grants no new permission to copy.

## What changes for C and JNI

| Area | C | JNI/Kotlin |
| --- | --- | --- |
| Leaves | One request dispatches ordinary conversion and grouped-row receipt handling; bulk remains unsupported where it is unsupported today. | One request selects JNI values or native bulk fields, with the existing row-local lifetime handling. |
| Options | The current pointer/enum/scalar niches, flags and defaults are prepared together. | The current null/integer niches, native-buffer layouts and flags are prepared together. |
| Collections | Typed C rows, callback delivery and native Vec/String owners retain their existing paths. | Ordinary rows, bulk materialization and per-element callback delivery retain their existing choices. |
| Source errors | Structured error declarations and explicit message mapping produce one error-channel plan. | Typed domain-error output and the separate binding-error handler remain intact. |
| Borrowed outputs | Address-preserving references, buffers and owner-backed getter views keep their dedicated operations. | Existing borrowing restrictions and explicit copies remain unchanged. |

The public language codec constructors need a deliberate migration: a new
`Default` implementation and `raw()` would retain today's manual raw-codec policy; language builders explicitly
select declaration-aware preparation. `JniCodec<'a>` can borrow callback policy
without another codec implementation. Existing unit-value uses of `CCodec` /
`JniCodec` become `CCodec::raw()` / `JniCodec::raw()` (or `::default()`). These and
the trait changes are **breaking low-level API changes**, requiring a migration
note and the appropriate release version policy. The high-level
`Cbindgen`/`JniGen` declaration API need not change.

## Constraints and tradeoffs

The proposal consolidates decisions; it does not eliminate the complexity of a
feature-rich adapter. Optional encodings are the biggest tradeoff: an adapter
overrides a complete operation instead of a handful of its substeps. The
existing niche algorithms should remain reusable utilities, rewritten to take
candidate wires and adapter wire-encoding functions rather than a second codec
interface. A niche-capable codec still supplies those wire semantics; they have
not disappeared. A flag-only codec and a codec without options need none of
them. [API details](api.md#reusing-the-niche-algorithms) sketches the helper
contract and its remaining cost. The public `Codec` has no extra required
provider traits. Optional
metadata remains on its canonical wire-tree node, not in an accompanying table.

Input/output plans and their staged Rust fragments remain unchanged. In
particular, this PR does not reinterpret `validation`, change
`Resource::registered`, reorder ownership acquisition, or fix mixed eager and
deferred inputs. Those tasks remain in #898. Separating them keeps the API
proposal reviewable against existing runtime behavior.

C continues to trust its callers. JNI continues to use Kotlin handle preflight
without repeating alias checks in Rust. No new runtime dispatch, staging
collections, copies, locks or validation are justified by this refactoring.
All new request/encoding objects exist during binding generation only.

More request/result types will be introduced. A smaller method count alone is
not sufficient evidence of improvement: the implementation must also remove
production forwarding and make a minimal codec substantially shorter. If a
prototype merely moves 55 decisions into one large enum or helper trait, it
does not meet this proposal's goal.

## Implementation and acceptance plan

1. Establish a working scalar codec baseline and unsupported-feature examples.
   Preserve current C/JNI generated artifacts before changing interfaces.
2. Introduce leaf/sequence requests and migrate **all** production and test codec
   implementations. Retain the old representation and construction dispatch.
3. Migrate groups, complete option/alternative encodings and source-error plans.
   Rewrite the reusable optional helpers; remove superseded `Codec` methods.
4. Fold the enum/data forwarding codecs into the language codecs. Remove the
   wrappers, rather than maintaining an old and a new path.
5. Exercise the minimal codec and representative records, nested options,
   selectors, ordinary/bulk sequences, borrowed views, callbacks and typed errors.
   Include malformed and unsupported declarations and retry after failure.

Measure the minimal implementation's method bodies and lines before/after,
count the forwarding bodies removed from each production wrapper, and report
the remaining representation/row-context branches. The minimum acceptance is:
remove seven mandatory unrelated method bodies from the scalar baseline;
remove both complete-trait forwarding wrapper implementations; and introduce
no duplicated optional control-flow or collection-construction algorithms.
Request/result type definitions count toward total API size, not toward the
minimal codec's implementation. A prototype must report both totals.

Each implementation stage should build and use one authoritative compiler path.
Staging the commits is useful; permanently exposing both interfaces is not.
Verify complete existing C/Rust/Kotlin generated output, including names, ABI,
ownership receipts, cleanup and explicit return-delivery policies. Run the real
C/Kotlin/Java boundary suite and all applicable CI. If generated hot-path code
changes, explain and measure that change before accepting it. No runtime speedup
is claimed or expected from the API reorganization itself.

## Decisions requested from the reviewer

The proposed direction is concrete enough to approve or reject:

1. One codec, with representation/construction context on leaf and sequence
   requests, instead of a trait per optional feature.
2. Complete optional encoding, alternative metadata prepared from an already
   named selector, and one source-error plan with mapped-message fallback,
   following the invariants in [API details](api.md).
3. Migration of both languages and removal of their forwarding wrappers, while
   preserving the existing prepared model and runtime policies.

The main review question is whether these are the right operations for an
adapter author. The [audit](audit.md) makes the disposition of every old hook
explicit so no behavior disappears behind the smaller interface.
