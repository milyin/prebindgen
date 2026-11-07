@file:Suppress("ASSIGNED_BUT_NEVER_ACCESSED_VARIABLE", "UNUSED_VALUE")
package io.prebindgen.covertest.rawinputs

internal object BulkInputs {
internal class Input_Range_5019fbfb9686d1a8 : BulkHandleWriter(BulkLayouts.r5019fbfb9686d1a8.size, 1) {
override fun patchHandles() {
var handleIndex0 = 0
val rows0Start = long((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r5019fbfb9686d1a8.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r5019fbfb9686d1a8.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
val rows1Start = long((rows0Start + rows0Index * io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.offsets[1]).toInt()
if (rows1Start >= 0) {
val rows1Count = long(((rows0Start + rows0Index * io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.offsets[1]) + 8).toInt()
var rows1Index = 0
while (rows1Index < rows1Count) {
handleIndex0 = patchHandle((rows1Start + rows1Index * io.prebindgen.covertest.rawinputs.BulkLayouts.rcbe830243575e259.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.rcbe830243575e259.offsets[0], handleIndex0, 0)
rows1Index++
}
}
rows0Index++
}
}
}
}
internal class Input_Range_9c778fd2ce1e4e75 : BulkHandleWriter(BulkLayouts.r9c778fd2ce1e4e75.size, 1) {
override fun patchHandles() {
var handleIndex0 = 0
val rows0Start = long((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r9c778fd2ce1e4e75.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r9c778fd2ce1e4e75.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
handleIndex0 = patchHandle((rows0Start + rows0Index * io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.offsets[0], handleIndex0, 0)
handleIndex0 = patchHandle((rows0Start + rows0Index * io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.offsets[6], handleIndex0, 0)
rows0Index++
}
}
}
}
internal class Input_i64_i64_8afc2761c446bf98 : BulkHandleWriter(BulkLayouts.r8afc2761c446bf98.size, 2) {
override fun patchHandles() {
var handleIndex0 = 0
var handleIndex1 = 0
handleIndex0 = patchHandle((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r8afc2761c446bf98.offsets[0], handleIndex0, 0)
handleIndex1 = patchHandle((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r8afc2761c446bf98.offsets[1], handleIndex1, 1)
}
}
}
