package io.prebindgen.covertest

internal class Return_String_Long2_ErrorEmpty_b2c3b4b34db92d24 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: String? = null
    var s1: Long = 0L
    fun success0(v0: String?, v1: Long) { this.s0 = v0; this.s1 = v1; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s0 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_String_Long2_ErrorEmpty_b2c3b4b34db92d24>()
        var depth = 0
        fun acquire(): Return_String_Long2_ErrorEmpty_b2c3b4b34db92d24 {
            val index = depth
            if (index == frames.size) frames.add(Return_String_Long2_ErrorEmpty_b2c3b4b34db92d24(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_String_Long2_ErrorEmpty_b2c3b4b34db92d24 = local.get().acquire()
    }
}
