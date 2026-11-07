package io.prebindgen.covertest.rawinputs

public enum class BulkKind(public val wireValue: kotlin.Int) {
    A(-3),
    B(9);

    public companion object Companion {
        public fun fromWire(`value`: kotlin.Int): BulkKind = when (`value`) {
            -3 -> A
            9 -> B
            else -> throw kotlin.IllegalArgumentException("invalid enum discriminant: ${`value`}")
        }
    }
}
