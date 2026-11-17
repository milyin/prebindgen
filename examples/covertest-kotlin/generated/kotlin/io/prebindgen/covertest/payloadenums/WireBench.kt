package io.prebindgen.covertest.payloadenums

public sealed interface WireBench {
public data class Count(
    public val v0: kotlin.ULong
) : io.prebindgen.covertest.payloadenums.WireBench
public data class Pair(
    public val left: kotlin.ULong,
    public val right: kotlin.ULong
) : io.prebindgen.covertest.payloadenums.WireBench
}
