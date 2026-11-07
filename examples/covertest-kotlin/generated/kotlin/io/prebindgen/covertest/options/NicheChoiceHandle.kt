package io.prebindgen.covertest.options

public class NicheChoiceHandle internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = -4211307990560966678L
    public fun take(): NicheChoiceHandle = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = NicheChoiceHandle(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_NicheChoiceHandle_820b60fc3ef75039(raw) }
}
