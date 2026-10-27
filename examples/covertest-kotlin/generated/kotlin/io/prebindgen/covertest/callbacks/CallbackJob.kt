package io.prebindgen.covertest.callbacks

public class CallbackJob internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackJob = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackJob(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop6(raw) }
}
