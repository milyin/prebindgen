package io.prebindgen.covertest.callbackreturns

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class CallbackFixedResult(
    public val first: io.prebindgen.covertest.callbackreturns.CallbackReturnResource,
    public val optional: io.prebindgen.covertest.callbackreturns.CallbackReturnResource?,
    public val character: kotlin.Int,
    public val last: io.prebindgen.covertest.callbackreturns.CallbackReturnResource
)
