package io.prebindgen.covertest.typeinputs

public class TypeInputA internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = 4664678373365233526L
    public fun take(): TypeInputA = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = TypeInputA(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_TypeInputA_ac61bf30bfd3c448(raw) }
}
