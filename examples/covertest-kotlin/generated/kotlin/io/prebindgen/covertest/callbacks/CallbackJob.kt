package io.prebindgen.covertest.callbacks

public class CallbackJob internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = -5815232616195323326L
    public fun take(): CallbackJob = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackJob(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackJob_7f5ec1b2a514af2f(raw) }
}
