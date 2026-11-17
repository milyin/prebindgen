package bench.allocations
internal object Counters {
    init { System.loadLibrary("bulk_bench") }
    @JvmStatic external fun start(): Boolean
    @JvmStatic external fun stop(): LongArray
}
private class Measured(val run: () -> Long, val close: () -> Unit)
private fun makePlain(kind: String, size: Int): Measured {
    return when (kind) {
        "canonical_value" -> Measured({bench.plain.canonicalValue(0)}, {})
        "expanded_value" -> Measured({bench.plain.expandedValue(0)}, {})
        "canonical_value_callback" -> Measured({bench.plain.canonicalValueCallback({it+1},0)}, {})
        "expanded_value_callback" -> Measured({bench.plain.expandedValueCallback({it+1},0)}, {})
        "canonical_values", "expanded_values" -> {
            val rows=List(size) {it.toLong()}
            Measured({val output=if(kind=="canonical_values") bench.plain.canonicalValues(rows) else bench.plain.expandedValues(rows);check(output==rows);size.toLong()}, {})
        }
        "callback_batches", "callback_batches_many", "callback_mixed_batches", "callback_mixed_batches_many" -> {
            val small = kind.endsWith("_many")
            val mixed = kind.startsWith("callback_mixed")
            fun batch(count: Int) = bench.plain.HandleBatch(bench.plain.inputHandleNew(1),
                List(count) { bench.plain.HandleRecord(bench.plain.inputHandleNew(it.toLong()), it.toLong()) })
            var rows = if (small) List(size) { batch(1) } else listOf(batch(size))
            val callback = bench.plain.OwnedBatchesCallback { rows }
            val packedCallback = bench.plain.OwnedMixedBatchesCallback { rows }
            Measured({ rows = if (mixed) bench.plain.callbackMixedBatchesOwnedHandles(packedCallback) else bench.plain.callbackBatchesOwnedHandles(callback); rows.sumOf { it.rows.size }.toLong() },
                { rows.forEach { b -> b.guard.close(); b.rows.forEach { it.handle.close() } } })
        }
        "callback_nested", "callback_nested_many" -> {
            var rows=if(kind=="callback_nested") listOf(emptyList(),List(size) {bench.plain.inputHandleNew(it.toLong())},emptyList())
                     else List(size) {listOf(bench.plain.inputHandleNew(it.toLong()))}
            val callback=bench.plain.OwnedNestedCallback {rows}
            Measured({rows=bench.plain.callbackNestedOwnedHandles(callback);rows.sumOf {it.size}.toLong()},{rows.forEach {group->group.forEach {it.close()}}})
        }
        "numbers_input" -> {
            val rows=List(size) {it.toLong()}
            val total=size.toLong()*(size-1)/2
            Measured({check(bench.plain.numbersSum(rows)==total);size.toLong()}, {})
        }
        "records_input" -> {
            val rows=List(size) {i -> val x=i.toLong();bench.plain.Row(x,x,x,x,x,x,x,x)}
            val total=8*size.toLong()*(size-1)/2
            Measured({check(bench.plain.rowsSum(rows)==total);size.toLong()}, {})
        }
        "flat" -> {
            var rows=List(size) { bench.plain.inputHandleNew(it.toLong()) }
            Measured({ rows=bench.plain.ownedHandlesEcho(rows); rows.size.toLong() }, { rows.forEach { it.close() } })
        }
        "records" -> {
            var rows=List(size) { bench.plain.HandleRecord(bench.plain.inputHandleNew(it.toLong()),it.toLong()) }
            Measured({ rows=bench.plain.ownedHandleRecordsEcho(rows); rows.size.toLong() }, { rows.forEach { it.handle.close() } })
        }
        "callback_record" -> {
            require(size==1)
            var row=bench.plain.HandleRecord(bench.plain.inputHandleNew(1),1L)
            val callback=bench.plain.OwnedRecordCallback { row }
            Measured({row=bench.plain.callbackOwnedRecord(callback);1L},{row.handle.close()})
        }
        "callback_records" -> {
            var rows=List(size) {bench.plain.HandleRecord(bench.plain.inputHandleNew(it.toLong()),it.toLong())}
            val callback=bench.plain.OwnedRecordsCallback {rows}
            Measured({rows=bench.plain.callbackOwnedRecords(callback);rows.size.toLong()},{rows.forEach {it.handle.close()}})
        }
        "buffers" -> {
            var rows=List(size) {bench.plain.BufferRecord(bench.plain.inputHandleNew(it.toLong()),"x".repeat(64),ByteArray(64))}
            Measured({rows=bench.plain.ownedBufferRecordsEcho(rows);check(rows.all {it.text.length==64 && it.bytes.size==64});rows.size.toLong()},{rows.forEach {it.handle.close()}})
        }
        "nested", "nested_many" -> {
            var rows=if (kind=="nested") listOf(emptyList(),List(size) {bench.plain.inputHandleNew(it.toLong())},emptyList())
                     else List(size) {listOf(bench.plain.inputHandleNew(it.toLong()))}
            Measured({ rows=bench.plain.nestedOwnedHandlesEcho(rows); rows.sumOf { it.size }.toLong() }, { rows.forEach { group -> group.forEach {it.close()} } })
        }
        "control" -> Measured({bench.plain.control(0)}, {})
        else -> error("unknown case")
    }
}
private fun makeBulk(kind: String, size: Int): Measured {
    return when (kind) {
        "canonical_value" -> Measured({bench.bulk.canonicalValue(0)}, {})
        "expanded_value" -> Measured({bench.bulk.expandedValue(0)}, {})
        "canonical_value_callback" -> Measured({bench.bulk.canonicalValueCallback({it+1},0)}, {})
        "expanded_value_callback" -> Measured({bench.bulk.expandedValueCallback({it+1},0)}, {})
        "canonical_values", "expanded_values" -> {
            val rows=List(size) {it.toLong()}
            Measured({val output=if(kind=="canonical_values") bench.bulk.canonicalValues(rows) else bench.bulk.expandedValues(rows);check(output==rows);size.toLong()}, {})
        }
        "callback_batches", "callback_batches_many", "callback_mixed_batches", "callback_mixed_batches_many" -> {
            val small = kind.endsWith("_many")
            val mixed = kind.startsWith("callback_mixed")
            fun batch(count: Int) = bench.bulk.HandleBatch(bench.bulk.inputHandleNew(1),
                List(count) { bench.bulk.HandleRecord(bench.bulk.inputHandleNew(it.toLong()), it.toLong()) })
            var rows = if (small) List(size) { batch(1) } else listOf(batch(size))
            val callback = bench.bulk.OwnedBatchesCallback { rows }
            val packedCallback = bench.bulk.OwnedMixedBatchesCallback { rows }
            Measured({ rows = if (mixed) bench.bulk.callbackMixedBatchesOwnedHandles(packedCallback) else bench.bulk.callbackBatchesOwnedHandles(callback); rows.sumOf { it.rows.size }.toLong() },
                { rows.forEach { b -> b.guard.close(); b.rows.forEach { it.handle.close() } } })
        }
        "callback_nested", "callback_nested_many" -> {
            var rows=if(kind=="callback_nested") listOf(emptyList(),List(size) {bench.bulk.inputHandleNew(it.toLong())},emptyList())
                     else List(size) {listOf(bench.bulk.inputHandleNew(it.toLong()))}
            val callback=bench.bulk.OwnedNestedCallback {rows}
            Measured({rows=bench.bulk.callbackNestedOwnedHandles(callback);rows.sumOf {it.size}.toLong()},{rows.forEach {group->group.forEach {it.close()}}})
        }
        "numbers_input" -> {
            val rows=List(size) {it.toLong()}
            val total=size.toLong()*(size-1)/2
            Measured({check(bench.bulk.numbersSum(rows)==total);size.toLong()}, {})
        }
        "records_input" -> {
            val rows=List(size) {i -> val x=i.toLong();bench.bulk.Row(x,x,x,x,x,x,x,x)}
            val total=8*size.toLong()*(size-1)/2
            Measured({check(bench.bulk.rowsSum(rows)==total);size.toLong()}, {})
        }
        "flat" -> {
            var rows=List(size) { bench.bulk.inputHandleNew(it.toLong()) }
            Measured({ rows=bench.bulk.ownedHandlesEcho(rows); rows.size.toLong() }, { rows.forEach { it.close() } })
        }
        "records" -> {
            var rows=List(size) { bench.bulk.HandleRecord(bench.bulk.inputHandleNew(it.toLong()),it.toLong()) }
            Measured({ rows=bench.bulk.ownedHandleRecordsEcho(rows); rows.size.toLong() }, { rows.forEach { it.handle.close() } })
        }
        "callback_record" -> {
            require(size==1)
            var row=bench.bulk.HandleRecord(bench.bulk.inputHandleNew(1),1L)
            val callback=bench.bulk.OwnedRecordCallback { row }
            Measured({row=bench.bulk.callbackOwnedRecord(callback);1L},{row.handle.close()})
        }
        "callback_records" -> {
            var rows=List(size) {bench.bulk.HandleRecord(bench.bulk.inputHandleNew(it.toLong()),it.toLong())}
            val callback=bench.bulk.OwnedRecordsCallback {rows}
            Measured({rows=bench.bulk.callbackOwnedRecords(callback);rows.size.toLong()},{rows.forEach {it.handle.close()}})
        }
        "buffers" -> {
            var rows=List(size) {bench.bulk.BufferRecord(bench.bulk.inputHandleNew(it.toLong()),"x".repeat(64),ByteArray(64))}
            Measured({rows=bench.bulk.ownedBufferRecordsEcho(rows);check(rows.all {it.text.length==64 && it.bytes.size==64});rows.size.toLong()},{rows.forEach {it.handle.close()}})
        }
        "nested", "nested_many" -> {
            var rows=if (kind=="nested") listOf(emptyList(),List(size) {bench.bulk.inputHandleNew(it.toLong())},emptyList())
                     else List(size) {listOf(bench.bulk.inputHandleNew(it.toLong()))}
            Measured({ rows=bench.bulk.nestedOwnedHandlesEcho(rows); rows.sumOf { it.size }.toLong() }, { rows.forEach { group -> group.forEach {it.close()} } })
        }
        "control" -> Measured({bench.bulk.control(0)}, {})
        else -> error("unknown case")
    }
}
fun main(args:Array<String>) {
    if (args.contentEquals(arrayOf("--probe-instrumentation"))) {
        val enabled=Counters.start()
        Counters.stop()
        println("NATIVE_ALLOCATIONS,$enabled")
        return
    }
    val mode=args[0]; val kind=args[1]; val size=args[2].toInt()
    require(mode=="plain" || mode=="bulk")
    val operation=if(mode=="plain") makePlain(kind,size) else makeBulk(kind,size)
    val expected=if(kind=="control") 1L else size.toLong()
    try {
        repeat(1000) { check(operation.run()==expected) }
        repeat(10) { batch ->
            check(Counters.start()) { "native allocation instrumentation is disabled" }
            val result=operation.run()
            val counts=Counters.stop()
            check(result==expected && counts.size==3)
            println("NATIVE_ALLOC,$mode,$kind,$size,$batch,${counts[0]},${counts[1]},${counts[2]}")
        }
    } finally { operation.close() }
}
