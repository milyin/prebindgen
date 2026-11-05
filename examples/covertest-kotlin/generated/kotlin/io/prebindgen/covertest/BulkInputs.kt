@file:Suppress("ASSIGNED_BUT_NEVER_ACCESSED_VARIABLE", "UNUSED_VALUE")
package io.prebindgen.covertest

internal object BulkInputs {
internal class Input_Range_2997fc5de2832bba : BulkHandleWriter(BulkLayouts.r2997fc5de2832bba.size) {
override fun patchHandles() {
var handleIndex = 0
val rows0Start = long((0) + io.prebindgen.covertest.BulkLayouts.r2997fc5de2832bba.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + io.prebindgen.covertest.BulkLayouts.r2997fc5de2832bba.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
handleIndex = patchHandle((rows0Start + rows0Index * io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.size) + io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.offsets[0], handleIndex)
handleIndex = patchHandle((rows0Start + rows0Index * io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.size) + io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.offsets[1], handleIndex)
rows0Index++
}
}
}
}
internal class Input_Range_42eab86179b36709 : BulkHandleWriter(BulkLayouts.r42eab86179b36709.size) {
override fun patchHandles() {
var handleIndex = 0
val rows0Start = long((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
handleIndex = patchHandle((rows0Start + rows0Index * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], handleIndex)
rows0Index++
}
}
}
}
internal class Input_Range_46147a291c22570d : BulkHandleWriter(BulkLayouts.r46147a291c22570d.size) {
override fun patchHandles() {
var handleIndex = 0
val rows0Start = long((0) + io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
val rows1Start = long((rows0Start + rows0Index * io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0]).toInt()
if (rows1Start >= 0) {
val rows1Count = long(((rows0Start + rows0Index * io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0]) + 8).toInt()
var rows1Index = 0
while (rows1Index < rows1Count) {
handleIndex = patchHandle((rows1Start + rows1Index * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], handleIndex)
rows1Index++
}
}
rows0Index++
}
}
}
}
internal class Input_Range_5019fbfb9686d1a8 : BulkHandleWriter(BulkLayouts.r5019fbfb9686d1a8.size) {
override fun patchHandles() {
var handleIndex = 0
val rows0Start = long((0) + io.prebindgen.covertest.BulkLayouts.r5019fbfb9686d1a8.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + io.prebindgen.covertest.BulkLayouts.r5019fbfb9686d1a8.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
val rows1Start = long((rows0Start + rows0Index * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1]).toInt()
if (rows1Start >= 0) {
val rows1Count = long(((rows0Start + rows0Index * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1]) + 8).toInt()
var rows1Index = 0
while (rows1Index < rows1Count) {
handleIndex = patchHandle((rows1Start + rows1Index * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0], handleIndex)
rows1Index++
}
}
rows0Index++
}
}
}
}
internal class Input_Range_ff46ac585710e08a : BulkHandleWriter(BulkLayouts.rff46ac585710e08a.size) {
override fun patchHandles() {
var handleIndex = 0
val rows0Start = long((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
handleIndex = patchHandle((rows0Start + rows0Index * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[0], handleIndex)
rows0Index++
}
}
}
}
internal class Input_i32_Range_6217a02e84ef490c : BulkHandleWriter(BulkLayouts.r6217a02e84ef490c.size) {
override fun patchHandles() {
var handleIndex = 0
handleIndex = patchHandle((0) + io.prebindgen.covertest.BulkLayouts.r6217a02e84ef490c.offsets[10], handleIndex)
}
}
internal class Input_i32_Range_f27ab2e6065597a6 : BulkHandleWriter(BulkLayouts.rf27ab2e6065597a6.size) {
override fun patchHandles() {
var handleIndex = 0
val rows0Start = long((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
handleIndex = patchHandle((rows0Start + rows0Index * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0], handleIndex)
rows0Index++
}
}
}
}
internal class Input_i64_638fef00b45d8f09 : BulkHandleWriter(BulkLayouts.r638fef00b45d8f09.size) {
override fun patchHandles() {
var handleIndex = 0
handleIndex = patchHandle((0) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], handleIndex)
}
}
internal class Input_i64_d515d9783303b819 : BulkHandleWriter(BulkLayouts.rd515d9783303b819.size) {
override fun patchHandles() {
var handleIndex = 0
handleIndex = patchHandle((0) + io.prebindgen.covertest.BulkLayouts.rd515d9783303b819.offsets[0], handleIndex)
}
}
internal class Input_i64_i32_7fe138bb3730f8e7 : BulkHandleWriter(BulkLayouts.r7fe138bb3730f8e7.size) {
override fun patchHandles() {
var handleIndex = 0
handleIndex = patchHandle((0) + io.prebindgen.covertest.BulkLayouts.r7fe138bb3730f8e7.offsets[0], handleIndex)
val rows0Start = long((0) + io.prebindgen.covertest.BulkLayouts.r7fe138bb3730f8e7.offsets[2]).toInt()
if (rows0Start >= 0) {
val rows0Count = long(((0) + io.prebindgen.covertest.BulkLayouts.r7fe138bb3730f8e7.offsets[2]) + 8).toInt()
var rows0Index = 0
while (rows0Index < rows0Count) {
val rows1Start = long((rows0Start + rows0Index * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1]).toInt()
if (rows1Start >= 0) {
val rows1Count = long(((rows0Start + rows0Index * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1]) + 8).toInt()
var rows1Index = 0
while (rows1Index < rows1Count) {
handleIndex = patchHandle((rows1Start + rows1Index * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0], handleIndex)
rows1Index++
}
}
rows0Index++
}
}
handleIndex = patchHandle((0) + io.prebindgen.covertest.BulkLayouts.r7fe138bb3730f8e7.offsets[3], handleIndex)
}
}
internal class Input_i64_i64_fd1551214c82cfe3 : BulkHandleWriter(BulkLayouts.rfd1551214c82cfe3.size) {
override fun patchHandles() {
var handleIndex = 0
handleIndex = patchHandle((0) + io.prebindgen.covertest.BulkLayouts.rfd1551214c82cfe3.offsets[0], handleIndex)
}
}
}
