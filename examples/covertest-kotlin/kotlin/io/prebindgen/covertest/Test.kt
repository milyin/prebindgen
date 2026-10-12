package io.prebindgen.covertest

import io.prebindgen.covertest.model.Report
import io.prebindgen.covertest.model.blobValueNew
import io.prebindgen.covertest.model.blobValueEcho

private fun expectInvalid(action: () -> Unit) {
    try { action() }
    catch (_: IllegalArgumentException) { return }
    catch (_: IllegalStateException) { return }
    error("expected conversion failure")
}

// Exercises the active subset of the migrated source and build declarations.
fun main() {
    for (text in listOf("", "ordinary", "héllo 世界", "embedded\u0000nul")) {
        check(stringNew(text) == text)
    }
    try { stringNew(null); error("expected null input failure") }
    catch (_: IllegalArgumentException) { }
    val storage = io.prebindgen.covertest.storage.storageNew()
    check(!storage.isClosed)
    val workers = (0 until 8).map { Thread { storage.close() }.also { it.start() } }
    workers.forEach { it.join() }
    check(storage.isClosed)
    storage.close()
    val archive = io.prebindgen.covertest.analytics.archiveNew()
    check(!archive.isClosed)
    archive.close()
    check(archive.isClosed)
    for (bytes in listOf(byteArrayOf(), byteArrayOf(0, 127, -128, -1))) {
        val blob = blobValueNew(Long.MIN_VALUE, bytes)
        val echoed = blobValueEcho(blob)
        check(blob.isClosed)
        check(!echoed.isClosed)
        expectInvalid { blobValueEcho(blob) }
        echoed.close()
        blob.close()
    }
    expectInvalid { blobValueNew(0, null) }
    for (label in listOf("", "héllo 世界", "embedded\u0000nul")) {
        val report = reportNew(42, 3.5, label) { count, total, secs, nanos, projected, handle ->
            check(count == 42L && total == 3.5 && secs == 1L && nanos == 2L)
            check(projected == label)
            handle
        }
        expectInvalid { reportIntoStruct(2, report, 0, 0.0, null) }
        check(!report.isClosed)
        reportIntoStruct(0, report, 0, 0.0, null).close()
        check(report.isClosed)
        expectInvalid { reportIntoStruct(0, report, 0, 0.0, null) }
        report.close()
        reportIntoStruct(1, null, 7, 2.5, label).close()
    }
    // A builder failure must reclaim the output handle, even if Kotlin retained it.
    var escaped: Report? = null
    val failure = IllegalStateException("builder failure")
    try {
        reportNew(1, 2.0, "rollback") { _, _, _, _, _, handle -> escaped = handle; throw failure }
    } catch (caught: IllegalStateException) { check(caught === failure) }
    val reclaimed = checkNotNull(escaped)
    check(reclaimed.isClosed)
    reclaimed.close()
    expectInvalid { reportNew(0, 0.0, null) { _, _, _, _, _, handle -> handle.close() } }
    io.prebindgen.covertest.model.stampNew(Long.MIN_VALUE, Long.MAX_VALUE) { secs, nanos ->
        check(secs == Long.MIN_VALUE && nanos == Long.MAX_VALUE)
    }
    io.prebindgen.covertest.model.unsignedRoundTrip(255, 65535, 4294967295L, ULong.MAX_VALUE).close()
    expectInvalid { io.prebindgen.covertest.model.unsignedRoundTrip(256, 0, 0, 0UL) }
    summaryNew(Long.MAX_VALUE, -1.25) { count, total -> check(count == Long.MAX_VALUE && total == -1.25) }
    check(summaryTotalRaw(0, 42, -1.25, null) == -1.25)
    checkOwnership()
    checkPrimitives()
    payloadNew(Long.MIN_VALUE, Int.MAX_VALUE, -0.0, true) { id, seq, value, flag ->
        check(id == Long.MIN_VALUE && seq == Int.MAX_VALUE && value.toRawBits() == (-0.0).toRawBits() && flag)
    }
    storageWithPayload(Long.MAX_VALUE, Int.MIN_VALUE, Double.POSITIVE_INFINITY, false).close()
    println("covertest-kotlin PASS: all primitives, restored fields, selectors, ownership, rollback and concurrent close")
}
