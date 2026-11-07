package io.prebindgen.covertest.callbacks

public class CallbackRowResource internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = -37292178962141063L
    public fun take(): CallbackRowResource = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackRowResource(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(raw) }
}
