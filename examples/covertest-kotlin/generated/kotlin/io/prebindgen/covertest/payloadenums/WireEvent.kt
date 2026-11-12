package io.prebindgen.covertest.payloadenums

public sealed interface WireEvent {
public data object Idle : io.prebindgen.covertest.payloadenums.WireEvent
public data class Count(
    public val v0: kotlin.ULong
) : io.prebindgen.covertest.payloadenums.WireEvent
public data class Packet(
    public val id: kotlin.ULong,
    public val bytes: kotlin.ByteArray
) : io.prebindgen.covertest.payloadenums.WireEvent {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is Packet &&
            this.id == other.id &&
            this.bytes.contentEquals(other.bytes))
    override fun hashCode(): kotlin.Int = 31 * (this.id.hashCode()) + this.bytes.contentHashCode()
    override fun toString(): kotlin.String = "Packet(id=${this.id}, bytes=${this.bytes.contentToString()})"
}
public data object EmptyTuple : io.prebindgen.covertest.payloadenums.WireEvent
public data object EmptyRecord : io.prebindgen.covertest.payloadenums.WireEvent
}
