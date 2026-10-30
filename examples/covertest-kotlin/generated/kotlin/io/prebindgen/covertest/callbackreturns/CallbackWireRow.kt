package io.prebindgen.covertest.callbackreturns

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class CallbackWireRow(
    public val `resource`: io.prebindgen.covertest.callbackreturns.CallbackReturnResource,
    public val `character`: kotlin.Int
)
