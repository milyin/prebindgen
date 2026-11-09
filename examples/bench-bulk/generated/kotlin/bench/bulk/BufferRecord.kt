package bench.bulk

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class BufferRecord(
    public val handle: bench.bulk.InputHandle,
    public val text: kotlin.String,
    public val bytes: kotlin.ByteArray
) {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is BufferRecord &&
            this.handle == other.handle &&
            this.text == other.text &&
            this.bytes.contentEquals(other.bytes))
    override fun hashCode(): kotlin.Int = 31 * (31 * (this.handle.hashCode()) + this.text.hashCode()) + this.bytes.contentHashCode()
    override fun toString(): kotlin.String = "BufferRecord(handle=${this.handle}, text=${this.text}, bytes=${this.bytes.contentToString()})"
}
