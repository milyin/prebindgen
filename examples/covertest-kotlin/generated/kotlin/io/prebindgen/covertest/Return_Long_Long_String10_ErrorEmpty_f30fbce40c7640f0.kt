package io.prebindgen.covertest

internal class Return_Long_Long_String10_ErrorEmpty_f30fbce40c7640f0 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Long = 0L
    var s1: Long = 0L
    var s2: String? = null
    var s3: Long = 0L
    var s4: String? = null
    var s5: Int = 0
    var s6: Long = 0L
    var s7: String? = null
    var s8: Array<Any?>? = null
    var s9: Long = 0L
    fun success0(v0: Long, v1: Long, v2: String?, v3: Long, v4: String?, v5: Int, v6: Long, v7: String?, v8: Array<Any?>?, v9: Long) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; this.s4 = v4; this.s5 = v5; this.s6 = v6; this.s7 = v7; this.s8 = v8; this.s9 = v9; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s2 = null
        s4 = null
        s7 = null
        s8 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Long_Long_String10_ErrorEmpty_f30fbce40c7640f0>()
        var depth = 0
        fun acquire(): Return_Long_Long_String10_ErrorEmpty_f30fbce40c7640f0 {
            val index = depth
            if (index == frames.size) frames.add(Return_Long_Long_String10_ErrorEmpty_f30fbce40c7640f0(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Long_Long_String10_ErrorEmpty_f30fbce40c7640f0 = local.get().acquire()
    }
}
