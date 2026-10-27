package io.prebindgen.covertest.callbacks

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class CallbackTransferredRow(
    public val `first`: io.prebindgen.covertest.callbacks.CallbackRowResource,
    public val `label`: kotlin.String,
    public val `second`: io.prebindgen.covertest.callbacks.CallbackRowResource
)
