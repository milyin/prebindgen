package io.prebindgen.covertest

internal class Return_ByteArray_Float_Double3_ErrorEmpty_796e683dacd5930a private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: ByteArray? = null
    var s1: Float = 0.0f
    var s2: Double = 0.0
    fun success0(v0: ByteArray?, v1: Float, v2: Double) { this.s0 = v0; this.s1 = v1; this.s2 = v2; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s0 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_ByteArray_Float_Double3_ErrorEmpty_796e683dacd5930a>()
        var depth = 0
        fun acquire(): Return_ByteArray_Float_Double3_ErrorEmpty_796e683dacd5930a {
            val index = depth
            if (index == frames.size) frames.add(Return_ByteArray_Float_Double3_ErrorEmpty_796e683dacd5930a(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_ByteArray_Float_Double3_ErrorEmpty_796e683dacd5930a = local.get().acquire()
    }
}
