package io.prebindgen.covertest.inputcontracts

internal object JNINative {
    init { io.prebindgen.covertest.NativeLibrary.ensureLoaded() }
    @JvmStatic external fun call_duration_millis_8c33b85e7452f8b5(millis_wire: Long, returns: Return_Empty_ErrorEmpty_a7ee757e32992e01): Long
}
