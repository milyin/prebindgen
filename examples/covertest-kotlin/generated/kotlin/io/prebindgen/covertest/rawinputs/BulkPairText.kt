package io.prebindgen.covertest.rawinputs

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class BulkPairText(
    public val borrowed_handle: io.prebindgen.covertest.rawinputs.BulkHandle,
    public val owned_handle: io.prebindgen.covertest.rawinputs.BulkHandle,
    public val text: kotlin.String
)
