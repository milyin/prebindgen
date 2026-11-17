package io.prebindgen.covertest.bulk

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class BulkCleanupBranches(
    public val first: io.prebindgen.covertest.bulk.BulkHandle,
    public val second: io.prebindgen.covertest.bulk.BulkCleanupChoice?
)
