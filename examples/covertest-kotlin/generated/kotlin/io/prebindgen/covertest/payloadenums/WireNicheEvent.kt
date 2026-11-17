package io.prebindgen.covertest.payloadenums

public sealed interface WireNicheEvent : AutoCloseable {
public data object Empty : io.prebindgen.covertest.payloadenums.WireNicheEvent { override fun close() {} }
public data class Data(
    public val small: io.prebindgen.covertest.payloadenums.WireOptionalOwner?,
    public val wide: io.prebindgen.covertest.callbackreturns.CallbackReturnResource?
) : io.prebindgen.covertest.payloadenums.WireNicheEvent
 { override fun close() { this.small?.let { member0 -> member0.resource.close()
 }
this.wide?.let { member0 -> member0.close()
 }
 } }
}
