package io.prebindgen.covertest

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface NativeBulkCallback {
    public fun run(arg0_0: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>>)
}
