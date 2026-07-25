# Data-carrying enums (sum types)

Status: **proposal**. Tracked by umbrella issue
[#151](https://github.com/milyin/prebindgen/issues/151); the staged issues are
[#146](https://github.com/milyin/prebindgen/issues/146)–[#150](https://github.com/milyin/prebindgen/issues/150),
listed in [Implementation stages](#implementation-stages).

This document defines how a `#[prebindgen]` enum **with payload variants** crosses a language
boundary: one language-neutral model in `api/core`, one lowering per adapter — native where the
target language has sums (Kotlin `sealed`), simulated where it does not (a C tagged union).

## 1. Problem

A source crate cannot currently express "exactly one of these alternatives". Both adapters reject
payload variants outright:

- `lang::Cbindgen` — `assert_unit_variants` guards the mirror emission and both converters
  (`src/api/lang/cbindgen/trait_impl.rs:231`, `:660`, `:1159`).
- `lang::JniGen` — `enum_class!`'s contract is "the enum must be unit-variant only"
  (`src/api/lang/jnigen/jni/decl.rs:739-748`).

So authors encode sums as products and demote the invariant to prose. Two live examples in
zenoh-flat:

```rust
// RecoveryConfig — the conflict is resolved by precedence, silently.
pub struct RecoveryConfig {
    /// … takes precedence over `heartbeat`.
    pub periodic_queries: Option<Duration>,
    pub heartbeat: bool,
}

// ReplyStruct — "exactly one of sample or error" is a doc comment;
// both-None and both-Some are representable.
pub struct ReplyStruct {
    pub sample: Option<SampleStruct>,
    pub error: Option<ReplyErrorStruct>,
}
```

A caller who sets both `RecoveryConfig` fields gets `heartbeat` ignored with no diagnostic; a caller
who checks only `ReplyStruct::sample` reads an error reply as an empty success. These are
[zenoh-flat #31](https://github.com/ZettaScaleLabs/zenoh-flat/issues/31) and
[#30](https://github.com/ZettaScaleLabs/zenoh-flat/issues/30). flat's README already forbids bending
its shapes to generator limits (§*Bindings choose; flat stays neutral*), so the fix belongs here.

Sum types are also not exotic: Kotlin, Swift, Rust, TypeScript and Python (tagged unions) all express
them directly. Exposing the concept once, neutrally, is strictly better than each source crate
hand-rolling a different simulation.

## 2. The model: a tag plus variant groups

> A sum is a **tag** — which alternative is live — plus one **leaf group per variant**: that
> variant's payload, expressed in leaves the pipeline already supports. Exactly one group is live;
> every other group is inert.

Nothing else is new. A group's leaves are ordinary leaves: scalars, `String`, `enum_class`
discriminants, opaque handles, value blobs, nested data classes. The two adapters differ only in
*where* the groups are overlaid:

| Adapter | Groups overlaid in | Inert groups |
|---|---|---|
| `JniGen` | the **signature** — tag slot + every group's slots, side by side | wire-defaulted (`0`, `false`, `null`, `""`) |
| `Cbindgen` | **memory** — a `#[repr(C)]` enum with payload variants, rendered by cbindgen as a tag + `union` | not present (union overlay) |

A unit-only enum is the degenerate case: every group is empty, so the lowering collapses to "a tag",
which is exactly today's `enum class` / C enum. That is why **existing enums are unaffected** — the
new machinery reduces to the old behavior rather than replacing it.

Two facts make this cheap to build:

1. **The type graph already walks variant payloads.** `Registry::scan_enum` and
   `Registry::immediate_edges` register every variant field type in both directions
   (`src/api/core/registry.rs:1177-1189`, `:1285-1292`), so payload converters resolve with no core
   change.
2. **Both directions already gate leaf groups — with `N = 1`.** An `Option<Struct>` input crosses as
   a synthetic `present: Boolean` plus field slots that are wire-defaulted when absent
   (`FlatLeaf::is_present_flag`, `src/api/lang/jnigen/jni/emit/flat_input.rs:306`); an
   `Option<nested>` output does the same in the `fromParts` bridge
   (`PlanFieldKind::Nested { optional }`, `src/api/lang/jnigen/jni/struct_plan.rs:66`). A sum is that
   mechanism with an `Int` tag instead of a `Boolean` flag and `N` groups instead of one.

### Chosen lowering: flattened, always

JniGen lowers sums as **tag-gated leaf groups in the signature**, never as a whole `JObject` built
Rust-side. Rejected alternative: give the sum an ordinary terminal converter with a `JObject` wire
(`match` → one `call_static_method("fromParts", …)`). That would need no core changes and would work
in every position immediately, but it reintroduces the per-crossing JVM object and per-field JNI
reflection that this codebase deliberately removed (the `fromParts` flattening was worth 2.4×,
leaf folds 3–8×), and it would force an exemption to the rule that a data class flattens *completely*
or generation fails (`src/api/lang/jnigen/jni/decl.rs:814-819`). `ReplyStruct` is on the
per-message path, so the slow shape would not have survived anyway.

## 3. Locked decisions

| Decision | Rule |
|---|---|
| **Tag numbering** | Declaration order, `0..N-1`. Payload enums carry no `repr` — a `repr` is a wire detail the neutral tier must not name. Unit enums keep explicit-discriminant support through `enum_discriminant_values` (`src/api/lang/jnigen/util.rs:98`), which **moves into core** so both adapters share one implementation instead of two. |
| **Leaf naming** | Variant field leaf = `<variant_snake>_<field>`, matching the existing nested-prefix convention (`payload_id` for the nested field `payload.id`). Tuple field ⇒ `<variant_snake>_0`. The synthetic tag leaf = `<field>__tag`, using the same double-underscore marker as the existing `<field>__present` gate. Core `DeconSpec` leaf names keep their `__` chain separator. |
| **Kotlin surface** | `sealed interface E` with the variant classes **nested inside it** — `data class PeriodicQueries(val period: Long) : E`, `data object Heartbeat : E` — so variant names cannot collide package-wide. Tuple payload fields are named `v0`, `v1`, …. A `fromParts(tag, …)` companion on the interface reassembles from the tag + slots. |
| **Unit-only enums** | Unchanged path. Handing a payload enum to `enum_class!` / `.enum_type()` is a **hard error** naming `sealed_class!` / `.tagged_union()` — no silent upgrade, matching the "invalid = ERROR" declaration policy. |
| **Handle payloads** | Allowed. `ReplyResult` (flat #30) needs them: `SampleStruct` carries `KeyExpr`, `ZBytes`, `Encoding`. A tag-gated handle leaf reuses `FlatLeaf::handle_nullable`, which already means "gated by an optional ancestor" (`flat_input.rs:315-317`), and joins the existing N-ary `withSortedHandleLocks` collection unchanged. |
| **Optionality** | `Option<E>` keeps its own present-flag gate; the tag domain is never overloaded with a `-1 = absent`. Optionality and choice are independent facts and stay independent leaves. |
| **Invalid tag** | A tag outside `0..N-1` is a **binding error** routed through the existing `__JniErr` / `onBindingError` channel (Rust side) or an `IllegalArgumentException` from `fromParts` (Kotlin side). Never a panic across the boundary. |
| **No slot sharing** | Two variants with same-descriptor fields get distinct slots. Overlaying them would shrink the signature but couple variant order to wire layout; wire width is a later optimization, correctness is now. |

## 4. JniGen lowering

Running example:

```rust
pub enum RecoveryMode {
    PeriodicQueries(Duration),   // tag 0
    Heartbeat,                   // tag 1
}
pub struct RecoveryConfig {
    pub mode: Option<RecoveryMode>,
    pub retention_period: Option<Duration>,
}
```

### 4.1 Declaration

```rust
.package(package!("io.zenoh.jni")
    .class(sealed_class!(RecoveryMode)
        .variant(variant!(PeriodicQueries).name("Periodic")))   // optional per-variant rename
    .class(data_class!(RecoveryConfig)))
```

`sealed_class!(E)` is a fifth class kind beside `ptr_class!` / `data_class!` / `enum_class!` /
`value_class!`: one simple argument, sub-builder, `.name()`, per-variant `.variant(variant!(V))`, and
the shared `class_interface_methods!` (`.interface()`, `.interface_name()`, `.implements()`). Like
`enum_class!` it has no `.method` / `.constructor`: a sum value has no object identity Rust-side, so
a "method" on it is a free function taking it.

### 4.2 Kotlin surface

```kotlin
public sealed interface RecoveryMode {
    public data class PeriodicQueries(val period: Long) : RecoveryMode
    public data object Heartbeat : RecoveryMode

    public companion object {
        @JvmStatic
        public fun fromParts(tag: Int, periodicQueries_period: Long): RecoveryMode = when (tag) {
            0 -> PeriodicQueries(periodicQueries_period)
            1 -> Heartbeat
            else -> throw IllegalArgumentException("RecoveryMode: invalid tag $tag")
        }
    }
}
```

A sum nested in a data class does **not** call its own `fromParts` through JNI — the parent's
`fromParts` inlines the `when`, exactly as it inlines a nested data class today:

```kotlin
public data class RecoveryConfig(val mode: RecoveryMode?, val retentionPeriod: Long?) {
    public companion object {
        @JvmStatic
        public fun fromParts(
            mode__present: Boolean,
            mode__tag: Int,
            mode_periodicQueries_period: Long,
            retentionPeriod__present: Boolean,
            retentionPeriod_value: Long,
        ): RecoveryConfig = RecoveryConfig(
            if (mode__present) when (mode__tag) {
                0 -> RecoveryMode.PeriodicQueries(mode_periodicQueries_period)
                1 -> RecoveryMode.Heartbeat
                else -> throw IllegalArgumentException("RecoveryConfig.mode: invalid tag $mode__tag")
            } else null,
            if (retentionPeriod__present) retentionPeriod_value else null,
        )
    }
}
```

### 4.3 Output path (Rust → Kotlin)

`PlanFieldKind::Sum { tag_slot, variants }` joins the shared bridge plan
(`src/api/lang/jnigen/jni/struct_plan.rs`). The Rust encoder emits **one `match`** binding the tag
and every slot, inert slots filled by the existing `primitive_default_for_descriptor`
(`src/api/lang/jnigen/jni/emit/struct_out.rs:51`):

```rust
let (mode__present, mode__tag, mode_periodic_queries_period) = match &v.mode {
    None => (false, 0i32, 0i64),
    Some(RecoveryMode::PeriodicQueries(p)) => (true, 0i32, __out_Duration(p.clone())),
    Some(RecoveryMode::Heartbeat)          => (true, 1i32, 0i64),
};
```

The slots then ride the parent's single `call_static_method("fromParts", …)`. No JVM object is built
for the sum, and both sides enumerate the same slots in the same order because both walk one
`StructPlan` — the invariant that module already exists to hold.

### 4.4 Input path (Kotlin → Rust)

`FlatFieldNode::Sum` joins `Value` / `Nested` (`src/api/lang/jnigen/jni/emit/flat_input.rs:349`), and
`FlatInputPlan.root` generalizes from `FlatStructNode` to a struct-or-sum root so a **sum-typed
parameter** flattens too. Extern signature:

```kotlin
external fun sessionDeclareAdvancedSubscriber(
    …,
    recoveryPresent: Boolean,
    recoveryModePresent: Boolean,
    recoveryModeTag: Int,
    recoveryModePeriodicQueriesPeriod: Long,
    recoveryRetentionPeriodPresent: Boolean,
    recoveryRetentionPeriodValue: Long,
    errorSink: Any,
): Long
```

Rust reconstruct, with an invalid tag going to the binding-error channel rather than panicking:

```rust
let mode = if mode_present {
    Some(match mode_tag {
        0 => RecoveryMode::PeriodicQueries(__in_Duration(mode_periodic_queries_period)),
        1 => RecoveryMode::Heartbeat,
        t => return __jni_err(env, error_sink, format!("RecoveryMode: invalid tag {t}")),
    })
} else { None };
```

**The one real refactor.** `FlatLeaf::kt_access_tail` is a *suffix string* appended to the call-site
base expression (`.field ?: 0`, `?.seq != null` — `flat_input.rs:292-296`). A variant slot needs
`(mode as? RecoveryMode.PeriodicQueries)?.period ?: 0L`, which is not a suffix. So `kt_access_tail`
becomes a small expression template (cast + tail + default) while `FlatLeaf::kt_access(base)` and
`kt_call_arg(base)` keep their signatures — the three coordinated sites (native wrapper signature,
`JNINative` decl, Kotlin call-site destructure) keep agreeing by construction, which is the whole
point of that struct.

The tag itself is computed once per call site:

```kotlin
recoveryModeTag = when (recovery?.mode) {
    is RecoveryMode.PeriodicQueries -> 0
    RecoveryMode.Heartbeat -> 1
    null -> 0            // gated off by recoveryModePresent = false
}
```

### 4.5 Returns and callback arguments

A sum returned by a function (or delivered as a callback argument) is the one position that needs
core work, because `api/core/unfold.rs` is a deterministic **product** — "every record always runs
and contributes its leaf … there is no selector" (`src/api/core/unfold.rs:8-11`). It gains one:

- `LeafSource::VariantField { variant, member }` beside `Accessor` / `Field`
  (`src/api/core/unfold/plan.rs:63-78`), so a leaf can be reached through a variant pattern rather
  than a field chain.
- Per-leaf group membership plus a synthesized tag leaf, so the Rust emitter emits one `match` over
  the value instead of independent per-leaf expressions.
- `apply_sum_returns`, modeled on `apply_value_structs` (`src/api/core/unfold.rs:455`): for every
  declared function returning a declared sum (`E` / `&E` / `Option<E>` / `Vec<E>`), build a
  `fixed_builder` `UnfoldPlan` whose foreign builder is the sealed interface's `fromParts`.

`Option` / `Vec` layers need nothing new — they ride the existing `Shape` fold
(`src/api/core/shape.rs`), same as for a data class.

### 4.6 Rejections

Generation errors, each naming the offending path:

- A **recursive** sum (a variant payload reaching its own type) — the flatten plans are finite; there
  is no `jobject_input`-style escape hatch for sums in this design.
- An unresolvable payload leaf — the same error the data-class flattener already reports
  (`FlatInputError`, `flat_input.rs:432`), extended with the variant name in the path.

## 5. Cbindgen lowering

### 5.1 Declaration

`.tagged_union(ty)` beside `.enum_type(ty)` (`src/api/lang/cbindgen/builder.rs:398`), wired into the
same three places every declarator touches: the already-declared guard (`:306`), the universal
`.name()` override, and the `CurrentDecl` cursor for error messages (`:701`). `.enum_type()` on a
payload enum errors with a pointer to it.

### 5.2 The mirror type

`prereq_tagged_unions` mirrors `prereq_enums` (`src/api/lang/cbindgen/trait_impl.rs:650`), emitting a
`#[repr(C)]` enum whose payload fields carry **wire** types chosen by the same `mirror_field_wire`
policy `data_struct` uses (`trait_impl.rs:513`):

```rust
#[repr(C)]
pub enum z_recovery_mode_t {
    PeriodicQueries { period: u64 },
    Heartbeat,
}
```

cbindgen (pinned `=0.29.4`) renders that as a tag enum plus a union struct — the idiomatic C tagged
union, no hand-written header fragment. The `[enum]` section of each consumer's `cbindgen.toml`
controls the rendered variant/body naming; `examples/example-cbindgen`'s goldens are the reference
for what it actually produces.

### 5.3 Converters

`in_tagged_union` / `out_tagged_union` generalize `in_enum` / `out_enum`
(`trait_impl.rs:225`, `:1157`) from "match idents" to "match idents and convert each arm's fields",
reusing the per-field converters the resolver already produced:

```rust
pub(crate) fn __in_z_recovery_mode_t(v: z_recovery_mode_t) -> RecoveryMode {
    match v {
        z_recovery_mode_t::PeriodicQueries { period } =>
            RecoveryMode::PeriodicQueries(__in_Duration(period)),
        z_recovery_mode_t::Heartbeat => RecoveryMode::Heartbeat,
    }
}
```

### 5.4 Payload ownership

A tagged union crosses **by value**, like a `data_struct`. If any variant's payload wire owns memory
(`char *`, an opaque pointer), the declaration must also produce a typed
`z_<name>_drop(z_<name>_t *)` that frees the **active arm** — consistent with the existing typed
per-pointer drops. Phase B requires it: an owning payload without a drop is a generation error, not a
leak.

## 6. What core owns

| Piece | Home |
|---|---|
| `EnumShape::{Unit, Sum}` classifier — the single definition of "is this enum C-like" | `src/api/core/types_util.rs` |
| `SumSpec { key, source, variants: Vec<SumVariant> }`, `SumVariant { ident, tag, fields }`, `SumField { member, name, ty }` — the neutral description both adapters read | `src/api/core/types_util.rs` |
| `enum_discriminant_values`, moved out of `jnigen/util.rs` | `src/api/core/types_util.rs` |
| The unfold selector (`LeafSource::VariantField`, tag leaf, group membership, `apply_sum_returns`) | `src/api/core/unfold.rs` |

Everything else is adapter-local. Payload **wire** choice stays per-adapter, exactly as `ValueDecon`
leaves are adapter-built today (`Prebindgen::value_struct_decons`,
`src/api/core/prebindgen.rs:218`) — core describes the sum, adapters decide what its leaves look like
on the wire.

## 7. Test obligations

Per the every-feature rule, each increment ships its own coverage in the same PR; library tests alone
do not count.

- `examples/example-flat` — a payload enum exercised in all four positions (parameter, data-class
  field, return, callback argument), with a unit variant, a single-payload variant, a multi-field
  named variant, and a tuple variant.
- `examples/covertest-kotlin` — `ext.rs` + `src/lib.rs` + `build.rs` + `Test.kt`: round-trip each
  variant both directions; `Option<sum>` and `Vec<sum>`; a sum field beside already-flattened
  siblings; a handle payload (lock collection + close); an invalid tag reaching `onBindingError`.
- `examples/example-cbindgen` — regenerated `generated/` and `include/` goldens (aarch64 remains the
  CI blind spot) plus a C smoke test constructing and reading each arm, and a drop test for an owning
  payload.
- `cargo fmt` and `clippy --all-targets --no-default-features --all-features --deny warnings` clean
  before any PR.

## 8. Non-goals

- **`Result<T, E>` as an ordinary field.** `Result` peeling is the error channel; a source crate that
  wants a sum writes a named enum. (flat #30 becomes `ReplyResult { Sample(..), Error(..) }`, not
  `result: Result<..>`.)
- **Slot sharing** between same-descriptor variant fields — a wire-width optimization, deliberately
  deferred.
- **Recursive sums.**
- **Non-exhaustive sums** / forward-compatible unknown tags. A tag outside the declared range is an
  error, not a variant.

## 9. Implementation stages

| Stage | Issue | Scope | Depends on |
|---|---|---|---|
| **A** | [#146](https://github.com/milyin/prebindgen/issues/146) | core: `EnumShape` classifier, `SumSpec`, `enum_discriminant_values` moved into core, the three `assert_unit_variants` sites and `enum_class!`'s contract replaced by classifier-driven errors. No behavior change for existing enums. | — |
| **B** | [#147](https://github.com/milyin/prebindgen/issues/147) | cbindgen: `.tagged_union()`, `prereq_tagged_unions`, `in_/out_tagged_union`, the payload-ownership drop rule, goldens + C smoke test. | A |
| **C** | [#148](https://github.com/milyin/prebindgen/issues/148) | jnigen: `sealed_class!` + `variant!`, `TypeKind::Sum`, `TypeConfig.sum_cfg`, the sealed-interface emitter (sibling of `write_enum_classes`, `src/api/lang/jnigen/jni/kotlin_emit.rs:555`). | A |
| **D** | [#149](https://github.com/milyin/prebindgen/issues/149) | jnigen: tag-gated groups on both flatten paths — `FlatFieldNode::Sum`, the struct-or-sum root, the `kt_access` expression-template refactor, `PlanFieldKind::Sum`. Unblocks flat #31 + #11, and flat #30 in its struct-field form (`ReplyStruct { result: ReplyResult, … }`). | C |
| **E** | [#150](https://github.com/milyin/prebindgen/issues/150) | core + jnigen: sum-typed returns and callback arguments — the unfold selector. Needed when a function's **own** return (or a callback argument) is the sum, e.g. a `reply_get_result(&Reply) -> ReplyResult` accessor mirroring base's `Reply::result()`. | D |
