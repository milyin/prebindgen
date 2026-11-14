package io.prebindgen.covertest

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface VoidCallback {
    public fun run()
}
