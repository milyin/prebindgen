package io.prebindgen.covertest.callbacks

public class CallbackTransferred internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = 6919471631145217327L
    public fun take(): CallbackTransferred = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackTransferred(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackTransferred_eab9ef9641575576(raw) }
}
