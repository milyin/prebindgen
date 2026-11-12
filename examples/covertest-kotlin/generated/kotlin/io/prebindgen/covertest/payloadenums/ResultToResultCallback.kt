package io.prebindgen.covertest.payloadenums

/** Invoked by Rust, possibly from a native worker thread. Failure to invoke or convert the returned value raises a Rust panic on the invoking thread; an uncaught worker panic terminates that worker. */
public fun interface ResultToResultCallback {
    public fun run(arg0_0: io.prebindgen.covertest.payloadenums.WireNotice): io.prebindgen.covertest.payloadenums.WireDecision
}
