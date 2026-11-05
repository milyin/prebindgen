package bench.gc

import io.prebindgen.covertest.gc.*
import java.lang.management.ManagementFactory

private fun interface Operation { fun run(): Long }

fun main(args: Array<String>) {
    val case = args.single()
    val read = case.contains("read")
    val managed = case.startsWith("gc")
    val plain = if (read && !managed) plainResourceNew(101uL, 42uL) else null
    val gc = if (read && managed) gcResourceNew(101uL, 42uL) else null
    val operation = when {
        read && managed -> Operation { gcResourceValue(gc).toLong() }
        read -> Operation { plainResourceValue(plain).toLong() }
        managed -> Operation { gcResourceNew(101uL, 42uL).close(); 42L }
        else -> Operation { plainResourceNew(101uL, 42uL).close(); 42L }
    }
    val warmup = if (read) 200_000 else 10_000
    var checksum = 0L
    repeat(10) { repeat(warmup) { checksum += operation.run() } }
    val bean = ManagementFactory.getThreadMXBean() as com.sun.management.ThreadMXBean
    check(bean.isThreadAllocatedMemorySupported)
    bean.isThreadAllocatedMemoryEnabled = true
    val thread = Thread.currentThread().id
    val count = if (read) 500_000 else 20_000
    repeat(10) { batch ->
        val bytesBefore = bean.getThreadAllocatedBytes(thread)
        val before = System.nanoTime()
        repeat(count) { checksum += operation.run() }
        val elapsed = System.nanoTime() - before
        val bytes = bean.getThreadAllocatedBytes(thread) - bytesBefore
        println("$case,$batch,$count,$elapsed,$bytes")
    }
    plain?.close()
    gc?.close()
    check(checksum == 42L * (10L * warmup + 10L * count))
}
