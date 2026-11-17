package io.prebindgen.covertest

internal class Return_Long_Long_Array_Any__6_ErrorEmpty_5b53ecd9dac487e2 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Long = 0L
    var s1: Long = 0L
    var s2: Array<Any?>? = null
    var s3: Int = 0
    var s4: Array<Any?>? = null
    var s5: Long = 0L
    fun success0(v0: Long, v1: Long, v2: Array<Any?>?, v3: Int, v4: Array<Any?>?, v5: Long) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; this.s4 = v4; this.s5 = v5; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s2 = null
        s4 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Long_Long_Array_Any__6_ErrorEmpty_5b53ecd9dac487e2>()
        var depth = 0
        fun acquire(): Return_Long_Long_Array_Any__6_ErrorEmpty_5b53ecd9dac487e2 {
            val index = depth
            if (index == frames.size) frames.add(Return_Long_Long_Array_Any__6_ErrorEmpty_5b53ecd9dac487e2(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Long_Long_Array_Any__6_ErrorEmpty_5b53ecd9dac487e2 = local.get().acquire()
    }
}
