package io.prebindgen.covertest

import io.prebindgen.covertest.getters.*
import java.util.concurrent.atomic.AtomicReference

private const val getterText = "héllo\u0000世界😀"
private fun getterSnapshot() = getterCounts().map { it.toLong() }
private fun checkGetterViews(number: Long, name: String, optionalNumber: Long?, optionalName: String?,
    details: GetterDetails?, rows: List<GetterDetails>, present: Boolean) {
    check(number == 71L && name == getterText)
    check(optionalNumber == if (present) 71L else null)
    check(optionalName == if (present) getterText else null)
    check(details == if (present) GetterDetails(23, "optional") else null)
    check(rows == listOf(GetterDetails(31, "first"), GetterDetails(37, "second")))
}

fun checkBorrowedGetters() {
    val firstStatic = getterStaticLeaf()
    val secondStatic = getterStaticLeaf()
    check(firstStatic !== secondStatic && getterLeafId(firstStatic) == 101L)
    firstStatic.close()
    check(getterLeafId(secondStatic) == 101L)
    secondStatic.close()
    val copiedField = getterFieldsNew { owner, number, text ->
        check(number == 31L)
        owner.close()
        text
    }
    check(copiedField == "first")
    val beforeClones = getterSnapshot()
    getterTwoClonesNew { first, second ->
        check(getterLeafId(first) == 19L && getterLeafId(second) == 19L)
        first.close()
        check(getterLeafId(second) == 19L)
        second.close()
    }
    val afterClones = getterSnapshot()
    check(afterClones[0] == beforeClones[0] + 1 && afterClones[1] == beforeClones[1] + 2)
    check(afterClones[2] == beforeClones[2] + 3)
    for (present in listOf(false, true)) {
        val before = getterSnapshot()
        lateinit var clone: GetterLeaf
        lateinit var copiedRows: List<GetterDetails>
        val owner = getterOwnerNew(present) { owner, number, name, optionalNumber, optionalName, details, rows, leaf ->
            checkGetterViews(number, name, optionalNumber, optionalName, details, rows, present)
            check(getterOwnerId(owner) == 71L && getterLeafId(leaf) == 19L)
            clone = leaf
            copiedRows = rows
            owner
        }
        var after = getterSnapshot()
        // The non-Clone owner and intermediate child are observed, not copied.
        check(after[0] == before[0] && after[1] == before[1] + 1 && after[2] == before[2])
        check(after[3] == before[3] + 1 && after[4] == before[4] + if (present) 2 else 1)
        val copiedName = getterOwnerRead(owner) { number, name -> check(number == 71L); name }
        check(!owner.isClosed)
        owner.close()
        check(copiedName == getterText && copiedRows[1].text == "second")
        check(getterLeafId(clone) == 19L) // Its owning clone survives the source owner.
        clone.close()
        after = getterSnapshot()
        check(after[0] == before[0] + 1 && after[1] == before[1] + 1 && after[2] == before[2] + 2)
    }

    val beforeBuilder = getterSnapshot()
    var escapedOwner: GetterOwner? = null
    var escapedLeaf: GetterLeaf? = null
    val original = IllegalStateException("expected getter builder failure")
    try {
        getterOwnerNew(true) { owner, _, _, _, _, _, _, leaf ->
            escapedOwner = owner; escapedLeaf = leaf; throw original
        }
        error("expected builder failure")
    } catch (failure: IllegalStateException) { check(failure === original) }
    check(escapedOwner?.isClosed == true && escapedLeaf?.isClosed == true)
    val afterBuilder = getterSnapshot()
    check(afterBuilder[0] == beforeBuilder[0] + 1 && afterBuilder[2] == beforeBuilder[2] + 2)

    val beforePanic = getterSnapshot()
    var invoked = false
    try {
        getterOwnerFailure { _, _, _ -> invoked = true }
        error("expected conversion failure")
    } catch (_: IllegalArgumentException) { }
    val afterPanic = getterSnapshot()
    check(!invoked && afterPanic[0] == beforePanic[0] + 1)
    check(afterPanic[1] == beforePanic[1] + 1 && afterPanic[2] == beforePanic[2] + 2)

    for (present in listOf(false, true)) {
        val before = getterSnapshot()
        val failure = AtomicReference<Throwable?>()
        var calls = 0
        getterCallback({ number, name, optionalNumber, optionalName, details, rows, leaf ->
            try {
                calls++
                checkGetterViews(number, name, optionalNumber, optionalName, details, rows, present)
                check(getterLeafId(leaf) == 19L)
            } catch (error: Throwable) { failure.set(error) }
            finally { leaf.close() }
            if (present) throw IllegalStateException("expected observer failure")
        }, present)
        failure.get()?.let { throw it }
        val after = getterSnapshot()
        check(calls == 1 && after[0] == before[0] + 1)
        check(after[1] == before[1] + 1 && after[2] == before[2] + 2)
    }
    println("borrowed getters PASS: nested/optional views, records, slices, ordering, clones, rollback and callbacks")
}
