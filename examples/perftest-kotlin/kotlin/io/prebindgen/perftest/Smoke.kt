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
    payloadNew(Long.MIN_VALUE, Int.MAX_VALUE, 1.25, true) { id, seq, value, flag ->
        check(id == Long.MIN_VALUE && seq == Int.MAX_VALUE && value == 1.25 && flag)
    }
    val stored = io.prebindgen.perftest.storage.storageNew()
    io.prebindgen.perftest.storage.storageGet(stored) { id, seq, value, flag ->
        check(id == null && seq == null && value == null && flag == null)
    }
    check(!storagePutOpt(stored, false, 0, 0, 0.0, false))
    check(storagePutOpt(stored, true, 42, -1, 3.5, false))
    io.prebindgen.perftest.storage.storageGet(stored) { id, seq, value, flag ->
        check(id == 42L && seq == -1 && value == 3.5 && flag == false)
    }
    stored.close()
    storageWithPayload(42, -1, 3.5, false).close()
    println("perftest-kotlin PASS: scalar Payload construction/projection and handle release")
}
