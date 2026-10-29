package io.prebindgen.covertest

internal class Return_Byte_Short_Int16_ErrorEmpty_4320f95dddb29024 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Byte = 0.toByte()
    var s1: Short = 0.toShort()
    var s2: Int = 0
    var s3: Long = 0L
    var s4: ByteArray? = null
    var s5: Long = 0L
    var s6: Int = 0
    var s7: Int = 0
    var s8: Long = 0L
    var s9: Long = 0L
    var s10: ByteArray? = null
    var s11: Long = 0L
    var s12: Float = 0.0f
    var s13: Double = 0.0
    var s14: Boolean = false
    var s15: Int = 0
    fun success0(v0: Byte, v1: Short, v2: Int, v3: Long, v4: ByteArray?, v5: Long, v6: Int, v7: Int, v8: Long, v9: Long, v10: ByteArray?, v11: Long, v12: Float, v13: Double, v14: Boolean, v15: Int) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; this.s4 = v4; this.s5 = v5; this.s6 = v6; this.s7 = v7; this.s8 = v8; this.s9 = v9; this.s10 = v10; this.s11 = v11; this.s12 = v12; this.s13 = v13; this.s14 = v14; this.s15 = v15; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s4 = null
        s10 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Byte_Short_Int16_ErrorEmpty_4320f95dddb29024>()
        var depth = 0
        fun acquire(): Return_Byte_Short_Int16_ErrorEmpty_4320f95dddb29024 {
            val index = depth
            if (index == frames.size) frames.add(Return_Byte_Short_Int16_ErrorEmpty_4320f95dddb29024(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Byte_Short_Int16_ErrorEmpty_4320f95dddb29024 = local.get().acquire()
    }
}
