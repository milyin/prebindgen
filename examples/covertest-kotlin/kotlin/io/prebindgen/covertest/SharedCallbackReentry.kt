package io.prebindgen.covertest

import io.prebindgen.covertest.callbacks.*
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference

fun checkSharedCallbackReentry() {
    val failure = AtomicReference<Throwable?>()
    fun inside(body: () -> Unit) {
        try { body() } catch (t: Throwable) { failure.compareAndSet(null, t) }
    }
    fun rejected(body: () -> Unit) {
        val t = try { body(); null } catch (t: Throwable) { t }
        check(t is HandleValidationFailure || t is IllegalArgumentException) { "expected rejected native access: $t" }
    }
    val value = callbackReentryNew(10)
    val other = callbackReentryNew(20)
    var calls = 0
    check(callbackReentryShared(value, { inside {
        calls++
        check(callbackReentryRead(value) == 10L)
        check(callbackReentryShared(value, { inside {
            calls++
            check(callbackReentryRead(value) == 10L)
            rejected { callbackReentryWrite(value, 99) }
            rejected { callbackReentryTake(value) }
            rejected { value.close() }
            rejected { value.take() }
        } }) == 10L)
    } }) == 10L)
    check(calls == 2)
    check(callbackReentryPair(value, value, { inside {
        calls++
        check(callbackReentryRead(value) == 10L)
    } }) == 20L)
    check(calls == 3)
    check(callbackReentryMixed(value, other, { inside {
        calls++
        check(callbackReentryRead(value) == 10L)
        rejected { callbackReentryRead(other) }
        rejected { callbackReentryWrite(other, 77) }
    } }) == 31L)
    check(calls == 4 && callbackReentryRead(other) == 21L)
    rejected { callbackReentryMixed(value, value, { calls++ }) }
    check(calls == 4 && callbackReentryRead(value) == 10L)
    check(callbackReentryExclusive(value, { inside {
        calls++
        rejected { callbackReentryRead(value) }
        rejected { callbackReentryWrite(value, 77) }
        rejected { value.close() }
        rejected { value.take() }
    } }) == 11L)
    check(calls == 5)
    check(callbackReentryOptional(value, { inside { calls++; check(callbackReentryRead(value) == 11L) } }) == 11L)
    check(callbackReentryOptional(null, { inside { calls++; check(callbackReentryRead(value) == 11L) } }) == -1L)
    check(callbackReentryConstructed(value, { inside { calls++; check(callbackReentryRead(value) == 11L) } }) == 11L)
    check(calls == 8)

    var interruptedCalls = 0
    check(callbackReentryShared(value, { inside {
        val thread = Thread.currentThread()
        thread.interrupt()
        try {
            check(callbackReentryRead(value) == 11L)
            check(thread.isInterrupted)
            interruptedCalls++
        } finally { Thread.interrupted() }
    } }) == 11L)
    check(interruptedCalls == 1)

    // Legacy/custom lock expressions containing plain handles stay exclusive.
    withHandleLocks(listOf(value)) { withNativeCallback {
        rejected { callbackReentryRead(value) }
    } }
    // Any exclusive request dominates repeated shared requests for the same handle.
    withHandleLocks(listOf(value.sharedNativeAccess(), value)) { withNativeCallback {
        rejected { callbackReentryRead(value) }
    } }
    // Failure after reserving an earlier handle must restore its counters.
    val early = if (value.address < other.address) value else other
    val late = if (early === value) other else value
    withHandleLocks(listOf(late)) {
        rejected { callbackReentryPair(early, late, {}) }
    }
    check(callbackReentryRead(value) == 11L && callbackReentryRead(other) == 21L)
    var throwCalls = 0
    check(callbackReentryShared(value, {
        inside { throwCalls++; check(callbackReentryRead(value) == 11L) }
        throw IllegalStateException("expected reentrant callback exception")
    }) == 11L)
    check(throwCalls == 1)
    callbackReentryWrite(value, 12)
    check(callbackReentryRead(value) == 12L)

    // A native worker cannot share this Send-but-not-Sync value concurrently.
    val workerCalls = AtomicInteger()
    check(callbackReentryWorker(value, { inside {
        val t = try { callbackReentryProbe(value); null } catch (t: Throwable) { t }
        check(t is CallbackHandleBusyException)
        workerCalls.incrementAndGet()
    } }) == 12L)
    check(calls == 8 && workerCalls.get() == 1)
    val entered = CountDownLatch(1)
    val release = CountDownLatch(1)
    val finished = CountDownLatch(1)
    val attempted = CountDownLatch(1)
    val owner = Thread {
        inside { callbackReentryShared(value, { inside {
            entered.countDown()
            check(release.await(5, TimeUnit.SECONDS))
        } }) }
    }.also { it.start() }
    check(entered.await(5, TimeUnit.SECONDS))
    val reader = Thread {
        attempted.countDown()
        try { inside { check(callbackReentryProbe(value) == 1L) } }
        finally { finished.countDown() }
    }.also { it.start() }
    try {
        check(attempted.await(5, TimeUnit.SECONDS))
        check(!finished.await(100, TimeUnit.MILLISECONDS))
    } finally { release.countDown() }
    owner.join(5000); reader.join(5000)
    check(!owner.isAlive && !reader.isAlive && finished.count == 0L)
    val moved = callbackReentryNew(33)
    check(callbackReentryConsuming(moved, { inside {
        calls++
        rejected { callbackReentryRead(moved) }
        rejected { moved.close() }
        rejected { moved.take() }
    } }) == 33L)
    check(calls == 9 && moved.isClosed)
    value.close(); other.close()
    failure.get()?.let { throw it }
    println("shared callback reentry PASS: shared reads, exclusive/move guards, aliases, Send-only serialization")
}
