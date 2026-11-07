package bench.plain

public class RowReturnHandler internal constructor(raw: Long) : bench.plain.NativeHandle(raw) {
    public fun take(): RowReturnHandler = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = RowReturnHandler(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { bench.plain.JNINative.drop_RowReturnHandler_0972ff7cb1556afa(raw) }
}
