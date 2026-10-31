package io.prebindgen.covertest
import io.prebindgen.covertest.bulk.*
fun checkBulk() {
    checkBulkMalformed()
    check(bulkScalar(41) == 42L)
    check(bulkSelect(0, 4L, 99L) == 5L)
    check(bulkSelect(1, 4L, 99L) == 99L)
    check(bulkKinds(listOf(BulkKind.A, BulkKind.B)) == listOf(BulkKind.A, BulkKind.B))
    check(bulkUnits(listOf(Unit, Unit)) == listOf(Unit, Unit))
    check(bulkUnits(emptyList()).isEmpty())
    check(bulkFlags(listOf(true, false, true)) == listOf(true, false, true))
    val numbers = listOf(Long.MIN_VALUE, 0L, Long.MAX_VALUE)
    check(bulkNumbers(numbers) == numbers)
    check(bulkNumbers(emptyList()).isEmpty())
    val large = (0L until 50000L).toList()
    check(bulkNumbers(large) == large)
    check(bulkArray(listOf(1L, 2L, 3L)) == listOf(1L, 2L, 3L))
    check(bulkArrayBorrow(listOf(1L, 2L, 3L)) == 6L)
    var arrayFailed = false
    check(bulkArray(listOf(1L), JniErrorHandler { arrayFailed = true; emptyList() }).isEmpty())
    check(arrayFailed)
    check(bulkNested(listOf(numbers, emptyList(), listOf(7L))) == listOf(numbers, emptyList<Long>(), listOf(7L)))
    val point = BulkPoint(9, 1.5)
    check(bulkPoint(point) == point)
    check(bulkEnvelope(null) == null)
    val envelope = BulkEnvelope(point, 11)
    check(bulkEnvelope(envelope) == envelope)
    check(bulkOptionalWide(null) == null)
    check(bulkOptionalWide(java.math.BigInteger.TEN) == java.math.BigInteger.TEN)
    check(bulkOptionalCallback(BulkOptionalCallback { null }) == 0uL)
    check(bulkOptionalCallback(BulkOptionalCallback { emptyList() }) == 0uL)
    val primitives = BulkPrimitives(true, (-128).toByte(), (-32768).toShort(), 255, 65535, 4294967295L, 0x10ffff, 1uL)
    check(bulkPrimitives(primitives) == primitives)
    check(bulkPointPlain(point) == point)
    check(bulkSlice(listOf(point, point)) == listOf(point, point))
    check(bulkPoints(listOf(point, BulkPoint(-1, Double.NEGATIVE_INFINITY))) == listOf(point, BulkPoint(-1, Double.NEGATIVE_INFINITY)))
    check(bulkPoints(emptyList()).isEmpty())
    check(bulkPointArray(listOf(point, point)) == listOf(point, point))
    for (text in listOf("", "héllo 世界", "embedded\u0000nul", "😀")) check(bulkText(text) == text)
    var invalidSurrogate = false
    check(bulkText("\ud800", JniErrorHandler { invalidSurrogate = true; "recovered" }) == "recovered")
    check(invalidSurrogate)
    check(bulkText(null, JniErrorHandler { "recovered" }) == "recovered")
    val strings = listOf("", "世界", "nul\u0000text")
    check(bulkStrings(strings) == strings)
    check(bulkOptional(null) == null)
    check(bulkOptional(emptyList()) == emptyList<String?>())
    check(bulkOptional(listOf(null, "", "世界", null)) == listOf(null, "", "世界", null))
    val bytes = listOf(byteArrayOf(), byteArrayOf(0, -1, 127))
    val decoded = bulkBytes(bytes)
    check(decoded.size == bytes.size && bytes.indices.all { bytes[it].contentEquals(decoded[it]) })

    check(bulkNative(listOf(BulkNative(1, Long.MIN_VALUE, ULong.MAX_VALUE), BulkNative(-7, Long.MAX_VALUE, 1uL))) == listOf(BulkNative(1, Long.MIN_VALUE, ULong.MAX_VALUE), BulkNative(-7, Long.MAX_VALUE, 1uL)))
    check(bulkExpanded(7, "世界") { number, text -> number == 7L && text == "世界" })
    val expanded = bulkExpandedRows(listOf(BulkExpanded(7, "a"), BulkExpanded(9, "世界")))
    check(expanded == listOf(BulkExpanded(7, "a"), BulkExpanded(9, "世界")))
    check(bulkExpandedRows(expanded) == expanded)
    check(bulkResult(7, "ok", true, onError = BulkFailureHandler { false }) { n, text -> n == 7L && text == "ok" })
    check(bulkResult(7, "error", false, onError = BulkFailureHandler { code -> code == 17L }) { _, _ -> false })
    val minWide = java.math.BigInteger.ONE.shiftLeft(127).negate()
    val signed = listOf(minWide, java.math.BigInteger.ZERO, minWide.negate().subtract(java.math.BigInteger.ONE))
    check(bulkWide(signed) == signed)
    val unsigned = listOf(java.math.BigInteger.ZERO, java.math.BigInteger.ONE.shiftLeft(128).subtract(java.math.BigInteger.ONE))
    check(bulkUnsignedWide(unsigned) == unsigned)
    var wideFailed = false
    try { bulkWide(listOf(minWide.negate())) } catch (_: IllegalArgumentException) { wideFailed = true }
    check(wideFailed)
    var nullWideFailed = false
    check(bulkWide(listOf(null), JniErrorHandler { nullWideFailed = true; emptyList() }).isEmpty())
    check(nullWideFailed)
    check(bulkCallback(BulkNumbersCallback { values -> values.map { it * 3 } }) == listOf(3L, 6L, 9L))
    check(bulkCallbackWorker(BulkWorkerCallback { values -> values.map { it + 10 } }) == listOf(13L, 14L))
    val drops = bulkDrops()
    val a = bulkHandleNew(4); val b = bulkHandleNew(8)
    check(bulkBorrow(a) == 4L)
    val returned = bulkHandles(listOf(a, b))
    check(bulkHandleValue(returned[0]) == 4L && bulkHandleValue(returned[1]) == 8L)
    returned.forEach { it.close() }
    check(bulkDrops() == drops + 2uL)
    val same = bulkHandleNew(21)
    val alias = BulkHandle(same.raw)
    val unread = bulkHandleNew(22)
    var aliasFailed = false
    check(bulkHandles(listOf(same, alias, unread), JniErrorHandler { aliasFailed = true; emptyList() }).isEmpty())
    check(aliasFailed && !same.isClosed && !alias.isClosed && !unread.isClosed)
    alias.markConsumed(); same.close()
    check(bulkHandleValue(unread) == 22L)
    unread.close()
    val live = bulkHandleNew(17)
    var bindingFailed = false
    val recovered = bulkChecked(live, 300, JniErrorHandler { bindingFailed = true; live })
    check(bindingFailed && recovered === live && live.isClosed)
    live.close()
    var callbackInputs: List<BulkHandle> = emptyList()
    val callbackReturned = bulkHandleCallback(BulkHandlesCallback { values -> callbackInputs = values; values })
    check(callbackReturned.map { bulkHandleValue(it) } == listOf(6L, 9L))
    check(callbackInputs.all { it.isClosed })
    callbackReturned.forEach { it.close() }
    check(bulkDrops() == drops + 7uL)
    var buildFailed = false
    try { bulkHandleViews(19) { handle, number -> check(bulkHandleValue(handle) == number); throw java.io.IOException("bulk build") } }
    catch (_: java.io.IOException) { buildFailed = true }
    check(buildFailed && bulkDrops() == drops + 8uL)
    val recovery = bulkHandleNew(91)
    var lateFailed = false
    val late = bulkLate(true, JniErrorHandler { lateFailed = true; recovery })
    check(lateFailed && late === recovery && bulkDrops() == drops + 9uL)
    recovery.close()
    check(bulkDrops() == drops + 10uL)
    check(bulkOptionalHandles(null) == null)
    check(bulkOptionalHandles(emptyList())!!.isEmpty())
    val optionalOwned = bulkHandleNew(53)
    val optionalResult = bulkOptionalHandles(listOf(null, optionalOwned, null))!!
    check(optionalOwned.isClosed && optionalResult[0] == null && optionalResult[2] == null)
    check(bulkHandleValue(optionalResult[1]!!) == 53L)
    optionalResult[1]!!.close()
    check(bulkDrops() == drops + 11uL)
}

private fun checkBulkMalformed() {
    fun reject(name: String, arguments: List<Any>, message: String) {
        val method = JNINative::class.java.declaredMethods.single { it.name.startsWith("call_${name}_") }
        val captureType = method.parameterTypes.last()
        val companion = captureType.getField("Companion").get(null)
        val capture = companion.javaClass.getMethod("acquire").invoke(companion)
        try {
            method.invoke(JNINative, *(arguments + capture).toTypedArray())
            check(captureType.getMethod("getStatus").invoke(capture) == 2)
            check((captureType.getMethod("getMessage").invoke(capture) as String).contains(message))
        } finally { captureType.getMethod("release").invoke(capture) }
    }
    reject("bulk_numbers", listOf(byteArrayOf(), 0), "truncated bulk root")
    reject("bulk_numbers", listOf(ByteArray(16), 17), "exceeds byte array")
    reject("bulk_numbers", listOf(ByteArray(16), -1), "invalid bulk frame")
    val oversized = java.nio.ByteBuffer.allocate(16).order(BulkLayouts.byteOrder).putLong(16).putLong(Long.MAX_VALUE).array()
    reject("bulk_numbers", listOf(oversized, oversized.size), "bulk row")
    val invalidText = java.nio.ByteBuffer.allocate(17).order(BulkLayouts.byteOrder).putLong(16).putLong(1).put(0xff.toByte()).array()
    reject("bulk_text", listOf(invalidText, invalidText.size), "UTF-8")
    val handle = bulkHandleNew(71)
    val frame = java.nio.ByteBuffer.allocate(8).order(BulkLayouts.byteOrder).putLong(handle.raw).array()
    val ownership = LongArray(1) { -1L }
    reject("bulk_checked", listOf(frame, frame.size, byteArrayOf(), 0, ownership), "truncated bulk root")
    check(ownership[0] == -2L && handle.isClosed.not())
    // The first argument was consumed before the invalid following bulk frame.
    handle.markConsumed()
    handle.close()
}
