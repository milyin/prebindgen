package io.prebindgen.covertest.options

public class OptionChoice internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = 3861337430608797521L
    public fun take(): OptionChoice = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = OptionChoice(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_OptionChoice_feab979b8bef4265(raw) }
}
