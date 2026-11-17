package io.prebindgen.covertest.nullability

public data class NullabilityValue(
    public val text: kotlin.String,
    public val bytes: kotlin.ByteArray,
    public val wide: java.math.BigInteger,
    public val choice: io.prebindgen.covertest.nullability.NullabilityChoice,
    public val handle_value: kotlin.Long
) {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is NullabilityValue &&
            this.text == other.text &&
            this.bytes.contentEquals(other.bytes) &&
            this.wide == other.wide &&
            this.choice == other.choice &&
            this.handle_value == other.handle_value)
    override fun hashCode(): kotlin.Int = 31 * (31 * (31 * (31 * (this.text.hashCode()) + this.bytes.contentHashCode()) + this.wide.hashCode()) + this.choice.hashCode()) + this.handle_value.hashCode()
    override fun toString(): kotlin.String = "NullabilityValue(text=${this.text}, bytes=${this.bytes.contentToString()}, wide=${this.wide}, choice=${this.choice}, handle_value=${this.handle_value})"
}
