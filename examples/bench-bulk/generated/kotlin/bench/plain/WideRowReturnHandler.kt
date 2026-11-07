package bench.plain

public class WideRowReturnHandler internal constructor(raw: Long) : bench.plain.NativeHandle(raw) {
    public fun take(): WideRowReturnHandler = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = WideRowReturnHandler(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { bench.plain.JNINative.drop_WideRowReturnHandler_ed750f04931eb9ca(raw) }
}
