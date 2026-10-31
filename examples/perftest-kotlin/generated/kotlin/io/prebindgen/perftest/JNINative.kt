package io.prebindgen.perftest

internal object JNINative {
    init { io.prebindgen.perftest.NativeLibrary.ensureLoaded() }
    @JvmStatic external fun drop_Storage_9651261943550368(id: Long)
    @JvmStatic external fun call_storage_new_a15b46920214a065(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_storage_get_b7f3d4a82ad753f5(p0_w1: Long, returns: Return_Int_Long_Int5_ErrorEmpty_1e88956ed04b15f0): Unit
    @JvmStatic external fun call_payload_new_4db541865be00283(p0_w1: Long, p1_w1: Int, p2_w1: Double, p3_w1: Boolean, returns: Return_Long_Int_Double4_ErrorEmpty_f89d68e81a93e976): Unit
    @JvmStatic external fun call_storage_with_payload_b44f150bf309e573(p0_w2: Long, p0_w4: Int, p0_w6: Double, p0_w8: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_storage_put_opt_38084ee844ee97a7(p0_w1: Long, p1_w1: Int, p1_w4: Long, p1_w6: Int, p1_w8: Double, p1_w10: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
}
