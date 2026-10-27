package io.prebindgen.covertest.callbacks

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class CallbackNestedRow(
    public val `row`: io.prebindgen.covertest.callbacks.CallbackRow,
    public val `detail`: io.prebindgen.covertest.callbacks.CallbackRowDetail?
)
