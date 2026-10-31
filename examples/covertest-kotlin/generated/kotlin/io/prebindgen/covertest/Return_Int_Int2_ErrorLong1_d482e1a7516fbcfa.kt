package io.prebindgen.covertest

internal class Return_Int_Int2_ErrorLong1_d482e1a7516fbcfa private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Int = 0
    var s1: Int = 0
    var d0: Long = 0L
    fun success0(v0: Int, v1: Int) { this.s0 = v0; this.s1 = v1; status = 1 }
    fun domainError0(v0: Long) { this.d0 = v0; status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Int_Int2_ErrorLong1_d482e1a7516fbcfa>()
        var depth = 0
        fun acquire(): Return_Int_Int2_ErrorLong1_d482e1a7516fbcfa {
            val index = depth
            if (index == frames.size) frames.add(Return_Int_Int2_ErrorLong1_d482e1a7516fbcfa(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Int_Int2_ErrorLong1_d482e1a7516fbcfa = local.get().acquire()
    }
}
