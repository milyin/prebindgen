package io.prebindgen.covertest

/** Invoked by Rust, possibly from a native worker thread. Failure to invoke or convert the returned value raises a Rust panic on the invoking thread; an uncaught worker panic terminates that worker. Returned handles transfer ownership to Rust. */
public fun interface BulkHandlesCallback {
    public fun run(`arg0_0`: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>
}
