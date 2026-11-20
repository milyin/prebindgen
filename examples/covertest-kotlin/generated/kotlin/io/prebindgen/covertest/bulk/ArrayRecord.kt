package io.prebindgen.covertest.bulk

public data class ArrayRecord(
    public val bytes: kotlin.ByteArray
) {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is ArrayRecord &&
            this.bytes.contentEquals(other.bytes))
    override fun hashCode(): kotlin.Int = this.bytes.contentHashCode()
    override fun toString(): kotlin.String = "ArrayRecord(bytes=${this.bytes.contentToString()})"
}
