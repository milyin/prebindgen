package bench.bulk

internal object BulkLayouts {
private val data = JNINative.bulkLayouts()
init { require(data.size == 112 && data[0] == 1L && data[1] == 4011831285180525180L && data[2] in 0L..1L && (data[3] == 4L || data[3] == 8L)) { "incompatible bulk layout catalogue" } }
val byteOrder = if(data[2] == 0L) java.nio.ByteOrder.LITTLE_ENDIAN else java.nio.ByteOrder.BIG_ENDIAN
val r0643563939dde3e9 = read(4, intArrayOf(8, 16))
val r06de1c5aa99699f6 = read(8, intArrayOf(8, 8))
val r1800d3dd3eceec6d = read(12, intArrayOf(16))
val r49da87bb4c377b9a = read(15, intArrayOf(16))
val r50871c2a8b72df4b = read(18, intArrayOf(16))
val r5e24e8a1a6fd1960 = read(21, intArrayOf(16))
val r633c6cf82d3acfc1 = read(24, intArrayOf(8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8))
val r71300b84e922dbba = read(58, intArrayOf(16))
val r789cd54eb117685f = read(61, intArrayOf(8, 8, 8, 8, 8, 8, 8, 8))
val r9cc3e9775849e9ab = read(71, intArrayOf(8, 16, 16))
val ra60d3960993b9a77 = read(76, intArrayOf(8))
val rb2426f0d5f2715c0 = read(79, intArrayOf(8))
val rb3c08eb92fbbc74c = read(82, intArrayOf(8, 8))
val rcb0e46a0a248e38c = read(86, intArrayOf(16))
val rd159346acd2a185e = read(89, intArrayOf(8))
val rd5ee2af5a2eda167 = read(92, intArrayOf(16))
val re5733c882bf9c317 = read(95, intArrayOf(8, 8, 8, 8, 8, 8, 8, 8, 16))
val rf4d5524a08b9867b = read(106, intArrayOf(16))
val rf738f16d97296060 = read(109, intArrayOf(16))
private fun read(at: Int, widths: IntArray): BulkLayout {
val size = data[at]; val alignment = data[at + 1]
require(size > 0 && size <= Int.MAX_VALUE && alignment > 0 && alignment <= Int.MAX_VALUE && (alignment and (alignment - 1)) == 0L && size % alignment == 0L) { "invalid bulk record layout" }
val offsets = IntArray(widths.size) { i -> val value = data[at + 2 + i]; require(value >= 0 && value <= size - widths[i]) { "invalid bulk field offset" }; value.toInt() }
return BulkLayout(size.toInt(), alignment.toInt(), offsets)
}
}
