@file:Suppress("ASSIGNED_BUT_NEVER_ACCESSED_VARIABLE", "UNUSED_VALUE")
package bench.bulk

internal object BulkInputs {
internal class Input_Range_49da87bb4c377b9a : BulkHandleWriter(BulkLayouts.r49da87bb4c377b9a.size) {
override fun patchHandles() {
var handleIndex = 0
val rows0Start = long((0) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
handleIndex = patchHandle((rows0Start + rows0Index * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0], handleIndex)
rows0Index++
}
}
}
}
internal class Input_Range_cb0e46a0a248e38c : BulkHandleWriter(BulkLayouts.rcb0e46a0a248e38c.size) {
override fun patchHandles() {
var handleIndex = 0
val rows0Start = long((0) + bench.bulk.BulkLayouts.rcb0e46a0a248e38c.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + bench.bulk.BulkLayouts.rcb0e46a0a248e38c.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
handleIndex = patchHandle((rows0Start + rows0Index * bench.bulk.BulkLayouts.r06de1c5aa99699f6.size) + bench.bulk.BulkLayouts.r06de1c5aa99699f6.offsets[0], handleIndex)
rows0Index++
}
}
}
}
internal class Input_Range_f738f16d97296060 : BulkHandleWriter(BulkLayouts.rf738f16d97296060.size) {
override fun patchHandles() {
var handleIndex = 0
val rows0Start = long((0) + bench.bulk.BulkLayouts.rf738f16d97296060.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + bench.bulk.BulkLayouts.rf738f16d97296060.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
handleIndex = patchHandle((rows0Start + rows0Index * bench.bulk.BulkLayouts.rd159346acd2a185e.size) + bench.bulk.BulkLayouts.rd159346acd2a185e.offsets[0], handleIndex)
rows0Index++
}
}
}
}
}
