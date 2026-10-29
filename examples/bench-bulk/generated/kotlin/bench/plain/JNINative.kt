package bench.plain

internal object JNINative {
    init { System.loadLibrary("bulk_bench") }
    @JvmStatic external fun drop_Handler_bb814c4111561dda(id: Long)
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
