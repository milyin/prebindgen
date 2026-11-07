package io.prebindgen.perftest

public class Storage internal constructor(raw: Long) : io.prebindgen.perftest.NativeHandle(raw) {
    public fun take(): Storage = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = Storage(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.perftest.JNINative.drop_Storage_9651261943550368(raw) }
}
