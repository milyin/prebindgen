package bench.plain

public class Handler internal constructor(raw: Long) : bench.plain.NativeHandle(raw) {
    override val handleType: Long get() = -3626799142228362805L
    public fun take(): Handler = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = Handler(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { bench.plain.JNINative.drop_Handler_bb814c4111561dda(raw) }
}
