# JniGen binding report

Base package: `io.prebindgen.covertest`

## package `io.prebindgen.covertest`

- `string_new` — `fun stringNew(s: String, onError: JniErrorHandler<String>): String`
- `val COVER_BANNER: String` — binding expression
- `val COVER_MAGIC` — `#[prebindgen]` const `COVER_MAGIC`
- `val COVER_TAG_RUNTIME` — nullary `#[prebindgen]` fn `cover_tag_runtime`
- `val COVER_TAG` — `#[prebindgen]` const `COVER_TAG`
- `val COVER_VERSION: String` — binding expression

## package `io.prebindgen.covertest.analytics`

- `archive_latest` — `fun archiveLatest(a: SummaryVault, onError: JniErrorHandler<Summary?>): Summary?`
- `archive_new` — `fun archiveNew(onError: JniErrorHandler<SummaryVault>): SummaryVault`
- `archive_store` — `fun archiveStore(a: SummaryVault, sSel: Int, s00: Long?, s01: Double?, s1: Summary?, onError: JniErrorHandler<Unit>)`
  - shaped by: param `s` expanded from `Summary` — variants [summary_new, self]
- `storage_expect_summary` — `fun storageExpectSummary(s: Storage, expectedSel: Int, expected00: Long?, expected01: Double?, expected1: Summary?, onError: JniErrorHandler<Boolean>): Boolean`
  - shaped by: param `expected` expanded from `Summary` — variants [summary_new, self]
- `storage_matches_summary` — `fun storageMatchesSummary(s: Storage, expectedSel: Int, expected00: Long?, expected01: Double?, expected1: Summary?, onError: JniErrorHandler<Boolean>): Boolean`
  - shaped by: param `expected` expanded from `Summary` — variants [summary_new, self]
- `storage_summary` — `fun <R> storageSummary(s: Storage, onError: JniErrorHandler<R>, build: SummaryBuilder<R>): R`
  - shaped by: return `Summary` decomposed → [count, total] (Callback delivery)
- `storage_summary_full` — `fun <R> storageSummaryFull(s: Storage, onError: JniErrorHandler<R>, build: SummaryStorageSummaryFullBuilder<R>): R`
  - shaped by: return `Summary` decomposed → [count, total, handle] (Callback delivery)
- `storage_summary_handle` — `fun storageSummaryHandle(s: Storage, onError: JniErrorHandler<Summary>): Summary`
- `storage_summary_probe` — `fun <R> storageSummaryProbe(s: Storage, onError: JniErrorHandler<R>, build: SummaryStorageSummaryProbeBuilder<R>): R`
  - shaped by: return `Summary` decomposed → [count, total, handle] (Callback delivery)
- `summary_describe` — `fun describeSummary(sSel: Int, s00: Long?, s01: Double?, s1: Summary?, verbose: Boolean, onError: JniErrorHandler<String>): String`
  - shaped by: param `s` expanded from `Summary` — variants [summary_new, self]
- `summary_merge` — `fun <R> summaryMerge(primarySel: Int, primary00: Long?, primary01: Double?, primary1: Summary?, fallbackSel: Int, fallback00: Long?, fallback01: Double?, fallback1: Summary?, onError: JniErrorHandler<R>, build: SummaryBuilder<R>): R`
  - shaped by: param `fallback` expanded from `Summary` — variants [summary_new, self]
  - shaped by: param `primary` expanded from `Summary` — variants [summary_new, self]
  - shaped by: return `Summary` decomposed → [count, total] (Callback delivery)
- `summary_prefer` — `fun summaryPrefer(primarySel: Int, primary00: Long?, primary01: Double?, primary1: Summary?, fallbackSel: Int, fallback00: Long?, fallback01: Double?, fallback1: Summary?, onError: JniErrorHandler<Long>): Long`
  - shaped by: param `fallback` expanded from `Summary` — variants [summary_new, self]
  - shaped by: param `primary` expanded from `Summary` — variants [summary_new, self]
- `summary_series` — `fun <A> summarySeries(count: Long, start: Long, acc: A, onError: JniErrorHandler<A>, fold: SummaryFolder<A>): A`
  - shaped by: return `Summary` decomposed → [count, total] (Callback delivery)
- `summary_series_opt` — `fun <A> summarySeriesOpt(count: Long, start: Long, acc: A, onError: JniErrorHandler<A?>, fold: SummaryFolder<A>): A?`
  - shaped by: return `Summary` decomposed → [count, total] (Callback delivery)
- `summary_total_opt` — `fun summaryTotalOpt(sSel: Int, s00: Long?, s01: Double?, s1: Summary?, onError: JniErrorHandler<Double>): Double`
  - shaped by: param `s` expanded from `Summary` — variants [summary_new, self]
- `summary_total_raw` — `fun summaryTotalRaw(s: Summary, onError: JniErrorHandler<Double>): Double`

## package `io.prebindgen.covertest.model`

- `annotated_new` — `fun annotatedNew(payload: Payload, ttl: Long?, priority: Priority?, onError: JniErrorHandler<Annotated>): Annotated`
- `annotated_payload_value` — `fun annotatedPayloadValue(a: Annotated, onError: JniErrorHandler<Double>): Double`
- `annotated_priority` — `fun annotatedPriority(a: Annotated, onError: JniErrorHandler<Priority?>): Priority?`
- `annotated_ttl` — `fun annotatedTtl(a: Annotated, onError: JniErrorHandler<Long?>): Long?`
- `celsius_double` — `fun celsiusDouble(c: Int, onError: JniErrorHandler<Int>): Int`
- `label_reverse` — `fun labelReverse(l: String, onError: JniErrorHandler<String>): String`
- `payload_priority` — `fun payloadPriority(p: Payload, onError: JniErrorHandler<Priority>): Priority`
- `percent_scale` — `fun percentScale(p: Int, factor: Int, onError: JniErrorHandler<Int>): Int`
- `priority_or` — `fun priorityOr(p: Priority?, fallback: Priority, onError: JniErrorHandler<Priority>): Priority`
- `priority_weight` — `fun priorityWeight(p: Priority, onError: JniErrorHandler<Int>): Int`
- `stamp_new` — `fun stampNew(secs: Long, nanos: Long, onError: JniErrorHandler<Stamp>): Stamp`
- `stamp_series` — `fun stampSeries(count: Long, onError: JniErrorHandler<List<Stamp>>): List<Stamp>`
  - shaped by: return `Stamp` decomposed → [] (Callback delivery)

## package `io.prebindgen.covertest.storage`

- `millis_add` — `fun addMillis(a: Long, b: Long, onError: JniErrorHandler<Long>): Long`
- `payload_handler_new` — `fun payloadHandlerNew(f: PayloadCallback, onError: JniErrorHandler<PayloadHandler>): PayloadHandler`
- `payload_vec_handler_new` — `fun payloadVecHandlerNew(f: PayloadListCallback, onError: JniErrorHandler<PayloadVecHandler>): PayloadVecHandler`
- `storage_callback` — `fun storageCallback(s: Storage, handler: PayloadHandler, onError: JniErrorHandler<Unit>)`
- `storage_callback_vec` — `fun storageCallbackVec(s: Storage, handler: PayloadVecHandler, onError: JniErrorHandler<Unit>)`
- `storage_emit` — `fun storageEmit(n: Long, h: StorageHandler, onError: JniErrorHandler<Unit>)`
- `storage_get` — `fun storageGet(s: Storage, onError: JniErrorHandler<Payload?>): Payload?`
  - shaped by: return `Payload` decomposed → [id, seq, value, flag, label] (Callback delivery)
- `storage_get_vec` — `fun storageGetVec(s: Storage, onError: JniErrorHandler<List<Payload>?>): List<Payload>?`
  - shaped by: return `Payload` decomposed → [id, seq, value, flag, label] (Callback delivery)
- `storage_handler_new` — `fun storageHandlerNew(f: StorageCallback, onError: JniErrorHandler<StorageHandler>): StorageHandler`
- `storage_labels` — `fun storageLabels(s: Storage, onError: JniErrorHandler<List<String>>): List<String>`
  - shaped by: return `String` decomposed → [] (Callback delivery)
- `storage_new` — `fun storageNew(onError: JniErrorHandler<Storage>): Storage`
- `storage_put_by_read` — `fun storagePutByRead(s: Storage, payload: Payload, onError: JniErrorHandler<Unit>)`
- `storage_put_by_take` — `fun storagePutByTake(s: Storage, payload: Payload, onError: JniErrorHandler<Unit>)`
- `storage_put_opt` — `fun storagePutOpt(s: Storage, p: Payload?, onError: JniErrorHandler<Boolean>): Boolean`
- `storage_put_slice` — `fun storagePutSlice(s: Storage, payloads: List<Payload>, onError: JniErrorHandler<Unit>)`
- `storage_shards` — `fun storageShards(count: Long, each: Long, onError: JniErrorHandler<List<Storage>>): List<Storage>`
  - shaped by: return `Storage` decomposed → [] (Callback delivery)
- `storage_shards_opt` — `fun storageShardsOpt(count: Long, each: Long, onError: JniErrorHandler<List<Storage>?>): List<Storage>?`
  - shaped by: return `Storage` decomposed → [] (Callback delivery)
- `storage_total_len` — `fun storageTotalLen(a: Storage, b: Storage, c: Storage, onError: JniErrorHandler<Long>): Long`
- `storage_try_from_stamp` — `fun storageTryFromStamp(s: Stamp, onBindingError: JniErrorHandler<Storage>, onError: StorageErrorHandler<Storage>): Storage`
  - shaped by: domain error `StorageError` decomposed → onError [message, handle] (binding failures → onBindingError)
- `storage_try_with_label` — `fun storageTryWithLabel(label: String, onBindingError: JniErrorHandler<Storage>, onError: StorageErrorHandler<Storage>): Storage`
  - shaped by: domain error `StorageError` decomposed → onError [message, handle] (binding failures → onBindingError)

## class `io.prebindgen.covertest.esc_pkg.Esc_Probe` (ptr_class, Rust `EscapeProbe`)

- `escape_probe_new` — `fun escapeProbeNew(value: Long, onError: JniErrorHandler<Esc_Probe>): Esc_Probe`
- `escape_probe_value` — `fun escapeProbeValue(onError: JniErrorHandler<Long>): Long`

## class `io.prebindgen.covertest.Payload` (data_class, Rust `Payload`)

- `payload_label_len` — `fun labelLen(onError: JniErrorHandler<Long?>): Long?`

## class `io.prebindgen.covertest.model.Stamp` (value_class, Rust `Stamp`)

- `stamp_nanos` — `fun nanos(onError: JniErrorHandler<Long>): Long`
- `stamp_secs` — `fun secs(onError: JniErrorHandler<Long>): Long`

## class `io.prebindgen.covertest.Storage` (ptr_class, Rust `Storage`)

- `storage_contains` — `fun contains(id: Long, onError: JniErrorHandler<Boolean>): Boolean`
- `storage_len` — `fun len(onError: JniErrorHandler<Long>): Long`
- `storage_with_payload` — `fun withPayload(payload: Payload, onError: JniErrorHandler<Storage>): Storage`

## class `io.prebindgen.covertest.errors.StorageError` (ptr_class, Rust `StorageError`)

- `storage_error_message` — `fun message(onError: JniErrorHandler<String>): String`

## class `io.prebindgen.covertest.analytics.Summary` (ptr_class, Rust `Summary`)

- `summary_count` — `fun count(onError: JniErrorHandler<Long>): Long`
- `summary_from_mean` — `fun fromMean(count: Long, mean: Double, onError: JniErrorHandler<Summary>): Summary`
- `summary_mean` — `fun mean(onError: JniErrorHandler<Double>): Double`
- `summary_new` — `fun of(count: Long, total: Double, onError: JniErrorHandler<Summary>): Summary`
- `summary_scaled` — `fun scaled(factor: Double, onError: JniErrorHandler<Double>): Double`
- `summary_total` — `fun total(onError: JniErrorHandler<Double>): Double`

## types

- `Annotated`: data_class → `io.prebindgen.covertest.model.Annotated` (wire `jni :: objects :: JObject`)
- `Archive`: ptr_class → `io.prebindgen.covertest.analytics.SummaryVault` (wire `jni :: sys :: jlong`)
- `EscapeProbe`: ptr_class → `io.prebindgen.covertest.esc_pkg.Esc_Probe` (wire `jni :: sys :: jlong`)
- `Payload`: data_class → `io.prebindgen.covertest.Payload` (wire `jni :: objects :: JObject`)
- `PayloadHandler`: ptr_class → `io.prebindgen.covertest.PayloadHandler` (wire `jni :: sys :: jlong`)
- `PayloadVecHandler`: ptr_class → `io.prebindgen.covertest.PayloadVecHandler` (wire `jni :: sys :: jlong`)
- `Priority`: enum_class → `io.prebindgen.covertest.model.Priority` (wire `jni :: sys :: jint`)
- `Stamp`: value_class → `io.prebindgen.covertest.model.Stamp` (wire `jni :: objects :: JByteArray`)
- `Storage`: ptr_class → `io.prebindgen.covertest.Storage` (wire `jni :: sys :: jlong`)
- `StorageError`: ptr_class → `io.prebindgen.covertest.errors.StorageError` (wire `jni :: sys :: jlong`)
- `StorageHandler`: ptr_class → `io.prebindgen.covertest.StorageHandler` (wire `jni :: sys :: jlong`)
- `Summary`: ptr_class → `io.prebindgen.covertest.analytics.Summary` (wire `jni :: sys :: jlong`)

## conversions

- `convert!(Celsius)`: input `Into` ⇄ `i32`, output `Into` ⇄ `i32`
- `convert!(Label)`: input `#[prebindgen]` fn `label_in`, output `#[prebindgen]` fn `label_out`
- `convert!(Millis)`: input `#[prebindgen]` fn `millis_from_long`, output `#[prebindgen]` fn `millis_value`
- `convert!(Percent)`: input `TryInto` ⇄ `i32`, output `Into` ⇄ `i32`
