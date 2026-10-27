package io.prebindgen.covertest.callbackreturns

public class CallbackReturnJob internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackReturnJob = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackReturnJob(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop11(raw) }
}
