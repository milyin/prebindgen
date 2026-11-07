package io.prebindgen.covertest.callbacks

public class CallbackReentry internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = 1613580938739374374L
    public fun take(): CallbackReentry = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackReentry(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackReentry_6c6c0fff370f6efb(raw) }
}
