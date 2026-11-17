package io.prebindgen.covertest.nullability

public enum class NullabilityChoice(public val wireValue: kotlin.Int) {
    A(0),
    B(1);

    public companion object Companion {
        public fun fromWire(`value`: kotlin.Int): NullabilityChoice = when (`value`) {
            0 -> A
            1 -> B
            else -> throw kotlin.IllegalArgumentException("invalid enum discriminant: ${`value`}")
        }
    }
}
