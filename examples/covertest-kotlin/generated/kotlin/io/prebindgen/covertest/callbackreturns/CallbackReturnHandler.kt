package io.prebindgen.covertest.callbackreturns

public class CallbackReturnHandler internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackReturnHandler = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackReturnHandler(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop10(raw) }
}
