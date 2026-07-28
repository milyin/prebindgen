# The prebindgen source language

Status: **partial specification**. Tracked by
[#211](https://github.com/milyin/prebindgen/issues/211).

This document states which Rust forms a `#[prebindgen]` source crate may use.
It is the contract between the source crate and every generator: what a form
*means* is decided once, by the frontend; how it *crosses a boundary* is decided
per adapter.

> **Read this first:** the [type grammar](#types) and the
> [array-length subgrammar](#array-lengths) have moved into `core::frontend`,
> but they are enforced there **only in modeled positions** — today, the fields
> of a `#[prebindgen]` struct. Item kinds, function signatures and every other
> type position are still decided at their use sites, often inside an adapter,
> and #211 tracks moving them.
>
> So a row below can be refused by the frontend in a struct field and reach an
> adapter unexamined in a function parameter. Where that distinction matters it
> is stated on the row.

Anything not listed here is **unspecified**. A form that happens to work is not
thereby part of the language, and the frontend may start refusing it.

## Item kinds

A `#[prebindgen]` attribute may be applied to:

| Form | Status | Notes |
|---|---|---|
| `struct` with named fields | supported | The only struct shape whose fields are scanned. |
| `struct` with unnamed fields (tuple struct) | indexed, fields not scanned | Usable as an opaque handle; its fields are not a boundary surface. |
| `enum`, unit variants only | supported | |
| `enum` with payload variants | adapter-dependent | See [`sum-types.md`](sum-types.md). |
| `fn` | supported | Free functions only — see [Functions](#functions). |
| `const` | supported | Also the only way to name a value in an [array length](#array-lengths). |
| `union` | captured, not supported | Recorded by the proc-macro; no adapter lowers one. |
| `type` alias | captured, not supported | Recorded; not resolved as a distinct type. |
| anything else | passthrough | Emitted verbatim into the generated file, uninterpreted. |

Names live in **one flat namespace** across every ingested source crate: two
`#[prebindgen]` items with the same bare name are a hard error
(`ScanError::DuplicateName`), whichever crates they came from.

An unnamed `const _` passes through ungated — it is how the injected feature
guard is emitted.

## Functions

Decided by `Registry::scan_fn_signature`
(`prebindgen/src/api/core/registry.rs`).

| Form | Status |
|---|---|
| free `fn` with ident parameters | supported |
| a `self` receiver | **refused** — `ScanError::UnsupportedReceiver` |
| a non-ident parameter pattern (`(a, b): (u8, u8)`) | **refused** — `ScanError::UnsupportedParamPattern` |
| generic parameters / `where` clauses | unspecified |
| `impl Fn(T, …) + Send + Sync + 'static` parameter | supported — the callback form, see [Callbacks](#callbacks) |
| any other `impl Trait` | **refused** — `ScanError::DisallowedImplTrait` |

A source crate stays **plain idiomatic Rust**: values are returned by value,
fallible calls return `Result<T, E>`, and there is no `#[repr(C)]`, no raw
pointer plumbing, and no destination-language vocabulary. Lowering to a C ABI or
to JNI is the adapter's job.

## Types

**Decided by the frontend** in modeled positions —
`core::frontend::model::lower_type` produces a closed `SourceType`, and lowering
is total: a form with no variant is refused. Elsewhere, type positions are
walked by `immediate_subtype_positions` (`prebindgen/src/api/core/registry.rs`)
and canonicalized at ingest by `types_util::normalize_type`.

| Form | Status | Notes |
|---|---|---|
| path type (`Foo`, `Vec<u8>`, `Option<&T>`) | supported | |
| reference (`&T`, `&'a T`, `&mut T`) | supported | |
| slice (`&[T]`) | supported | |
| fixed-size array (`[T; N]`) | supported | Length: see [below](#array-lengths). |
| the unit `()` | supported | |
| a non-empty tuple | **refused** | No adapter has ever lowered one; only `()` is in the language. |
| raw pointer | walked, adapter-dependent | |
| `impl Fn(…) + Send + Sync + 'static` | supported in a parameter position | Must return `()` — see [Callbacks](#callbacks). |
| any other `impl Trait` | refused | |
| a path with a qualified self (`<T as Trait>::Assoc`) | **refused** | `#[prebindgen]` never captures `impl` blocks, so what it resolves to is unknowable. Outside modeled positions `normalize_type` still leaves it verbatim. |

### Lifetimes are part of a type's name

A lifetime is preserved exactly — `Foo<'static>` is **not** `Foo`, and
`&'static T` is not `&T` — but it is never modeled as structure, because it
means nothing to a destination language. The model keeps it verbatim so the
projection reconstructs the type it came from.

That matters beyond spelling: declarations are matched against the normalized
form, so `ptr_class!(ZKeyExpr<'static>)` only ever matches a captured
`ZKeyExpr<'static>`.

### A builtin must be spelled bare

`String` / `Option` / `Vec` / `Box` / `Result` are recognized only as a bare,
single-segment path. `normalize_type` has already reduced the genuine std
spellings at ingest and deliberately leaves unknown crate paths alone, so
`foreign::Option<u8>` is a **foreign named type** that merely shares a name —
collapsing it would silently retype the value and select the wrong converter.

### Path normalization

`normalize_type` reduces a captured type path to one canonical spelling. The
complete rule set is documented on that function; in summary:

* `Group` / `Paren` wrappers unwrap;
* a path headed by `crate` / `self`, or by an ingested source crate's module
  name, reduces to its **final** segment (`crate::a::Foo<T>` ≡ `Foo<T>`);
* a std-prelude path reduces to its bare form — exactly
  `std|core|alloc :: vec::Vec | option::Option | result::Result | string::String
  | boxed::Box`;
* **nothing else** is touched. `std::ffi::CString` stays qualified, and an
  unknown crate path (`zenoh::KeyExpr`) is never reduced — the registry has no
  index of a foreign namespace, so `a::KeyExpr` and `b::KeyExpr` may be
  genuinely distinct types.
* Lifetimes are **not** normalized: `&'a T` ≠ `&T`, `Foo<'static>` ≠ `Foo`.

Because a declaration in `build.rs` is matched against the *normalized*
spelling, declaring a source item with its crate path
(`ptr_class!(myflat::Foo)`) is a hard error with a fix-it, not a silent miss.

## Callbacks

**Decided by the frontend** — `core::registry::extract_fn_trait_sig`, the single
acceptance gate for every callback position.

The accepted form is exactly `impl Fn(T, …) + Send + Sync + 'static`, and the
`Fn` **must return `()`** (elided or written). Bound order is free and
canonicalized; so are a trailing input comma and an explicit `-> ()`.

| Form | Status |
|---|---|
| `impl Fn(u8) + Send + Sync + 'static` | supported |
| `impl Fn(u8,) -> () + Sync + Send + 'static` | supported — same type, canonicalized |
| `impl Fn(u8) -> u16 + …` | **reserved** — issue [#216](https://github.com/milyin/prebindgen/issues/216) |
| `impl Fn(u8) -> impl Fn(u8) + …` | **reserved** |
| `impl for<'a> Fn(&'a u8) + …` | **unsupported** |

### Reserved is not the same as unsupported

A refusal says which of two things it is, because they call for opposite
responses:

* **reserved** — the language intends this and the machinery does not exist
  yet. Wait, or work around it; the diagnostic names the tracking issue.
* **unsupported** — it cannot work here and will not. Redesign.

A callback returning a value is *reserved*: every callback wire is void-shaped
today — C's `call` has no return, jnigen's `run` returns void — so the result
would be dropped. It used to be **accepted** and silently dropped, which is the
bug this refusal replaces.

A higher-ranked binder is *unsupported*: no FFI boundary can be generic over a
lifetime, so no adapter could ever carry one.

## Array lengths

**Decided by the frontend** — `core::frontend::lower_array_len`. This is the
only subgrammar with an executable acceptance matrix
(`prebindgen/src/api/core/frontend/tests.rs`); it is closed, and both adapters
consume the same `ArrayExtent`.

### A length is always a number

That is the contract, not an implementation detail. A binding generator runs in
`build.rs`, where it cannot evaluate arbitrary Rust; and some destination
languages cannot reference a Rust const at all — a surface that groups a small
array into scalars needs the count literally. A length the frontend cannot
reduce to a `usize` is therefore not a length prebindgen accepts.

Two spellings reach that number, and nothing else does:

| Form | Status | Lowers to |
|---|---|---|
| `[u8; 4]`, `[u8; 16usize]` | supported | `Literal(4)` / `Literal(16)` |
| `[u8; N]` where `#[prebindgen] pub const N: usize = 4;` | supported | `{ value: 4, source: Const(N) }` |
| everything else | **refused** | see below |

Both carry the value, and a const extent also carries **which const**. Which of
the two an emitter uses is that adapter's choice:

* generated **Rust** always uses the number, so no length is ever a path there
  and there is nothing to qualify — `[u8; ARRAY_BYTES]` emits as `[u8; 4]`;
* a **C header** uses the name — `uint8_t tag[MARKER_TAG_LEN]` — because a
  symbolic extent is part of that API's meaning;
* **Kotlin** has nowhere to put an extent at all.

The value is what makes a const length and the same number written literally
**one type and one converter** — they always were in Rust, and the frontend
agrees.

### Three identities, kept apart

Normalization is deliberately lossy, and that dictates which question each layer
can answer:

```text
[u8; A]  where A = 4
[u8; B]  where B = 4     ──>   [u8; 4]     (one Rust type, one converter)
[u8; 4]
```

| Layer | Keyed by | Answers |
|---|---|---|
| const index | const name + origin crate | `A` is `4` in crate X |
| type table (`Registry::array_len`) | `TypeKey` | this array type's length is `4` |
| **source model** (`Registry::source_struct`) | **use site** — struct + field | field `S::a` was written `[u8; A]` |

`Registry::array_len` returns a bare `usize` and **cannot** report the spelling:
by the time a `TypeKey` exists that question has more than one true answer, and
storing one would make it depend on which occurrence was seen last.

The spelling lives on the **use site** instead, in the typed source model, as an
`ArrayExtent { value, source }`. Both halves have consumers:

* `value` is what a destination language needs when it cannot reference a Rust
  const at all — Kotlin gets `[u8; 4]`;
* `source` is what a C header needs, because a symbolic extent is part of that
  API's meaning. `uint8_t tag[MARKER_TAG_LEN]` makes changing the size one edit
  rather than a hunt through literals.

An adapter reads the extent **as a decided fact**. It must never recover it by
re-reading `syn` — that is issue #211's invariant 6, and the reason the model
exists rather than a side channel carrying the original syntax.

A const an extent names is carried into the destination language by the adapter
that spells it: `lang::Cbindgen` re-emits such a const with its literal value so
cbindgen produces `#define MARKER_TAG_LEN 4`. Without that the header would name
a symbol it never defines.

### What is refused, and why

| Form | Reason |
|---|---|
| `[u8; MAX]` where `MAX` is not `#[prebindgen]` | `NotAMarkedConst` |
| `[u8; N]` where `N`'s value is not an integer literal | `ConstIsNotALiteral` |
| `[u8; N]` where `N` is marked in a *different* source crate | `ForeignSourceConst` |
| `[u8; crate::limits::MAX]`, `[u8; myflat::MAX]`, `[u8; limits::MAX]` | `NotABareName` |
| `[u8; Holder::N]` — an associated const | `NotABareName` |
| `[u8; <Holder>::N]`, `[u8; <Holder as Tr>::N]` | `NotABareName` |
| `[u8; usize::MAX]`, `[u8; ::MAX]`, `[u8; other_crate::X]` | `NotABareName` |
| `[u8; A + 1]`, `[u8; -1]`, `[u8; A as usize]`, `[u8; (A)]` | `NotLiteralOrName` |
| `[u8; array_len()]` | `NotLiteralOrName` |
| `[u8; const { … }]`, `[u8; match … ]`, `[u8; if let … ]`, `[u8; \|\| 3]` | `NotLiteralOrName` |
| a non-integer or oversized literal | `NotAnIntegerLiteral` / `IntegerOutOfRange` |

**Only a bare name.** `#[prebindgen]` items live in one flat, uniquely named
namespace, so the bare name is the item's complete address. Any longer path
either restates that (`crate::limits::MAX`) or reaches somewhere the frontend
cannot follow — a module it does not index, an `impl` block it never captured, a
foreign crate. Neither can be reduced to a number, and guessing between them is
how a length silently becomes the wrong one.

This also removes a genuine ambiguity: without indexing modules, a relative
`limits::MAX` is indistinguishable from an external crate path of the same
shape.

**Only your own crate's const.** Uniqueness holds across the *marked* namespace
only. A source crate may have an unmarked `MAX` of its own and mean that one,
while a chained source has a marked `MAX` — and the marked one is all the
frontend can see. Resolving to it would silently change the length rather than
fail, so a length must name a const from the item's own source crate.

### The `#[prebindgen]` requirement

A const used as an array length must itself be marked:

```rust
#[prebindgen]
pub const ARRAY_BYTES: usize = 4;   // ← the attribute is load-bearing

#[prebindgen]
pub struct Arrays {
    pub bytes: [u8; ARRAY_BYTES],
}
```

The generated crate sees **only** what the macro exposed. Without the attribute
the const is never captured, so the frontend has no value to read and refuses
the length outright — it is not a warning and not a downstream compile error.

The const does **not** have to be declared to the binding: a length is a
compile-time value, not part of any destination language's surface. The
covertest fixture in `examples/perftest-flat/src/ext.rs` pins exactly this
arrangement.

Note the restriction is on **lengths**, not on consts. A `#[prebindgen] const`
may be computed however you like; it just cannot be used as an array length
unless its value is a plain integer literal.

### Why this one is closed

Acceptance used to be a *separate judgement* from qualification: a whitelist
decided what was allowed and a rewriter decided what it could resolve, with
nothing tying them together. They disagreed eight times
([#210](https://github.com/milyin/prebindgen/issues/210)) — most sharply over
`<Holder>::N`, which the whitelist accepted and the rewriter silently declined
to qualify, emitting an unresolvable path into the generated crate.

`lower_array_len` returns `Ok` only if the length was fully understood **and**
reduced to a number. "Accepted" therefore means "lowered", by construction, and
a form the function does not lower is a form the language does not accept. There
is no second list to drift from — and because the result is a number, there is
no path left to qualify and no namespace left to get wrong.

## Diagnostics

Source-language violations surface as `ScanError` from
`Registry::from_items` — **before** any adapter runs, so every generator refuses
the same input with the same message. A refused item leaves no partially
rewritten model behind.
