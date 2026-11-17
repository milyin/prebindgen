package io.prebindgen.covertest.payloadenums

public sealed interface WireCheckedEvent : AutoCloseable {
public data object Empty : io.prebindgen.covertest.payloadenums.WireCheckedEvent { override fun close() {} }
public data class Data(
    public val first: io.prebindgen.covertest.callbackreturns.CallbackReturnResource,
    public val marker: kotlin.Long,
    public val last: io.prebindgen.covertest.callbackreturns.CallbackReturnResource
) : io.prebindgen.covertest.payloadenums.WireCheckedEvent
 { override fun close() { this.first.close()
this.last.close()
 } }
}
