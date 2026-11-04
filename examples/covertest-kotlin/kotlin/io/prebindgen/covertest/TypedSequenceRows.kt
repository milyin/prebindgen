package io.prebindgen.covertest

import io.prebindgen.covertest.collections.*
import io.prebindgen.covertest.resources.*

fun checkTypedSequenceRows() {
    // Zero/shared empty arrays, growth boundaries, reentry and concurrent calls.
    for (size in listOf(0, 1, 2, 3, 7, 8, 9, 63, 64, 65, 1025)) {
        val values = List(size) { 1000000L + it }
        check(collectionI64Fold(values, 0L) { sum, value -> sum + value } == values.sum())
    }
    var nestedCalls = 0
    check(collectionI64Fold(listOf(1001L, 1002L, 1003L), 0L) { sum, value ->
        nestedCalls++
        check(collectionI64Fold(listOf(value, -value), 7L) { a, b -> a + b } == 7L)
        sum + value
    } == 3006L && nestedCalls == 3)
    val failures = java.util.concurrent.ConcurrentLinkedQueue<Throwable>()
    val threads = List(2) { thread -> Thread {
        try {
            repeat(20) {
                val values = List(129) { 1000000L * (thread + 1) + it }
                check(collectionI64Fold(values, 0L) { a, b -> a + b } == values.sum())
            }
        } catch (failure: Throwable) { failures.add(failure) }
    } }
    threads.forEach { it.start() }; threads.forEach { it.join() }
    check(failures.isEmpty()) { failures.toString() }

    // Constructor/fold failure after growth must release the failing and unread
    // owners, preserve accepted owners and keep the original exception.
    val before = ownedResourceDrops()
    val inputs = List(65) { collectionResourceNew(1000000L + it) }
    val retained = mutableListOf<OwnedResource>()
    val original = IllegalStateException("typed row fold failure")
    var calls = 0
    try {
        collectionHandlesFold(inputs, 65, 0, Unit, onError = ResultFailureHandler { error("unexpected source error $it") }) { _, value ->
            check(inputs.all { it.isClosed })
            if (++calls == 33) throw original
            retained.add(value)
        }
        error("fold should throw")
    } catch (failure: IllegalStateException) { check(failure === original) }
    check(retained.size == 32 && retained.none { it.isClosed })
    check(ownedResourceDrops() == before + 33uL)
    retained.forEach { it.close() }
    check(ownedResourceDrops() == before + 65uL)

    println("typed sequence capture PASS: growth boundaries, reentry, concurrency and ownership")
}
