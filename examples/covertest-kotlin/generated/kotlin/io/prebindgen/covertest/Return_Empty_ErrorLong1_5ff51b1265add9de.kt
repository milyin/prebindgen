package io.prebindgen.covertest

internal class Return_Empty_ErrorLong1_5ff51b1265add9de private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var d0: Long = 0L
    fun success0() { status = 1 }
    fun domainError0(v0: Long) { this.d0 = v0; status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Empty_ErrorLong1_5ff51b1265add9de>()
        var depth = 0
        fun acquire(): Return_Empty_ErrorLong1_5ff51b1265add9de {
            val index = depth
            if (index == frames.size) frames.add(Return_Empty_ErrorLong1_5ff51b1265add9de(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Empty_ErrorLong1_5ff51b1265add9de = local.get().acquire()
    }
}
