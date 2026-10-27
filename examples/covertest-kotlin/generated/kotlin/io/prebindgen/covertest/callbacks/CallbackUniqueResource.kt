package io.prebindgen.covertest.callbacks

public class CallbackUniqueResource internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackUniqueResource = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackUniqueResource(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop16(raw) }
}
