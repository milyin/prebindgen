package io.prebindgen.covertest

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface i64i64Callback {
    public fun run(`arg0_0`: Long, `arg1_0`: Long)
}
