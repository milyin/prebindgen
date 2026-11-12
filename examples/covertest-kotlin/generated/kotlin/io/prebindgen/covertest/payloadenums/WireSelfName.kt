package io.prebindgen.covertest.payloadenums

public sealed interface WireSelfName {
public data class WireSelfName(
    public val v0: kotlin.ULong
) : io.prebindgen.covertest.payloadenums.WireSelfName
public data object Other : io.prebindgen.covertest.payloadenums.WireSelfName
}
