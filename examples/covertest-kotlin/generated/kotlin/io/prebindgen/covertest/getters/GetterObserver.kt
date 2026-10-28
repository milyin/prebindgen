package io.prebindgen.covertest.getters

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface GetterObserver {
    public fun run(`arg0_0`: Long, `arg0_1`: String, `arg0_2`: Long?, `arg0_3`: String?, `arg0_4`: io.prebindgen.covertest.getters.GetterDetails?, `arg0_5`: kotlin.collections.List<io.prebindgen.covertest.getters.GetterDetails>, `arg0_6`: io.prebindgen.covertest.getters.GetterLeaf)
}
