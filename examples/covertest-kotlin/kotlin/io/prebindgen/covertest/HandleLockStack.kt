package io.prebindgen.covertest

import io.prebindgen.covertest.callbackreturns.*
import io.prebindgen.covertest.gc.*
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference

// Monitor-only probes never pass these synthetic addresses to JNI.
private class MonitorProbe(address: Long, private val drops: AtomicInteger) : NativeHandle(address) {
    override val handleType = 1L
    override fun freeNative(raw: Long) { drops.incrementAndGet() }
}
private fun awaitLockState(thread: Thread, state: Thread.State, failure: AtomicReference<Throwable?>) {
    val deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(10)
    while (thread.state != state && thread.isAlive && System.nanoTime() < deadline) Thread.yield()
    failure.get()?.let { throw it }
    check(thread.state == state) { "${thread.name}: expected $state, got ${thread.state}" }
}
private fun awaitLockLatch(latch: CountDownLatch) { check(latch.await(10, TimeUnit.SECONDS)) }
private fun finishLockThread(thread: Thread) { thread.join(10000); check(!thread.isAlive) { "lock thread did not finish" } }

private fun checkMonitorGroups() {
    val drops = AtomicInteger()
    for (size in listOf(0, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 8192)) {
        val handles = List(size) { MonitorProbe((it + 1) * 2L, drops) }
        val expected = Any()
        var calls = 0
        check(withOrderedHandleLocks(handles) {
            calls++; for (handle in handles) check(Thread.holdsLock(handle)); expected
        } === expected && calls == 1)
        val failure = IllegalStateException("expected body failure")
        try {
            withHandleLocks(handles.reversed()) { throw failure }
            error("body failure missing")
        } catch (caught: IllegalStateException) { check(caught === failure) }
        for (handle in handles) check(!Thread.holdsLock(handle))
        if (size <= 33) withOrderedHandleLocks(handles) {
            withOrderedHandleLocks(handles) { for (handle in handles) check(Thread.holdsLock(handle)) }
        }
        if (size >= 5) {
            for (fault in setOf(handles.size / 2, handles.size - 1)) {
                val broken = object : AbstractList<NativeHandle>() {
                    override val size = handles.size
                    override fun get(index: Int): NativeHandle = if (index == fault) throw failure else handles[index]
                }
                try { withOrderedHandleLocks(broken) { error("partially acquired body called") }; error("get failure missing") }
                catch (caught: IllegalStateException) { check(caught === failure) }
                for (handle in handles) check(!Thread.holdsLock(handle))
            }
        }
    }
}

private fun checkLargeConcurrentMonitors() {
    val drops = AtomicInteger()
    val handles = List(8192) { MonitorProbe((it + 1) * 2L, drops) }
    val blockedMonitor = handles[handles.size / 2]
    val held = CountDownLatch(1); val releaseMiddle = CountDownLatch(1)
    val entered = CountDownLatch(1); val releaseBody = CountDownLatch(1)
    val failure = AtomicReference<Throwable?>()
    val expected = IllegalStateException("expected concurrent body failure")
    fun thread(name: String, body: () -> Unit) = Thread({ try { body() } catch (e: Throwable) { failure.compareAndSet(null, e) } }, name).also { it.isDaemon = true; it.start() }
    val holder = thread("middle monitor holder") { synchronized(blockedMonitor) { held.countDown(); awaitLockLatch(releaseMiddle) } }
    awaitLockLatch(held)
    val reader = thread("grouped monitor reader") {
        try { withOrderedHandleLocks(handles) {
            check(handles.all { Thread.holdsLock(it) && !it.isClosed })
            entered.countDown(); awaitLockLatch(releaseBody); throw expected
        } } catch (caught: IllegalStateException) { check(caught === expected) }
    }
    var closer: Thread? = null
    try {
        awaitLockState(reader, Thread.State.BLOCKED, failure)
        closer = thread("prefix handle closer") { handles[0].close() }
        awaitLockState(closer, Thread.State.BLOCKED, failure)
        check(drops.get() == 0)
        releaseMiddle.countDown(); awaitLockLatch(entered)
        check(drops.get() == 0 && !handles[0].isClosed)
    } finally { releaseMiddle.countDown(); releaseBody.countDown() }
    finishLockThread(holder); finishLockThread(reader); closer?.let(::finishLockThread)
    failure.get()?.let { throw it }
    check(drops.get() == 1 && handles[0].isClosed)
    withOrderedHandleLocks(handles) { check(handles.all { Thread.holdsLock(it) }) }
    val start = CountDownLatch(1)
    val completed = AtomicInteger()
    val readers = listOf(handles, handles.reversed()).mapIndexed { index, order -> thread("sorted large reader $index") {
        awaitLockLatch(start)
        repeat(20) { withHandleLocks(order) { completed.incrementAndGet() } }
    } }
    start.countDown(); readers.forEach(::finishLockThread)
    failure.get()?.let { throw it }
    check(completed.get() == 40)
}

// Run in its own fresh JVM with -Xss1m: no earlier test may warm the helpers.
fun checkHandleLockStack() {
    // First acquisition is an actual JNI borrow of 8,192 Cleaner-managed owners.
    val count = 8192
    val before = gcResourceDrops(90uL)
    val resources = List(count) { gcResourceNew(90uL, it.toULong()) }
    val sum = count.toULong() * (count - 1).toULong() / 2uL
    check(gcResourceReadRows(resources) == sum)
    check(resources.all { !it.isClosed } && gcResourceDrops(90uL) == before)
    check(gcResourceConsumeRows(resources) == sum)
    check(resources.all { it.isClosed } && gcResourceDrops(90uL) == before + count.toULong())

    callbackReturnResourceAuditStart()
    // Attached Rust workers retain their native stack size; -Xss bounds JVM-created threads.
    for (worker in listOf(false, true)) for (fail in listOf(false, true)) {
        var calls = 0
        val total = callbackFixedResults(FixedResultsCallback {
            check(callbackReturnResourceLive() == 0UL)
            val bad = calls++ == 0 && fail
            List(4096) { index -> CallbackFixedResult(callbackReturnResourceNew(10), null,
                if (bad && index == 2048) 'p'.code else 'c'.code, callbackReturnResourceNew(30)) }
        }, fail, worker)
        check(total == 4096L * 40 && calls == if (fail) 2 else 1)
        check(callbackReturnResourceLive() == 0UL)
    }
    callbackReturnResourceAuditFinish()
    checkMonitorGroups()
    checkLargeConcurrentMonitors()
    println("handle lock stack PASS: cold JNI borrow/move, callback results, workers, tails, reentry, partial acquisition and concurrent close at 8192 handles")
}
