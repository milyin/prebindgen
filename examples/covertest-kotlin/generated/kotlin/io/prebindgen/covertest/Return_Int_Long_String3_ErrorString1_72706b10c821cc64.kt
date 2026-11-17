package io.prebindgen.covertest

internal class Return_Int_Long_String3_ErrorString1_72706b10c821cc64 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Int = 0
    var s1: Long = 0L
    var s2: String? = null
    var d0: String? = null
    fun success0(v0: Int, v1: Long, v2: String?) { this.s0 = v0; this.s1 = v1; this.s2 = v2; status = 1 }
    fun domainError0(v0: String?) { this.d0 = v0; status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s2 = null
        d0 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Int_Long_String3_ErrorString1_72706b10c821cc64>()
        var depth = 0
        fun acquire(): Return_Int_Long_String3_ErrorString1_72706b10c821cc64 {
            val index = depth
            if (index == frames.size) frames.add(Return_Int_Long_String3_ErrorString1_72706b10c821cc64(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Int_Long_String3_ErrorString1_72706b10c821cc64 = local.get().acquire()
    }
}
