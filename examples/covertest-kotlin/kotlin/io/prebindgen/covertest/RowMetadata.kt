package io.prebindgen.covertest

import java.util.concurrent.CountDownLatch
import java.util.concurrent.atomic.AtomicReference

internal object RowMetadataProbe {
    init { NativeLibrary.ensureLoaded() }
    @JvmStatic external fun read(kind: Int, row: Array<Any?>, failure: Int, worker: Boolean, repeats: Int): LongArray
}
// Probe report: success, value, FindClass, GetObjectClass, GetMethodID,
// NewGlobalRef, DeleteGlobalRef, IsInstanceOf, primitive method calls, pending exception.
private fun metadataRead(kind: Int, value: Any?, worker: Boolean = false, repeats: Int = 1): LongArray =
    RowMetadataProbe.read(kind, arrayOf(value), 0, worker, repeats)
private fun noLookups(result: LongArray) { check(result.slice(2..5).all { it == 0L }) }

fun checkRowMetadata() {
    // This test runs first: null must not initialize the boxed Long cache.
    val absent = metadataRead(3, null)
    check(absent[0] == 0L && absent[7] == 0L && absent[8] == 0L)
    noLookups(absent)
    val gate = CountDownLatch(1)
    val ready = CountDownLatch(8)
    val failure = AtomicReference<Throwable?>()
    val reports = arrayOfNulls<LongArray>(8)
    val threads = List(8) { i -> Thread {
        try {
            ready.countDown();gate.await()
            reports[i] = metadataRead(3, Long.MIN_VALUE, worker = i % 2 == 0, repeats = 16)
        } catch (error: Throwable) { failure.set(error) }
    }.also { it.start() } }
    ready.await();gate.countDown();threads.forEach { it.join() }
    failure.get()?.let { throw it }
    val races = reports.map { requireNotNull(it) }
    check(races.all { it[0] == 1L && it[1] == Long.MIN_VALUE && it[7] == 16L && it[8] == 16L })
    check(races.sumOf { it[2] } in 1L..8L && races.all { it[3] == 0L && it[2] == it[4] && it[4] == it[5] })
    check(races.sumOf { it[5] - it[6] } == 1L)

    // Failed FindClass/GetMethodID/NewGlobalRef must leave the cache retryable.
    // A null global reference without a pending exception must also be rejected.
    for ((kind, injected, value) in listOf(
        Triple(0, 3, (-8).toByte()), Triple(1, 1, (-1600).toShort()),
        Triple(2, 2, 123), Triple(4, 4, 1.25f)
    )) {
        val bad = RowMetadataProbe.read(kind, arrayOf(value), injected, true, 1)
        check(bad[0] == 0L && bad[8] == 0L && bad[9] == (if (injected == 4) 0L else 1L))
        val retried = metadataRead(kind, value, true)
        check(retried[0] == 1L && retried[2] == 1L && retried[4] == 1L && retried[5] == 1L)
    }
    val values = listOf<Any>(Byte.MIN_VALUE, Short.MIN_VALUE, Int.MIN_VALUE, Long.MIN_VALUE, -0.0f, -0.0, true)
    val expected = listOf(Byte.MIN_VALUE.toLong(), Short.MIN_VALUE.toLong(), Int.MIN_VALUE.toLong(), Long.MIN_VALUE,
        (-0.0f).toRawBits().toLong() and 0xffffffffL, (-0.0).toRawBits(), 1L)
    val incorrect = listOf<Any>(1, 1, 1L, 1, 1.0, 1.0f, 1)
    for (kind in 0..6) {
        check(metadataRead(kind, values[kind])[1] == expected[kind])
        val warm = metadataRead(kind, values[kind], true, 16)
        check(warm[0] == 1L && warm[1] == expected[kind] && warm[7] == 16L && warm[8] == 16L)
        noLookups(warm)
        val wrong = metadataRead(kind, incorrect[kind])
        check(wrong[0] == 0L && wrong[7] == 1L && wrong[8] == 0L);noLookups(wrong)
        val nil = metadataRead(kind, null)
        check(nil[0] == 0L && nil[7] == 0L && nil[8] == 0L);noLookups(nil)
    }
    for ((kind, value) in listOf(7 to "héllo", 8 to byteArrayOf(0, -1), 9 to arrayOf<Any?>(42L))) {
        check(metadataRead(kind, value, true)[0] == 1L)
        val warm = metadataRead(kind, value, true, 16)
        check(warm[0] == 1L && warm[1] == 1L && warm[7] == 16L && warm[8] == 0L);noLookups(warm)
        val nil = metadataRead(kind, null)
        check(nil[0] == 1L && nil[1] == -1L && nil[7] == 0L);noLookups(nil)
        val wrong = metadataRead(kind, 42L)
        check(wrong[0] == 0L && wrong[7] == 1L);noLookups(wrong)
    }
    check(metadataRead(10, arrayOf<Any?>(42L))[0] == 1L)
    check(metadataRead(10, arrayOf<Any?>(42L, 43L))[0] == 0L)
    check(metadataRead(10, null)[0] == 0L)
    check(metadataRead(10, longArrayOf(42L))[0] == 0L)
    val root = metadataRead(10, arrayOf<Any?>(42L), true, 16)
    check(root[0] == 1L && root[7] == 16L);noLookups(root)
    println("row metadata PASS: cold races, initialization failure/retry, warm lookup omission, all primitives, nullable references and native workers")
}
