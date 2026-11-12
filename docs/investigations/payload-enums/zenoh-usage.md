# Check the payload-enum plan against Zenoh

The shared variant model fits Zenoh's source types. The original plan needed
two changes to fit its binding configuration: explicit enum-field expansions
must inherit existing field defaults, and those expansions must work without
an ordinary sealed/tagged declaration. Otherwise the generator could require
classes and handle conversions that the binding deliberately avoids.

This is a source/configuration audit, with runnable generation controls. It is
not a full build of Zenoh against this branch. The [implementation notes](implementation.md)
link the equivalent generated-boundary tests and distinguish the remaining port prerequisites.

## Sources checked

The repositories' `main` references were refreshed without modifying their
working trees. The checked revisions are:

| Repository | Revision | Role |
| --- | --- | --- |
| eclipse-zenoh/zenoh-flat | `99a7e7084e00824f85c94cbabfa9573108d53dea` | Source types/functions, including unstable APIs |
| eclipse-zenoh/zenoh-flat-jni | `d325933cbbf8a7bb421e139f68a59eb118bc4b75` | Actual JNI configuration and binding-local helpers |
| eclipse-zenoh/zenoh-java | `d6882f5e6e3730613521e125c3a66964b017305b` | Java SDK's Kotlin adapters |
| eclipse-zenoh/zenoh-kotlin | `3d75693f80571b9b76fae8b76ac0d680bd00db03` | Kotlin SDK's adapters/configuration |
| ZettaScaleLabs/zenoh-flat-c | `3f75262ebae66a23101427e9afb8a2e4bd6e1160` | Existing C ownership conventions and examples |

The C consumer's main revision is from June 2026 and uses an older prebindgen
API. It is evidence of how its callers use handles/callbacks, not proof of a
current complete C port or a payload-enum consumer. The JNI consumer is at
1.10.2 and explicitly enables zenoh-flat's `unstable` feature for both runtime
and build dependencies.

## 1. Recovery mode needs an enum inside optional configuration

The real source shape is:

```rust
pub enum RecoveryMode {
    PeriodicQueries(Duration),
    Heartbeat,
}
pub struct RecoveryConfig {
    pub mode: Option<RecoveryMode>,
    pub retention_period: Option<Duration>,
}
```

Zenoh JNI declares `sealed_class!(RecoveryMode)` and
`data_class!(RecoveryConfig)`. The Kotlin SDK actually constructs
`RecoveryMode.PeriodicQueries(milliseconds.toULong())` or `Heartbeat`, then puts
that value in `RecoveryConfig`. This is a concrete **input** requirement, not
just an output-rendering example. The shared compiler must compose record,
optional, selected variant and canonical Duration conversion recursively.

`retention_period` is a sibling of `mode`, not another enum arm or a payload
present only with PeriodicQueries. Test all three modes: no mode, Heartbeat and
PeriodicQueries, with independent retention-period values. A Heartbeat must
neither convert nor read a nonexistent period payload.

The binding-local Duration helper is a separate port prerequisite. Current JNI
uses `fun!(crate::duration_from_millis).sig(...)` and
`fun!(crate::duration_to_millis).sig(...)`, plus a valid range reserving
`u64::MAX`. #808's binding-local callable signatures and declared scalar domains
are still backlog items. Payload enums must accept the resulting canonical
converter when those are implemented; do not special-case Duration or silently
reserve a legitimate scalar value. Without a declared niche, use the existing
explicit optional-presence representation.

Sources: [Rust configuration](https://github.com/eclipse-zenoh/zenoh-flat/blob/99a7e7084e00824f85c94cbabfa9573108d53dea/src/base/advanced_subscriber/mod.rs),
[JNI declarations](https://github.com/eclipse-zenoh/zenoh-flat-jni/blob/d325933cbbf8a7bb421e139f68a59eb118bc4b75/build.rs),
[Duration helpers](https://github.com/eclipse-zenoh/zenoh-flat-jni/blob/d325933cbbf8a7bb421e139f68a59eb118bc4b75/src/duration.rs),
[Kotlin SDK construction](https://github.com/eclipse-zenoh/zenoh-kotlin/blob/3d75693f80571b9b76fae8b76ac0d680bd00db03/zenoh-kotlin/src/commonMain/kotlin/io/zenoh/Session.kt).

## 2. Reply delivery must keep the expanded callback path

Zenoh's source callback is `impl Fn(Reply) + Send + Sync + 'static`. A reply
has a consuming value projection:

```rust
pub enum ReplyResult {
    Sample(Sample),
    Error(ReplyError),
}
pub struct ReplyStruct {
    pub result: ReplyResult,
    pub replier_id: Option<EntityGlobalId>, // unstable
}
// Source helper: consumes Reply and moves its selected payload.
// pub fn reply_into_struct(r: Reply) -> ReplyStruct
```

Current JNI does not expose ReplyResult through a sealed declaration. It
expands Reply using `reply_get_replier_id`, `reply_is_ok`, `reply_get_sample`
and `reply_get_err`. The success branch further expands Sample; the error
branch expands payload/encoding. Both SDKs receive those fields in one JNI
callback and build their own SDK reply objects. Requiring a generated sealed
ReplyResult as an intermediate would change this path and can add an unwanted
allocation before the SDK constructs its own value.

The proposed enum operation should allow a cleaner configuration of the same
kind, using the consuming source projection. This configuration uses
the existing expansion builders around the enum-field operation:

```rust,ignore
.expand(expand_return!(ReplyResult).view(Output::enum_fields([])))
.expand(expand_return!(Reply).fields_into(fields!(reply_into_struct)))
```

`view` and `fields_into` already exist in #808's
[expansion builders](https://github.com/milyin/prebindgen/blob/4ea0ad688ec902d9d0f03a8a3a2a664249d4a56b/prebindgen-declarations/src/builders.rs).
The pinned consumer uses the older `fields_self_into` spelling, which is a
deprecated alias in #808. Only `enum_fields` is new in this configuration.

Unspecified enum-field recipes inherit the configured Sample and ReplyError
output expansions. They do not revert to ordinary Sample/ReplyError handles.
The callback receives the tag and expanded selected payload, plus the sibling
replier id. No ReplyResult/ReplyStruct class declaration or object is needed.
The integer tag replaces the current boolean discriminator; this requires
adapting the SDK callback signature, not adding accessor JNI calls. Local field
overrides remain available, including explicit canonical handle output.
Prefix expanded leaf names by their structural path, so Sample.payload and
ReplyError.payload remain distinct (`resultSamplePayload` versus
`resultErrorPayload`). Emit named tag constants such as
`REPLY_RESULT_SAMPLE_TAG` even without a sealed declaration. SDK adapters use
those constants and must be rebuilt with the binding when its ABI changes.
JNI's initialization check must include field-only mappings as well as sealed
ones; a stale generated JNI artifact must not silently decode another mapping.

The branch choice comes from matching the one ReplyResult value, not from
independently calling two optional getters and a boolean getter. The consuming
`reply_into_struct` moves the selected native Sample/ReplyError. Conversely,
`reply_to_struct(&Reply)` and `reply_get_result(&Reply)` deliberately clone in
the source crate. Selecting those helpers is an explicit cost; the generator
must not clone their already-owned result again.

There is another cost to distinguish: the source currently provides only
`reply_error_to_struct(&ReplyError)`, whose body clones its fields. There is no
captured `reply_error_into_struct` helper. Moving ReplyResult::Error itself is
possible; claiming a completely move-only error-field decomposition would
require an appropriate consuming source helper or an explicitly chosen clone.
Do not invent an accessor or hide that cost inside variant conversion.

Sample's configured consuming decomposition also discards `timestamp_stack`.
The new enum recursion must preserve that override. Its default omission saves
the callback from materializing instrumentation that callers rarely request.

Finally, these are two different error channels in the real API:

* `session_get(...) -> Result<(), Error>` reports failure to start the query.
* `ReplyResult::Error(ReplyError)` is a received application reply, with binary
  payload and encoding, delivered to the reply callback.

The second must not become a binding failure, trigger the function's error
handler or be mandatorily converted to text. Its source variant happens to be
named `Error`; that spelling does not determine error routing.

Sources: [Rust reply/value projections](https://github.com/eclipse-zenoh/zenoh-flat/blob/99a7e7084e00824f85c94cbabfa9573108d53dea/src/base/query/reply.rs),
[query signature](https://github.com/eclipse-zenoh/zenoh-flat/blob/99a7e7084e00824f85c94cbabfa9573108d53dea/src/base/session/mod.rs),
[Sample consuming projection](https://github.com/eclipse-zenoh/zenoh-flat/blob/99a7e7084e00824f85c94cbabfa9573108d53dea/src/base/sample/mod.rs),
[JNI expansions](https://github.com/eclipse-zenoh/zenoh-flat-jni/blob/d325933cbbf8a7bb421e139f68a59eb118bc4b75/build.rs),
[Java SDK callback adapter](https://github.com/eclipse-zenoh/zenoh-java/blob/d6882f5e6e3730613521e125c3a66964b017305b/zenoh-java/src/commonMain/kotlin/io/zenoh/FlatCallbacks.kt),
[Kotlin SDK callback adapter](https://github.com/eclipse-zenoh/zenoh-kotlin/blob/3d75693f80571b9b76fae8b76ac0d680bd00db03/zenoh-kotlin/src/commonMain/kotlin/io/zenoh/FlatCallbacks.kt).

## 3. Instrumentation needs arrays of records containing enums

The real shape is more demanding than `Vec<EnumWithScalar>`:

```rust
pub enum InstrumentationTimestamp {
    Uhlc(Timestamp),
    Custom(Vec<u8>),
}
pub struct TimestampStackRecord {
    pub point: InterceptionPoint,
    pub timestamp: InstrumentationTimestamp,
}
pub struct Timestamp {
    pub ntp64: u64,
    pub id: Vec<u8>,
}
// pub fn timestamp_stack_get_records(s: &TimestampStack)
//     -> Vec<TimestampStackRecord>
```

JNI declares the timestamp enum and both records. A bulk row therefore contains
a fieldless enum plus a selected payload that is either another record with a
byte buffer, or a byte buffer directly. Test ordinary delivery and bulk output,
both arms mixed in one list, input reconstruction where configured, empty lists,
zero-length byte buffers and failure in a later row's selected descriptor.
Tag-first decoding must recurse into those selected child records; merely
supporting a union of scalar leaves does not cover this real shape.

The public Kotlin `u64` property is `ULong`, with a raw JNI Long bit pattern.
The probe confirms that #808 already generates this for the real Timestamp
record. Preserve the high bit of `ntp64` through ordinary and bulk paths. The
proposal's introductory Kotlin example has been corrected to use ULong too.

`timestamp_stack_get_records` is explicitly on demand: the source copies
Custom byte vectors when producing the requested records from a borrowed
stack. Enum conversion must move those produced vectors, not add another copy.
Neither ordinary sample nor reply delivery should start requesting the records
just because the enum and bulk conversions now exist.

Sources: [Rust timestamp/record/list](https://github.com/eclipse-zenoh/zenoh-flat/blob/99a7e7084e00824f85c94cbabfa9573108d53dea/src/base/time/timestamp_stack.rs),
[Timestamp fields and high-bit source test](https://github.com/eclipse-zenoh/zenoh-flat/blob/99a7e7084e00824f85c94cbabfa9573108d53dea/src/base/time/timestamp.rs),
[JNI declarations and on-demand policy](https://github.com/eclipse-zenoh/zenoh-flat-jni/blob/d325933cbbf8a7bb421e139f68a59eb118bc4b75/build.rs).

## 4. C's current callback ownership is a separate compatibility gap

The C manifest declares Sample, Reply, Query and Hello as inline owned values
and marks their callback arguments `.takeable_param(0)`. Callers receive a
pointer valid during the callback. The trampoline drops the object afterward,
unless the callback moved it out with the generated take function. The
throughput subscriber deliberately does not drop each Sample itself; the
queryable example can take a Query to answer later.

#808's C callbacks currently transfer owning arguments to C, and the payload
plan describes by-value transfer for owning enum arguments. That is not the
same caller contract. Keep it explicit as an adapter-wide port prerequisite:
adding enum carriers does not make the existing takeable callers compatible.
The enum's active owning leaves must remain available to an outer delivery
policy; branch conversion must not bake in a different lifetime policy.

C can also keep Sample/Reply opaque and use direct borrowed getters. The
payload feature must stay opt-in and must not force those functions to return
tagged objects or materialize Rust value forms. General visible C records and
plain inline non-owning storage remain their existing backlog tasks.

Sources: [C declaration and callback ownership](https://github.com/ZettaScaleLabs/zenoh-flat-c/blob/3f75262ebae66a23101427e9afb8a2e4bd6e1160/build.rs),
[throughput subscriber](https://github.com/ZettaScaleLabs/zenoh-flat-c/blob/3f75262ebae66a23101427e9afb8a2e4bd6e1160/examples/z_sub_thr.c),
[takeable subscriber](https://github.com/ZettaScaleLabs/zenoh-flat-c/blob/3f75262ebae66a23101427e9afb8a2e4bd6e1160/examples/z_sub.c),
[queryable](https://github.com/ZettaScaleLabs/zenoh-flat-c/blob/3f75262ebae66a23101427e9afb8a2e4bd6e1160/examples/z_queryable.c).

## What the implementation must demonstrate

| Case | Required result | Independent prerequisite or limit |
| --- | --- | --- |
| RecoveryConfig -> optional RecoveryMode -> Duration | Both input modes and absence compose, retention stays independent | Binding-local helper signatures and declared Duration domain/niche |
| Reply -> ReplyStruct -> expanded ReplyResult callback | One native callback, inherited payload expansions, local omissions, no intermediate sealed object | SDK callback signature migration; owned error-field helper if completely move-only decomposition is wanted |
| Function error versus reply error | Distinct existing channels; reply payload/encoding preserved | No mandatory stringification of data errors |
| TimestampStackRecord list | Nested record/enum/bytes work in bulk; selected-arm cleanup | No eager access to the stack during normal message delivery |
| u64 Timestamp field | Public ULong and bit-preserving JNI/bulk conversion | No signed range rejection of a valid high-bit timestamp |
| Optional EntityGlobalId beside ReplyResult | Remains a sibling, including its fixed-width node id | Current #808 data-class properties reject ZenohId's `[u8; ZENOH_ID_MAX_SIZE]`; record the fixed-array prerequisite |
| Existing C callback examples | Scoped/takeable objects still require their original cleanup contract | Separate callback delivery-policy migration, not solved by payload enums |

The Rust source contains 44 textual `impl Fn` occurrences across both tiers,
all with unit return. Three take Reply. Callback return-value support and
`Fn(Result<T, E>)` remain valid general features, but neither is the actual
source signature of Zenoh's current reply path. Do not rewrite that source API
or claim those hypothetical signatures prove the real consumer port.

Feature-gated source capture also remains authoritative. RecoveryMode and
instrumentation are unstable-only; ReplyStruct's replier id is conditional.
Test generated bindings with stable and unstable capture streams. Do not
replace a disabled field with a zero/default slot or change a variant's meaning
to compensate for different enabled features.

For performance, prioritize a consuming success-reply callback against the
current expanded callback and the mixed instrumentation list against ordinary
object delivery. Keep source-helper clones separate from generator clones and
report allocation counts/native crossings alongside latency. Use local JNI
artifacts explicitly in Gradle consumer runs; published Maven artifacts do not
test this generator branch. Measurements of the implemented equivalent shapes are recorded in the implementation notes;
they are not full SDK throughput measurements.

## Reproduce the checks

The [probe](probe/src/zenoh.rs) includes
[16 selected real declaration shapes](probe/src/zenoh_shapes.rs), with source
attributes/docs omitted and unstable fields present. The optional snapshot
check compares normalized Rust syntax with the **pinned Git objects**, not the
working tree; it neither fetches nor edits the Zenoh checkout.

```sh
cargo run --manifest-path docs/investigations/payload-enums/probe/Cargo.toml -- \
  --zenoh-flat ../zenoh-flat
```

The checks confirm that:

* All 16 fixture declarations match the pinned source after attribute removal.
* The three nested source cases now generate through the payload-enum compiler.
* The real Timestamp shape already generates ULong/ByteArray Kotlin properties.
* ZenohId's fixed-array property remains unsupported by the current data-class
  adapter, independently of payload enums.
* A small record control can expand an opaque child through its configured
  output recipe with no foreign class, for both a function and a callback.
  It uses the existing type-wide `view` builder. Declaring a data class for that same
  record fails because its child lacks a canonical foreign representation.
  Enum-field expansion must preserve that existing distinction.

These controls generate/parse Rust and inspect Kotlin source. They do not
compile or run the full Zenoh/native/JVM boundary. The source audit and controls establish concrete requirements. Equivalent
RecoveryConfig, instrumentation-list and expanded-reply cases run through JNI in
this branch; a complete consumer port still requires the independent features
listed above.
