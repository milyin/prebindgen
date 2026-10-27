package io.prebindgen.covertest.enums

public enum class Code(public val wireValue: kotlin.Int) {
    `Bottom`(-2147483648),
    `MinusTwo`(-2),
    `MinusOne`(-1),
    `Zero`(0),
    `Seven`(7),
    `Eight`(8),
    `Top`(2147483647);

    public companion object Companion {
        public fun fromWire(value: kotlin.Int): Code = when (value) {
            -2147483648 -> `Bottom`
            -2 -> `MinusTwo`
            -1 -> `MinusOne`
            0 -> `Zero`
            7 -> `Seven`
            8 -> `Eight`
            2147483647 -> `Top`
            else -> throw kotlin.IllegalArgumentException("invalid enum discriminant: $value")
        }
    }
}
