package io.prebindgen.covertest

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface DefaultBulkMixedCallback {
    public fun run(`arg0_0`: kotlin.collections.List<kotlin.collections.List<Long>>, `arg1_0`: kotlin.collections.List<kotlin.collections.List<Long>>)
}
