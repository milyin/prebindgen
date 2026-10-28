package io.prebindgen.covertest.model

public data class BlobValue(
    public val `stamp`: io.prebindgen.covertest.model.Stamp,
    public val `id`: kotlin.ByteArray
) {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is BlobValue &&
            this.`stamp` == other.`stamp` &&
            this.`id`.contentEquals(other.`id`))
    override fun hashCode(): kotlin.Int = 31 * (this.`stamp`.hashCode()) + this.`id`.contentHashCode()
    override fun toString(): kotlin.String = "BlobValue(stamp=${this.`stamp`}, id=${this.`id`.contentToString()})"
}
