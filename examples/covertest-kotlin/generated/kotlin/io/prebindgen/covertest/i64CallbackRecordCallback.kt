package io.prebindgen.covertest

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface i64CallbackRecordCallback {
    public fun run(`arg0_0`: Long, `arg1_0`: io.prebindgen.covertest.callbacks.CallbackRecord)
}
