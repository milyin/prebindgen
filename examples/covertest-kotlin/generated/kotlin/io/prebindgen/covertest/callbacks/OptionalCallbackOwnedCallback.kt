package io.prebindgen.covertest.callbacks

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface OptionalCallbackOwnedCallback {
    public fun run(`arg0_0`: io.prebindgen.covertest.callbacks.CallbackOwned?)
}
