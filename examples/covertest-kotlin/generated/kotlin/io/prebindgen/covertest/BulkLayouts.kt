package io.prebindgen.covertest

internal object BulkLayouts {
private val data = JNINative.bulkLayouts()
init { require(data.size == 141 && data[0] == 1L && data[1] == 4179160881830042015L && data[2] in 0L..1L && (data[3] == 4L || data[3] == 8L)) { "incompatible bulk layout catalogue" } }
val byteOrder = if(data[2] == 0L) java.nio.ByteOrder.LITTLE_ENDIAN else java.nio.ByteOrder.BIG_ENDIAN
val r01c263066f8f1a73 = read(4, intArrayOf(16))
val r062153e85348d725 = read(7, intArrayOf(4))
val r0b9b7f90ad764db3 = read(10, intArrayOf(8, 16))
val r1800d3dd3eceec6d = read(14, intArrayOf(16))
val r220da08e0b780b69 = read(17, intArrayOf(4, 8, 8))
val r31d2b5a17a152d89 = read(22, intArrayOf(16))
val r3711b7e845f5d8b9 = read(25, intArrayOf(8))
val r37f6d28aa836bfb7 = read(28, intArrayOf(8, 16))
val r42eab86179b36709 = read(32, intArrayOf(16))
val r49be3e7148c9ff8f = read(35, intArrayOf(16))
val r535f9e324da6eaa9 = read(38, intArrayOf(8, 16))
val r554fa1d6f8692db9 = read(42, intArrayOf(8, 8))
val r638fef00b45d8f09 = read(46, intArrayOf(8))
val r659298f90cfa7005 = read(49, intArrayOf(16))
val r79427dd643a1b061 = read(52, intArrayOf(16))
val r807a81a0a53fa8e0 = read(55, intArrayOf(4, 16))
val r87bd2f61e8a69677 = read(59, intArrayOf(16))
val r8a5c993ab6685b67 = read(62, intArrayOf(1, 1, 2, 4, 4, 8, 4, 8))
val r904fefb3d01cb2ae = read(72, intArrayOf(1))
val r963ae3ba49e0cb2b = read(75, intArrayOf(16))
val r97145e0ef6a1193d = read(78, intArrayOf(8))
val r98c254983b527011 = read(81, intArrayOf(16))
val ra5f90e11a63369ed = read(84, intArrayOf(16))
val ra60d3960993b9a77 = read(87, intArrayOf(8))
val ra8ec97310260b4fd = read(90, intArrayOf(16))
val rba88ec47a5d9bc85 = read(93, intArrayOf(16))
val rc8d6433a01ad4efb = read(96, intArrayOf(4, 16))
val rc958cabd18f5a53a = read(100, intArrayOf(1))
val rcbe830243575e259 = read(103, intArrayOf(8))
val rd515d9783303b819 = read(106, intArrayOf(8))
val rdafeaae0e426fdd5 = read(109, intArrayOf(4, 8, 8))
val rdc22244d89c2587b = read(114, intArrayOf(4))
val re048ef6766326a01 = read(117, intArrayOf(16))
val re0c6fa25bb7482b5 = read(120, intArrayOf(16))
val re5f8866547bdf41a = read(123, intArrayOf(4, 16))
val ree997f40a731b1db = read(127, intArrayOf(8, 8))
val rf27ab2e6065597a6 = read(131, intArrayOf(4, 16))
val rf41b5b157c049db1 = read(135, intArrayOf(16))
val rf4f41f2e97b83c0c = read(138, intArrayOf(16))
private fun read(at: Int, widths: IntArray): BulkLayout {
val size = data[at]; val alignment = data[at + 1]
require(size > 0 && size <= Int.MAX_VALUE && alignment > 0 && alignment <= Int.MAX_VALUE && (alignment and (alignment - 1)) == 0L && size % alignment == 0L) { "invalid bulk record layout" }
val offsets = IntArray(widths.size) { i -> val value = data[at + 2 + i]; require(value >= 0 && value <= size - widths[i]) { "invalid bulk field offset" }; value.toInt() }
return BulkLayout(size.toInt(), alignment.toInt(), offsets)
}
}
