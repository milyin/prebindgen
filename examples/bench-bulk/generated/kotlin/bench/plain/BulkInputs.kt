@file:Suppress("ASSIGNED_BUT_NEVER_ACCESSED_VARIABLE", "UNUSED_VALUE")
package bench.plain

internal object BulkInputs {
internal class Input_Range_71300b84e922dbba : BulkHandleWriter(BulkLayouts.r71300b84e922dbba.size, 1) {
override fun patchHandles() {
var handleIndex0 = 0
val rows0Start = long((0) + bench.plain.BulkLayouts.r71300b84e922dbba.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + bench.plain.BulkLayouts.r71300b84e922dbba.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
handleIndex0 = patchHandle((rows0Start + rows0Index * bench.plain.BulkLayouts.rb3c08eb92fbbc74c.size) + bench.plain.BulkLayouts.rb3c08eb92fbbc74c.offsets[0], handleIndex0, 0)
rows0Index++
}
}
}
}
}
