# JniGen API critique

An analysis of the `prebindgen::lang::JniGen` public API: where it is misleading,
illogical, inconvenient, or incomplete. Evidence base: the builder/decl/config
surface (`prebindgen/src/api/lang/jnigen/jni/{builder,decl,config,mod,selector,kotlin_emit}.rs`),
the real consumer ([zenoh-flat-jni](https://github.com/ZettaScaleLabs/zenoh-flat-jni)
`build.rs`, ~600 lines), and the Kotlin that consumer generates.

Each finding has a stable ID (`M*` misleading, `I*` illogical, `C*` inconvenient,
`N*` incomplete) referenced from the tracking PR's checklist. Findings are kept
even after being fixed, with a **Resolution** note appended, so the document
remains a design-decision record.

---

## Misleading

### M1. Symmetric names, opposite composition semantics

Repeated `.default_param_expand(ctor)` calls accumulate *alternatives* (OR —
runtime-dispatched variants), while repeated `.default_return_expand(accessor)`
calls accumulate *fields* (AND — all delivered in one crossing). Identical call
shape, sum-vs-product semantics; nothing in the names distinguishes them.

**Resolution (2026-07-13, with I1):** the type-level declarations are now
dedicated builders whose arm names carry the semantics — `param_expand!(T)`
accumulates `.variant()` arms (OR), `return_expand!(T)` accumulates `.field()`
entries (AND). The per-fn overrides keep the `param_expand(param, …)` /
`return_expand(…)` names; their differing shapes (param-keyed vs not) already
distinguish them.

### M2. `_self` means three different things

- Class-level `.default_param_expand_self()` alone: a documented **no-op**
  (identity is already the default; `builder.rs` skips it) — yet the consumer
  calls it anyway, "for documentation".
- Fn-level `.param_expand_self(param)`: a meaningful **opt-out** of the class's
  default variants.
- Fn-level `.return_expand_self()`: "the raw handle" — but for borrowed returns
  (`&T`) it actually means **clone into a fresh owned handle**: a hidden
  allocation behind a name that says "identity".

### M3. `default_param_expand` docs oversell ergonomics

Doc: "lets every function taking a KeyExpr also accept a plain String."
Generated reality:

```kotlin
public fun sessionPut(
    session: Session,
    keyExprSel: Int,
    keyExpr0: String?,
    keyExpr1: KeyExpr?,
    ...
)
```

— the caller passes a manual discriminator int plus per-variant nullable slots
(`0, "key", null`). No overloads, no sealed type. If this tier is meant to be
hand-wrapped, the decl docs shouldn't be phrased in end-user ergonomics terms.

### M4. Error support requires declaring a phantom class

To make `Result<T, Error>` resolve, `Error` must be declared `ptr_class!(Error)`
— but the handle "never actually crosses" (consumer's own comment). The
generator still emits a full public `class Error(initialPtr: Long) :
NativeHandle` with `close()`, `take()`, `freePtr()` — a dead class with a
public raw-pointer constructor no user can legitimately call. Errors deserve a
first-class declaration.

### M5. "wrapper" is overloaded

`ScalarTypeWrapperDecl` = a conversion rule for a *concrete* type;
`GenericTypeWrapperDecl` = a rule for Rust *wrapper types* (`Option`/`Result`).
Their direction pairs are also named inconsistently: `on_param`/`on_return` vs
`input`/`output` vs `param_expand`/`return_expand`.

### M6. Two "package" concepts

`.package(package!("bytes"))` adds a declaration batch under a *sub*package;
`set_package_prefix` sets the actual package. `package!()` (no args) means
"no subpackage", not "no package".

### M7. Stale docs inside the API

`kotlin_emit.rs` module doc says functions get a home "via `.method(...)`" —
renamed away long ago. The `JniGen` doc example (`jni/mod.rs`) uses
`ZKeyExpr`/`z_keyexpr_as_str` — pre-de-prefix names that no longer exist
anywhere in the workspace.

### M8. Generator internals leak into the public generated API

`ErrorHandler.run(je: String?, message: String)` — `je` ("JNI error") is an
internal abbreviation in a public Kotlin signature; capture fields `ze0`,
`__cap.ze0!!` force-unwraps.

---

## Illogical / inconsistent

### I1. The same fact must be declared twice

Sample's 13 getters each appear as `.fun(fun!(sample_get_payload).name("payload"))`
*and* `.default_return_expand(fun!(sample_get_payload).name("payload"))` — 26
declarations, `.name()` repeated verbatim (same pattern for Query, Reply, Hello,
Encoding). The `ClassMember` docs confirm records "don't consult this list".
Constructors doubling as param variants also repeat
(`.constructor(fun!(zbytes_new_from_vec).name("fromVec"))` +
`.default_param_expand(fun!(zbytes_new_from_vec))`). Roughly a third of the
consumer's decl text is restatement.

**Resolution (2026-07-13):** rather than a member-modifier shortcut (rejected:
a second way to say the same thing), the two jobs were separated. A class decl
now declares only the Kotlin surface; the boundary shape is declared once,
per direction, at the generator level:
`.param_expand(param_expand!(T).variant(fun!(ctor)).variant_self())` and
`.return_expand(return_expand!(T).field(fun!(get)).field_self())`. A
`.field(fun!(x))` inherits its Kotlin name from the class member declaration
of the same fn (explicit `.name()` wins), so the name is written once. The
declarations remain order-independent: `JniGen` stores boundary decls raw and
assembles the expansion sets at the point of use (the `Prebindgen` trait's
`expansions()`/`deconstructors()` hooks now return by value).

### I2. Order-independence advertised where it's cheap, absent where it's load-bearing

Settings are "order-independent by construction" (`config.rs` doc) — but
`.default_return_expand_self()` MUST be declared *last* after nested-handle
fields, or the generated Rust fails with "use of moved value". The consumer's
build.rs carries a shouting ORDER MATTERS comment. The generator could sort
identity leaves last, or hard-error with a hint; instead the invariant lives in
a consumer comment.

**Correction (2026-07-13):** overstated — the generator already owns this
invariant with a decl-time hard error (`UnfoldError::RootIdentityBeforeNested`,
message includes the fix), so the wrong order never reaches non-compiling
Rust. The consumer comment predates that check. Remaining question for this
item: keep the hard error, or silently sort identity leaves last.

### I3. Flipped receivers, hidden temporal coupling

`registry.write_rust(&jni, path)` but `jni.write_kotlin(&registry, path)` — the
two halves of one generation run with inverted receivers, and `write_kotlin`
silently requires the resolution `write_rust` performed. Nothing in the types
enforces the order.

### I4. Inconsistent argument kinds for the same concept

`ignore_fun(FunctionDecl)` vs `ignore_class(syn::Type)`. Several ways to spell
a function reference coexist (`fun!(x)`, `FunctionDecl::new(ident!(x))`, `pq!`
for params) — the consumer file uses three of them.

### I5. Member support varies by class kind for implementation reasons

`data_class` forbids `.fun`/`.constructor` ("no handle to hang a method on")
while `value_class` — equally handle-less — allows them. `.kotlin_type()` (map
onto an existing Kotlin type) exists only on data/value classes; enums and ptr
classes can't be mapped onto existing types.

### I6. Three overlapping const mechanisms

`constant`, `constant_fun`, `constant_expr`: the flagship consumer only needed
`constant_expr` for the non-trivial cases; the superseded `constant_fun` /
path-alias rounds shipped anyway.

### I7. Mixed failure modes with no policy

Dotted `.name()` → panic at decl time; builtin scalar-wrapper pattern → panic;
typo'd `fun!(name)` → **cargo warning + silent omission** (`registry.rs`
"declared function not found"); non-`Copy` value_class → compile error inside
`generated_bindings.rs`; malformed value-blob bytes → runtime Kotlin error;
wildcard rank > 3 → silent `return None`. A *declared*-but-missing function is
explicit intent gone wrong and should be a hard error.

---

## Inconvenient

### C1. `.name()` needed on nearly every member

Default Kotlin name is `snake_to_camel` of the full Rust ident —
`sample_get_payload` becomes `sampleGetPayload` *on the Sample class*, so
effectively every `.fun()` carries a manual `.name()`. The generator knows the
class's Rust name; it could strip `<class>_get_` / `<class>_` prefixes for
members, or offer a member-name mangle hook (five global mangle hooks exist;
none applies here).

### C2. No bulk/pattern ignore

53 hand-enumerated `ident!(encoding_const_*)` calls exist only to silence
undeclared-fn warnings — and the same 53 names appear a second time (lowercase)
to build the consts. No prefix/regex ignore, no way to derive one list from the
other.

### C3. The advanced wrapper tier demands syn/quote fluency

Bodies are token-interpolation closures (`|v| pq!(Millis(*#v as u64))`) with a
fixed value-ident convention, `jni::sys::*` wire names, patterns matched by
*exact canonical path equality* (no short-name matching — invisible until
nothing resolves), and closure arity encoding rank via phantom-typed
`WrapperBuilder<Arity1..3>` — a wrong arity yields an opaque
unsatisfied-trait-bound error.

### C4. Manual output hygiene

The consumer must `remove_dir_all` the Kotlin root itself to avoid stale files
after package moves; `write_kotlin` returns paths written but offers no prune
option.

### C5. Parameters named by bare ident

`param_expand_self(pq!(key_expr))` — no decl-time validation against the
function's real signature; typos surface late (or as warnings).

### C6. Global-only lock toggle

`set_emit_handle_locks(bool)` is all-or-nothing; the known per-call lock
overhead on hot paths can't be traded per class or function.

### C7. No introspection

Canonical inputs/outputs "AUTO-APPLY" at a distance; the only way to learn what
a decl produces is to run generation and diff. The extensive prose comments in
the consumer's build.rs are compensating for a missing explain/dry-run mode.

---

## Incomplete

### N1. Doc comments dropped

`#[prebindgen]` items carry `///` docs (e.g. `session_put`), but the generated
Kotlin functions have no KDoc — a real gap for a published Maven artifact.

### N2. Acknowledged coverage holes

`Vec<closeable-handle>` is an explicit `panic!`; `Option<ptr_class>` params
can't be built through by the recursive input fold (forces Sample's
identity-only input); opaque-handle consts rejected; `&mut [T]` unsupported;
output-side `Option<prim>` boxing deliberately not done.

### N3. API split across branches

**Resolution (2026-07-13): already resolved before this effort started** —
`const_support` was squash-merged to `main` as PR #31 (`e90ce85`).

### N4. Public-surface hygiene

`JniGen.source_module`/`package` are `pub` fields — direct writes bypass
`set_package_prefix`'s trimming; `pub mod jni` exposes undocumented internals
(`MethodEntry`, `WireBody` plumbing) beyond the curated `lang::` re-export list.

### N5. Decl-time validation gaps

Nothing checks a `.fun()` target takes `&Self` first or a `.constructor()`
returns `Self` at declaration; mistakes surface as resolver errors during
`write_rust`, far from the offending decl. The locking scaffold has no
JVM-runtime test (design-verified only).

---

## What the API gets right

Plain-value decls with no typestate cursor (the consumer's encoding-const loop
is ordinary Rust); order-independent settings via derived-at-read names;
mergeable packages; the macro layer genuinely dodging E0283; deterministic
most-specific-first pattern ordering; warnings instead of silence for
undeclared items; the single `JNINative` choke point with the `init{}` loading
hook.

## Ranked recommendations

1. **Kill the double declaration (I1):** let a member decl opt into being a
   return-field record, e.g. `.fun(fun!(x).name("y").as_return_field())`, or
   have `default_return_expand(fun!(x))` reuse the member's `.name()`.
2. **Own the field-ordering invariant (I2):** emit identity leaves last
   automatically, or hard-error with the fix in the message.
3. **First-class error declaration (M4):** an error-type declaration that feeds
   the resolver without emitting a phantom handle class.
4. **Hard-error on declared-not-found + pattern ignore (I7, C2).**
5. **Member-name prefix stripping or a member-name mangle hook (C1).**
6. **Rename for semantics (M1/M2/M5):** e.g. `param_accepts(ctor)` /
   `param_accepts_self()` (variants, OR) vs `return_field(accessor)` /
   `return_with_self()` (fields, AND); a distinct name for the borrowed-return
   clone case.
7. **Carry `///` docs into KDoc (N1).**
8. **Fix stale docs/examples (M7); privatize `pub` fields (N4); add a
   `write_kotlin` prune option (C4).**
