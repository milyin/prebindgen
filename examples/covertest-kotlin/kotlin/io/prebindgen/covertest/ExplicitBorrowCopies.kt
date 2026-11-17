package io.prebindgen.covertest

import io.prebindgen.covertest.borrowcopies.*
import java.util.concurrent.atomic.AtomicReference

fun checkExplicitBorrowCopies() {
    fun counts() = List(3) { borrowPacketCounter(it.toULong()) }
    val failure = AtomicReference<Throwable?>()
    fun inside(body: () -> Unit) {
        try { body() } catch (error: Throwable) { failure.compareAndSet(null, error) }
    }
    val before = counts()
    val original = borrowPacketNew(4096UL)
    val first = borrowPacketRef(original)
    check(borrowPacketOptional(original, false) == null)
    val second = checkNotNull(borrowPacketOptional(original, true))
    check(first !== original && second !== original && first !== second)
    check(counts()[0] == before[0] + 2UL && counts()[1] == before[1] + 8192UL)
    original.close()
    check(borrowPacketLen(first) == 4096UL && borrowPacketLen(second) == 4096UL)
    first.close(); second.close()
    check(counts()[2] == before[2] + 3UL)

    val packet = borrowPacketNew(65536UL)
    val copies = mutableListOf<BorrowPacket>()
    val copyStart = counts()
    // Explicit callback defaults transfer their owning copies to the author.
    BufferAllocationProbe.start()
    val copiedAllocations = try {
        borrowPacketEmit(packet, { copy -> inside { copies.add(copy) } }, 32UL)
        BufferAllocationProbe.stop()
    } catch (error: Throwable) { BufferAllocationProbe.stop(); throw error }
    check(copiedAllocations == 32L)
    check(copies.size == 32 && copies.none { it.isClosed })
    check(counts()[0] == copyStart[0] + 32UL)
    check(counts()[1] == copyStart[1] + 32UL * 65536UL)
    copies.forEach { check(borrowPacketLen(it) == 65536UL); it.close() }

    val expandedStart = counts()
    var length = 0UL
    BufferAllocationProbe.start()
    val expandedAllocations = try {
        borrowPacketEmitExpanded(packet, { size -> inside { length += size } }, 32UL)
        BufferAllocationProbe.stop()
    } catch (error: Throwable) { BufferAllocationProbe.stop(); throw error }
    check(expandedAllocations == 0L)
    check(length == 32UL * 65536UL)
    check(borrowPacketRefExpanded(packet) == 65536UL)
    check(counts() == expandedStart) // Local overrides supersede both copy defaults.

    var optional: BorrowPacket? = null
    borrowPacketEmitOptional(packet, { copy -> inside { check(copy == null) } }, false)
    check(counts() == expandedStart)
    borrowPacketEmitOptional(packet, { copy -> inside { optional = checkNotNull(copy) } }, true)
    check(borrowPacketLen(checkNotNull(optional)) == 65536UL)
    optional!!.close()
    check(counts()[0] == expandedStart[0] + 1UL)

    val workerCopy = AtomicReference<BorrowPacket?>()
    val caller = Thread.currentThread()
    borrowPacketEmitWorker(packet, { copy -> inside {
        check(Thread.currentThread() !== caller)
        workerCopy.set(copy)
    } })
    check(borrowPacketLen(checkNotNull(workerCopy.get())) == 65536UL)
    workerCopy.get()!!.close()
    borrowPacketEmit(packet, { copy -> copy.close(); throw IllegalStateException("expected explicit-copy callback failure") }, 1UL)
    check(!packet.isClosed && borrowPacketLen(packet) == 65536UL)
    packet.close()
    val noClone = borrowNoCloneNew()
    var observed = 0L
    borrowNoCloneEmit(noClone, { number -> inside { observed = number } })
    check(observed == 91L && borrowNoCloneValue(noClone) == 91L)
    noClone.close()
    val row = borrowEnumRows().single()
    check(row.mode == BorrowMode.Ready && borrowPacketLen(row.packet) == 128UL)
    row.packet.close()
    failure.get()?.let { throw it }
    println("explicit borrow copies PASS: retained copies, exact clone/byte counts, global/local overrides, optional borrows, workers and non-Clone projections")
}
