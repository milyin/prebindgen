# prebindgen-jni binding report

Base package: `io.prebindgen.covertest`

## package `io.prebindgen.covertest`

- class `Dossier` (data class)
- class `Holder` (data class)
- `Payload.labelLen` ← `payload_label_len`
- class `Payload` (data class)
- class `PayloadHandler` (handle)
- class `PayloadVecHandler` (handle)
- `Storage.len` ← `storage_len`
- `Storage.contains` ← `storage_contains`
- `Storage.withPayload` ← `storage_with_payload`
- class `Storage` (handle)
- class `StorageHandler` (handle)
- class `WrappedFields` (data class)
- `val COVER_MAGIC: Long`
- `val COVER_TAG: String`
- `val COVER_TAG_RUNTIME: String`
- `val COVER_VERSION: String`
- `val COVER_BANNER: String`
- `stringNew` ← `string_new`

## package `io.prebindgen.covertest.analytics`

- `Summary.count` ← `summary_count`
- `Summary.total` ← `summary_total`
- `Summary.scaled` ← `summary_scaled`
- `Summary.mean` ← `summary_mean`
- `Summary.of` ← `summary_new`
- `Summary.fromMean` ← `summary_from_mean`
- class `Summary` (gc-managed handle)
- class `SummaryVault` (handle)
- `storageSummary` ← `storage_summary`
- `describeSummary` ← `summary_describe`
- `storageMatchesSummary` ← `storage_matches_summary`
- `storageSummaryHandle` ← `storage_summary_handle`
- `summaryTotalRaw` ← `summary_total_raw`
- `storageSummaryFull` ← `storage_summary_full`
- `storageSummaryProbe` ← `storage_summary_probe`
- `storageExpectSummary` ← `storage_expect_summary`
- `summaryPrefer` ← `summary_prefer`
- `summaryMerge` ← `summary_merge`
- `summaryTotalOpt` ← `summary_total_opt`
- `summarySeries` ← `summary_series`
- `summarySeriesOpt` ← `summary_series_opt`
- `archiveNew` ← `archive_new`
- `archiveStore` ← `archive_store`
- `archiveLatest` ← `archive_latest`

## package `io.prebindgen.covertest.errors`

- `StorageError.message` ← `storage_error_message`
- class `StorageError` (handle)

## package `io.prebindgen.covertest.esc_pkg`

- `Esc_Probe.escapeProbeValue` ← `escape_probe_value`
- `Esc_Probe.escapeProbeNew` ← `escape_probe_new`
- class `Esc_Probe` (handle)

## package `io.prebindgen.covertest.model`

- class `Annotated` (data class)
- class `Arrays` (data class)
- class `BlobValue` (data class)
- class `CacheConfig` (data class)
- class `DurationBoundary` (data class)
- class `Hold` (sealed interface)
- class `HoldPolicy` (data class)
- class `Lookup` (sealed interface)
- class `Marker` (sealed interface)
- class `ObjectBoundary` (data class)
- class `ObjectBoundary16` (data class)
- class `ObjectBoundary2` (data class)
- class `ObjectBoundary32` (data class)
- class `ObjectBoundary4` (data class)
- class `ObjectBoundary63` (data class)
- class `ObjectBoundary64` (data class)
- class `ObjectBoundary8` (data class)
- class `ObjectBoundaryLeaf` (data class)
- class `Observation` (data class)
- class `Priority` (enum class)
- class `Probe` (handle)
- class `Reading` (sealed interface)
- class `RepliesConfig` (data class)
- class `Report` (handle)
- class `Span` (handle)
- class `SpanHolder` (handle)
- `Stamp.secs` ← `stamp_secs`
- `Stamp.nanos` ← `stamp_nanos`
- class `Stamp` (data class)
- class `Tagged` (data class)
- class `Unsigned` (data class)
- class `Verdict` (data class)
- `payloadPriority` ← `payload_priority`
- `priorityWeight` ← `priority_weight`
- `priorityOr` ← `priority_or`
- `stampNew` ← `stamp_new`
- `stampSeries` ← `stamp_series`
- `celsiusDouble` ← `celsius_double`
- `percentScale` ← `percent_scale`
- `percentOptional` ← `percent_optional`
- `percentInvalidOutput` ← `percent_invalid_output`
- `labelReverse` ← `label_reverse`
- `labelSeriesEcho` ← `label_series_echo`
- `annotatedNew` ← `annotated_new`
- `annotatedAlternateValue` ← `annotated_alternate_value`
- `annotatedTtl` ← `annotated_ttl`
- `annotatedPriority` ← `annotated_priority`
- `annotatedPayloadValue` ← `annotated_payload_value`
- `observationNew` ← `observation_new`
- `observationWhich` ← `observation_which`
- `taggedNew` ← `tagged_new`
- `taggedRank` ← `tagged_rank`
- `markerOf` ← `marker_of`
- `readingOf` ← `reading_of`
- `readingMaybe` ← `reading_maybe`
- `readingSeries` ← `reading_series`
- `readingEach` ← `reading_each`
- `lookupOf` ← `lookup_of`
- `lookupEach` ← `lookup_each`
- `verdictNew` ← `verdict_new`
- `dossierNew` ← `dossier_new`
- `reportEach` ← `report_each`
- `probeNew` ← `probe_new`
- `probeEach` ← `probe_each`
- `ledgerEach` ← `ledger_each`
- `spanHolderNew` ← `span_holder_new`
- `boxedNoteEcho` ← `boxed_note_echo`
- `plainNoteEcho` ← `plain_note_echo`
- `wrappedFieldsSum` ← `wrapped_fields_sum`
- `holderTagOr` ← `holder_tag_or`
- `boxedPayloadId` ← `boxed_payload_id`
- `boxedOptPayloadId` ← `boxed_opt_payload_id`
- `boxedOptPriorityWeight` ← `boxed_opt_priority_weight`
- `boxedElemIdSum` ← `boxed_elem_id_sum`
- `boxedRunIdSum` ← `boxed_run_id_sum`
- `sliceIdSum` ← `slice_id_sum`
- `refVecIdSum` ← `ref_vec_id_sum`
- `boxedLatest` ← `boxed_latest`
- `ledgerNew` ← `ledger_new`
- `archiveSetReading` ← `archive_set_reading`
- `archiveReading` ← `archive_reading`
- `archiveReadingMaybe` ← `archive_reading_maybe`
- `holdEcho` ← `hold_echo`
- `holdPolicyEcho` ← `hold_policy_echo`
- `cacheConfigWeight` ← `cache_config_weight`
- `objectBoundaryValue` ← `object_boundary_value`
- `unsignedRoundTrip` ← `unsigned_round_trip`
- `unsignedOptional` ← `unsigned_optional`
- `unsignedDataMaybe` ← `unsigned_data_maybe`
- `unsignedEmit` ← `unsigned_emit`
- `unsignedSeries` ← `unsigned_series`
- `blobValueNew` ← `blob_value_new`
- `blobValueEcho` ← `blob_value_echo`
- `arraysEcho` ← `arrays_echo`
- `durationOptional` ← `duration_optional`
- `boxedDurationEcho` ← `boxed_duration_echo`
- `durationBoundaryEcho` ← `duration_boundary_echo`
- `durationEmit` ← `duration_emit`
- `durationOutOfRange` ← `duration_out_of_range`

## package `io.prebindgen.covertest.storage`

- `storageNew` ← `storage_new`
- `storageGet` ← `storage_get`
- `storagePutByTake` ← `storage_put_by_take`
- `storagePutByRead` ← `storage_put_by_read`
- `storagePutSlice` ← `storage_put_slice`
- `storageGetVec` ← `storage_get_vec`
- `payloadHandlerNew` ← `payload_handler_new`
- `storageCallback` ← `storage_callback`
- `payloadVecHandlerNew` ← `payload_vec_handler_new`
- `storageCallbackVec` ← `storage_callback_vec`
- `storageTryWithLabel` ← `storage_try_with_label`
- `storageTryFromStamp` ← `storage_try_from_stamp`
- `storageShards` ← `storage_shards`
- `storageShardsOpt` ← `storage_shards_opt`
- `storageHandlerNew` ← `storage_handler_new`
- `storageEmit` ← `storage_emit`
- `storageTotalLen` ← `storage_total_len`
- `storageLabels` ← `storage_labels`
- `storagePutOpt` ← `storage_put_opt`
- `addMillis` ← `millis_add`
