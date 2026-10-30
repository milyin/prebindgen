package bench.plain

/** Invoked by Rust, possibly from a native worker thread. Failure to invoke or convert the returned value raises a Rust panic on the invoking thread; an uncaught worker panic terminates that worker. */
public fun interface WideRowReturnCallback {
    public fun run(`arg0_0`: Long): bench.plain.WideRow
}
