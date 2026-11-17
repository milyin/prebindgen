package io.prebindgen.covertest.payloadenums

public sealed interface WireOwnedEvent : AutoCloseable {
public data object Empty : io.prebindgen.covertest.payloadenums.WireOwnedEvent { override fun close() {} }
public data class Pair(
    public val first: io.prebindgen.covertest.callbackreturns.CallbackReturnResource,
    public val second: io.prebindgen.covertest.callbackreturns.CallbackReturnResource
) : io.prebindgen.covertest.payloadenums.WireOwnedEvent
 { override fun close() { this.first.close()
this.second.close()
 } }
public data class Text(
    public val v0: kotlin.String
) : io.prebindgen.covertest.payloadenums.WireOwnedEvent
 { override fun close() {  } }
}
