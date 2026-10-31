package io.prebindgen.covertest.model

public class ReportStruct internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): ReportStruct = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = ReportStruct(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_ReportStruct_2659d7dec79ecce0(raw) }
}
