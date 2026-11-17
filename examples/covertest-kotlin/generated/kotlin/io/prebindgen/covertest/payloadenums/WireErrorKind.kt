package io.prebindgen.covertest.payloadenums

public sealed interface WireErrorKind {
public data class Code(
    public val v0: kotlin.Long
) : io.prebindgen.covertest.payloadenums.WireErrorKind
public data class Message(
    public val v0: kotlin.String
) : io.prebindgen.covertest.payloadenums.WireErrorKind
}
