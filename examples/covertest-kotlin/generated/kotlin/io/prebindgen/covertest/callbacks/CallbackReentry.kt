package io.prebindgen.covertest.callbacks

public class CallbackReentry internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackReentry = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackReentry(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop9(raw) }
}
