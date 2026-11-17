package bench.plain

public class WideRowReturnHandler internal constructor(raw: Long) : bench.plain.NativeHandle(raw) {
    override val handleType: Long get() = -5882193418749463495L
    public fun take(): WideRowReturnHandler = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = WideRowReturnHandler(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { bench.plain.JNINative.drop_WideRowReturnHandler_ed750f04931eb9ca(raw) }
}
