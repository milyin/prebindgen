package io.prebindgen.covertest

import io.prebindgen.covertest.gc.*
import java.lang.ref.WeakReference
import java.util.concurrent.CountDownLatch
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference

private fun awaitGcDrops(id: ULong, count: ULong, wrappers: List<WeakReference<GcResource>>) {
    val deadline = System.nanoTime() + 15_000_000_000L
    while (gcResourceDrops(id) != count && System.nanoTime() < deadline) {
        System.gc()
        Thread.sleep(10)
    }
    check(gcResourceDrops(id) == count) { "id=$id: expected $count drops, got ${gcResourceDrops(id)}" }
    check(wrappers.all { it.get() == null }) { "Cleaner retained a wrapper" }
}

// Keep wrappers out of the caller's frame while testing actual GC reachability.
private fun abandonedGcResources(): List<WeakReference<GcResource>> {
    val refs = ArrayList<WeakReference<GcResource>>()
    refs.add(WeakReference(gcResourceNew(10uL, 42uL)))
    gcResourceEmitOwned(11uL, 43uL, { refs.add(WeakReference(it)) })
    for (value in gcResourceRows(12uL, 16uL)) refs.add(WeakReference(value))
    return refs
}

private fun settledGcResources(): List<WeakReference<GcResource>> {
    val closed = gcResourceNew(13uL, 1uL)
    closed.close()
    val consumed = gcResourceNew(13uL, 2uL)
    check(gcResourceConsume(consumed) == 2uL)
    val transferred = gcResourceNew(13uL, 3uL)
    val next = transferred.take()
    next.close()
    return listOf(closed, consumed, transferred, next).map { WeakReference(it) }
}

private fun temporaryBorrowedRows(): ULong =
    gcResourceReadRows(List(256) { gcResourceNew(20uL, it.toULong()) })

fun checkGcHandles() {
    val a = gcResourceNew(0uL, 42uL)
    check(gcResourceValue(a) == 42uL)
    a.close()
    a.close()
    check(a.isClosed && gcResourceDrops(0uL) == 1uL)
    try { a.take(); error("closed take accepted") } catch (_: IllegalStateException) { }

    val old = gcResourceNew(1uL, 43uL)
    val owner = old.take()
    check(old.isClosed && !owner.isClosed && gcResourceDrops(1uL) == 0uL)
    old.close()
    check(gcResourceValue(owner) == 43uL)
    owner.close()
    check(gcResourceDrops(1uL) == 1uL)

    val consumed = gcResourceNew(2uL, 44uL)
    check(gcResourceConsume(consumed) == 44uL)
    check(consumed.isClosed && gcResourceDrops(2uL) == 1uL)
    consumed.close()

    val failure = gcResourceNew(3uL, 45uL)
    check(gcResourceFail(failure, { error(it ?: "binding error") }) { it.toULong() } == 17uL)
    check(failure.isClosed && gcResourceDrops(3uL) == 1uL)
    failure.close()

    val errorOwner = gcResourceNew(14uL, 45uL)
    val handlerFailure = IllegalStateException("expected domain handler failure")
    try {
        gcResourceFail(errorOwner, { error(it ?: "binding error") }) { throw handlerFailure }
        error("handler failure missing")
    } catch (caught: IllegalStateException) { check(caught === handlerFailure) }
    check(errorOwner.isClosed && gcResourceDrops(14uL) == 1uL)
    errorOwner.close()

    val duplicate = gcResourceNew(4uL, 46uL)
    try { gcResourcePair(duplicate, duplicate); error("duplicate move accepted") }
    catch (_: IllegalArgumentException) { }
    // Ordinary arguments may consume a validated prefix before a later alias
    // fails. Its receipt must cancel cleanup on this shared wrapper exactly once.
    check(duplicate.isClosed && gcResourceDrops(4uL) == 1uL)
    duplicate.close()

    val shared = gcResourceNew(15uL, 4uL)
    check(gcResourceThree(shared, shared, shared) == 12uL)
    shared.close()
    val inputs = gcResourceRows(16uL, 4uL)
    check(gcResourceConsumeRows(inputs) == 6uL)
    check(inputs.all { it.isClosed } && gcResourceDrops(16uL) == 4uL)
    inputs.forEach { it.close() }
    check(temporaryBorrowedRows() == 32640uL)
    awaitGcDrops(20uL, 256uL, emptyList())

    // Previously accepted rows stay caller-owned on fold failure. The current
    // and unread suffix are reclaimed, disarming every constructed wrapper.
    var firstRow: GcResource? = null
    val foldFailure = IllegalStateException("expected fold failure")
    try {
        gcResourceRowsFold(17uL, 4uL, 0) { index, row ->
            if (index == 0) { firstRow = row; 1 } else throw foldFailure
        }
        error("fold failure missing")
    } catch (caught: IllegalStateException) { check(caught === foldFailure) }
    check(gcResourceDrops(17uL) == 3uL)
    check(gcResourceValue(checkNotNull(firstRow)) == 0uL)
    firstRow!!.close()
    check(gcResourceDrops(17uL) == 4uL)

    // Simulate allocation failure while registering a constructed collection
    // owner. The raw native field must remain the only owner after own() fails.
    val unregistered = gcResourceNew(18uL, 50uL)
    val nativeField = unregistered.raw
    val journal = CollectionOwners()
    val registrationFailure = OutOfMemoryError("expected owner registration failure")
    val ownersField = CollectionOwners::class.java.getDeclaredField("owners")
    ownersField.isAccessible = true
    ownersField.set(journal, object : ArrayList<NativeHandle>() {
        override fun add(element: NativeHandle): Boolean { throw registrationFailure }
    })
    try { journal.own(unregistered); error("registration failure missing") }
    catch (caught: OutOfMemoryError) { check(caught === registrationFailure) }
    check(unregistered.isClosed && gcResourceDrops(18uL) == 0uL)
    GcResource(nativeField).close()
    check(gcResourceDrops(18uL) == 1uL)

    var delivered: GcResource? = null
    gcResourceEmit(5uL, 47uL, { delivered = it.take() })
    check(gcResourceValue(checkNotNull(delivered)) == 47uL)
    delivered!!.close()
    check(gcResourceDrops(5uL) == 1uL)
    var returned: GcResource? = null
    check(gcResourceReturn({ gcResourceNew(21uL, 51uL).also { returned = it } }) == 51uL)
    check(checkNotNull(returned).isClosed && gcResourceDrops(21uL) == 1uL)
    returned!!.close()

    // The source wrapper is only a temporary; it must stay alive across JNI,
    // including a callback that triggers GC before Rust reads the borrow again.
    val callbackFailure = AtomicReference<Throwable?>()
    val warm = gcResourceNew(19uL, 48uL)
    repeat(10_000) { check(gcResourceHold(warm, {}) == 48uL) }
    warm.close()
    check(gcResourceHold(gcResourceNew(6uL, 48uL), {
        try {
            repeat(3) { System.gc(); Thread.sleep(10) }
            check(gcResourceDrops(6uL) == 0uL)
        } catch (failure: Throwable) { callbackFailure.set(failure) }
    }) == 48uL)
    callbackFailure.get()?.let { throw it }

    val racing = gcResourceNew(7uL, 49uL)
    val start = CountDownLatch(1)
    val threads = List(8) { Thread { start.await(); repeat(100) { racing.close() } }.also { it.start() } }
    start.countDown()
    threads.forEach { it.join() }
    check(racing.isClosed && gcResourceDrops(7uL) == 1uL)

    // A separate action never captures the handle, and even concurrent attempts
    // at executing that action claim its pointer exactly once.
    val releases = AtomicInteger()
    val action = NativeCleaner.Release(2L) { check(it == 2L); releases.incrementAndGet() }
    val cleaners = List(8) { Thread { repeat(100) { action.run() } }.also { it.start() } }
    cleaners.forEach { it.join() }
    check(releases.get() == 1)
    val disarmed = NativeCleaner.Release(2L) { error("consumed pointer released") }
    disarmed.disarm()
    disarmed.run()
    for (raw in listOf(0L, 1L, 3L)) NativeCleaner.Release(raw) { error("closed pointer released") }.run()

    val abandoned = abandonedGcResources()
    awaitGcDrops(10uL, 1uL, abandoned.take(1))
    awaitGcDrops(11uL, 1uL, abandoned.drop(1).take(1))
    awaitGcDrops(12uL, 16uL, abandoned.drop(2))
    val settled = settledGcResources()
    val deadline = System.nanoTime() + 15_000_000_000L
    while (settled.any { it.get() != null } && System.nanoTime() < deadline) { System.gc(); Thread.sleep(10) }
    check(settled.all { it.get() == null })
    repeat(3) { System.gc(); Thread.sleep(10) }
    check(gcResourceDrops(13uL) == 3uL)
    println("gc-managed PASS: close, take, consumption, failures, callbacks, collections, in-flight borrow and abandoned cleanup")
}
