package io.prebindgen.covertest.typeinputs

public class TypeInputB internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = 4664677273853605315L
    public fun take(): TypeInputB = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = TypeInputB(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_TypeInputB_ac61c230bfd3c961(raw) }
}
