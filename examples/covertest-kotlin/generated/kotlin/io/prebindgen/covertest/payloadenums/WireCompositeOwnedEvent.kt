package io.prebindgen.covertest.payloadenums

public sealed interface WireCompositeOwnedEvent : AutoCloseable {
public data object Empty : io.prebindgen.covertest.payloadenums.WireCompositeOwnedEvent { override fun close() {} }
public data class Data(
    public val resource: io.prebindgen.covertest.callbackreturns.CallbackReturnResource,
    public val detail: io.prebindgen.covertest.payloadenums.WireEvent
) : io.prebindgen.covertest.payloadenums.WireCompositeOwnedEvent
 { override fun close() { this.resource.close()
 } }
}
