package io.prebindgen.covertest

internal class Return_Empty_ErrorArray_Any___ByteArray2_5ead62d97d876746 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var d0: Array<Any?>? = null
    var d1: ByteArray? = null
    fun success0() { status = 1 }
    fun domainError0(v0: Array<Any?>?, v1: ByteArray?) { this.d0 = v0; this.d1 = v1; status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        d0 = null
        d1 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Empty_ErrorArray_Any___ByteArray2_5ead62d97d876746>()
        var depth = 0
        fun acquire(): Return_Empty_ErrorArray_Any___ByteArray2_5ead62d97d876746 {
            val index = depth
            if (index == frames.size) frames.add(Return_Empty_ErrorArray_Any___ByteArray2_5ead62d97d876746(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Empty_ErrorArray_Any___ByteArray2_5ead62d97d876746 = local.get().acquire()
    }
}
