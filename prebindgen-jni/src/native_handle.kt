package PACKAGE

/** Owns one boxed Rust allocation. The low bit records close or consumption. */
public abstract class NativeHandle internal constructor(raw: Long) : AutoCloseable {
    @Volatile internal var raw: Long = raw
        private set
    internal val address: Long = raw
    internal val lock = java.util.concurrent.locks.ReentrantLock()
    @Volatile internal var activeCalls: Int = 0
        private set
    private var activeExclusive: Boolean = false
    internal fun sharedNativeAccess(): SharedNativeAccess = SharedNativeAccess(this)
    public val isClosed: Boolean get() = raw == 0L || raw and 1L != 0L
    internal fun markConsumed() { raw = raw or 1L }
    // A synchronous callback must not close/move a value borrowed by its caller,
    // or block on a handle whose owner may be waiting for that callback.
    internal fun lockForNativeAccess(shared: Boolean = false) {
        if (nativeCallbackDepth.get() > 0) {
            if (lock.isHeldByCurrentThread && activeCalls != 0 && (!shared || activeExclusive)) {
                throw HandleValidationFailure("callback cannot reenter a handle already in native use")
            }
            if (lock.isHeldByCurrentThread) {
                lock.lock()
                return
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
    internal fun beginNativeCall(shared: Boolean) {
        if (isClosed) throw HandleValidationFailure("closed or consumed handle")
        if (activeCalls != 0 && (!shared || activeExclusive)) {
            throw HandleValidationFailure("handle already in incompatible native use")
        }
        activeCalls++
        if (!shared) activeExclusive = true
    }
    internal fun endNativeCall() {
        activeCalls--
        if (activeCalls == 0) activeExclusive = false
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

/** A shared Rust borrow; plain NativeHandle requests remain exclusive. */
internal class SharedNativeAccess(val handle: NativeHandle)

private fun nativeAccessHandle(access: Any): NativeHandle = when (access) {
    is SharedNativeAccess -> access.handle
    is NativeHandle -> access
    else -> throw HandleValidationFailure("invalid native handle access")
}

private inline fun <R> withOneHandleLock(access: Any, body: () -> R): R {
    val handle = nativeAccessHandle(access)
    val shared = access is SharedNativeAccess
    handle.lockForNativeAccess(shared)
    var reserved = false
    try {
        handle.beginNativeCall(shared)
        reserved = true
        return body()
    } finally {
        if (reserved) handle.endNativeCall()
        handle.lock.unlock()
    }
}

// Merge access modes before locking: any move or mutable borrow is exclusive.
// All requests are still serialized between threads, including Send-only types.
internal fun <R> withHandleLocks(handles: List<Any>, body: () -> R): R {
    if (handles.isEmpty()) return body()
    if (handles.size == 1) return withOneHandleLock(handles[0], body)
    val shared = java.util.IdentityHashMap<NativeHandle, Boolean>(handles.size)
    for (access in handles) {
        val handle = nativeAccessHandle(access)
        // Reject consumed owners before sorting aliases sharing the old address.
        if (handle.isClosed) throw HandleValidationFailure("closed or consumed handle")
        val read = access is SharedNativeAccess
        shared[handle] = shared[handle]?.let { it && read } ?: read
    }
    val ordered = shared.keys.sortedBy { it.address }
    var acquired = 0
    var reserved = 0
    try {
        for (handle in ordered) {
            handle.lockForNativeAccess(shared[handle]!!)
            acquired++
        }
        for (handle in ordered) {
            handle.beginNativeCall(shared[handle]!!)
            reserved++
        }
        return body()
    } finally {
        for (index in reserved - 1 downTo 0) ordered[index].endNativeCall()
        for (index in acquired - 1 downTo 0) ordered[index].lock.unlock()
    }
}
