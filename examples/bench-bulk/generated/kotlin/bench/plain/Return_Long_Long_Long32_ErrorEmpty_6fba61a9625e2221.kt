package bench.plain

internal class Return_Long_Long_Long32_ErrorEmpty_6fba61a9625e2221 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: Long = 0L
    var s1: Long = 0L
    var s2: Long = 0L
    var s3: Long = 0L
    var s4: Long = 0L
    var s5: Long = 0L
    var s6: Long = 0L
    var s7: Long = 0L
    var s8: Long = 0L
    var s9: Long = 0L
    var s10: Long = 0L
    var s11: Long = 0L
    var s12: Long = 0L
    var s13: Long = 0L
    var s14: Long = 0L
    var s15: Long = 0L
    var s16: Long = 0L
    var s17: Long = 0L
    var s18: Long = 0L
    var s19: Long = 0L
    var s20: Long = 0L
    var s21: Long = 0L
    var s22: Long = 0L
    var s23: Long = 0L
    var s24: Long = 0L
    var s25: Long = 0L
    var s26: Long = 0L
    var s27: Long = 0L
    var s28: Long = 0L
    var s29: Long = 0L
    var s30: Long = 0L
    var s31: Long = 0L
    fun success0(v0: Long, v1: Long, v2: Long, v3: Long, v4: Long, v5: Long, v6: Long, v7: Long, v8: Long, v9: Long, v10: Long, v11: Long, v12: Long, v13: Long, v14: Long, v15: Long, v16: Long, v17: Long, v18: Long, v19: Long, v20: Long, v21: Long, v22: Long, v23: Long, v24: Long, v25: Long, v26: Long, v27: Long, v28: Long, v29: Long, v30: Long, v31: Long) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; this.s4 = v4; this.s5 = v5; this.s6 = v6; this.s7 = v7; this.s8 = v8; this.s9 = v9; this.s10 = v10; this.s11 = v11; this.s12 = v12; this.s13 = v13; this.s14 = v14; this.s15 = v15; this.s16 = v16; this.s17 = v17; this.s18 = v18; this.s19 = v19; this.s20 = v20; this.s21 = v21; this.s22 = v22; this.s23 = v23; this.s24 = v24; this.s25 = v25; this.s26 = v26; this.s27 = v27; this.s28 = v28; this.s29 = v29; this.s30 = v30; this.s31 = v31; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_Long_Long_Long32_ErrorEmpty_6fba61a9625e2221>()
        var depth = 0
        fun acquire(): Return_Long_Long_Long32_ErrorEmpty_6fba61a9625e2221 {
            val index = depth
            if (index == frames.size) frames.add(Return_Long_Long_Long32_ErrorEmpty_6fba61a9625e2221(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_Long_Long_Long32_ErrorEmpty_6fba61a9625e2221 = local.get().acquire()
    }
}
