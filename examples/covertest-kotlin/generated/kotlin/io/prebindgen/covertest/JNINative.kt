package io.prebindgen.covertest

internal object JNINative {
    init { io.prebindgen.covertest.NativeLibrary.ensureLoaded() }
    @JvmStatic external fun drop_Archive_0aa4417b9f320a03(id: Long)
    @JvmStatic external fun drop_BorrowNoClone_583d0749922ca782(id: Long)
    @JvmStatic external fun drop_BorrowPacket_969214a55d92aefe(id: Long)
    @JvmStatic external fun drop_BulkHandle_393b82dfa5eaa2d5(id: Long)
    @JvmStatic external fun drop_CallbackBoundaryHandler_0e611baf5b66c32b(id: Long)
    @JvmStatic external fun drop_CallbackConfigHandler_67dc52e4c6670317(id: Long)
    @JvmStatic external fun drop_CallbackConstructor_65ec2182d775ceb8(id: Long)
    @JvmStatic external fun drop_CallbackConstructorEnvelope_fa0e9faef9fa1f09(id: Long)
    @JvmStatic external fun drop_CallbackConstructorJob_c88b6ee076189420(id: Long)
    @JvmStatic external fun drop_CallbackJob_7f5ec1b2a514af2f(id: Long)
    @JvmStatic external fun drop_CallbackOwned_d936a95539d532a5(id: Long)
    @JvmStatic external fun drop_CallbackProjected_74e71cedcb164780(id: Long)
    @JvmStatic external fun drop_CallbackReentry_6c6c0fff370f6efb(id: Long)
    @JvmStatic external fun drop_CallbackReturnHandler_8a6b761b8bbde2f7(id: Long)
    @JvmStatic external fun drop_CallbackReturnJob_c59afe63a0d1957d(id: Long)
    @JvmStatic external fun drop_CallbackReturnResource_289c5b28e5450a79(id: Long)
    @JvmStatic external fun drop_CallbackRowResource_abd70bbf025eb35c(id: Long)
    @JvmStatic external fun drop_CallbackSet_42869ab2828f84d4(id: Long)
    @JvmStatic external fun drop_CallbackTransferred_eab9ef9641575576(id: Long)
    @JvmStatic external fun drop_CallbackUniqueResource_d31d0c3f2ae0419c(id: Long)
    @JvmStatic external fun drop_GcResource_0ee976436a63104d(id: Long)
    @JvmStatic external fun drop_GetterLeaf_b28f97ae79569aa8(id: Long)
    @JvmStatic external fun drop_GetterOwner_bc1803eb5b8c76ef(id: Long)
    @JvmStatic external fun drop_NicheChoiceHandle_820b60fc3ef75039(id: Long)
    @JvmStatic external fun drop_NullabilityHandle_aefadfc6a0601b82(id: Long)
    @JvmStatic external fun drop_OptionChoice_feab979b8bef4265(id: Long)
    @JvmStatic external fun drop_OptionCtor_9701b4353b1ab7b0(id: Long)
    @JvmStatic external fun drop_OwnedResource_e39aa0af54304c88(id: Long)
    @JvmStatic external fun drop_PayloadHandle_af81cbfac6e10b9d(id: Long)
    @JvmStatic external fun drop_PlainResource_b08b40798e6273bd(id: Long)
    @JvmStatic external fun drop_ProjectionHandle_46b41135284839d8(id: Long)
    @JvmStatic external fun drop_Report_965f7809bbde41dd(id: Long)
    @JvmStatic external fun drop_ReportStruct_2659d7dec79ecce0(id: Long)
    @JvmStatic external fun drop_Storage_9651261943550368(id: Long)
    @JvmStatic external fun drop_Summary_caa4a50522c6b815(id: Long)
    @JvmStatic external fun drop_TypeInputA_ac61bf30bfd3c448(id: Long)
    @JvmStatic external fun drop_TypeInputB_ac61c230bfd3c961(id: Long)
    @JvmStatic external fun drop_TypeInputControl_7c64d7b5440ff06a(id: Long)
    internal class SequenceRow_Array_Any___1_00f39593b4ea53ea(@JvmField val c0: Array<Any?>?)
    internal class SequenceRow_Long_String_Long_3_b4558cb7b740d7c6(@JvmField val c0: Long, @JvmField val c1: String?, @JvmField val c2: Long)
    internal class SequenceRow_Long_Long_Int_4_851daa809dd7a1b1(@JvmField val c0: Long, @JvmField val c1: Long, @JvmField val c2: Int, @JvmField val c3: Long)
    internal class SequenceRow_Int_Long_String_7_e1006826998dc4d4(@JvmField val c0: Int, @JvmField val c1: Long, @JvmField val c2: String?, @JvmField val c3: Long, @JvmField val c4: Int, @JvmField val c5: Long, @JvmField val c6: String?)
    internal class SequenceRow_Int_Long_String_4_3817dc835fa26831(@JvmField val c0: Int, @JvmField val c1: Long, @JvmField val c2: String?, @JvmField val c3: Long)
    internal class SequenceRow_Long_String_2_3ea2c8aa47ab7590(@JvmField val c0: Long, @JvmField val c1: String?)
    internal class SequenceRow_Long_1_127a3c24b600dbe0(@JvmField val c0: Long)
    internal class SequenceRow_Long_Double_2_012db11776ed72c5(@JvmField val c0: Long, @JvmField val c1: Double)
    internal class SequenceRow_Long_String_ByteArray_3_e5ce0d0d57155c04(@JvmField val c0: Long, @JvmField val c1: String?, @JvmField val c2: ByteArray?)
    internal class SequenceRow_ByteArray_1_92dc0f7ab995287c(@JvmField val c0: ByteArray?)
    internal class SequenceRow_Long_Long_Long_3_ad466713697fa9e4(@JvmField val c0: Long, @JvmField val c1: Long, @JvmField val c2: Long)
    internal class SequenceRow_Long_Long_Long_127_725c521ebd1c733a(@JvmField val c0: Long, @JvmField val c1: Long, @JvmField val c2: Long, @JvmField val c3: Long, @JvmField val c4: Long, @JvmField val c5: Long, @JvmField val c6: Long, @JvmField val c7: Long, @JvmField val c8: Long, @JvmField val c9: Long, @JvmField val c10: Long, @JvmField val c11: Long, @JvmField val c12: Long, @JvmField val c13: Long, @JvmField val c14: Long, @JvmField val c15: Long, @JvmField val c16: Long, @JvmField val c17: Long, @JvmField val c18: Long, @JvmField val c19: Long, @JvmField val c20: Long, @JvmField val c21: Long, @JvmField val c22: Long, @JvmField val c23: Long, @JvmField val c24: Long, @JvmField val c25: Long, @JvmField val c26: Long, @JvmField val c27: Long, @JvmField val c28: Long, @JvmField val c29: Long, @JvmField val c30: Long, @JvmField val c31: Long, @JvmField val c32: Long, @JvmField val c33: Long, @JvmField val c34: Long, @JvmField val c35: Long, @JvmField val c36: Long, @JvmField val c37: Long, @JvmField val c38: Long, @JvmField val c39: Long, @JvmField val c40: Long, @JvmField val c41: Long, @JvmField val c42: Long, @JvmField val c43: Long, @JvmField val c44: Long, @JvmField val c45: Long, @JvmField val c46: Long, @JvmField val c47: Long, @JvmField val c48: Long, @JvmField val c49: Long, @JvmField val c50: Long, @JvmField val c51: Long, @JvmField val c52: Long, @JvmField val c53: Long, @JvmField val c54: Long, @JvmField val c55: Long, @JvmField val c56: Long, @JvmField val c57: Long, @JvmField val c58: Long, @JvmField val c59: Long, @JvmField val c60: Long, @JvmField val c61: Long, @JvmField val c62: Long, @JvmField val c63: Long, @JvmField val c64: Long, @JvmField val c65: Long, @JvmField val c66: Long, @JvmField val c67: Long, @JvmField val c68: Long, @JvmField val c69: Long, @JvmField val c70: Long, @JvmField val c71: Long, @JvmField val c72: Long, @JvmField val c73: Long, @JvmField val c74: Long, @JvmField val c75: Long, @JvmField val c76: Long, @JvmField val c77: Long, @JvmField val c78: Long, @JvmField val c79: Long, @JvmField val c80: Long, @JvmField val c81: Long, @JvmField val c82: Long, @JvmField val c83: Long, @JvmField val c84: Long, @JvmField val c85: Long, @JvmField val c86: Long, @JvmField val c87: Long, @JvmField val c88: Long, @JvmField val c89: Long, @JvmField val c90: Long, @JvmField val c91: Long, @JvmField val c92: Long, @JvmField val c93: Long, @JvmField val c94: Long, @JvmField val c95: Long, @JvmField val c96: Long, @JvmField val c97: Long, @JvmField val c98: Long, @JvmField val c99: Long, @JvmField val c100: Long, @JvmField val c101: Long, @JvmField val c102: Long, @JvmField val c103: Long, @JvmField val c104: Long, @JvmField val c105: Long, @JvmField val c106: Long, @JvmField val c107: Long, @JvmField val c108: Long, @JvmField val c109: Long, @JvmField val c110: Long, @JvmField val c111: Long, @JvmField val c112: Long, @JvmField val c113: Long, @JvmField val c114: Long, @JvmField val c115: Long, @JvmField val c116: Long, @JvmField val c117: Long, @JvmField val c118: Long, @JvmField val c119: Long, @JvmField val c120: Long, @JvmField val c121: Long, @JvmField val c122: Long, @JvmField val c123: Long, @JvmField val c124: Long, @JvmField val c125: Long, @JvmField val c126: Long)
    @JvmStatic private external fun init_OutputRows_776ecb9d0c3754e8(classes: Array<Class<*>>)
    init { init_OutputRows_776ecb9d0c3754e8(arrayOf(SequenceRow_Array_Any___1_00f39593b4ea53ea::class.java, SequenceRow_ByteArray_1_92dc0f7ab995287c::class.java, SequenceRow_Int_Long_String_4_3817dc835fa26831::class.java, SequenceRow_Int_Long_String_7_e1006826998dc4d4::class.java, SequenceRow_Long_1_127a3c24b600dbe0::class.java, SequenceRow_Long_Double_2_012db11776ed72c5::class.java, SequenceRow_Long_Long_Int_4_851daa809dd7a1b1::class.java, SequenceRow_Long_Long_Long_127_725c521ebd1c733a::class.java, SequenceRow_Long_Long_Long_3_ad466713697fa9e4::class.java, SequenceRow_Long_String_2_3ea2c8aa47ab7590::class.java, SequenceRow_Long_String_ByteArray_3_e5ce0d0d57155c04::class.java, SequenceRow_Long_String_Long_3_b4558cb7b740d7c6::class.java)) }
    @JvmStatic external fun bulkLayouts(): LongArray
    @JvmStatic external fun call_type_input_a_new_a666cfeb0749bacc(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_a_read_752e87ac619cd2ab(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_b_new_0ac223fcf1a09c24(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_b_read_2a0fffe1bd334d9d(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_control_new_c01186712acd5471(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_live_03e8c255db4b9ee9(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_conversions_1b6cd416be082059(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_fixed_157e836568a582a4(p0_w1: Long, p1_w1: Long, p2_w1: Long, p3_w1: Long, p4_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_fixed_mut_1469d98e8752a26b(p0_w1: Long, p1_w1: Long, p2_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_shared_lists_82c951ffa3f2a084(p0_w1: ByteArray?, p0_w1_length: Int, p1_w1: ByteArray?, p1_w1_length: Int, p2_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_mut_lists_30242e7654d4250f(p0_w1: ByteArray?, p0_w1_length: Int, p1_w1: ByteArray?, p1_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_mixed_7b268c9e6948a110(p0_w1: ByteArray?, p0_w1_length: Int, p1_w1: ByteArray?, p1_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_row_d1ca0f73c8d133b6(p0_w1_f0: Long, p0_w1_f1: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_holder_4673e9a6dda65297(p0_w1_f0_present: Int, p0_w1_f0_some_f0: Long, p0_w1_f0_some_f1: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_rows_1943c070e2ef84cb(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_rows_plain_8d430d0aa0f4b840(p0_w1: Array<Any?>?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_nested_d53c1e8c8b8963c9(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_callback_e567d8ed47f4c01f(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_nested_callback_91c187ec446a933d(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_optional_6826ec5cb3fb4351(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_type_input_array_8fb1d0d4f19b9470(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_borrow_enum_rows_fbee7d589d39a15c(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_borrow_packet_new_87a8c3bbfa51949d(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_borrow_packet_len_6cbb722741c58243(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_borrow_packet_ref_770d4322f0c247a7(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_borrow_packet_optional_d64541d84155036d(p0_w1: Long, p1_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_borrow_packet_ref_expanded_92a58b0514f2fea9(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_borrow_packet_emit_366f113bb0a1bdca(p0_w1: Long, p1_callback: Any?, p2_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_borrow_packet_emit_copy_12e2378690dda8c6(p0_w1: Long, p1_callback: Any?, p2_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_borrow_packet_emit_optional_f12e8c1950468f84(p0_w1: Long, p1_callback: Any?, p2_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_borrow_packet_emit_expanded_e83c8cf8dcff206e(p0_w1: Long, p1_callback: Any?, p2_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_borrow_packet_emit_worker_dca4822cf131fe16(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_borrow_packet_counter_4b9e951a734376ce(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_borrow_no_clone_new_62088c19c27300c5(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_borrow_no_clone_value_d9e7b7aca40dea2e(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_borrow_no_clone_emit_452f216be3572703(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_collection_input_plain_49edb34e988dff39(p0_w1: Array<Any?>?, p1_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_input_bulk_7564a92b14e3a524(p0_w1: ByteArray?, p0_w1_length: Int, p1_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_input_custom_3f5932b5fe2bdeb0(p0_w1: Array<Any?>?, p1_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_input_custom_bulk_80b5719b93fcace3(p0_w1: ByteArray?, p0_w1_length: Int, p1_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_input_narrow_a2e0942031835410(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_input_narrow_bulk_01859c8a0312491b(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_input_constructions_07412dbe4e08ef68(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_input_slice_009ff8f8b9e77cd5(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_input_array_517465a977f1d8a5(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_input_optional_array_c11bc48fb5ea6c63(p0_w3: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_input_optional_array_bulk_c810a58a5f8b13ca(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_input_optional_9f143cf5440a4074(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_default_bulk_echo_19271b2932c312bb(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_array_f969bde7cc1c432a(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_optional_ea20da6fc813acee(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_nested_799f9a1fa2a2a9a1(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_borrowed_7528ee78c961424e(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_numbers_677f84d2a60bc3b3(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_units_12bee997ef805cee(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_texts_db6a5274075ed7aa(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_optional_texts_53f1ccde499e0f96(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_buffers_c8800191206cf047(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_plain_e53a41329b22bbda(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_default_bulk_callback_af0d489a7a6a5660(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_default_bulk_callback_mixed_d29393bf89d94532(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_bulk_replay_rows_aafaa0d2a20132d8(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_replay_redundant_rows_53c9ef64704ee61d(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_replay_nested_rows_fd3636f6cc750f4e(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_replay_nested_callback_43b7f2dcc306c446(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_replay_object_rows_be4c714a838acb22(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_replay_callback_929e98d2cfd600ef(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_nested_handles_input_dba24072c6255486(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_cleanup_nested_callback_5f8da21ad356132b(p0_callback: Any?, p1_w1: Boolean, p2_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_cleanup_pair_callback_f2f2462e96f2da09(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_replay_reset_ec26adbfb8cc43d7(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_bulk_replay_count_0e31779505af7ccf(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_patch_rows_d32fa8a44880666b(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_patch_callback_22fbb64046f78357(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_nested_envelope_fac9f0f836298b23(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_pair_text_callback_45fa8b29f215e390(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_handle_texts_415eab299e560837(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_collection_lock_groups_7fe8555600f1dcf6(p0_w3: Array<Any?>?, p1_w1: Array<Any?>?, p2_callback: Any?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
    @JvmStatic external fun call_bulk_handle_nested_ee0f317e439c05be(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_handle_text_callback_bf41b6097b186476(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_optional_borrow_pair_11a84638f9db14d4(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_select_b3dae360ea0eecfb(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_kinds_9b033197b22af4fe(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_units_1560a205c63d4c8a(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_flags_fe0ac76234b14846(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_envelope_c78cdfdfd18c33f1(p0_w1: Int, p0_w3_f0_f0: Long, p0_w3_f0_f1: Double, p0_w3_f1: Long, returns: Return_Int_Long_Double4_ErrorEmpty_4244d173b136f580): Unit
    @JvmStatic external fun call_bulk_primitives_be9bffc80840ca87(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_optional_wide_0dbde2915189c95e(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_optional_callback_3ffeb9ae490fb489(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_native_939b6cd52b96a019(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_expanded_65b04628441e244b(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_expanded_rows_8611c0e3cec0b75a(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_result_22b6c65613a56b35(p0_w1: ByteArray?, p0_w1_length: Int, p1_w1: Boolean, returns: Return_Empty_ErrorByteArray1_056b654d5078f47e): ByteArray?
    @JvmStatic external fun call_bulk_handle_new_e5a9aa77fa861745(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_handle_value_4d14e8e3b89bd1bd(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_drops_0c93c5a0b1492a12(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_late_051f1ea6dcd2506f(p0_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_handle_views_c1f9da5de20e3fc3(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_handles_16b087b5f7f79484(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_borrow_034d937ec5b796d9(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_checked_b7e284c31e7e5a38(p0_w1: ByteArray?, p0_w1_length: Int, p1_w1: ByteArray?, p1_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_wide_6b955d7b4ee7dc0d(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_unsigned_wide_407b0f069bb7ed7a(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_callback_worker_4498defd15d5c1bf(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_callback_63ae4882756ac2cf(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_handle_callback_39d3540632be44a7(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_point_e770ca9da9c12852(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_points_6101af74290565c5(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_point_array_aa3dcf2590ca3a28(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_slice_c30fbd330fdb06e6(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_point_plain_53e7bd65e4464218(p0_w1_f0: Long, p0_w1_f1: Double, returns: Return_Long_Double2_ErrorEmpty_a4ec6c8f44b5ff7d): Unit
    @JvmStatic external fun call_bulk_optional_handles_44d1ca20cf3dbdbe(p0_w1: ByteArray?, p0_w1_length: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_array_b64f34d955cfbc66(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_array_borrow_08a33cfde93f0897(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_scalar_859e15b07fe6305f(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_numbers_a3f5c3da005fa8ec(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_nested_0e44e904f1fd8209(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_text_af334975f2297859(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_strings_106737903dcecf60(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_optional_4abfbe9a71e24339(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_bytes_711128b36c976142(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_return_limited_54299b9182f8d252(p0_w1_f0_f0: Long, p0_w1_f0_f1: Long, p0_w1_f0_f2: Long, p0_w1_f0_f3: Long, p0_w1_f0_f4: Long, p0_w1_f0_f5: Long, p0_w1_f0_f6: Long, p0_w1_f0_f7: Long, p0_w1_f0_f8: Long, p0_w1_f0_f9: Long, p0_w1_f0_f10: Long, p0_w1_f0_f11: Long, p0_w1_f0_f12: Long, p0_w1_f0_f13: Long, p0_w1_f0_f14: Long, p0_w1_f0_f15: Long, p0_w1_f0_f16: Long, p0_w1_f0_f17: Long, p0_w1_f0_f18: Long, p0_w1_f0_f19: Long, p0_w1_f0_f20: Long, p0_w1_f0_f21: Long, p0_w1_f0_f22: Long, p0_w1_f0_f23: Long, p0_w1_f0_f24: Long, p0_w1_f0_f25: Long, p0_w1_f0_f26: Long, p0_w1_f0_f27: Long, p0_w1_f0_f28: Long, p0_w1_f0_f29: Long, p0_w1_f0_f30: Long, p0_w1_f0_f31: Long, p0_w1_f0_f32: Long, p0_w1_f0_f33: Long, p0_w1_f0_f34: Long, p0_w1_f0_f35: Long, p0_w1_f0_f36: Long, p0_w1_f0_f37: Long, p0_w1_f0_f38: Long, p0_w1_f0_f39: Long, p0_w1_f0_f40: Long, p0_w1_f0_f41: Long, p0_w1_f0_f42: Long, p0_w1_f0_f43: Long, p0_w1_f0_f44: Long, p0_w1_f0_f45: Long, p0_w1_f0_f46: Long, p0_w1_f0_f47: Long, p0_w1_f0_f48: Long, p0_w1_f0_f49: Long, p0_w1_f0_f50: Long, p0_w1_f0_f51: Long, p0_w1_f0_f52: Long, p0_w1_f0_f53: Long, p0_w1_f0_f54: Long, p0_w1_f0_f55: Long, p0_w1_f0_f56: Long, p0_w1_f0_f57: Long, p0_w1_f0_f58: Long, p0_w1_f0_f59: Long, p0_w1_f0_f60: Long, p0_w1_f0_f61: Long, p0_w1_f0_f62: Long, p0_w1_f1_f0: Long, p0_w1_f1_f1: Long, p0_w1_f1_f2: Long, p0_w1_f1_f3: Long, p0_w1_f1_f4: Long, p0_w1_f1_f5: Long, p0_w1_f1_f6: Long, p0_w1_f1_f7: Long, p0_w1_f1_f8: Long, p0_w1_f1_f9: Long, p0_w1_f1_f10: Long, p0_w1_f1_f11: Long, p0_w1_f1_f12: Long, p0_w1_f1_f13: Long, p0_w1_f1_f14: Long, p0_w1_f1_f15: Long, p0_w1_f1_f16: Long, p0_w1_f1_f17: Long, p0_w1_f1_f18: Long, p0_w1_f1_f19: Long, p0_w1_f1_f20: Long, p0_w1_f1_f21: Long, p0_w1_f1_f22: Long, p0_w1_f1_f23: Long, p0_w1_f1_f24: Long, p0_w1_f1_f25: Long, p0_w1_f1_f26: Long, p0_w1_f1_f27: Long, p0_w1_f1_f28: Long, p0_w1_f1_f29: Long, p0_w1_f1_f30: Long, p0_w1_f1_f31: Long, p0_w1_f1_f32: Long, p0_w1_f1_f33: Long, p0_w1_f1_f34: Long, p0_w1_f1_f35: Long, p0_w1_f1_f36: Long, p0_w1_f1_f37: Long, p0_w1_f1_f38: Long, p0_w1_f1_f39: Long, p0_w1_f1_f40: Long, p0_w1_f1_f41: Long, p0_w1_f1_f42: Long, p0_w1_f1_f43: Long, p0_w1_f1_f44: Long, p0_w1_f1_f45: Long, p0_w1_f1_f46: Long, p0_w1_f1_f47: Long, p0_w1_f1_f48: Long, p0_w1_f1_f49: Long, p0_w1_f1_f50: Long, p0_w1_f1_f51: Long, p0_w1_f1_f52: Long, p0_w1_f1_f53: Long, p0_w1_f1_f54: Long, p0_w1_f1_f55: Long, p0_w1_f1_f56: Long, p0_w1_f1_f57: Long, p0_w1_f1_f58: Long, p0_w1_f1_f59: Long, p0_w1_f1_f60: Long, p0_w1_f1_f61: Long, p0_w1_f1_f62: Long, p0_w1_f1_f63: Long, p1_w1: Boolean): Array<Any?>
    @JvmStatic external fun call_return_wide_error_6801c788d55410a5(returns: Return_Empty_ErrorLong_Long_Long130_414eccc9f9c1f833): Unit
    @JvmStatic external fun call_return_ownership_e57618289bd585e0(p0_w1: Long, p1_w1: Boolean, p2_callback: Any?, returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_return_single_ownership_9b8a79f3a1788fe8(p0_w1: Long, p1_w1: Boolean, p2_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_return_domain_cb3675a75d892ba1(p0_w1: Long, p1_w1: Boolean, p2_callback: Any?, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Long
    @JvmStatic external fun call_return_pair_8bb51b9e0f2fc283(p0_w1: Long, returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_return_wide_af7145445fe5e699(p0_w1: Long, returns: Return_Long_Long_Long130_ErrorEmpty_d02793509212a1f3): Unit
    @JvmStatic external fun call_nullability_handle_new_52d93c3d58bc9ddd(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_handle_read_87c3a62f7392e4f0(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_handle_optional_0d97fddf5e04f9dc(p0_w3: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_text_371469abc18caa34(p0_w1: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_text_optional_7c4699f3d1e235f0(p0_w3: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_bytes_ed1ce1214d744c8d(p0_w1: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_bytes_optional_48809d71f31114f1(p0_w3: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_wide_9d858f42bb045bf8(p0_w1: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_nullability_wide_optional_675c57dd8593ffa4(p0_w1: Int, p0_w3: ByteArray?, returns: Return_Int_ByteArray2_ErrorEmpty_a2aad6159c8c4c14): Unit
    @JvmStatic external fun call_nullability_value_sum_9ffd3f4f4ef95c92(p0_w1: Int, p0_w3_f0: String?, p0_w3_f1: ByteArray?, p0_w3_f2: ByteArray?, p0_w3_f3: Int, p0_w3_f4: Long, p0_w6: String?, p0_w8: ByteArray?, p0_w10: ByteArray?, p0_w12: Int, p0_w14: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_value_bulk_cde4e2ae3684af79(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_value_optional_9875ca4fcdb053b1(p0_w4: String?, p0_w6: ByteArray?, p0_w8: ByteArray?, p0_w10: Int, p0_w12: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_values_b3abc03a5e5fdbda(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_values_optional_bd4a7ceb471c6c54(p0_w3: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_handles_optional_95fc2bf9cec835af(p0_w3: Array<Any?>?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nullability_text_callback_42d80e83e23401a2(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_nullability_optional_text_callback_426adf31f0aa74b4(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_nullability_callback_choose_6b3e84beed47a860(p0_w1: Int, p0_w3_f0: String?, p0_w3_f1: ByteArray?, p0_w3_f2: ByteArray?, p0_w3_f3: Int, p0_w3_f4: Long, p0_w6: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_parameter_sum_83f208ac1978fef1(p0_w2: Long, p0_w4: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_parameter_optional_98841e5b8a3bad26(p0_w1: Int, p0_w4: Long, p0_w6: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_parameter_bulk_sum_b42ccdd89f7b8352(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_parameter_sum_object_f81116dd2577a97f(p0_w1_f0: Long, p0_w1_f1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_parameter_variants_8d7d583d82d7fc46(p0_w1: Int, p0_w3_f0: Long, p0_w3_f1: Long, p0_w6: Long, p0_w8: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_parameter_collisions_0fcb430c5f288a73(p0_w1: Long, p1_w1: Long, p2_w1: Long, p3_w1: Long, p4_w1: Long, p5_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_parameter_escaped_names_3a6c23522a1c4542(p0_w1: Long, p1_w1: Long, p2_w1: Long, p3_w1: Long, p4_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_parameter_record_306b9dc9d6961224(p0_w1_f0: Long, p0_w1_f1: String?, returns: Return_Long_String2_ErrorEmpty_1b009765d5d6eed0): Unit
    @JvmStatic external fun call_parameter_records_3f3e92735953a16b(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_gc_resource_new_7ae6fd267af3001d(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_gc_resource_value_9ca9e696d72ab415(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_gc_resource_consume_9318049a3f0cf4e9(p0_w1: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_gc_resource_fail_6be93bc9cd0b8290(p0_w1: Long, ownership: LongArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Long
    @JvmStatic external fun call_gc_resource_pair_6efba23f0576e080(p0_w1: Long, p1_w1: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_gc_resource_drops_53900a369e1b88bb(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_gc_resource_three_a606daff102e146f(p0_w1: Long, p1_w1: Long, p2_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_gc_resource_consume_rows_75eea78d55b4e81f(p0_w1: Array<Any?>?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_gc_resource_read_rows_57e058d6915b4e28(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_gc_resource_emit_c3aa9044644f4b9c(p0_w1: Long, p1_w1: Long, p2_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_gc_resource_hold_d19978b99df91680(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_gc_resource_return_5d3f561d66e965c2(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_gc_resource_emit_owned_5c69b3edc59eb5a1(p0_w1: Long, p1_w1: Long, p2_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_gc_resource_rows_cdbbaa46996fd3a0(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_plain_resource_new_d97fe9314c02793e(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_plain_resource_value_98ccec4a9f9d6c37(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_buffer_views_optional_text_2e5df9089cef5211(p0_w1: Boolean, returns: Return_String_Long2_ErrorEmpty_b2c3b4b34db92d24): Unit
    @JvmStatic external fun call_buffer_views_optional_bytes_f3f56b8a8b1996c2(p0_w1: Boolean, returns: Return_ByteArray_Long2_ErrorEmpty_34705c584e211183): Unit
    @JvmStatic external fun call_buffer_views_text_d3011d2eb33189c1(returns: Return_String_Long2_ErrorEmpty_b2c3b4b34db92d24): Unit
    @JvmStatic external fun call_buffer_views_bytes_491539966ab74896(returns: Return_ByteArray_Long2_ErrorEmpty_34705c584e211183): Unit
    @JvmStatic external fun call_jni_buffer_resource_f33d027f936f6663(returns: Return_Long_String_ByteArray3_ErrorEmpty_b6abe218690243a7): Unit
    @JvmStatic external fun call_jni_buffer_domain_failure_f36020e4f4562fe0(returns: Return_Empty_ErrorLong_String_ByteArray3_2803e103835e4ca7): Unit
    @JvmStatic external fun call_jni_buffer_resource_rows_54fd0c38ecd1f227(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_jni_buffer_text_d33d1c8fd91af20d(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_jni_buffer_string_649b053bccecf6a7(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_jni_buffer_bytes_801aa24b9714561c(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_jni_buffer_vec_18f26d75535912ae(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_jni_buffer_optional_text_d47e68fa29b4bf83(p0_w1: Int, returns: Return_Int_String2_ErrorEmpty_7e68b805da700de5): Unit
    @JvmStatic external fun call_jni_buffer_optional_bytes_2e41382fb70d4d62(p0_w1: Int, returns: Return_Int_ByteArray2_ErrorEmpty_a2aad6159c8c4c14): Unit
    @JvmStatic external fun call_jni_buffer_owned_optional_3ea9b140c39283fe(p0_w1: Int, returns: Return_Int_String_ByteArray3_ErrorEmpty_e58bbdba0f222ca2): Unit
    @JvmStatic external fun call_jni_buffer_owned_1b20562b3c5bfef8(returns: Return_String_ByteArray2_ErrorEmpty_248068589bab9cfc): Unit
    @JvmStatic external fun call_jni_buffer_rows_0e85d0bcc7902cb5(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_jni_buffer_wide_6983ec538bbdc661(returns: Return_String_String_String48_ErrorEmpty_181c8abbd254d130): Unit
    @JvmStatic external fun call_jni_buffer_callback_a41319fcdf187075(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_jni_buffer_failure_6fdac6591b1a6726(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_jni_buffer_worker_8d2b0e9314030afd(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_jni_buffer_owned_callback_9fa8a662ae731b98(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_getter_owner_new_9b3691104cf4f094(p0_w1: Boolean, returns: Return_Long_Long_String10_ErrorEmpty_9bb213f8e50528d9): Unit
    @JvmStatic external fun call_getter_owner_failure_983fd2bf1ecc08ff(returns: Return_Long_Long_String3_ErrorEmpty_857937f364385c79): Unit
    @JvmStatic external fun call_getter_owner_id_a8c03c5a37bc12c7(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_getter_leaf_id_2fb1e3b5dd3d2d62(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_getter_static_leaf_3de084de8ab56cf6(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_getter_owner_read_6853655c2dc05353(p0_w1: Long, returns: Return_Long_String2_ErrorEmpty_1b009765d5d6eed0): Unit
    @JvmStatic external fun call_getter_counts_ac432be3e581775f(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_getter_two_clones_new_c6e10ed44149bf78(returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_getter_fields_new_6223254642400003(returns: Return_Long_Long_String3_ErrorEmpty_857937f364385c79): Unit
    @JvmStatic external fun call_getter_callback_261a8655bbbdf16f(p0_callback: Any?, p1_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_jni_output_optional_b3a3cb08bb632879(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_jni_output_mixed_23877cf213bf0c0c(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_c_output_rows_76dcbc8fbe8acf89(p0_w1: Long, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Array<Any?>?
    @JvmStatic external fun call_c_output_nested_09e14469d381a037(p0_w1: Long, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Array<Any?>?
    @JvmStatic external fun call_c_output_pair_654ab15b1845473b(p0_w1: Long, returns: Return_Array_Any___Array_Any__2_ErrorLong1_59a6678d4242c812): Unit
    @JvmStatic external fun call_callback_boundary_register_f9d2a0713d344329(p0_callback: Any?, p1_w1: Boolean, returns: Return_Empty_ErrorLong_String2_a3fb4f88f2014cb0): Long
    @JvmStatic external fun call_callback_boundary_fire_7050b5a0a6b6f271(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_boundary_fold_fdb4940d384f5a6d(p0_callback: Any?, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_callback_boundary_faults_b864f6da96359033(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_boundary_conversion_0ab973f530f23b0b(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_boundary_borrowed_2779cd6570f360a9(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_boundary_strings_79f049084aabc87a(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_config_default_7ba54fc6e91d1046(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_config_type_default_2e21d1170d142a9b(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_config_argument_f2ca22fac606e26f(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_config_return_1a2a025b2d23bf61(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_config_pair_075058877ec579cf(p0_callback: Any?, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_config_nested_c023613beb216159(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_config_optional_c1ccbb78d74cca03(p0_w1: Int, p0_w4: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_config_ordinary_7a1de11aa6329a97(p0_w2: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_nested_batch_echo_3dada1cd491c5035(p0_w1_f0: Long, p0_w1_f1: Long, p0_w2: Array<Any?>?, p0_w1_f3: Int, p0_w15: Array<Any?>?, p0_w1_f5: Long, ownership: LongArray, returns: Return_Long_Long_Array_Any__6_ErrorEmpty_5b53ecd9dac487e2): Unit
    @JvmStatic external fun call_callback_nested_results_b9be5ae57009af02(p0_callback: Any?, p1_w1: Boolean, p2_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_nested_batch_2c220f1124510108(p0_callback: Any?, p1_w1: Boolean, p2_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_fixed_result_7b9617992129d1cc(p0_callback: Any?, p1_w1: Boolean, p2_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_fixed_results_ee0852d3858ba775(p0_callback: Any?, p1_w1: Boolean, p2_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_fixed_optional_results_58d579f82e8266ff(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_fixed_borrowed_results_e2d63138e2b54abf(p0_callback: Any?, p1_w1: Boolean, p2_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_wire_rows_catch_001b9a48fe8c7967(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_wire_row_catch_05921b2c51a5708c(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_i64_c08776c9ce71e81d(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_i8_753f4b94dff729a6(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Byte
    @JvmStatic external fun call_callback_return_i16_9b06d1210ff8b2b7(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Short
    @JvmStatic external fun call_callback_return_f32_1d9d4691d73646c5(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Float
    @JvmStatic external fun call_callback_return_u128_54bd6e65c7018d0b(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_callback_return_zero_d965a5274955145f(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_bool_468d68aae8a03d87(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
    @JvmStatic external fun call_callback_return_char_ac2c7988ad8d876f(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_callback_return_float_e8e49d9c6e5ee55e(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Double
    @JvmStatic external fun call_callback_return_unsigned_fce893a55ce5733f(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_wide_3b195257768b36eb(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_callback_return_text_b4839b1b734afa0b(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_callback_return_bytes_8891244c4fbb186c(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_callback_return_optional_b061087be5b62673(p0_callback: Any?, p1_w1: Boolean, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_callback_return_vector_f78658284aa91f49(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_callback_return_record_b21187a8bb28b01d(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_enum_01ed3ed2bab330bf(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
    @JvmStatic external fun call_callback_return_resource_new_9e73b822affaedb3(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_resource_live_b530072d9d196364(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_owned_5cfe5c76741495b8(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_take_3ffb2a58590a9f8f(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_owned_optional_3f9c79bb9e2a1c84(p0_callback: Any?, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_callback_return_owned_vector_c955ff3794caa0f7(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_spawn_7212e8a490c09a14(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_join_1a06270dc90abd5f(p0_w1: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_handler_new_773c695edbbbf41e(p0_callback: Any?, p1_w1: Boolean, returns: Return_Empty_ErrorString1_6b62d91b16305083): Long
    @JvmStatic external fun call_callback_return_handler_call_ea4b685d6ef94d77(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_catch_fb340e4965e59b90(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_handler_many_8518ace5a4249c8b(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_char_catch_2304a076cc2d2d5d(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_callback_return_owned_vector_catch_712110277489aed4(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_unit_dd4499fffb863427(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_return_handler_value_a23aec1750dc3226(p0_w2: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_typed_rows_worker_6b9e6aab9bf9dcf9(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_typed_rows_limit_2e92146cd4dd53c0(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_typed_rows_wide_3a9a88de1b1dce19(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_typed_rows_mixed_f837b40bf5799358(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_callback_rows_ref_e8fc186edb0f16d5(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_row_return_cleanup_failure_056118ca27a67821(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_unique_resource_id_e80db4a3f497e062(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_rows_unique_019d8d474de1bdc3(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_unique_error_40154160e9803c99(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_row_resource_id_dc9253bb393e6277(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_row_resource_live_510223cb2d40d9e1(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_row_resource_drops_01de29492e0e46ba(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_row_resource_hold_a82671c2a7310f8d(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_owned_probe_18cebe9ab6081eb8(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_optional_probe_bed3ad3142d5a05b(p0_callback: Any?, p1_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_nested_probe_88c40199386d8891(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_pair_probe_de30e0e31a7df3e7(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_transferred_probe_a4ac030f6ad6a92d(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_transferred_pair_probe_43ce9ece795705de(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_owned_3e990348ad64d75b(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_borrowed_57b23a1fb6669005(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_optional_0d52c7d730c41d75(p0_callback: Any?, p1_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_nested_5da56d6464038743(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_pair_a961f029753aca2e(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_row_handles_0883c9c06120fdc3(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_transferred_bab23b0a8a8f4840(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_transferred_pair_e141e947123b4d58(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_conversion_failure_9ae85f90d1f5c256(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_spawn_4aac2c41997a9b73(p0_callback: Any?, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_row_job_finished_e593d9b0a16e2bc6(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
    @JvmStatic external fun call_callback_reentry_new_7c866d733e6695ab(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_read_163ceba922956210(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_write_a7d55ad80b6fe129(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_reentry_take_833f1b6bd5ec9e72(p0_w1: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_shared_24c2640bb542ea50(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_exclusive_4a731f58f2740351(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_consuming_3be54c5a354f741d(p0_w1: Long, p1_callback: Any?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_pair_cf445d8ea75641d4(p0_w1: Long, p1_w1: Long, p2_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_mixed_e6aa9763b3d05c8d(p0_w1: Long, p1_w1: Long, p2_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_worker_0a51904596acdc7a(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_probe_d336877ed9e13d45(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_optional_9676511d65a8c7e8(p0_w3: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_constructed_3e15b9644deb63ff(p0_w2: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_constructor_new_4d91f17ba5ed43a7(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_constructor_fire_f870340c7995bc02(p0_w2: Any?, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_owned_fire_5025d0be7b00c9b9(p0_w2: Any?, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_pair_9c4ca5ba547ab06e(p0_w2: Any?, p1_w2: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_mixed_0d8af3de66dbc771(p0_callback: Any?, p1_w2: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_set_fire_66ee90d277a83d28(p0_w2: Any?, p0_w4: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_nested_cb4b380338cdfee2(p0_w3: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_spawn_34c96d991b52d675(p0_w2: Any?, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_constructor_run_1483f2b3c5476def(p0_w1: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_selected_f4b532a5cf25fe0c(p0_w1: Int, p0_w4: Any?, p0_w6: Long, p1_w1: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_optional_7e8221b306fdc76a(p0_w1: Int, p0_w4: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
    @JvmStatic external fun call_callback_constructor_list_9de391e0ca3c5896(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_checked_68d8204b4d954b4b(p0_w2: Any?, p1_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_set_new_84d26d4b7902b874(p0_callback: Any?, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_set_fire_fd8448ed712f5ecd(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_set_spawn_b31481854c69274a(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_job_join_3bface1ca474cf53(p0_w1: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_owned_new_1ad5877d98b31f02(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_owned_id_43797cc4da3830d1(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_owned_drops_d617a886f0baab03(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_owned_fire_5ac0f47b60fb0ab1(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_owned_borrow_18156795b5ab342a(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_transferred_id_2de587e529e4f1e6(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_transferred_drops_4ff92471b79e1575(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_transferred_fire_b5720cc21e6a881a(p0_callback: Any?, p1_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_projection_fire_fab0fb05d382a2b3(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_projected_id_ab694d1e4f294104(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_preflight_f3d64440a541f0fe(p0_w1: Long, p1_callback: Any?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_scalars_107e667aaf4d1540(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_texts_c776f9ba3bb1e5fa(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_optional_owned_c339b71b13e68762(p0_callback: Any?, p1_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_envelope_bede25767fde1a47(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_primitives_ff126f0005bfa2b5(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_enums_b6723b6e70db554a(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_native_bulk_rows_f3ef23b4e405ea00(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_native_bulk_nested_a866f03d6554ae4e(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_native_bulk_pair_a4a8c00804a0cda0(p0_w1: Long, returns: Return_ByteArray_ByteArray2_ErrorEmpty_cdd3629414305251): Unit
    @JvmStatic external fun call_native_bulk_callback_b0aa3d8beb220a07(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_output_cleanup_surround_58e43f8fdb96cdf0(returns: Return_Long_Array_Any___Long3_ErrorEmpty_2b0b20baf8d8d1fd): Unit
    @JvmStatic external fun call_output_cleanup_surround_bulk_f8fb155e97de0ce3(returns: Return_Long_ByteArray_Long3_ErrorEmpty_1d2a2254ab7aac6e): Unit
    @JvmStatic external fun call_output_cleanup_rows_d61c4972dafc1c15(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_output_cleanup_rows_bulk_b9d0f7fac05677c7(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_output_cleanup_nested_de8d2b4b9cdb3154(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_output_cleanup_nested_bulk_16ea88d5215cb029(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_output_cleanup_callback_3661b832911fa506(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_output_cleanup_callback_bulk_4a1f8c925d7c71c7(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_collection_scalars_d9a5730159aaf1ce(p0_w1: Array<Any?>?, p1_w1: Double, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_u128_04e9df44fc750d1f(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_construct_all_301fa7e444a58a6b(p0_w1: Array<Any?>?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_make_items_c6725210ed9bf318(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_bundle_f76991cc8c87924d(returns: Return_ByteArray_Long2_ErrorEmpty_34705c584e211183): Unit
    @JvmStatic external fun call_collection_optional_handles_9bea1ae6ac9dc934(p0_w3: Array<Any?>?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_nested_handles_340f5f0733d40480(p0_w1: Array<Any?>?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_numbers_58708da43cac559a(returns: Return_ByteArray_Long2_ErrorEmpty_34705c584e211183): Unit
    @JvmStatic external fun call_collection_resource_ids_f5c907b9c01914ba(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_i64_90bde5aeb8c647ea(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_text_19d59571edb173e9(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_bytes_f8c8c3aaf2aa2be0(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_nested_4394d3d5a3b0ca8f(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_options_a293ccc475eb588e(p0_w1: Array<Any?>?, p1_w1: Short, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_optional_1f78a6a6c734d45b(p0_w3: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_optional_items_a463b73617be6af0(p0_w3: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_enum_a72cd6089a34be29(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_probe_cf67d7568b2f663c(p0_w1: Array<Any?>?, p1_w1: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_borrow_0edafd22349f21ff(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_slice_00044f8d9833eed8(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_resource_new_7274517961a66222(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_handles_62cdb8fbfb349eda(p0_w1: Array<Any?>?, p1_w1: Int, p2_w1: Int, ownership: LongArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): ByteArray?
    @JvmStatic external fun call_collection_alias_2250b76699c9c260(p0_w1: Array<Any?>?, p1_w1: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_result_be11b22c862519eb(p0_w1: Array<Any?>?, p1_w1: Boolean, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): ByteArray?
    @JvmStatic external fun call_collection_fail_e960b89bf99875af(returns: Return_Empty_ErrorByteArray_ByteArray2_8bb1103e963f51d1): Unit
    @JvmStatic external fun call_collection_result_optional_4054433f3ab4e1e5(p0_w1: Array<Any?>?, p1_w1: Int, ownership: LongArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): ByteArray?
    @JvmStatic external fun call_storage_put_slice_2321b992f47ae6b3(p0_w1: Long, p1_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_storage_get_vec_41fb29ce4d0d84db(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_c_result_error_drop_8c05c7fda32d0f81(returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Unit
    @JvmStatic external fun call_c_result_late_niche_9d6cee99c69463b1(p0_w1: Boolean, returns: Return_Long_Int2_ErrorLong_String2_d7386b6fbdbb3a71): Unit
    @JvmStatic external fun call_c_result_resource_error_f29a434ac3a17580(p0_w1: Boolean, returns: Return_Empty_ErrorLong_String2_a3fb4f88f2014cb0): Unit
    @JvmStatic external fun call_c_result_rich_error_e97eeeaedd2fcd23(p0_w1: Boolean, returns: Return_Empty_ErrorInt_String2_b786195c4ed75c25): Unit
    @JvmStatic external fun call_result_scalar_717ad7e5f39fb113(p0_w1: Long, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Long
    @JvmStatic external fun call_result_unit_error_6007a70d185fe955(p0_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_result_unit_fc0177d655b15009(p0_w1: Boolean, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Unit
    @JvmStatic external fun call_result_optional_01608adeaf654d25(p0_w1: Int, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Int
    @JvmStatic external fun call_result_text_d4e50fb117070bd3(p0_w1: Boolean, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): String?
    @JvmStatic external fun call_result_record_1be5deeed131a35d(p0_w1: Boolean, returns: Return_Int_Int2_ErrorLong1_d482e1a7516fbcfa): Unit
    @JvmStatic external fun call_result_owned_a3e41c438ccc8a54(p0_w1: Long, p1_w1: Int, p2_w1: Int, ownership: LongArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Long
    @JvmStatic external fun call_result_other_bf6ac73e7d98d05c(p0_w1: Boolean, returns: Return_Empty_ErrorString1_6b62d91b16305083): Long
    @JvmStatic external fun call_storage_try_with_label_1db3fb1ae12e2bed(p0_w1: String?, returns: Return_Empty_ErrorString1_6b62d91b16305083): Long
    @JvmStatic external fun call_result_error_drops_37b19328e81c2636(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_enum_boundary_probe_602c56993c15d467(p0_w1: ByteArray?, p1_w5: Int, p2_w1: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_enum_payload_new_9a992a3f2e4e9d40(p0_w1: Double, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_payload_priority_05fa05ed04713bec(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_priority_weight_bcf2d93ac3538d33(p0_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_priority_or_8b7e9d67b98b36fd(p0_w3: Int, p1_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_sparse_echo_ff112271a9f96bd5(p0_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_sparse_optional_d96a4fb0a1baa04f(p0_w3: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_sparse_nested_80137c780e0dbc13(p0_w5: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_sparse_ref_d09b189574416726(p0_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_sparse_borrowed_2c5728a75c56fd13(p0_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_enum_record_echo_1fa3f892fbef60f0(p0_w1_f0: Int, p0_w1_f1_some: Int, returns: Return_Int_Int2_ErrorEmpty_1d2a9157b2d89875): Unit
    @JvmStatic external fun call_enum_names_echo_45deb1696c7faead(p0_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_niche_choice_new_a532294534010060(p0_w3: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_niche_choice_id_61dfc8efcd02257b(p0_w3: Int, p0_w8: Int, p0_w10: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_nested_u32_2e0792d512f0f24f(p0_w5: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_triple_text_cae6b50b503944e6(p0_w3: Int, p0_w7: String?, returns: Return_Int_String2_ErrorEmpty_7e68b805da700de5): Unit
    @JvmStatic external fun call_option_triple_bytes_4f662b4b06709c31(p0_w3: Int, p0_w7: ByteArray?, returns: Return_Int_ByteArray2_ErrorEmpty_a2aad6159c8c4c14): Unit
    @JvmStatic external fun call_option_triple_i64_53dfd00875940117(p0_w5: Int, p0_w7: Long, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_triple_unit_c0762ed5037f126a(p0_w5: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_optional_text_input_len_5334cfb37cec76c0(p0_w4: String?, p0_w6: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_optional_bytes_input_len_7c4fd8ed8adbd4f7(p0_w4: ByteArray?, p0_w6: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nested_niche_choice_id_e0ad555f54dbd8e9(p0_w5: Int, p0_w10: Int, p0_w12: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_nested_projection_views_d622fb44db024930(p0_w1: Boolean, p1_w1: Boolean, returns: Return_Int_Long_Long3_ErrorEmpty_2e9e07b44c221bd1): Unit
    @JvmStatic external fun call_option_ctor_new_3e8a52ff9fd43303(p0_w1: Int, p0_w3: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_ctor_id_649c578491a01066(p0_w1: Int, p0_w4: Int, p0_w6: Long, p0_w8: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_nested_resource_8f975483d195b195(p0_w1: Int, p0_w5: Long, ownership: LongArray, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_choice_id_1dd579df3bd36ecc(p0_w3: Int, p0_w6: Long, p0_w8: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_str_echo_94daf2f0c810e1a7(p0_w3: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_option_slice_echo_d601bb0588850bdb(p0_w3: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_option_str_len_e1179a8d3f3ccc82(p0_w3: String?, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_slice_len_c45226bd8197ca90(p0_w3: ByteArray?, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_string_len_5af3d51e1e87974d(p0_w3: String?, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_i64_ref_fda4b6cd002e52b6(p0_w1: Int, p0_w3: Long, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_resource_pair_fdf6ca6e5732a477(p0_w3: Long, p1_w3: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_resource_choice_id_0bd95922697c4442(p0_w4: Long, p0_w6: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_optional_pair_new_a373cf8922fb383f(returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_option_projection_views_5f2b052a4aadad2a(p0_w1: Boolean, returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_optional_bytes_echo_16b56811763936ed(p0_w1_f0_some: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_option_resource_mut_6ada7b8856f978ef(p0_w3: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_projection_ref_00f05d41cfe82ce6(p0_w1: Long, p1_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_i64_351387af348bdd22(p0_w1: Int, p0_w3: Long, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_char_8b0e6ac80fbfb499(p0_w3: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_option_u8_28e6cb3d73ac58b7(p0_w3: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_option_bool_abd9a75cd1e6da39(p0_w1: Int, p0_w3: Boolean, returns: Return_Int_Boolean2_ErrorEmpty_2c463b65ab58550c): Unit
    @JvmStatic external fun call_option_text_1006bf1c6da09727(p0_w3: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_option_bytes_2c7d160b5c896230(p0_w3: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_option_unit_d5c35f5bcf4cdd85(p0_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_option_nested_3c7b2ba952e35029(p0_w3: Int, p0_w5: Long, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_nested_char_0fbb5a6359c6a2ee(p0_w5: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_option_resource_new_cd6aa3cab1e8ab93(p0_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_resource_echo_2bb01ab528b6954b(p0_w3: Long, p1_w1: Int, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_resource_id_c42dbb9913c07a92(p0_w3: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_optional_fields_echo_303b818252f2c60b(p0_w1: Int, p0_w3_f0_present: Int, p0_w3_f0_some: Long, p0_w3_f1_some: String?, p0_w3_f2_some: Int, returns: Return_Int_Int_Long5_ErrorEmpty_072908d3b52dbcec): Unit
    @JvmStatic external fun call_optional_field_new_16797b4e790c764e(p0_w1: Boolean, p1_w1: Int, p1_w3: Long, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_optional_resource_new_79dab16fa38a4430(p0_w1: Boolean, returns: Return_Int_Long_Int7_ErrorEmpty_7a075b93a01b6645): Unit
    @JvmStatic external fun call_stamp_new_e7d8c1d49f7ad8b1(p0_w1: Long, p1_w1: Long, returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_plain_note_echo_04effec8337b6f4f(p0_w3: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_unsigned_round_trip_b59be7bd92f5b575(p0_w1: Int, p1_w1: Int, p2_w1: Long, p3_w1: Long, p4_w1: Int, p4_w3: Long, returns: Return_Int_Int_Long6_ErrorEmpty_b3ecce38dd00826e): Unit
    @JvmStatic external fun call_unsigned_optional_7f17c540c43bf08f(p0_w1: Int, p0_w3: Long, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_blob_value_new_34260fb08c70acda(p0_w1: Long, p1_w1: ByteArray?, returns: Return_Long_Long_ByteArray3_ErrorEmpty_aa6ead24a4efe458): Unit
    @JvmStatic external fun call_blob_value_echo_711e78e87dab6447(p0_w1_f0_f0: Long, p0_w1_f0_f1: Long, p0_w1_f1: ByteArray?, returns: Return_Long_Long_ByteArray3_ErrorEmpty_aa6ead24a4efe458): Unit
    @JvmStatic external fun call_archive_new_e55a20288bf01013(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_archive_store_aee2ebbb250b7a73(p0_w1: Long, p1_w1: Int, p1_w4: Long, p1_w6: Double, p1_w8: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_archive_latest_c7923acc8a7d647a(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_storage_new_a15b46920214a065(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_storage_get_b38090762950a015(p0_w1: Long, returns: Return_Int_Long_Int5_ErrorEmpty_1e88956ed04b15f0): Unit
    @JvmStatic external fun call_storage_put_opt_179bf292b0cec6bb(p0_w1: Long, p1_w1: Int, p1_w4: Long, p1_w6: Int, p1_w8: Double, p1_w10: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
    @JvmStatic external fun call_string_new_e13dcae0deb5ba72(p0_w1: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_string_len_0599f1bfcc1a4ae6(p0_w2: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_i64_ref_double_fe479ebaa0f31f86(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_text_echo_219b404084eaf51f(p0_w1: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_bytes_echo_a8ab3cbd8140d73a(p0_w1: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_report_new_8c29e03007e0b826(p0_w1: Long, p1_w1: Double, p2_w1: String?, returns: Return_Long_Double_Long6_ErrorEmpty_a10857827755376a): Unit
    @JvmStatic external fun call_report_into_struct_c0775a0f2b6c3486(p0_w1: Int, p0_w3: Long, p0_w6: Long, p0_w8: Double, p0_w10: String?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_payload_new_4db541865be00283(p0_w1: Long, p1_w1: Int, p2_w1: Double, p3_w1: Boolean, returns: Return_Long_Int_Double4_ErrorEmpty_f89d68e81a93e976): Unit
    @JvmStatic external fun call_storage_with_payload_b44f150bf309e573(p0_w2: Long, p0_w4: Int, p0_w6: Double, p0_w8: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_summary_new_c8a7979c4a30e563(p0_w1: Long, p1_w1: Double, returns: Return_Long_Double2_ErrorEmpty_a4ec6c8f44b5ff7d): Unit
    @JvmStatic external fun call_summary_total_raw_1a3f55f0a5f07787(p0_w1: Int, p0_w4: Long, p0_w6: Double, p0_w8: Long, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Double
    @JvmStatic external fun call_primitive_i8_8b845afaa2c2b930(p0_w1: Byte, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Byte
    @JvmStatic external fun call_primitive_i16_f014be4488893e1b(p0_w1: Short, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Short
    @JvmStatic external fun call_primitive_i32_47001a8bec684907(p0_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_primitive_i64_80af91ea21ca8b59(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_primitive_i128_7c7adcb71f399482(p0_w1: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_primitive_isize_f73f4c9446afe299(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_primitive_u8_524c6ffed8776520(p0_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_primitive_u16_f914d3a206cd401b(p0_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_primitive_u32_c7f6c43082a5b737(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_primitive_u64_180c1433590aabe9(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_primitive_u128_ada5db066814cfea(p0_w1: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_primitive_usize_c1dde3a165f98019(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_primitive_f32_0c5b24fab8306069(p0_w1: Float, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Float
    @JvmStatic external fun call_primitive_f64_3ac16aca0f3ac727(p0_w1: Double, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Double
    @JvmStatic external fun call_primitive_bool_044d0aea7d03630a(p0_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
    @JvmStatic external fun call_primitive_char_60d88328057d650a(p0_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_primitive_unit_d880ab199b1d4c5a(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_report_label_364968097c832e7c(p0_w2: Int, p0_w5: Long, p0_w7: Double, p0_w9: Long, p0_w11: String?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_report_relabel_3cf665ccef08aaa6(p0_w1: Long, p1_w1: String?, ownership: LongArray, returns: Return_Long_Double_Long6_ErrorEmpty_a10857827755376a): Unit
    @JvmStatic external fun call_storage_len_b50243a8692d2f03(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_storage_contains_8bc866018d04e804(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
    @JvmStatic external fun call_storage_total_len_5586730a7f37c6df(p0_w1: Long, p1_w1: Long, p2_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_storage_accumulate_lengths_7831d8906339cda9(p0_w1: Long, p1_w1: Long, p2_w1: Long, p3_w1: Long, p4_w1: Long, p5_w1: Long, p6_w1: Long, p7_w1: Long, p8_w3: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_storage_put_by_take_b2b6012205db749f(p0_w1: Long, p1_w2: Long, p1_w4: Int, p1_w6: Double, p1_w8: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_summary_merge_72601e4c502719c5(p0_w1: Int, p0_w4: Long, p0_w6: Double, p0_w8: Long, p1_w1: Int, p1_w4: Long, p1_w6: Double, p1_w8: Long, ownership: LongArray, returns: Return_Long_Double2_ErrorEmpty_a4ec6c8f44b5ff7d): Unit
    @JvmStatic external fun call_storage_summary_handle_dd25a249548287f1(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_projection_pair_borrowed_bc9032fce77ca56f(returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_projection_pair_owned_eda94d58bc123b34(returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_projection_pair_callback_7a64636be1de6f9f(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_projection_pair_mixed_833184fb3cea18c0(returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_projection_pair_views_05197d1569e216c6(returns: Return_Long_Long_Long4_ErrorEmpty_4a91da1fbd5b4965): Unit
    @JvmStatic external fun call_projection_clones_4cb30df2ddad16fb(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_projection_drops_0b54af22f7d4ea40(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_resource_bundle_new_4c184c76ecdb9bcd(returns: Return_String_Long_Long7_ErrorEmpty_dca1ad735f69d5d9): Unit
    @JvmStatic external fun call_resource_bundle_default_69a6f8678d97e5a4(returns: Return_String_Long_Long7_ErrorEmpty_dca1ad735f69d5d9): Unit
    @JvmStatic external fun call_resource_bundle_fields_552abfd813e477ad(returns: Return_String_Long_Long7_ErrorEmpty_dca1ad735f69d5d9): Unit
    @JvmStatic external fun call_api_record_new_1c66bd6fa63a1f6a(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_api_record_views_18103c07542f54c4(returns: Return_Long_String2_ErrorEmpty_1b009765d5d6eed0): Unit
    @JvmStatic external fun call_owned_resource_id_226d1cb8f69ef53f(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_owned_resource_drops_27038bacb19950fb(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_resource_bundle_views_98686c0f53809c0c(returns: Return_String_Long_Long8_ErrorEmpty_74faddb97f994c93): Unit
    @JvmStatic external fun call_data_primitives_echo_c1f83db309ac8aab(p0_w1: Int, p0_w3_f0: Byte, p0_w3_f1: Short, p0_w3_f2: Int, p0_w3_f3: Long, p0_w3_f4: ByteArray?, p0_w3_f5: Long, p0_w3_f6: Int, p0_w3_f7: Int, p0_w3_f8: Long, p0_w3_f9: Long, p0_w3_f10: ByteArray?, p0_w3_f11: Long, p0_w3_f12: Float, p0_w3_f13: Double, p0_w3_f14: Boolean, p0_w3_f15: Int, returns: Return_Byte_Short_Int16_ErrorEmpty_4320f95dddb29024): Unit
    @JvmStatic external fun call_data_primitives_zero_52866d71f82311ff(returns: Return_Byte_Short_Int16_ErrorEmpty_4320f95dddb29024): Unit
    @JvmStatic external fun call_data_envelope_echo_3968cde0709118c6(p0_w1_f0: String?, p0_w1_f1_f0_f0: Long, p0_w1_f1_f0_f1: Long, p0_w1_f1_f1: ByteArray?, p0_w1_f2_f0: Int, p0_w1_f2_f1: Int, p0_w1_f2_f2: Long, p0_w1_f2_f3: Long, p0_w1_f2_f4_present: Int, p0_w1_f2_f4_some: Long, p0_w1_f3: Boolean, p0_w1_f4: Int, p0_w1_f5: ByteArray?, p0_w1_f6: ByteArray?, returns: Return_String_Long_Long14_ErrorEmpty_f6cccd4cd56d1dfa): Unit
    @JvmStatic external fun call_data_with_storage_ba1c94c812ca7dc1(p0_w1: Long, p1_w1_f0: String?, p1_w1_f1_f0_f0: Long, p1_w1_f1_f0_f1: Long, p1_w1_f1_f1: ByteArray?, p1_w1_f2_f0: Int, p1_w1_f2_f1: Int, p1_w1_f2_f2: Long, p1_w1_f2_f3: Long, p1_w1_f2_f4_present: Int, p1_w1_f2_f4_some: Long, p1_w1_f3: Boolean, p1_w1_f4: Int, p1_w1_f5: ByteArray?, p1_w1_f6: ByteArray?, ownership: LongArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_data_envelope_constructed_dab47fa5443a2b68(p0_w2_f0_f0: Long, p0_w2_f0_f1: Long, p0_w2_f1: ByteArray?, returns: Return_String_Long_Long14_ErrorEmpty_f6cccd4cd56d1dfa): Unit
    @JvmStatic external fun call_data_floats_echo_cfa4c7cc9e4653c8(p0_w1_f0: ByteArray?, p0_w1_f1: Float, p0_w1_f2: Double, returns: Return_ByteArray_Float_Double3_ErrorEmpty_796e683dacd5930a): Unit
    @JvmStatic external fun call_blob_value_selected_bb0c7d8f79d57c4b(p0_w1: Int, p0_w3_f0_f0: Long, p0_w3_f0_f1: Long, p0_w3_f1: ByteArray?, p0_w6: Long, p0_w8: ByteArray?, returns: Return_Long_Long_ByteArray3_ErrorEmpty_aa6ead24a4efe458): Unit
    @JvmStatic external fun call_blob_value_views_9983ceb667085a50(p0_w1_f0_f0: Long, p0_w1_f0_f1: Long, p0_w1_f1: ByteArray?, returns: Return_Long_Long_ByteArray9_ErrorEmpty_3b29111153baa628): Unit
    internal interface SequenceCapacity { fun reserve(capacity: Int) }
    @JvmStatic private external fun init_SequenceCapacity_e067e94ae7b50e2a(capacity: Class<*>)
    init { init_SequenceCapacity_e067e94ae7b50e2a(SequenceCapacity::class.java) }
    internal interface SequenceSink_default_bulk_array_fold_95a8899ff0517a5d : SequenceCapacity { fun run(v0: Long, v1: Double) }
    internal class SequenceCollector_default_bulk_array_fold_385d020277360226 : SequenceSink_default_bulk_array_fold_95a8899ff0517a5d {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_Double_2_012db11776ed72c5>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long, v1: Double) { rows.add(SequenceRow_Long_Double_2_012db11776ed72c5(v0, v1)) }
    }
    @JvmStatic external fun call_default_bulk_array_fold_8b682a05955403df(callback: SequenceSink_default_bulk_array_fold_95a8899ff0517a5d, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_default_bulk_optional_fold_98317e3c22588d99 : SequenceCapacity { fun run(v0: Long, v1: Double) }
    internal class SequenceCollector_default_bulk_optional_fold_514df038c7a4b1ee : SequenceSink_default_bulk_optional_fold_98317e3c22588d99 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_Double_2_012db11776ed72c5>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long, v1: Double) { rows.add(SequenceRow_Long_Double_2_012db11776ed72c5(v0, v1)) }
    }
    @JvmStatic external fun call_default_bulk_optional_fold_a9233ea21645798f(p0_w1: Long, callback: SequenceSink_default_bulk_optional_fold_98317e3c22588d99, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_gc_resource_rows_fold_98b54dd95d8e015f : SequenceCapacity { fun run(v0: Long) }
    internal class SequenceCollector_gc_resource_rows_fold_a0cec018fb39eb7c : SequenceSink_gc_resource_rows_fold_98b54dd95d8e015f {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_1_127a3c24b600dbe0>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long) { rows.add(SequenceRow_Long_1_127a3c24b600dbe0(v0)) }
    }
    @JvmStatic external fun call_gc_resource_rows_fold_5c247ce3042d4525(p0_w1: Long, p1_w1: Long, callback: SequenceSink_gc_resource_rows_fold_98b54dd95d8e015f, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_jni_output_mixed_frames_fold_b4828e3a4ee22135 : SequenceCapacity { fun run(v0: ByteArray?) }
    internal class SequenceCollector_jni_output_mixed_frames_fold_a5668456db986db2 : SequenceSink_jni_output_mixed_frames_fold_b4828e3a4ee22135 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_ByteArray_1_92dc0f7ab995287c>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: ByteArray?) { rows.add(SequenceRow_ByteArray_1_92dc0f7ab995287c(v0)) }
    }
    @JvmStatic external fun call_jni_output_mixed_frames_fold_e9474c6b8b0056cf(p0_w1: Long, callback: SequenceSink_jni_output_mixed_frames_fold_b4828e3a4ee22135, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_jni_output_fold_drop_fold_b6a0b19ec3a0bfe6 : SequenceCapacity { fun run(v0: Array<Any?>?) }
    internal class SequenceCollector_jni_output_fold_drop_fold_a39a891c792a6c19 : SequenceSink_jni_output_fold_drop_fold_b6a0b19ec3a0bfe6 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Array_Any___1_00f39593b4ea53ea>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Array<Any?>?) { rows.add(SequenceRow_Array_Any___1_00f39593b4ea53ea(v0)) }
    }
    @JvmStatic external fun call_jni_output_fold_drop_fold_8e0eaffd2786301c(callback: SequenceSink_jni_output_fold_drop_fold_b6a0b19ec3a0bfe6, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_jni_output_fold_fold_a6fbab1712a6f896 : SequenceCapacity { fun run(v0: Long, v1: String?, v2: Long) }
    internal class SequenceCollector_jni_output_fold_fold_da0d511a519a94f5 : SequenceSink_jni_output_fold_fold_a6fbab1712a6f896 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_String_Long_3_b4558cb7b740d7c6>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long, v1: String?, v2: Long) { rows.add(SequenceRow_Long_String_Long_3_b4558cb7b740d7c6(v0, v1, v2)) }
    }
    @JvmStatic external fun call_jni_output_fold_fold_30e359f733b27f80(p0_w1: Long, callback: SequenceSink_jni_output_fold_fold_a6fbab1712a6f896, outputAccepted: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Int
    internal interface SequenceSink_callback_boundary_fold_fold_0165aba4bc8f4bac : SequenceCapacity { fun run(v0: Long) }
    internal class SequenceCollector_callback_boundary_fold_fold_acbaca85aff36cbb : SequenceSink_callback_boundary_fold_fold_0165aba4bc8f4bac {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_1_127a3c24b600dbe0>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long) { rows.add(SequenceRow_Long_1_127a3c24b600dbe0(v0)) }
    }
    @JvmStatic external fun call_callback_boundary_fold_fold_f4fcc537824dc196(p0_callback: Any?, p1_w1: Long, callback: SequenceSink_callback_boundary_fold_fold_0165aba4bc8f4bac, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_output_cleanup_rows_fold_75168968ea0ba55c : SequenceCapacity { fun run(v0: Long, v1: Long, v2: Long) }
    internal class SequenceCollector_output_cleanup_rows_fold_a3d8debfd37d4def : SequenceSink_output_cleanup_rows_fold_75168968ea0ba55c {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_Long_Long_3_ad466713697fa9e4>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long, v1: Long, v2: Long) { rows.add(SequenceRow_Long_Long_Long_3_ad466713697fa9e4(v0, v1, v2)) }
    }
    @JvmStatic external fun call_output_cleanup_rows_fold_d4a060fc1b46cffe(callback: SequenceSink_output_cleanup_rows_fold_75168968ea0ba55c, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_output_cleanup_rows_bulk_fold_c0b0ae2e88fae38a : SequenceCapacity { fun run(v0: ByteArray?) }
    internal class SequenceCollector_output_cleanup_rows_bulk_fold_aeddb997c707853d : SequenceSink_output_cleanup_rows_bulk_fold_c0b0ae2e88fae38a {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_ByteArray_1_92dc0f7ab995287c>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: ByteArray?) { rows.add(SequenceRow_ByteArray_1_92dc0f7ab995287c(v0)) }
    }
    @JvmStatic external fun call_output_cleanup_rows_bulk_fold_8a047108d9782a30(callback: SequenceSink_output_cleanup_rows_bulk_fold_c0b0ae2e88fae38a, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_output_cleanup_nested_fold_697431074ae8d31b : SequenceCapacity { fun run(v0: Array<Any?>?) }
    internal class SequenceCollector_output_cleanup_nested_fold_f1c07d387aa43c20 : SequenceSink_output_cleanup_nested_fold_697431074ae8d31b {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Array_Any___1_00f39593b4ea53ea>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Array<Any?>?) { rows.add(SequenceRow_Array_Any___1_00f39593b4ea53ea(v0)) }
    }
    @JvmStatic external fun call_output_cleanup_nested_fold_4f16ec74e528db69(callback: SequenceSink_output_cleanup_nested_fold_697431074ae8d31b, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_output_cleanup_nested_bulk_fold_a1404af9d5f22fac : SequenceCapacity { fun run(v0: ByteArray?) }
    internal class SequenceCollector_output_cleanup_nested_bulk_fold_f9eaa6c74f7feefb : SequenceSink_output_cleanup_nested_bulk_fold_a1404af9d5f22fac {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_ByteArray_1_92dc0f7ab995287c>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: ByteArray?) { rows.add(SequenceRow_ByteArray_1_92dc0f7ab995287c(v0)) }
    }
    @JvmStatic external fun call_output_cleanup_nested_bulk_fold_2c8fd0b69d9609fe(callback: SequenceSink_output_cleanup_nested_bulk_fold_a1404af9d5f22fac, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal class SequenceRow_Boolean_Byte_Short_9_fa65775165ad52ea(@JvmField val c0: Boolean, @JvmField val c1: Byte, @JvmField val c2: Short, @JvmField val c3: Int, @JvmField val c4: Long, @JvmField val c5: Float, @JvmField val c6: Double, @JvmField val c7: ByteArray?, @JvmField val c8: String?)
    internal interface SequenceSink_collection_scalars_fold_70028742f64f0191 : SequenceCapacity { fun run(v0: Boolean, v1: Byte, v2: Short, v3: Int, v4: Long, v5: Float, v6: Double, v7: ByteArray?, v8: String?) }
    internal class SequenceCollector_collection_scalars_fold_b8b5464d6a39b802 : SequenceSink_collection_scalars_fold_70028742f64f0191 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Boolean_Byte_Short_9_fa65775165ad52ea>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Boolean, v1: Byte, v2: Short, v3: Int, v4: Long, v5: Float, v6: Double, v7: ByteArray?, v8: String?) { rows.add(SequenceRow_Boolean_Byte_Short_9_fa65775165ad52ea(v0, v1, v2, v3, v4, v5, v6, v7, v8)) }
    }
    @JvmStatic external fun call_collection_scalars_fold_3627b263ec4e572b(p0_w1: Array<Any?>?, p1_w1: Double, callback: SequenceSink_collection_scalars_fold_70028742f64f0191, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_collection_u128_fold_6f48cddbc993447a : SequenceCapacity { fun run(v0: ByteArray?) }
    internal class SequenceCollector_collection_u128_fold_0c6bd38fdbd087c1 : SequenceSink_collection_u128_fold_6f48cddbc993447a {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_ByteArray_1_92dc0f7ab995287c>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: ByteArray?) { rows.add(SequenceRow_ByteArray_1_92dc0f7ab995287c(v0)) }
    }
    @JvmStatic external fun call_collection_u128_fold_5aa4076e5dce00fc(p0_w1: Array<Any?>?, callback: SequenceSink_collection_u128_fold_6f48cddbc993447a, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_collection_optional_handles_fold_0322a075b01009ab : SequenceCapacity { fun run(v0: Long) }
    internal class SequenceCollector_collection_optional_handles_fold_6b1b4695033414c8 : SequenceSink_collection_optional_handles_fold_0322a075b01009ab {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_1_127a3c24b600dbe0>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long) { rows.add(SequenceRow_Long_1_127a3c24b600dbe0(v0)) }
    }
    @JvmStatic external fun call_collection_optional_handles_fold_cff425efcb503bb5(p0_w3: Array<Any?>?, ownership: LongArray, callback: SequenceSink_collection_optional_handles_fold_0322a075b01009ab, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_collection_nested_handles_fold_0692fa392fcf7c3f : SequenceCapacity { fun run(v0: Array<Any?>?) }
    internal class SequenceCollector_collection_nested_handles_fold_2316fd6b75ceae74 : SequenceSink_collection_nested_handles_fold_0692fa392fcf7c3f {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Array_Any___1_00f39593b4ea53ea>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Array<Any?>?) { rows.add(SequenceRow_Array_Any___1_00f39593b4ea53ea(v0)) }
    }
    @JvmStatic external fun call_collection_nested_handles_fold_8c0143fc3ebd6d9d(p0_w1: Array<Any?>?, ownership: LongArray, callback: SequenceSink_collection_nested_handles_fold_0692fa392fcf7c3f, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_collection_i64_fold_7481b0c247e22a81 : SequenceCapacity { fun run(v0: Long) }
    internal class SequenceCollector_collection_i64_fold_04492883dbff433a : SequenceSink_collection_i64_fold_7481b0c247e22a81 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_1_127a3c24b600dbe0>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long) { rows.add(SequenceRow_Long_1_127a3c24b600dbe0(v0)) }
    }
    @JvmStatic external fun call_collection_i64_fold_808c7d955e03904b(p0_w1: Array<Any?>?, callback: SequenceSink_collection_i64_fold_7481b0c247e22a81, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal class SequenceRow_String_1_b5440014bed04f6a(@JvmField val c0: String?)
    internal interface SequenceSink_collection_text_fold_eee532ef372c2cc4 : SequenceCapacity { fun run(v0: String?) }
    internal class SequenceCollector_collection_text_fold_2069b29da2dbd573 : SequenceSink_collection_text_fold_eee532ef372c2cc4 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_String_1_b5440014bed04f6a>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: String?) { rows.add(SequenceRow_String_1_b5440014bed04f6a(v0)) }
    }
    @JvmStatic external fun call_collection_text_fold_a7b24a9c45b3e186(p0_w1: Array<Any?>?, callback: SequenceSink_collection_text_fold_eee532ef372c2cc4, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_collection_bytes_fold_6625bb31d7dc18eb : SequenceCapacity { fun run(v0: ByteArray?) }
    internal class SequenceCollector_collection_bytes_fold_ba37d21197c79c60 : SequenceSink_collection_bytes_fold_6625bb31d7dc18eb {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_ByteArray_1_92dc0f7ab995287c>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: ByteArray?) { rows.add(SequenceRow_ByteArray_1_92dc0f7ab995287c(v0)) }
    }
    @JvmStatic external fun call_collection_bytes_fold_9e9770856c6c4641(p0_w1: Array<Any?>?, callback: SequenceSink_collection_bytes_fold_6625bb31d7dc18eb, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_collection_nested_fold_63e97a79272db78e : SequenceCapacity { fun run(v0: Array<Any?>?) }
    internal class SequenceCollector_collection_nested_fold_d276919d7d3850f5 : SequenceSink_collection_nested_fold_63e97a79272db78e {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Array_Any___1_00f39593b4ea53ea>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Array<Any?>?) { rows.add(SequenceRow_Array_Any___1_00f39593b4ea53ea(v0)) }
    }
    @JvmStatic external fun call_collection_nested_fold_b79bf8a96a5df66c(p0_w1: Array<Any?>?, callback: SequenceSink_collection_nested_fold_63e97a79272db78e, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal class SequenceRow_Int_Long_2_2d1ae54437bd2ee1(@JvmField val c0: Int, @JvmField val c1: Long)
    internal interface SequenceSink_collection_options_fold_b6fa2828841c6e69 : SequenceCapacity { fun run(v0: Int, v1: Long) }
    internal class SequenceCollector_collection_options_fold_68c36a49e36a111a : SequenceSink_collection_options_fold_b6fa2828841c6e69 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Int_Long_2_2d1ae54437bd2ee1>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Int, v1: Long) { rows.add(SequenceRow_Int_Long_2_2d1ae54437bd2ee1(v0, v1)) }
    }
    @JvmStatic external fun call_collection_options_fold_ded63bd11a5c595b(p0_w1: Array<Any?>?, p1_w1: Short, callback: SequenceSink_collection_options_fold_b6fa2828841c6e69, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_collection_optional_fold_78209d4e9082c96a : SequenceCapacity { fun run(v0: Long) }
    internal class SequenceCollector_collection_optional_fold_df6ab4cb4ee0f229 : SequenceSink_collection_optional_fold_78209d4e9082c96a {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_1_127a3c24b600dbe0>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long) { rows.add(SequenceRow_Long_1_127a3c24b600dbe0(v0)) }
    }
    @JvmStatic external fun call_collection_optional_fold_1efcbb0a1a187e64(p0_w3: Array<Any?>?, callback: SequenceSink_collection_optional_fold_78209d4e9082c96a, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_collection_optional_items_fold_0dd26be8936deb87 : SequenceCapacity { fun run(v0: Int, v1: Long) }
    internal class SequenceCollector_collection_optional_items_fold_f78a529c867c557c : SequenceSink_collection_optional_items_fold_0dd26be8936deb87 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Int_Long_2_2d1ae54437bd2ee1>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Int, v1: Long) { rows.add(SequenceRow_Int_Long_2_2d1ae54437bd2ee1(v0, v1)) }
    }
    @JvmStatic external fun call_collection_optional_items_fold_912719d84a3df595(p0_w3: Array<Any?>?, callback: SequenceSink_collection_optional_items_fold_0dd26be8936deb87, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal class SequenceRow_Int_1_740d8169e2b203e5(@JvmField val c0: Int)
    internal interface SequenceSink_collection_enum_fold_c4e4e95a0f2dd06c : SequenceCapacity { fun run(v0: Int) }
    internal class SequenceCollector_collection_enum_fold_09c4e027e41fd2af : SequenceSink_collection_enum_fold_c4e4e95a0f2dd06c {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Int_1_740d8169e2b203e5>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Int) { rows.add(SequenceRow_Int_1_740d8169e2b203e5(v0)) }
    }
    @JvmStatic external fun call_collection_enum_fold_7a76112af7324a8e(p0_w1: Array<Any?>?, callback: SequenceSink_collection_enum_fold_c4e4e95a0f2dd06c, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_collection_borrow_fold_b4856d1bed461622 : SequenceCapacity { fun run(v0: ByteArray?) }
    internal class SequenceCollector_collection_borrow_fold_7de9c6190123dac1 : SequenceSink_collection_borrow_fold_b4856d1bed461622 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_ByteArray_1_92dc0f7ab995287c>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: ByteArray?) { rows.add(SequenceRow_ByteArray_1_92dc0f7ab995287c(v0)) }
    }
    @JvmStatic external fun call_collection_borrow_fold_22d09716b260bda8(p0_w1: Array<Any?>?, callback: SequenceSink_collection_borrow_fold_b4856d1bed461622, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_collection_slice_fold_2b262532b9b8b02f : SequenceCapacity { fun run(v0: Long) }
    internal class SequenceCollector_collection_slice_fold_a2b3f546f3d7cf6c : SequenceSink_collection_slice_fold_2b262532b9b8b02f {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_1_127a3c24b600dbe0>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long) { rows.add(SequenceRow_Long_1_127a3c24b600dbe0(v0)) }
    }
    @JvmStatic external fun call_collection_slice_fold_6f331d2d69f04ead(p0_w1: Array<Any?>?, callback: SequenceSink_collection_slice_fold_2b262532b9b8b02f, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_collection_handles_fold_94b94355358878bd : SequenceCapacity { fun run(v0: Long) }
    internal class SequenceCollector_collection_handles_fold_d618321dfdd81036 : SequenceSink_collection_handles_fold_94b94355358878bd {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_1_127a3c24b600dbe0>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long) { rows.add(SequenceRow_Long_1_127a3c24b600dbe0(v0)) }
    }
    @JvmStatic external fun call_collection_handles_fold_46dc064168aacb23(p0_w1: Array<Any?>?, p1_w1: Int, p2_w1: Int, ownership: LongArray, callback: SequenceSink_collection_handles_fold_94b94355358878bd, outputAccepted: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Int
    internal interface SequenceSink_collection_result_fold_8dc1da1d06d04f3a : SequenceCapacity { fun run(v0: Long) }
    internal class SequenceCollector_collection_result_fold_ff20bca811548f31 : SequenceSink_collection_result_fold_8dc1da1d06d04f3a {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_1_127a3c24b600dbe0>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long) { rows.add(SequenceRow_Long_1_127a3c24b600dbe0(v0)) }
    }
    @JvmStatic external fun call_collection_result_fold_72fabc4d8e10f6c8(p0_w1: Array<Any?>?, p1_w1: Boolean, callback: SequenceSink_collection_result_fold_8dc1da1d06d04f3a, outputAccepted: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Int
    internal interface SequenceSink_collection_result_optional_fold_522f3e931c5b4b1c : SequenceCapacity { fun run(v0: ByteArray?) }
    internal class SequenceCollector_collection_result_optional_fold_4c5723413f4bd16b : SequenceSink_collection_result_optional_fold_522f3e931c5b4b1c {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_ByteArray_1_92dc0f7ab995287c>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: ByteArray?) { rows.add(SequenceRow_ByteArray_1_92dc0f7ab995287c(v0)) }
    }
    @JvmStatic external fun call_collection_result_optional_fold_235165576a1334d2(p0_w1: Array<Any?>?, p1_w1: Int, ownership: LongArray, callback: SequenceSink_collection_result_optional_fold_522f3e931c5b4b1c, outputAccepted: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Int
    internal interface SequenceSink_sequence_late_drop_fold_c7e3bfa988c8796d : SequenceCapacity { fun run(v0: Long) }
    internal class SequenceCollector_sequence_late_drop_fold_896f50a533967c16 : SequenceSink_sequence_late_drop_fold_c7e3bfa988c8796d {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_1_127a3c24b600dbe0>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long) { rows.add(SequenceRow_Long_1_127a3c24b600dbe0(v0)) }
    }
    @JvmStatic external fun call_sequence_late_drop_fold_7e360d29a6fc0d5f(callback: SequenceSink_sequence_late_drop_fold_c7e3bfa988c8796d, outputAccepted: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Int
    internal interface SequenceSink_sequence_scalars_fold_47a6ace8a9aad013 : SequenceCapacity { fun run(v0: ByteArray?) }
    internal class SequenceCollector_sequence_scalars_fold_380518ae064ebca8 : SequenceSink_sequence_scalars_fold_47a6ace8a9aad013 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_ByteArray_1_92dc0f7ab995287c>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: ByteArray?) { rows.add(SequenceRow_ByteArray_1_92dc0f7ab995287c(v0)) }
    }
    @JvmStatic external fun call_sequence_scalars_fold_5108aab400f755b1(callback: SequenceSink_sequence_scalars_fold_47a6ace8a9aad013, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_sequence_parts_fold_34d4f8eed2926471 : SequenceCapacity { fun run(v0: Long, v1: String?, v2: Long) }
    internal class SequenceCollector_sequence_parts_fold_93c88adeff7d3a7a : SequenceSink_sequence_parts_fold_34d4f8eed2926471 {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_String_Long_3_b4558cb7b740d7c6>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long, v1: String?, v2: Long) { rows.add(SequenceRow_Long_String_Long_3_b4558cb7b740d7c6(v0, v1, v2)) }
    }
    @JvmStatic external fun call_sequence_parts_fold_aeb653ed535ad4ab(p0_w1: Int, callback: SequenceSink_sequence_parts_fold_34d4f8eed2926471, outputAccepted: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Int
    internal class SequenceRow_Long_Int_Double_4_5e82f5e29d89a757(@JvmField val c0: Long, @JvmField val c1: Int, @JvmField val c2: Double, @JvmField val c3: Boolean)
    internal interface SequenceSink_storage_get_vec_fold_886389afd9ad2e6a : SequenceCapacity { fun run(v0: Long, v1: Int, v2: Double, v3: Boolean) }
    internal class SequenceCollector_storage_get_vec_fold_26f37d7c5ac1e8a1 : SequenceSink_storage_get_vec_fold_886389afd9ad2e6a {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_Int_Double_4_5e82f5e29d89a757>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long, v1: Int, v2: Double, v3: Boolean) { rows.add(SequenceRow_Long_Int_Double_4_5e82f5e29d89a757(v0, v1, v2, v3)) }
    }
    @JvmStatic external fun call_storage_get_vec_fold_3251bfb51da9217c(p0_w1: Long, callback: SequenceSink_storage_get_vec_fold_886389afd9ad2e6a, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    internal interface SequenceSink_api_record_rows_fold_5bd1116027b7610c : SequenceCapacity { fun run(v0: Long, v1: String?) }
    internal class SequenceCollector_api_record_rows_fold_3cd13ff4cf046193 : SequenceSink_api_record_rows_fold_5bd1116027b7610c {
        @JvmField val rows = kotlin.collections.ArrayList<SequenceRow_Long_String_2_3ea2c8aa47ab7590>()
        override fun reserve(capacity: Int) { rows.ensureCapacity(capacity) }
        override fun run(v0: Long, v1: String?) { rows.add(SequenceRow_Long_String_2_3ea2c8aa47ab7590(v0, v1)) }
    }
    @JvmStatic external fun call_api_record_rows_fold_847767faa33a5dce(callback: SequenceSink_api_record_rows_fold_5bd1116027b7610c, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
}
