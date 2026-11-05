package io.prebindgen.covertest

import io.prebindgen.covertest.paramnames.*

fun checkParameterNames() {
    check(parameterSum(pointX = 7L, pointY = 11L) == 18L)
    check(parameterBulkSum(pointX = 7L, pointY = 11L) == 18L)
    check(parameterSumObject(point = ParameterPoint(7L, 11L)) == 18L)
    check(parameterOptional(pointPresent = false, pointX = 7L, pointY = 11L) == -1L)
    check(parameterOptional(pointPresent = true, pointX = 7L, pointY = 11L) == 18L)
    check(parameterVariants(pointVariant = 0, pointVariant0 = ParameterPoint(7L, 11L), pointVariant1X = 0L, pointVariant1Y = 0L) == 18L)
    check(parameterVariants(pointVariant = 1, pointVariant0 = null, pointVariant1X = 7L, pointVariant1Y = 11L) == 18L)
    val value = WireNamedFields(p0_w1 = 9L, p1_w1 = "p0_w1 p1_w1")
    check(parameterRecord(value = value) == value)
    check(parameterRecords(items = listOf(value)) == listOf(value))
    check(parameterCollisions(
        fooBar_feb41f8be0a636b4 = 1L,
        fooBar_4f2f13b8615afe7e = 2L,
        onError_b339b5ed021da585 = 3L,
        returnCapture_158ff1cab4233194 = 4L,
        `when` = 5L,
        io_c6e3d58b26b43793 = 6L,
    ) == 654321L)
    var delivered = false
    val external = com.prebindgen.paramnames.parameterExternal(3L, 7L, callback = {
        check(it.x == 3L && it.y == 7L)
        delivered = true
    })
    check(delivered && external.x == 3L && external.y == 7L)
    println("parameter names PASS: named arguments, constructors, local overrides, bulk, options, selectors and wire-like member names")
}
