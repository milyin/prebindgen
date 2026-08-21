# The model a binding is described in

`prebindgen-registry` generates a language binding from one annotated Rust
source. What a binding author writes, and what a language adapter answers, are
both stated in the same small vocabulary. These chapters are that vocabulary and
the machinery immediately around it.

They describe the model, not the API surface an adapter programs against — the
table, sites, the `Compile` hooks and the error set are documented separately.

## Chapters

1. [**The vocabulary**](01-vocabulary.md) — the fifteen terms everything else is
   stated in, in dependency order: declaration, the registry, boundary, Rust and
   wire values, direction, crossing, callback type, site, part, shape, the
   table, row, fragment, plan.
2. [**How the file is built**](02-how-the-file-is-built.md) — who calls whom, in
   what order, and where the generated Rust comes from. Converters, wrappers,
   prerequisites, and what the foreign side ends up able to call.
3. [**Directions and crossings**](03-directions-and-crossings.md) — the two
   directions, the crossing that pairs one with a Rust type, its key, and the
   name a row is filed under.

## Where the crates sit

Four take part.

- **`prebindgen`** — the `#[prebindgen]` proc macro. It captures each marked
  item of the source crate into a data file at build time. The source crate
  contains nothing about any foreign language.
- **`prebindgen-flat`** — parses those records into **the model**, `Flat`:
  `Struct`, `Variant` (an enum whose alternatives carry payloads, each an
  `Alternative`), `Enum`, `Function`, `Field`, and `TypeRef`, the model's
  reading of one Rust type. `TypeKey` is the identity that same type is stored
  under.
- **`prebindgen-registry`** — the language-agnostic half of generation. It is
  what these chapters describe.
- **an adapter**, one per target language — `prebindgen-c`, whose generator type
  is `Cbindgen`, and `prebindgen-jni`, whose generator type is `JniGen`.

A **binding crate** such as `zenoh-flat-c` runs an adapter over a model in its
build script, and compiles what comes out.
