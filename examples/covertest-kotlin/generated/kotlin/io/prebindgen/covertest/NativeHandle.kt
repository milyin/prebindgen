package io.prebindgen.covertest

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

/** Distinguish generated wire validation from exceptions in user list implementations. */
internal object InputValidation {
    inline fun <T> value(body: () -> T): T = try { body() }
        catch (failure: IllegalArgumentException) { throw HandleValidationFailure(failure.message ?: "invalid input") }
    inline fun require(value: Boolean, message: () -> String) {
        if (!value) throw HandleValidationFailure(message())
    }
}

/** One-call input preparation, local to one generated invocation.
 * encode encodes under the handle locks; the generated invocation calls it once.
 * The cached references remain live for ownership-receipt handling afterwards.
 */
internal interface PreparedInput {
    val handles: List<NativeHandle>
    val owners: List<NativeHandle>
    fun encode(): Any
    fun consume(visit: (NativeHandle) -> Unit) { for (owner in owners) visit(owner) }
}

/** The row arrays are the actual JNI carrier, rather than per-row callback objects. */
internal class InputRows(private val rows: Array<Any?>, private val handleKinds: IntArray?) : PreparedInput {
    private var savedHandles: ArrayList<NativeHandle>? = null
    private var savedOwners: ArrayList<NativeHandle>? = null
    override val handles: List<NativeHandle> get() = savedHandles ?: emptyList()
    override val owners: List<NativeHandle> get() = savedOwners ?: emptyList()
    init {
        for (raw in rows) {
            val row = raw as Array<*>
            for (column in row.indices) {
                val value = row[column]
                if (value is NativeHandle) {
                    if (savedHandles == null) savedHandles = ArrayList()
                    savedHandles!!.add(value)
                    if (handleKinds?.get(column) == 2) {
                        if (savedOwners == null) savedOwners = ArrayList()
                        savedOwners!!.add(value)
                    }
                } else if (value is PreparedInput) {
                    if (value.handles.isNotEmpty()) {
                        if (savedHandles == null) savedHandles = ArrayList()
                        savedHandles!!.addAll(value.handles)
                    }
                    if (value.owners.isNotEmpty()) {
                        if (savedOwners == null) savedOwners = ArrayList()
                        savedOwners!!.addAll(value.owners)
                    }
                }
            }
        }
    }
    @Suppress("UNCHECKED_CAST")
    override fun encode(): Array<Any?> {
        for (raw in rows) {
            val row = raw as Array<Any?>
            for (column in row.indices) {
                val value = row[column]
                if ((handleKinds?.get(column) ?: 0) != 0) {
                    if (value is NativeHandle) row[column] = value.raw
                    else if (value == null) row[column] = 0L
                } else if (value is PreparedInput) row[column] = value.encode()
            }
        }
        return rows
    }
}
