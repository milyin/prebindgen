package io.prebindgen.covertest.bulk

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class BulkHandleText(
    public val `handle`: io.prebindgen.covertest.bulk.BulkHandle,
    public val `text`: kotlin.String
)
