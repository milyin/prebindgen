package io.prebindgen.covertest

import io.prebindgen.covertest.model.Report
import io.prebindgen.covertest.model.blobValueNew
import io.prebindgen.covertest.model.blobValueEcho

private fun expectInvalid(action: () -> Unit) {
    try { action(); error("expected conversion failure") }
    catch (_: IllegalArgumentException) { }
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
        val blob = blobValueNew(bytes)
        val echoed = blobValueEcho(blob)
        check(blob.isClosed)
        check(!echoed.isClosed)
        expectInvalid { blobValueEcho(blob) }
        echoed.close()
        blob.close()
    }
    expectInvalid { blobValueNew(null) }
    for (label in listOf("", "héllo 世界", "embedded\u0000nul")) {
        val report = reportNew(label) { projected, handle ->
            check(projected == label)
            handle
        }
        expectInvalid { reportIntoStruct(2, report, null) }
        check(!report.isClosed)
        reportIntoStruct(0, report, null).close()
        check(report.isClosed)
        expectInvalid { reportIntoStruct(0, report, null) }
        report.close()
        reportIntoStruct(1, null, label).close()
    }
    // A builder failure must reclaim the output handle, even if Kotlin retained it.
    var escaped: Report? = null
    val failure = IllegalStateException("builder failure")
    try {
        reportNew("rollback") { _, handle -> escaped = handle; throw failure }
    } catch (caught: IllegalStateException) { check(caught === failure) }
    val reclaimed = checkNotNull(escaped)
    check(reclaimed.isClosed)
    reclaimed.close()
    expectInvalid { reportNew(null) { _, handle -> handle.close() } }
    println("covertest-kotlin PASS: strings, bytes, selectors, field outputs, ownership, rollback and concurrent close")
}
