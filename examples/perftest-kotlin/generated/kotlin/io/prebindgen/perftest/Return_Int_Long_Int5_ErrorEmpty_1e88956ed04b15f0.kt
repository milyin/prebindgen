package io.prebindgen.perftest

internal class Return_Int_Long_Int5_ErrorEmpty_1e88956ed04b15f0 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Int = 0
    var s1: Long = 0L
    var s2: Int = 0
    var s3: Double = 0.0
    var s4: Boolean = false
    fun success0(v0: Int, v1: Long, v2: Int, v3: Double, v4: Boolean) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; this.s4 = v4; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Int_Long_Int5_ErrorEmpty_1e88956ed04b15f0>()
        var depth = 0
        fun acquire(): Return_Int_Long_Int5_ErrorEmpty_1e88956ed04b15f0 {
            val index = depth
            if (index == frames.size) frames.add(Return_Int_Long_Int5_ErrorEmpty_1e88956ed04b15f0(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Int_Long_Int5_ErrorEmpty_1e88956ed04b15f0 = local.get().acquire()
    }
}
