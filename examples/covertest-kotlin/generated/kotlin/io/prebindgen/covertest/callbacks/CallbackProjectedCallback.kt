package io.prebindgen.covertest.callbacks

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface CallbackProjectedCallback {
    public fun run(`id`: Long, `arg0_1`: io.prebindgen.covertest.callbacks.CallbackProjected)
}
