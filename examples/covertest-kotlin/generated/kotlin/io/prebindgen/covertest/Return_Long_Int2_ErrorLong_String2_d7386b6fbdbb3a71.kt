package io.prebindgen.covertest

internal class Return_Long_Int2_ErrorLong_String2_d7386b6fbdbb3a71 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Long = 0L
    var s1: Int = 0
    var d0: Long = 0L
    var d1: String? = null
    fun success0(v0: Long, v1: Int) { this.s0 = v0; this.s1 = v1; status = 1 }
    fun domainError0(v0: Long, v1: String?) { this.d0 = v0; this.d1 = v1; status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        d1 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Long_Int2_ErrorLong_String2_d7386b6fbdbb3a71>()
        var depth = 0
        fun acquire(): Return_Long_Int2_ErrorLong_String2_d7386b6fbdbb3a71 {
            val index = depth
            if (index == frames.size) frames.add(Return_Long_Int2_ErrorLong_String2_d7386b6fbdbb3a71(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Long_Int2_ErrorLong_String2_d7386b6fbdbb3a71 = local.get().acquire()
    }
}
