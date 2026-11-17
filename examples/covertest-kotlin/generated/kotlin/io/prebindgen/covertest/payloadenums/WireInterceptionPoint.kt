package io.prebindgen.covertest.payloadenums

public enum class WireInterceptionPoint(public val wireValue: kotlin.Int) {
    Send(0),
    Route(1),
    Receive(2);

    public companion object Companion {
        public fun fromWire(`value`: kotlin.Int): WireInterceptionPoint = when (`value`) {
            0 -> Send
            1 -> Route
            2 -> Receive
            else -> throw kotlin.IllegalArgumentException("invalid enum discriminant: ${`value`}")
        }
    }
}
