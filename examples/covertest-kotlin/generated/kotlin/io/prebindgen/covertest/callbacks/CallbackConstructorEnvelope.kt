package io.prebindgen.covertest.callbacks

public class CallbackConstructorEnvelope internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackConstructorEnvelope = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackConstructorEnvelope(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop4(raw) }
}
