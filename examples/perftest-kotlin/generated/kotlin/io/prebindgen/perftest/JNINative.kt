package io.prebindgen.perftest

internal object JNINative {
    init { io.prebindgen.perftest.NativeLibrary.ensureLoaded() }
    @JvmStatic external fun drop0(id: Long)
    @JvmStatic external fun call0(): Array<Any?>
    @JvmStatic external fun call1(p0_w1: Long): Array<Any?>
    @JvmStatic external fun call2(p0_w1: Long, p1_w1: Int, p2_w1: Double, p3_w1: Boolean): Array<Any?>
    @JvmStatic external fun call3(p0_w2: Long, p0_w4: Int, p0_w6: Double, p0_w8: Boolean): Array<Any?>
    @JvmStatic external fun call4(p0_w1: Long, p1_w1: Int, p1_w4: Long, p1_w6: Int, p1_w8: Double, p1_w10: Boolean): Array<Any?>
}
