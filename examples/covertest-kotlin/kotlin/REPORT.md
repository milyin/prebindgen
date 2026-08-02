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

- `annotated_alternate_value` — `fun annotatedAlternateValue(a: Annotated, onError: JniErrorHandler<Double?>): Double?`
- `annotated_new` — `fun annotatedNew(payload: Payload, ttl: Long?, priority: Priority?, onError: JniErrorHandler<Annotated>): Annotated`
- `annotated_payload_value` — `fun annotatedPayloadValue(a: Annotated, onError: JniErrorHandler<Double>): Double`
- `annotated_priority` — `fun annotatedPriority(a: Annotated, onError: JniErrorHandler<Priority?>): Priority?`
- `annotated_ttl` — `fun annotatedTtl(a: Annotated, onError: JniErrorHandler<Long?>): Long?`
- `archive_reading` — `fun archiveReading(a: SummaryVault, onError: JniErrorHandler<Reading>): Reading`
  - shaped by: return `Reading` decomposed → [tag, exact_v0, range_low, range_high, tagged_v0, tagged_v1, companion_v0] (Callback delivery)
- `archive_reading_maybe` — `fun archiveReadingMaybe(a: SummaryVault, onError: JniErrorHandler<Reading?>): Reading?`
  - shaped by: return `Reading` decomposed → [tag, exact_v0, range_low, range_high, tagged_v0, tagged_v1, companion_v0] (Callback delivery)
- `archive_set_reading` — `fun archiveSetReading(a: SummaryVault, which: Int, onError: JniErrorHandler<Unit>)`
- `arrays_echo` — `fun arraysEcho(a: Arrays, onError: JniErrorHandler<Arrays>): Arrays`
  - shaped by: return `Arrays` decomposed → [bytes, shorts, ints, longs, doubles, flags, raw] (Callback delivery)
- `blob_value_echo` — `fun blobValueEcho(value: BlobValue, onError: JniErrorHandler<BlobValue>): BlobValue`
  - shaped by: return `BlobValue` decomposed → [stamp__secs, stamp__nanos, id, chunks] (Callback delivery)
- `blob_value_new` — `fun blobValueNew(secs: Long, id: ByteArray, chunks: List<ByteArray>, onError: JniErrorHandler<BlobValue>): BlobValue`
  - shaped by: return `BlobValue` decomposed → [stamp__secs, stamp__nanos, id, chunks] (Callback delivery)
- `boxed_elem_id_sum` — `fun boxedElemIdSum(ps: List<Payload>, onError: JniErrorHandler<Long>): Long`
- `boxed_latest` — `fun <R> boxedLatest(a: SummaryVault, onError: JniErrorHandler<R?>, build: SummaryBuilder<R>): R?`
  - shaped by: return `Summary` decomposed → [count, total] (Callback delivery)
- `boxed_note_echo` — `fun boxedNoteEcho(note: String?, onError: JniErrorHandler<String?>): String?`
- `boxed_opt_payload_id` — `fun boxedOptPayloadId(p: Payload?, onError: JniErrorHandler<Long>): Long`
- `boxed_opt_priority_weight` — `fun boxedOptPriorityWeight(p: Priority?, onError: JniErrorHandler<Long>): Long`
- `boxed_payload_id` — `fun boxedPayloadId(p: Payload, onError: JniErrorHandler<Long>): Long`
- `boxed_run_id_sum` — `fun boxedRunIdSum(ps: List<Payload>, onError: JniErrorHandler<Long>): Long`
- `cache_config_weight` — `fun cacheConfigWeight(cache: CacheConfig?, onError: JniErrorHandler<Int>): Int`
- `celsius_double` — `fun celsiusDouble(c: Int, onError: JniErrorHandler<Int>): Int`
- `duration_boundary_echo` — `fun durationBoundaryEcho(value: DurationBoundary, onError: JniErrorHandler<DurationBoundary>): DurationBoundary`
  - shaped by: return `DurationBoundary` decomposed → [required, delay] (Callback delivery)
- `duration_emit` — `fun durationEmit(value: ULong, f: DurationCallback, onError: JniErrorHandler<Unit>)`
- `duration_optional` — `fun durationOptional(value: ULong?, onError: JniErrorHandler<ULong?>): ULong?`
- `duration_out_of_range` — `fun durationOutOfRange(onError: JniErrorHandler<ULong?>): ULong?`
- `hold_echo` — `fun holdEcho(h: Hold, onError: JniErrorHandler<Hold>): Hold`
  - shaped by: return `Hold` decomposed → [tag, for_v0] (Callback delivery)
- `hold_policy_echo` — `fun holdPolicyEcho(p: HoldPolicy, onError: JniErrorHandler<HoldPolicy>): HoldPolicy`
- `label_reverse` — `fun labelReverse(l: String, onError: JniErrorHandler<String>): String`
- `label_series_echo` — `fun labelSeriesEcho(labels: List<String>, onError: JniErrorHandler<List<String>>): List<String>`
- `ledger_each` — `fun ledgerEach(n: Long, sink: LedgerCallback, onError: JniErrorHandler<Unit>)`
- `ledger_new` — `fun <R> ledgerNew(n: Long, onError: JniErrorHandler<R>, build: LedgerBuilder<R>): R`
  - shaped by: return `Ledger` decomposed → [ledgerFiled__summary__count, ledgerFiled__summary__total, ledgerFiled__taken, ledgerFiled__origin__secs, ledgerFiled__origin__nanos, ledgerFiled__outcome__tag, ledgerFiled__outcome__found_v0, ledgerFiled__outcome__failed_v0, ledgerFiled__label, ledgerArchived__summary__count, ledgerArchived__summary__total, ledgerArchived__taken, ledgerArchived__origin__secs, ledgerArchived__origin__nanos, ledgerArchived__outcome__tag, ledgerArchived__outcome__found_v0, ledgerArchived__outcome__failed_v0, ledgerArchived__label] (Callback delivery)
- `lookup_each` — `fun lookupEach(n: Long, total: Double, sink: LookupCallback, onError: JniErrorHandler<Unit>)`
- `lookup_of` — `fun lookupOf(count: Long, total: Double, onError: JniErrorHandler<Lookup>): Lookup`
  - shaped by: return `Lookup` decomposed → [tag, found_v0, failed_v0] (Callback delivery)
- `marker_of` — `fun markerOf(which: Int, onError: JniErrorHandler<Marker>): Marker`
  - shaped by: return `Marker` decomposed → [tag, ranked_v0] (Callback delivery)
- `object_boundary_value` — `fun objectBoundaryValue(value: ObjectBoundary, onError: JniErrorHandler<Long>): Long`
- `observation_new` — `fun observationNew(which: Int, withFallback: Boolean, onError: JniErrorHandler<Observation>): Observation`
- `observation_which` — `fun observationWhich(o: Observation, onError: JniErrorHandler<Int>): Int`
- `payload_priority` — `fun payloadPriority(p: Payload, onError: JniErrorHandler<Priority>): Priority`
- `percent_invalid_output` — `fun percentInvalidOutput(onError: JniErrorHandler<Int?>): Int?`
- `percent_optional` — `fun percentOptional(p: Int?, onError: JniErrorHandler<Int?>): Int?`
- `percent_scale` — `fun percentScale(p: Int, factor: Int, onError: JniErrorHandler<Int>): Int`
- `plain_note_echo` — `fun plainNoteEcho(note: String?, onError: JniErrorHandler<String?>): String?`
- `priority_or` — `fun priorityOr(p: Priority?, fallback: Priority, onError: JniErrorHandler<Priority>): Priority`
- `priority_weight` — `fun priorityWeight(p: Priority, onError: JniErrorHandler<Int>): Int`
- `reading_each` — `fun readingEach(n: Int, sink: ReadingCallback, onError: JniErrorHandler<Unit>)`
- `reading_maybe` — `fun readingMaybe(which: Int, onError: JniErrorHandler<Reading?>): Reading?`
  - shaped by: return `Reading` decomposed → [tag, exact_v0, range_low, range_high, tagged_v0, tagged_v1, companion_v0] (Callback delivery)
- `reading_of` — `fun readingOf(which: Int, onError: JniErrorHandler<Reading>): Reading`
  - shaped by: return `Reading` decomposed → [tag, exact_v0, range_low, range_high, tagged_v0, tagged_v1, companion_v0] (Callback delivery)
- `reading_series` — `fun readingSeries(n: Int, onError: JniErrorHandler<List<Reading>>): List<Reading>`
  - shaped by: return `Reading` decomposed → [tag, exact_v0, range_low, range_high, tagged_v0, tagged_v1, companion_v0] (Callback delivery)
- `report_each` — `fun reportEach(n: Long, sink: ReportCallback, onError: JniErrorHandler<Unit>)`
- `stamp_new` — `fun stampNew(secs: Long, nanos: Long, onError: JniErrorHandler<Stamp>): Stamp`
  - shaped by: return `Stamp` decomposed → [secs, nanos] (Callback delivery)
- `stamp_series` — `fun stampSeries(count: Long, onError: JniErrorHandler<List<Stamp>>): List<Stamp>`
  - shaped by: return `Stamp` decomposed → [secs, nanos] (Callback delivery)
- `tagged_new` — `fun taggedNew(which: Int, onError: JniErrorHandler<Tagged>): Tagged`
- `tagged_rank` — `fun taggedRank(t: Tagged, onError: JniErrorHandler<Int>): Int`
- `unsigned_data_maybe` — `fun unsignedDataMaybe(value: Unsigned, onError: JniErrorHandler<ULong?>): ULong?`
- `unsigned_emit` — `fun unsignedEmit(value: ULong, f: u64Callback, onError: JniErrorHandler<Unit>)`
- `unsigned_optional` — `fun unsignedOptional(value: ULong?, onError: JniErrorHandler<ULong?>): ULong?`
- `unsigned_round_trip` — `fun unsignedRoundTrip(byte: Int, short: Int, int: Long, long: ULong, maybeLong: ULong?, onError: JniErrorHandler<Unsigned>): Unsigned`
  - shaped by: return `Unsigned` decomposed → [byte, short, int, long, maybeLong] (Callback delivery)
- `unsigned_series` — `fun unsignedSeries(onError: JniErrorHandler<List<ULong>>): List<ULong>`
  - shaped by: return `u64` decomposed → [] (Callback delivery)
- `wrapped_fields_sum` — `fun wrappedFieldsSum(w: WrappedFields, onError: JniErrorHandler<Long>): Long`

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
- `storage_try_from_stamp` — `fun storageTryFromStamp(s: Stamp, tag: ByteArray, onBindingError: JniErrorHandler<Storage>, onError: StorageErrorHandler<Storage>): Storage`
  - shaped by: domain error `StorageError` decomposed → onError [message, handle] (binding failures → onBindingError)
- `storage_try_with_label` — `fun storageTryWithLabel(label: String, onBindingError: JniErrorHandler<Storage>, onError: StorageErrorHandler<Storage>): Storage`
  - shaped by: domain error `StorageError` decomposed → onError [message, handle] (binding failures → onBindingError)

## class `io.prebindgen.covertest.esc_pkg.Esc_Probe` (ptr_class, Rust `EscapeProbe`)

- `escape_probe_new` — `fun escapeProbeNew(value: Long, onError: JniErrorHandler<Esc_Probe>): Esc_Probe`
- `escape_probe_value` — `fun escapeProbeValue(onError: JniErrorHandler<Long>): Long`

## class `io.prebindgen.covertest.Payload` (data_class, Rust `Payload`)

- `payload_label_len` — `fun labelLen(onError: JniErrorHandler<Long?>): Long?`

## class `io.prebindgen.covertest.model.Stamp` (data_class, Rust `Stamp`)

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
- `Arrays`: data_class → `io.prebindgen.covertest.model.Arrays` (wire `jni :: objects :: JObject`)
- `BlobValue`: data_class → `io.prebindgen.covertest.model.BlobValue` (wire `jni :: objects :: JObject`, input `JObject` opt-in)
- `CacheConfig`: data_class → `io.prebindgen.covertest.model.CacheConfig` (wire `jni :: objects :: JObject`)
- `DurationBoundary`: data_class → `io.prebindgen.covertest.model.DurationBoundary` (wire `jni :: objects :: JObject`, input `JObject` opt-in)
- `EscapeProbe`: ptr_class → `io.prebindgen.covertest.esc_pkg.Esc_Probe` (wire `jni :: sys :: jlong`)
- `Hold`: sealed_class → `io.prebindgen.covertest.model.Hold` (wire `?`)
- `HoldPolicy`: data_class → `io.prebindgen.covertest.model.HoldPolicy` (wire `jni :: objects :: JObject`)
- `Lookup`: sealed_class → `io.prebindgen.covertest.model.Lookup` (wire `?`)
- `Marker`: sealed_class → `io.prebindgen.covertest.model.Marker` (wire `?`)
- `ObjectBoundary`: data_class → `io.prebindgen.covertest.model.ObjectBoundary` (wire `jni :: objects :: JObject`, input `JObject` opt-in)
- `ObjectBoundary16`: data_class → `io.prebindgen.covertest.model.ObjectBoundary16` (wire `jni :: objects :: JObject`)
- `ObjectBoundary2`: data_class → `io.prebindgen.covertest.model.ObjectBoundary2` (wire `jni :: objects :: JObject`)
- `ObjectBoundary32`: data_class → `io.prebindgen.covertest.model.ObjectBoundary32` (wire `jni :: objects :: JObject`)
- `ObjectBoundary4`: data_class → `io.prebindgen.covertest.model.ObjectBoundary4` (wire `jni :: objects :: JObject`)
- `ObjectBoundary63`: data_class → `io.prebindgen.covertest.model.ObjectBoundary63` (wire `jni :: objects :: JObject`)
- `ObjectBoundary64`: data_class → `io.prebindgen.covertest.model.ObjectBoundary64` (wire `jni :: objects :: JObject`)
- `ObjectBoundary8`: data_class → `io.prebindgen.covertest.model.ObjectBoundary8` (wire `jni :: objects :: JObject`)
- `ObjectBoundaryLeaf`: data_class → `io.prebindgen.covertest.model.ObjectBoundaryLeaf` (wire `jni :: objects :: JObject`)
- `Observation`: data_class → `io.prebindgen.covertest.model.Observation` (wire `jni :: objects :: JObject`)
- `Payload`: data_class → `io.prebindgen.covertest.Payload` (wire `jni :: objects :: JObject`)
- `PayloadHandler`: ptr_class → `io.prebindgen.covertest.PayloadHandler` (wire `jni :: sys :: jlong`)
- `PayloadVecHandler`: ptr_class → `io.prebindgen.covertest.PayloadVecHandler` (wire `jni :: sys :: jlong`)
- `Priority`: enum_class → `io.prebindgen.covertest.model.Priority` (wire `jni :: sys :: jint`)
- `Reading`: sealed_class → `io.prebindgen.covertest.model.Reading` (wire `?`)
- `RepliesConfig`: data_class → `io.prebindgen.covertest.model.RepliesConfig` (wire `jni :: objects :: JObject`)
- `Report`: ptr_class → `io.prebindgen.covertest.model.Report` (wire `jni :: sys :: jlong`)
- `Stamp`: data_class → `io.prebindgen.covertest.model.Stamp` (wire `jni :: objects :: JObject`)
- `Storage`: ptr_class → `io.prebindgen.covertest.Storage` (wire `jni :: sys :: jlong`)
- `StorageError`: ptr_class → `io.prebindgen.covertest.errors.StorageError` (wire `jni :: sys :: jlong`)
- `StorageHandler`: ptr_class → `io.prebindgen.covertest.StorageHandler` (wire `jni :: sys :: jlong`)
- `Summary`: ptr_class → `io.prebindgen.covertest.analytics.Summary` (wire `jni :: sys :: jlong`)
- `Tagged`: data_class → `io.prebindgen.covertest.model.Tagged` (wire `jni :: objects :: JObject`)
- `Unsigned`: data_class → `io.prebindgen.covertest.model.Unsigned` (wire `jni :: objects :: JObject`)
- `WrappedFields`: data_class → `io.prebindgen.covertest.WrappedFields` (wire `jni :: objects :: JObject`)

## conversions

- `convert!(Celsius)`: input `Into` ⇄ `i32`, output `Into` ⇄ `i32`
- `convert!(Duration)`: input `#[prebindgen]` fn `duration_from_millis`, output `#[prebindgen]` fn `duration_to_millis`
- `convert!(Label)`: input `#[prebindgen]` fn `label_in`, output `#[prebindgen]` fn `label_out`
- `convert!(Millis)`: input `#[prebindgen]` fn `millis_from_long`, output `#[prebindgen]` fn `millis_value`
- `convert!(Percent)`: input `TryInto` ⇄ `i32`, output `#[prebindgen]` fn `percent_out`

## rust-side-only types

- `Ledger` (never materializes in Kotlin)
