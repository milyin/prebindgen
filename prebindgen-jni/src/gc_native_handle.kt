package PACKAGE

import java.lang.ref.Cleaner
import java.lang.ref.Reference
import java.util.concurrent.atomic.AtomicLong

/**
 * Opt-in cleanup for abandoned native handles. Pointer reads and handle locks
 * remain in NativeHandle; only ownership release touches the separate state.
 * Explicit close is preferred: Cleaner execution is asynchronous.
 */
public abstract class GcNativeHandle internal constructor(raw: Long, free: (Long) -> Unit) : NativeHandle(raw) {
    // A separate state is essential: the registered action must never reference
    // this wrapper, otherwise the Cleaner would keep its own referent alive.
    private val release = NativeCleaner.Release(raw, free)
    private val cleanable = if (isClosed) null else NativeCleaner.cleaner.register(this, release)

    internal final override fun markConsumed() {
        super.markConsumed()
        release.disarm()
        cleanable?.clean()
    }
}

internal object NativeCleaner {
    val cleaner: Cleaner = Cleaner.create()

    internal class Release(raw: Long, private val free: (Long) -> Unit) : Runnable {
        private val pointer = AtomicLong(if (raw == 0L || raw and 1L != 0L) 0L else raw)
        fun disarm() { pointer.getAndSet(0L) }
        override fun run() {
            val owned = pointer.getAndSet(0L)
            if (owned != 0L) free(owned)
        }
    }
}

// Keep opted-in wrappers reachable until native access has finished, including
// temporary arguments and exceptional exits. These are JVM reachability fences,
// not additional locks or pointer validation. Plain-only calls use the original
// withHandleLocks overloads and do not reference this Java 9 API.
internal inline fun <R> withGcHandleLocks(handle: NativeHandle?, body: () -> R): R =
    try { withHandleLocks(handle, body) } finally { Reference.reachabilityFence(handle) }

internal inline fun <R> withGcHandleLocks(a: NativeHandle?, b: NativeHandle?, body: () -> R): R =
    try { withHandleLocks(a, b, body) } finally {
        Reference.reachabilityFence(a)
        Reference.reachabilityFence(b)
    }

internal inline fun <R> withGcHandleLocks(a: NativeHandle?, b: NativeHandle?, c: NativeHandle?, body: () -> R): R =
    try { withHandleLocks(a, b, c, body) } finally {
        Reference.reachabilityFence(a)
        Reference.reachabilityFence(b)
        Reference.reachabilityFence(c)
    }

internal fun <R> withGcHandleLocks(handles: List<NativeHandle>, body: () -> R): R =
    try { withHandleLocks(handles, body) } finally { Reference.reachabilityFence(handles) }
