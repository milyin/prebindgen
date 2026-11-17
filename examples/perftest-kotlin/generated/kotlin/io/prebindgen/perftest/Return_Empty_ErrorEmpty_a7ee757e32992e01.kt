package io.prebindgen.perftest

internal class Return_Empty_ErrorEmpty_a7ee757e32992e01 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    fun success0() { status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Empty_ErrorEmpty_a7ee757e32992e01>()
        var depth = 0
        fun acquire(): Return_Empty_ErrorEmpty_a7ee757e32992e01 {
            val index = depth
            if (index == frames.size) frames.add(Return_Empty_ErrorEmpty_a7ee757e32992e01(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Empty_ErrorEmpty_a7ee757e32992e01 = local.get().acquire()
    }
}
