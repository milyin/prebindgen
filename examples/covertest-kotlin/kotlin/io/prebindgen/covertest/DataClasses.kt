package io.prebindgen.covertest

import io.prebindgen.covertest.model.BlobValue
import io.prebindgen.covertest.model.Stamp
import io.prebindgen.covertest.model.Unsigned
import io.prebindgen.covertest.model.blobValueEcho
import io.prebindgen.covertest.values.*
import java.math.BigInteger

private fun invalidData(action: () -> Unit) {
    try { action() } catch (_: IllegalArgumentException) { return }
    error("expected data conversion failure")
}

fun checkDataClasses() {
    val bytes = byteArrayOf(0, -1, 127, -128)
    val blob = BlobValue(Stamp(Long.MIN_VALUE, Long.MAX_VALUE), bytes)
    val same = BlobValue(Stamp(Long.MIN_VALUE, Long.MAX_VALUE), bytes.copyOf())
    check(blob == same && blob.hashCode() == same.hashCode())
    check(blob.toString().contains("[0, -1, 127, -128]"))
    check(blob.copy(id = byteArrayOf(1)) != blob)
    check(blob.copy(stamp = blob.stamp.copy(secs = 9)) != blob)
    check(blobValueEcho(blob) == blob)
    val (stamp, id) = blob
    check(stamp == blob.stamp && id.contentEquals(bytes))
    val unsigned = Unsigned(255, 65535, 4294967295L, ULong.MAX_VALUE)
    val min128 = BigInteger.ONE.shiftLeft(127).negate()
    val max128 = BigInteger.ONE.shiftLeft(128).subtract(BigInteger.ONE)
    val envelope = Envelope("héllo\u0000世界", blob, unsigned, true, 0x10ffff, min128, max128)
    val storage = io.prebindgen.covertest.storage.storageNew()
    invalidData { dataWithStorage(storage, envelope.copy(code = 0xd800)) }
    check(!storage.isClosed) // data validation must precede native handle transfer
    dataWithStorage(storage, envelope).close()
    check(storage.isClosed)
    val constructed = dataEnvelopeConstructed(blob)
    check(constructed.blob == blob && constructed.label == "from blob")
    val returned = dataEnvelopeEcho(envelope)
    check(returned == envelope && returned.hashCode() == envelope.hashCode())
    check(returned.blob.id !== bytes) // bytes are copied through Rust/JNI
    check(dataEnvelopeEcho(envelope.copy(label = "", enabled = false, code = 0)) == envelope.copy(label = "", enabled = false, code = 0))
    invalidData { dataEnvelopeEcho(envelope.copy(unsigned = unsigned.copy(byte = 256))) }
    invalidData { dataEnvelopeEcho(envelope.copy(code = 0xd800)) }
    invalidData { dataEnvelopeEcho(envelope.copy(signed = min128.subtract(BigInteger.ONE))) }
    invalidData { dataEnvelopeEcho(envelope.copy(wide = max128.add(BigInteger.ONE))) }
    // Each selected arm uses the same data class differently; inactive arms are
    // not read, and can contain null/invalid values without failing conversion.
    check(blobValueSelected(0, blob, 0, null) == blob)
    check(blobValueSelected(1, null, 19, bytes) == BlobValue(Stamp(19, 0), bytes))
    invalidData { blobValueSelected(2, null, 0, null) }
    invalidData { blobValueSelected(0, null, 0, null) }
    // Even invalid, unselected class fields must not be decoded.
    val primitives = dataPrimitivesZero().copy(i8v = Byte.MIN_VALUE, i16v = Short.MIN_VALUE,
        i32v = Int.MIN_VALUE, i64v = Long.MIN_VALUE, i128v = min128, isizev = -123L, usizev = 456UL,
        u8v = 255, u16v = 65535, u32v = 4294967295L, u64v = ULong.MAX_VALUE,
        u128v = max128, f32v = -0.0f, f64v = Double.POSITIVE_INFINITY,
        boolv = true, charv = 0x10ffff)
    check(dataPrimitivesEcho(0, primitives) == primitives)
    check(dataPrimitivesEcho(1, null) == dataPrimitivesZero())
    check(dataPrimitivesEcho(1, primitives.copy(u8v = 999)) == dataPrimitivesZero())
    invalidData { dataPrimitivesEcho(0, null) }
    invalidData { dataPrimitivesEcho(0, primitives.copy(u8v = 999)) }
    blobValueViews(blob) { first, second, projectedStamp ->
        check(first == blob && second == blob && projectedStamp == blob.stamp)
        check(first.id !== second.id)
    }
    val failure = IllegalStateException("data builder failure")
    try {
        blobValueViews(blob) { _, _, _ -> throw failure }
        error("expected builder exception")
    } catch (caught: IllegalStateException) { check(caught === failure) }
    check(blobValueEcho(blob) == blob)
    val nan = DataFloats(bytes, Float.NaN, Double.NaN)
    check(dataFloatsEcho(nan) == nan)
    check(nan == nan.copy(bytes = bytes.copyOf()))
    check(nan.hashCode() == nan.copy(bytes = bytes.copyOf()).hashCode())
    val zero = DataFloats(bytes, 0.0f, 0.0)
    val negative = DataFloats(bytes.copyOf(), -0.0f, -0.0)
    check(zero != negative)
    check(dataFloatsEcho(negative) == negative)
}
