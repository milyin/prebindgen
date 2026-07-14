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

**Resolution (2026-07-13):** the per-fn override mechanism was rebuilt to be
symmetric with the type level: `FunctionDecl::expand_param("param", decl)` /
`expand_return(decl)` take the **same expand-decl objects** as
`JniGen::expand` (macro family renamed `expand_param!` / `expand_return!`).
One rule now holds at both scopes and directions — *the decl states the
complete variant/field set*; an identity-only set (`.variant_self()` /
`.field_self()` alone) **is** the plain form and normalizes to it. This
dissolves the modal flips: `_self` is always just an element of a stated set,
and "opt-out" is simply "the set is {self}". The decl's type is cross-checked
against the actual parameter/return type (hard errors `ParamTypeMismatch` /
`ReturnTypeMismatch`), unknown parameter names error, duplicate per-fn decls
panic at decl time, and per-fn `.field(fun!(x))` inherits member Kotlin names
like the type level. The borrowed-return clone semantics of `field_self` is
now documented in one place (`ExpandReturnDecl::field_self` /
`FunctionDecl::expand_return`); a rename for the clone case was considered
out of scope.

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

**Resolution (2026-07-13): generalized rather than special-cased.** A
dedicated error declaration (and an `.internal()` marker on `ptr_class!`) were
both rejected — the former makes errors special for no reason, the latter
keeps pretending there is Kotlin wrapping where none happens. Instead,
boundary decls now work for **rust-side-only types**: `param_expand!` /
`return_expand!` accept types not declared in any package. Such a type is
always built from its ingredients on input and decomposed into its fields on
output (including the `Result<_, E>` error channel); it never materializes in
Kotlin — no class, no handle, no `freePtr`. The one structural restriction:
`variant_self()` / `field_self()` hard-error (there is no Kotlin object to
pass or deliver). Derived interfaces (`<T>Handler`, `<T>Builder`) land in the
base package. zenoh-flat-jni's `Error` is now exactly this: one
`return_expand!(Error).field(fun!(error_get_message).name("message"))` line;
the dead Kotlin class, its uncallable `message()` method, and its `freePtr`
extern are gone.

### M5. "wrapper" is overloaded

`ScalarTypeWrapperDecl` = a conversion rule for a *concrete* type;
`GenericTypeWrapperDecl` = a rule for Rust *wrapper types* (`Option`/`Result`).
Their direction pairs are also named inconsistently: `on_param`/`on_return` vs
`input`/`output` vs `param_expand`/`return_expand`.

**Resolution (2026-07-13): the wrapper tier is gone; conversions are named
functions.** Investigation deepened the finding: the scalar decl's position
words (`on_param`/`on_return`) were actively misleading (converters are
direction-things — the "return" body also runs for callback *arguments*), the
generic tier had zero users and zero tests anywhere, and the injected
`syn`/`quote` expression bodies were unchecked splices (most of C3). The
replacement is `convert!`:

```rust
.convert(convert!(Millis)
    .input(fun!(millis_from_long))    // fn(i64) -> Millis (or Result<Millis, E>)
    .output(fun!(millis_value)))      // fn(&Millis) -> i64
```

— ordinary `#[prebindgen]` functions, type-checked by rustc; the wire and
Kotlin surface derive from the signature's other-side type through its own
converter chain (no verbatim strings). `convert!` is the type's **canonical
single-value aspect**, orthogonal to the `expand_*!` boundary decls: a type
may declare both — expansion wins at the fn boundaries where declared, the
conversion serves everything else (`Result`-Ok, struct fields, `Option`/`Vec`
nesting). Direction words (`input`/`output`) are kept deliberately, against
the expand family's position words, and documented.

Conversion fns the flat crate doesn't provide live in a **helper crate**
(same-crate `#[prebindgen]` markers cannot feed the same crate's build.rs —
macro expansion runs after it): the helper's item stream is chained into the
same `Registry::from_items` call, per-item origin crates are recorded, and
generated calls qualify each fn with its defining crate. Proven in covertest
via `examples/covertest-helpers`.

Scope cut, recorded: deleting `GenericTypeWrapperDecl` removes user-defined
*wildcard* wrapper patterns (`MyWrapper<_>`). The rank tables and the built-in
`Result<_,_>` peel remain internal, so a function-based wildcard door can be
reopened when a real consumer needs one; concrete instantiations are already
coverable by `convert!`.

**Follow-up (2026-07-14): four conversion sources.** Requiring a
`#[prebindgen]` fn was too narrow — the conversion may already exist as a
trait impl or as a plain fn in the binding crate. Each direction now picks
one of: a `#[prebindgen]` fn (`.input`/`.output`), an `Into`/`From` impl
(`.input_from(ty!(i32))`/`.output_into(...)`), a fallible `TryInto`/`TryFrom`
impl (`.input_try_from(...)` — the associated `Error` routes to `onError`),
or a binding-local callable (`.input_with(ty!(String), path!(crate::f))`,
fallible via `.input_try_with(repr, error, path)` with the error type stated
in the decl — works because the generated file compiles inside the binding
crate; closes the "helpers need a separate crate" gap entirely). All
three new kinds are demonstrated in covertest (`Celsius`/`Percent`/`Label`)
with JVM assertions including the TryFrom error path.

**Follow-up (2026-07-14): `set_source_module` removed entirely.** With
per-item origins recorded for every named item at multi-source ingestion,
the setter had no information the registry didn't already own. The first
source doubles as the default module (declared-but-unindexed types,
deliberately unmarked types); item-level registries (`from_items`, tests)
use `Registry::set_default_module`. `constant_expr` getters now glob-import
every source module, so binding-defined expressions compose items from all
sources. One less setting, one less way for the declaration to disagree
with the data.

**Follow-up (2026-07-14): `from_sources`/`add_source` removed — origins
ride the stream.** Taking whole `Source` objects bypassed the item-stream
design (`from_items` exists precisely so callers prefilter by group, cfg,
or any hand-rolled chain). Instead, `Source` stamps its crate name into
each yielded item's `SourceLocation` (`crate_name: Option<String>`; not
part of the captured JSONL — the proc-macro can't know the name consumers
see), so streams stay plain `(syn::Item, SourceLocation)` values that
compose with ordinary iterator combinators:
`Registry::from_items(flat.items_all().chain(helpers.items_all()))`.
`from_items` records per-item origins and the first-seen origin as the
default module; duplicate-name errors still name both crates. One
constructor, full prefiltering, multi-source for free.

**Follow-up (2026-07-15): rename override moved to the `Source` builder —
`Registry::set_default_module` removed.** The stamped origin is the source
crate's `CARGO_PKG_NAME` at capture time, which breaks when the binding
crate renames the dependency (`cov_helpers = { package =
"covertest-helpers", .. }`). The registry-level override could only fix
the *default* (first) module — incomplete with chained multi-source
streams — so the override now lives where the stamp is made:
`Source::builder(dir).crate_name("cov_helpers")`, per-source and therefore
complete. It also fixes the features-guard qualification
(`cov_helpers::FEATURES`) in one move. Proven in covertest: the helpers
dependency is renamed in Cargo.toml on purpose. Hand-built origin-less
streams (tests) stamp `SourceLocation.crate_name` directly — the same
mechanism production uses.

### M6. Two "package" concepts

`.package(package!("bytes"))` adds a declaration batch under a *sub*package;
`set_package_prefix` sets the actual package. `package!()` (no args) means
"no subpackage", not "no package".

> **Resolution (2026-07-14): intended design, no change.** The
> absolute-base + relative-subpackage structure is deliberate: one knob
> (`set_package_prefix`) relocates the whole binding tree, and a declaration
> cannot accidentally land outside it. The naming friction this item flags is
> real but already mitigated: the `set_` convention marks the prefix as a
> generator-wide setting vs. `.package(...)` as a surface declaration
> (`config.rs` module doc), and the `package!` / `PackageDecl` docs state
> explicitly that the argument is relative to `set_package_prefix` and that
> `package!()` is the base package — not the JVM default package. The one
> rename that would dissolve the `package!()` misreading (`subpackage!`) was
> rejected: it trades this asymmetry for a method/macro mismatch
> (`.package(subpackage!("model"))`) and a breaking rename across consumers.

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
`.expand(param_expand!(T).variant(fun!(ctor)).variant_self())` and
`.expand(return_expand!(T).field(fun!(get)).field_self())`. A
`.field(fun!(x))` inherits its Kotlin name from the class member declaration
of the same fn (explicit `.name()` wins), so the name is written once. The
declarations remain order-independent: `JniGen` stores boundary decls raw and
assembles the expansion sets at the point of use (the `Prebindgen` trait's
`expansions()`/`deconstructors()` hooks now return by value).

**Follow-up (2026-07-13):** the two acceptors were unified into a single
`JniGen::expand(impl Into<ExpandDecl>)`, the direction carried by the decl
object — the same pattern as `PackageDecl::class(impl Into<ClassDecl>)` with
its four class-kind macros. The `param_expand!` / `return_expand!` macros are
unchanged; `ExpandDecl` deliberately has no `From<syn::Type>` (a bare type
doesn't say which direction it describes).

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
Rust. The consumer comment predates that check.

**Resolution (2026-07-13): keep the hard error, no code change.** Declaration
order defines leaf order everywhere else, so silently re-sorting the `_self`
leaf would make declaration order and emitted order diverge; the error message
already names the fix.

### I3. Flipped receivers, hidden temporal coupling

`registry.write_rust(&jni, path)` but `jni.write_kotlin(&registry, path)` — the
two halves of one generation run with inverted receivers, and `write_kotlin`
silently requires the resolution `write_rust` performed. Nothing in the types
enforces the order.

> **Resolution (2026-07-15): fixed — `Generation` object.**
> `Registry::resolve(self, adapter)` consumes both halves (scan + plans +
> type resolution) and returns `Generation<E>`; `gen.write_rust(path)` and
> `gen.write_kotlin(root)` (an inherent impl on `Generation<JniGen>`) are
> pure emissions on ONE receiver. The resolve-before-write coupling is
> enforced by construction — the old entry points no longer exist — and the
> two writes commute: the write phase was verified to take `&Registry`
> (all mutation happens in resolve), the adapter holds no interior
> mutability, and a test asserts byte-identical output with the calls
> flipped. The adapter is named once (`resolve(jni)`) instead of being
> re-passed to every call; cbindgen consumers chain
> `from_items(…)?.resolve(cbindgen)?.write_rust(…)?`. Escape hatches:
> `gen.registry()` / `gen.adapter()`.

### I4. Inconsistent argument kinds for the same concept

`ignore_fun(FunctionDecl)` vs `ignore_class(syn::Type)`. Several ways to spell
a function reference coexist (`fun!(x)`, `FunctionDecl::new(ident!(x))`, `pq!`
for params) — the consumer file uses three of them.

> **Resolution (2026-07-15): fixed — one `.ignore(impl Into<IgnoreDecl>)`
> acceptor**, the kind carried by what you built (mirroring `.class` /
> `.constant` / `.expand` / `.convert`): `fun!(x)` / `ty!(T)` /
> `constant!(X)` for exact items, plus `matching(|n| …)` — a **universal**
> name-family predicate covering ANY undeclared item kind (fn, struct/enum,
> const). Kind-agnosticism costs nothing: prebindgen items live in one flat
> namespace, so a name filter needs no kind; the fn-only
> `ignore_funs_where` from I7's first cut is superseded (core hook renamed
> `ignored_name_predicates`, applied in all three skip-warning filters).
> The four `ignore_*` methods are deleted. Surface overrides on ignored
> decls (`.name()`, expand overrides, const value sources) are decl-time
> panics — an ignore names a bare source item. Spelling standard recorded:
> decl macros (`fun!`/`ty!`/`constant!`) are the normal form,
> `FunctionDecl::new` / `ConstDecl::named` the runtime loop forms; the
> `pq!` spelling survives only in the Cbindgen builder
> (`ignore_function(syn::Ident)` / `ignore_type(syn::Type)`), a different
> pre-decl-macro model, out of this item's scope.

### I5. Member support varies by class kind for implementation reasons

`data_class` forbids `.fun`/`.constructor` ("no handle to hang a method on")
while `value_class` — equally handle-less — allows them. `.kotlin_type()` (map
onto an existing Kotlin type) exists only on data/value classes; enums and ptr
classes can't be mapped onto existing types.

> **Resolution (2026-07-15): fixed — the matrix now follows two rules.**
>
> | capability | ptr | data | value | enum |
> |---|---|---|---|---|
> | `.fun` / `.constructor` | ✓ | ✓ *(new)* | ✓ | ✗ by rule |
> | `.kotlin_type()` | ✗ by rule | ✓ | ✓ | ✓ *(new)* |
>
> **Rule 1 — members**: meaningful wherever an instance can re-enter Rust.
> A ptr receiver re-enters as its handle, a value receiver as its blob, a
> data receiver as its **field leaves** — the same call-site destructuring
> a data-class *parameter* already got, rebased to `this` (implemented;
> zero new wire machinery, the extern is unchanged). An enum value is a
> bare scalar with no object identity — a "method" is just a free fn.
> **Rule 2 — `.kotlin_type()`**: meaningful wherever the generated class is
> pure surface. Data/value/enum all qualify (the enum mapping only requires
> the target type to honor the `fromInt`/`.value` protocol; implemented, no
> file generated). A ptr class is NOT pure surface — it owns a lifecycle
> contract (NativeHandle base, `ptr` slot, `close()`, lock protocol, paired
> `freePtr`) an existing type can't be assumed to honor.
> `.kotlin_type()` + members is rejected (a mapped type has no generated
> class to hold them; assert added to data AND value). Demonstrated in
> covertest: `Payload.labelLen()` — the free fn moved into the data class,
> regen diff is exactly the receiver rebasing to `this.id, this.seq, …`.

### I6. Three overlapping const mechanisms

`constant`, `constant_fun`, `constant_expr`: the flagship consumer only needed
`constant_expr` for the non-trivial cases; the superseded `constant_fun` /
path-alias rounds shipped anyway.

> **Resolution (2026-07-15): one mechanism, source-kinded — aligned with
> `convert!`.** Constants and conversions are the nullary and unary edges of
> one source-kind vocabulary, so the three acceptors collapsed into one
> `PackageDecl::constant(ConstDecl)` with the source carried by the decl:
>
> | source | `convert!` (unary) | `constant!` (nullary) |
> |---|---|---|
> | prebindgen item | — | `constant!(MAX_LEN)` (bare) |
> | prebindgen fn | `.input_fun(fun!)` / `.output_fun(fun!)` | `.fun(fun!)` |
> | trait impl | `.input_from` / `.output_into` / `_try_` | — (nothing flows in) |
> | binding-local fn | `.input_with(ty!, path!)` / `_try_with` | `.with(ty!, path!)` |
> | expression | — | `.expr(ty!, expr!)` |
>
> `convert!`'s bare `.input`/`.output` were renamed `input_fun`/`output_fun`
> so the fn source is named uniformly. The `.expr` source (new root `expr!`
> macro, one simple argument like `ty!`/`path!`) exists ONLY for constants:
> an expression binds no arguments exactly when nothing flows in — a unary
> expr source would resurrect the value-ident closure convention M5 deleted.
> `.with` lowers to `.expr(path())` internally, but stays a distinct decl
> (same "(stated type, named callable)" pair as convert's `_with`). Macro
> style rule adopted: decl macros take ONE simple argument — the
> `constant_expr!(N: T = e)` val-decl DSL arm was rejected and deleted;
> loops use `ConstDecl::named(name).expr(ty, expr)`. All four sources
> demonstrated in covertest (`COVER_MAGIC`/`COVER_TAG_RUNTIME`/
> `COVER_VERSION`/`COVER_BANNER`); zenoh's 106-val loop unchanged in
> spirit, one call shorter in form.

### I7. Mixed failure modes with no policy

Dotted `.name()` → panic at decl time; builtin scalar-wrapper pattern → panic;
typo'd `fun!(name)` → **cargo warning + silent omission** (`registry.rs`
"declared function not found"); non-`Copy` value_class → compile error inside
`generated_bindings.rs`; malformed value-blob bytes → runtime Kotlin error;
wildcard rank > 3 → silent `return None`. A *declared*-but-missing function is
explicit intent gone wrong and should be a hard error.

> **Resolution (2026-07-14): fixed — declared-but-missing is now a hard
> error.** Policy: a *declaration* names a specific item, so its target
> being absent is always a bug and fails the scan
> (`ScanError::DeclaredNotFound`) — declared functions, `convert!`/boundary
> helper functions, and declared constants; all missing names are collected
> into ONE error (sorted) so a broken build.rs is fixed in a single pass.
> An *ignore*, by contrast, is bookkeeping about something deliberately
> unused — a stale entry keeps its soft `cargo:warning`. jnigen's
> boundary-referenced fns (`expand_*!` ctors/accessors) were rerouted from
> the ignore channel to the helper channel so a typo'd `fun!` inside an
> expand decl is also a hard error. Two of the flagged examples were
> already stale: the scalar-wrapper panic died with the wrapper tier (M5),
> and the rank > 3 silent `None` died with the rank resolver (the
> structural resolver hard-errors via `ResolveError::Unresolved`).

---

## Inconvenient

### C1. `.name()` needed on nearly every member

Default Kotlin name is `snake_to_camel` of the full Rust ident —
`sample_get_payload` becomes `sampleGetPayload` *on the Sample class*, so
effectively every `.fun()` carries a manual `.name()`. The generator knows the
class's Rust name; it could strip `<class>_get_` / `<class>_` prefixes for
members, or offer a member-name mangle hook (five global mangle hooks exist;
none applies here).

> **Resolution (2026-07-15): fixed — namespace-relative member names +
> `set_member_name_mangle`.** The default rule is a statement about
> namespaces, not string munging: a flat Rust crate spells the type
> namespace inside the ident (`storage_len`, `keyexpr_intersects`)
> precisely because flat FFI has no method syntax; a Kotlin member already
> lives in its class's namespace, so the default removes exactly that
> prefix — nothing else — and camel-cases the rest. Matching is
> underscore-insensitive on both sides (class `KeyExpr` strips `keyexpr_*`
> AND `key_expr_*`); no prefix ⇒ full-ident camelCase as before; the
> extern/`JNINative` tier keeps the full ident (it's the JNI symbol).
> `get_`/`new_` dropping is deliberately NOT automatic — zenoh itself keeps
> `get` in `getSchema`/`getStr` and drops it in `id` — those are style
> choices for `.name()` or the new **sixth mangle hook**
> `set_member_name_mangle` (same `Fn(&str) -> String` shape as the other
> five, receives the stripped camelCase default, skipped by `.name()`,
> order-independent — `ClassMember` now stores the raw override and the
> effective name derives at point of use; expand `.field(fun!(x))`
> inheritance follows it automatically). Validated by byte-identical
> regen after deleting the now-redundant `.name()`s: 10 in covertest, 13
> in zenoh-flat-jni (the 45 remaining are genuine renames).

### C2. No bulk/pattern ignore

53 hand-enumerated `ident!(encoding_const_*)` calls exist only to silence
undeclared-fn warnings — and the same 53 names appear a second time (lowercase)
to build the consts. No prefix/regex ignore, no way to derive one list from the
other.

> **Resolution (2026-07-14): fixed —
> `.ignore_funs_where(|name| name.starts_with("encoding_const_"))`.** A
> plain-Rust closure predicate (no glob/regex mini-dialect to learn), in
> the style of the existing mangle hooks; core hook
> `Prebindgen::ignored_function_predicates` (`NamePredicate` alias).
> Semantics: a predicate is a *filter*, not a claim — matching undeclared
> fns are acknowledged skips, a declared fn matching a predicate is
> unaffected (declaration wins), and a predicate matching nothing is
> silent (unlike an exact-name ignore, which warns when stale — match
> counts legitimately vary across feature configs). zenoh-flat-jni's
> 53-line loop is now one line; covertest demonstrates both mechanisms
> (`storage_get_into_*` via predicate, the two loners via `ignore_fun`).

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

> **Resolution (2026-07-15): fixed — the Kotlin root is generator-owned.**
> `write_kotlin` (via `kt::write_files`) deletes and recreates `kotlin_root`
> on every run, so stale files from a renamed package or removed
> declaration can never linger and the identical `remove_dir_all`
> boilerplate is gone from all three consumers. The contract is documented
> on both entry points: point the root at a dedicated directory (the
> established `kotlin/generated/` convention), never at hand-written
> sources. A marker-based selective prune was considered and rejected as
> too complex for the problem — whole-directory ownership is the simple,
> predictable rule.

### C5. Parameters named by bare ident

`param_expand_self(pq!(key_expr))` — no decl-time validation against the
function's real signature; typos surface late (or as warnings).

> **Resolution (2026-07-15): already fixed by M2/I7/I3 — no change; test
> gaps filled.** The example API is gone (M2 replaced the modal
> `param_expand_*` methods). Every bare-ident reference a declaration
> writes is now checked against the registry when `Registry::resolve`
> runs — the earliest moment signatures exist (decl objects are built
> before any source is read, so literal decl-time validation is
> structurally impossible; resolve-time hard errors are what the critique
> was actually asking for). The map, all hard `Err`s out of `.resolve()`:
>
> | reference | validation |
> |---|---|
> | declared fn / member / const / helper fn | `ScanError::DeclaredNotFound` (I7) |
> | `.expand_param("name", …)` param string | `ExpandError::UnknownParam` |
> | per-fn expand decl type | `ParamTypeMismatch` / `ReturnTypeMismatch` (M2) |
> | `expand_param!` variant ctor | `UnknownConstructor` / `TargetMismatch` / `NoConstructor` |
> | `expand_return!` field accessor | `UnfoldError::UnknownAccessor` / `AccessorTargetMismatch` / `RecordNotAccessor` |
> | output leaf names | `DuplicateLeafName` (explicit-literal rule) |
> | exact `.ignore(...)` entries | warning **by design** (an ignore is bookkeeping, not a claim) |
>
> Gap-fill: `UnknownConstructor`, ctor `TargetMismatch`, and
> `UnknownAccessor` had no direct tests — added (core expand/unfold test
> modules); the rest were already covered.

### C6. Global-only lock toggle

`set_emit_handle_locks(bool)` is all-or-nothing; the known per-call lock
overhead on hot paths can't be traded per class or function.

> **Resolution (2026-07-15): premise tested and found false — toggle
> re-documented as verification-only, no granularity added.** The "known
> per-call lock overhead" was measured (perftest Kotlin leg, N = 5M per
> op, best of two runs per config, JDK 21 / macOS arm64): locks-on vs
> locks-off deltas ranged −3.2%…+3.6% with **mixed sign** — i.e. within
> run-to-run noise (which itself exceeded 30% on the cheapest op across
> passes). The tightest bound comes from `put native.null` (~33 ns/call,
> no string, no real work): ~1 ns for the entire uncontended
> `withSortedHandleLocks` pair + closed-handle guard. Every op doing real
> work is dominated by the JNI crossing. Per-class/per-fn lock granularity
> is therefore a knob without a payoff and is rejected;
> `set_emit_handle_locks(false)` stays global, documented as existing so
> anyone can independently re-verify this claim on their own workload.

### C7. No introspection

Canonical inputs/outputs "AUTO-APPLY" at a distance; the only way to learn what
a decl produces is to run generation and diff. The extensive prose comments in
the consumer's build.rs are compensating for a missing explain/dry-run mode.

> **Resolution (2026-07-15): fixed — `Generation::<JniGen>::report()`.**
> The missing explain mode, computed from the resolved registry after
> `resolve()`: per package/class every wrapper's FINAL Kotlin signature —
> rendered through the same `render_wrapper_fn` the emitters use, so it
> cannot drift from real output — plus `shaped by:` provenance (param
> expansions with variant lists, return decompositions with leaf names and
> delivery, error decompositions), then the type table (kind / FQN / wire),
> `convert!` sources, and rust-side-only types. Deterministic; consumers
> commit `kotlin/REPORT.md` next to the regen (covertest + zenoh-flat-jni),
> so a decl's effect is reviewable in a PR without reading generated Kotlin
> and drift is caught by regen-check. Placement is the `write_kotlin` seam
> (I3): an inherent method on `Generation<JniGen>`, not a core trait hook —
> the *pattern* (describe your resolved surface) is adapter-universal, the
> *content* is intrinsically destination-language vocabulary, so each
> adapter implements its own (`Generation<Cbindgen>::report()` is a natural
> future twin); a language-neutral core dump was rejected as describing the
> machinery rather than the surface.

---

## Incomplete

### N1. Doc comments dropped

`#[prebindgen]` items carry `///` docs (e.g. `session_put`), but the generated
Kotlin functions have no KDoc — a real gap for a published Maven artifact.

> **Resolution (2026-07-15): fixed — KDoc = author prose + shape notes.**
> The `///` docs were already captured (as `#[doc]` attrs in the JSONL) and
> the `kt` model already rendered `kdoc` — the gap was pure plumbing. Every
> wrapper fn, class member, companion factory, data/enum/value/ptr class,
> and const `val` now carries the Rust item's doc verbatim; where the
> generator writes a framework line (typed handle, enum surface, value
> blob, const mirror), author prose comes first, framework line after. On
> top of the prose, **shape notes** document the REAL prototype after all
> expansions (user addition): every position a plan reshaped gets a
> caller-phrased note — expanded params ("pass EITHER its `summary_new`
> inputs OR an existing `Summary` — the selector chooses the arm"),
> decomposed returns ("the builder callback receives (`count`, `total`)"),
> error decompositions ("`onError` receives `je` plus the decomposed
> `StorageError` (…)"). Sourced from the same resolved plan maps as C7's
> report, but phrased for the caller rather than as provenance. An
> undocumented, unshaped fn keeps no KDoc. Callback/handler interfaces are
> out of scope (framework-documented protocol types); enum *variant* docs
> are skipped (entries carry no kdoc slot).

### N2. Acknowledged coverage holes

`Vec<closeable-handle>` is an explicit `panic!`; `Option<ptr_class>` params
can't be built through by the recursive input fold (forces Sample's
identity-only input); opaque-handle consts rejected; `&mut [T]` unsupported;
output-side `Option<prim>` boxing deliberately not done.

> **Resolution (2026-07-15): catalogued in
> [issue #33](https://github.com/milyin/prebindgen/issues/33) — feature
> work, not API-shape defects; tracked outside this cleanup PR.** Triage:
> *deferred gaps* — `Vec<handle>` INPUT (loud panic with hint; output side
> already folds Kotlin-side), `Option<ptr_class>` through the recursive
> fold (loud `UnsupportedOptional`; per-fn identity-only opt-out is the
> workaround), `&mut [T]`/`&mut T` (falls through to the generic
> unresolved-type error — a targeted reject-with-hint is the issue's first
> sub-task, per the I7 policy); *by-design exclusions* — opaque-handle
> consts (a shared closeable `val` is semantically wrong; the loaning
> factory + `constant!(N).expr(…)` idiom is the pattern) and output-side
> `Option<prim>` boxing (inherent to the single-value return channel;
> revisit only with a benchmark). Each entry in the issue carries the
> failure-mode anchor, the workaround, and an implementation sketch.

### N3. API split across branches

**Resolution (2026-07-13): already resolved before this effort started** —
`const_support` was squash-merged to `main` as PR #31 (`e90ce85`).

### N4. Public-surface hygiene

`JniGen.source_module`/`package` are `pub` fields — direct writes bypass
`set_package_prefix`'s trimming; `pub mod jni` exposes undocumented internals
(`MethodEntry`, `WireBody` plumbing) beyond the curated `lang::` re-export list.

> **Resolution (2026-07-15): fixed — one field sealed; the rest was
> already sealed by earlier rounds.** Audit of the flagged leaks:
> `source_module` no longer exists (deleted with `set_source_module`,
> M5 follow-up); `WireBody` died with the wrapper tier (M5); the module
> leak is structurally closed — `lib.rs` declares `pub(crate) mod api`, so
> `lang::jnigen::jni` is not a public path and the ONLY public surface is
> the curated `lang::{…}` re-export list (`MethodEntry`/`TypeConfig`'s pub
> fields are crate-internal convention, unreachable from outside). The one
> real remaining leak was `JniGen.package: pub String` — `JniGen` itself
> is publicly reachable, so a direct field write compiled and bypassed
> `set_package_prefix`'s trimming, corrupting every derived form
> (FindClass paths, extern idents, Kotlin package lines). Sealed to
> `pub(crate)`. Sweep of the rest of the curated list (decl types,
> `Cbindgen`, `JniBindingError`, `CachedIfaceMethod`) found no other pub
> fields; `KotlinFile`'s model fields are its deliberate surface. A stale
> doc-comment path in covertest (`lang::jnigen::jni::decl`) corrected.

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
