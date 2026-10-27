package io.prebindgen.covertest.analytics

public class SummaryValue internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): SummaryValue = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = SummaryValue(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop26(raw) }
}
