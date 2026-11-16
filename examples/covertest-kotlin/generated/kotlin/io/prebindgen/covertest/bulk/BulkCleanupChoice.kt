package io.prebindgen.covertest.bulk

public sealed interface BulkCleanupChoice : AutoCloseable {
public data object Empty : io.prebindgen.covertest.bulk.BulkCleanupChoice { override fun close() {} }
public data class One(
    public val `value`: io.prebindgen.covertest.bulk.BulkHandle
) : io.prebindgen.covertest.bulk.BulkCleanupChoice
 { override fun close() { this.`value`.close()
 } }
public data class Two(
    public val `value`: io.prebindgen.covertest.bulk.BulkHandle
) : io.prebindgen.covertest.bulk.BulkCleanupChoice
 { override fun close() { this.`value`.close()
 } }
}
