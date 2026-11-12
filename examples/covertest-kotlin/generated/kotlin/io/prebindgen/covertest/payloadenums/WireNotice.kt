package io.prebindgen.covertest.payloadenums

public sealed interface WireNotice {
public data class Ok(
    public val v0: kotlin.Unit
) : io.prebindgen.covertest.payloadenums.WireNotice
public data class Err(
    public val v0: kotlin.String
) : io.prebindgen.covertest.payloadenums.WireNotice
}
