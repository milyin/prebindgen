package io.prebindgen.covertest.getters

public class GetterLeaf internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = -5998258300314906714L
    public fun take(): GetterLeaf = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = GetterLeaf(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_GetterLeaf_b28f97ae79569aa8(raw) }
}
