package bench.lock

import com.sun.management.ThreadMXBean
import java.lang.management.ManagementFactory

private class Handle(value: Long) : NativeHandle(value) {
    override val handleType = 1L
    override fun freeNative(raw: Long) { }
}
@Volatile private var published: List<NativeHandle> = emptyList()
@Volatile private var sink: Any? = null
@Volatile private var bodyToken: Any = Any()
private fun fixedBody(): Any = bodyToken

fun main(args: Array<String>) {
    val mode = args[0]; val variant = args[1]; val size = args[2].toInt()
    val operation: (List<NativeHandle>, () -> Any) -> Any = when(variant) {
        "base" -> ::withOrderedHandleLocks
        "2" -> ::group2; "4" -> ::group4; "8" -> ::group8; "16" -> ::group16; "32" -> ::group32
        else -> error("unknown variant")
    }
    val token = bodyToken
    val body = { fixedBody() }
    val handles = List(size) { Handle((it + 1) * 2L) }
    published = handles
    if (mode == "limit") {
        if (args[3] == "warm") {
            val short = handles.take(64)
            repeat(20000) { operation(short, body) }
        }
        var success = false
        try { success = operation(handles) { for (h in handles) check(Thread.holdsLock(h)); token } === token }
        catch (_: StackOverflowError) { }
        for (h in handles) check(!Thread.holdsLock(h))
        println("LIMIT,$variant,$size,${args[3]},$success")
        return
    }
    val bean = ManagementFactory.getThreadMXBean() as ThreadMXBean
    bean.isThreadAllocatedMemoryEnabled = true
    val thread = Thread.currentThread().id
    fun sample(iterations: Int): Pair<Long, Long> {
        val beforeBytes = bean.getThreadAllocatedBytes(thread)
        val start = System.nanoTime()
        var result: Any = token
        repeat(iterations) { result = operation(handles, body) }
        val nanos = System.nanoTime() - start
        val bytes = bean.getThreadAllocatedBytes(thread) - beforeBytes
        sink = result
        check(result === token)
        return nanos to bytes
    }
    val iterations = (300000 / (size + 1)).coerceIn(20, 300000)
    repeat(12) { sample(iterations) }
    repeat(11) { batch ->
        val (nanos, bytes) = sample(iterations)
        println("LOCK,$variant,$size,$batch,$iterations,$nanos,$bytes")
    }
}
