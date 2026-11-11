package bench.bulk

internal object JNINative {
    init { System.loadLibrary("bulk_bench") }
    @JvmStatic external fun drop_Handler_bb814c4111561dda(id: Long)
    @JvmStatic external fun drop_InputHandle_7cc67535bbd4a8ee(id: Long)
    @JvmStatic external fun drop_RowReturnHandler_0972ff7cb1556afa(id: Long)
    @JvmStatic external fun drop_WideRowReturnHandler_ed750f04931eb9ca(id: Long)
    @JvmStatic external fun bulkLayouts(): LongArray
    @JvmStatic external fun call_canonical_value_6b06907d41ff5474(p0_w3: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_expanded_value_bb1afc8a668b541f(p0_w2: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_canonical_values_4a18f6c897a6e47d(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_expanded_values_69fd56d541c804cc(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_canonical_value_callback_32a58c45934fa8ba(p0_callback: Any?, p1_w3: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_expanded_value_callback_785f73f6925ad475(p0_callback: Any?, p1_w2: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_batches_owned_handles_33b783abb5780e95(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_callback_mixed_batches_owned_handles_2c0ba11b9a082cc3(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_input_handle_new_d7aa473c3cabcfa5(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_input_handle_value_ae1ea4e2f311cdb7(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_buffer_sizes_09330eed51da0791(p0_w1: String?, p1_w1: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_selected_handle_5e0482e736d8e05c(p0_w1: Int, p0_w3: Long, p0_w6: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_handle_values_sum_0c275022668af098(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_handle_records_sum_4d3c2822396924df(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_owned_handles_echo_e97ab382b6bce5df(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_owned_handle_records_echo_7223d199891bf713(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_callback_owned_records_59a5e85070935754(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_callback_owned_record_c829b73c0af8c75b(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_owned_buffer_records_echo_32801b2928e6b02f(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_nested_owned_handles_echo_656072b3182d8e7b(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_callback_nested_owned_handles_45191dd02ebf0e07(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_row_make_bcd43e7b136151dd(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_wide_row_make_1dbd8ca2fd4616ac(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_wide_row_sum_861bdfef124a25ad(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_row_return_handler_new_9a8fcadcab7b9640(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_wide_row_return_handler_new_67f38d7168020371(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_row_return_fire_434e8dbfa624de44(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_wide_row_return_fire_ea0468a8c78895fe(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_control_b0073ee32096caea(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_scalar_a4a5da3f79ebbcaf(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_row_sum_db65517187cece4c(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_numbers_sum_47543406a5cdf588(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_rows_sum_d4a5ecfa43bf26b9(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_rows_make_6f7eb2a3ee93e0ae(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_rows_for_reduce_3c2e82abbc965f1c(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_rows_echo_5c26c8cf4751c454(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_packet_sum_c07ede9eb5de050f(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_handler_new_916a49ec8343b596(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_fire_ff69607332471b89(p0_w1: Long, p1_w1: Long, p2_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
}
