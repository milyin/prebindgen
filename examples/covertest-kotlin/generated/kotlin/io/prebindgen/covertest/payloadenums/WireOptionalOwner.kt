package io.prebindgen.covertest.payloadenums

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class WireOptionalOwner(
    public val enabled: kotlin.Boolean,
    public val resource: io.prebindgen.covertest.callbackreturns.CallbackReturnResource
)
