package io.prebindgen.covertest.resources

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface ProjectionPairCallback {
    public fun run(`arg0_0`: io.prebindgen.covertest.resources.ProjectionPair)
}
