package io.prebindgen.covertest.callbackreturns

public enum class CallbackReturnChoice(public val wireValue: kotlin.Int) {
    `First`(0),
    `Second`(1);

    public companion object Companion {
        public fun fromWire(value: kotlin.Int): CallbackReturnChoice = when (value) {
            0 -> `First`
            1 -> `Second`
            else -> throw kotlin.IllegalArgumentException("invalid enum discriminant: $value")
        }
    }
}
