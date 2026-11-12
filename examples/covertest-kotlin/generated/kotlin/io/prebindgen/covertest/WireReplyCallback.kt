package io.prebindgen.covertest

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface WireReplyCallback {
    public fun run(tag: Int, sampleResource: io.prebindgen.covertest.callbackreturns.CallbackReturnResource?, samplePayload: ByteArray?, errorPayload: ByteArray?, errorEncoding: Int, arg0_5: Long?)
}
