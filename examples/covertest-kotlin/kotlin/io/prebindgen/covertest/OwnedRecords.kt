package io.prebindgen.covertest

import io.prebindgen.covertest.resources.*

fun checkOwnedRecords() {
    val cloneStart = projectionClones()
    val dropStart = projectionDrops()
    val borrowedPair = projectionPairBorrowed()
    check(projectionClones() == cloneStart + 2uL)
    check(projectionDrops() == dropStart + 2uL)
    borrowedPair.first.close(); borrowedPair.second.close()
    check(projectionDrops() == dropStart + 4uL)
    val ownedPair = projectionPairOwned()
    check(projectionClones() == cloneStart + 2uL)
    check(projectionDrops() == dropStart + 4uL)
    ownedPair.first.close(); ownedPair.second.close()
    check(projectionDrops() == dropStart + 6uL)
    val mixedPair = projectionPairMixed()
    check(projectionClones() == cloneStart + 3uL)
    check(projectionDrops() == dropStart + 7uL)
    mixedPair.first.close(); mixedPair.second.close()
    check(projectionDrops() == dropStart + 9uL)
    var movedView: ProjectionPair? = null
    var clonedView: ProjectionPair? = null
    val viewFailure = IllegalStateException("mixed delivery")
    try {
        projectionPairViews { moved, cloned ->
            movedView = moved; clonedView = cloned
            check(moved.first !== cloned.first)
            throw viewFailure
        }
        error("expected mixed delivery failure")
    } catch (e: IllegalStateException) { check(e === viewFailure) }
    check(checkNotNull(movedView).first.isClosed && checkNotNull(movedView).second.isClosed)
    check(checkNotNull(clonedView).first.isClosed && checkNotNull(clonedView).second.isClosed)
    check(projectionClones() == cloneStart + 5uL)
    check(projectionDrops() == dropStart + 13uL)

    val before = ownedResourceDrops()
    val bundle = resourceBundleNew()
    check(bundle.label == "owned bundle")
    check(bundle.details.bytes.contentEquals(byteArrayOf(1, 2, 3)))
    check(bundle.details.f == 1.5f && bundle.details.d == 2.5)
    check(ownedResourceId(bundle.pair.first) == 1L)
    check(ownedResourceId(bundle.pair.second) == 2L)
    check(ownedResourceId(bundle.tail) == 3L)
    check(ownedResourceDrops() == before)
    val copy = bundle.copy(label = "shared handles")
    check(copy.pair === bundle.pair && copy.tail === bundle.tail)
    bundle.pair.first.close()
    check(copy.pair.first.isClosed)
    bundle.pair.second.close()
    copy.tail.close()
    bundle.tail.close()
    check(ownedResourceDrops() == before + 3uL)

    // Observation runs before decomposition, even though the owned view is first.
    io.prebindgen.covertest.resourceViews.resourceBundleViews { owned, label ->
        check(label == owned.label)
        owned.pair.first.close(); owned.pair.second.close(); owned.tail.close()
    }
    check(ownedResourceDrops() == before + 6uL)

    // A throwing Kotlin delivery callback closes every nested handle once.
    var escaped: ResourceBundle? = null
    val failure = IllegalStateException("delivery failed")
    try {
        io.prebindgen.covertest.resourceViews.resourceBundleViews { owned, _ ->
            escaped = owned
            owned.pair.first.close() // rollback must tolerate an already closed member
            throw failure
        }
        error("expected failure")
    } catch (caught: IllegalStateException) { check(caught === failure) }
    val failed = checkNotNull(escaped)
    check(failed.pair.first.isClosed && failed.pair.second.isClosed && failed.tail.isClosed)
    failed.tail.close()
    check(ownedResourceDrops() == before + 9uL)

    // Type-wide owned policy, separately from resourceBundleNew's field override.
    val defaultBundle = resourceBundleDefault()
    check(defaultBundle.details.bytes.contentEquals(byteArrayOf(1, 2, 3)))
    defaultBundle.pair.first.close(); defaultBundle.pair.second.close(); defaultBundle.tail.close()
    check(ownedResourceDrops() == before + 12uL)
    // Consuming accessor plus per-field expansion overrides.
    resourceBundleFields { label, pair, tail, details ->
        check(label == "owned bundle" && details.f == 1.5f)
        check(ownedResourceId(pair.first) == 1L && ownedResourceId(tail) == 3L)
        pair.first.close(); pair.second.close(); tail.close()
    }
    check(ownedResourceDrops() == before + 15uL)
}
