package io.prebindgen.covertest.gc

public class GcResource internal constructor(raw: Long) : io.prebindgen.covertest.GcNativeHandle(raw, { io.prebindgen.covertest.JNINative.drop_GcResource_0ee976436a63104d(it) }) {
    override val handleType: Long get() = -1699858189008869541L
    public fun take(): GcResource = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = GcResource(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_GcResource_0ee976436a63104d(raw) }
}
