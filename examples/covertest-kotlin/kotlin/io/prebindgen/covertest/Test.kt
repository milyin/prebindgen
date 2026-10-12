package io.prebindgen.covertest

// Only main's currently supported declarations are active in build.rs.
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
    println("covertest-kotlin PASS: strings, subpackages, renamed handle, concurrent close")
}
