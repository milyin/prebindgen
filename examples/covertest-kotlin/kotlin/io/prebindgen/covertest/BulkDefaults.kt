package io.prebindgen.covertest

import io.prebindgen.covertest.defaults.*

fun checkBulkDefaults() {
    val expected = listOf(DefaultBulkRow(Long.MIN_VALUE, 1.5), DefaultBulkRow(Long.MAX_VALUE, -2.0))
    check(defaultBulkEcho(expected) == expected)
    check(defaultBulkEcho(emptyList()).isEmpty())
    check(defaultBulkArray() == expected)
    check(defaultBulkOptional(2) == expected)
    check(defaultBulkOptional(1)?.isEmpty() == true)
    check(defaultBulkOptional(0) == null)
    check(defaultBulkBorrowed() == expected)
    check(defaultBulkNested() == listOf(listOf(Long.MIN_VALUE, 0L, Long.MAX_VALUE), emptyList<Long>()))
    check(defaultBulkOptionalFold(0, 9L) { sum, row -> sum + row.number } == null)
    check(defaultBulkOptionalFold(1, 9L) { sum, row -> sum + row.number } == 9L)
    check(defaultBulkOptionalFold(2, 0L) { sum, row -> sum + row.number } == -1L)
    check(defaultBulkNumbers() == listOf(Long.MIN_VALUE, 0, Long.MAX_VALUE))
    check(defaultBulkPlain() == expected) // Local ordinary-output override.
    check(defaultBulkArrayFold(0L) { sum, row -> sum + row.number } == -1L)
    var calls = 0
    var delivered: List<DefaultBulkRow>? = null
    defaultBulkCallback(DefaultBulkRowVecCallback { rows -> calls++; delivered = rows })
    check(calls == 1 && delivered == expected)
    val nested = listOf(listOf(Long.MIN_VALUE, 0L, Long.MAX_VALUE), emptyList<Long>())
    var ordinary: List<List<Long>>? = null
    var packed: List<List<Long>>? = null
    defaultBulkCallbackMixed(DefaultBulkMixedCallback { first, second -> ordinary = first; packed = second })
    check(ordinary == nested && packed == nested)
    repeat(1000) { check(defaultBulkEcho(expected) == expected) }
}
