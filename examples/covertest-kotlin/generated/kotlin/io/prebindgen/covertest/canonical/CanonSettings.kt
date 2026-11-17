package io.prebindgen.covertest.canonical

public data class CanonSettings(
    public val delay: kotlin.Long,
    public val optional: kotlin.Long?,
    public val history: kotlin.collections.List<kotlin.Long>,
    public val label: kotlin.String
)
