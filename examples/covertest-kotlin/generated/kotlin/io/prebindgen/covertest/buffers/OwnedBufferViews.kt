package io.prebindgen.covertest.buffers

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface OwnedBufferViews {
    public fun run(`arg0_0`: String?, `arg0_1`: ByteArray?)
}
