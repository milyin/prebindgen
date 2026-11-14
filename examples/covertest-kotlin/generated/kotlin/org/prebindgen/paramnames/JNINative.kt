package org.prebindgen.paramnames

internal object JNINative {
    init { io.prebindgen.covertest.NativeLibrary.ensureLoaded() }
    @JvmStatic external fun call_parameter_external_12d641fa6bd87fee(p0_w1: Long, p1_w1: Long, p2_callback: Any?, returns: Return_Long_Long2_ErrorEmpty_951bc171243c8a73): Unit
}
