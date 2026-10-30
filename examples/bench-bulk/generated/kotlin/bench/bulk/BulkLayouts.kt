package bench.bulk

internal object BulkLayouts {
private val data = JNINative.bulkLayouts()
init { require(data.size == 68 && data[0] == 1L && data[1] == 6794024638180570187L && data[2] in 0L..1L && (data[3] == 4L || data[3] == 8L)) { "incompatible bulk layout catalogue" } }
val byteOrder = if(data[2] == 0L) java.nio.ByteOrder.LITTLE_ENDIAN else java.nio.ByteOrder.BIG_ENDIAN
val r1800d3dd3eceec6d = read(4, intArrayOf(16))
val r633c6cf82d3acfc1 = read(7, intArrayOf(8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8))
val r789cd54eb117685f = read(41, intArrayOf(8, 8, 8, 8, 8, 8, 8, 8))
val ra60d3960993b9a77 = read(51, intArrayOf(8))
val re5733c882bf9c317 = read(54, intArrayOf(8, 8, 8, 8, 8, 8, 8, 8, 16))
val rf4d5524a08b9867b = read(65, intArrayOf(16))
private fun read(at: Int, widths: IntArray): BulkLayout {
val size = data[at]; val alignment = data[at + 1]
require(size > 0 && size <= Int.MAX_VALUE && alignment > 0 && alignment <= Int.MAX_VALUE && (alignment and (alignment - 1)) == 0L && size % alignment == 0L) { "invalid bulk record layout" }
val offsets = IntArray(widths.size) { i -> val value = data[at + 2 + i]; require(value >= 0 && value <= size - widths[i]) { "invalid bulk field offset" }; value.toInt() }
return BulkLayout(size.toInt(), alignment.toInt(), offsets)
}
}
