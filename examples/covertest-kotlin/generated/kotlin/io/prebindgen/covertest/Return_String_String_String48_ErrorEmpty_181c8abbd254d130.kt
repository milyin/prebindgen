package io.prebindgen.covertest

internal class Return_String_String_String48_ErrorEmpty_181c8abbd254d130 private constructor(private val pool: Pool) {
    var status: Int = 0
    var message: String? = null
    var discardSuccess: Boolean = false
    var s0: String? = null
    var s1: String? = null
    var s2: String? = null
    var s3: String? = null
    var s4: String? = null
    var s5: String? = null
    var s6: String? = null
    var s7: String? = null
    var s8: String? = null
    var s9: String? = null
    var s10: String? = null
    var s11: String? = null
    var s12: String? = null
    var s13: String? = null
    var s14: String? = null
    var s15: String? = null
    var s16: String? = null
    var s17: String? = null
    var s18: String? = null
    var s19: String? = null
    var s20: String? = null
    var s21: String? = null
    var s22: String? = null
    var s23: String? = null
    var s24: String? = null
    var s25: String? = null
    var s26: String? = null
    var s27: String? = null
    var s28: String? = null
    var s29: String? = null
    var s30: String? = null
    var s31: String? = null
    var s32: String? = null
    var s33: String? = null
    var s34: String? = null
    var s35: String? = null
    var s36: String? = null
    var s37: String? = null
    var s38: String? = null
    var s39: String? = null
    var s40: String? = null
    var s41: String? = null
    var s42: String? = null
    var s43: String? = null
    var s44: String? = null
    var s45: String? = null
    var s46: String? = null
    var s47: String? = null
    fun success0(v0: String?, v1: String?, v2: String?, v3: String?, v4: String?, v5: String?, v6: String?, v7: String?, v8: String?, v9: String?, v10: String?, v11: String?, v12: String?, v13: String?, v14: String?, v15: String?, v16: String?, v17: String?, v18: String?, v19: String?, v20: String?, v21: String?, v22: String?, v23: String?, v24: String?, v25: String?, v26: String?, v27: String?, v28: String?, v29: String?, v30: String?, v31: String?, v32: String?, v33: String?, v34: String?, v35: String?, v36: String?, v37: String?, v38: String?, v39: String?, v40: String?, v41: String?, v42: String?, v43: String?, v44: String?, v45: String?, v46: String?, v47: String?) { this.s0 = v0; this.s1 = v1; this.s2 = v2; this.s3 = v3; this.s4 = v4; this.s5 = v5; this.s6 = v6; this.s7 = v7; this.s8 = v8; this.s9 = v9; this.s10 = v10; this.s11 = v11; this.s12 = v12; this.s13 = v13; this.s14 = v14; this.s15 = v15; this.s16 = v16; this.s17 = v17; this.s18 = v18; this.s19 = v19; this.s20 = v20; this.s21 = v21; this.s22 = v22; this.s23 = v23; this.s24 = v24; this.s25 = v25; this.s26 = v26; this.s27 = v27; this.s28 = v28; this.s29 = v29; this.s30 = v30; this.s31 = v31; this.s32 = v32; this.s33 = v33; this.s34 = v34; this.s35 = v35; this.s36 = v36; this.s37 = v37; this.s38 = v38; this.s39 = v39; this.s40 = v40; this.s41 = v41; this.s42 = v42; this.s43 = v43; this.s44 = v44; this.s45 = v45; this.s46 = v46; this.s47 = v47; status = 1 }
    fun domainError0() { status = 3 }
    fun bindingError(message: String?, committed: Boolean) { this.message = message; discardSuccess = committed; status = 2 }
    fun release() {
        s0 = null
        s1 = null
        s2 = null
        s3 = null
        s4 = null
        s5 = null
        s6 = null
        s7 = null
        s8 = null
        s9 = null
        s10 = null
        s11 = null
        s12 = null
        s13 = null
        s14 = null
        s15 = null
        s16 = null
        s17 = null
        s18 = null
        s19 = null
        s20 = null
        s21 = null
        s22 = null
        s23 = null
        s24 = null
        s25 = null
        s26 = null
        s27 = null
        s28 = null
        s29 = null
        s30 = null
        s31 = null
        s32 = null
        s33 = null
        s34 = null
        s35 = null
        s36 = null
        s37 = null
        s38 = null
        s39 = null
        s40 = null
        s41 = null
        s42 = null
        s43 = null
        s44 = null
        s45 = null
        s46 = null
        s47 = null
        status = 0; message = null; discardSuccess = false
        pool.depth--
    }
    private class Pool {
        val frames = java.util.ArrayList<Return_String_String_String48_ErrorEmpty_181c8abbd254d130>()
        var depth = 0
        fun acquire(): Return_String_String_String48_ErrorEmpty_181c8abbd254d130 {
            val index = depth
            if (index == frames.size) frames.add(Return_String_String_String48_ErrorEmpty_181c8abbd254d130(this))
            depth = index + 1
            return frames[index]
        }
    }
    companion object {
        private val local = ThreadLocal.withInitial { Pool() }
        fun acquire(): Return_String_String_String48_ErrorEmpty_181c8abbd254d130 = local.get().acquire()
    }
}
