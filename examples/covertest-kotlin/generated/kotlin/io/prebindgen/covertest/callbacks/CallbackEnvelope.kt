package io.prebindgen.covertest.callbacks

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class CallbackEnvelope(
    public val `handle`: io.prebindgen.covertest.callbacks.CallbackOwned,
    public val `label`: kotlin.String
)
