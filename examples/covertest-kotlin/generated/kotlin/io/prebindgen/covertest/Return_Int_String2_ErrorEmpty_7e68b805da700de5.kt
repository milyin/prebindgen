package io.prebindgen.covertest

internal class Return_Int_String2_ErrorEmpty_7e68b805da700de5 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Int = 0
    var s1: String? = null
    fun success0(v0: Int, v1: String?) { this.s0 = v0; this.s1 = v1; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s1 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Int_String2_ErrorEmpty_7e68b805da700de5>()
        var depth = 0
        fun acquire(): Return_Int_String2_ErrorEmpty_7e68b805da700de5 {
            val index = depth
            if (index == frames.size) frames.add(Return_Int_String2_ErrorEmpty_7e68b805da700de5(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Int_String2_ErrorEmpty_7e68b805da700de5 = local.get().acquire()
    }
}
