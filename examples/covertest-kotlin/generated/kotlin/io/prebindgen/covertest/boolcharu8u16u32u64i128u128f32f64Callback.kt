package io.prebindgen.covertest

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface boolcharu8u16u32u64i128u128f32f64Callback {
    public fun run(`arg0_0`: Boolean, `arg1_0`: Int, `arg2_0`: Int, `arg3_0`: Int, `arg4_0`: Long, `arg5_0`: ULong, `arg6_0`: java.math.BigInteger, `arg7_0`: java.math.BigInteger, `arg8_0`: Float, `arg9_0`: Double)
}
