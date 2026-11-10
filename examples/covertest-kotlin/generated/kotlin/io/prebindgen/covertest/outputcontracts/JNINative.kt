package io.prebindgen.covertest.outputcontracts

internal object JNINative {
    init { io.prebindgen.covertest.NativeLibrary.ensureLoaded() }
    @JvmStatic external fun drop_DurationPair_a80e8ee6ea4081d2(id: Long)
    @JvmStatic external fun bulkLayouts(): LongArray
    @JvmStatic external fun call_duration_echo_9eff5d3e7eedf8fc(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_duration_packet_f309e8fdef6a1a5c(p0_w1: Long): Array<Any?>
    @JvmStatic external fun call_duration_pair_7f1ff9c108dc7c4e(returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_duration_series_0147306c7d8df97a(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_duration_optional_cfe965c40fd3f602(p0_w1: Boolean, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_duration_result_1a1b3c1552bf21b6(p0_w1: Long, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Long
    @JvmStatic external fun call_plain_result_3f31a43f28b72fff(p0_w1: Long, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Long
    @JvmStatic external fun call_duration_callback_90cbff1e8af6c9e8(custom_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_plain_callback_1c7d8c18a52f8e01(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    internal interface SequenceCapacity { fun reserve(capacity: Int) }
    @JvmStatic private external fun init_SequenceCapacity_e067e94ae7b50e2a(capacity: Class<*>)
    init { init_SequenceCapacity_e067e94ae7b50e2a(SequenceCapacity::class.java) }
    internal class SequenceRow_Long_1_127a3c24b600dbe0(@JvmField val c0: Long)
    internal interface SequenceSink_duration_series_fold_c394f649e0b50f49 : SequenceCapacity { fun run(v0: Long) }
    internal class SequenceCollector_duration_series_fold_e15a6be57122d068 : SequenceSink_duration_series_fold_c394f649e0b50f49 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_1_127a3c24b600dbe0>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long) { rows.add(SequenceRow_Long_1_127a3c24b600dbe0(v0)) }
    }
    @JvmStatic external fun call_duration_series_fold_c232f7a7643f8f2b(callback: SequenceSink_duration_series_fold_c394f649e0b50f49, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
}
