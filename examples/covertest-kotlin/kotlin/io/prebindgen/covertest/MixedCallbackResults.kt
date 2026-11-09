package io.prebindgen.covertest

import io.prebindgen.covertest.callbackreturns.*

private fun mixedHandles(batch: CallbackNestedBatch): List<CallbackReturnResource> =
    listOf(batch.first, batch.observed, batch.last) +
        (batch.groups.filterNotNull().flatten() + batch.tail).flatMap { listOfNotNull(it.first, it.optional, it.last) }

fun checkMixedCallbackResults() {
    check(callbackMixedScalars(MixedScalarBatchesCallback { emptyList() }) == 0L)
    check(callbackMixedScalars(MixedScalarBatchesCallback { List(3) {
        CallbackScalarBatch(7, listOf(CallbackReturnRecord(11, "a"), CallbackReturnRecord(12, "é")))
    } }) == 99L)
    callbackReturnResourceAuditStart()
    fun row(present: Boolean, character: Int = 'c'.code) = CallbackFixedResult(
        callbackReturnResourceNew(10), if (present) callbackReturnResourceNew(20) else null,
        character, callbackReturnResourceNew(30))
    for (worker in listOf(false, true)) for (present in listOf(false, true)) {
        val observed = callbackReturnResourceNew(70)
        val rowTotal = if (present) 60L else 40L
        for (phase in listOf("valid", "groups_panic", "groups_unicode", "marker_panic", "marker_unicode", "tail_panic", "tail_unicode")) {
            var calls = 0
            val transferred = mutableListOf<CallbackReturnResource>()
            val total = callbackMixedBatches(MixedBatchesCallback {
                check(callbackReturnResourceLive() == 1UL)
                val fail = calls++ == 0
                fun character(place: String, bad: Boolean): Int =
                    if (fail && bad && phase == place + "_panic") 'p'.code
                    else if (fail && bad && phase == place + "_unicode") 0xd800 else 'c'.code
                List(3) { batch ->
                    CallbackNestedBatch(callbackReturnResourceNew(5), observed,
                        listOf(null, emptyList(), List(3) { row(present, character("groups", batch == 1 && it == 1)) }),
                        character("marker", batch == 1),
                        List(2) { row(present, character("tail", batch == 1 && it == 0)) },
                        callbackReturnResourceNew(7)).also { transferred.addAll(mixedHandles(it).filter { handle -> handle !== observed }) }
                }
            }, phase != "valid", worker)
            check(total == 3 * (1105L + 5 * rowTotal))
            check(calls == if (phase == "valid") 1 else 2)
            check(transferred.all { it.isClosed } && !observed.isClosed && callbackReturnResourceLive() == 1UL)
        }
        for (bad in listOf(false, true)) {
            var calls = 0
            val total = callbackMixedBatch(MixedBatchCallback {
                check(callbackReturnResourceLive() == 1UL)
                val fail = calls++ == 0 && bad
                CallbackNestedBatch(callbackReturnResourceNew(5), observed,
                    listOf(null, emptyList(), List(3) { row(present, if (fail && it == 1) 'p'.code else 'c'.code) }),
                    'c'.code, List(2) { row(present) }, callbackReturnResourceNew(7))
            }, bad, worker)
            check(total == 1105L + 5 * rowTotal && calls == if (bad) 2 else 1)
            check(!observed.isClosed && callbackReturnResourceLive() == 1UL)
        }
        // The same owner cannot be transferred in an ordinary field and a packed row.
        var calls = 0
        val total = callbackMixedBatches(MixedBatchesCallback {
            check(callbackReturnResourceLive() == 1UL)
            val duplicate = callbackReturnResourceNew(5)
            val bad = calls++ == 0
            listOf(CallbackNestedBatch(duplicate, observed, listOf(emptyList()), 'c'.code,
                listOf(CallbackFixedResult(if (bad) duplicate else callbackReturnResourceNew(10), null, 'c'.code, callbackReturnResourceNew(30))),
                callbackReturnResourceNew(7)))
        }, true, worker)
        check(calls == 2 && total == 132L && callbackReturnResourceLive() == 1UL && !observed.isClosed)
        observed.close()
    }
    check(callbackMixedBatches(MixedBatchesCallback { emptyList() }, false, false) == 0L)
    // Ordinary function inputs share the same per-row bulk decoding.
    val original = List(3) { CallbackNestedBatch(callbackReturnResourceNew(5), callbackReturnResourceNew(70),
        listOf(null, emptyList(), listOf(row(true))), 'c'.code, listOf(row(false)), callbackReturnResourceNew(7)) }
    val inputs = original.flatMap(::mixedHandles)
    val echoed = callbackMixedBatchesEcho(original)
    check(inputs.all { it.isClosed })
    check(echoed.all { it.groups[0] == null && it.groups[1]!!.isEmpty() && it.groups[2]!!.size == 1 })
    val outputs = echoed.flatMap(::mixedHandles)
    check(outputs.size == inputs.size && callbackReturnResourceLive() == outputs.size.toULong())
    outputs.forEach { it.close() }
    check(callbackReturnResourceLive() == 0UL)
    checkMalformedMixedCallbackResults()
    callbackReturnResourceAuditFinish()
    println("mixed callback results PASS: per-row bulk frames, optional nested lists, partial rows, borrowed owners, duplicate rejection, retries and workers")
}

private class RawMixedCallback(private val body: () -> Array<Any?>?) {
    fun run(): Array<Any?>? = body()
}

// These layouts describe the configured callback expansion, including the
// explicit presence field of each optional inner list. Offsets come from Rust.
private fun mixedFrame(present: Boolean, mode: String, fail: Boolean, groups: Boolean): ByteArray {
    val root = if (groups) BulkLayouts.r31afb2a04211128a else BulkLayouts.r08a37f4477600073
    val group = BulkLayouts.r1033cb21c6bf06e2
    val row = BulkLayouts.r3f18e0d568ad44e3
    val values = root.size + if (groups) 4 * group.size else 0
    val count = if (groups) 4 else if (fail && mode == "tail_range") 0 else 2
    val buffer = java.nio.ByteBuffer.allocate(values + count * row.size).order(BulkLayouts.byteOrder)
    fun range(at: Int, offset: Long, length: Long) { buffer.putLong(at, offset); buffer.putLong(at + 8, length) }
    if (groups) {
        range(root.offsets[0], root.size.toLong(), 4)
        for (i in 0..3) {
            val at = root.size + i * group.size
            buffer.putInt(at + group.offsets[0], if (i == 1) 0 else if (fail && mode == "tag" && i == 2) 7 else 1)
            range(at + group.offsets[1], when { i == 1 -> -1L; fail && mode == "range" && i == 2 -> Long.MAX_VALUE; i == 3 -> (values + 3 * row.size).toLong(); else -> values.toLong() },
                when(i) { 0 -> 3L; 1, 2 -> 0L; else -> 1L })
        }
    } else range(root.offsets[0], if (fail && mode == "tail_range") Long.MAX_VALUE else values.toLong(), count.toLong())
    for (i in 0 until count) {
        val at = values + i * row.size
        val bad = fail && if (groups) i == 1 else i == 0
        val first = callbackReturnResourceNew(10)
        val optional = if (present) callbackReturnResourceNew(20) else null
        val last = if (groups && bad && mode == "pointer") null else callbackReturnResourceNew(30)
        buffer.putLong(at + row.offsets[0], first.raw)
        buffer.putLong(at + row.offsets[1], optional?.raw ?: 0L)
        buffer.putInt(at + row.offsets[2], if (bad && mode == if (groups) "char" else "tail_char") 0xd800 else 'c'.code)
        buffer.putLong(at + row.offsets[3], last?.raw ?: 1L)
        first.markConsumed(); optional?.markConsumed(); last?.markConsumed()
    }
    return buffer.array()
}

private fun checkMalformedMixedCallbackResults() {
    val method = JNINative::class.java.declaredMethods.single {
        it.name.startsWith("call_callback_mixed_batches_") && it.parameterTypes.firstOrNull() == Any::class.java
    }.also { it.isAccessible = true }
    for (worker in listOf(false, true)) for (present in listOf(false, true)) {
        val observed = callbackReturnResourceNew(70)
        for (mode in listOf("range", "tag", "char", "pointer", "tail_range", "tail_char", "root_scalar", "root_short", "bytes", "length", "negative_length", "short_length", "long_length", "row", "null", "first_pointer", "last_pointer")) {
            var calls = 0
            val raw = RawMixedCallback {
                check(callbackReturnResourceLive() == 1UL) { "mixed cleanup leaked before retry: $mode" }
                val fail = calls++ == 0
                if (fail && mode == "null") return@RawMixedCallback null
                Array<Any?>(3) { batch ->
                    val bad = fail && batch == 1
                    if (bad && mode == "row") return@Array "wrong outer row class"
                    val first = if (bad && mode == "first_pointer") null else callbackReturnResourceNew(5)
                    val packed = if (bad && mode == "bytes") ByteArray(0) else mixedFrame(present, mode, bad, true)
                    val fields = mutableListOf<Any?>(first?.raw ?: 1L, observed.raw,
                        if (bad && mode == "bytes") "wrong byte array class" else packed,
                        when { bad && mode == "length" -> "wrong length class"; bad && mode == "negative_length" -> -1; bad && mode == "short_length" -> packed.size - 1; bad && mode == "long_length" -> packed.size + 1; else -> packed.size })
                    first?.markConsumed()
                    if (!(bad && mode == "root_short")) {
                        fields.add(if (bad && mode == "root_scalar") "wrong marker class" else 'c'.code)
                        val tail = mixedFrame(present, mode, bad, false)
                        fields.add(tail); fields.add(tail.size)
                        val last = if (bad && mode == "last_pointer") null else callbackReturnResourceNew(7)
                        fields.add(last?.raw ?: 1L); last?.markConsumed()
                    }
                    fields.toTypedArray()
                }
            }
            val result = method.invokePacket(raw, true, worker)
            val rowTotal = if (present) 60L else 40L
            check(result[0] == null && result[1] == 3 * (1116L + 6 * rowTotal)) { "$mode: ${result.toList()}" }
            check(calls == 2 && !observed.isClosed && callbackReturnResourceLive() == 1UL)
        }
        observed.close()
    }
}
