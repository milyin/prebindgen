package io.prebindgen.covertest.callbacks

public class CallbackSet internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackSet = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackSet(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop14(raw) }
}
