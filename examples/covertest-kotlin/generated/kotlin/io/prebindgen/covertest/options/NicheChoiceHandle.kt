package io.prebindgen.covertest.options

public class NicheChoiceHandle internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): NicheChoiceHandle = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = NicheChoiceHandle(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop17(raw) }
}
