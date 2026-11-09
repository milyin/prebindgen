package io.prebindgen.covertest

import io.prebindgen.covertest.typeinputs.*
import java.lang.management.ManagementFactory
import java.util.concurrent.atomic.AtomicReference

private fun preflightRejected(body:()->Unit) {
    val failure=runCatching(body).exceptionOrNull()
    check(failure is IllegalArgumentException) { "expected input rejection, got $failure" }
}
fun checkTypeInputs() {
    check(typeInputLive()==0L)
    val a=typeInputANew(11);val a2=typeInputANew(13)
    val b=typeInputBNew(17);val b2=typeInputBNew(19);val control=typeInputControlNew(23)
    try {
        check(typeInputFixed(a,b,a,b, {
            Thread.holdsLock(a) && Thread.holdsLock(b)
        })==56L)
        check(typeInputSharedLists(listOf(a,a),listOf(b,b),control)==79L)
        check(typeInputMixed(List(1024){a},listOf(a2))==1024*11L+13)
        val before=typeInputConversions()
        preflightRejected {typeInputFixedMut(a,a,b)}
        preflightRejected {typeInputMutLists(listOf(a,a2,a),listOf(b,b2))}
        preflightRejected {typeInputMutLists(listOf(a,a2),listOf(b,b2,b))}
        preflightRejected {typeInputMixed(listOf(a,a2),listOf(a2))}
        check(typeInputConversions()==before) {"invalid aliases entered native conversion"}
        check(listOf(a,a2,b,b2,control).none{it.isClosed})
    }finally{listOf(a,a2,b,b2,control).forEach{it.close()}}

    val captured=typeInputANew(47);val spare=typeInputANew(53);val capturedB=typeInputBNew(59)
    var reads=0
    TypeInputRow.readA={ original -> if(reads++==0)original else spare }
    try{check(typeInputRow(TypeInputRow(captured,capturedB))==106L)}
    finally{TypeInputRow.readA=null}
    check(reads==1 && captured.isClosed && capturedB.isClosed && !spare.isClosed)
    spare.close();check(typeInputLive()==0L)

    val optionalA=typeInputANew(61);val optionalB=typeInputBNew(67)
    var optionReads=0
    TypeInputHolder.readRow={ original -> if(optionReads++==0)original else null }
    try{check(typeInputHolder(TypeInputHolder(TypeInputRow(optionalA,optionalB)))==128L)}
    finally{TypeInputHolder.readRow=null}
    check(optionReads==1 && optionalA.isClosed && optionalB.isClosed && typeInputLive()==0L)
    check(typeInputHolder(TypeInputHolder(null))==-1L)

    fun rows()=listOf(TypeInputRow(typeInputANew(1),typeInputBNew(2)),TypeInputRow(typeInputANew(3),typeInputBNew(4)))
    fun handles(rows:List<TypeInputRow>)=rows.flatMap{listOf(it.a,it.b)}
    for(mode in 0..4){
        val values=rows();val owners=handles(values)
        val sum=when(mode){
            0->typeInputRows(values)
            1->typeInputRowsPlain(values)
            2->typeInputNested(listOf(emptyList(),values.take(1),values.drop(1)))
            3->typeInputCallback(TypeInputCallback {values})
            else->typeInputNestedCallback(TypeInputNestedCallback {listOf(values.take(1),emptyList(),values.drop(1))})
        }
        check(sum==10L && owners.all{it.isClosed} && typeInputLive()==0L)
        val duplicate=rows();val aliased=listOf(duplicate[0],TypeInputRow(duplicate[0].a,duplicate[1].b))
        preflightRejected{when(mode){
            0->typeInputRows(aliased)
            1->typeInputRowsPlain(aliased)
            2->typeInputNested(aliased.map{listOf(it)})
            3->typeInputCallback(TypeInputCallback {aliased})
            else->typeInputNestedCallback(TypeInputNestedCallback {aliased.map{listOf(it)}})
        }}
        if(mode<3)check(handles(duplicate).none{it.isClosed})
        else check(listOf(duplicate[0].a,duplicate[0].b,duplicate[1].b).all{it.isClosed})
        handles(duplicate).forEach{it.close()};check(typeInputLive()==0L)
    }
    // Failure after an owned field was acquired must drop it, report its
    // consumed prefix, and keep ordinary unread owners with their caller.
    for(callback in listOf(false,true)){
        val values=rows();values[1].b.close()
        preflightRejected{if(callback)typeInputCallback(TypeInputCallback {values}) else typeInputRows(values)}
        check(values[0].a.isClosed && values[0].b.isClosed && values[1].b.isClosed)
        if(callback)check(handles(values).all{it.isClosed})
        else if(!values[1].a.isClosed)check(typeInputARead(values[1].a)==3L)
        handles(values).forEach{it.close()};check(typeInputLive()==0L)
    }
    check(typeInputOptional(null)==-1L && typeInputOptional(emptyList())==0L)
    val optional=typeInputANew(31);check(typeInputOptional(listOf(optional))==31L && optional.isClosed)
    val array=listOf(typeInputANew(37),typeInputANew(41));check(typeInputArray(array)==78L && array.all{it.isClosed})
    val short=listOf(typeInputANew(43));preflightRejected{typeInputArray(short)};check(!short[0].isClosed);short[0].close()
    check(typeInputLive()==0L)
    checkHandleOrderRoutes()
    println("type-aware inputs PASS: shared/exclusive types, records, nested/plain/bulk arrays, callbacks, GC fences and failure cleanup")
}

// Deliberately reverse addresses relative to type order. These monitor-only
// stand-ins never reach JNI or own native allocations.
private class OrderHandle(raw:Long,override val handleType:Long):NativeHandle(raw){
    override fun freeNative(raw:Long){}
}
private fun checkHandleOrderRoutes(){
    val handles=listOf(OrderHandle(64,0),OrderHandle(48,0),OrderHandle(32,1),OrderHandle(16,1))
    val bean=ManagementFactory.getThreadMXBean()
    fun route(kind:Int,input:List<OrderHandle>,body:()->Unit){
        when(kind){
            0->withHandleLocks(input[0],input[1],body)
            1->withHandleLocks(input[0],input[1],input[2],body)
            2->withHandleLocks(input,body)
            else->{
                val first=prepareInputHandles(input.filter{it.handleType==0L},false)
                val second=prepareInputHandles(input.filter{it.handleType==1L},true)
                withOrderedHandleLocks(first){withOrderedHandleLocks(second,body)}
            }
        }
    }
    for(kind in 0..3){
        val input=when(kind){0->listOf(handles[0],handles[2]);1->handles.take(3);else->handles}
        val ordered=input.sortedWith(Comparator{a,b->compareHandleOrder(a,b)})
        for(block in 1 until ordered.size){
            val failure=AtomicReference<Throwable?>()
            val thread=Thread{try{route(kind,input){}}catch(t:Throwable){failure.set(t)}}.apply{isDaemon=true}
            try{synchronized(ordered[block]){
                thread.start();val until=System.nanoTime()+5_000_000_000L
                while(thread.state!=Thread.State.BLOCKED && System.nanoTime()<until)Thread.sleep(1)
                check(thread.state==Thread.State.BLOCKED)
                val info=bean.getThreadInfo(longArrayOf(thread.id),true,true)[0]
                val actual=info.lockedMonitors.map{it.identityHashCode}.toSet()
                for(i in ordered.indices)check((System.identityHashCode(ordered[i]) in actual)==(i<block)){"route $kind uses inconsistent monitor order"}
            }}finally{thread.join(5000)}
            check(!thread.isAlive);failure.get()?.let{throw it}
        }
    }
    // Stale/reused or forged wrappers may not let equal-address ties choose
    // opposite monitor orders in shared-only calls.
    val first=OrderHandle(96,0);val alias=OrderHandle(96,0)
    check(runCatching{withHandleLocks(first,alias){}}.exceptionOrNull() is HandleValidationFailure)
    first.close()
    check(runCatching{withHandleLocks(alias,first){}}.exceptionOrNull() is HandleValidationFailure)
}
