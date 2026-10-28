package io.prebindgen.covertest.callbacks

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class CallbackObservedRow(
    public val `resource`: io.prebindgen.covertest.callbacks.CallbackRowResource,
    public val `label`: kotlin.String
)
