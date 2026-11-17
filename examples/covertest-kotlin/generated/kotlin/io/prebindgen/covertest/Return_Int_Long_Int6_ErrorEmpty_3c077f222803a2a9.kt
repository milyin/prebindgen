package io.prebindgen.covertest

internal class Return_Int_Long_Int6_ErrorEmpty_3c077f222803a2a9 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Int = 0
    var s1: Long = 0L
    var s2: Int = 0
    var s3: Long = 0L
    var s4: Long = 0L
    var s5: ByteArray? = null
    fun success0(v0: Int, v1: Long, v2: Int, v3: Long, v4: Long, v5: ByteArray?) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; this.s4 = v4; this.s5 = v5; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s5 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Int_Long_Int6_ErrorEmpty_3c077f222803a2a9>()
        var depth = 0
        fun acquire(): Return_Int_Long_Int6_ErrorEmpty_3c077f222803a2a9 {
            val index = depth
            if (index == frames.size) frames.add(Return_Int_Long_Int6_ErrorEmpty_3c077f222803a2a9(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Int_Long_Int6_ErrorEmpty_3c077f222803a2a9 = local.get().acquire()
    }
}
