package io.prebindgen.covertest

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface CanonMillisCallback {
    public fun run(arg0_0: Long)
}
