package io.prebindgen.covertest.callbacks

public class CallbackConstructorJob internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackConstructorJob = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackConstructorJob(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop5(raw) }
}
