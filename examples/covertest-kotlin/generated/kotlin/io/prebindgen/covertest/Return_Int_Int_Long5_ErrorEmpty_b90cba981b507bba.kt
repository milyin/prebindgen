package io.prebindgen.covertest

internal class Return_Int_Int_Long5_ErrorEmpty_b90cba981b507bba private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Int = 0
    var s1: Int = 0
    var s2: Long = 0L
    var s3: String? = null
    var s4: String? = null
    fun success0(v0: Int, v1: Int, v2: Long, v3: String?, v4: String?) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; this.s4 = v4; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s3 = null
        s4 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Int_Int_Long5_ErrorEmpty_b90cba981b507bba>()
        var depth = 0
        fun acquire(): Return_Int_Int_Long5_ErrorEmpty_b90cba981b507bba {
            val index = depth
            if (index == frames.size) frames.add(Return_Int_Int_Long5_ErrorEmpty_b90cba981b507bba(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Int_Int_Long5_ErrorEmpty_b90cba981b507bba = local.get().acquire()
    }
}
