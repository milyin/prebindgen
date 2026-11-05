package io.prebindgen.covertest.gc

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface GcOwnedCallback {
    public fun run(arg0_0: io.prebindgen.covertest.gc.GcResource)
}
