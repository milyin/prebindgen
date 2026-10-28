package io.prebindgen.covertest.resources

public class OwnedResource internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): OwnedResource = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = OwnedResource(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(raw) }
}
