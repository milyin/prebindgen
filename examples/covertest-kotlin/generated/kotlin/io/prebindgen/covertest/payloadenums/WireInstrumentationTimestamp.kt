package io.prebindgen.covertest.payloadenums

public sealed interface WireInstrumentationTimestamp {
public data class Uhlc(
    public val v0: io.prebindgen.covertest.payloadenums.WireTimestamp
) : io.prebindgen.covertest.payloadenums.WireInstrumentationTimestamp
public data class Custom(
    public val v0: kotlin.ByteArray
) : io.prebindgen.covertest.payloadenums.WireInstrumentationTimestamp {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is Custom &&
            this.v0.contentEquals(other.v0))
    override fun hashCode(): kotlin.Int = this.v0.contentHashCode()
    override fun toString(): kotlin.String = "Custom(v0=${this.v0.contentToString()})"
}
}
