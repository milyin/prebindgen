package io.prebindgen.perftest

import io.prebindgen.perftest.storage.payloadHandlerNew
import io.prebindgen.perftest.storage.storageCallback
import io.prebindgen.perftest.storage.storageNew

private data class PayloadValues(val id: Long, val seq: Int, val value: Double, val flag: Boolean)

private fun checkRetainedPayloadHandler() {
    val received = mutableListOf<PayloadValues>()
    var failNext = false
    val callback = PayloadCallback { id, seq, value, flag ->
        received += PayloadValues(id, seq, value, flag)
        if (failNext) {
            failNext = false
            throw IllegalStateException("expected retained Payload callback failure")
        }
    }
    storageNew().use { storage ->
        payloadHandlerNew(callback).use { handler ->
            storageCallback(storage, handler)
            check(received.isEmpty())

            val first = PayloadValues(42, -1, 3.5, false)
            check(storagePutOpt(storage, true, first.id, first.seq, first.value, first.flag))
            storageCallback(storage, handler)
            check(received == listOf(first))

            // storagePutOpt appends. The second payload must still arrive even
            // when the first callback in the same native delivery throws.
            val second = PayloadValues(-7, 8, 1.25, true)
            check(storagePutOpt(storage, true, second.id, second.seq, second.value, second.flag))
            failNext = true
            storageCallback(storage, handler)
            check(!failNext)
            check(received == listOf(first, first, second))

            storageCallback(storage, handler)
            check(received == listOf(first, first, second, first, second))
        }
    }
}

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
    checkRetainedPayloadHandler()
    println("perftest-kotlin PASS: Payload construction, projection, retained callback and handle release")
}
