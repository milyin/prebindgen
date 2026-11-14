package io.prebindgen.covertest.callbacks

public class CallbackConstructorJob internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = 4508689536636120186L
    public fun take(): CallbackConstructorJob = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackConstructorJob(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackConstructorJob_c88b6ee076189420(raw) }
}
