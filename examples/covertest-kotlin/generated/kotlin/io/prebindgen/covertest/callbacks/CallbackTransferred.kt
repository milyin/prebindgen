package io.prebindgen.covertest.callbacks

public class CallbackTransferred internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackTransferred = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackTransferred(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop15(raw) }
}
