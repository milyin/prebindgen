package bench.plain

internal object JNINative {
    init { System.loadLibrary("bulk_bench") }
    @JvmStatic external fun drop_Handler_bb814c4111561dda(id: Long)
    @JvmStatic external fun drop_InputHandle_7cc67535bbd4a8ee(id: Long)
    @JvmStatic external fun drop_RowReturnHandler_0972ff7cb1556afa(id: Long)
    @JvmStatic external fun drop_WideRowReturnHandler_ed750f04931eb9ca(id: Long)
    internal class SequenceRow_Array_Any___1_00f39593b4ea53ea(@JvmField val c0: Array<Any?>?)
    internal class SequenceRow_Long_1_127a3c24b600dbe0(@JvmField val c0: Long)
    internal class SequenceRow_Long_Long_2_c3e8677e51c52ca2(@JvmField val c0: Long, @JvmField val c1: Long)
    internal class SequenceRow_Long_Long_Long_8_3346a47ef3bb7c4a(@JvmField val c0: Long, @JvmField val c1: Long, @JvmField val c2: Long, @JvmField val c3: Long, @JvmField val c4: Long, @JvmField val c5: Long, @JvmField val c6: Long, @JvmField val c7: Long)
    internal class SequenceRow_Long_String_ByteArray_3_e5ce0d0d57155c04(@JvmField val c0: Long, @JvmField val c1: String?, @JvmField val c2: ByteArray?)
    @JvmStatic private external fun init_OutputRows_776ecb9d0c3754e8(classes: Array<Class<*>>)
    init { init_OutputRows_776ecb9d0c3754e8(arrayOf(SequenceRow_Array_Any___1_00f39593b4ea53ea::class.java, SequenceRow_Long_1_127a3c24b600dbe0::class.java, SequenceRow_Long_Long_2_c3e8677e51c52ca2::class.java, SequenceRow_Long_Long_Long_8_3346a47ef3bb7c4a::class.java, SequenceRow_Long_String_ByteArray_3_e5ce0d0d57155c04::class.java)) }
    @JvmStatic external fun call_input_handle_new_d7aa473c3cabcfa5(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_input_handle_value_ae1ea4e2f311cdb7(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_buffer_sizes_09330eed51da0791(p0_w1: String?, p1_w1: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_selected_handle_5e0482e736d8e05c(p0_w1: Int, p0_w3: Long, p0_w6: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_handle_values_sum_0c275022668af098(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_handle_records_sum_4d3c2822396924df(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_owned_handles_echo_e97ab382b6bce5df(p0_w1: Array<Any?>?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_owned_handle_records_echo_7223d199891bf713(p0_w1: Array<Any?>?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_callback_owned_records_59a5e85070935754(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_callback_owned_record_c829b73c0af8c75b(p0_callback: Any?, returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_owned_buffer_records_echo_32801b2928e6b02f(p0_w1: Array<Any?>?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_nested_owned_handles_echo_656072b3182d8e7b(p0_w1: Array<Any?>?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_callback_nested_owned_handles_45191dd02ebf0e07(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_row_make_bcd43e7b136151dd(p0_w1: Long, returns: Return_Long_Long_Long8_ErrorEmpty_f77363a35e75c509): Unit
    @JvmStatic external fun call_wide_row_make_1dbd8ca2fd4616ac(p0_w1: Long, returns: Return_Long_Long_Long32_ErrorEmpty_6fba61a9625e2221): Unit
    @JvmStatic external fun call_wide_row_sum_861bdfef124a25ad(p0_w1_f0: Long, p0_w1_f1: Long, p0_w1_f2: Long, p0_w1_f3: Long, p0_w1_f4: Long, p0_w1_f5: Long, p0_w1_f6: Long, p0_w1_f7: Long, p0_w1_f8: Long, p0_w1_f9: Long, p0_w1_f10: Long, p0_w1_f11: Long, p0_w1_f12: Long, p0_w1_f13: Long, p0_w1_f14: Long, p0_w1_f15: Long, p0_w1_f16: Long, p0_w1_f17: Long, p0_w1_f18: Long, p0_w1_f19: Long, p0_w1_f20: Long, p0_w1_f21: Long, p0_w1_f22: Long, p0_w1_f23: Long, p0_w1_f24: Long, p0_w1_f25: Long, p0_w1_f26: Long, p0_w1_f27: Long, p0_w1_f28: Long, p0_w1_f29: Long, p0_w1_f30: Long, p0_w1_f31: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_row_return_handler_new_9a8fcadcab7b9640(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_wide_row_return_handler_new_67f38d7168020371(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_row_return_fire_434e8dbfa624de44(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_wide_row_return_fire_ea0468a8c78895fe(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_control_b0073ee32096caea(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_scalar_a4a5da3f79ebbcaf(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_row_sum_db65517187cece4c(p0_w1_f0: Long, p0_w1_f1: Long, p0_w1_f2: Long, p0_w1_f3: Long, p0_w1_f4: Long, p0_w1_f5: Long, p0_w1_f6: Long, p0_w1_f7: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_numbers_sum_47543406a5cdf588(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_rows_sum_d4a5ecfa43bf26b9(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_rows_make_6f7eb2a3ee93e0ae(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_rows_echo_5c26c8cf4751c454(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_packet_sum_c07ede9eb5de050f(p0_w1_f0_f0: Long, p0_w1_f0_f1: Long, p0_w1_f0_f2: Long, p0_w1_f0_f3: Long, p0_w1_f0_f4: Long, p0_w1_f0_f5: Long, p0_w1_f0_f6: Long, p0_w1_f0_f7: Long, p0_w1_f1: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_handler_new_916a49ec8343b596(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_fire_ff69607332471b89(p0_w1: Long, p1_w1: Long, p2_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    internal interface SequenceCapacity { fun reserve(capacity: Int) }
    @JvmStatic private external fun init_SequenceCapacity_e067e94ae7b50e2a(capacity: Class<*>)
    init { init_SequenceCapacity_e067e94ae7b50e2a(SequenceCapacity::class.java) }
    internal interface SequenceSink_owned_handles_echo_fold_3b50da75bf6f9626 : SequenceCapacity { fun run(v0: Long) }
    internal class SequenceCollector_owned_handles_echo_fold_8c0e0fda84512cef : SequenceSink_owned_handles_echo_fold_3b50da75bf6f9626 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_1_127a3c24b600dbe0>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long) { rows.add(SequenceRow_Long_1_127a3c24b600dbe0(v0)) }
    }
    @JvmStatic external fun call_owned_handles_echo_fold_de46f033d154a724(p0_w1: Array<Any?>?, ownership: LongArray, callback: SequenceSink_owned_handles_echo_fold_3b50da75bf6f9626, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_nested_owned_handles_echo_fold_5978b5e7b216ac8a : SequenceCapacity { fun run(v0: Array<Any?>?) }
    internal class SequenceCollector_nested_owned_handles_echo_fold_9847d4ca29b981db : SequenceSink_nested_owned_handles_echo_fold_5978b5e7b216ac8a {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Array_Any___1_00f39593b4ea53ea>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Array<Any?>?) { rows.add(SequenceRow_Array_Any___1_00f39593b4ea53ea(v0)) }
    }
    @JvmStatic external fun call_nested_owned_handles_echo_fold_f1a4a186e9c98620(p0_w1: Array<Any?>?, ownership: LongArray, callback: SequenceSink_nested_owned_handles_echo_fold_5978b5e7b216ac8a, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_callback_nested_owned_handles_fold_cd6e4ebeaa22525e : SequenceCapacity { fun run(v0: Array<Any?>?) }
    internal class SequenceCollector_callback_nested_owned_handles_fold_e69accf7e0c74c5f : SequenceSink_callback_nested_owned_handles_fold_cd6e4ebeaa22525e {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Array_Any___1_00f39593b4ea53ea>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Array<Any?>?) { rows.add(SequenceRow_Array_Any___1_00f39593b4ea53ea(v0)) }
    }
    @JvmStatic external fun call_callback_nested_owned_handles_fold_496cd4317fea585c(p0_callback: Any?, callback: SequenceSink_callback_nested_owned_handles_fold_cd6e4ebeaa22525e, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_rows_for_reduce_fold_6202aebdeb4d4c0b : SequenceCapacity { fun run(v0: Long, v1: Long, v2: Long, v3: Long, v4: Long, v5: Long, v6: Long, v7: Long) }
    internal class SequenceCollector_rows_for_reduce_fold_bac33805078e0166 : SequenceSink_rows_for_reduce_fold_6202aebdeb4d4c0b {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_Long_Long_8_3346a47ef3bb7c4a>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long, v1: Long, v2: Long, v3: Long, v4: Long, v5: Long, v6: Long, v7: Long) { rows.add(SequenceRow_Long_Long_Long_8_3346a47ef3bb7c4a(v0, v1, v2, v3, v4, v5, v6, v7)) }
    }
    @JvmStatic external fun call_rows_for_reduce_fold_587a8aef1d9a0485(p0_w1: Long, p1_w1: Long, callback: SequenceSink_rows_for_reduce_fold_6202aebdeb4d4c0b, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
}
