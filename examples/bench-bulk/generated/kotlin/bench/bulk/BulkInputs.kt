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
internal class Input_Range_50871c2a8b72df4b : BulkHandleWriter(BulkLayouts.r50871c2a8b72df4b.size) {
override fun patchHandles() {
var handleIndex = 0
val rows0Start = long((0) + bench.bulk.BulkLayouts.r50871c2a8b72df4b.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + bench.bulk.BulkLayouts.r50871c2a8b72df4b.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
handleIndex = patchHandle((rows0Start + rows0Index * bench.bulk.BulkLayouts.r9cc3e9775849e9ab.size) + bench.bulk.BulkLayouts.r9cc3e9775849e9ab.offsets[0], handleIndex)
rows0Index++
}
}
}
}
internal class Input_Range_5e24e8a1a6fd1960 : BulkHandleWriter(BulkLayouts.r5e24e8a1a6fd1960.size) {
override fun patchHandles() {
var handleIndex = 0
val rows0Start = long((0) + bench.bulk.BulkLayouts.r5e24e8a1a6fd1960.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + bench.bulk.BulkLayouts.r5e24e8a1a6fd1960.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
val rows1Start = long((rows0Start + rows0Index * bench.bulk.BulkLayouts.r49da87bb4c377b9a.size) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0]).toInt()
if (rows1Start >= 0) {
val rows1Count = long(((rows0Start + rows0Index * bench.bulk.BulkLayouts.r49da87bb4c377b9a.size) + bench.bulk.BulkLayouts.r49da87bb4c377b9a.offsets[0]) + 8).toInt()
var rows1Index = 0
while (rows1Index < rows1Count) {
handleIndex = patchHandle((rows1Start + rows1Index * bench.bulk.BulkLayouts.rb2426f0d5f2715c0.size) + bench.bulk.BulkLayouts.rb2426f0d5f2715c0.offsets[0], handleIndex)
rows1Index++
}
}
rows0Index++
}
}
}
}
internal class Input_Range_71300b84e922dbba : BulkHandleWriter(BulkLayouts.r71300b84e922dbba.size) {
override fun patchHandles() {
var handleIndex = 0
val rows0Start = long((0) + bench.bulk.BulkLayouts.r71300b84e922dbba.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + bench.bulk.BulkLayouts.r71300b84e922dbba.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
handleIndex = patchHandle((rows0Start + rows0Index * bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.size) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0], handleIndex)
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
internal class Input_i64_i64_b3c08eb92fbbc74c : BulkHandleWriter(BulkLayouts.rb3c08eb92fbbc74c.size) {
override fun patchHandles() {
var handleIndex = 0
handleIndex = patchHandle((0) + bench.bulk.BulkLayouts.rb3c08eb92fbbc74c.offsets[0], handleIndex)
}
}
}
