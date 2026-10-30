package io.prebindgen.covertest

internal class Return_Empty_ErrorByteArray_ByteArray2_8bb1103e963f51d1 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var d0: ByteArray? = null
    var d1: ByteArray? = null
    fun success0() { status = 1 }
    fun domainError0(v0: ByteArray?, v1: ByteArray?) { this.d0 = v0; this.d1 = v1; status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        d0 = null
        d1 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Empty_ErrorByteArray_ByteArray2_8bb1103e963f51d1>()
        var depth = 0
        fun acquire(): Return_Empty_ErrorByteArray_ByteArray2_8bb1103e963f51d1 {
            val index = depth
            if (index == frames.size) frames.add(Return_Empty_ErrorByteArray_ByteArray2_8bb1103e963f51d1(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Empty_ErrorByteArray_ByteArray2_8bb1103e963f51d1 = local.get().acquire()
    }
}
