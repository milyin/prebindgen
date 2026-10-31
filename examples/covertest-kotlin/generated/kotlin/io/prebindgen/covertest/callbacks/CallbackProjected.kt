package io.prebindgen.covertest.callbacks

public class CallbackProjected internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackProjected = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackProjected(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackProjected_74e71cedcb164780(raw) }
}
