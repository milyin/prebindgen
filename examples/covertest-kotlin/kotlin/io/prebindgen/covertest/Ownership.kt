package io.prebindgen.covertest

import java.util.concurrent.CountDownLatch
import java.util.concurrent.atomic.AtomicReference

private fun rejected(action: () -> Unit) {
    try { action() } catch (_: IllegalArgumentException) { return }
    error("expected input rejection")
}

fun checkOwnership() {
    val report = reportNew(1, 2.0, "original") { _, _, _, _, handle -> handle }
    rejected { reportRelabel(report, null) { _, _, _, _, handle -> handle.close() } }
    check(!report.isClosed)
    reportRelabel(report, "changed") { _, _, _, label, handle ->
        check(label == "changed" && report.isClosed)
        handle.close()
    }
    report.close()
    val a = storageWithPayload(42, 1, 2.5, false)
    val b = storageWithPayload(7, 2, 4.0, true)
    val c = io.prebindgen.covertest.storage.storageNew()
    check(storageLen(a) == 1L && storageContains(a, 42))
    storagePutByTake(c, 9, 3, 1.5, false)
    check(storageContains(c, 9) && !c.isClosed)
    // Repeated shared borrows are legal. All monitors use one ordering.
    check(storageTotalLen(a, a, a) == 3L)
    val failure = AtomicReference<Throwable?>()
    val start = CountDownLatch(1)
    val readers = listOf(listOf(a, b, c), listOf(c, b, a)).map { order ->
        Thread {
            try {
                start.await()
                repeat(2000) { check(storageTotalLen(order[0], order[1], order[2]) == 3L) }
            } catch (e: Throwable) { failure.set(e) }
        }.also { it.isDaemon = true; it.start() }
    }
    start.countDown()
    readers.forEach { it.join(10000); check(!it.isAlive) { "handle lock deadlock" } }
    failure.get()?.let { throw it }

    val nested: io.prebindgen.covertest.analytics.SummaryValue = storageSummaryHandle(a)
    rejected { reportLabel(1, 0, 0.0, nested, null) }
    check(!nested.isClosed)
    check(reportLabel(1, 0, 0.0, nested, "nested") == "nested")
    check(nested.isClosed)
    nested.close()
    val first = storageSummaryHandle(a)
    val second = storageSummaryHandle(b)
    // The second selector/null input fails before the first object is consumed.
    rejected { summaryMerge(1, 0, 0.0, first, 2, 0, 0.0, second) { _, _ -> } }
    check(!first.isClosed && !second.isClosed)
    rejected { summaryMerge(1, 0, 0.0, first, 1, 0, 0.0, null) { _, _ -> } }
    check(!first.isClosed)
    rejected { summaryMerge(1, 0, 0.0, first, 1, 0, 0.0, first) { _, _ -> } }
    check(!first.isClosed)
    summaryMerge(1, 0, 0.0, first, 1, 0, 0.0, second) { count, total ->
        check(count == 2L && total == 6.5)
        check(first.isClosed && second.isClosed)
    }
    first.close(); second.close()

    // A native panic after transfer still marks the selected input consumed.
    val doomed = storageSummaryHandle(a)
    rejected { summaryMerge(1, 0, 0.0, doomed, 0, Long.MAX_VALUE, 0.0, null) { _, _ -> } }
    check(doomed.isClosed)
    doomed.close()

    // close races against a borrow and a consuming call; exactly one can own it.
    repeat(100) {
        val owner = storageSummaryHandle(a)
        val gate = CountDownLatch(1)
        val taker = Thread {
            try {
                gate.await()
                try { summaryTotalRaw(1, 0, 0.0, owner) }
                catch (_: IllegalStateException) { }
            } catch (e: Throwable) { failure.set(e) }
        }
        val closer = Thread { gate.await(); owner.close() }
        taker.start(); closer.start(); gate.countDown()
        taker.join(); closer.join()
        check(owner.isClosed)
    }
    failure.get()?.let { throw it }
    val borrowed = storageWithPayload(1, 0, 0.0, false)
    val reader = Thread {
        try { repeat(2000) { try { storageLen(borrowed) } catch (_: IllegalStateException) { } } }
        catch (e: Throwable) { failure.set(e) }
    }
    reader.start(); borrowed.close(); reader.join()
    failure.get()?.let { throw it }
    a.close(); b.close(); c.close()
}
