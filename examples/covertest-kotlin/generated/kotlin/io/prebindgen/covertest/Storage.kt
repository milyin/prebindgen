package io.prebindgen.covertest

public class Storage internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): Storage = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = Storage(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_Storage_9651261943550368(raw) }
}
