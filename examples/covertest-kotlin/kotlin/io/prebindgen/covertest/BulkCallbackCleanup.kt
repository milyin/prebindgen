package io.prebindgen.covertest

import io.prebindgen.covertest.bulk.*
import java.nio.ByteBuffer

private class RawBulkCleanup(private val body: () -> Array<Any?>) {
    fun run(): Array<Any?> = body()
}

// Use the native layouts initialized for the callback expansion: a root Range,
// optional inner collections (presence + Range), and nullable i64 handles.
private fun nestedCleanupFrame(owners: List<BulkHandle>, mode: Int): ByteArray {
    val root = BulkLayouts.r5019fbfb9686d1a8
    val row = BulkLayouts.rf27ab2e6065597a6
    val handle = BulkLayouts.rcbe830243575e259
    val rows = root.size
    val values = rows + 3 * row.size
    val frame = ByteBuffer.allocate(values + 4 * handle.size).order(BulkLayouts.byteOrder)
    frame.putLong(root.offsets[0], rows.toLong()).putLong(root.offsets[0] + 8, 3)
    fun row(i: Int, at: Long, size: Long) {
        val offset = rows + i * row.size
        frame.putInt(offset + row.offsets[0], if (at == -1L) 0 else 1)
        frame.putLong(offset + row.offsets[1], at).putLong(offset + row.offsets[1] + 8, size)
    }
    fun value(i: Int, address: Long) { frame.putLong(values + i * handle.size + handle.offsets[0], address) }
    row(0, values.toLong(), 1)
    value(0, owners[0].raw)
    when (mode) {
        0 -> { // First inner owner is consumed; malformed middle range must not hide the last owner.
            row(1, Long.MAX_VALUE, 1)
            row(2, (values + handle.size).toLong(), 1)
            value(1, owners[1].raw)
        }
        1 -> { // An unread alias of an already consumed owner must not be freed again.
            row(1, (values + handle.size).toLong(), 2)
            value(1, owners[0].raw); value(2, owners[1].raw)
            row(2, -1, Long.MAX_VALUE)
        }
        2 -> { // Invalid pointer encoding is rejected before any unsafe acquisition.
            row(1, (values + handle.size).toLong(), 1); value(1, 5)
            row(2, (values + 2 * handle.size).toLong(), 1); value(2, owners[1].raw)
        }
        3 -> { // Invalid first container still permits cleanup of valid later containers.
            row(0, Long.MAX_VALUE, 1)
            row(1, values.toLong(), 1)
            row(2, (values + handle.size).toLong(), 1); value(1, owners[1].raw)
        }
        else -> error("unknown raw frame mode")
    }
    return frame.array()
}

fun checkBulkCallbackCleanup() {
    for (worker in listOf(false, true)) {
        val owners = List(3) { bulkHandleNew(100L + it) }
        val before = bulkDrops()
        val result = bulkCleanupNestedCallback(BulkCleanupNestedCallback {
            listOf(null, listOf(null, owners[0], null, owners[1]), emptyList(), listOf(owners[2]))
        }, false, worker)
        check(result == listOf(-4L, 4L, -3L, 100L, -3L, 101L, 0L, 1L, 102L))
        check(owners.all { it.isClosed } && bulkDrops() == before + 3uL)
    }

    // Ordinary inputs report their consumed prefix and leave unread owners to Kotlin.
    val inputMethod=io.prebindgen.covertest.rawinputs.JNINative::class.java.declaredMethods.single {it.name.startsWith("call_bulk_nested_handles_input_")}.also {it.isAccessible=true}
    for(mode in 0..3) {
        val owners=List(2) {bulkHandleNew(500L+it)}
        val before=bulkDrops()
        val bytes=nestedCleanupFrame(owners,mode)
        val consumed=if(mode==3)0 else 1
        val ownership=LongArray(1) {Long.MIN_VALUE}
        val packet=withHandleLocks(owners) {inputMethod.invokePacket(bytes,bytes.size,ownership)}
        check((packet[0] as String).isNotEmpty() && ownership[0]==-1L-consumed)
        owners.take(consumed).forEach {it.markConsumed()}
        check(bulkDrops()==before+consumed.toULong())
        owners.drop(consumed).forEachIndexed {i,h->check(bulkHandleValue(h)==500L+consumed+i)}
        owners.forEach {it.close()}
        check(bulkDrops()==before+2uL)
    }

    val method = io.prebindgen.covertest.rawinputs.JNINative::class.java.declaredMethods.single {
        it.name.startsWith("call_bulk_cleanup_nested_callback_")
    }.also { it.isAccessible = true }
    for (worker in listOf(false, true)) {
        for (mode in 0..3) {
            val owners = List(2) { bulkHandleNew(200L + it) }
            val before = bulkDrops()
            val bytes = nestedCleanupFrame(owners, mode)
            val raw = RawBulkCleanup {
                owners.forEach { it.markConsumed() }
                arrayOf(bytes, bytes.size)
            }
            val packet = method.invokePacket(raw, false, worker)
            check(packet[0] is String)
            val message = packet[0] as String
            check(message.isNotEmpty())
            check(owners.all { it.isClosed } && bulkDrops() == before + 2uL)
        }

        // Cleanup must finish before a second callback invocation, also on an
        // attached native worker. The successful frame has only absent children.
        val owners = List(2) { bulkHandleNew(300L + it) }
        val before = bulkDrops()
        val bad = nestedCleanupFrame(owners, 0)
        val root = BulkLayouts.r5019fbfb9686d1a8
        val row = BulkLayouts.rf27ab2e6065597a6
        val good = ByteBuffer.allocate(root.size + row.size).order(BulkLayouts.byteOrder)
        good.putLong(root.offsets[0], root.size.toLong()).putLong(root.offsets[0] + 8, 1)
        good.putInt(root.size + row.offsets[0], 0)
        good.putLong(root.size + row.offsets[1], -1).putLong(root.size + row.offsets[1] + 8, Long.MAX_VALUE)
        var calls = 0
        val raw = RawBulkCleanup {
            val bytes = if (calls++ == 0) {
                owners.forEach { it.markConsumed() }
                bad
            } else {
                check(bulkDrops() == before + 2uL)
                good.array()
            }
            arrayOf(bytes, bytes.size)
        }
        val packet = method.invokePacket(raw, true, worker)
        check(packet[0] == null && calls == 2 && bulkDrops() == before + 2uL)
    }

    val validBorrow = bulkHandleNew(390)
    val validOwner = bulkHandleNew(391)
    val validDrops = bulkDrops()
    check(bulkCleanupPairCallback(BulkCleanupPairCallback {
        BulkPairText(validBorrow, validOwner, "ok")
    }) == 1uL)
    check(!validBorrow.isClosed && validOwner.isClosed && bulkDrops() == validDrops + 2uL)
    validBorrow.close()

    val pairMethod = io.prebindgen.covertest.rawinputs.JNINative::class.java.declaredMethods.single {
        it.name.startsWith("call_bulk_cleanup_pair_callback_")
    }.also { it.isAccessible = true }
    val borrowed = bulkHandleNew(400)
    val owned = bulkHandleNew(401)
    val before = bulkDrops()
    // A truncated record still contains a readable borrowed/owned handle pair.
    // No text descriptor is read during cleanup, and the borrow remains live.
    val pair = ByteBuffer.allocate(16).order(BulkLayouts.byteOrder)
        .putLong(borrowed.raw).putLong(owned.raw).array()
    val raw = RawBulkCleanup { owned.markConsumed(); arrayOf(pair, pair.size) }
    val packet = pairMethod.invokePacket(raw)
    check((packet[0] as String).isNotEmpty())
    check(owned.isClosed && !borrowed.isClosed && bulkHandleValue(borrowed) == 400L)
    check(bulkDrops() == before + 1uL)
    borrowed.close()
    println("bulk callback cleanup PASS: cached lifetimes, nested failures, consumed aliases, pointer checks, borrows, retries and workers")
}
