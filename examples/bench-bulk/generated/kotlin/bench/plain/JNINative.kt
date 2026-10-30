package bench.plain

internal object JNINative {
    init { System.loadLibrary("bulk_bench") }
    @JvmStatic external fun drop_Handler_bb814c4111561dda(id: Long)
    @JvmStatic external fun drop_RowReturnHandler_0972ff7cb1556afa(id: Long)
    @JvmStatic external fun drop_WideRowReturnHandler_ed750f04931eb9ca(id: Long)
    @JvmStatic external fun call_row_make_bcd43e7b136151dd(p0_w1: Long, returns: Return_Long_Long_Long8_ErrorEmpty_f77363a35e75c509): Unit
    @JvmStatic external fun call_wide_row_make_1dbd8ca2fd4616ac(p0_w1: Long, returns: Return_Long_Long_Long32_ErrorEmpty_6fba61a9625e2221): Unit
    @JvmStatic external fun call_wide_row_sum_861bdfef124a25ad(p0_w1_f0: Long, p0_w1_f1: Long, p0_w1_f2: Long, p0_w1_f3: Long, p0_w1_f4: Long, p0_w1_f5: Long, p0_w1_f6: Long, p0_w1_f7: Long, p0_w1_f8: Long, p0_w1_f9: Long, p0_w1_f10: Long, p0_w1_f11: Long, p0_w1_f12: Long, p0_w1_f13: Long, p0_w1_f14: Long, p0_w1_f15: Long, p0_w1_f16: Long, p0_w1_f17: Long, p0_w1_f18: Long, p0_w1_f19: Long, p0_w1_f20: Long, p0_w1_f21: Long, p0_w1_f22: Long, p0_w1_f23: Long, p0_w1_f24: Long, p0_w1_f25: Long, p0_w1_f26: Long, p0_w1_f27: Long, p0_w1_f28: Long, p0_w1_f29: Long, p0_w1_f30: Long, p0_w1_f31: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_row_return_handler_new_9a8fcadcab7b9640(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_wide_row_return_handler_new_67f38d7168020371(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_row_return_fire_7c8a1183dcdd277c(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_wide_row_return_fire_88ad5bbf9c8c73e2(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_scalar_a4a5da3f79ebbcaf(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_row_sum_db65517187cece4c(p0_w1_f0: Long, p0_w1_f1: Long, p0_w1_f2: Long, p0_w1_f3: Long, p0_w1_f4: Long, p0_w1_f5: Long, p0_w1_f6: Long, p0_w1_f7: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_numbers_sum_47543406a5cdf588(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_rows_sum_d4a5ecfa43bf26b9(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_rows_make_6f7eb2a3ee93e0ae(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_rows_echo_5c26c8cf4751c454(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_packet_sum_c07ede9eb5de050f(p0_w1_f0_f0: Long, p0_w1_f0_f1: Long, p0_w1_f0_f2: Long, p0_w1_f0_f3: Long, p0_w1_f0_f4: Long, p0_w1_f0_f5: Long, p0_w1_f0_f6: Long, p0_w1_f0_f7: Long, p0_w1_f1: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_handler_new_916a49ec8343b596(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_fire_6fdab6ebdaeffb11(p0_w1: Long, p1_w1: Long, p2_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    internal fun interface SequenceSink_rows_for_reduce_fold_6202aebdeb4d4c0b { fun run(v0: Long, v1: Long, v2: Long, v3: Long, v4: Long, v5: Long, v6: Long, v7: Long) }
    @JvmStatic external fun call_rows_for_reduce_fold_587a8aef1d9a0485(p0_w1: Long, p1_w1: Long, callback: SequenceSink_rows_for_reduce_fold_6202aebdeb4d4c0b, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
}
