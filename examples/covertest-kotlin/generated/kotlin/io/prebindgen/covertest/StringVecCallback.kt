package io.prebindgen.covertest

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface StringVecCallback {
    public fun run(`arg0_0`: kotlin.collections.List<String>)
}
