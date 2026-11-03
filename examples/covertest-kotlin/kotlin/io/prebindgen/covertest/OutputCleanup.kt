package io.prebindgen.covertest

import io.prebindgen.covertest.cleanup.*
import io.prebindgen.covertest.collections.collectionResourceNew
import io.prebindgen.covertest.resources.*

internal object OutputCleanupProbe {
    var callbackCalls = 0
    var failRow = -1L
    var transfer = false
    var closeEarly = false
    var retained: OwnedResource? = null
    val seen = mutableListOf<OwnedResource>()
    val failure = IllegalStateException("injected row constructor failure")
    fun row(row: OutputCleanupRow) {
        row.first?.let { seen.add(it) }; row.second?.let { seen.add(it) }
        if (closeEarly && row.id == 0L) row.second!!.close()
        if (row.id == failRow) {
            if (transfer) retained = row.first!!.take()
            throw failure
        }
    }
    fun reset(fail: Long, take: Boolean = false, close: Boolean = false) {
        callbackCalls = 0; failRow = fail; transfer = take; closeEarly = close; retained = null; seen.clear()
    }
}

fun checkOutputCleanup() {
    val decoders: List<() -> Unit> = listOf(
        { outputCleanupRows().forEach { it.first?.close(); it.second?.close() } },
        { outputCleanupRowsBulk().forEach { it.first?.close(); it.second?.close() } },
        { outputCleanupNested().filterNotNull().flatten().forEach { it.first?.close(); it.second?.close() } },
        { outputCleanupNestedBulk().filterNotNull().flatten().forEach { it.first?.close(); it.second?.close() } },
        { outputCleanupCallback(OutputCleanupCallback { rows -> OutputCleanupProbe.callbackCalls++; rows.filterNotNull().flatten().forEach { it.first?.close(); it.second?.close() } }) },
        { outputCleanupCallbackBulk(OutputCleanupBulkCallback { rows -> OutputCleanupProbe.callbackCalls++; rows.filterNotNull().flatten().forEach { it.first?.close(); it.second?.close() } }) },
    )
    for ((index, decode) in decoders.withIndex()) {
        for (fail in listOf(-1L, 0L, 1L, 2L)) {
            OutputCleanupProbe.reset(fail)
            val before = ownedResourceDrops()
            // Void callbacks report and clear upcall exceptions; ordinary
            // function-result reconstruction propagates the original exception.
            if (index >= 4) decode()
            else {
                try { decode(); check(fail == -1L) }
                catch (caught: IllegalStateException) { check(fail >= 0L && caught === OutputCleanupProbe.failure) { "decode=$index fail=$fail caught=$caught same=${caught === OutputCleanupProbe.failure}" } }
            }
            if (index >= 4) check(OutputCleanupProbe.callbackCalls == if (fail == -1L) 1 else 0)
            check(OutputCleanupProbe.seen.all { it.isClosed }) { "live wrappers: decode=$index fail=$fail" }
            check(ownedResourceDrops() == before + 4uL) { "drops: decode=$index fail=$fail before=$before after=${ownedResourceDrops()}" }
        }
        // Completed-row handles can already be closed. A take in the failing
        // row transfers ownership: cleanup must leave that new owner alive.
        OutputCleanupProbe.reset(1, take = true, close = true)
        val before = ownedResourceDrops()
        if (index >= 4) decode()
        else {
            try { decode(); error("constructor should throw") }
            catch (caught: IllegalStateException) { check(caught === OutputCleanupProbe.failure) }
        }
        check(OutputCleanupProbe.seen.all { it.isClosed })
        check(ownedResourceDrops() == before + 3uL)
        check(ownedResourceId(checkNotNull(OutputCleanupProbe.retained)) == 11L)
        OutputCleanupProbe.retained!!.close()
        check(ownedResourceDrops() == before + 4uL)
    }
    // A handle before and after the collection uses the same native allocation
    // transfer protocol but is outside the collection ownership list.
    val surrounded: List<((OwnedResource, List<List<OutputCleanupRow>?>, OwnedResource) -> Unit) -> Unit> = listOf(
        { build -> outputCleanupSurround(build = build) },
        { build -> outputCleanupSurroundBulk(build = build) },
    )
    for (decode in surrounded) {
        OutputCleanupProbe.reset(1)
        var before = ownedResourceDrops()
        try { decode { _, _, _ -> error("builder must not run") }; error("row constructor should fail") }
        catch (caught: IllegalStateException) { check(caught === OutputCleanupProbe.failure) }
        check(OutputCleanupProbe.seen.all { it.isClosed })
        check(ownedResourceDrops() == before + 6uL)
        // The collection is complete when the outer builder fails. Its escaped
        // wrappers still need cleanup, without walking already-transferred fields.
        OutputCleanupProbe.reset(-1)
        before = ownedResourceDrops()
        val failure = IllegalStateException("outer builder failure")
        try { decode { _, _, _ -> throw failure }; error("builder should fail") }
        catch (caught: IllegalStateException) { check(caught === failure) }
        check(OutputCleanupProbe.seen.all { it.isClosed })
        check(ownedResourceDrops() == before + 6uL)
    }
    OutputCleanupProbe.reset(-1)
    checkCallbackFoldCleanup()
    checkCollectionOwnerFailures()
}

private fun checkCallbackFoldCleanup() {
    val folds: List<((List<OutputCleanupRow>) -> Unit) -> Unit> = listOf(
        { visit -> outputCleanupRowsFold(Unit) { _, row -> visit(listOf(row)) } },
        { visit -> outputCleanupNestedFold(Unit) { _, rows -> visit(rows.orEmpty()) } },
    )
    fun handles(rows: List<OutputCleanupRow>) = rows.flatMap { listOfNotNull(it.first, it.second) }
    for ((mode, decode) in folds.withIndex()) {
        // Constructor failures clean only the failing and unread rows. Earlier
        // rows have already been delivered to the fold and remain user-owned.
        for (fail in listOf(-1L, 0L, 1L, 2L)) {
            OutputCleanupProbe.reset(fail)
            val accepted = mutableListOf<OutputCleanupRow>()
            val before = ownedResourceDrops()
            try {
                decode { rows ->
                    accepted.addAll(rows)
                    // Reentry uses a separate ownership journal and no JNI lock.
                    check(io.prebindgen.covertest.collections.collectionI64Fold(listOf(3L, 4L), 0L) { a, n -> a + n } == 7L)
                }
                check(fail == -1L)
            } catch (caught: IllegalStateException) {
                check(fail >= 0L && caught === OutputCleanupProbe.failure)
            }
            check(handles(accepted).none { it.isClosed }) { "earlier fold row closed: mode=$mode fail=$fail" }
            val delivered = handles(accepted).toSet()
            check(OutputCleanupProbe.seen.filter { it !in delivered }.all { it.isClosed })
            handles(accepted).forEach { it.close() }
            check(ownedResourceDrops() == before + 4uL) { "fold constructor leak: mode=$mode fail=$fail" }
        }
        // The fold itself fails after reconstruction; close() and take() inside
        // user code must retain their normal ownership effects during rollback.
        for (take in listOf(false, true)) {
            OutputCleanupProbe.reset(-1)
            val accepted = mutableListOf<OutputCleanupRow>()
            val before = ownedResourceDrops()
            val failure = IllegalStateException("fold user failure")
            var retained: OwnedResource? = null
            var failedRow: List<OutputCleanupRow> = emptyList()
            try {
                decode { rows ->
                    if (rows.any { it.id == 1L }) {
                        failedRow = rows
                        rows.flatMap { listOfNotNull(it.second) }.forEach { it.close() }
                        if (take) retained = rows.first { it.id == 1L }.first!!.take()
                        throw failure
                    }
                    accepted.addAll(rows)
                }
                error("fold should fail")
            } catch (caught: IllegalStateException) { check(caught === failure) }
            check(handles(failedRow).all { it.isClosed })
            check(handles(accepted).none { it.isClosed })
            handles(accepted).forEach { it.close() }
            check(ownedResourceDrops() == before + if (take) 3uL else 4uL)
            if (take) {
                check(ownedResourceId(checkNotNull(retained)) == 11L)
                retained!!.close()
                check(ownedResourceDrops() == before + 4uL)
            }
        }
    }
    OutputCleanupProbe.reset(-1)
    val primitiveFailure = IllegalStateException("resource-free fold failure")
    try {
        io.prebindgen.covertest.collections.collectionI64Fold(listOf(1L, 2L, 3L), 0L) { a, n ->
            if (n == 2L) throw primitiveFailure
            a + n
        }
        error("primitive fold should throw")
    } catch (caught: IllegalStateException) { check(caught === primitiveFailure) }
}

private fun checkCollectionOwnerFailures() {
    // Failure while retaining a newly constructed wrapper must leave its native
    // allocation with the wire, rather than silently leaking or freeing it.
    val journal = CollectionOwners()
    val sentinel = OutOfMemoryError("injected owner-list allocation failure")
    val field = CollectionOwners::class.java.getDeclaredField("owners").also { it.isAccessible = true }
    field.set(journal, object : java.util.ArrayList<NativeHandle>() {
        override fun add(element: NativeHandle): Boolean = throw sentinel
    })
    val rawOwner = collectionResourceNew(99)
    val wrapper = OwnedResource(rawOwner.raw)
    try { journal.own(wrapper); error("registration should fail") }
    catch (caught: OutOfMemoryError) { check(caught === sentinel) }
    check(wrapper.isClosed)
    check(journal.close(null) == null)
    check(ownedResourceId(rawOwner) == 99L)
    rawOwner.close()

    // Successful fold delivery clears references without closing the delivered
    // owners; the next row starts a fresh unread prefix in the same journal.
    val reused = CollectionOwners()
    var released = 0
    fun owner() = object : NativeHandle(2L) {
        override fun freeNative(raw: Long) { released++ }
    }
    val delivered = reused.own(owner())
    reused.complete = true
    reused.release()
    check(!reused.complete && !delivered.isClosed)
    val current = reused.own(owner())
    check(!reused.unread() && reused.unread())
    check(reused.close(null) == null)
    check(released == 1 && current.isClosed && !delivered.isClosed)
    delivered.close()
    check(released == 2)

    // One failing destructor cannot prevent later owners from being released or
    // replace the original delivery failure. No native pointers are used here.
    val failed = CollectionOwners()
    val original = IllegalStateException("delivery failure")
    val first = IllegalArgumentException("first cleanup")
    val second = IllegalArgumentException("second cleanup")
    var releases = 0
    for (failure in listOf(first, second)) {
        failed.own(object : NativeHandle(2L) {
            override fun freeNative(raw: Long) { releases++; throw failure }
        })
    }
    check(failed.close(original) === original)
    check(releases == 2 && original.suppressed.toList() == listOf(first, second))
}
