package bench.bulk

internal object JNINative {
    init { System.loadLibrary("bulk_bench") }
    @JvmStatic external fun drop_Handler_bb814c4111561dda(id: Long)
    @JvmStatic external fun bulkLayouts(): LongArray
    @JvmStatic external fun call_scalar_a4a5da3f79ebbcaf(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_row_sum_db65517187cece4c(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_numbers_sum_47543406a5cdf588(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_rows_sum_d4a5ecfa43bf26b9(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_rows_make_6f7eb2a3ee93e0ae(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_rows_for_reduce_3c2e82abbc965f1c(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_rows_echo_5c26c8cf4751c454(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_packet_sum_c07ede9eb5de050f(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_handler_new_916a49ec8343b596(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_fire_6fdab6ebdaeffb11(p0_w1: Long, p1_w1: Long, p2_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
}
