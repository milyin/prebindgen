package io.prebindgen.covertest

import io.prebindgen.covertest.bulk.*
import io.prebindgen.covertest.collections.collectionI64

fun checkPreparedInputs() {
    val before = bulkDrops()
    val handle = bulkHandleNew(7)
    check(bulkOptionalBorrowPair(false, null, 3) == -1L)
    check(bulkOptionalBorrowPair(true, handle, 3) == 10L && !handle.isClosed)
    var invalidPresent = false
    check(bulkOptionalBorrowPair(true, null, 3, JniErrorHandler { invalidPresent = true; -2L }) == -2L)
    check(invalidPresent)
    handle.close()
    check(bulkDrops() == before + 1uL)

    // Model malformed Java/reflection input without weakening the public data-class type.
    val handleField = BulkHandleText::class.java.getDeclaredField("handle").apply { isAccessible = true }
    fun rowsWithNull(owners: List<BulkHandle>, bad: Int): List<BulkHandleText> {
        var next = 0
        return List(3) { i -> BulkHandleText(if (i == bad) owners[0] else owners[next++], "ok").also {
            if (i == bad) handleField.set(it, null)
        } }
    }
    // Invalid required handles never consume the other members of the group.
    for (bad in 0..2) {
        val owners = List(2) { bulkHandleNew(it.toLong()) }
        val rows = rowsWithNull(owners, bad)
        val drops = bulkDrops()
        var rejected = false
        check(bulkHandleTexts(rows, JniErrorHandler { rejected = true; emptyList() }).isEmpty())
        check(rejected && owners.none { it.isClosed } && bulkDrops() == drops)
        owners.forEach { it.close() }
        check(bulkDrops() == drops + 2uL)

        val returned = List(2) { bulkHandleNew(it.toLong()) }
        val returnedRows = rowsWithNull(returned, bad)
        var callbackRejected = false
        check(bulkHandleTextCallback(BulkHandleTextCallback { returnedRows }, JniErrorHandler { callbackRejected = true; 0uL }) == 0uL)
        check(callbackRejected && returned.all { it.isClosed } && bulkDrops() == drops + 4uL)
    }

    val valid = listOf(BulkHandleText(bulkHandleNew(11), "first"), BulkHandleText(bulkHandleNew(12), "世界"))
    val moved = bulkHandleTexts(valid)
    check(valid.all { it.handle!!.isClosed })
    check(moved.map { bulkHandleValue(it.handle) } == listOf(11L,12L))
    moved.forEach { it.handle!!.close() }

    // Encoding errors leave ordinary inputs owned by the caller, but reclaim
    // values returned by a callback whose encoding fails before transfer.
    for (returned in listOf(false,true)) {
        val owners = List(2) { bulkHandleNew(it.toLong()) }
        val rows = listOf(BulkHandleText(owners[0], "ok"), BulkHandleText(owners[1], "\ud800"))
        val drops = bulkDrops()
        var rejected = false
        if (returned) bulkHandleTextCallback(BulkHandleTextCallback { rows }, JniErrorHandler { rejected = true; 0uL })
        else bulkHandleTexts(rows, JniErrorHandler { rejected = true; emptyList() })
        check(rejected && owners.all { it.isClosed == returned })
        check(bulkDrops() == drops + if (returned) 2uL else 0uL)
        if (!returned) owners.forEach { it.close() }
    }

    val borrowed = bulkHandleNew(31)
    val owned = bulkHandleNew(32)
    val mixedDrops = bulkDrops()
    var mixedFailed = false
    bulkPairTextCallback(BulkPairTextCallback { listOf(BulkPairText(borrowed, owned, "\ud800")) },
        JniErrorHandler { mixedFailed = true; 0uL })
    check(mixedFailed && !borrowed.isClosed && owned.isClosed && bulkDrops() == mixedDrops + 1uL)
    borrowed.close()

    // Error cleanup deduplicates different wrappers referring to one owner.
    val original = bulkHandleNew(33)
    val alias = BulkHandle(original.raw)
    val aliasDrops = bulkDrops()
    bulkHandleTextCallback(BulkHandleTextCallback {
        listOf(BulkHandleText(original,"ok"),BulkHandleText(alias,"\ud800"))
    },JniErrorHandler { 0uL })
    check(original.isClosed && alias.isClosed && bulkDrops() == aliasDrops + 1uL)

    val chosen = bulkHandleNew(21)
    val other = bulkHandleNew(22)
    var reads = 0
    val changing = object : AbstractList<List<BulkHandle?>>() {
        override val size: Int get() = 1
        override fun get(index: Int): List<BulkHandle?> = listOf(if (reads++ == 0) chosen else other)
    }
    val nested = bulkHandleNested(changing)
    check(reads == 1 && chosen.isClosed && !other.isClosed)
    check(bulkHandleValue(nested.single().single()) == 21L)
    nested.flatten().forEach { it.close() }; other.close()

    for (present in listOf(false,true)) {
        val first = if (present) bulkHandleNew(41) else null
        val second = bulkHandleNew(42)
        check(bulkCollectionLockGroups(first?.let { listOf(it) },listOf(second),CollectionLocksProbe {
            (first == null || Thread.holdsLock(first)) && Thread.holdsLock(second)
        }))
        check((first == null || first.isClosed) && second.isClosed)
    }

    val sentinel = IllegalArgumentException("user list failure")
    val throwing = object : AbstractList<Long>() {
        override val size: Int get() = 1
        override fun get(index: Int): Long = throw sentinel
    }
    var intercepted = false
    try { collectionI64(throwing, JniErrorHandler { intercepted = true; emptyList() }); error("missing user failure") }
    catch (failure: IllegalArgumentException) { check(failure === sentinel) }
    check(!intercepted)
    println("prepared inputs PASS: optional niches, null ownership, nested snapshots, encoding rollback and user exceptions")
}
