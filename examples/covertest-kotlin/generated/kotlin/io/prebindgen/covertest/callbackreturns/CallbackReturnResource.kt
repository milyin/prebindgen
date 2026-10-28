package io.prebindgen.covertest.callbackreturns

public class CallbackReturnResource internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackReturnResource = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackReturnResource(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw) }
}
