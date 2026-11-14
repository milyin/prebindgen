package io.prebindgen.covertest

internal class Return_Int_Long_Long3_ErrorEmpty_2e9e07b44c221bd1 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Int = 0
    var s1: Long = 0L
    var s2: Long = 0L
    fun success0(v0: Int, v1: Long, v2: Long) { this.s0 = v0; this.s1 = v1; this.s2 = v2; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Int_Long_Long3_ErrorEmpty_2e9e07b44c221bd1>()
        var depth = 0
        fun acquire(): Return_Int_Long_Long3_ErrorEmpty_2e9e07b44c221bd1 {
            val index = depth
            if (index == frames.size) frames.add(Return_Int_Long_Long3_ErrorEmpty_2e9e07b44c221bd1(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Int_Long_Long3_ErrorEmpty_2e9e07b44c221bd1 = local.get().acquire()
    }
}
