package io.prebindgen.covertest.getters

public class GetterOwner internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): GetterOwner = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = GetterOwner(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_GetterOwner_bc1803eb5b8c76ef(raw) }
}
