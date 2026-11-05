package io.prebindgen.covertest.gc

public class PlainResource internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): PlainResource = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = PlainResource(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_PlainResource_b08b40798e6273bd(raw) }
}
