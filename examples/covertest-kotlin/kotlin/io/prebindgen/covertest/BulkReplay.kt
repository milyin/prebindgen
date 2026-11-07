package io.prebindgen.covertest

import io.prebindgen.covertest.bulk.*

fun checkBulkReplay() {
    checkBulkReplayRaw()
    checkLargeAliasHistory()
    fun rows(): List<BulkReplayRow> = listOf(
        BulkReplayRow(bulkHandleNew(1), "世界", byteArrayOf(0,-1), BulkKind.A, bulkHandleNew(2)),
        BulkReplayRow(bulkHandleNew(3), null, null, null, bulkHandleNew(4)),
        BulkReplayRow(bulkHandleNew(5), "ok", byteArrayOf(), BulkKind.B, bulkHandleNew(6))
    )
    fun owners(rows: List<BulkReplayRow>) = rows.flatMap { listOf(it.first,it.last) }
    for (mode in 0..5) {
        val returned=mode==1 || mode==5
        val values=rows();val handles=owners(values);val drops=bulkDrops()
        bulkReplayReset(0uL)
        fun nested(rows: List<BulkReplayRow>) = listOf(emptyList(),rows.take(1),emptyList(),rows.drop(1),emptyList())
        val result=when(mode){4->bulkReplayNestedRows(nested(values));5->bulkReplayNestedCallback(BulkReplayNestedCallback {nested(values)});3->bulkReplayObjectRows(values);1->bulkReplayCallback(BulkReplayCallback {values});2->bulkReplayRedundantRows(values);else->bulkReplayRows(values)}
        check(result==34uL && handles.all {it.isClosed})
        check(bulkDrops()==drops+6uL)
        check((0uL..2uL).all {bulkReplayCount(it)==2uL} && bulkReplayCount(3uL)==1uL)
        for(fail in listOf(1uL,2uL)) {
            val failing=rows();val pending=owners(failing);val before=bulkDrops()
            bulkReplayReset(fail);var rejected=false
            val error=JniErrorHandler {rejected=true;0uL}
            val value=when(mode){4->bulkReplayNestedRows(nested(failing),error);5->bulkReplayNestedCallback(BulkReplayNestedCallback {nested(failing)},error);3->bulkReplayObjectRows(failing,error);1->bulkReplayCallback(BulkReplayCallback {failing},error);2->bulkReplayRedundantRows(failing,error);else->bulkReplayRows(failing,error)}
            check(value==0uL && rejected && bulkReplayCount(3uL)==0uL)
            val consumed=if(returned || fail==2uL) 6 else 2
            check(pending.withIndex().all {(i,h)->h.isClosed==(i<consumed)})
            check(bulkDrops()==before+consumed.toULong())
            pending.forEach {it.close()}
            check(bulkDrops()==before+6uL)
        }
    }
    bulkReplayReset(0uL)
    check(bulkReplayRows(emptyList())==0uL)
    println("bulk replay PASS: optional borrowed text/bytes/enums, constructors once, panic prefixes and callback unread-owner cleanup")
}

private fun checkLargeAliasHistory() {
    // Two owned handles per inner list cross the runtime's vector/index boundary.
    // Constructor failure occurs after promotion, covering callback unread-owner
    // cleanup and ordinary-input ownership receipts through the actual JNI call.
    val count = 256
    for (callback in listOf(false, true)) for (fail in listOf(0uL, 100uL)) {
        val rows = List(count) { i -> BulkReplayRow(
            bulkHandleNew(2L * i + 1), "ok", byteArrayOf(7), BulkKind.A,
            bulkHandleNew(2L * i + 2)
        ) }
        val handles = rows.flatMap { listOf(it.first, it.last) }
        val nested = rows.map { listOf(it) }
        val before = bulkDrops()
        bulkReplayReset(fail)
        var failed = false
        val error = JniErrorHandler { failed = true; 0uL }
        val result = if (callback) {
            bulkReplayNestedCallback(BulkReplayNestedCallback { nested }, error)
        } else {
            bulkReplayNestedRows(nested, error)
        }
        check(failed == (fail != 0uL))
        if (!failed) {
            check(result == count.toULong() * (2uL * count.toULong() + 5uL))
            check(bulkReplayCount(3uL) == 1uL)
        } else {
            check(result == 0uL && bulkReplayCount(0uL) == fail && bulkReplayCount(3uL) == 0uL)
        }
        val consumed = handles.count { it.isClosed }
        if (callback || !failed) check(consumed == handles.size)
        else check(consumed in 1 until handles.size)
        check(bulkDrops() == before + consumed.toULong())
        handles.forEachIndexed { i, handle ->
            if (!handle.isClosed) check(bulkHandleValue(handle) == i.toLong() + 1)
            handle.close()
        }
        check(bulkDrops() == before + handles.size.toULong())
    }
    bulkReplayReset(0uL)
}

private fun checkBulkReplayRaw() {
    val root=BulkLayouts.r55fb9dde8a946716
    val row=BulkLayouts.r01496d6ef6ccdfde
    val method=io.prebindgen.covertest.rawinputs.JNINative::class.java.declaredMethods.single {it.name.startsWith("call_bulk_replay_rows_")}
    val captureType=method.parameterTypes.last()
    val companion=captureType.getField("Companion").get(null)
    for(mode in 0..3) {
        val handles=List(6) {bulkHandleNew(it.toLong()+10)}
        val before=bulkDrops();bulkReplayReset(if(mode==3)2uL else 0uL)
        val payload=root.size+3*row.size
        val frame=java.nio.ByteBuffer.allocate(payload+2).order(BulkLayouts.byteOrder)
        frame.putLong(root.offsets[0],root.size.toLong()).putLong(root.offsets[0]+8,3)
        frame.put(payload,'x'.code.toByte()).put(payload+1,0xff.toByte())
        for(i in 0..2) {
            val at=root.size+i*row.size
            frame.putLong(at+row.offsets[0],handles[if(mode==2 && i==1)0 else 2*i].raw)
            frame.putInt(at+row.offsets[1],if(i<2)1 else 0)
            frame.putLong(at+row.offsets[2],if(i==2)Long.MIN_VALUE else (payload+if(mode==0 && i==1)1 else 0).toLong())
            frame.putLong(at+row.offsets[2]+8,if(i==2)Long.MAX_VALUE else 1)
            frame.putInt(at+row.offsets[3],if(mode==1 && i==1)1 else 0)
            frame.putLong(at+row.offsets[4],Long.MAX_VALUE).putLong(at+row.offsets[4]+8,1)
            frame.putInt(at+row.offsets[5],if(i==0)-3 else -1)
            frame.putLong(at+row.offsets[6],handles[2*i+1].raw)
        }
        val capture=companion.javaClass.getMethod("acquire").invoke(companion)
        val ownership=LongArray(1) {Long.MIN_VALUE}
        try {
            withHandleLocks(handles) {method.invoke(io.prebindgen.covertest.rawinputs.JNINative,frame.array(),frame.capacity(),ownership,capture)}
            check(captureType.getMethod("getStatus").invoke(capture)==2)
            val message=captureType.getMethod("getMessage").invoke(capture) as String
            check(message.isNotEmpty())
            if(mode==0)check(message.contains("UTF-8"))
            val consumed=if(mode==2)0 else 4
            check(ownership[0]==-1L-consumed)
            handles.take(consumed).forEach {it.markConsumed()}
            check(bulkDrops()==before+consumed.toULong())
            handles.drop(consumed).forEachIndexed {i,h->check(bulkHandleValue(h)==10L+consumed+i)}
            check(bulkReplayCount(0uL)==when(mode){0->1uL;2->0uL;else->2uL})
            check(bulkReplayCount(1uL)==0uL && bulkReplayCount(3uL)==0uL)
            handles.forEach {it.close()};check(bulkDrops()==before+6uL)
        } finally {captureType.getMethod("release").invoke(capture)}
    }
    bulkReplayReset(0uL)
}
