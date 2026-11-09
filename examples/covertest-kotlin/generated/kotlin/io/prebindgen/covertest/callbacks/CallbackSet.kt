package io.prebindgen.covertest.callbacks

public class CallbackSet internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = 8475426775633789593L
    public fun take(): CallbackSet = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackSet(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackSet_42869ab2828f84d4(raw) }
}
