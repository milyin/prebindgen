package io.prebindgen.covertest

internal object JNINative {
    init { io.prebindgen.covertest.NativeLibrary.ensureLoaded() }
    @JvmStatic external fun drop_Archive_0aa4417b9f320a03(id: Long)
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
    @JvmStatic external fun drop_GetterLeaf_b28f97ae79569aa8(id: Long)
    @JvmStatic external fun drop_GetterOwner_bc1803eb5b8c76ef(id: Long)
    @JvmStatic external fun drop_NicheChoiceHandle_820b60fc3ef75039(id: Long)
    @JvmStatic external fun drop_OptionChoice_feab979b8bef4265(id: Long)
    @JvmStatic external fun drop_OptionCtor_9701b4353b1ab7b0(id: Long)
    @JvmStatic external fun drop_OwnedResource_e39aa0af54304c88(id: Long)
    @JvmStatic external fun drop_PayloadHandle_af81cbfac6e10b9d(id: Long)
    @JvmStatic external fun drop_ProjectionHandle_46b41135284839d8(id: Long)
    @JvmStatic external fun drop_Report_965f7809bbde41dd(id: Long)
    @JvmStatic external fun drop_ReportStruct_2659d7dec79ecce0(id: Long)
    @JvmStatic external fun drop_Storage_9651261943550368(id: Long)
    @JvmStatic external fun drop_Summary_caa4a50522c6b815(id: Long)
    @JvmStatic external fun bulkLayouts(): LongArray
    @JvmStatic external fun call_default_bulk_echo_19271b2932c312bb(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_array_f969bde7cc1c432a(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_optional_ea20da6fc813acee(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_nested_799f9a1fa2a2a9a1(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_borrowed_7528ee78c961424e(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_numbers_677f84d2a60bc3b3(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_default_bulk_plain_e53a41329b22bbda(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_default_bulk_callback_af0d489a7a6a5660(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_default_bulk_callback_mixed_d29393bf89d94532(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_bulk_select_b3dae360ea0eecfb(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_kinds_9b033197b22af4fe(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_units_1560a205c63d4c8a(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_flags_fe0ac76234b14846(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_envelope_c78cdfdfd18c33f1(p0_w1: Int, p0_w3_f0_f0: Long, p0_w3_f0_f1: Double, p0_w3_f1: Long, returns: Return_Int_Long_Double4_ErrorEmpty_4244d173b136f580): Unit
    @JvmStatic external fun call_bulk_primitives_be9bffc80840ca87(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_optional_wide_0dbde2915189c95e(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_optional_callback_0b45be4b854f6c11(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_native_939b6cd52b96a019(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_expanded_65b04628441e244b(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_expanded_rows_8611c0e3cec0b75a(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_result_22b6c65613a56b35(p0_w1: ByteArray?, p0_w1_length: Int, p1_w1: Boolean, returns: Return_Empty_ErrorByteArray1_056b654d5078f47e): ByteArray?
    @JvmStatic external fun call_bulk_handle_new_e5a9aa77fa861745(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_handle_value_020d7d074b3973cd(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_drops_0c93c5a0b1492a12(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_late_051f1ea6dcd2506f(p0_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_handle_views_c1f9da5de20e3fc3(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_handles_0a30179490c49dbc(p0_w1: ByteArray?, p0_w1_length: Int, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_borrow_588a078d0466d481(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_bulk_checked_01f9de13d8c4a8b8(p0_w1: ByteArray?, p0_w1_length: Int, p1_w1: ByteArray?, p1_w1_length: Int, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_wide_6b955d7b4ee7dc0d(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_unsigned_wide_407b0f069bb7ed7a(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_callback_worker_4498defd15d5c1bf(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_callback_63ae4882756ac2cf(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_handle_callback_34c4be17902be9ab(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_point_e770ca9da9c12852(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_points_6101af74290565c5(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_point_array_aa3dcf2590ca3a28(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_slice_c30fbd330fdb06e6(p0_w1: ByteArray?, p0_w1_length: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_bulk_point_plain_53e7bd65e4464218(p0_w1_f0: Long, p0_w1_f1: Double, returns: Return_Long_Double2_ErrorEmpty_a4ec6c8f44b5ff7d): Unit
    @JvmStatic external fun call_bulk_optional_handles_f105b0e272c7fda2(p0_w1: ByteArray?, p0_w1_length: Int, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
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
    @JvmStatic external fun call_jni_buffer_rows_0e85d0bcc7902cb5(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_jni_buffer_wide_6983ec538bbdc661(returns: Return_String_String_String48_ErrorEmpty_181c8abbd254d130): Unit
    @JvmStatic external fun call_jni_buffer_callback_a41319fcdf187075(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_jni_buffer_failure_6fdac6591b1a6726(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_jni_buffer_worker_8d2b0e9314030afd(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_jni_buffer_owned_callback_9fa8a662ae731b98(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_getter_owner_new_9b3691104cf4f094(p0_w1: Boolean, returns: Return_Long_Long_String10_ErrorEmpty_f30fbce40c7640f0): Unit
    @JvmStatic external fun call_getter_owner_failure_983fd2bf1ecc08ff(returns: Return_Long_Long_String3_ErrorEmpty_857937f364385c79): Unit
    @JvmStatic external fun call_getter_owner_id_6e5c79e15f3c8ecb(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_getter_leaf_id_2b89bfb1695b1a8e(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_getter_static_leaf_3de084de8ab56cf6(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_getter_owner_read_405afcf1de6633af(p0_w1: Long, returns: Return_Long_String2_ErrorEmpty_1b009765d5d6eed0): Unit
    @JvmStatic external fun call_getter_counts_ac432be3e581775f(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_getter_two_clones_new_c6e10ed44149bf78(returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_getter_fields_new_6223254642400003(returns: Return_Long_Long_String3_ErrorEmpty_857937f364385c79): Unit
    @JvmStatic external fun call_getter_callback_261a8655bbbdf16f(p0_callback: Any?, p1_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_boundary_register_f9d2a0713d344329(p0_callback: Any?, p1_w1: Boolean, returns: Return_Empty_ErrorLong_String2_a3fb4f88f2014cb0): Long
    @JvmStatic external fun call_callback_boundary_fire_a2aad327338bd2a9(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
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
    @JvmStatic external fun call_callback_return_owned_856115d8ce303838(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_take_affc1f4d290ade63(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_owned_optional_29ef924759d7e5bc(p0_callback: Any?, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_callback_return_owned_vector_3ee929da6cb7571b(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_spawn_7212e8a490c09a14(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_join_64d9e75305501c53(p0_w1: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_handler_new_773c695edbbbf41e(p0_callback: Any?, p1_w1: Boolean, returns: Return_Empty_ErrorString1_6b62d91b16305083): Long
    @JvmStatic external fun call_callback_return_handler_call_1b7a8dd7a35a6f9b(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_catch_fb340e4965e59b90(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_handler_many_8518ace5a4249c8b(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_char_catch_2304a076cc2d2d5d(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_callback_return_owned_vector_catch_0161633ca648736c(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_return_unit_dd4499fffb863427(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_return_handler_value_a23aec1750dc3226(p0_w2: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_rows_ref_e8fc186edb0f16d5(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_row_return_cleanup_failure_f029b8a3edf61e39(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_unique_resource_id_f503b662b364158e(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_rows_unique_019d8d474de1bdc3(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_rows_unique_error_40154160e9803c99(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_row_resource_id_754894c5c9affc9b(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_row_resource_live_510223cb2d40d9e1(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_row_resource_drops_01de29492e0e46ba(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_row_resource_hold_5cf985322f526efd(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
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
    @JvmStatic external fun call_callback_row_job_finished_f2cd7260ebbf9dfa(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
    @JvmStatic external fun call_callback_reentry_new_7c866d733e6695ab(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_read_3eac5f6ba179e3a0(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_write_d51c3e7985c5d5b1(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_reentry_take_784cf035dcf4a6fe(p0_w1: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_shared_eb391d1c7013a7e0(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_exclusive_c4c879f41d91e189(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_consuming_ba9f8f27ccfa4a2d(p0_w1: Long, p1_callback: Any?, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_pair_6ab40de513e04e6c(p0_w1: Long, p1_w1: Long, p2_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_mixed_78bee5923db373fd(p0_w1: Long, p1_w1: Long, p2_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_worker_c9405aa7f035bc96(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_probe_b759005342e62645(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_optional_23c1984d5bdcc548(p0_w3: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_reentry_constructed_7dd7a899513e45f3(p0_w2: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_constructor_new_4d91f17ba5ed43a7(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_constructor_fire_f870340c7995bc02(p0_w2: Any?, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_owned_fire_5025d0be7b00c9b9(p0_w2: Any?, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_pair_9c4ca5ba547ab06e(p0_w2: Any?, p1_w2: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_mixed_0d8af3de66dbc771(p0_callback: Any?, p1_w2: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_set_fire_66ee90d277a83d28(p0_w2: Any?, p0_w4: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_nested_cb4b380338cdfee2(p0_w3: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_spawn_34c96d991b52d675(p0_w2: Any?, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_constructor_run_a024114588f63dc3(p0_w1: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_selected_e1407ac02615a674(p0_w1: Int, p0_w4: Any?, p0_w6: Long, p1_w1: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_optional_7e8221b306fdc76a(p0_w1: Int, p0_w4: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
    @JvmStatic external fun call_callback_constructor_list_9de391e0ca3c5896(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_constructor_checked_68d8204b4d954b4b(p0_w2: Any?, p1_w1: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_set_new_84d26d4b7902b874(p0_callback: Any?, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_set_fire_c9c8320f5ea50f3d(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_set_spawn_b320f0fcb65848c6(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_job_join_c037874626d44faf(p0_w1: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_owned_new_1ad5877d98b31f02(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_owned_id_03a7cb65fe6b1309(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_owned_drops_d617a886f0baab03(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_owned_fire_5ac0f47b60fb0ab1(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_owned_borrow_37039e3928d2d226(p0_w1: Long, p1_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_transferred_id_eed01c8238b82f9a(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_transferred_drops_4ff92471b79e1575(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_transferred_fire_b5720cc21e6a881a(p0_callback: Any?, p1_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_projection_fire_fab0fb05d382a2b3(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_projected_id_79c245701232b63c(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_callback_preflight_a67d235ef587d6e2(p0_w1: Long, p1_callback: Any?, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_scalars_107e667aaf4d1540(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_texts_c776f9ba3bb1e5fa(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_optional_owned_c339b71b13e68762(p0_callback: Any?, p1_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_envelope_bede25767fde1a47(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_primitives_ff126f0005bfa2b5(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_callback_enums_b6723b6e70db554a(p0_callback: Any?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_collection_scalars_d9a5730159aaf1ce(p0_w1: Array<Any?>?, p1_w1: Double, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_collection_u128_04e9df44fc750d1f(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_collection_construct_all_2135887528f6e1b7(p0_w1: Array<Any?>?, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_make_items_c6725210ed9bf318(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_collection_bundle_f76991cc8c87924d(returns: Return_Array_Any___Long2_ErrorEmpty_9ac4dd96dd8aa1a2): Unit
    @JvmStatic external fun call_collection_optional_handles_d09dc27d024df24c(p0_w3: Array<Any?>?, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_collection_nested_handles_a1c0a814a0de57b0(p0_w1: Array<Any?>?, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_collection_numbers_58708da43cac559a(returns: Return_Array_Any___Long2_ErrorEmpty_9ac4dd96dd8aa1a2): Unit
    @JvmStatic external fun call_collection_resource_ids_25588809310a31d6(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_i64_90bde5aeb8c647ea(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_text_19d59571edb173e9(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_collection_bytes_f8c8c3aaf2aa2be0(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_collection_nested_4394d3d5a3b0ca8f(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_options_a293ccc475eb588e(p0_w1: Array<Any?>?, p1_w1: Short, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_optional_1f78a6a6c734d45b(p0_w3: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_optional_items_a463b73617be6af0(p0_w3: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_enum_a72cd6089a34be29(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    @JvmStatic external fun call_collection_probe_cf67d7568b2f663c(p0_w1: Array<Any?>?, p1_w1: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_borrow_0edafd22349f21ff(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_slice_00044f8d9833eed8(p0_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_collection_resource_new_7274517961a66222(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_handles_eceeff64e2628ef6(p0_w1: Array<Any?>?, p1_w1: Int, p2_w1: Int, ownership: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Array<Any?>?
    @JvmStatic external fun call_collection_alias_4512077905b51b90(p0_w1: Array<Any?>?, p1_w1: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_collection_result_be11b22c862519eb(p0_w1: Array<Any?>?, p1_w1: Boolean, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): ByteArray?
    @JvmStatic external fun call_collection_fail_e960b89bf99875af(returns: Return_Empty_ErrorArray_Any___Array_Any__2_ee1a9a01b387d74d): Unit
    @JvmStatic external fun call_collection_result_optional_2bf5c4a387d195e5(p0_w1: Array<Any?>?, p1_w1: Int, ownership: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Array<Any?>?
    @JvmStatic external fun call_storage_put_slice_0e1853c97ea92d0f(p0_w1: Long, p1_w1: Array<Any?>?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_storage_get_vec_1f0f8386b7d2b147(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
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
    @JvmStatic external fun call_result_owned_ed97ae30ee5fa2ec(p0_w1: Long, p1_w1: Int, p2_w1: Int, ownership: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Long
    @JvmStatic external fun call_result_other_bf6ac73e7d98d05c(p0_w1: Boolean, returns: Return_Empty_ErrorString1_6b62d91b16305083): Long
    @JvmStatic external fun call_storage_try_with_label_1db3fb1ae12e2bed(p0_w1: String?, returns: Return_Empty_ErrorString1_6b62d91b16305083): Long
    @JvmStatic external fun call_result_error_drops_37b19328e81c2636(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_enum_boundary_probe_602c56993c15d467(p0_w1: ByteArray?, p1_w5: Int, p2_w1: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_enum_payload_new_9a992a3f2e4e9d40(p0_w1: Double, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_payload_priority_1865922a1dc262d4(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
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
    @JvmStatic external fun call_niche_choice_id_ecb04a4cc855b867(p0_w3: Int, p0_w8: Int, p0_w10: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_nested_u32_2e0792d512f0f24f(p0_w5: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_triple_text_cae6b50b503944e6(p0_w3: Int, p0_w7: String?, returns: Return_Int_String2_ErrorEmpty_7e68b805da700de5): Unit
    @JvmStatic external fun call_option_triple_bytes_4f662b4b06709c31(p0_w3: Int, p0_w7: ByteArray?, returns: Return_Int_ByteArray2_ErrorEmpty_a2aad6159c8c4c14): Unit
    @JvmStatic external fun call_option_triple_i64_53dfd00875940117(p0_w5: Int, p0_w7: Long, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_triple_unit_c0762ed5037f126a(p0_w5: Int, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Int
    @JvmStatic external fun call_optional_text_input_len_5334cfb37cec76c0(p0_w4: String?, p0_w6: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_optional_bytes_input_len_7c4fd8ed8adbd4f7(p0_w4: ByteArray?, p0_w6: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_nested_niche_choice_id_3b8d8d5f9bee3571(p0_w5: Int, p0_w10: Int, p0_w12: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_nested_projection_views_d622fb44db024930(p0_w1: Boolean, p1_w1: Boolean, returns: Return_Int_Long_Long3_ErrorEmpty_2e9e07b44c221bd1): Unit
    @JvmStatic external fun call_option_ctor_new_3e8a52ff9fd43303(p0_w1: Int, p0_w3: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_ctor_id_5d244c0e4c436a1a(p0_w1: Int, p0_w4: Int, p0_w6: Long, p0_w8: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_nested_resource_b0bbbca600a8c975(p0_w1: Int, p0_w5: Long, ownership: BooleanArray, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_choice_id_4d12974690909134(p0_w3: Int, p0_w6: Long, p0_w8: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_str_echo_94daf2f0c810e1a7(p0_w3: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_option_slice_echo_d601bb0588850bdb(p0_w3: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_option_str_len_e1179a8d3f3ccc82(p0_w3: String?, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_slice_len_c45226bd8197ca90(p0_w3: ByteArray?, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_string_len_5af3d51e1e87974d(p0_w3: String?, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_i64_ref_fda4b6cd002e52b6(p0_w1: Int, p0_w3: Long, returns: Return_Int_Long2_ErrorEmpty_62eb913ef507b63c): Unit
    @JvmStatic external fun call_option_resource_pair_b954aa61fb636e9b(p0_w3: Long, p1_w3: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_resource_choice_id_7f8664c9adf8ecee(p0_w4: Long, p0_w6: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_optional_pair_new_a373cf8922fb383f(returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_option_projection_views_5f2b052a4aadad2a(p0_w1: Boolean, returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
    @JvmStatic external fun call_optional_bytes_echo_16b56811763936ed(p0_w1_f0_some: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_option_resource_mut_7bbfc5f1d8d2d0c3(p0_w3: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_projection_ref_9f590934a25a729a(p0_w1: Long, p1_w1: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
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
    @JvmStatic external fun call_option_resource_echo_92588c87a7537517(p0_w3: Long, p1_w1: Int, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_option_resource_id_3befc66c4766a89e(p0_w3: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
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
    @JvmStatic external fun call_archive_store_9db71c711db0bbcf(p0_w1: Long, p1_w1: Int, p1_w4: Long, p1_w6: Double, p1_w8: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_archive_latest_479f37d3e4b70496(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_storage_new_a15b46920214a065(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_storage_get_b7f3d4a82ad753f5(p0_w1: Long, returns: Return_Int_Long_Int5_ErrorEmpty_1e88956ed04b15f0): Unit
    @JvmStatic external fun call_storage_put_opt_38084ee844ee97a7(p0_w1: Long, p1_w1: Int, p1_w4: Long, p1_w6: Int, p1_w8: Double, p1_w10: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
    @JvmStatic external fun call_string_new_e13dcae0deb5ba72(p0_w1: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_string_len_0599f1bfcc1a4ae6(p0_w1: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_i64_ref_double_fe479ebaa0f31f86(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_text_echo_219b404084eaf51f(p0_w1: String?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_bytes_echo_a8ab3cbd8140d73a(p0_w1: ByteArray?, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): ByteArray?
    @JvmStatic external fun call_report_new_8c29e03007e0b826(p0_w1: Long, p1_w1: Double, p2_w1: String?, returns: Return_Long_Double_Long6_ErrorEmpty_a10857827755376a): Unit
    @JvmStatic external fun call_report_into_struct_47f91854fc7dffba(p0_w1: Int, p0_w3: Long, p0_w6: Long, p0_w8: Double, p0_w10: String?, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_payload_new_4db541865be00283(p0_w1: Long, p1_w1: Int, p2_w1: Double, p3_w1: Boolean, returns: Return_Long_Int_Double4_ErrorEmpty_f89d68e81a93e976): Unit
    @JvmStatic external fun call_storage_with_payload_b44f150bf309e573(p0_w2: Long, p0_w4: Int, p0_w6: Double, p0_w8: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_summary_new_c8a7979c4a30e563(p0_w1: Long, p1_w1: Double, returns: Return_Long_Double2_ErrorEmpty_a4ec6c8f44b5ff7d): Unit
    @JvmStatic external fun call_summary_total_raw_1331b588e0f6bb8b(p0_w1: Int, p0_w4: Long, p0_w6: Double, p0_w8: Long, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Double
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
    @JvmStatic external fun call_report_label_66cca46db0773f04(p0_w2: Int, p0_w5: Long, p0_w7: Double, p0_w9: Long, p0_w11: String?, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): String?
    @JvmStatic external fun call_report_relabel_f546697a9cc1715a(p0_w1: Long, p1_w1: String?, ownership: BooleanArray, returns: Return_Long_Double_Long6_ErrorEmpty_a10857827755376a): Unit
    @JvmStatic external fun call_storage_len_f00c3742e1e3063f(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_storage_contains_8645f2f664fd853c(p0_w1: Long, p1_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Boolean
    @JvmStatic external fun call_storage_total_len_1b16bba6fbe4c9d3(p0_w1: Long, p1_w1: Long, p2_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_storage_accumulate_lengths_422059e479322e31(p0_w1: Long, p1_w1: Long, p2_w1: Long, p3_w1: Long, p4_w1: Long, p5_w1: Long, p6_w1: Long, p7_w1: Long, p8_w3: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_storage_put_by_take_6be9a8d4b7a5eb93(p0_w1: Long, p1_w2: Long, p1_w4: Int, p1_w6: Double, p1_w8: Boolean, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Unit
    @JvmStatic external fun call_summary_merge_ac6a9002e0a4eec5(p0_w1: Int, p0_w4: Long, p0_w6: Double, p0_w8: Long, p1_w1: Int, p1_w4: Long, p1_w6: Double, p1_w8: Long, ownership: BooleanArray, returns: Return_Long_Double2_ErrorEmpty_a4ec6c8f44b5ff7d): Unit
    @JvmStatic external fun call_storage_summary_handle_7c23e0abb4322c29(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
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
    @JvmStatic external fun call_owned_resource_id_7691bb5261073433(p0_w1: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_owned_resource_drops_27038bacb19950fb(returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_resource_bundle_views_98686c0f53809c0c(returns: Return_String_Long_Long8_ErrorEmpty_74faddb97f994c93): Unit
    @JvmStatic external fun call_data_primitives_echo_c1f83db309ac8aab(p0_w1: Int, p0_w3_f0: Byte, p0_w3_f1: Short, p0_w3_f2: Int, p0_w3_f3: Long, p0_w3_f4: ByteArray?, p0_w3_f5: Long, p0_w3_f6: Int, p0_w3_f7: Int, p0_w3_f8: Long, p0_w3_f9: Long, p0_w3_f10: ByteArray?, p0_w3_f11: Long, p0_w3_f12: Float, p0_w3_f13: Double, p0_w3_f14: Boolean, p0_w3_f15: Int, returns: Return_Byte_Short_Int16_ErrorEmpty_4320f95dddb29024): Unit
    @JvmStatic external fun call_data_primitives_zero_52866d71f82311ff(returns: Return_Byte_Short_Int16_ErrorEmpty_4320f95dddb29024): Unit
    @JvmStatic external fun call_data_envelope_echo_3968cde0709118c6(p0_w1_f0: String?, p0_w1_f1_f0_f0: Long, p0_w1_f1_f0_f1: Long, p0_w1_f1_f1: ByteArray?, p0_w1_f2_f0: Int, p0_w1_f2_f1: Int, p0_w1_f2_f2: Long, p0_w1_f2_f3: Long, p0_w1_f2_f4_present: Int, p0_w1_f2_f4_some: Long, p0_w1_f3: Boolean, p0_w1_f4: Int, p0_w1_f5: ByteArray?, p0_w1_f6: ByteArray?, returns: Return_String_Long_Long14_ErrorEmpty_f6cccd4cd56d1dfa): Unit
    @JvmStatic external fun call_data_with_storage_e7325638663f3d59(p0_w1: Long, p1_w1_f0: String?, p1_w1_f1_f0_f0: Long, p1_w1_f1_f0_f1: Long, p1_w1_f1_f1: ByteArray?, p1_w1_f2_f0: Int, p1_w1_f2_f1: Int, p1_w1_f2_f2: Long, p1_w1_f2_f3: Long, p1_w1_f2_f4_present: Int, p1_w1_f2_f4_some: Long, p1_w1_f3: Boolean, p1_w1_f4: Int, p1_w1_f5: ByteArray?, p1_w1_f6: ByteArray?, ownership: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
    @JvmStatic external fun call_data_envelope_constructed_dab47fa5443a2b68(p0_w2_f0_f0: Long, p0_w2_f0_f1: Long, p0_w2_f1: ByteArray?, returns: Return_String_Long_Long14_ErrorEmpty_f6cccd4cd56d1dfa): Unit
    @JvmStatic external fun call_data_floats_echo_cfa4c7cc9e4653c8(p0_w1_f0: ByteArray?, p0_w1_f1: Float, p0_w1_f2: Double, returns: Return_ByteArray_Float_Double3_ErrorEmpty_796e683dacd5930a): Unit
    @JvmStatic external fun call_blob_value_selected_bb0c7d8f79d57c4b(p0_w1: Int, p0_w3_f0_f0: Long, p0_w3_f0_f1: Long, p0_w3_f1: ByteArray?, p0_w6: Long, p0_w8: ByteArray?, returns: Return_Long_Long_ByteArray3_ErrorEmpty_aa6ead24a4efe458): Unit
    @JvmStatic external fun call_blob_value_views_9983ceb667085a50(p0_w1_f0_f0: Long, p0_w1_f0_f1: Long, p0_w1_f1: ByteArray?, returns: Return_Long_Long_ByteArray9_ErrorEmpty_3b29111153baa628): Unit
    internal fun interface SequenceSink_default_bulk_array_fold_95a8899ff0517a5d { fun run(v0: Long, v1: Double) }
    @JvmStatic external fun call_default_bulk_array_fold_8b682a05955403df(callback: SequenceSink_default_bulk_array_fold_95a8899ff0517a5d, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_default_bulk_optional_fold_98317e3c22588d99 { fun run(v0: Long, v1: Double) }
    @JvmStatic external fun call_default_bulk_optional_fold_a9233ea21645798f(p0_w1: Long, callback: SequenceSink_default_bulk_optional_fold_98317e3c22588d99, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_callback_boundary_fold_fold_0165aba4bc8f4bac { fun run(v0: Long) }
    @JvmStatic external fun call_callback_boundary_fold_fold_f4fcc537824dc196(p0_callback: Any?, p1_w1: Long, callback: SequenceSink_callback_boundary_fold_fold_0165aba4bc8f4bac, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_scalars_fold_70028742f64f0191 { fun run(v0: Boolean, v1: Byte, v2: Short, v3: Int, v4: Long, v5: Float, v6: Double, v7: ByteArray?, v8: String?) }
    @JvmStatic external fun call_collection_scalars_fold_3627b263ec4e572b(p0_w1: Array<Any?>?, p1_w1: Double, callback: SequenceSink_collection_scalars_fold_70028742f64f0191, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_u128_fold_6f48cddbc993447a { fun run(v0: ByteArray?) }
    @JvmStatic external fun call_collection_u128_fold_5aa4076e5dce00fc(p0_w1: Array<Any?>?, callback: SequenceSink_collection_u128_fold_6f48cddbc993447a, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_optional_handles_fold_0322a075b01009ab { fun run(v0: Long) }
    @JvmStatic external fun call_collection_optional_handles_fold_f02c8c63b4591395(p0_w3: Array<Any?>?, ownership: BooleanArray, callback: SequenceSink_collection_optional_handles_fold_0322a075b01009ab, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_nested_handles_fold_0692fa392fcf7c3f { fun run(v0: Array<Any?>?) }
    @JvmStatic external fun call_collection_nested_handles_fold_b65f6cde439e67ad(p0_w1: Array<Any?>?, ownership: BooleanArray, callback: SequenceSink_collection_nested_handles_fold_0692fa392fcf7c3f, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_i64_fold_7481b0c247e22a81 { fun run(v0: Long) }
    @JvmStatic external fun call_collection_i64_fold_808c7d955e03904b(p0_w1: Array<Any?>?, callback: SequenceSink_collection_i64_fold_7481b0c247e22a81, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_text_fold_eee532ef372c2cc4 { fun run(v0: String?) }
    @JvmStatic external fun call_collection_text_fold_a7b24a9c45b3e186(p0_w1: Array<Any?>?, callback: SequenceSink_collection_text_fold_eee532ef372c2cc4, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_bytes_fold_6625bb31d7dc18eb { fun run(v0: ByteArray?) }
    @JvmStatic external fun call_collection_bytes_fold_9e9770856c6c4641(p0_w1: Array<Any?>?, callback: SequenceSink_collection_bytes_fold_6625bb31d7dc18eb, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_nested_fold_63e97a79272db78e { fun run(v0: Array<Any?>?) }
    @JvmStatic external fun call_collection_nested_fold_b79bf8a96a5df66c(p0_w1: Array<Any?>?, callback: SequenceSink_collection_nested_fold_63e97a79272db78e, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_options_fold_b6fa2828841c6e69 { fun run(v0: Int, v1: Long) }
    @JvmStatic external fun call_collection_options_fold_ded63bd11a5c595b(p0_w1: Array<Any?>?, p1_w1: Short, callback: SequenceSink_collection_options_fold_b6fa2828841c6e69, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_optional_fold_78209d4e9082c96a { fun run(v0: Long) }
    @JvmStatic external fun call_collection_optional_fold_1efcbb0a1a187e64(p0_w3: Array<Any?>?, callback: SequenceSink_collection_optional_fold_78209d4e9082c96a, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_optional_items_fold_0dd26be8936deb87 { fun run(v0: Int, v1: Long) }
    @JvmStatic external fun call_collection_optional_items_fold_912719d84a3df595(p0_w3: Array<Any?>?, callback: SequenceSink_collection_optional_items_fold_0dd26be8936deb87, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_enum_fold_c4e4e95a0f2dd06c { fun run(v0: Int) }
    @JvmStatic external fun call_collection_enum_fold_7a76112af7324a8e(p0_w1: Array<Any?>?, callback: SequenceSink_collection_enum_fold_c4e4e95a0f2dd06c, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_borrow_fold_b4856d1bed461622 { fun run(v0: Long) }
    @JvmStatic external fun call_collection_borrow_fold_22d09716b260bda8(p0_w1: Array<Any?>?, callback: SequenceSink_collection_borrow_fold_b4856d1bed461622, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_slice_fold_2b262532b9b8b02f { fun run(v0: Long) }
    @JvmStatic external fun call_collection_slice_fold_6f331d2d69f04ead(p0_w1: Array<Any?>?, callback: SequenceSink_collection_slice_fold_2b262532b9b8b02f, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_collection_handles_fold_94b94355358878bd { fun run(v0: Long) }
    @JvmStatic external fun call_collection_handles_fold_31a24f87629a2d5f(p0_w1: Array<Any?>?, p1_w1: Int, p2_w1: Int, ownership: BooleanArray, callback: SequenceSink_collection_handles_fold_94b94355358878bd, outputAccepted: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Array<Any?>?
    internal fun interface SequenceSink_collection_result_fold_8dc1da1d06d04f3a { fun run(v0: Long) }
    @JvmStatic external fun call_collection_result_fold_72fabc4d8e10f6c8(p0_w1: Array<Any?>?, p1_w1: Boolean, callback: SequenceSink_collection_result_fold_8dc1da1d06d04f3a, outputAccepted: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Array<Any?>?
    internal fun interface SequenceSink_collection_result_optional_fold_522f3e931c5b4b1c { fun run(v0: Long) }
    @JvmStatic external fun call_collection_result_optional_fold_a6c94691b7ce90de(p0_w1: Array<Any?>?, p1_w1: Int, ownership: BooleanArray, callback: SequenceSink_collection_result_optional_fold_522f3e931c5b4b1c, outputAccepted: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Array<Any?>?
    internal fun interface SequenceSink_sequence_late_drop_fold_c7e3bfa988c8796d { fun run(v0: Long) }
    @JvmStatic external fun call_sequence_late_drop_fold_7e360d29a6fc0d5f(callback: SequenceSink_sequence_late_drop_fold_c7e3bfa988c8796d, outputAccepted: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Array<Any?>?
    internal fun interface SequenceSink_sequence_scalars_fold_47a6ace8a9aad013 { fun run(v0: Boolean, v1: Byte, v2: Short, v3: Int, v4: Long, v5: Float, v6: Double, v7: ByteArray?, v8: String?) }
    @JvmStatic external fun call_sequence_scalars_fold_5108aab400f755b1(callback: SequenceSink_sequence_scalars_fold_47a6ace8a9aad013, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_sequence_parts_fold_34d4f8eed2926471 { fun run(v0: Long, v1: String?, v2: Long) }
    @JvmStatic external fun call_sequence_parts_fold_aeb653ed535ad4ab(p0_w1: Int, callback: SequenceSink_sequence_parts_fold_34d4f8eed2926471, outputAccepted: BooleanArray, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Array<Any?>?
    internal fun interface SequenceSink_storage_get_vec_fold_886389afd9ad2e6a { fun run(v0: Long, v1: Int, v2: Double, v3: Boolean) }
    @JvmStatic external fun call_storage_get_vec_fold_5d94b3f03ff97a04(p0_w1: Long, callback: SequenceSink_storage_get_vec_fold_886389afd9ad2e6a, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
    internal fun interface SequenceSink_api_record_rows_fold_5bd1116027b7610c { fun run(v0: Long, v1: String?) }
    @JvmStatic external fun call_api_record_rows_fold_847767faa33a5dce(callback: SequenceSink_api_record_rows_fold_5bd1116027b7610c, outputAccepted: BooleanArray, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Array<Any?>?
}
