package io.prebindgen.covertest.callbackreturns

public class CallbackReturnJob internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackReturnJob = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackReturnJob(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackReturnJob_c59afe63a0d1957d(raw) }
}
