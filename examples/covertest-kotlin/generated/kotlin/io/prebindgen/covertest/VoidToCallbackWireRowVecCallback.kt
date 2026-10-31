package io.prebindgen.covertest

/** Invoked by Rust, possibly from a native worker thread. Failure to invoke or convert the returned value raises a Rust panic on the invoking thread; an uncaught worker panic terminates that worker. Returned handles transfer ownership to Rust. */
public fun interface VoidToCallbackWireRowVecCallback {
    public fun run(): kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackWireRow>
}
