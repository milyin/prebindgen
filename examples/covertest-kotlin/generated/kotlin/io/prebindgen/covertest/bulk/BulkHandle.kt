package io.prebindgen.covertest.bulk

public class BulkHandle internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = -1884207526850516181L
    public fun take(): BulkHandle = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = BulkHandle(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5(raw) }
}
