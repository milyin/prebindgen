package io.prebindgen.covertest.payloadenums

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface WireEventCallback {
    public fun run(arg0_0: io.prebindgen.covertest.payloadenums.WireEvent)
}
