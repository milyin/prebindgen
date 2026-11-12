package io.prebindgen.covertest
import io.prebindgen.covertest.payloadenums.*
import kotlin.system.measureNanoTime

// Run explicitly with the release native library: TestKt payload-bench.
// Reused scalar objects isolate transport; lists alternate both payload shapes.
private var payloadBenchSink=0uL
fun benchmarkPayloadEnums() {
    val allocationBean=java.lang.management.ManagementFactory.getThreadMXBean() as com.sun.management.ThreadMXBean
    check(allocationBean.isThreadAllocatedMemorySupported)
    allocationBean.isThreadAllocatedMemoryEnabled=true
    val thread=Thread.currentThread().id
    val cases=payloadBenchCases()
    repeat(4) { for((_,_,call) in cases) repeat(10_000){payloadBenchSink=call()} }
    val samples=cases.associate { it.first to mutableListOf<Double>() }
    val allocations=cases.associate { it.first to mutableListOf<Double>() }
    repeat(9) { round ->
        for(index in cases.indices.map { (it+round)%cases.size }) {
            val (name,size,call)=cases[index]
            val iterations=if(size==1) 300_000 else 10_000
            val before=allocationBean.getThreadAllocatedBytes(thread)
            val elapsed=measureNanoTime {var sum=0uL;repeat(iterations){sum=sum xor call()};payloadBenchSink=sum}
            allocations.getValue(name).add((allocationBean.getThreadAllocatedBytes(thread)-before).toDouble()/iterations)
            samples.getValue(name).add(elapsed.toDouble()/iterations)
        }
    }
    for((name,values) in samples) println("$name: median ${values.sorted()[values.size/2]} ns/call, ${allocations.getValue(name).sorted()[values.size/2]} JVM bytes/call")
}

internal fun payloadBenchCases():List<Triple<String,Int,()->ULong>> {
    val single=WireBench.Pair(11uL,23uL)
    val batch=List(256) { i -> if(i%2==0) WireBench.Count(i.toULong()) else WireBench.Pair(i.toULong(),1uL) }
    return listOf(
        Triple("manual scalar tag/fields",1, { wireManualTotal(1,0uL,11uL,23uL) }),
        Triple("sealed scalar",1, { wireBenchTotal(single) }),
        Triple("bulk scalar",1, { wireBenchScalarBulk(single) }),
        Triple("manual callback (two owners)",2, { wireManualOwnedBench({_,_->1L}).toULong() }),
        Triple("enum callback (two owners)",2, { wireOwnedBench({1L}).toULong() }),
        Triple("ordinary enum rows (256)",256, { wireBenchObjects(batch) }),
        Triple("bulk enum rows (256)",256, { wireBenchBulk(batch) })
    )
}
