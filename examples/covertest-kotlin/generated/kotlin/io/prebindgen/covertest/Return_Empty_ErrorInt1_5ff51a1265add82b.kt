package io.prebindgen.covertest

internal class Return_Empty_ErrorInt1_5ff51a1265add82b private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var d0: Int = 0
    fun success0() { status = 1 }
    fun domainError0(v0: Int) { this.d0 = v0; status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Empty_ErrorInt1_5ff51a1265add82b>()
        var depth = 0
        fun acquire(): Return_Empty_ErrorInt1_5ff51a1265add82b {
            val index = depth
            if (index == frames.size) frames.add(Return_Empty_ErrorInt1_5ff51a1265add82b(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Empty_ErrorInt1_5ff51a1265add82b = local.get().acquire()
    }
}
