package io.prebindgen.covertest.callbacks

public class CallbackOwned internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackOwned = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackOwned(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw) }
}
