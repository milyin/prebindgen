package io.prebindgen.covertest.callbackboundaries

public class CallbackBoundaryHandler internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): CallbackBoundaryHandler = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackBoundaryHandler(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop1(raw) }
}
