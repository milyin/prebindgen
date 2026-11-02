package io.prebindgen.covertest

internal class Return_Long_Array_Any___Long3_ErrorEmpty_2b0b20baf8d8d1fd private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Long = 0L
    var s1: Array<Any?>? = null
    var s2: Long = 0L
    fun success0(v0: Long, v1: Array<Any?>?, v2: Long) { this.s0 = v0; this.s1 = v1; this.s2 = v2; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s1 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Long_Array_Any___Long3_ErrorEmpty_2b0b20baf8d8d1fd>()
        var depth = 0
        fun acquire(): Return_Long_Array_Any___Long3_ErrorEmpty_2b0b20baf8d8d1fd {
            val index = depth
            if (index == frames.size) frames.add(Return_Long_Array_Any___Long3_ErrorEmpty_2b0b20baf8d8d1fd(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Long_Array_Any___Long3_ErrorEmpty_2b0b20baf8d8d1fd = local.get().acquire()
    }
}
