package io.prebindgen.covertest.isolatederror

internal object JNINative {
    init { io.prebindgen.covertest.NativeLibrary.ensureLoaded() }
    @JvmStatic external fun call_duration_result_1a1b3c1552bf21b6(p0_w1: Long, returns: Return_Empty_ErrorLong1_5ff51b1265add9de): Long
}
