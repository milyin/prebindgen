package io.prebindgen.covertest.payloadenums

public sealed interface WireDecision {
public data class Ok(
    public val v0: kotlin.Long
) : io.prebindgen.covertest.payloadenums.WireDecision
public data class Err(
    public val v0: kotlin.String
) : io.prebindgen.covertest.payloadenums.WireDecision
}
