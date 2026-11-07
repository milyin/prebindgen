package io.prebindgen.covertest.options

public data class OptionalBytes(
    public val `data`: kotlin.ByteArray?
) {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is OptionalBytes &&
            this.`data`.contentEquals(other.`data`))
    override fun hashCode(): kotlin.Int = this.`data`.contentHashCode()
    override fun toString(): kotlin.String = "OptionalBytes(data=${this.`data`.contentToString()})"
}
