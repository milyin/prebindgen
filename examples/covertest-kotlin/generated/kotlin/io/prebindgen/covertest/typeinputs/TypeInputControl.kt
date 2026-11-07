package io.prebindgen.covertest.typeinputs

public class TypeInputControl internal constructor(raw: Long) : io.prebindgen.covertest.GcNativeHandle(raw, { io.prebindgen.covertest.JNINative.drop_TypeInputControl_7c64d7b5440ff06a(it) }) {
    override val handleType: Long get() = -2167593434678564446L
    public fun take(): TypeInputControl = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = TypeInputControl(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_TypeInputControl_7c64d7b5440ff06a(raw) }
}
