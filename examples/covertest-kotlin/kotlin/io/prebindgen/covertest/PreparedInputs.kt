package io.prebindgen.covertest

import io.prebindgen.covertest.bulk.*
import io.prebindgen.covertest.collections.collectionI64

fun checkPreparedInputs() {
    checkBulkReplay()
    checkBulkCallbackCleanup()
    // Recomputed positions must preserve the packer's depth-first order across
    // handles before/after nested rows, absent/empty ranges and nullable slots.
    run {
        val patchOwners = List(8) { bulkHandleNew(100L+it) }
        val patchDrops = bulkDrops()
        check(bulkPatchRows(patchOwners[0],listOf(null,emptyList(),listOf(null,patchOwners[1],null,patchOwners[2])),patchOwners[3])==listOf(100L,3L,-4L,0L,4L,-3L,101L,-3L,102L,103L))
        check(bulkPatchRows(null,null,patchOwners[4])==listOf(-1L,-2L,104L))
        check(bulkPatchRows(patchOwners[5],emptyList(),patchOwners[6])==listOf(105L,0L,106L))
        check(bulkPatchRows(null,listOf(listOf(null)),patchOwners[7])==listOf(-1L,1L,1L,-3L,107L))
        check(patchOwners.all { it.isClosed } && bulkDrops()==patchDrops+8uL)
    }
    val callbackOwners=List(3) { bulkHandleNew(200L+it) }
    val callbackDrops=bulkDrops()
    check(bulkPatchCallback(BulkPatchCallback { listOf(null,emptyList(),listOf(null,callbackOwners[0],null,callbackOwners[1]),null,listOf(callbackOwners[2]),null) })==listOf(-4L,0L,4L,-3L,200L,-3L,201L,-4L,1L,202L,-4L))
    check(callbackOwners.all { it.isClosed } && bulkDrops()==callbackDrops+3uL)
    val mixedBorrowed=bulkHandleNew(210)
    val mixedOwned=bulkHandleNew(211)
    val mixedPatchDrops=bulkDrops()
    check(bulkPairTextCallback(BulkPairTextCallback { listOf(BulkPairText(mixedBorrowed,mixedOwned,"ok")) })==1uL)
    check(!mixedBorrowed.isClosed && mixedOwned.isClosed && bulkHandleValue(mixedBorrowed)==210L)
    check(bulkDrops()==mixedPatchDrops+2uL)
    mixedBorrowed.close()
    val closedPatch=List(4) { bulkHandleNew(220L+it) }
    val closedPatchDrops=bulkDrops()
    closedPatch[1].close()
    var closedRejected=false
    check(bulkPatchRows(closedPatch[0],listOf(listOf(null,closedPatch[1],closedPatch[2])),closedPatch[3],JniErrorHandler {closedRejected=true;emptyList()}).isEmpty())
    // Earlier constructor arguments may already be consumed; unread owners stay live.
    check(closedRejected && !closedPatch[2].isClosed && !closedPatch[3].isClosed)
    closedPatch.forEach {it.close()}
    check(bulkDrops()==closedPatchDrops+4uL)
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

    // A consumed wrapper and its live successor legitimately share an address.
    val previous = bulkHandleNew(34)
    val successor = previous.take()
    val reuseDrops = bulkDrops()
    bulkHandleTextCallback(BulkHandleTextCallback {
        listOf(BulkHandleText(previous,"ok"),BulkHandleText(successor,"\ud800"))
    },JniErrorHandler { 0uL })
    check(previous.isClosed && successor.isClosed && bulkDrops() == reuseDrops + 1uL)

    val chosen = bulkHandleNew(21)
    val other = bulkHandleNew(22)
    var reads = 0
    val changing = object : AbstractList<List<BulkHandle>>() {
        override val size: Int get() = 1
        override fun get(index: Int): List<BulkHandle> = listOf(if (reads++ == 0) chosen else other)
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
    try { bulkNestedEnvelope(listOf(listOf(throwing)), JniErrorHandler { intercepted = true; -1L }); error("missing nested user failure") }
    catch (failure: IllegalArgumentException) { check(failure === sentinel) }
    check(!intercepted)

    println("prepared inputs PASS: optional niches, null ownership, nested snapshots, encoding rollback and user exceptions")
}
