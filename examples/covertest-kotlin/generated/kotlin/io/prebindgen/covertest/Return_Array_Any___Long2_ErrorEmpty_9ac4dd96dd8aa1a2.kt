package io.prebindgen.covertest

internal class Return_Array_Any___Long2_ErrorEmpty_9ac4dd96dd8aa1a2 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Array<Any?>? = null
    var s1: Long = 0L
    fun success0(v0: Array<Any?>?, v1: Long) { this.s0 = v0; this.s1 = v1; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s0 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Array_Any___Long2_ErrorEmpty_9ac4dd96dd8aa1a2>()
        var depth = 0
        fun acquire(): Return_Array_Any___Long2_ErrorEmpty_9ac4dd96dd8aa1a2 {
            val index = depth
            if (index == frames.size) frames.add(Return_Array_Any___Long2_ErrorEmpty_9ac4dd96dd8aa1a2(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Array_Any___Long2_ErrorEmpty_9ac4dd96dd8aa1a2 = local.get().acquire()
    }
}
