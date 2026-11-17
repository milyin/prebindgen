package io.prebindgen.covertest

internal class Return_Empty_ErrorInt_Long_String3_b35d95e7a73a9cfe private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var d0: Int = 0
    var d1: Long = 0L
    var d2: String? = null
    fun success0() { status = 1 }
    fun domainError0(v0: Int, v1: Long, v2: String?) { this.d0 = v0; this.d1 = v1; this.d2 = v2; status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        d2 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Empty_ErrorInt_Long_String3_b35d95e7a73a9cfe>()
        var depth = 0
        fun acquire(): Return_Empty_ErrorInt_Long_String3_b35d95e7a73a9cfe {
            val index = depth
            if (index == frames.size) frames.add(Return_Empty_ErrorInt_Long_String3_b35d95e7a73a9cfe(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Empty_ErrorInt_Long_String3_b35d95e7a73a9cfe = local.get().acquire()
    }
}
