package io.prebindgen.covertest.rawinputs

internal object BulkLayouts {
private val data = JNINative.bulkLayouts()
init { require(data.size == 40 && data[0] == 1L && data[1] == -5596972784569740004L && data[2] in 0L..1L && (data[3] == 4L || data[3] == 8L)) { "incompatible bulk layout catalogue" } }
val byteOrder = if(data[2] == 0L) java.nio.ByteOrder.LITTLE_ENDIAN else java.nio.ByteOrder.BIG_ENDIAN
val r1800d3dd3eceec6d = read(4, intArrayOf(16))
val r5019fbfb9686d1a8 = read(7, intArrayOf(16))
val r8afc2761c446bf98 = read(10, intArrayOf(8, 8, 16))
val r9c778fd2ce1e4e75 = read(15, intArrayOf(16))
val ra60d3960993b9a77 = read(18, intArrayOf(8))
val rae5d49bdac718245 = read(21, intArrayOf(16))
val rcbe830243575e259 = read(24, intArrayOf(8))
val re7fb16c195e57181 = read(27, intArrayOf(8, 4, 16, 4, 16, 4, 8))
val rf27ab2e6065597a6 = read(36, intArrayOf(4, 16))
private fun read(at: Int, widths: IntArray): BulkLayout {
val size = data[at]; val alignment = data[at + 1]
require(size > 0 && size <= Int.MAX_VALUE && alignment > 0 && alignment <= Int.MAX_VALUE && (alignment and (alignment - 1)) == 0L && size % alignment == 0L) { "invalid bulk record layout" }
val offsets = IntArray(widths.size) { i -> val value = data[at + 2 + i]; require(value >= 0 && value <= size - widths[i]) { "invalid bulk field offset" }; value.toInt() }
return BulkLayout(size.toInt(), alignment.toInt(), offsets)
}
}
