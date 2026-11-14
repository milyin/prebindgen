package io.prebindgen.returnbench

import io.prebindgen.covertest.primitives.*
import io.prebindgen.covertest.summaryNew
import io.prebindgen.covertest.storage.storageNew
import java.lang.management.ManagementFactory
import com.sun.management.ThreadMXBean

@Volatile private var observed = 0L

fun main() {
    val bean = ManagementFactory.getThreadMXBean() as ThreadMXBean
    check(bean.isThreadAllocatedMemorySupported)
    bean.isThreadAllocatedMemoryEnabled = true
    val thread = Thread.currentThread().id
    val iterations = 200000
    fun scalar(): Long {
        var sum = 0L
        repeat(iterations) { sum = sum xor primitiveI64((it.toLong() + 1000) shl 10) }
        return sum
    }
    fun unit(): Long {
        repeat(iterations) { primitiveUnit() }
        return 0L
    }
    fun expanded(): Long {
        var sum = 0L
        repeat(iterations) { sum = sum xor summaryNew(it.toLong() + 1000, 1.25) { a, b -> a xor b.toRawBits() } }
        return sum
    }
    fun handle(): Long {
        repeat(iterations) { storageNew().close() }
        return 0L
    }
    for ((name, run) in listOf("scalar" to ::scalar, "unit" to ::unit, "expanded" to ::expanded, "handle" to ::handle)) {
        repeat(3) { observed = run() }
        val samples = (0..4).map {
            val allocated = bean.getThreadAllocatedBytes(thread)
            val start = System.nanoTime()
            observed = run()
            val elapsed = System.nanoTime() - start
            val bytes = bean.getThreadAllocatedBytes(thread) - allocated
            elapsed.toDouble() / iterations to bytes.toDouble() / iterations
        }
        println("RETURN_BENCH $name ns=${samples.map { it.first }.sorted()[2]} bytes=${samples.map { it.second }.sorted()[2]}")
    }
}
