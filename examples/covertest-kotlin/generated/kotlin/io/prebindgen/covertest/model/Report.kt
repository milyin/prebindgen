package io.prebindgen.covertest.model

public class Report internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): Report = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = Report(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_Report_965f7809bbde41dd(raw) }
}
