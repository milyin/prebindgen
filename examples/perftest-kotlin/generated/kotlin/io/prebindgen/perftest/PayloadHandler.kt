package io.prebindgen.perftest

public class PayloadHandler internal constructor(raw: Long) : io.prebindgen.perftest.NativeHandle(raw) {
    override val handleType: Long get() = -2853349306431172153L
    public fun take(): PayloadHandler = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = PayloadHandler(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.perftest.JNINative.drop_PayloadHandler_8f188783a3c63cb5(raw) }
}
