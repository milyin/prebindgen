import bench.bulk.*
import java.util.Random
// Use fixed invocation warm-up so both native variants reach comparable JVM compilation.
fun main() {
    val n=512
    val permutation=(0 until n).toMutableList().also {java.util.Collections.shuffle(it,Random(42))}
    for(returned in listOf(false,true)) for(many in listOf(false,true)) {
        var warm=if(many)List(8){listOf(inputHandleNew(1))}else listOf(emptyList(),List(8){inputHandleNew(1)},emptyList())
        val callback=OwnedNestedCallback {warm}
        repeat(20_000){warm=if(returned)callbackNestedOwnedHandles(callback)else nestedOwnedHandlesEcho(warm)}
        warm.forEach {group->group.forEach {it.close()}}
    }
    println("FIXED_WARMUP_DONE")
    for(shape in listOf("one","many")) for(order in listOf("ascending","descending","shuffled")) for(returned in listOf(false,true)) {
        var rows=if(shape=="one")listOf(emptyList(),List(n){inputHandleNew(1)},emptyList()) else List(n){listOf(inputHandleNew(1))}
        val callback=OwnedNestedCallback {rows}
        try {
            repeat(16) {batch->
                var nanos=0L;var iterations=0
                while(nanos<30_000_000L || iterations<16) {
                    // Normalize pointer rank outside the timed native-wrapper call.
                    val sorted=rows.flatten().sortedBy {it.raw}
                    val ordered=when(order){"descending"->sorted.asReversed();"shuffled"->permutation.map {sorted[it]};else->sorted}
                    rows=if(shape=="one")listOf(emptyList(),ordered,emptyList())else ordered.map {listOf(it)}
                    val begin=System.nanoTime()
                    rows=if(returned)callbackNestedOwnedHandles(callback) else nestedOwnedHandlesEcho(rows)
                    nanos+=System.nanoTime()-begin
                    check(rows.sumOf {it.size}==n)
                    iterations++
                }
                if(batch>=7)println("ADDRESS,$shape,$order,$returned,${batch-7},$iterations,$nanos")
            }
        } finally {rows.forEach {group->group.forEach {it.close()}}}
    }
}
