package io.prebindgen.covertest.rawinputs

internal object JNINative {
    init { io.prebindgen.covertest.NativeLibrary.ensureLoaded() }
    @JvmStatic external fun drop_BulkHandle_393b82dfa5eaa2d5(id: Long)
    @JvmStatic external fun bulkLayouts(): LongArray
    @JvmStatic external fun call_bulk_replay_rows_647e979b1d69f1d8(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_handle_new_e5a9aa77fa861745(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_handle_value_020d7d074b3973cd(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_replay_nested_rows_888a142b3d0bc092(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_replay_nested_callback_e01ab5cc8380c27a(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_nested_handles_input_20a8692eaca21fba(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_cleanup_nested_callback_1167603045ff0977(p0_callback: Any?, p1_w1: Boolean, p2_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_cleanup_pair_callback_75c0d45b83cad591(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
}
