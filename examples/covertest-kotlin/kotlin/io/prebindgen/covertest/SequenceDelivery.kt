package io.prebindgen.covertest

import io.prebindgen.covertest.collections.*
import io.prebindgen.covertest.enums.Code
import io.prebindgen.covertest.resources.ownedResourceDrops
import io.prebindgen.covertest.resources.ownedResourceId

fun checkSequenceDelivery() {
    val fail = ResultFailureHandler<Nothing> { error("unexpected source error $it") }
    check(collectionI64Fold(listOf(1L, 2L, 3L), 10L) { a, v -> a + v } == 16L)
    val initial = mutableListOf<Long>()
    check(collectionI64Fold(emptyList(), initial) { a, v -> a.apply { add(v) } } === initial)
    check(collectionI64Fold(emptyList(), null as String?) { a, v -> "$a$v" } == null)
    check(collectionTextFold(listOf("", "héllo 世界", "x\u0000y"), "") { a, v -> "$a[$v]" } == "[][héllo 世界][x\u0000y]")
    check(collectionBytesFold(listOf(byteArrayOf(1, 2), byteArrayOf()), 0) { a, v -> a + v.size } == 2)
    check(collectionNestedFold(listOf(listOf(1L, 2L), emptyList(), listOf(3L)), 0L) { a, v -> a + v.sum() } == 6L)
    check(collectionOptionsFold(listOf(null, 4L, null), 0, 0L) { a, v -> a + (v ?: 1L) } == 6L)
    check(collectionEnumFold(listOf(Code.Bottom, Code.Seven), 0) { a, _ -> a + 1 } == 2)
    check(collectionBorrowFold(listOf(2L, 3L), 1L) { a, v -> a * v } == 6L)
    check(collectionSliceFold(listOf(2L, 3L), 0L) { a, v -> a + v } == 5L)
    var calls = 0
    check(collectionOptionalFold(null, 12L) { a, v -> calls++; a + v } == null)
    check(collectionOptionalFold(emptyList(), 12L) { a, v -> calls++; a + v } == 12L)
    check(calls == 0)
    check(collectionOptionalFold(listOf(1L, 2L), 12L) { a, v -> calls++; a + v } == 15L && calls == 2)
    check(collectionResultFold(listOf(1L, 2L), true, 0L, onError = fail) { a, v -> a + v } == 3L)
    check(collectionResultFold(listOf(1L), false, 0L, onError = ResultFailureHandler { -it }) { _, _ -> error("fold after Err") } == -7L)

    // Input ownership is settled before user code executes. Borrow locks are also released.
    val movedInput = collectionResourceNew(20)
    val moved = collectionHandlesFold(listOf(movedInput), 65, 0, mutableListOf<io.prebindgen.covertest.resources.OwnedResource>(), onError = fail) { a, v ->
        check(movedInput.isClosed)
        a.apply { add(v) }
    }
    check(ownedResourceId(moved.single()) == 20L)
    moved.single().close()
    var bindingCalls = 0
    check(collectionHandlesFold(listOf(null), 65, 0, 0, onBindingError = JniErrorHandler { bindingCalls++; -1 }, onError = fail) { _, _ -> error("fold after binding error") } == -1)
    check(bindingCalls == 1)
    val storage = io.prebindgen.covertest.storage.storageNew()
    storagePutSlice(storage, listOf(Payload(1, 2, 3.0, true)))
    check(storageGetVecFold(storage, 0L) { a, v ->
        val closer = Thread { storage.close() }
        closer.start(); closer.join(5000)
        check(!closer.isAlive) { "fold still holds input lock" }
        a + v.id
    } == 1L)

    // Decomposed elements need no Kotlin data class or intermediate List<SequencePart>.
    val wideMin = java.math.BigInteger.ONE.shiftLeft(127).negate()
    var scalarRows = 0
    check(sequenceScalarsFold(0) { a, flag, byte, short, number, long, float, double, wide, text ->
        if (scalarRows++ == 0) {
            check(flag && byte == (-7).toByte() && short == (-1234).toShort() && number == -123456)
            check(long == Long.MIN_VALUE && float == 1.25f && double == -2.5 && wide == wideMin && text == "世界")
        } else {
            check(!flag && byte == 7.toByte() && short == 1234.toShort() && number == 123456)
            check(long == Long.MAX_VALUE && float == -1.25f && double == 2.5 && wide == wideMin.negate().subtract(java.math.BigInteger.ONE) && text == null)
        }
        a + 1
    } == 2 && scalarRows == 2)
    val start = ownedResourceDrops()
    check(sequencePartsFold(0, 0L, onError = fail) { a, number, text, resource ->
        check(text == "part $number 世界" && ownedResourceId(resource) == number)
        resource.close()
        a + number
    } == 6L)
    check(ownedResourceDrops() == start + 3uL)
    check(sequencePartsFold(1, 0L, onError = ResultFailureHandler { -it }) { _, _, _, _ -> error("fold after source error") } == -7L)
    check(ownedResourceDrops() == start + 3uL)
    check(sequencePartsFold(2, 0L, onBindingError = JniErrorHandler { -1L }, onError = fail) { _, _, _, _ -> error("fold before completed conversion") } == -1L)
    check(ownedResourceDrops() == start + 6uL)
    val original = IllegalStateException("folder identity")
    val escaped = mutableListOf<io.prebindgen.covertest.resources.OwnedResource>()
    try {
        sequencePartsFold(0, Unit, onError = fail) { _, number, _, resource ->
            escaped.add(resource)
            if (number == 2L) throw original
        }
        error("folder did not throw")
    } catch (caught: IllegalStateException) { check(caught === original) }
    check(!escaped[0].isClosed && escaped[1].isClosed)
    check(ownedResourceDrops() == start + 8uL)
    escaped[0].close()
    check(ownedResourceDrops() == start + 9uL)
    var retained: io.prebindgen.covertest.resources.OwnedResource? = null
    try {
        sequencePartsFold(0, Unit, onError = fail) { _, _, _, resource -> retained = resource.take(); throw original }
        error("folder did not throw")
    } catch (caught: IllegalStateException) { check(caught === original) }
    check(ownedResourceId(checkNotNull(retained)) == 1L)
    check(ownedResourceDrops() == start + 11uL)
    retained!!.close()
    check(ownedResourceDrops() == start + 12uL)
    check(sequenceLateDropFold(0, onBindingError = JniErrorHandler { -1 }, onError = fail) { _, _ -> error("fold after late native panic") } == -1)
    check(ownedResourceDrops() == start + 14uL)
    val nestedStart = ownedResourceDrops()
    val nestedInputs = listOf(listOf(collectionResourceNew(91), collectionResourceNew(92)), listOf(collectionResourceNew(93)))
    var nestedEscaped: List<io.prebindgen.covertest.resources.OwnedResource>? = null
    try {
        collectionNestedHandlesFold(nestedInputs, Unit) { _, values -> nestedEscaped = values; throw original }
        error("nested folder did not throw")
    } catch (caught: IllegalStateException) { check(caught === original) }
    check(nestedInputs.flatten().all { it.isClosed })
    check(checkNotNull(nestedEscaped).all { it.isClosed })
    check(ownedResourceDrops() == nestedStart + 3uL)
    for (mode in 0..2) {
        val inputs = listOf(collectionResourceNew(101), collectionResourceNew(102))
        val result = collectionResultOptionalFold(inputs, mode, 0L, onError = ResultFailureHandler { code -> check(inputs.all { it.isClosed }); -code }) { a, resource ->
            check(inputs.all { it.isClosed })
            val id = ownedResourceId(resource)
            resource.close()
            a + id
        }
        check(result == when (mode) {0 -> null; 1 -> 203L; else -> -7L})
        check(inputs.all { it.isClosed })
    }
    check(ownedResourceDrops() == nestedStart + 9uL)

    // A failing raw collector is a pending JVM exception: native ownership never
    // commits and the original exception must survive JNI's binding-error fallback.
    val native = Class.forName("io.prebindgen.covertest.JNINative")
    val nativeParts = native.declaredMethods.single { method ->
        val p = method.parameterTypes
        p.size == 3 && p[0] == Int::class.javaPrimitiveType && p[1].simpleName.startsWith("SequenceSink") && p[2] == BooleanArray::class.java
    }.also { it.isAccessible = true }
    val sinkType = nativeParts.parameterTypes[1]
    var collectorCalls = 0
    val sink = java.lang.reflect.Proxy.newProxyInstance(sinkType.classLoader, arrayOf(sinkType)) { _, method, _ ->
        check(method.name == "run")
        if (++collectorCalls == 2) throw original
        null
    }
    val beforeCollector = ownedResourceDrops()
    val outputAccepted = BooleanArray(1)
    try { nativeParts.invoke(null, 0, sink, outputAccepted); error("collector did not throw") }
    catch (caught: java.lang.reflect.InvocationTargetException) { check(caught.cause === original) }
    check(!outputAccepted[0] && collectorCalls == 2 && ownedResourceDrops() == beforeCollector + 3uL)
    collectorCalls = 0
    val invalidAck = nativeParts.invoke(null, 0, sink, BooleanArray(0)) as Array<*>
    check(invalidAck[0] == "invalid sequence output acknowledgement" && collectorCalls == 0)
    check(ownedResourceDrops() == beforeCollector + 3uL)
    val nullSinkAck = BooleanArray(1)
    val nullSink = nativeParts.invoke(null, 0, null, nullSinkAck) as Array<*>
    check(nullSink[0] == "null sequence callback" && !nullSinkAck[0])
    check(ownedResourceDrops() == beforeCollector + 3uL)
    check(collectionI64Fold(List(20000) { it.toLong() }, 0L) { a, v -> a + v } == 199990000L)
}
