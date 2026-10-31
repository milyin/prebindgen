package io.prebindgen.covertest

internal class Return_String_Long_Long8_ErrorEmpty_74faddb97f994c93 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: String? = null
    var s1: Long = 0L
    var s2: Long = 0L
    var s3: Long = 0L
    var s4: ByteArray? = null
    var s5: Float = 0.0f
    var s6: Double = 0.0
    var s7: String? = null
    fun success0(v0: String?, v1: Long, v2: Long, v3: Long, v4: ByteArray?, v5: Float, v6: Double, v7: String?) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; this.s4 = v4; this.s5 = v5; this.s6 = v6; this.s7 = v7; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s0 = null
        s4 = null
        s7 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_String_Long_Long8_ErrorEmpty_74faddb97f994c93>()
        var depth = 0
        fun acquire(): Return_String_Long_Long8_ErrorEmpty_74faddb97f994c93 {
            val index = depth
            if (index == frames.size) frames.add(Return_String_Long_Long8_ErrorEmpty_74faddb97f994c93(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_String_Long_Long8_ErrorEmpty_74faddb97f994c93 = local.get().acquire()
    }
}
