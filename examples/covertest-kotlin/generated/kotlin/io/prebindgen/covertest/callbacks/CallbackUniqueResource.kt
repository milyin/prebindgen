package io.prebindgen.covertest.callbacks

public class CallbackUniqueResource internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = 7687277209708742138L
    public fun take(): CallbackUniqueResource = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackUniqueResource(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackUniqueResource_d31d0c3f2ae0419c(raw) }
}
