package io.prebindgen.covertest.buffers

public data class JniBufferRow(
    public val `text`: kotlin.String,
    public val `bytes`: kotlin.ByteArray
) {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is JniBufferRow &&
            this.`text` == other.`text` &&
            this.`bytes`.contentEquals(other.`bytes`))
    override fun hashCode(): kotlin.Int = 31 * (this.`text`.hashCode()) + this.`bytes`.contentHashCode()
    override fun toString(): kotlin.String = "JniBufferRow(text=${this.`text`}, bytes=${this.`bytes`.contentToString()})"
}
