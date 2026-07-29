# Parse once, consume elements everywhere — integration map

Umbrella for making every component of prebindgen consume
[`core::language`]'s `Element`s instead of parsing captured Rust itself.

[#211](https://github.com/milyin/prebindgen/issues/211) remains the authority on
the invariants and the frontend/adapter boundary. This document does not restate
them; it records the design this program follows, what has landed, and the order.

This file is the one place stage state is edited. Change the doc, then re-sync
the umbrella PR body — never the other way round.

## The design

```
Source(s) ──items──> Language ──Elements──> Registry ──> adapters
  raw records          parse +               indexes        classify off `kind`
  (syn::Item)          validate              elements       spell off `syntax`
```

An `Element` is two things at once, and the pairing is the whole point:

* a **closed classification** — `TypeKind`, `StructFields`, the variant list —
  that says what the source *means*, in terms every destination language shares;
* the **exact syntax** each part was built from, sliced down to the parameter,
  field, variant and type.

```rust
pub struct Type    { pub kind: TypeKind, pub syntax: syn::Type }
pub struct Param   { pub name: syn::Ident, pub ty: Type, pub syntax: syn::PatType }
pub struct Variant { pub tag: i32, pub discriminant: Option<i64>, pub fields: Vec<Field>,
                     pub syntax: syn::Variant }
```

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
| `B()` vs `B` | `Variant::syntax` (via `Variant::spell`) | generated Rust only |
| `= 0x07` vs `= 7` | `Variant::syntax.discriminant` | a C mirror re-emits it |
| the number 7 | `Variant::discriminant` | Kotlin `NAME(7)`, `jint` decode |
| `Foo<'a, T>` | `Type::syntax` | generated Rust only |
| "it is a `Foo` with one type argument" | `TypeKind::Named` | every adapter |
| `[u8; TAG_LEN]` — spelling / number / const identity | `Type::syntax` / `ArrayExtent::value` / `ExtentSource::Const` | C header / Kotlin / both |

### The rule

> **Classify off `kind`, spell off `syntax`.**
>
> Matching a `syn::Type` or `syn::Expr` variant outside `core::language` is a
> classifier, and #211 says classification lives there alone. Passing a `syntax`
> slice into `quote!` is spelling, and spelling the source is exactly what
> generated Rust must do.

This is mechanically measured, and needed no new mechanism:
`core::language::boundary` (ported from
[#224](https://github.com/milyin/prebindgen/pull/224)) counts *variant mentions*
of watched syn enums per file, so `quote!(#slice)` is invisible to it while
`matches!(ty, syn::Type::Reference(_))` is counted. The committed ledger is the
scoreboard for this whole program.

## Size of the problem

Seeded by L0 at **202 classification sites** outside `core::language`, plus
**113** reads of the registry's `syn`-keyed item maps:

| Area | Ledger sites | Registry map reads | Stage |
|---|---:|---:|---|
| `api/core` (`types_util` 40, `unfold` 15, `registry` 13, `expand` 4) | 72 | 39 | L2 |
| `api/lang/cbindgen` | 25 | 25 | L3 |
| `api/lang/jnigen` | 105 | 49 | L4 |
| **total** | **202** | **113** | |

Not every site must go: some inspect types the adapter itself *synthesized* —
wire types, converter signatures — which is legitimately the adapter's business.
Separating the two populations is not a document to write up front; it is each
entry's fate as it comes off the ledger, with a stated reason in the PR that
moves it.

## Stages

| Stage | Owns | State |
|---|---|---|
| L0 | `Language` + `Element` + the ledger | **done** — [#227](https://github.com/milyin/prebindgen/pull/227) |
| L0.5 | `Flat`: the model, indexed and resolved | **done** — this branch |
| L1 | `Registry` consumes elements | not started |
| L2 | `api/core` stops classifying source syntax | not started |
| L3 | `Cbindgen` consumes elements | not started |
| L4 | `JniGen` consumes elements *(the long pole — 105 sites)* | not started |
| L5 | Close the seam: the public contract stops being `syn` | not started |

### L0 — the parser — **done** (#227)

- [x] `Language::parse` over any `(syn::Item, SourceLocation)` stream — the seam
      `Registry::from_items` occupies, so multi-source composition is unchanged
- [x] `Element` = `Function | Struct | Enum | Const | Unsupported | Passthrough`,
      every element and component carrying its syntax slice
- [x] `Type { kind, syntax }`; lowering total over the accepted grammar
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
- [x] `MaybeUninit<T>` becomes `TypeKind::Uninit`: a boundary concept the adapter
      was classifying, and the one foreign generic no alias can name
- [x] The example flat APIs are closed, and covertest-kotlin's build script
      asserts they stay closed across both its sources
- [x] **Did not move**: every generated artifact byte-identical

**Still open**: `zenoh-flat` and its two consumers are separate repos. Their 28
unmarked types (26 zenoh aliases, plus `Duration` and `Cow<'_, [u8]>`) need the
same treatment before they parse. `Cow<'_, [u8]>` has no alias spelling — generic
and lifetime-bearing — so `zbytes_to_bytes` needs either the `MaybeUninit`
treatment or a signature change.

### L1 — `Registry` consumes elements

The seam that makes the direction real. Adapters must not need touching.

- [ ] `Registry::from_flat(&Flat)`; `from_items` becomes `Flat::builder` +
      `from_flat`, so both entry points share one parser
- [ ] The `functions` / `structs` / `enums` / `consts` / `passthrough` maps are
      rebuilt from each element's retained `syntax` — a projection, not a second
      source of truth
- [ ] `scan_fn_signature`'s receiver / parameter-pattern / `impl Trait` guards
      are deleted: the diagnosis is already on `Element::Unsupported`, and
      declaring such an item is what raises it
- [ ] `ScanError`'s per-item variants map onto `ItemError`, so one authority
      produces the message
- [ ] **Must not move**: every generated artifact byte-identical
      (`examples/regen-check.sh`)

### L2 — `api/core` stops classifying source syntax

- [ ] `types_util` — 40 sites, the largest single file. `normalize_type`,
      `immediate_pattern_children`, `match_pattern`, the `is_*` predicates
- [ ] `registry::immediate_subtype_positions` — near-duplicate of
      `immediate_pattern_children`, and the two already diverge on `Type::Path`
- [ ] `unfold` (15) and `expand` (4) read element types
- [ ] `TypeKey` derivable from a `Type` so a lookup stops routing through a
      spelling
- [ ] Ledger down by the migrated count; every entry that *stays* is justified in
      the PR as adapter-synthesized

### L3 — `Cbindgen` consumes elements

- [ ] `builder` (8), `trait_impl` (6), `emit` (5), `mod` (5), `convert` (1)
- [ ] Variant patterns and constructors come from `Variant::spell`, not from
      re-deriving delimiters
- [ ] A discriminant is re-emitted from `Variant::syntax`, and the number comes
      from `Variant::discriminant`
- [ ] Generated C artifacts byte-identical

### L4 — `JniGen` consumes elements

The long pole. Split by area, each PR independently green.

- [ ] `emit/names` (17), `jni/builder` (13), `jni/trait_impl` (11),
      `emit/wrapper` (11), `emit/flat_input` (10), `render` (8), `selector` (7),
      and the rest
- [ ] `classify.rs` — a whole classifier with **zero** watched sites, so the
      ledger cannot see it: it must be migrated on its own merit
- [ ] `prim_array_of` reads `ArrayExtent` instead of re-matching `Type::Array`
- [ ] Generated Rust and Kotlin byte-identical

### L5 — close the seam

The public contract stops being `syn`, which is what stops the population from
growing back.

- [ ] `Registry`'s public item maps stop being the adapter-facing contract —
      relates to [#92](https://github.com/milyin/prebindgen/issues/92)
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

- One documented entry point from captured records to elements — `Language::parse`.
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
it, and it is not a base for anything here: every stage of this program targets
`main`.

## Review protocol

Each stage PR states its own exit:

- **Must not move** — byte-identical artifacts, enforced by
  `examples/regen-check.sh`. A diff is a bug.
- **Reviewed diff** — expected to change, cause stated up front. A diff outside
  that cause is a bug.
- **Asserted** — the invariant the stage adds, and the ledger delta it claims.

[`core::language`]: ../prebindgen/src/api/core/language/mod.rs
