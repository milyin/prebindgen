package io.prebindgen.covertest.model

public class ReportStruct internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): ReportStruct = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = ReportStruct(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop24(raw) }
}
