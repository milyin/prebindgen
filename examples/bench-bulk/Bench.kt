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
private fun rowPlain(x: Long) = bench.plain.Row(x, x+1, x+2, x+3, x+4, x+5, x+6, x+7)
private fun wideRowPlain(x: Long) = bench.plain.WideRow(
    x, x+1, x+2, x+3, x+4, x+5, x+6, x+7, x+8, x+9, x+10, x+11, x+12, x+13, x+14, x+15, x+16, x+17, x+18, x+19, x+20, x+21, x+22, x+23, x+24, x+25, x+26, x+27, x+28, x+29, x+30, x+31)
private fun checksumPlain(r: bench.plain.Row) = 1 * r.a + 2 * r.b + 3 * r.c + 4 * r.d + 5 * r.e + 6 * r.f + 7 * r.g + 8 * r.h
private fun checksumPlain(r: bench.plain.WideRow) =
    1 * r.f0 + 2 * r.f1 + 3 * r.f2 + 4 * r.f3 + 5 * r.f4 + 6 * r.f5 + 7 * r.f6 + 8 * r.f7 + 9 * r.f8 + 10 * r.f9 + 11 * r.f10 + 12 * r.f11 + 13 * r.f12 + 14 * r.f13 + 15 * r.f14 + 16 * r.f15 + 17 * r.f16 + 18 * r.f17 + 19 * r.f18 + 20 * r.f19 + 21 * r.f20 + 22 * r.f21 + 23 * r.f22 + 24 * r.f23 + 25 * r.f24 + 26 * r.f25 + 27 * r.f26 + 28 * r.f27 + 29 * r.f28 + 30 * r.f29 + 31 * r.f30 + 32 * r.f31
private fun casesPlain(sizes: List<Int>, payloads: List<Int>, callbacks: LongArray): Pair<List<Case>, AutoCloseable> {
    val handler = bench.plain.handlerNew(bench.plain.RowsCallback { rows ->
        callbacks[0]++; checksumPlain(rows)
    })
    val rowReturn = bench.plain.rowReturnHandlerNew(bench.plain.RowReturnCallback { seed ->
        callbacks[0]++; rowPlain(seed)
    })
    val wideReturn = bench.plain.wideRowReturnHandlerNew(bench.plain.WideRowReturnCallback { seed ->
        callbacks[0]++; wideRowPlain(seed)
    })
    val wide = wideRowPlain(BASE)
    val one = bench.plain.Row(BASE, BASE + 1, BASE + 2, BASE + 3, BASE + 4, BASE + 5, BASE + 6, BASE + 7)
    val cases = mutableListOf(
        Case("control", 1, BASE + 1, 1, Operation { i -> bench.plain.control(BASE + (i and MASK)) }),
        Case("scalar", 1, BASE + 1, 1, Operation { i -> bench.plain.scalar(BASE + (i and MASK)) }),
        Case("record_input", 1, rowsTotal(1), 0, Operation { _ -> bench.plain.rowSum(one) }),
        Case("record_return", 1, 36 * BASE + 168, 36, Operation { i -> checksumPlain(bench.plain.rowMake(BASE + (i and MASK))) }),
        Case("wide_record_input", 1, 528 * BASE + 10912, 0, Operation { _ -> bench.plain.wideRowSum(wide) }),
        Case("wide_record_return", 1, 528 * BASE + 10912, 528, Operation { i -> checksumPlain(bench.plain.wideRowMake(BASE + (i and MASK))) }),
        Case("callback_record_return", 1, 36 * BASE + 168, 36, Operation { i -> bench.plain.rowReturnFire(rowReturn, BASE + (i and MASK)) }, true),
        Case("callback_wide_record_return", 1, 528 * BASE + 10912, 528, Operation { i -> bench.plain.wideRowReturnFire(wideReturn, BASE + (i and MASK)) }, true)
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
    return cases to AutoCloseable {
        try { handler.close() } finally {
            try { rowReturn.close() } finally { wideReturn.close() }
        }
    }
}

private fun checksumBulk(rows: List<bench.bulk.Row>): Long {
    var sum = 0L
    for (r in rows) sum += r.a + r.b + r.c + r.d + r.e + r.f + r.g + r.h
    return sum
}
private fun rowBulk(x: Long) = bench.bulk.Row(x, x+1, x+2, x+3, x+4, x+5, x+6, x+7)
private fun wideRowBulk(x: Long) = bench.bulk.WideRow(
    x, x+1, x+2, x+3, x+4, x+5, x+6, x+7, x+8, x+9, x+10, x+11, x+12, x+13, x+14, x+15, x+16, x+17, x+18, x+19, x+20, x+21, x+22, x+23, x+24, x+25, x+26, x+27, x+28, x+29, x+30, x+31)
private fun checksumBulk(r: bench.bulk.Row) = 1 * r.a + 2 * r.b + 3 * r.c + 4 * r.d + 5 * r.e + 6 * r.f + 7 * r.g + 8 * r.h
private fun checksumBulk(r: bench.bulk.WideRow) =
    1 * r.f0 + 2 * r.f1 + 3 * r.f2 + 4 * r.f3 + 5 * r.f4 + 6 * r.f5 + 7 * r.f6 + 8 * r.f7 + 9 * r.f8 + 10 * r.f9 + 11 * r.f10 + 12 * r.f11 + 13 * r.f12 + 14 * r.f13 + 15 * r.f14 + 16 * r.f15 + 17 * r.f16 + 18 * r.f17 + 19 * r.f18 + 20 * r.f19 + 21 * r.f20 + 22 * r.f21 + 23 * r.f22 + 24 * r.f23 + 25 * r.f24 + 26 * r.f25 + 27 * r.f26 + 28 * r.f27 + 29 * r.f28 + 30 * r.f29 + 31 * r.f30 + 32 * r.f31
private fun casesBulk(sizes: List<Int>, payloads: List<Int>, callbacks: LongArray): Pair<List<Case>, AutoCloseable> {
    val handler = bench.bulk.handlerNew(bench.bulk.RowsCallback { rows ->
        callbacks[0]++; checksumBulk(rows)
    })
    val rowReturn = bench.bulk.rowReturnHandlerNew(bench.bulk.RowReturnCallback { seed ->
        callbacks[0]++; rowBulk(seed)
    })
    val wideReturn = bench.bulk.wideRowReturnHandlerNew(bench.bulk.WideRowReturnCallback { seed ->
        callbacks[0]++; wideRowBulk(seed)
    })
    val wide = wideRowBulk(BASE)
    val one = bench.bulk.Row(BASE, BASE + 1, BASE + 2, BASE + 3, BASE + 4, BASE + 5, BASE + 6, BASE + 7)
    val cases = mutableListOf(
        Case("control", 1, BASE + 1, 1, Operation { i -> bench.bulk.control(BASE + (i and MASK)) }),
        Case("scalar", 1, BASE + 1, 1, Operation { i -> bench.bulk.scalar(BASE + (i and MASK)) }),
        Case("record_input", 1, rowsTotal(1), 0, Operation { _ -> bench.bulk.rowSum(one) }),
        Case("record_return", 1, 36 * BASE + 168, 36, Operation { i -> checksumBulk(bench.bulk.rowMake(BASE + (i and MASK))) }),
        Case("wide_record_input", 1, 528 * BASE + 10912, 0, Operation { _ -> bench.bulk.wideRowSum(wide) }),
        Case("wide_record_return", 1, 528 * BASE + 10912, 528, Operation { i -> checksumBulk(bench.bulk.wideRowMake(BASE + (i and MASK))) }),
        Case("callback_record_return", 1, 36 * BASE + 168, 36, Operation { i -> bench.bulk.rowReturnFire(rowReturn, BASE + (i and MASK)) }, true),
        Case("callback_wide_record_return", 1, 528 * BASE + 10912, 528, Operation { i -> bench.bulk.wideRowReturnFire(wideReturn, BASE + (i and MASK)) }, true)
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
    return cases to AutoCloseable {
        try { handler.close() } finally {
            try { rowReturn.close() } finally { wideReturn.close() }
        }
    }
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
    val selectedNames = args[7].split(',').toSet()
    val selectedCases = allCases.filter { it.name in selectedNames }
    require(selectedCases.isNotEmpty() && selectedCases.map { it.name }.toSet() == selectedNames)
    val ordered = if (trial % 2 == 0) selectedCases else selectedCases.reversed()
    val shift = (trial / 2) % ordered.size
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
