package io.prebindgen.covertest.borrowcopies

public enum class BorrowMode(public val wireValue: kotlin.Int) {
    Ready(0),
    Idle(1);

    public companion object Companion {
        public fun fromWire(`value`: kotlin.Int): BorrowMode = when (`value`) {
            0 -> Ready
            1 -> Idle
            else -> throw kotlin.IllegalArgumentException("invalid enum discriminant: ${`value`}")
        }
    }
}
