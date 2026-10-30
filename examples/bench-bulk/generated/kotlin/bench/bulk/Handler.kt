package bench.bulk

public class Handler internal constructor(raw: Long) : bench.bulk.NativeHandle(raw) {
    public fun take(): Handler = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = Handler(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { bench.bulk.JNINative.drop_Handler_bb814c4111561dda(raw) }
}
