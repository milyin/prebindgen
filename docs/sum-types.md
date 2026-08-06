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
  (`prebindgen/src/api/lang/cbindgen/trait_impl.rs:231`, `:660`, `:1159`).
- `lang::JniGen` — `enum_class!`'s contract is "the enum must be unit-variant only"
  (`prebindgen/src/api/lang/jnigen/jni/decl.rs:739-748`).

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
[zenoh-flat #31](https://github.com/eclipse-zenoh/zenoh-flat/issues/31) and
[#30](https://github.com/eclipse-zenoh/zenoh-flat/issues/30). flat's README already forbids bending
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
   (`prebindgen/src/api/core/registry.rs:1177-1189`, `:1285-1292`), so payload converters
   resolve with no core change.
2. **Both directions already gate leaf groups — with `N = 1`.** An `Option<Struct>` input crosses as
   a synthetic `present: Boolean` plus field slots that are wire-defaulted when absent
   (`FlatLeaf::is_present_flag`, `prebindgen/src/api/lang/jnigen/jni/emit/flat_input.rs:306`); an
   `Option<nested>` output does the same in the `fromParts` bridge
   (`PlanFieldKind::Nested { optional }`,
   `prebindgen/src/api/lang/jnigen/jni/struct_plan.rs:66`). A sum is that mechanism with an `Int`
   tag instead of a `Boolean` flag and `N` groups instead of one.

### Chosen lowering: flattened, always

JniGen lowers sums as **tag-gated leaf groups in the signature**, never as a whole `JObject` built
Rust-side. Rejected alternative: give the sum an ordinary terminal converter with a `JObject` wire
(`match` → one `call_static_method("fromParts", …)`). That would need no core changes and would work
in every position immediately, but it reintroduces the per-crossing JVM object and per-field JNI
reflection that this codebase deliberately removed (the `fromParts` flattening was worth 2.4×,
leaf folds 3–8×), and it would force an exemption to the rule that a data class flattens *completely*
or generation fails (`prebindgen/src/api/lang/jnigen/jni/decl.rs:814-819`). `ReplyStruct` is on the
per-message path, so the slow shape would not have survived anyway.

## 3. Locked decisions

| Decision | Rule |
|---|---|
| **Tag numbering** | Declaration order, `0..N-1`. Payload enums carry no `repr` — a `repr` is a wire detail the neutral tier must not name. Unit enums keep explicit-discriminant support through `enum_discriminant_values` (`prebindgen/src/api/lang/jnigen/util.rs:98`), which **moves into core** so both adapters share one implementation instead of two. |
| **Leaf naming** | A variant field's **wire slot** = `<variantCamel>_<property>` (`range_low`, `exact_v0`), matching the existing nested-prefix convention (`payload_id` for the nested field `payload.id`). It is keyed on the **Kotlin** variant name, so a `variant!(V).name(...)` rename carries through to the slots. Under a parent field the slot is prefixed with it (`mode_periodicQueries_period`) and the synthetic tag leaf is `<field>__tag`, using the same double-underscore marker as the existing `<field>__present` gate; standing alone (a sum in return position) the tag leaf is just `tag`. Core's neutral `SumField::name` (`<variant_snake>_<field>`, tuple ⇒ `<variant_snake>_0`) is a language-agnostic label an adapter may ignore — the JNI wire slot is the rule above. Core `DeconSpec` leaf names keep their `__` chain separator. |
| **Kotlin surface** | `sealed interface E` with the variant classes **nested inside it** — `data class PeriodicQueries(val period: Long) : E`, `data object Heartbeat : E` — so variant names cannot collide package-wide. A named payload field keeps its camelCased name; tuple payload fields are named `v0`, `v1`, … (so a tuple variant `Exact(i64)` surfaces as `data class Exact(val v0: Long)` with the slot `exact_v0`). A `fromParts(tag, …)` companion on the interface reassembles from the tag + slots. |
| **Unit-only enums** | Unchanged path. Handing a payload enum to `enum_class!` / `.enum_type()` is a **hard error** naming `sealed_class!` / `.tagged_union()` — no silent upgrade, matching the "invalid = ERROR" declaration policy. |
| **Handle payloads** | Allowed. `ReplyResult` (flat #30) needs them: `SampleStruct` carries `KeyExpr`, `ZBytes`, `Encoding`. A tag-gated handle leaf reuses `FlatLeaf::handle_nullable`, which already means "gated by an optional ancestor" (`flat_input.rs:315-317`), and joins the existing N-ary `withSortedHandleLocks` collection unchanged. |
| **Optionality** | `Option<E>` keeps its own present-flag gate; the tag domain is never overloaded with a `-1 = absent`. Optionality and choice are independent facts and stay independent leaves. |
| **Invalid tag** | A tag outside `0..N-1` is a **binding error** routed through the existing `__JniErr` / `onBindingError` channel (Rust side) or an `IllegalArgumentException` from `fromParts` (Kotlin side). Never a panic across the boundary. |
| **No slot sharing** | Two variants with same-descriptor fields get distinct slots. Overlaying them would shrink the signature but couple variant order to wire layout; wire width is a later optimization, correctness is now. |

## 4. JniGen lowering

Running example:

```rust
pub enum RecoveryMode {
    PeriodicQueries { period: Duration },   // tag 0
    Heartbeat,                              // tag 1
}
pub struct RecoveryConfig {
    pub mode: Option<RecoveryMode>,
    pub retention_period: Option<Duration>,
}
```

### 4.1 Declaration

```rust
.package(package!("io.zenoh.jni")
    .class(sealed_class!(RecoveryMode))
    .class(data_class!(RecoveryConfig)))
```

A per-variant `.variant(variant!(V).name("…"))` renames the emitted class **and every slot derived
from it**, so the surface stays self-consistent; the running example below keeps the source names so
each snippet reads as one contract. `examples/covertest-kotlin` exercises the rename
(`variant!(Labeled).name("Tagged")` ⇒ the class `Tagged` and the slots `tagged_v0` / `tagged_v1`).

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
(`prebindgen/src/api/lang/jnigen/jni/struct_plan.rs`). The Rust encoder emits **one `match`**
binding the tag and every slot, inert slots filled by the existing
`primitive_default_for_descriptor` (`prebindgen/src/api/lang/jnigen/jni/emit/struct_out.rs:51`):

```rust
let (mode__present, mode__tag, mode_periodic_queries_period) = match &v.mode {
    None => (false, 0i32, 0i64),
    Some(RecoveryMode::PeriodicQueries { period }) => (true, 0i32, __out_Duration(period.clone())),
    Some(RecoveryMode::Heartbeat)                 => (true, 1i32, 0i64),
};
```

The slots then ride the parent's single `call_static_method("fromParts", …)`. No JVM object is built
for the sum, and both sides enumerate the same slots in the same order because both walk one
`StructPlan` — the invariant that module already exists to hold.

**`Option<sum>` gates the same way in both output paths, by two different means** (#220). On the
`fromParts` bridge it is the separate `<field>__present` flag above; on a **value form**'s leaf list
it is the selector leaf's own **nullability** — the tag boxes, and JVM `null` means "no sum at all",
which tag `0` cannot say because that is a real alternative. The leaf list needs no present-flag
concept for it: a sum's leaves are not independent, so the whole segment binds as one tuple whose
absent arm carries every slot's wire default — the shape a sum under a *conditional* value form
already crossed by, applied to an optional step inside the segment's own path
(`prebindgen-jni/src/jni/emit/delivery.rs`, the sum-segment loop). `Vec<sum>` stays refused in a leaf
list: variable arity has no fixed layout to lay out.

### 4.4 Input path (Kotlin → Rust)

`FlatFieldNode::Sum` joins `Value` / `Nested`
(`prebindgen/src/api/lang/jnigen/jni/emit/flat_input.rs:349`), and `FlatInputPlan.root` generalizes
from `FlatStructNode` to a struct-or-sum root so a **sum-typed parameter** flattens too. Extern
signature:

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
        0 => RecoveryMode::PeriodicQueries { period: __in_Duration(mode_periodic_queries_period) },
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
and contributes its leaf … there is no selector" (`prebindgen/src/api/core/unfold.rs:8-11`). It
gains one:

- `LeafSource::VariantField { variant, member }` beside `Accessor` / `Field`
  (`prebindgen/src/api/core/unfold/plan.rs`), so a leaf can be reached through a variant pattern
  rather than a field chain, plus `LeafSource::SumTag` for the synthesized selector.
- Per-leaf group membership (`UnfoldLeaf::group`), so the Rust emitter emits one `match` over
  the value instead of independent per-leaf expressions. The tag leaf is the one leaf with **no
  converter** (`UnfoldLeaf::has_converter()`): it is assigned per arm, so requiring an output
  converter for it would make every sum depend on an unrelated `i32` crossing existing.
- `apply_sum_returns`, modeled on `apply_value_structs`: for every declared function returning a
  declared sum (`E` / `&E` / `Option<E>` / `Vec<E>`) and every `impl Fn(E)` / `impl Fn(&E)`
  callback parameter, build a `fixed_builder` `UnfoldPlan` over the tag + groups. It also drops the
  declared return's scan-time output requirement at **every layer** (`E`, `Option<E>`, `Vec<E>`) —
  a sum has no whole-value converter by construction, and the boundary-only pass only reaches the
  bare type.

`Option` / `Vec` layers need nothing new — they ride the existing `Shape` fold
(`prebindgen/src/api/core/shape.rs`), same as for a data class.

**The wire target is the hoisted builder singleton, not `Sum.fromParts`.** §4.2's `fromParts` is the
Kotlin-facing convenience: its parameters are the variants' **property** types (`Priority`, not the
`Int` discriminant) and its object slots are non-null, which an inert group's `JObject::null()`
would trip on inside the JVM's intrinsic null check before any generated code ran. So the return
path reassembles through the same inlined `when` over the tag that a sum-typed struct field already
gets, emitted into the hoisted `__<Name>Builder` / folder-appender singleton (and into the `asRaw`
proxy for a callback argument). Both come from one derivation, so the two positions cannot drift:

```kotlin
internal val __ReadingBuilder: ReadingBuilder<Reading> =
    ReadingBuilder { tag, exact_v0, range_low, range_high, tagged_v0, tagged_v1, companion_v0 ->
        when (tag) {
            0 -> Reading.Missing
            1 -> Reading.Exact(exact_v0)
            2 -> Reading.Range(range_low, range_high)
            3 -> Reading.Tagged(tagged_v0!!, Priority.fromInt(tagged_v1))
            4 -> Reading.Companion(companion_v0)
            else -> throw IllegalArgumentException("Reading: invalid tag $tag")
        }
    }
```

The `!!` is §4.4's inert-slot rule at the interface boundary: an object-shaped group slot is
declared nullable (`tagged_v0: String?`) because an inert group is wire-defaulted to JVM null, and
re-asserted inside its own live arm. Primitive slots keep their `0`/`false` default unboxed — a
handle payload rides its raw `jlong`, so an inert handle group is the `0L` sentinel that is simply
never wrapped.

**Who closes a handle payload: the receiver, in both positions.** A sum payload is a plan **leaf**,
so it takes `WrapKind::Handle` — the generated code wraps the pointer into its typed handle class
and stops there. It does *not* take the `WrapKind::HandleOwned` contract that a plan-less
`impl Fn(Handle)` argument gets, where the proxy also `close()`s the handle in a `finally`
(close-unless-taken). So:

- a **returned** sum hands over a handle the caller owns and must `close()`;
- a sum delivered to a **callback** does the same — the handle is live for the duration of `run` and
  stays live after it returns, and closing it is the receiver's job.

The two positions agree, which is the point: the reassembly comes from one derivation, so the
ownership contract cannot differ between them. Both are exercised on the JVM in
`examples/covertest-kotlin` ("sum return with a handle payload", "sum with a handle payload
delivered to a callback"), the latter asserting the handle is usable inside `run` and still
closeable afterwards.

**A borrowed sum return (`&E`, `Option<&E>`) crosses like an owned one.** `unfold::returns_type`
peels the leading `&` and `wire_fixed_returns` records `by_ref`, so the encoder matches *through*
the reference instead of moving the value out of its owner; each live group clones what it needs.
Kotlin therefore receives an ordinary value with no borrow to track and nothing to close — the
borrow never crosses the boundary — and the owner is unchanged and can be read again.

### 4.6 Rejections

Generation errors, each naming the offending path:

- A **recursive** sum (a variant payload reaching its own type) — the flatten plans are finite; there
  is no `jobject_input`-style escape hatch for sums in this design.
- An unresolvable payload leaf — the same error the data-class flattener already reports
  (`FlatInputError`, `flat_input.rs:432`), extended with the variant name in the path.

## 5. Cbindgen lowering

### 5.1 Declaration

`.tagged_union(ty)` beside `.enum_type(ty)` (`prebindgen/src/api/lang/cbindgen/builder.rs:398`),
wired into the same three places every declarator touches: the already-declared guard (`:306`), the
universal `.name()` override, and the `CurrentDecl` cursor for error messages (`:701`).
`.enum_type()` on a payload enum errors with a pointer to it.

### 5.2 The mirror type

`prereq_tagged_unions` mirrors `prereq_enums`
(`prebindgen/src/api/lang/cbindgen/trait_impl.rs`), emitting a `#[repr(C)]` enum whose payload
fields carry **wire** types chosen by `payload_field_wire`.

A payload's wire is its **resolved converter destination** — the same source a `data_struct` field
effectively uses — not the layout-preserving `mirror_field_wire` policy. That policy exists for
`repr_c_struct`, where one whole-struct `Transmute` reinterprets the bytes, so it can only accept
shapes that survive a reinterpret. **A tagged union is not reinterpreted; it is rebuilt arm by arm
through per-field converters**, so its payloads are not constrained that way. What that admits:

| Payload | Wire |
|---|---|
| scalar (except `bool`) | itself |
| `bool` | `MaybeUninit<bool>` — the one scalar with a restricted domain, so a C-supplied byte is normalised (nonzero is true), never materialised (§5.3) |
| `String` | `char *` (the one type whose two directions disagree — `*const` in, `*mut` out — so the field fixes the owning form and the arms convert by hand) |
| declared `enum_type` | `MaybeUninit<mirror>`, validated on the way in (§5.3) |
| `Box<T>` / `Option<Box<T>>`, `T` an `opaque_ptr` | `*mut t_t` |
| nested `data_struct` | its mirror, **by value** |
| bare `opaque_ptr` handle | `*mut t_t` |
| converted leaf (`Duration` → `u64`) | the converter's destination |
| `Vec<_>` | **rejected** — it needs two wires (pointer + length) and a union field carries one |

The two directions must agree on the wire, since one field serves both; a disagreement is a
generation error naming the payload, not a silent pick of one side. A payload with a **fallible
output** converter is also refused: the union's encoder has no error channel, because Rust always
writes a live arm.

Because a payload can now reach its own converter, it is registered as a resolver dependency and the
union's converter **defers** (returns `None`, which the resolver retries) until that converter has
resolved. `subs` alone cannot order this — it only drives the post-resolution propagation pass.

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
            RecoveryMode::PeriodicQueries { period: __in_Duration(period) },
        z_recovery_mode_t::Heartbeat => RecoveryMode::Heartbeat,
    }
}
```

The **boundary-validity rule** the tag check rests on: the mirror is received as
`MaybeUninit<mirror>` (`#[repr(transparent)]`, so identical ABI and legal to hold any bit pattern),
its leading `c_int` tag is range-checked, and only then is the value `assume_init`ed. That is sound
because **every payload wire is bit-pattern-agnostic**, which is what makes the tag the sole
obligation. Two Rust types are not: a declared `enum_type` (a discriminant no variant has) and
`bool` (anything but `0`/`1`), so both ride behind `MaybeUninit` too. The enum payload goes through
its own validating decode and propagates a rejection with `?`; a `bool` byte is read as a `u8` and
normalised the way C converts to `_Bool` — nonzero is true, no rejection, because unlike a tag every
byte here has an unambiguous meaning. **Invisible in C either way**: cbindgen simplifies
`MaybeUninit<T>` to `T`.

(A `bool` reached through a nested `data_struct` payload is *not* covered — that is the pre-existing
`c_field_wire` policy, which a plain `bool` parameter shares. Tracked as #170.)

### 5.4 Payload ownership

A tagged union crosses **by value**, like a `data_struct`. If any variant's payload wire owns memory
(`char *`, an opaque pointer), the declaration must also produce a typed
`z_<name>_drop(z_<name>_t *)` that frees the **active arm** — consistent with the existing typed
per-pointer drops. Phase B requires it: an owning payload without a drop is a generation error, not a
leak.

Ownership follows from the wire, so a **nested `data_struct` payload** is owning when its own mirror
has owning fields, even though the payload wire is a struct by value rather than a pointer. The drop
reaches through the payload and releases each of them, nulling the slot so a second drop is a no-op.
Without that a `String` or handle inside a struct payload would leak silently — which is exactly the
shape zenoh-flat#30 needs (`ReplyResult`'s alternatives are structs whose fields are handles).

That reach is **recursive one level further**: a struct payload's own field may itself be a declared
`tagged_union` with an owning arm, whose wire is again a by-value mirror rather than a pointer. The
outer drop delegates to *that* union's typed drop, which nulls what it frees, so idempotence
composes. Nothing else can reach it — at top level the data-struct contract is that C releases each
owning field itself, but a union arm is not a top-level struct field. One predicate
(`tagged_union_has_drop`) serves as both the drop's emission condition and the containing struct's
test for whether to call it, so a nested union cannot be freed through a symbol that was never
emitted.

The drop is also a second C entry point into the same bytes, so it validates the tag the same way
`in_tagged_union` does (§5.3) and treats an out-of-range one as nothing to release.

## 6. What core owns

| Piece | Home |
|---|---|
| `EnumShape::{Unit, Sum}` classifier — the single definition of "is this enum C-like" | `prebindgen/src/api/core/types_util.rs` |
| `SumSpec { key, source, variants: Vec<SumVariant> }`, `SumVariant { ident, tag, fields }`, `SumField { member, name, ty }` — the neutral description both adapters read | `prebindgen/src/api/core/types_util.rs` |
| `enum_discriminant_values`, moved out of `jnigen/util.rs` | `prebindgen/src/api/core/types_util.rs` |
| The unfold selector (`LeafSource::VariantField`, tag leaf, group membership, `apply_sum_returns`) | `prebindgen/src/api/core/unfold.rs` |

Everything else is adapter-local. Payload **wire** choice stays per-adapter, exactly as `ValueDecon`
leaves are adapter-built today (`Prebindgen::value_struct_decons`,
`prebindgen/src/api/core/prebindgen.rs:218`) — core describes the sum, adapters decide what its
leaves look like on the wire.

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
| **C** | [#148](https://github.com/milyin/prebindgen/issues/148) | jnigen: `sealed_class!` + `variant!`, `TypeKind::Sum`, `TypeConfig.sum_cfg`, the sealed-interface emitter (sibling of `write_enum_classes`, `prebindgen/src/api/lang/jnigen/jni/kotlin_emit.rs:555`). | A |
| **D** | [#149](https://github.com/milyin/prebindgen/issues/149) | jnigen: tag-gated groups on both flatten paths — `FlatFieldNode::Sum`, the struct-or-sum root, the `kt_access` expression-template refactor, `PlanFieldKind::Sum`. Unblocks flat #31 + #11, and flat #30 in its struct-field form (`ReplyStruct { result: ReplyResult, … }`). | C |
| **E** | [#150](https://github.com/milyin/prebindgen/issues/150) | core + jnigen: sum-typed returns and callback arguments — the unfold selector. Needed when a function's **own** return (or a callback argument) is the sum, e.g. a `reply_get_result(&Reply) -> ReplyResult` accessor mirroring base's `Reply::result()`. | D |

A sum nested as a **data-class field** keeps the whole-value `fromParts` path (stage D's
`PlanFieldKind::Sum`); the fixed-builder leaf synthesis for value structs still declines a sum
field. Flattening that too is a wire-width/allocation optimization of an already-correct path, not
a capability, so it is deliberately not part of stage E.
