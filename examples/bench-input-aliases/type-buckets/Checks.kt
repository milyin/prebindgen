package bench.types

fun checks() {
    val a=listOf(aNew(1),aNew(11));val b=listOf(bNew(2),bNew(12))
    val c=listOf(cNew(3),cNew(13));val d=listOf(dNew(4),dNew(14));val token=tokenNew(7)
    try {
        check(mixedMut(a,b,c,d)==60L)
        check(mixedShared(a+a,b+b,c+c,d+d,token)==127L)
        var failed=false
        mixedMut(a+a,b,c,d,JniErrorHandler {failed=true;0L});check(failed)
        // build.py/kotlin.py also assert the actual emitted type-bucket order.
        val input=(a+b+c+d+token).reversed()
        val bucketed=input.groupBy {it.handleType}.toSortedMap().values.flatMap {prepareInputHandles(it,true)}
        val flat=input.sortedWith(Comparator {x,y->compareHandleOrder(x,y)})
        check(bucketed==flat)
        val failure=java.util.concurrent.atomic.AtomicReference<Throwable?>()
        val threads=List(2) { index -> Thread {
            try {
                val aa=if(index==0)a else a.reversed();val bb=if(index==0)b else b.reversed()
                val cc=if(index==0)c else c.reversed();val dd=if(index==0)d else d.reversed()
                repeat(1000) {check(mixedMut(aa,bb,cc,dd)==60L)}
            } catch(t:Throwable){failure.compareAndSet(null,t)}
        }.apply {isDaemon=true;start()} }
        threads.forEach {it.join(10000);check(!it.isAlive){"lock ordering deadlocked"}}
        failure.get()?.let {throw it}
    } finally {token.close();(a+b+c+d).asReversed().forEach{it.close()}}
    check(live()==0L)
    println("CHECKS_OK")
}
fun main(){checks()}
