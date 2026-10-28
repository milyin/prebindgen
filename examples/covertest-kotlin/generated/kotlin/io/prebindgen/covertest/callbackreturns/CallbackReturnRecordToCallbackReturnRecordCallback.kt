package io.prebindgen.covertest.callbackreturns

/** Invoked by Rust, possibly from a native worker thread. Failure to invoke or convert the returned value raises a Rust panic on the invoking thread; an uncaught worker panic terminates that worker. */
public fun interface CallbackReturnRecordToCallbackReturnRecordCallback {
    public fun run(`arg0_0`: io.prebindgen.covertest.callbackreturns.CallbackReturnRecord): io.prebindgen.covertest.callbackreturns.CallbackReturnRecord
}
