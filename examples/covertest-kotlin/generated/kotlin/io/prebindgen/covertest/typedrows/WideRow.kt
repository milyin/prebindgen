package io.prebindgen.covertest.typedrows

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class WideRow(
    public val `left`: io.prebindgen.covertest.typedrows.Row64,
    public val `right`: io.prebindgen.covertest.typedrows.Row64,
    public val `owner`: io.prebindgen.covertest.callbacks.CallbackRowResource
)
