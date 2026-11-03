package bench.outputowners

import bench.plain.*
import bench.allocations.Counters
import com.sun.management.ThreadMXBean
import java.lang.management.ManagementFactory

private class Operation(val run: () -> Long, val close: () -> Unit = {})
@Volatile private var observed = 0L

private fun operation(kind: String, size: Int): Operation = when (kind) {
    "flat", "fold_flat" -> {
        var rows = List(size) { inputHandleNew(1000000L + it) }
        val fold: (MutableList<InputHandle>, InputHandle) -> MutableList<InputHandle> = { list, value -> list.add(value); list }
        Operation({
            rows = if (kind == "flat") ownedHandlesEcho(rows)
                   else ownedHandlesEchoFold(rows, ArrayList(size), fold = fold)
            rows.size.toLong()
        }, { rows.forEach { it.close() } })
    }
    "nested", "fold_nested", "fold_wide_nested" -> {
        var rows: List<List<InputHandle>> = if (kind == "fold_wide_nested")
            listOf(List(size) { inputHandleNew(1000000L + it) })
            else List(size) { listOf(inputHandleNew(1000000L + it)) }
        val fold: (MutableList<List<InputHandle>>, List<InputHandle>) -> MutableList<List<InputHandle>> = { list, value -> list.add(value); list }
        Operation({
            rows = if (kind == "nested") nestedOwnedHandlesEcho(rows)
                   else nestedOwnedHandlesEchoFold(rows, ArrayList(size), fold = fold)
            rows.sumOf { it.size }.toLong()
        }, { rows.forEach { group -> group.forEach { it.close() } } })
    }
    "primitive_array", "primitive_fold" -> {
        val fold: (Long, Row) -> Long = { a, row -> a + row.a }
        Operation({
            val sum = if (kind == "primitive_array") rowsMake(size.toLong(), 1000000).sumOf { it.a }
                      else rowsForReduceFold(size.toLong(), 1000000, 0L, fold = fold)
            check(sum == size * 1000000L + size.toLong() * (size - 1) / 2)
            size.toLong()
        })
    }
    "control" -> Operation({ control(0) })
    else -> error("unknown case: $kind")
}

fun main(args: Array<String>) {
    val kind = args[0]
    val size = args[1].toInt()
    val count = args[2] == "counts"
    val op = operation(kind, size)
    val expected = if (kind == "control") 1L else size.toLong()
    try {
        repeat(if (count) 100 else 16) { check(op.run() == expected) }
        if (count) {
            repeat(10) { batch ->
                check(Counters.start())
                val result = op.run()
                val counts = Counters.stop()
                check(result == expected)
                println("COUNT,$kind,$size,$batch,${counts[0]},${counts[1]},${counts[2]}")
            }
            return
        }
        val bean = ManagementFactory.getThreadMXBean() as ThreadMXBean
        bean.isThreadAllocatedMemoryEnabled = true
        bean.isThreadCpuTimeEnabled = true
        val thread = Thread.currentThread().id
        val compiler = ManagementFactory.getCompilationMXBean()
        val gc = ManagementFactory.getGarbageCollectorMXBeans()
        fun sample(n: Int): LongArray {
            val bytes = bean.getThreadAllocatedBytes(thread)
            val cpu = bean.currentThreadCpuTime
            val begin = System.nanoTime()
            var result = 0L
            repeat(n) { result += op.run() }
            val elapsed = System.nanoTime() - begin
            val usedCpu = bean.currentThreadCpuTime - cpu
            val allocated = bean.getThreadAllocatedBytes(thread) - bytes
            check(result == expected * n)
            observed = result
            return longArrayOf(elapsed, usedCpu, allocated)
        }
        var iterations = 16
        while (sample(iterations)[0] < 20000000 && iterations < 1048576) iterations *= 2
        val warmStart = System.nanoTime()
        var quietStart = warmStart
        var compilation = compiler.totalCompilationTime
        var settled = false
        while (System.nanoTime() - warmStart < 15000000000L) {
            sample(iterations)
            val now = System.nanoTime()
            val current = compiler.totalCompilationTime
            if (current != compilation) { quietStart = now; compilation = current }
            if (now - warmStart >= 2000000000L && now - quietStart >= 500000000L) { settled = true; break }
        }
        println("WARMUP,$settled,${System.nanoTime()-warmStart},$iterations")
        repeat(12) { batch ->
            val collections = gc.sumOf { it.collectionCount }
            val value = sample(iterations)
            println("TIME,$kind,$size,$batch,$iterations,${value[0]},${value[1]},${value[2]},${gc.sumOf { it.collectionCount }-collections}")
        }
    } finally { op.close() }
}
