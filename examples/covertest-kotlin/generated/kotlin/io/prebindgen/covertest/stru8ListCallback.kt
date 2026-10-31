package io.prebindgen.covertest

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface stru8ListCallback {
    public fun run(`arg0_0`: String, `arg1_0`: ByteArray)
}
