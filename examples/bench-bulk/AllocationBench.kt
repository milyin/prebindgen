package bench.allocations
internal object Counters {
    init { System.loadLibrary("bulk_bench") }
    @JvmStatic external fun start(): Boolean
    @JvmStatic external fun stop(): LongArray
}
private class Measured(val run: () -> Long, val close: () -> Unit)
private fun makePlain(kind: String, size: Int): Measured {
    return when (kind) {
        "flat" -> {
            var rows=List(size) { bench.plain.inputHandleNew(it.toLong()) }
            Measured({ rows=bench.plain.ownedHandlesEcho(rows); rows.size.toLong() }, { rows.forEach { it.close() } })
        }
        "records" -> {
            var rows=List(size) { bench.plain.HandleRecord(bench.plain.inputHandleNew(it.toLong()),it.toLong()) }
            Measured({ rows=bench.plain.ownedHandleRecordsEcho(rows); rows.size.toLong() }, { rows.forEach { it.handle.close() } })
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
        "flat" -> {
            var rows=List(size) { bench.bulk.inputHandleNew(it.toLong()) }
            Measured({ rows=bench.bulk.ownedHandlesEcho(rows); rows.size.toLong() }, { rows.forEach { it.close() } })
        }
        "records" -> {
            var rows=List(size) { bench.bulk.HandleRecord(bench.bulk.inputHandleNew(it.toLong()),it.toLong()) }
            Measured({ rows=bench.bulk.ownedHandleRecordsEcho(rows); rows.size.toLong() }, { rows.forEach { it.handle.close() } })
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
