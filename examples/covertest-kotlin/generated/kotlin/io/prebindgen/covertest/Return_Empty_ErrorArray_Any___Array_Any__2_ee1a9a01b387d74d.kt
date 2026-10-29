package io.prebindgen.covertest

internal class Return_Empty_ErrorArray_Any___Array_Any__2_ee1a9a01b387d74d private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var d0: Array<Any?>? = null
    var d1: Array<Any?>? = null
    fun success0() { status = 1 }
    fun domainError0(v0: Array<Any?>?, v1: Array<Any?>?) { this.d0 = v0; this.d1 = v1; status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        d0 = null
        d1 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Empty_ErrorArray_Any___Array_Any__2_ee1a9a01b387d74d>()
        var depth = 0
        fun acquire(): Return_Empty_ErrorArray_Any___Array_Any__2_ee1a9a01b387d74d {
            val index = depth
            if (index == frames.size) frames.add(Return_Empty_ErrorArray_Any___Array_Any__2_ee1a9a01b387d74d(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Empty_ErrorArray_Any___Array_Any__2_ee1a9a01b387d74d = local.get().acquire()
    }
}
