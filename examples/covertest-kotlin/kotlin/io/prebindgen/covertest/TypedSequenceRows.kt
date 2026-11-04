package io.prebindgen.covertest

import io.prebindgen.covertest.collections.*
import io.prebindgen.covertest.resources.*
import java.lang.reflect.InvocationTargetException

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
    val original = IllegalStateException("typed columns fold failure")
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

    // Use a generated three-Long capture directly to inject a failure while
    // copying its last column. This exercises the same unwind path as a failed
    // allocation without exhausting the JVM heap. No fake pointers are decoded.
    val type = JNINative::class.java.declaredClasses.single { candidate ->
        candidate.simpleName.startsWith("SequenceRows_") && candidate.declaredMethods.any {
            it.name == "add" && it.parameterTypes.toList() == List(3) { Long::class.javaPrimitiveType }
        }
    }
    val constructor = type.getDeclaredConstructor().also { it.isAccessible = true }
    val rows = constructor.newInstance()
    val other = constructor.newInstance()
    val columns = (0..2).map { type.getDeclaredField("c$it").also { it.isAccessible = true } }
    check(columns.all { it.get(rows) === it.get(other) && (it.get(rows) as LongArray).isEmpty() })
    val add = type.getDeclaredMethod("add", *Array(3) { Long::class.javaPrimitiveType!! }).also { it.isAccessible = true }
    val size = type.getDeclaredMethod("getSize").also { it.isAccessible = true }
    add.invoke(rows, 11L, 12L, 13L)
    val previous = columns.map { it.get(rows) as LongArray }
    columns[2].set(rows, null)
    try { add.invoke(rows, 21L, 22L, 23L); error("column copy should fail") }
    catch (failure: InvocationTargetException) { check(failure.cause is NullPointerException) }
    check(size.invoke(rows) == 1)
    check((columns[0].get(rows) as LongArray)[0] == 11L && (columns[1].get(rows) as LongArray)[0] == 12L)
    check(previous.map { it.single() } == listOf(11L, 12L, 13L))
    columns[2].set(rows, previous[2])
    add.invoke(rows, 21L, 22L, 23L)
    check(size.invoke(rows) == 2)
    check(columns.map { (it.get(rows) as LongArray).toList() } == listOf(listOf(11L, 21L), listOf(12L, 22L), listOf(13L, 23L)))
    check(size.invoke(other) == 0 && columns.all { (it.get(other) as LongArray).isEmpty() })
    println("typed sequence capture PASS: growth, reentry, concurrency, rollback and ownership")
}
