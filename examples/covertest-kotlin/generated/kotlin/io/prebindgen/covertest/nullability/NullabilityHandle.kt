package io.prebindgen.covertest.nullability

public class NullabilityHandle internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = 3622710398308042493L
    public fun take(): NullabilityHandle = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = NullabilityHandle(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_NullabilityHandle_aefadfc6a0601b82(raw) }
}
