package io.prebindgen.covertest

internal class Return_String_Long_Long14_ErrorEmpty_f6cccd4cd56d1dfa private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: String? = null
    var s1: Long = 0L
    var s2: Long = 0L
    var s3: ByteArray? = null
    var s4: Int = 0
    var s5: Int = 0
    var s6: Long = 0L
    var s7: Long = 0L
    var s8: Int = 0
    var s9: Long = 0L
    var s10: Boolean = false
    var s11: Int = 0
    var s12: ByteArray? = null
    var s13: ByteArray? = null
    fun success0(v0: String?, v1: Long, v2: Long, v3: ByteArray?, v4: Int, v5: Int, v6: Long, v7: Long, v8: Int, v9: Long, v10: Boolean, v11: Int, v12: ByteArray?, v13: ByteArray?) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; this.s4 = v4; this.s5 = v5; this.s6 = v6; this.s7 = v7; this.s8 = v8; this.s9 = v9; this.s10 = v10; this.s11 = v11; this.s12 = v12; this.s13 = v13; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s0 = null
        s3 = null
        s12 = null
        s13 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_String_Long_Long14_ErrorEmpty_f6cccd4cd56d1dfa>()
        var depth = 0
        fun acquire(): Return_String_Long_Long14_ErrorEmpty_f6cccd4cd56d1dfa {
            val index = depth
            if (index == frames.size) frames.add(Return_String_Long_Long14_ErrorEmpty_f6cccd4cd56d1dfa(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_String_Long_Long14_ErrorEmpty_f6cccd4cd56d1dfa = local.get().acquire()
    }
}
