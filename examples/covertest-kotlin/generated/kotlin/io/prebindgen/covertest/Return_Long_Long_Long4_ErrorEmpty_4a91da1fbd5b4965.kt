package io.prebindgen.covertest

internal class Return_Long_Long_Long4_ErrorEmpty_4a91da1fbd5b4965 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Long = 0L
    var s1: Long = 0L
    var s2: Long = 0L
    var s3: Long = 0L
    fun success0(v0: Long, v1: Long, v2: Long, v3: Long) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Long_Long_Long4_ErrorEmpty_4a91da1fbd5b4965>()
        var depth = 0
        fun acquire(): Return_Long_Long_Long4_ErrorEmpty_4a91da1fbd5b4965 {
            val index = depth
            if (index == frames.size) frames.add(Return_Long_Long_Long4_ErrorEmpty_4a91da1fbd5b4965(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Long_Long_Long4_ErrorEmpty_4a91da1fbd5b4965 = local.get().acquire()
    }
}
