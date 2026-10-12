package io.prebindgen.perftest

// The benchmark operations remain disabled until their declarations are supported.
fun main() {
    repeat(100) {
        val storage = io.prebindgen.perftest.storage.storageNew()
        check(!storage.isClosed)
        storage.close()
        storage.close()
        check(storage.isClosed)
    }
    println("perftest-kotlin PASS: generated handle creation and release")
}
