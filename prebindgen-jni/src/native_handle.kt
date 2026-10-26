package PACKAGE

/** Owns one boxed Rust allocation. The low bit records close or consumption. */
public abstract class NativeHandle internal constructor(raw: Long) : AutoCloseable {
    @Volatile internal var raw: Long = raw
        private set
    internal val address: Long = raw
    internal val lock = java.util.concurrent.locks.ReentrantLock()
    @Volatile internal var activeCalls: Int = 0
    public val isClosed: Boolean get() = raw == 0L || raw and 1L != 0L
    internal fun markConsumed() { raw = raw or 1L }
    // A synchronous callback must not close/move a value borrowed by its caller,
    // or block on a handle whose owner may be waiting for that callback.
    internal fun lockForNativeAccess() {
        if (nativeCallbackDepth.get() > 0) {
            if (lock.isHeldByCurrentThread && activeCalls != 0) {
                throw HandleValidationFailure("callback cannot reenter a handle already in native use")
            }
            val acquired = try { lock.tryLock(100, java.util.concurrent.TimeUnit.MILLISECONDS) }
                catch (failure: InterruptedException) {
                    Thread.currentThread().interrupt()
                    throw CallbackHandleBusyException("interrupted while waiting for a native handle", failure)
                }
            if (!acquired) throw CallbackHandleBusyException("callback timed out waiting for a native handle")
        } else {
            lock.lock()
        }
    }
    protected abstract fun freeNative(raw: Long)
    final override fun close() {
        lockForNativeAccess()
        try {
            check(activeCalls == 0) { "handle is in native use" }
            if (!isClosed) {
                val owned = raw
                markConsumed()
                freeNative(owned)
            }
        } finally { lock.unlock() }
    }
}

/** A callback could not acquire a contended handle within its bounded wait. */
public class CallbackHandleBusyException(message: String, cause: Throwable? = null) : IllegalStateException(message, cause)

internal class HandleValidationFailure(message: String) : IllegalStateException(message)

internal val nativeCallbackDepth: ThreadLocal<Int> = ThreadLocal.withInitial { 0 }
internal inline fun <R> withNativeCallback(body: () -> R): R {
    val prior = nativeCallbackDepth.get()
    nativeCallbackDepth.set(prior + 1)
    try { return body() } finally { nativeCallbackDepth.set(prior) }
}

// One order for all parameters, including selected constructor arguments.
// Generated constructors are internal: one Kotlin owner exists per allocation.
internal fun <R> withHandleLocks(handles: List<NativeHandle>, body: () -> R): R {
    val ordered = handles.distinct().sortedBy { it.address }
    var acquired = 0
    var reserved = false
    try {
        for (handle in ordered) {
            handle.lockForNativeAccess()
            acquired++
        }
        if (ordered.any { it.isClosed }) throw HandleValidationFailure("closed or consumed handle")
        if (ordered.any { it.activeCalls != 0 }) throw HandleValidationFailure("handle already in native use")
        for (handle in ordered) handle.activeCalls++
        reserved = true
        return body()
    } finally {
        if (reserved) for (handle in ordered) handle.activeCalls--
        for (index in acquired - 1 downTo 0) ordered[index].lock.unlock()
    }
}
