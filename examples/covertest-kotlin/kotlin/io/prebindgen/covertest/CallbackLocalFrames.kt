package io.prebindgen.covertest

import io.prebindgen.covertest.collections.*
import io.prebindgen.covertest.resources.*

internal object CallbackFrameProbe {
    init { NativeLibrary.ensureLoaded() }
    @JvmStatic external fun measure(mode: Int, size: Int): LongArray
}

object CallbackFrameEntryPoint {
    @JvmField val rejected = java.io.IOException("primitive callback collector rejected")
    @JvmStatic fun capacityClass(): Class<*> = JNINative.SequenceCapacity::class.java
    @JvmStatic fun collectorClass(mode: Int): Class<*> {
        val name = when (mode) {
            0 -> "i64"
            1, 5, 7 -> "handles"
            2 -> "text"
            3 -> "bytes"
            4 -> "borrow"
            6 -> "optional_items"
            8, 9, 10 -> "result_optional"
            else -> error("invalid frame case")
        }
        return JNINative::class.java.declaredClasses.single {
            it.simpleName.startsWith("SequenceCollector_collection_${name}_fold_")
        }
    }
    @JvmStatic fun run(mode: Int, size: Int) {
        when (mode) {
            0 -> check(collectionI64Fold(List(size) { it.toLong() }, 0L) { a, n -> a + n } == size.toLong() * (size - 1) / 2)
            1, 5, 7 -> {
                val before = ownedResourceDrops()
                val inputs = List(size) { collectionResourceNew(it.toLong()) }
                var calls = 0
                var rejected = false
                try {
                    collectionHandlesFold(inputs, 65, 0, Unit, onError = ResultFailureHandler { error("unexpected source error $it") }) { _, value ->
                        calls++
                        check(inputs.all { it.isClosed })
                        value.close()
                    }
                } catch (failure: java.io.IOException) {
                    check((mode == 5 || mode == 7) && failure === CallbackFrameEntryPoint.rejected)
                    rejected = true
                }
                check(rejected == (mode == 5 || mode == 7))
                check(calls == if (mode == 5 || mode == 7) 0 else size)
                check(inputs.all { it.isClosed })
                check(ownedResourceDrops() == before + size.toULong())
            }
            2 -> {
                val values = List(size) { "row\u0000世界$it" }
                check(collectionTextFold(values, 0) { a, s -> a + s.length } == values.sumOf { it.length })
            }
            3 -> check(collectionBytesFold(List(size) { byteArrayOf(it.toByte(), -1) }, 0) { a, b -> a + b.size } == size * 2)
            // Bulk-packed Rust i64 rows are ByteArray references on the wire.
            4 -> check(collectionBorrowFold(List(size) { it.toLong() }, 0L) { a, n -> a + n } == size.toLong() * (size - 1) / 2)
            6 -> {
                val values = List<Long?>(size) { if (it % 2 == 0) null else it.toLong() }
                check(collectionOptionalItemsFold(values, 0L) { a, n -> a + (n ?: 0L) } == values.sumOf { it ?: 0L })
            }
            8, 9, 10 -> {
                val before = ownedResourceDrops()
                val inputs = List(size) { collectionResourceNew(it.toLong()) }
                var calls = 0
                val sourceMode = when (mode) { 8 -> 0; 9 -> 2; else -> 1 }
                val result = collectionResultOptionalFold(inputs, sourceMode, 0L, onError = ResultFailureHandler { code ->
                    check(inputs.all { it.isClosed })
                    -code
                }) { a, resource ->
                    check(inputs.all { it.isClosed })
                    calls++
                    resource.close()
                    a + 1
                }
                check(result == when (mode) { 8 -> null; 9 -> -7L; else -> size.toLong() })
                check(calls == if (mode == 10) size else 0)
                check(inputs.all { it.isClosed })
                check(ownedResourceDrops() == before + size.toULong())
            }
            else -> error("invalid frame case")
        }
    }
}

fun checkCallbackLocalFrames() {
    for (size in listOf(0, 1, 10, 11, 63, 64, 65, 4096)) {
        for (mode in listOf(0, 1, 2, 3, 4, 6, 10)) {
            val counts = CallbackFrameProbe.measure(mode, size)
            val expected = if (mode in 2..4 || mode == 10) size.toLong() else 0L
            check(counts[0] == expected && counts[1] == expected) {
                "mode=$mode size=$size pushes=${counts[0]} pops=${counts[1]} expected=$expected"
            }
            check(counts[2] == size.toLong())
            check(counts[3] == 0L)
            check(counts[4] == if (size >= 64) 1L else 0L)
            check(counts[5] == if (size >= 64) size.toLong() else 0L)
        }
    }
    for (mode in listOf(8, 9)) check(CallbackFrameProbe.measure(mode, 65).contentEquals(LongArray(6)))
    val rejected = CallbackFrameProbe.measure(5, 3)
    check(rejected.contentEquals(longArrayOf(0, 0, 2, 1, 0, 0)))
    val reserveRejected = CallbackFrameProbe.measure(7, 65)
    check(reserveRejected.contentEquals(longArrayOf(0, 0, 0, 1, 1, 0)))
}
