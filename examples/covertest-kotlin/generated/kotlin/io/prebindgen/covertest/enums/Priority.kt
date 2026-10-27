package io.prebindgen.covertest.enums

public enum class Priority(public val wireValue: kotlin.Int) {
    `Low`(0),
    `Normal`(1),
    `High`(2);

    public companion object Companion {
        public fun fromWire(value: kotlin.Int): Priority = when (value) {
            0 -> `Low`
            1 -> `Normal`
            2 -> `High`
            else -> throw kotlin.IllegalArgumentException("invalid enum discriminant: $value")
        }
    }
}
