package io.prebindgen.covertest

import io.prebindgen.covertest.primitives.*
import java.math.BigInteger

private fun rejects(action: () -> Unit) {
    try { action(); error("expected invalid primitive") }
    catch (_: IllegalArgumentException) { }
}

fun checkPrimitives() {
    for (v in listOf(Byte.MIN_VALUE, 0.toByte(), Byte.MAX_VALUE)) check(primitiveI8(v) == v)
    for (v in listOf(Short.MIN_VALUE, 0.toShort(), Short.MAX_VALUE)) check(primitiveI16(v) == v)
    for (v in listOf(Int.MIN_VALUE, 0, Int.MAX_VALUE)) check(primitiveI32(v) == v)
    for (v in listOf(Long.MIN_VALUE, 0L, Long.MAX_VALUE)) check(primitiveI64(v) == v)
    for (v in listOf(0, 255)) check(primitiveU8(v) == v)
    for (v in listOf(0, 65535)) check(primitiveU16(v) == v)
    for (v in listOf(0L, 4294967295L)) check(primitiveU32(v) == v)
    for (v in listOf(0UL, ULong.MAX_VALUE, 1UL shl 63)) check(primitiveU64(v) == v)
    rejects { primitiveU8(-1) }; rejects { primitiveU8(256) }
    rejects { primitiveU16(-1) }; rejects { primitiveU16(65536) }
    rejects { primitiveU32(-1) }; rejects { primitiveU32(4294967296L) }
    val bits = System.getProperty("sun.arch.data.model").toInt()
    val maxSigned = if (bits == 64) Long.MAX_VALUE else Int.MAX_VALUE.toLong()
    val minSigned = if (bits == 64) Long.MIN_VALUE else Int.MIN_VALUE.toLong()
    val maxUnsigned = if (bits == 64) ULong.MAX_VALUE else UInt.MAX_VALUE.toULong()
    check(primitiveIsize(minSigned) == minSigned)
    check(primitiveIsize(maxSigned) == maxSigned)
    check(primitiveUsize(maxUnsigned) == maxUnsigned)
    if (bits == 32) {
        rejects { primitiveIsize(maxSigned + 1) }
        rejects { primitiveIsize(minSigned - 1) }
        rejects { primitiveUsize(maxUnsigned + 1UL) }
    }
    for (v in listOf(false, true)) check(primitiveBool(v) == v)
    for (v in listOf(0, 0xD7FF, 0xE000, 0x1F642, 0x10FFFF)) check(primitiveChar(v) == v)
    for (v in listOf(-1, 0xD800, 0xDFFF, 0x110000)) rejects { primitiveChar(v) }
    for (v in listOf(0.0f, -0.0f, Float.MIN_VALUE, Float.MAX_VALUE, Float.NEGATIVE_INFINITY, Float.POSITIVE_INFINITY)) check(primitiveF32(v).toRawBits() == v.toRawBits())
    check(primitiveF32(Float.NaN).isNaN())
    for (v in listOf(0.0, -0.0, Double.MIN_VALUE, Double.MAX_VALUE, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY)) check(primitiveF64(v).toRawBits() == v.toRawBits())
    check(primitiveF64(Double.NaN).isNaN())
    val two127 = BigInteger.ONE.shiftLeft(127)
    val two128 = BigInteger.ONE.shiftLeft(128)
    for (v in listOf(-two127, BigInteger.valueOf(-1), BigInteger.ZERO, two127 - BigInteger.ONE)) check(primitiveI128(v) == v)
    for (v in listOf(BigInteger.ZERO, two127, two128 - BigInteger.ONE)) check(primitiveU128(v) == v)
    rejects { primitiveI128(two127) }; rejects { primitiveI128(-two127 - BigInteger.ONE) }
    rejects { primitiveU128(two128) }; rejects { primitiveU128(BigInteger.valueOf(-1)) }
    rejects { primitiveI128(null) }; rejects { primitiveU128(null) }
    primitiveUnit()
}
