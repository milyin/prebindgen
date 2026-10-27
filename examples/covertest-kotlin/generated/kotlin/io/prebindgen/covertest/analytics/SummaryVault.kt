package io.prebindgen.covertest.analytics

public class SummaryVault internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): SummaryVault = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = SummaryVault(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_Archive_0aa4417b9f320a03(raw) }
}
