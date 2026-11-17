package io.prebindgen.covertest.outputcontracts

public class DurationPair internal constructor(raw: Long) : io.prebindgen.covertest.outputcontracts.NativeHandle(raw) {
    override val handleType: Long get() = -8275453696664369409L
    public fun take(): DurationPair = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = DurationPair(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.outputcontracts.JNINative.drop_DurationPair_a80e8ee6ea4081d2(raw) }
}
