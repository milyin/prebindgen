package bench.plain

internal object BulkLayouts {
private val data = JNINative.bulkLayouts()
init { require(data.size == 18 && data[0] == 1L && data[1] == 3176938622011698635L && data[2] in 0L..1L && (data[3] == 4L || data[3] == 8L)) { "incompatible bulk layout catalogue" } }
val byteOrder = if(data[2] == 0L) java.nio.ByteOrder.LITTLE_ENDIAN else java.nio.ByteOrder.BIG_ENDIAN
val r0643563939dde3e9 = read(4, intArrayOf(8, 16))
val r71300b84e922dbba = read(8, intArrayOf(16))
val rb3c08eb92fbbc74c = read(11, intArrayOf(8, 8))
val rd5ee2af5a2eda167 = read(15, intArrayOf(16))
private fun read(at: Int, widths: IntArray): BulkLayout {
val size = data[at]; val alignment = data[at + 1]
require(size > 0 && size <= Int.MAX_VALUE && alignment > 0 && alignment <= Int.MAX_VALUE && (alignment and (alignment - 1)) == 0L && size % alignment == 0L) { "invalid bulk record layout" }
val offsets = IntArray(widths.size) { i -> val value = data[at + 2 + i]; require(value >= 0 && value <= size - widths[i]) { "invalid bulk field offset" }; value.toInt() }
return BulkLayout(size.toInt(), alignment.toInt(), offsets)
}
}
