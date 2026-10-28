package io.prebindgen.covertest.callbacks

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class CallbackUniqueRow(
    public val `resource`: io.prebindgen.covertest.callbacks.CallbackUniqueResource,
    public val `label`: kotlin.String
)
