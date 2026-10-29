import com.sun.management.ThreadMXBean
import java.lang.management.ManagementFactory

private const val BASE = 1000000L
private const val MASK = 1023
@Volatile private var observed = 0L
private fun interface Operation { fun run(index: Int): Long }
private data class Case(val name: String, val size: Int, val constant: Long, val coefficient: Long,
                        val operation: Operation, val callback: Boolean = false) {
    fun expected(iterations: Int): Long {
        val chunks = iterations / (MASK + 1)
        val tail = (iterations % (MASK + 1)).toLong()
        val indices = chunks.toLong() * MASK * (MASK + 1) / 2 + tail * (tail - 1) / 2
        return constant * iterations + coefficient * indices
    }
}
private data class Sample(val nanos: Long, val allocated: Long)
private fun rowsTotal(size: Int) = size * (8 * BASE + 28) + 4L * size * (size - 1)

private fun checksumPlain(rows: List<bench.plain.Row>): Long {
    var sum = 0L
    for (r in rows) sum += r.a + r.b + r.c + r.d + r.e + r.f + r.g + r.h
    return sum
}
private fun casesPlain(sizes: List<Int>, payloads: List<Int>, callbacks: LongArray): Pair<List<Case>, AutoCloseable> {
    val handler = bench.plain.handlerNew(bench.plain.RowsCallback { rows ->
        callbacks[0]++; checksumPlain(rows)
    })
    val one = bench.plain.Row(BASE, BASE + 1, BASE + 2, BASE + 3, BASE + 4, BASE + 5, BASE + 6, BASE + 7)
    val cases = mutableListOf(
        Case("scalar", 1, BASE + 1, 1, Operation { i -> bench.plain.scalar(BASE + (i and MASK)) }),
        Case("record_input", 1, rowsTotal(1), 0, Operation { _ -> bench.plain.rowSum(one) })
    )
    for (size in sizes) {
        val numbers = List(size) { BASE + it }
        val rows = List(size) { i -> val x = BASE + i; bench.plain.Row(x, x+1, x+2, x+3, x+4, x+5, x+6, x+7) }
        cases.add(Case("numbers_input", size, size * BASE + size.toLong() * (size - 1) / 2, 0,
            Operation { _ -> bench.plain.numbersSum(numbers) }))
        cases.add(Case("records_input", size, rowsTotal(size), 0,
            Operation { _ -> bench.plain.rowsSum(rows) }))
        cases.add(Case("records_output", size, rowsTotal(size), 8L * size,
            Operation { i -> checksumPlain(bench.plain.rowsMake(size.toLong(), BASE + (i and MASK))) }))
        val reduced = longArrayOf(0)
        val fold: (Unit, bench.plain.Row) -> Unit = { _, r -> reduced[0] += r.a + r.b + r.c + r.d + r.e + r.f + r.g + r.h }
        cases.add(Case("records_reduce", size, rowsTotal(size), 8L * size,
            Operation { i ->
                reduced[0] = 0
                bench.plain.rowsForReduceFold(size.toLong(), BASE + (i and MASK), Unit, fold = fold)
                reduced[0]
            }))
        cases.add(Case("records_roundtrip", size, rowsTotal(size), 0,
            Operation { _ -> checksumPlain(bench.plain.rowsEcho(rows)) }))
        cases.add(Case("callback_records", size, rowsTotal(size), 8L * size,
            Operation { i -> bench.plain.fire(handler, size.toLong(), BASE + (i and MASK)) }, true))
    }
    for (size in payloads) {
        val packet = bench.plain.Packet(one, ByteArray(size) { (it and 255).toByte() })
        val full = size / 256
        val tail = (size % 256).toLong()
        val expected = rowsTotal(1) + full.toLong() * 32640 + tail * (tail - 1) / 2
        cases.add(Case("packet_input", size, expected, 0, Operation { _ -> bench.plain.packetSum(packet) }))
    }
    return cases to handler
}

private fun checksumBulk(rows: List<bench.bulk.Row>): Long {
    var sum = 0L
    for (r in rows) sum += r.a + r.b + r.c + r.d + r.e + r.f + r.g + r.h
    return sum
}
private fun casesBulk(sizes: List<Int>, payloads: List<Int>, callbacks: LongArray): Pair<List<Case>, AutoCloseable> {
    val handler = bench.bulk.handlerNew(bench.bulk.RowsCallback { rows ->
        callbacks[0]++; checksumBulk(rows)
    })
    val one = bench.bulk.Row(BASE, BASE + 1, BASE + 2, BASE + 3, BASE + 4, BASE + 5, BASE + 6, BASE + 7)
    val cases = mutableListOf(
        Case("scalar", 1, BASE + 1, 1, Operation { i -> bench.bulk.scalar(BASE + (i and MASK)) }),
        Case("record_input", 1, rowsTotal(1), 0, Operation { _ -> bench.bulk.rowSum(one) })
    )
    for (size in sizes) {
        val numbers = List(size) { BASE + it }
        val rows = List(size) { i -> val x = BASE + i; bench.bulk.Row(x, x+1, x+2, x+3, x+4, x+5, x+6, x+7) }
        cases.add(Case("numbers_input", size, size * BASE + size.toLong() * (size - 1) / 2, 0,
            Operation { _ -> bench.bulk.numbersSum(numbers) }))
        cases.add(Case("records_input", size, rowsTotal(size), 0,
            Operation { _ -> bench.bulk.rowsSum(rows) }))
        cases.add(Case("records_output", size, rowsTotal(size), 8L * size,
            Operation { i -> checksumBulk(bench.bulk.rowsMake(size.toLong(), BASE + (i and MASK))) }))
        val reduced = longArrayOf(0)
        cases.add(Case("records_reduce", size, rowsTotal(size), 8L * size,
            Operation { i ->
                reduced[0] = 0
                for (r in bench.bulk.rowsForReduce(size.toLong(), BASE + (i and MASK)))
                    reduced[0] += r.a + r.b + r.c + r.d + r.e + r.f + r.g + r.h
                reduced[0]
            }))
        cases.add(Case("records_roundtrip", size, rowsTotal(size), 0,
            Operation { _ -> checksumBulk(bench.bulk.rowsEcho(rows)) }))
        cases.add(Case("callback_records", size, rowsTotal(size), 8L * size,
            Operation { i -> bench.bulk.fire(handler, size.toLong(), BASE + (i and MASK)) }, true))
    }
    for (size in payloads) {
        val packet = bench.bulk.Packet(one, ByteArray(size) { (it and 255).toByte() })
        val full = size / 256
        val tail = (size % 256).toLong()
        val expected = rowsTotal(1) + full.toLong() * 32640 + tail * (tail - 1) / 2
        cases.add(Case("packet_input", size, expected, 0, Operation { _ -> bench.bulk.packetSum(packet) }))
    }
    return cases to handler
}

fun main(args: Array<String>) {
    val mode = args[0]
    require(mode == "plain" || mode == "bulk")
    val sizes = args[1].split(',').map { it.toInt() }
    val warmup = args[2].toInt()
    val batches = args[3].toInt()
    val targetNanos = args[4].toLong() * 1000000
    val trial = args[5].toInt()
    val payloads = args[6].split(',').map { it.toInt() }
    require(sizes.all { it >= 0 } && warmup > 0 && batches > 0 && targetNanos > 0)
    val bean = ManagementFactory.getThreadMXBean() as ThreadMXBean
    check(bean.isThreadAllocatedMemorySupported)
    bean.isThreadAllocatedMemoryEnabled = true
    val thread = Thread.currentThread().id
    val callbacks = longArrayOf(0)
    val (allCases, handler) = if (mode == "plain") casesPlain(sizes, payloads, callbacks) else casesBulk(sizes, payloads, callbacks)
    val ordered = if (trial % 2 == 0) allCases else allCases.reversed()
    val shift = trial * 7 % ordered.size
    val cases = ordered.drop(shift) + ordered.take(shift)
    fun sample(case: Case, iterations: Int): Sample {
        val beforeCallbacks = callbacks[0]
        val beforeBytes = bean.getThreadAllocatedBytes(thread)
        val start = System.nanoTime()
        var sum = 0L
        var i = 0
        while (i < iterations) { sum += case.operation.run(i); i++ }
        val elapsed = System.nanoTime() - start
        val allocated = bean.getThreadAllocatedBytes(thread) - beforeBytes
        observed = observed xor sum
        check(sum == case.expected(iterations)) { "${case.name}/${case.size}: incorrect checksum $sum" }
        check(callbacks[0] - beforeCallbacks == if (case.callback) iterations.toLong() else 0L)
        return Sample(elapsed, allocated)
    }
    try {
        repeat(2) { for (case in cases) sample(case, 256) }
        for (case in cases) {
            var iterations = (10000 / (case.size + 1)).coerceIn(16, 10000)
            fun calibrate(nanos: Long): Int = (iterations * (targetNanos.toDouble() / nanos).coerceIn(0.25, 8.0)).toInt().coerceIn(16, 5000000)
            repeat(3) { iterations = calibrate(sample(case, iterations).nanos) }
            var last = 1L
            repeat(warmup) { last = sample(case, iterations).nanos }
            iterations = calibrate(last)
            repeat(batches) { batch ->
                val s = sample(case, iterations)
                println("BULK_BENCH,$mode,${case.name},${case.size},$batch,$iterations,${s.nanos},${s.allocated}")
            }
        }
        println("SINK,$observed")
    } finally { handler.close() }
}
