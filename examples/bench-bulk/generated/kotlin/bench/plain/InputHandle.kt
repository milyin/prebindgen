package bench.plain

public class InputHandle internal constructor(raw: Long) : bench.plain.NativeHandle(raw) {
    public fun take(): InputHandle = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = InputHandle(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { bench.plain.JNINative.drop_InputHandle_7cc67535bbd4a8ee(raw) }
}
