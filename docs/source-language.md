# The prebindgen source language

Status: **partial specification**. Tracked by
[#211](https://github.com/milyin/prebindgen/issues/211).

This document states which Rust forms a `#[prebindgen]` source crate may use.
It is the contract between the source crate and every generator: what a form
*means* is decided once, by the frontend; how it *crosses a boundary* is decided
per adapter.

> **Read this first:** only the [array-length subgrammar](#array-lengths) has
> actually moved into `core::frontend` so far. Every other row below records
> where the decision is made *today*, which is often inside an adapter. #211
> tracks moving them. Rows marked **frontend** are decided once and are
> identical for every generator; rows marked with a site are not, and may differ
> between C and JNI until they migrate.

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
| `impl Fn(T, …) + Send + Sync + 'static` parameter | supported — the callback form |
| any other `impl Trait` | **refused** — `ScanError::DisallowedImplTrait` |

A source crate stays **plain idiomatic Rust**: values are returned by value,
fallible calls return `Result<T, E>`, and there is no `#[repr(C)]`, no raw
pointer plumbing, and no destination-language vocabulary. Lowering to a C ABI or
to JNI is the adapter's job.

## Types

Type positions are walked by `immediate_subtype_positions`
(`prebindgen/src/api/core/registry.rs`) and canonicalized at ingest by
`types_util::normalize_type`.

| Form | Status | Notes |
|---|---|---|
| path type (`Foo`, `Vec<u8>`, `Option<&T>`) | supported | |
| reference (`&T`, `&mut T`) | supported | |
| slice (`&[T]`) | supported | |
| fixed-size array (`[T; N]`) | supported | Length: see [below](#array-lengths). |
| tuple | supported | |
| raw pointer | walked, adapter-dependent | |
| `impl Fn(…)` | supported in a parameter position | |
| any other `impl Trait` | refused | |
| a path with a qualified self (`<T as Trait>::Assoc`) | left verbatim | Never normalized; its spelling is its identity. |

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

## Array lengths

**Decided by the frontend** — `core::frontend::lower_array_len`. This is the
only subgrammar with an executable acceptance matrix
(`prebindgen/src/api/core/frontend/tests.rs`); it is closed, and both adapters
consume the same `ArrayLen`.

A length is a literal or a plain path, and nothing else:

```rust
enum ArrayLen {
    Literal(usize),
    SourceConst  { path },   // resolved to a #[prebindgen] item, stored absolute
    ExternalConst { path },  // not a source path; emitted verbatim
}
```

### How a path is resolved

The source crate may spell a path **however it likes** — bare, `crate`-rooted,
through the module the item is declared in. Lowering is two decisions, in this
order:

**1. Is the path source-relative?** Yes if its head segment is `crate`, `self`,
an ingested source crate's module name, or an indexed item's name. Everything
else is `ExternalConst` and is emitted **verbatim, exactly as written**.

Classification comes first, before any rewriting, so the verbatim guarantee
actually holds. It also mirrors the type rule: a foreign crate's path is never
touched, because the registry has no index of a foreign namespace and
`a::Holder` and `b::Holder` may be genuinely different things. So
`other_crate::Holder::N` stays put even though `Holder` happens to name an
indexed item.

**2. Which segment names the item?** The **leftmost** segment that names an
indexed item — skipping a const or function that has segments after it, since
nothing is reachable *through* a const.

Everything before that anchor is a module path **within the source crate** and
is replaced wholesale by the item's origin module. Everything after it is
relative to the item and is kept.

Replacing the prefix is sound because prebindgen items live in **one flat,
uniquely named namespace**, so the bare name already identifies the item and the
module prefix carries no information. That is the same invariant
`normalize_type` relies on for types.

> **This requires the item to be reachable as `<crate>::<bare name>`.** An item
> declared in a private or nested module must be re-exported at the source
> crate's root. `zenoh-flat` does exactly this: `ZENOH_ID_MAX_SIZE` lives in
> `base::config::zenoh_id` and is re-exported from `lib.rs`.

Leftmost-first matters: in `Holder::N` where a free const `N` is *also* indexed,
`Holder` is the anchor and `N` is its associated const — not the other way
round.

| Form | Status | Lowers to |
|---|---|---|
| `[u8; 4]`, `[u8; 16usize]` | supported | `Literal` |
| `[u8; MAX]` — a `#[prebindgen]` const | supported | `SourceConst`, spelled `myflat::MAX` |
| `[u8; Holder::N]` — an associated const | supported | `SourceConst`, `myflat::Holder::N`; segments after the item are kept |
| `[u8; crate::MAX]`, `[u8; myflat::MAX]` | supported | Same value as the bare spelling |
| `[u8; crate::limits::MAX]`, `[u8; myflat::limits::MAX]` | supported | Same value again — the intermediate module is dropped |
| `[u8; crate::limits::Holder::N]` | supported | `myflat::Holder::N` |
| `[u8; usize::MAX]`, `[u8; other_crate::X]` | supported | `ExternalConst`, verbatim |
| `[u8; crate::limits::UNMARKED]` | **refused** | `UnresolvedSourcePath` — claims a source item, names none |
| `[u8; <Holder>::N]`, `[u8; <Holder as Tr>::N]` | **refused** | `QualifiedSelf` |
| `[u8; ::MAX]` | **refused** | `CrateRootPath` |
| `[u8; A + 1]`, `[u8; -1]`, `[u8; A as usize]`, `[u8; (A)]` | **refused** | Const arithmetic is not in the language |
| `[u8; array_len()]` | **refused** | A `const fn` call is not in the language |
| `[u8; const { … }]`, `[u8; match … ]`, `[u8; if let … ]`, `[u8; \|\| 3]` | **refused** | Anything that can bind a name |
| a non-integer or oversized literal | **refused** | |

Everything refused for *shape* has the same fix: **hoist the value into a named
`const`** and use that as the length. `UnresolvedSourcePath` has a different
one: mark the intended item `#[prebindgen]`.

### The `#[prebindgen]` trap

A const used as a length must itself be marked `#[prebindgen]`:

```rust
#[prebindgen]
pub const ARRAY_BYTES: usize = 4;   // ← the attribute is load-bearing

#[prebindgen]
pub struct Arrays {
    pub bytes: [u8; ARRAY_BYTES],
}
```

Without it the registry never indexes the const, and what happens depends on how
the length was spelled:

* **source-relative** (`crate::limits::UNMARKED`, `myflat::UNMARKED`) — a **hard
  frontend error**. The path claims to name a source item and names none, so
  there is nothing to qualify it with.
* **bare** (`UNMARKED`) — indistinguishable from an external namespace, so it
  lowers to `ExternalConst` and is emitted verbatim. The generated crate, which
  is not the source crate, then fails to resolve it. This is the one remaining
  quiet case; prefer the `crate::`-rooted spelling if you want the loud one.

The const does **not** have to be declared to the binding: a length is a
compile-time namespace, not part of any destination language's surface. The
covertest fixture in `examples/perftest-flat/src/ext.rs` pins exactly this
arrangement.

### Why this one is closed

Acceptance used to be a *separate judgement* from qualification: a whitelist
decided what was allowed and a rewriter decided what it could resolve, with
nothing tying them together. They disagreed eight times
([#210](https://github.com/milyin/prebindgen/issues/210)) — most sharply over
`<Holder>::N`, which the whitelist accepted and the rewriter silently declined
to qualify, emitting an unresolvable path into the generated crate.

`lower_array_len` returns `Ok` only if the length was fully understood **and**
fully resolved. "Accepted" therefore means "lowered", by construction, and a
form the function does not lower is a form the language does not accept. There
is no second list to drift from.

## Diagnostics

Source-language violations surface as `ScanError` from
`Registry::from_items` — **before** any adapter runs, so every generator refuses
the same input with the same message. A refused item leaves no partially
rewritten model behind.
