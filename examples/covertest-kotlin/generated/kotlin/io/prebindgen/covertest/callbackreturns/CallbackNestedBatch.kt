package io.prebindgen.covertest.callbackreturns

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class CallbackNestedBatch(
    public val first: io.prebindgen.covertest.callbackreturns.CallbackReturnResource,
    public val observed: io.prebindgen.covertest.callbackreturns.CallbackReturnResource,
    public val groups: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>?>,
    public val marker: kotlin.Int,
    public val tail: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>,
    public val last: io.prebindgen.covertest.callbackreturns.CallbackReturnResource
)
