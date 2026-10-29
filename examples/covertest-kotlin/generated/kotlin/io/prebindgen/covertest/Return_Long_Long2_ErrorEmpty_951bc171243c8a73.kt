package io.prebindgen.covertest

internal class Return_Long_Long2_ErrorEmpty_951bc171243c8a73 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Long = 0L
    var s1: Long = 0L
    fun success0(v0: Long, v1: Long) { this.s0 = v0; this.s1 = v1; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Long_Long2_ErrorEmpty_951bc171243c8a73>()
        var depth = 0
        fun acquire(): Return_Long_Long2_ErrorEmpty_951bc171243c8a73 {
            val index = depth
            if (index == frames.size) frames.add(Return_Long_Long2_ErrorEmpty_951bc171243c8a73(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Long_Long2_ErrorEmpty_951bc171243c8a73 = local.get().acquire()
    }
}
