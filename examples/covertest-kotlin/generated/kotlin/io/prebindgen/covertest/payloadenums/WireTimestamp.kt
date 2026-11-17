package io.prebindgen.covertest.payloadenums

public data class WireTimestamp(
    public val ntp64: kotlin.ULong,
    public val id: kotlin.ByteArray
) {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is WireTimestamp &&
            this.ntp64 == other.ntp64 &&
            this.id.contentEquals(other.id))
    override fun hashCode(): kotlin.Int = 31 * (this.ntp64.hashCode()) + this.id.contentHashCode()
    override fun toString(): kotlin.String = "WireTimestamp(ntp64=${this.ntp64}, id=${this.id.contentToString()})"
}
