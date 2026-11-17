package io.prebindgen.covertest.payloadenums

public sealed interface WireNestedDecision {
public data class Ok(
    public val v0: io.prebindgen.covertest.payloadenums.WireDecision
) : io.prebindgen.covertest.payloadenums.WireNestedDecision
public data class Err(
    public val v0: kotlin.String
) : io.prebindgen.covertest.payloadenums.WireNestedDecision
}
