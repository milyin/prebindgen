package io.prebindgen.covertest.callbackconfig

public class CallbackConfigHandler internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackConfigHandler = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackConfigHandler(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackConfigHandler_67dc52e4c6670317(raw) }
}
