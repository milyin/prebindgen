package PACKAGE

/** Owns one boxed Rust allocation. The low bit records close or consumption. */
public abstract class NativeHandle internal constructor(raw: Long) : AutoCloseable {
    @Volatile internal var raw: Long = raw
        private set
    internal val address: Long = raw
    internal val lock = java.util.concurrent.locks.ReentrantLock()
    public val isClosed: Boolean get() = raw == 0L || raw and 1L != 0L
    internal fun markConsumed() { raw = raw or 1L }
    protected abstract fun freeNative(raw: Long)
    final override fun close() {
        lock.lock()
        try {
            if (!isClosed) {
                val owned = raw
                markConsumed()
                freeNative(owned)
            }
        } finally { lock.unlock() }
    }
}

// One order for all parameters, including selected constructor arguments.
// Generated constructors are internal: one Kotlin owner exists per allocation.
internal fun <R> withHandleLocks(handles: List<NativeHandle>, body: () -> R): R {
    val ordered = handles.distinct().sortedBy { it.address }
    var acquired = 0
    try {
        for (handle in ordered) {
            handle.lock.lock()
            acquired++
        }
        check(ordered.none { it.isClosed }) { "closed or consumed handle" }
        return body()
    } finally {
        for (index in acquired - 1 downTo 0) ordered[index].lock.unlock()
    }
}
