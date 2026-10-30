package io.prebindgen.covertest

import io.prebindgen.covertest.defaults.*

fun checkBulkDefaults() {
    val expected = listOf(DefaultBulkRow(Long.MIN_VALUE, 1.5), DefaultBulkRow(Long.MAX_VALUE, -2.0))
    check(defaultBulkEcho(expected) == expected)
    check(defaultBulkEcho(emptyList()).isEmpty())
    check(defaultBulkArray() == expected)
    check(defaultBulkOptional(true) == expected)
    check(defaultBulkOptional(false) == null)
    check(defaultBulkNumbers() == listOf(Long.MIN_VALUE, 0, Long.MAX_VALUE))
    check(defaultBulkPlain() == expected) // Local ordinary-output override.
    check(defaultBulkArrayFold(0L) { sum, row -> sum + row.number } == -1L)
    var calls = 0
    defaultBulkCallback(DefaultBulkRowVecCallback { rows -> calls++; check(rows == expected) })
    check(calls == 1)
    repeat(1000) { check(defaultBulkEcho(expected) == expected) }
}
