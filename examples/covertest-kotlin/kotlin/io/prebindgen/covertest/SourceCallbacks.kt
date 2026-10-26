package io.prebindgen.covertest

import io.prebindgen.covertest.callbacks.*
import java.lang.ref.WeakReference
import java.util.concurrent.atomic.AtomicInteger

// Native callbacks contain user exceptions; retain assertion failures so the
// JVM test fails after the upcall instead of merely logging a failed assertion.
private val callbackAssertionFailure = java.util.concurrent.atomic.AtomicReference<Throwable?>()
private fun verify(condition: Boolean) {
    if (!condition) {
        val failure = AssertionError("source callback assertion failed")
        callbackAssertionFailure.compareAndSet(null, failure)
        throw failure
    }
}
private fun expectCallbackRejection(action: () -> Unit) {
    val rejected = try { action(); false }
        catch (_: IllegalStateException) { true }
        catch (_: IllegalArgumentException) { true }
    verify(rejected)
}

private fun capturedCallback(): Pair<CallbackSet, WeakReference<Any>> {
    val marker = Any()
    val weak = WeakReference(marker)
    val callback = callbackSetNew({ _, _ -> marker.hashCode(); Unit }, {})
    return callback to weak
}

private fun requireCollected(weak: WeakReference<Any>) {
    repeat(100) {
        System.gc()
        if (weak.get() == null) return
        Thread.sleep(10)
    }
    error("native callback retained its JVM capture after close")
}

fun checkSourceCallbacks() {
    callbackScalars({ a, b -> verify(a == 17L && b == 29L) })
    callbackEnums({ value, optional ->
        verify(value == io.prebindgen.covertest.enums.Priority.High)
        verify(optional == io.prebindgen.covertest.enums.Priority.Low)
    })
    callbackPrimitives({ bool, char, byte, short, int, long, signed, unsigned, float, double ->
        verify(bool && char == 0x1F980)
        verify(byte == 255 && short == 65535 && int == 4294967295L && long == ULong.MAX_VALUE)
        verify(signed == java.math.BigInteger.ONE.shiftLeft(127).negate())
        verify(unsigned == java.math.BigInteger.ONE.shiftLeft(128).subtract(java.math.BigInteger.ONE))
        verify(float.toRawBits() == (-0.0f).toRawBits() && double == Double.POSITIVE_INFINITY)
    })
    callbackTexts({ text, bytes, numbers ->
        verify(text == "hello\u0000世界")
        verify(bytes.contentEquals(byteArrayOf(0, 127, -128, -1)))
        verify(numbers == listOf(1L, -2L, Long.MAX_VALUE))
    })
    val values = AtomicInteger()
    val empty = AtomicInteger()
    val mainThread = Thread.currentThread()
    val callbacks = callbackSetNew({ index, record ->
        verify(record.number == index + 10)
        verify(record.text == "héllo\u0000世界")
        values.incrementAndGet()
    }, { empty.incrementAndGet(); Unit })
    callbackSetFire(callbacks, 4)
    verify(values.get() == 4 && empty.get() == 4)
    val jobs = (0 until 4).map { callbackSetSpawn(callbacks, 1000) }
    callbacks.close()
    jobs.forEach { callbackJobJoin(it); verify(it.isClosed) }
    verify(values.get() == 4004 && empty.get() == 4004)

    var nativeThread = false
    val workerCallbacks = callbackSetNew({ _, _ -> nativeThread = Thread.currentThread() !== mainThread }, {})
    callbackJobJoin(callbackSetSpawn(workerCallbacks, 1))
    workerCallbacks.close()
    verify(nativeThread)

    val throwingCalls = AtomicInteger()
    val throwing = callbackSetNew({ i, _ ->
        throwingCalls.incrementAndGet()
        if (i == 1L) throw IllegalStateException("expected callback failure")
    }, {})
    callbackSetFire(throwing, 4)
    verify(throwingCalls.get() == 4)
    callbackJobJoin(callbackSetSpawn(throwing, 4))
    verify(throwingCalls.get() == 8)
    throwing.close()

    // A callback must not wait on a native borrower running on another thread.
    val entered = java.util.concurrent.CountDownLatch(1)
    val release = java.util.concurrent.CountDownLatch(1)
    val busy = callbackSetNew({ _, _ ->
        entered.countDown()
        verify(release.await(5, java.util.concurrent.TimeUnit.SECONDS))
    }, {})
    val borrower = Thread { callbackSetFire(busy, 1) }.also { it.start() }
    verify(entered.await(5, java.util.concurrent.TimeUnit.SECONDS))
    val ordinaryFinished = java.util.concurrent.CountDownLatch(1)
    val ordinaryFailure = java.util.concurrent.atomic.AtomicReference<Throwable?>()
    val ordinary = Thread {
        try { callbackSetFire(busy, 0) }
        catch (failure: Throwable) { ordinaryFailure.set(failure) }
        finally { ordinaryFinished.countDown() }
    }.also { it.start() }
    try {
        callbackOwnedFire({ _ ->
            val failure = try { callbackSetFire(busy, 0); null } catch (failure: Throwable) { failure }
            verify(failure is CallbackHandleBusyException)
        })
        verify(ordinaryFinished.count == 1L)
    } finally { release.countDown() }
    borrower.join(5000)
    ordinary.join(5000)
    verify(!borrower.isAlive && !ordinary.isAlive && ordinaryFailure.get() == null)
    busy.close()

    val (captured, weak) = capturedCallback()
    System.gc()
    verify(weak.get() != null)
    callbackSetFire(captured, 1)
    captured.close()
    requireCollected(weak)

    var escaped: CallbackOwned? = null
    val initial = callbackOwnedDrops()
    callbackOwnedFire({ value ->
        verify(callbackOwnedId(value) == 42L)
        escaped = value
    })
    verify(checkNotNull(escaped).isClosed)
    verify(callbackOwnedDrops() == initial + 1UL)
    var retained: CallbackOwned? = null
    callbackOwnedFire({ value -> retained = value.take() })
    verify(!checkNotNull(retained).isClosed)
    verify(callbackOwnedId(retained) == 42L)
    verify(callbackOwnedDrops() == initial + 1UL)
    retained!!.close()
    verify(callbackOwnedDrops() == initial + 2UL)

    callbackOptionalOwned({ value -> verify(value == null) }, false)
    callbackOptionalOwned({ value -> verify(callbackOwnedId(value) == 56L); escaped = value }, true)
    verify(checkNotNull(escaped).isClosed)
    callbackEnvelope({ record ->
        verify(record.label == "scoped record" && callbackOwnedId(record.handle) == 57L)
        escaped = record.handle
    })
    verify(checkNotNull(escaped).isClosed)
    val original = callbackOwnedNew(55)
    callbackOwnedBorrow(original, { value ->
        escaped = value
        verify(callbackOwnedId(value) == 55L)
    })
    verify(checkNotNull(escaped).isClosed && !original.isClosed)
    // Reentry cannot invalidate a Rust borrow or wait for a handle owned by its caller.
    callbackOwnedBorrow(original, { _ ->
        expectCallbackRejection { original.close() }
        expectCallbackRejection { original.take() }
        expectCallbackRejection { callbackOwnedId(original) }
        verify(!original.isClosed)
    })
    val beforeRejected = callbackOwnedDrops()
    try { callbackPreflight(original, null); error("expected null callback rejection") }
    catch (_: IllegalArgumentException) { }
    verify(!original.isClosed && callbackOwnedDrops() == beforeRejected)
    original.close()

    var first: CallbackTransferred? = null
    var second: CallbackTransferred? = null
    val beforeTransfer = callbackTransferredDrops()
    callbackTransferredFire({ a, b -> first = a; second = b }, true)
    verify(callbackTransferredId(first) == 7L && callbackTransferredId(second) == 8L)
    verify(callbackTransferredDrops() == beforeTransfer)
    first!!.close(); second!!.close()
    verify(callbackTransferredDrops() == beforeTransfer + 2UL)
    callbackTransferredFire({ a, b ->
        first = a; second = b
        throw IllegalStateException("expected exception after transfer")
    }, false)
    verify(second == null && !checkNotNull(first).isClosed)
    verify(callbackTransferredId(first) == 7L)
    first!!.close()
    verify(callbackTransferredDrops() == beforeTransfer + 3UL)
    var projected: CallbackProjected? = null
    callbackProjectionFire({ id, value -> verify(id == 9L); projected = value })
    verify(callbackProjectedId(projected) == 9L)
    projected!!.close()

    var pendingRan = false
    try {
        SourceCallbackRuntimeProbe.pending(object { fun run() { pendingRan = true } })
        error("expected original pending exception")
    } catch (failure: java.io.IOException) { verify(failure.message == "outer exception") }
    verify(!pendingRan)
    verify(SourceCallbackRuntimeProbe.rejected(object {
        @Suppress("UNUSED_PARAMETER")
        fun run(accepted: BooleanArray, pointer: Long) { throw IllegalStateException("before acceptance") }
    }) == 1L)
    verify(SourceCallbackRuntimeProbe.rejected(object {
        @Suppress("UNUSED_PARAMETER")
        fun run(accepted: BooleanArray, pointer: Long) { }
    }) == 1L)
    callbackAssertionFailure.get()?.let { throw it }
    println("source callbacks PASS: retained captures, native threads, typed arguments, ownership, reentry guards and exceptions")
}

private object SourceCallbackRuntimeProbe {
    @JvmStatic external fun pending(callback: Any)
    @JvmStatic external fun rejected(callback: Any): Long
}
