package io.prebindgen.covertest.options

public class OptionCtor internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): OptionCtor = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = OptionCtor(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop19(raw) }
}
