package io.prebindgen.covertest.canonical

public class CanonToken internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = -1750105490603960763L
    public fun take(): CanonToken = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CanonToken(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7(raw) }
}
