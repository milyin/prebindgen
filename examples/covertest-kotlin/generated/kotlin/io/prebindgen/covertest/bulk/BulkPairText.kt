package io.prebindgen.covertest.bulk

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class BulkPairText(
    public val `borrowed_handle`: io.prebindgen.covertest.bulk.BulkHandle,
    public val `owned_handle`: io.prebindgen.covertest.bulk.BulkHandle,
    public val `text`: kotlin.String
)
