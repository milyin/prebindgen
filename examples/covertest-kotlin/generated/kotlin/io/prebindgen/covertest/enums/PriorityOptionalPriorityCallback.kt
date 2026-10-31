package io.prebindgen.covertest.enums

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface PriorityOptionalPriorityCallback {
    public fun run(`arg0_0`: io.prebindgen.covertest.enums.Priority, `arg1_0`: io.prebindgen.covertest.enums.Priority?)
}
