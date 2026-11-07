package bench.bulk

public class RowReturnHandler internal constructor(raw: Long) : bench.bulk.NativeHandle(raw) {
    public fun take(): RowReturnHandler = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = RowReturnHandler(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { bench.bulk.JNINative.drop_RowReturnHandler_0972ff7cb1556afa(raw) }
}
