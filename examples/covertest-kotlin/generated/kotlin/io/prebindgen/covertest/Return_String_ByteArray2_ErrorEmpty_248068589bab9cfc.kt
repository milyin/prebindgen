package io.prebindgen.covertest

internal class Return_String_ByteArray2_ErrorEmpty_248068589bab9cfc private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: String? = null
    var s1: ByteArray? = null
    fun success0(v0: String?, v1: ByteArray?) { this.s0 = v0; this.s1 = v1; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s0 = null
        s1 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_String_ByteArray2_ErrorEmpty_248068589bab9cfc>()
        var depth = 0
        fun acquire(): Return_String_ByteArray2_ErrorEmpty_248068589bab9cfc {
            val index = depth
            if (index == frames.size) frames.add(Return_String_ByteArray2_ErrorEmpty_248068589bab9cfc(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_String_ByteArray2_ErrorEmpty_248068589bab9cfc = local.get().acquire()
    }
}
