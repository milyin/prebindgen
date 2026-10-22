package PACKAGE

/** Owns one boxed Rust allocation. The low bit records close or consumption. */
public abstract class NativeHandle internal constructor(raw: Long) : AutoCloseable {
    @Volatile internal var raw: Long = raw
        private set
    internal val address: Long = raw
    public val isClosed: Boolean get() = raw == 0L || raw and 1L != 0L
    internal fun markConsumed() { raw = raw or 1L }
    protected abstract fun freeNative(raw: Long)
    final override fun close() { synchronized(this) {
        if (!isClosed) {
            val owned = raw
            markConsumed()
            freeNative(owned)
        }
    } }
}

internal class HandleValidationFailure(message: String) : IllegalStateException(message)

// One order for all parameters, including selected constructor arguments.
// Generated constructors are internal: one Kotlin owner exists per allocation.
internal fun <R> withHandleLocks(handles: List<NativeHandle>, body: () -> R): R {
    val ordered = handles.distinct().sortedBy { it.address }
    fun lock(index: Int): R = if (index == ordered.size) {
        if (ordered.any { it.isClosed }) throw HandleValidationFailure("closed or consumed handle")
        body()
    } else synchronized(ordered[index]) { lock(index + 1) }
    return lock(0)
}
