# Parse once, consume elements everywhere — integration map

Umbrella for making every component of prebindgen consume [`core::flat`]'s
`Element`s instead of parsing captured Rust itself.

[#211](https://github.com/milyin/prebindgen/issues/211) remains the authority on
the invariants and the frontend/adapter boundary. This document does not restate
them; it records the design this program follows, what has landed, and the order.

This file is the one place stage state is edited. Change the doc, then re-sync
the umbrella PR body — never the other way round.

## The design

```
Source(s) ──items──> Flat ──Elements──> Registry ──> adapters
  raw records          parse +           projects       classify off `kind`
  (syn::Item)          resolve           the model      spell off `origin`
```

An `Element` is two things at once, and the pairing is the whole point:

* a **closed model** — `TypeKind`, the field list, which of the two enum shapes
  an item is. For a **type** that model is the accepted subset of `syn::Type`
  and nothing more (see *A type is its syntax* below); above the type level it
  is the concept — a field list, a sum's alternatives;
* one `Origin`, carrying the **exact syntax** the node was built from and the
  source it arrived in. Every node has one, at every level — item, parameter,
  field, alternative, type, array extent.

```rust
pub struct Origin<S>  { pub syntax: S, pub location: Rc<SourceLocation> }
pub struct TypeRef    { pub kind: TypeKind, pub origin: Origin<syn::Type> }
pub struct Param      { pub name: syn::Ident, pub ty: TypeRef, pub origin: Origin<syn::PatType> }
```

The two enum shapes are separate entities, because they are numbered differently
and consumed as different constructs:

```rust
pub struct Variant { pub name: syn::Ident, pub alternatives: Vec<Alternative>, .. }  // a sum
pub struct Enum    { pub name: syn::Ident, pub values: Vec<EnumValue>, .. }          // C-style
```

### A type is its syntax

`TypeKind` began as a **destination-neutral classification**: one variant per
concept a target language would act on, several Rust spellings folding into
each. `String` and `str` were one `Str`; `Vec<T>` and `[T]` one `Sequence`;
`Box<T>` and `Cow<'_, T>` disappeared into whatever they wrapped.

It leaked, and the leak was not at the edges:

* `&T` earned a layer of its own while `Box<T>` was declared transparent — two
  wrappers, opposite treatments, on no principle either adapter shared;
* `Cbindgen` picked its C type off the Rust spelling regardless, so the
  neutrality the kind claimed was not what any adapter used;
* every fold had to be *undone* somewhere. `erased_wrappers()` and
  `stripped_syntax()` exist because the model dropped something a consumer
  needed back.

So `TypeKind` is now the **subset of `syn::Type` the flat API accepts**, and
nothing else. One variant per accepted form: `Str` and `String`, `Vec` and
`Slice`, `Boxed`, `Cow`, `Uninit`, `Ref { lifetime, mutable }`, `Named { id,
args }`. A lifetime and a generic argument are kept, because they are what the
source wrote.

**The folds did not disappear — they moved to where they are decided**, and are
one shared reading each rather than a property of the classification:

| The reading | Answers | Replaces |
|---|---|---|
| `TypeRef::unwrapped()` | `Box`/`Cow` peeled to the value | the erasure in `lower_path` |
| `TypeRef::sequence_elem()` | the element of `Vec<T>`, `[T]`, or either behind a wrapper | `TypeKind::Sequence` |
| `TypeRef::borrow_target()` | what a borrow points at, past an out-parameter's slot | `RefMode::Out`'s absorption |
| `TypeRef::is_exclusive_borrow()` | `&mut T`, and not `&mut MaybeUninit<T>` | `RefMode::Exclusive` |

What this buys is one property, and it is the point:

> **The syntax is recoverable from the kind.** `TypeKind::to_syn()`, checked
> against `TypeRef::origin.syntax` over the whole acceptance corpus
> (`syntax_is_recoverable_from_kind`), with two named exemptions: a callback's
> bound *order*, and a `Group`/`Paren` the lowering sees through.

The slice still rides along and generated Rust still spells it — it is exact and
free. It is no longer *load-bearing*, and that is the difference: a fact missing
from `kind` used to be invisible, because the syntax was there to cover for it.

**Did not move**: every generated artifact byte-identical.

### Why the syntax rides along

The predecessor design ([#215](https://github.com/milyin/prebindgen/issues/215))
built a **syn-free** semantic model and kept hitting one wall: the generated Rust
glue is itself a destination artifact, and it is the only consumer that needs
syntax fidelity. Each time it did, the answer was to model the syntax —
`DiscriminantSource::Explicit(syn::Expr)`, `syn::Member`, `syn::Lifetime`,
`to_syn()`, and finally a `VariantShape` whose only job was to make generated
Rust spell `E::B()` instead of `E::B`. A model that carries no syntax has to
become *lossless* to serve that consumer, which is how a language-neutral IR
turns back into a second `syn`.

Carrying the original slice costs nothing and removes the pressure, so the
classification stays small and genuinely neutral:

| Fact | Where it lives | Who reads it |
|---|---|---|
| `B()` vs `B` | `Alternative::origin.syntax` (via `spell::fields`) | generated Rust only |
| `= 0x07` vs `= 7` | `EnumValue::origin.syntax.discriminant` | a C mirror re-emits it |
| the number 7 | `EnumValue::discriminant` | Kotlin `NAME(7)`, `jint` decode |
| which alternative of a sum | `Alternative::index` | a sum has no Rust number to borrow |
| `Foo<'a, T>` | `TypeRef::origin.syntax` | generated Rust only |
| "it is a `Foo`" | `TypeKind::Named` | every adapter |
| `[u8; TAG_LEN]` — spelling / number / const identity | `TypeRef::origin.syntax` / `ArrayExtent::value` / `ExtentSource::Const` | C header / Kotlin / both |
| the `Box` in `Box<Option<T>>`, and the `Option<T>` under it | `TypeKind::Boxed` — kept, and read through by `TypeRef::unwrapped()` / `erased_wrappers()` | everyone: a classifier unwraps, an emitter that **rebuilds or destructures** puts it back |
| where an item came from | `Origin::location` — **absent for a synthesized one** | diagnostics |

### The rule

> **Classify off `kind`, spell off `syntax`.**
>
> Matching a `syn::Type` or `syn::Expr` variant outside `core::flat` is a
> classifier, and #211 says classification lives there alone. Passing an
> `Origin`'s syntax into `quote!` is spelling, and spelling the source is exactly
> what generated Rust must do.

#### What the split does *not* say: who decides the destination type

"Classify off `kind`, spell off `syntax`" tells an adapter where to get each
fact. It is silent on the question adapters actually face, which is what the
**destination language** ends up seeing. The companion rule:

> **Same `kind` ⇒ same destination-language type.** The *wire* is the
> generator's to choose, and may differ per spelling.

The weaker-sounding half is the important one. It is tempting to write "same
`kind` ⇒ same wire", and that is **false** — prebindgen violates it deliberately:

| Rust | `kind` | Kotlin type | wire |
|---|---|---|---|
| `&[Payload]` | `Ref(Slice)` | `List<Payload>` | `Long` — a jlong handle to a Rust-side `Vec` |
| `Vec<Box<Payload>>` | `Vec(Boxed)` | `List<Payload>` | `List<Payload>` — a `JObject` |

Two wires, one surface. Choosing a wire is exactly the generator's job, and the
destination-language wrapper absorbs the difference; a caller cannot tell. What
a caller *can* tell — and what the shared `unwrapped()` reading is there to
prevent — is the **type** changing because the source spelled a `Box`.

The rule scopes to **converted** positions, which is where a converter stands
between the Rust value and the destination and is free to bridge. It cannot apply
to a **layout mirror**: `Cbindgen`'s `repr_c_struct` crosses a struct zero-copy,
so the C struct is reinterpreted from the source struct's bytes and its field
types are a *layout* fact. There `Box<T>` (a pointer) genuinely is a different C
type from `T` (inline), the spelling is load-bearing by construction, and no
erasure can apply. A mirror reads `syntax` for the destination type on purpose —
the one place the usual split inverts, and it inverts because the contract is
layout rather than surface.

Reusing a mirror's spelling test in a converted position is how the rule gets
broken. A tagged-union payload is converted, and it used to take its
opaque-pointer arm from the `Box` in the spelling: `Option<Box<Handle>>` crossed
as `handle_t *` while `Option<Handle>` — the same optional handle to every
destination — was **refused outright**. An erased wrapper decided whether the
shape was expressible. It asks the declaration now, and the two spellings share
one C type with different converter bodies.

This is mechanically measured, and needed no new mechanism:
`core::flat::boundary` (ported from
[#224](https://github.com/milyin/prebindgen/pull/224)) counts *variant mentions*
of watched syn enums per file, so `quote!(#slice)` is invisible to it while
`matches!(ty, syn::Type::Reference(_))` is counted. The committed ledger is the
scoreboard for this whole program.

## Size of the problem

Seeded by L0 at **202 classification sites** outside the frontend, split
`api/core` 71, `cbindgen` 25, `jnigen` 106. The second population it was seeded
alongside — **113** reads of the registry's `syn`-keyed item maps — is **gone**:
L1.5 deleted those maps, so every one of those reads now goes through the model.

Those are the numbers this document keeps, because a seed is a fixed fact. **The
current count is in `boundary.ledger`**, which is generated, and its stage-by-stage
history is in [#229](https://github.com/milyin/prebindgen/pull/229). A table of
live counts copied into prose here would be wrong after the next merge, and was.

Two things the falling count has taught, which the count itself does not show:

**A site leaving is not the same as a site migrating.** The largest single drop was
[#248](https://github.com/milyin/prebindgen/pull/248) deleting a pattern engine
whose tables held one entry in the whole crate. Nothing was migrated to read an
element; the code holding the sites went away. Both are real progress, and a stage
that does not say which one it achieved is not reporting.

Not every site must go: some inspect types the adapter itself *synthesized* —
wire types, converter signatures — which is legitimately the adapter's business.
Separating the two populations is not a document to write up front; it is each
entry's fate as it comes off the ledger, with a stated reason in the PR that
moves it.

## Stages

| Stage | Owns | State |
|---|---|---|
| L0 | The parser, `Element`, and the ledger | **done** — [#227](https://github.com/milyin/prebindgen/pull/227) |
| L0.5 | `Flat`: the model, indexed and resolved | **done** — this branch |
| L1 | `Registry` consumes elements | **done** — [#238](https://github.com/milyin/prebindgen/pull/238) |
| L1.5 | The model is the only index | **done** — #239–#246 |
| L1.75 | The registry becomes describable | **done** — #249–#253, squashed into #248's commit |
| L2 | `api/core` stops classifying source syntax | **done** — #248, #257, #258, #261, #263 |
| L3 | `Cbindgen` consumes elements | not started |
| L4 | `JniGen` consumes elements *(the long pole)* | not started |
| L5 | Close the seam: the public contract stops being `syn` | not started |

### L0 — the parser — **done** (#227)

- [x] One parse over any `(syn::Item, SourceLocation)` stream — the seam
      `Registry::from_items` occupies, so multi-source composition is unchanged
- [x] `Element` per modelled kind plus `Unsupported`, every element and component
      carrying the syntax it was built from. (There is **no** verbatim-passthrough
      variant: the proc-macro refuses to mark a `use`/`mod`/`macro_rules!`, so
      nothing reached one. The exact variant list is L0.5's, below.)
- [x] A type reference is a classification plus its syntax; lowering total over
      the accepted grammar
- [x] The array-length subgrammar and `ArrayExtent`, ported from #212
- [x] Enum tag / discriminant numbering, ported from #226, with `checked_add`
- [x] Round-trip tests: syntax slices are the source's tokens, including the
      cases a reconstruction loses (empty delimiters, `0x07`, lifetimes, docs)
- [x] Acceptance matrix: spelling → element, or a diagnosis naming the item
      **and** the component
- [x] Boundary ledger ported (#224) and seeded

**Acceptance is preserved, not expanded.** An item the language cannot express
becomes `Element::Unsupported` carrying its diagnosis, because the pipeline has
always scanned a signature only once an adapter declared it, and a source crate
may mark items no binding uses. Only a duplicate name — which no declaration can
disambiguate — fails the parse. Tuple-struct fields stay unmodelled for the same
reason.

### L0.5 — `Flat`: the model, indexed and resolved — **done**

L0 produced a `Vec<Element>`, which nobody could ask anything. This stage makes it
a model, and takes two bullets off L1 in the process.

- [x] `core::language` → `core::flat`, `Language` → `Flat`: the thing being
      modelled is the **flat API**
- [x] `Element = Function | Type | Constant | Unsupported`, with `Struct`,
      `Variant`, `Enum` and `Opaque` under `Type`; the type *reference* becomes
      `TypeRef`
- [x] `Opaque` is an entity, declared by `#[prebindgen] pub type X = ..` — the way
      a foreign or crate-private handle gets a **name** in the flat API. This is
      the prerequisite for everything below it
- [x] `FlatBuilder` collects, `Flat` answers by name: `function`,
      `declared_type`, `constant`, `element`, the per-kind iterators, `resolve`
- [x] **References resolve at parse time.** An item naming an undeclared type is
      `Element::Unsupported` with `ItemError::UnresolvedType` — so a dangling name
      is reported here, by name, instead of surfacing downstream as an unresolved
      *converter* from whichever adapter looked first
- [x] `&mut MaybeUninit<T>` is modelled — first as `RefMode::Out`, then (see
      *A type is its syntax*) as the two forms the source wrote, with
      `borrow_target()` as the reading that sees past the slot
- [x] The example flat APIs are closed, and covertest-kotlin's build script
      asserts they stay closed across both its sources
- [x] **Did not move**: every generated artifact byte-identical

`Cow<'_, T>` needed neither an alias nor a grammar addition in the end: both
adapters already treat it as the `Vec<T>` it borrows
([#236](https://github.com/milyin/prebindgen/pull/236)). It first landed *as*
that reading — lowered to whatever `T` is — and is now a `TypeKind::Cow` that
`unwrapped()` reads through, which is the same behaviour with the fold moved to
where it is decided.

**Still open**: `zenoh-flat` and its two consumers are separate repos. Their
unmarked types — the 26 zenoh aliases, plus `Duration`, which is not in the
prelude and so needs a marked alias like any other foreign type — need the same
treatment before they parse.

### L1 — `Registry` consumes elements — **done**

The seam that makes the direction real. Adapters were not touched.

- [x] `Registry::from_flat(Flat)`; `from_items` is `Flat::builder` + `from_flat`,
      so both entry points share one parser
- [x] The registry **holds** the model (`registry.flat()`), which is how L2–L4
      reach it: an adapter already has the registry
- [x] The maps are a projection of the elements — plus synthesis, since `resolve`
      injects adapter-declared binding-local fns into `functions`
- [x] `scan_fn_signature`'s receiver / parameter-pattern / `impl Trait` guards
      deleted with their `ScanError` variants, along with `index_item`,
      `check_no_duplicate` and `first_seen_loc`: `Flat` owns indexing and
      duplicate detection
- [x] `ParseError::DuplicateName` carries both crate names, so one authority
      produces the message
- [x] **Did not move**: every generated artifact byte-identical

**Correctness is checked by default**, superseding L0's "inert until declared":
ingestion fails on anything the language cannot express, listing every offender at
once so a source crate needing migration sees one list. An opt-out for
deliberately-unsupported elements is #237.

The cost landed in test fixtures: 167 of 524 tests held an item naming a type they
never declared. `test_util::declare_referenced` supplies a marked alias for those
where the handle is incidental; the rest were real corrections — a path-qualified
`std::time::Duration` that no declaration can name, and two array-length tests
asserting shapes the subgrammar dropped in #212.

**Still open**: `zenoh-flat`'s 26 unmarked aliases. Until they are marked,
`zenoh-flat-c` and `zenoh-flat-jni` do not generate.

### L1.5 — the model is the only index — **done**

L1 made the registry a *projection* of the model, but it still kept its own copies.
A projection that copies is two stores that can disagree, so this stage deleted the
copies. Not planned as a stage; it fell out of reviewing L1 and is recorded here
because the map should show where the program actually went.

- [x] **The seven fields go** ([#243](https://github.com/milyin/prebindgen/pull/243)):
      `functions`, `structs`, `enums`, `consts`, `guards`, `item_origins`,
      `source_modules`. `Flat` grows `struct_type` / `enum_item` /
      `source_modules`, and the registry answers `origin_module`,
      `default_module`, `named_item_idents` off the model. The
      `SourceLocation` half of every deleted map entry was provably **dead** — all
      44 `.get()` sites bound it to `_`
- [x] **Binding-local fns join the model**, lowered through the same grammar
      (`Flat::lower_signature`) and admitted by `add_local_function` — otherwise
      "one index" would be a lie, since a `sig!(..)` never passed through the parser
- [x] **The type table carries the reading**
      ([#239](https://github.com/milyin/prebindgen/pull/239)): a cell is
      `TypeCell { subject, root, entry }`, the subject being the frontend's
      `TypeRef`. `required` stopped being stored — it was one name over three
      storages — and is derived by `resolve`. The subject was originally a
      two-variant `TypeSubject`, the second variant meaning *"a type only the
      binding authored, with no reading"*; L2 found that population empty and
      deleted it, so every cell now carries a reading
- [x] **`const _` is a `Guard`, not a `Constant`**
      ([#240](https://github.com/milyin/prebindgen/pull/240)): an anonymous const
      has no address, so it is not API. Four sentinel `ident == "_"` checks had
      already gone dead without anyone noticing — the failure mode a sentinel invites
- [x] **A lookup takes the name the caller holds**
      ([#244](https://github.com/milyin/prebindgen/pull/244)): the sealed `Name`
      trait, because `Ident` hashes via `to_string()` and has no `Borrow<str>` —
      the allocation can be *moved*, never removed
- [x] **An alias is a declaration of its name**
      ([#245](https://github.com/milyin/prebindgen/pull/245)): the two type
      diagnostics had excluded `Extern` as an artefact of asking the old
      `structs`/`enums` maps, which had nowhere to put one
- [x] **`Flat` owns the type index**
      ([#246](https://github.com/milyin/prebindgen/pull/246)): the last index
      living outside its owner. `from_flat` collapses to *check expressibility,
      store the model*. Canonicalization becomes one definition
      (`canonical_type`, moved into `core::flat::spelling` by L2) that both the
      index and `TypeKey` derive from
- [x] **A reading and a reportable position are different facts**: a synthesized
      signature has readings but no file, so `SourceLocation::has_position` gates
      what diagnostics print. Fixed a pre-existing `:0:0:` for hand-built streams
      as well

**What is left in `Registry` is now genuinely its own**: the two type tables
(adapter answers plus roots) and the five adapter-declared plan maps.

### L1.75 — the registry becomes describable — **done**

Also not planned as a stage, and it moves no ledger sites — the count is 167
before it and 167 after. It is here for the same reason L1.5 is: once L1.5 made
the registry a projection with nothing of its own to hide, its API could be
closed, and closing it is what makes a generator for a fourth language writable
by someone who has not read `resolve`. Tracked by
[#251](https://github.com/milyin/prebindgen/issues/251).

- [x] **The caller states its declarations; the registry stops asking**
      ([#249](https://github.com/milyin/prebindgen/pull/249)) — the five
      decomposition callbacks become one handed-over value
- [x] **Say what the registry is for**
      ([#250](https://github.com/milyin/prebindgen/pull/250)): *which type
      conversions a binding needs, and whether it has them all.* Its module doc
      had been a list of fields, and a stale one since #243 deleted them
- [x] **State the shape, then build it**
      ([#252](https://github.com/milyin/prebindgen/pull/252)): `RegistryBuilder`
      and `Registry` are two types because being-described and finished are two
      states. 13 `Prebindgen` hooks called from 9 points inside `resolve` become
      `describe, hand over the answers, read it`. **Nothing calls back into the
      generator** — not by trait hook, and not by a `next_request`/`supply` pull
      loop, which is the same protocol with the arrow reversed
- [x] **The generator owns the model and the registry**
      ([#253](https://github.com/milyin/prebindgen/pull/253)): a build script
      names one type. `JniGen::builder().source(..).build()` replaces the
      `Flat::builder()` → `Registry::builder()` → `resolve` → `write_*` dance;
      `Flat` and `Registry` stop being names a `build.rs` has to know

**All of it is on this branch, in one commit.** The stack landed PR-into-PR onto
`flat-drop-pattern-engine`, and #248 squash-merged that branch afterwards, so
`d845c8f` — titled for the pattern engine — carries the registry and generator
redesign too. Do not read the commit log as the inventory: `flat-drop-pattern-engine`
still reports 28 commits ahead of `language-integration` because a squash records
no ancestry, while the trees differ by nothing. Diff the content, not the history.

### L2 — `api/core` stops classifying source syntax — **done**

- [x] **The pattern engine is deleted**
      ([#248](https://github.com/milyin/prebindgen/pull/248)): `match_pattern`,
      `unify`, `immediate_pattern_children`, `substitute_wildcards`, both rank
      tables. The general machinery composed converters for any parametrized type;
      its tables held **one** entry in the whole crate, `Result<_, _>`, which the
      model already names `TypeKind::Fallible`. 592 deletions against 124
      insertions, and the `ConverterImpl` tail extracted verbatim rather than
      rewritten. Ledger 202 → 167
- [x] **The scan walks the model's edges**
      ([#257](https://github.com/milyin/prebindgen/pull/257)): `registry/walk.rs`
      is deleted and `immediate_edges` takes its children from `TypeKind`. Three of
      its arms were dead rather than migrated — the grammar refuses non-unit tuples
      and raw pointers, and `Group`/`Paren` are transparent. Ledger 167 → 158
- [x] **A composed type is classified where it enters, and the answer is kept**:
      expansion builds spellings the source never wrote, so `ensure_entry` asks the
      grammar once, when a cell is born, and stores the reading **in that cell**.
      `Flat` is consulted, never extended — its index means *what the source
      wrote*, and a wire-side intermediate is not that
- [x] **Every cell carries a reading**: with the above, the "no reading" half of
      `TypeSubject` had no members left (measured: zero refusals across every
      in-tree example and the whole suite), so the enum is gone. A spelling the
      grammar really does refuse is now a reported error naming it, rather than a
      cell that quietly means less than its neighbours
- [x] **Spelling moves to its owner**: `canonical_type`, `normalize_type`,
      `type_from_ident` and the rest become `core::flat::spelling`. They decide what
      spelling a type *has* before anything keys on it — the same authority that
      decides what it *means*. Ledger 158 → 154
- [x] **One layer read** ([#261](https://github.com/milyin/prebindgen/pull/261)):
      the twenty sites in `unfold` and `expand` that peeled `Option`, then `Vec`,
      then `&` by taking a spelling apart now read the model's arity stack.
      Ledger 154 → 135
- [x] **The reading is carried, not re-derived**
      ([#263](https://github.com/milyin/prebindgen/pull/263)): fourteen sites still
      reached into `origin.syntax` for a fact the element already held — a
      `Function::ret` that is a `TypeRef`, callback arguments that are `TypeRef`s.
      The helpers now take `&TypeRef`, so the round trip does not compile. **The
      ledger did not move**, which is the finding, not a footnote — see below

**#248 is deletion, not migration**, and the distinction is worth keeping visible:
35 sites left because their code left. The same caveat applies to the spelling
move, which is a **move**. Only the last item above is a migration in the full
sense, and it is the one that took the most arguing.

#### What L2 taught: a peel must match what the consumer can build

Three defects in the layer read, all one root — a peel that answered more than its
caller could represent — and none of them visible to the evidence this programme
usually relies on. The suite passed and regen stayed byte-identical through all
three, because no in-tree example exercises the shapes involved.

- **`Vec<T>` matched a `T` constructor.** Expansion builds one value; its plan
  shape has no iterable arm. A peel that removed the `Sequence` anyway made a
  `Vec<T>` parameter match a `T` constructor, and the wrapper would have handed one
  reconstructed `T` to a parameter expecting the collection.
- **The stack recursed past its own contract.** `Vec<Option<T>>` read as an
  optional inside a run, so a return matched a decomposition target `T` and
  installed a fold — for a type the explicit path next to it refuses outright. Two
  paths disagreeing about one return, the silent one winning.
- **`Layers` was a fourth copy** of `core::shape::Shape`, whose own module doc says
  it replaced three. Encoded as flags, so a caller could only *ignore* a layer it
  could not build; the stack lets it **decline** by not matching.

What came out of it is the rule, and it outlives the stage: **the peel is chosen by
the consumer's capability, not by the type's structure.** `TypeRef` therefore
offers both — `layer_stack` for a consumer that implements every layer, and
`optional_inner` / `sequence_elem` / `borrow_target` for one that composes exactly
what it can honour.

#### What L2 taught twice: the ledger measures the wrong thing for this

The fourth defect was the measurement itself, and it is the one worth carrying
furthest. L2 was first reported done on the strength of the count falling 154 → 135.
Then a review pointed at this, which had survived all of it:

```rust
let item_fn = flat.function(&f).map(|f| f.origin.syntax.clone())?;
let ret = fn_return(&item_fn);        // dig the return out of raw syntax
returns_type(registry, &ret, &key)    // -> classify() -> re-lower it
```

`Function::ret` is **already** a `TypeRef` with `kind` computed at parse time. The
model handed the answer over; the code reached into `origin` and derived it again —
in six places, with five more re-extracting callback arguments the model held as
`TypeRef`s, and three digging parameters out of a cloned `ItemFn`.

The ledger could not see any of it. It counts **variant mentions of watched syn
enums per file, outside `core::flat`**, so moving a match into one shared classifier
drops the count without changing the data flow. Both facts are real, and they are
different facts:

> The ledger measures **who matches syn variants**. It does not measure **who
> reasons from `origin`**. Those came apart the moment the matching moved into one
> place, and only the second is what #211 asks for.

The fix is a signature rather than a checker — `peel`, `peel_borrow` and
`returns_type` take a `&TypeRef`, so a caller must already hold a reading and the
round trip does not compile. `Flat::classify` belongs to the registry, which is the
authority on what a type means because it is the thing that **stores** readings.
`origin.syntax` is read only where a value is stored for emission.

**So a count is a proxy, and this one has a known blind spot.** A stage that reports
only its delta is reporting the proxy. Where a rule can be made structural, it
should be — the deltas L3 and L4 report are worth exactly as much as the invariants
they can point at underneath them.

#### Where `api/core` ends, and why it is not zero

**13 sites**: `types_util` 10, `registry/scan` 2, `unfold` 1.

Every classifying helper still in `types_util` is called overwhelmingly from the
adapters — `option_inner_type` 40 times, `bare_path_ident` 22, `is_unit` 18 — and
none takes the model as an argument, so it cannot consult it from the inside. L2
stopped `api/core` from *calling* them; only L3 and L4 can free them to be deleted.
`unfold`'s one is `peel_ref`, in the same position with three jnigen callers.

The two in `registry/scan` are different and stay for good: they inspect a key a
**build-script author** wrote, to diagnose that spelling — no source type is being
classified, so there is no element to read instead. They are the first entries to
land in the *"legitimately the adapter's business"* category this document predicts.

### L3 — `Cbindgen` consumes elements

- [ ] `builder` (8), `trait_impl` (6), `emit` (5), `mod` (5), `convert` (1)
- [ ] Variant patterns and constructors come from `Variant::spell`, not from
      re-deriving delimiters
- [ ] A discriminant is re-emitted from `Variant::syntax`, and the number comes
      from `Variant::discriminant`
- [ ] Generated C artifacts byte-identical

### L4 — `JniGen` consumes elements

The long pole — 97 sites, down from 106 because #248 took `jni/builder` from 13 to
4 with the rank tables. Split by area, each PR independently green.

- [ ] `emit/names` (17), `jni/trait_impl` (11), `emit/wrapper` (11),
      `emit/flat_input` (10), `render` (8), `selector` (7), `iface` (5),
      `jni/builder` (4), and the rest
- [x] `classify.rs` — a whole classifier with **zero** watched sites, so the
      ledger cannot see it: it must be migrated on its own merit. Its one leak
      (`DataStruct { st: &syn::ItemStruct }`) closed with #267, which needed it:
      a field record cannot carry its own reading while the walk is handed a
      `syn::ItemStruct`
- [ ] `prim_array_of` reads `ArrayExtent` instead of re-matching `Type::Array`
- [ ] Generated Rust and Kotlin byte-identical

#### What L4 taught: an erasure sits outside the layer it wraps

Reading through `Box` and `Cow` is right — `Box<Option<T>>` is one optional to
every destination. But **conversion follows the syntax**, and the two facts a
rebuild needs were not on the model at the time: what was taken off, and what is
left under it. #292 added them as derived readings, `TypeRef::erased_wrappers()`
and `stripped_syntax()`, defined by an invariant rather than by a loop — the
stripped spelling is *the one whose own lowering yields the kind `unwrapped()`
reaches*, so the peel runs to a fixed point (`Box<Box<T>>` unwraps to `T`, and
one strip leaves a `Box<T>` that does not match).

Both readings survived *A type is its syntax* unchanged, computed off the kind
rather than off the spelling. The lesson below is the reason the fold had to
become a reading in the first place.

The rule, which outlives the stage:

> **The unwrapped reading is precisely the thing the wrapper is missing from, so
> taking it before checking for a wrapper always discards one.**

`Box<&Vec<T>>` *reads* as a `Ref`; take that reading first and the wrapper is
gone from everywhere a consumer will look. `&Box<Vec<T>>` hides it on the referent, where a
question asked of the outer `syn::Type::Reference` cannot see it. Neither check
subsumes the other, so a walk must ask at **every layer, on the way down** —
which is also why the wrapper is a *list*, gathered as the walk descends.

The audit that came with it found the population is two, and only one has to ask:
a site that **classifies** must never consult the wrapper — that is the erasure
working, and every cbindgen site and all but one jnigen site are of that kind. A
site that **binds a source value and destructures or rebuilds it** must. There
was exactly one unguarded instance, and it was a live miscompilation: builder
delivery bound the returned value and matched it against `Option`'s patterns,
which match ergonomics does not see through a `Box`. Fixed at the single point
the value enters the delivery, not at each of the four matches downstream.

Two things it taught about evidence:

* **#290's guards were hand-maintained and wrong twice in one PR.** "Are all the
  peel sites guarded?" is answered by inspection until the model carries the
  facts and one shared helper consumes them.
* **The suite could not see the defect.** 737 `contains(..)` assertions pass on
  Rust that does not compile (#269). The fixture that proves this one is in
  `perftest-flat`, whose generated binding covertest `include!`s and **builds** —
  the only place in the tree where an `E0308` is a test failure. It was verified
  by disabling the fix and watching the build break.

### L5 — close the seam

The public contract stops being `syn`, which is what stops the population from
growing back.

- [x] `Registry`'s public item maps stop being the adapter-facing contract —
      done early by L1.5, which deleted them outright; relates to
      [#92](https://github.com/milyin/prebindgen/issues/92)
- [x] `Prebindgen::on_function` / `on_struct` / `on_enum` / `on_const` take
      **elements**, not `syn` items — the item methods were the widest part of
      the public `syn` surface, and the one that decided what adapters could
      know. An adapter handed a `flat::Function` cannot ask what a parameter
      means and be told "no reading"; a `&syn::ItemFn` gave no such guarantee.
      `on_enum` split into `on_variant` + `on_enum` along the model's own
      distinction. Done as #275's first half rather than waiting for this stage,
      because it was the last thing keeping the spelling accessors alive
- [x] **What a spelling adds over its classification is the model's answer, not
      a peel each adapter writes** (#292): `erased_wrappers()` / `stripped_syntax()`.
      Belongs here because the alternative was every rebuilding emitter taking a
      `syn::Type` apart for itself, which is the population growing back — the
      completion criterion below already forbids reconstructing a spelling from a
      classification, and this is the fact that makes obeying it possible
- [ ] `Prebindgen::post_process_item(&mut syn::Item)` — the hook that let
      qualification live in an adapter in the first place
- [ ] `ConverterImpl::function` / `TypeEntry::function` as `syn::ItemFn`;
      `prerequisites` / `local_functions` returning raw items
- [ ] `Niches { value: syn::Expr, matches: syn::Expr }` — a semantic fact carried
      as raw expression syntax
- [ ] Extend the ledger's `WATCHED` beyond `Type` / `Expr` — `Item`, `Fields`,
      `FnArg`, `ReturnType`, `GenericArgument`, `Pat` — one enum at a time, each
      addition a regenerated ledger whose diff *is* the decision
- [ ] Close or accept the blind spots the ledger header lists (token-string
      classification, ident-name classification, helper delegation)

## Completion criteria

#211's, restated for this design:

- One documented entry point from captured records to elements —
  `Flat::builder().items(..).build()`, which `Registry::from_items` also routes
  through.
- Both `Cbindgen` and `JniGen` take every **source** fact from an element.
- No component re-derives a source fact by matching captured syntax; the ledger
  has reached the irreducible set, and every remaining entry is documented as
  inspecting adapter-synthesized types.
- The accepted Rust subset is covered by the acceptance matrix with precise
  diagnostics naming item and component.
- Spelling generated Rust is done by re-emitting a `syntax` slice, never by
  reconstructing one from a classification.

## Relationship to #215

#215 is superseded. Its four merged PRs are not lost: L0 ports the
array-length subgrammar (#212), the type grammar and its acceptance tests, the
enum tag/discriminant numbering (#226) and the boundary ledger (#224). What is
dropped is the syn-free model itself — `SourceType::to_syn`,
`DiscriminantSource`, `VariantShape`, `NamedArg::Lifetime` — because carrying the
source's own slice does that job without a modelling cost.

The `source-frontend` branch stays in place as the reference. Nothing depends on
it, and it is not a base for anything here: every stage of this program lands on
`language-integration`, which merges to `main` when the program does.

## Review protocol

Each stage PR states its own exit:

- **Reported** — what `examples/regen-check.sh` did, always. The check is
  **instrumentation, not a constraint**: it says what moved, not whether the
  change was allowed. Byte-identical is the strongest evidence a refactor did
  nothing unintended and is worth claiming when it holds — but generated output
  that moves without changing semantics or performance is fully acceptable, and
  no architecture decision may be reshaped to keep bytes matching.
- **Explained** — if output moved, why the change is semantically and
  performance neutral. A movement outside that explanation is a bug.
- **Asserted** — the invariant the stage adds, and the ledger delta it claims.

Run the check the way that makes it mean something: `git clean -fd examples/`
first (an earlier `--all-features` run leaves artifacts the check reads as drift),
then `cargo clean -p example-cbindgen -p example-flat` (it only regenerates what
cargo decides to rebuild, so a cached run passes without checking anything).

[`core::flat`]: ../prebindgen/src/api/core/flat/mod.rs
