package io.prebindgen.covertest.callbacks

public class CallbackConstructor internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackConstructor = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackConstructor(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackConstructor_65ec2182d775ceb8(raw) }
}
