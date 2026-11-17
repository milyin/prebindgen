package io.prebindgen.covertest

internal class Return_Int_Long_Long4_ErrorEmpty_ae385d2af0c9940c private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Int = 0
    var s1: Long = 0L
    var s2: Long = 0L
    var s3: Double = 0.0
    fun success0(v0: Int, v1: Long, v2: Long, v3: Double) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Int_Long_Long4_ErrorEmpty_ae385d2af0c9940c>()
        var depth = 0
        fun acquire(): Return_Int_Long_Long4_ErrorEmpty_ae385d2af0c9940c {
            val index = depth
            if (index == frames.size) frames.add(Return_Int_Long_Long4_ErrorEmpty_ae385d2af0c9940c(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Int_Long_Long4_ErrorEmpty_ae385d2af0c9940c = local.get().acquire()
    }
}
