package io.prebindgen.covertest

internal class Return_Int_Int2_ErrorEmpty_1d2a9157b2d89875 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Int = 0
    var s1: Int = 0
    fun success0(v0: Int, v1: Int) { this.s0 = v0; this.s1 = v1; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Int_Int2_ErrorEmpty_1d2a9157b2d89875>()
        var depth = 0
        fun acquire(): Return_Int_Int2_ErrorEmpty_1d2a9157b2d89875 {
            val index = depth
            if (index == frames.size) frames.add(Return_Int_Int2_ErrorEmpty_1d2a9157b2d89875(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Int_Int2_ErrorEmpty_1d2a9157b2d89875 = local.get().acquire()
    }
}
