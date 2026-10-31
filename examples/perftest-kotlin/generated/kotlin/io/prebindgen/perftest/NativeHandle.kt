package io.prebindgen.perftest

/** Owns one boxed Rust allocation. The low bit records close or consumption. */
public abstract class NativeHandle internal constructor(raw: Long) : AutoCloseable {
    @Volatile internal var raw: Long = raw
        private set
    internal val address: Long = raw
    public val isClosed: Boolean get() = raw == 0L || raw and 1L != 0L
    internal fun markConsumed() { raw = raw or 1L }
    protected abstract fun freeNative(raw: Long)
    final override fun close() = synchronized(this) {
        if (!isClosed) {
            val owned = raw
            markConsumed()
            freeNative(owned)
        }
    }
}

internal class HandleValidationFailure(message: String) : IllegalStateException(message)

// Monitors serialize threads and allow same-thread reentry. Callers
// must respect Rust borrows and must not close or move a value still in use.
internal inline fun <R> withHandleLocks(handle: NativeHandle?, body: () -> R): R =
    if (handle == null) body() else synchronized(handle) { body() }

internal inline fun <R> withHandleLocks(a: NativeHandle?, b: NativeHandle?, body: () -> R): R {
    if (a == null) return withHandleLocks(b, body)
    if (b == null) return withHandleLocks(a, body)
    val first = if (a.address <= b.address) a else b
    val second = if (first === a) b else a
    return synchronized(first) { synchronized(second) { body() } }
}

internal inline fun <R> withHandleLocks(a: NativeHandle?, b: NativeHandle?, c: NativeHandle?, body: () -> R): R {
    if (a == null) return withHandleLocks(b, c, body)
    if (b == null) return withHandleLocks(a, c, body)
    if (c == null) return withHandleLocks(a, b, body)
    var x = a
    var y = b
    var z = c
    if (x.address > y.address) { val t = x; x = y; y = t }
    if (y.address > z.address) { val t = y; y = z; z = t }
    if (x.address > y.address) { val t = x; x = y; y = t }
    return synchronized(x) { synchronized(y) { synchronized(z) { body() } } }
}

internal fun <R> withHandleLocks(handles: List<NativeHandle>, body: () -> R): R {
    when (handles.size) {
        0 -> return body()
        1 -> return withHandleLocks(handles[0], body)
        2 -> return withHandleLocks(handles[0], handles[1], body)
        3 -> return withHandleLocks(handles[0], handles[1], handles[2], body)
    }
    val ordered = handles.sortedBy { it.address }
    fun acquire(index: Int): R = if (index == ordered.size) body()
        else synchronized(ordered[index]) { acquire(index + 1) }
    return acquire(0)
}
