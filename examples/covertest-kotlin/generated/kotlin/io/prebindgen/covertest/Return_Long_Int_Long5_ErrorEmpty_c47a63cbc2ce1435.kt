package io.prebindgen.covertest

internal class Return_Long_Int_Long5_ErrorEmpty_c47a63cbc2ce1435 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Long = 0L
    var s1: Int = 0
    var s2: Long = 0L
    var s3: Array<Any?>? = null
    var s4: String? = null
    fun success0(v0: Long, v1: Int, v2: Long, v3: Array<Any?>?, v4: String?) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; this.s4 = v4; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s3 = null
        s4 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Long_Int_Long5_ErrorEmpty_c47a63cbc2ce1435>()
        var depth = 0
        fun acquire(): Return_Long_Int_Long5_ErrorEmpty_c47a63cbc2ce1435 {
            val index = depth
            if (index == frames.size) frames.add(Return_Long_Int_Long5_ErrorEmpty_c47a63cbc2ce1435(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Long_Int_Long5_ErrorEmpty_c47a63cbc2ce1435 = local.get().acquire()
    }
}
