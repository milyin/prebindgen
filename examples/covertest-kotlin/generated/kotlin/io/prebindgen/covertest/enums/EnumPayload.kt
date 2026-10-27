package io.prebindgen.covertest.enums

public class EnumPayload internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): EnumPayload = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = EnumPayload(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop21(raw) }
}
