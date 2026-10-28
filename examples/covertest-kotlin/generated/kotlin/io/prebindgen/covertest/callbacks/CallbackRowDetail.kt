package io.prebindgen.covertest.callbacks

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class CallbackRowDetail(
    public val `resource`: io.prebindgen.covertest.callbacks.CallbackRowResource?,
    public val `note`: kotlin.String
)
