package io.prebindgen.covertest.canonical

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class CanonOutput(
    public val token: io.prebindgen.covertest.canonical.CanonToken,
    public val percent: kotlin.Int
)
