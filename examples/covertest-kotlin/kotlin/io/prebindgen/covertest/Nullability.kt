package io.prebindgen.covertest

import io.prebindgen.covertest.nullability.*
import java.math.BigInteger

// Test-only: emulate a Java or erased-generic value violating a non-null type.
@Suppress("UNCHECKED_CAST")
internal fun <T> invalidJavaNull(): T = null as T

internal fun rejectNullPrecondition(body: () -> Unit) {
    try { body() } catch (_: NullPointerException) { return }
    error("expected a non-null API precondition failure")
}

fun checkInputNullability() {
    val h = nullabilityHandleNew(11L)
    check(nullabilityHandleRead(h) == 11L)
    check(nullabilityHandleOptional(null) == -1L)
    check(nullabilityHandleOptional(h) == 11L)
    check(nullabilityText("abc") == 3L)
    check(nullabilityTextOptional(null) == -1L)
    check(nullabilityTextOptional("") == 0L)
    check(nullabilityBytes(byteArrayOf(1, 2)) == 2L)
    check(nullabilityBytesOptional(null) == -1L)
    check(nullabilityBytesOptional(byteArrayOf()) == 0L)
    val wide = BigInteger.valueOf(7L)
    check(nullabilityWide(wide) == wide)
    check(nullabilityWideOptional(null) == null)
    check(nullabilityWideOptional(wide) == wide)
    val row = NullabilityValue("abc", byteArrayOf(1, 2), wide, NullabilityChoice.B, 11L)
    check(nullabilityValues(listOf(row)) == 24L)
    check(nullabilityValuesOptional(null) == -1L)
    check(nullabilityValuesOptional(emptyList()) == 0L)
    check(nullabilityValuesOptional(listOf(row)) == 24L)
    check(nullabilityHandlesOptional(null) == -1L)
    check(nullabilityHandlesOptional(emptyList()) == 0L)
    val arrayHandle = nullabilityHandleNew(17L)
    check(nullabilityHandlesOptional(listOf(arrayHandle)) == 17L && arrayHandle.isClosed)
    check(nullabilityTextCallback({ "required" }) == "required")
    check(nullabilityOptionalTextCallback({ null }) == null)
    check(nullabilityOptionalTextCallback({ "optional" }) == "optional")

    // Inactive constructor references may be null; active ones are validated.
    check(nullabilityValueSum(0, row, null, null, null, null, null) == 24L)
    check(nullabilityValueBulk(0, row, null, null, null, null, null) == 24L)
    check(nullabilityValueSum(1, null, "abc", byteArrayOf(1, 2), wide, NullabilityChoice.B, h) == 24L)
    check(nullabilityValueBulk(1, null, "abc", byteArrayOf(1, 2), wide, NullabilityChoice.B, h) == 24L)
    val inactive = nullabilityHandleNew(99L)
    inactive.close()
    check(nullabilityValueSum(0, row, null, null, null, null, inactive) == 24L)
    check(nullabilityValueBulk(0, row, null, null, null, null, inactive) == 24L)
    check(nullabilityValueOptional(false, null, null, null, null, null) == -1L)
    check(nullabilityValueOptional(true, "abc", byteArrayOf(1, 2), wide, NullabilityChoice.B, h) == 24L)
    check(nullabilityCallbackChoose(0, row, null) == 24L)
    check(nullabilityCallbackChoose(1, null, { "abc" }) == 3L)
    var conditionalError = false
    check(nullabilityValueSum(1, null, "abc", byteArrayOf(1, 2), wide, NullabilityChoice.B, null,
        JniErrorHandler { conditionalError = true; -10L }) == -10L)
    check(conditionalError && !h.isClosed)
    h.close()
    println("input nullability PASS: required values, options, row element types and inactive constructor slots")
}
