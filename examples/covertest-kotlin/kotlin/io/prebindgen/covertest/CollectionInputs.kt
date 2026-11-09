package io.prebindgen.covertest

import io.prebindgen.covertest.collectioninputs.*

fun checkCollectionInputs() {
    for (size in listOf(0, 1, 8, 64, 512, 4096)) {
        val values = List(size) { it.toLong() }
        val sum = values.sum()
        check(collectionInputPlain(values, 7) == sum + 7)
        check(collectionInputBulk(values, 7) == sum + 7)
        check(collectionInputSlice(values) == sum)
        check(collectionInputOptional(values) == sum)
        for (bulk in listOf(false, true)) {
            val before = collectionInputConstructions()
            var rejected = false
            if (bulk) collectionInputCustomBulk(values, 256, onError = { rejected = true; -1L })
            else collectionInputCustom(values, 256, onError = { rejected = true; -1L })
            check(rejected && collectionInputConstructions() == before + size.toULong())
            val result = if (bulk) collectionInputCustomBulk(values, 7) else collectionInputCustom(values, 7)
            check(result == sum + 7 && collectionInputConstructions() == before + 2uL * size.toULong())
        }
    }
    check(collectionInputOptionalArray(null) == -1L)
    check(collectionInputOptionalArrayBulk(null) == -1L)
    check(collectionInputOptionalArray(listOf(1L, 2L, 3L, 4L)) == 10L)
    check(collectionInputOptionalArrayBulk(listOf(1L, 2L, 3L, 4L)) == 10L)
    for (size in listOf(0, 3, 5)) {
        var rejected = 0
        collectionInputOptionalArray(List(size) { 1L }, onError = { rejected++; -1L })
        collectionInputOptionalArrayBulk(List(size) { 1L }, onError = { rejected++; -1L })
        check(rejected == 2)
    }
    check(collectionInputOptional(null) == -1L)
    check(collectionInputArray(listOf(-7, 0, 9, 12).map { it.toLong() }) == 14L)
    for (size in listOf(0, 3, 5)) {
        var rejected = false
        collectionInputArray(List(size) { 1L }, onError = { rejected = true; -1L })
        check(rejected)
    }
    // Earlier rows are constructed before a later invalid row is encountered.
    for (bulk in listOf(false, true)) {
        val before = collectionInputConstructions()
        var rejected = false
        if (bulk) collectionInputNarrowBulk(listOf(1, 256, 3), onError = { rejected = true; -1L })
        else collectionInputNarrow(listOf(1, 256, 3), onError = { rejected = true; -1L })
        check(rejected && collectionInputConstructions() == before + 1uL)
        val result = if (bulk) collectionInputNarrowBulk(listOf(1, 2, 3)) else collectionInputNarrow(listOf(1, 2, 3))
        check(result == 6L && collectionInputConstructions() == before + 4uL)
    }
    println("collection inputs PASS: primitive ordinary/bulk inputs, slices, arrays, options and eager constructors")
}
