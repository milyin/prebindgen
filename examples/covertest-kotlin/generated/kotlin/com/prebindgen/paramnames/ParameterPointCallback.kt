package com.prebindgen.paramnames

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface ParameterPointCallback {
    public fun run(arg0_0: com.prebindgen.paramnames.ParameterPoint)
}
