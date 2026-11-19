package io.prebindgen.perftest

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface PayloadCallback {
    public fun run(arg0_0: Long, arg0_1: Int, arg0_2: Double, arg0_3: Boolean)
}
