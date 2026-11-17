package io.prebindgen.covertest.callbackreturns

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class CallbackOptionalBatch(
    public val first: io.prebindgen.covertest.callbackreturns.CallbackReturnResource?,
    public val rows: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>,
    public val last: io.prebindgen.covertest.callbackreturns.CallbackReturnResource
)
