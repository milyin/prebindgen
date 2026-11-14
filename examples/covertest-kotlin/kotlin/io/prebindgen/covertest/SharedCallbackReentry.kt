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
    val value = callbackReentryNew(10)
    val other = callbackReentryNew(20)
    var calls = 0
    check(callbackReentryShared(value, { inside {
        calls++
        check(callbackReentryRead(value) == 10L)
        check(callbackReentryShared(value, { inside {
            calls++
            check(callbackReentryRead(value) == 10L)
        } }) == 10L)
    } }) == 10L)
    check(calls == 2)
    check(callbackReentryPair(value, value, { inside {
        calls++
        check(callbackReentryRead(value) == 10L)
    } }) == 20L)
    // The callback may read the shared input, but must respect the other &mut borrow.
    check(callbackReentryMixed(value, other, { inside {
        calls++
        check(callbackReentryRead(value) == 10L)
    } }) == 31L)
    check(calls == 4 && callbackReentryRead(other) == 21L)
    val aliasFailure = try { callbackReentryMixed(value, value, { calls++ }); null }
        catch (t: IllegalArgumentException) { t }
    check(aliasFailure != null && calls == 4)
    // Exclusive and consuming calls can safely use a different handle in callbacks.
    check(callbackReentryExclusive(value, { inside {
        calls++
        check(callbackReentryRead(other) == 21L)
    } }) == 11L)
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
    var throwCalls = 0
    check(callbackReentryShared(value, {
        inside { throwCalls++; check(callbackReentryRead(value) == 11L) }
        throw IllegalStateException("expected reentrant callback exception")
    }) == 11L)
    check(throwCalls == 1)
    callbackReentryWrite(value, 12)
    check(callbackReentryRead(value) == 12L)

    val workerCalls = AtomicInteger()
    check(callbackReentryWorker(value, { inside {
        check(callbackReentryRead(other) == 21L)
        workerCalls.incrementAndGet()
    } }) == 12L)
    check(workerCalls.get() == 1)
    // Shared borrows still serialize across threads for Send-but-not-Sync types.
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
        check(!finished.await(150, TimeUnit.MILLISECONDS))
    } finally { release.countDown() }
    owner.join(5000); reader.join(5000)
    check(!owner.isAlive && !reader.isAlive && finished.count == 0L)
    val moved = callbackReentryNew(33)
    check(callbackReentryConsuming(moved, { inside {
        calls++
        check(callbackReentryRead(value) == 12L)
    } }) == 33L)
    check(calls == 9 && moved.isClosed)
    value.close(); other.close()
    failure.get()?.let { throw it }
    println("shared callback reentry PASS: monitor reentry, aliases, Send-only serialization")
}
