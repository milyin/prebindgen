package bench.bulk

public data class Packet(
    public val `header`: bench.bulk.Row,
    public val `bytes`: kotlin.ByteArray
) {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is Packet &&
            this.`header` == other.`header` &&
            this.`bytes`.contentEquals(other.`bytes`))
    override fun hashCode(): kotlin.Int = 31 * (this.`header`.hashCode()) + this.`bytes`.contentHashCode()
    override fun toString(): kotlin.String = "Packet(header=${this.`header`}, bytes=${this.`bytes`.contentToString()})"
}
