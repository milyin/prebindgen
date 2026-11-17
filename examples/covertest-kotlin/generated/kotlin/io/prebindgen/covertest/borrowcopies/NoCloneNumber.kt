package io.prebindgen.covertest.borrowcopies

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface NoCloneNumber {
    public fun run(arg0_0: Long)
}
