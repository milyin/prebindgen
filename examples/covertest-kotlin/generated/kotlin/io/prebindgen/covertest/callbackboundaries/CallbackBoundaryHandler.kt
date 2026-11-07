package io.prebindgen.covertest.callbackboundaries

public class CallbackBoundaryHandler internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = -4614888541111252990L
    public fun take(): CallbackBoundaryHandler = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = CallbackBoundaryHandler(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_CallbackBoundaryHandler_0e611baf5b66c32b(raw) }
}
