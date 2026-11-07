package io.prebindgen.covertest.rawinputs

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class BulkReplayRow(
    public val first: io.prebindgen.covertest.rawinputs.BulkHandle,
    public val text: kotlin.String?,
    public val bytes: kotlin.ByteArray?,
    public val kind: io.prebindgen.covertest.rawinputs.BulkKind?,
    public val last: io.prebindgen.covertest.rawinputs.BulkHandle
) {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is BulkReplayRow &&
            this.first == other.first &&
            this.text == other.text &&
            this.bytes.contentEquals(other.bytes) &&
            this.kind == other.kind &&
            this.last == other.last)
    override fun hashCode(): kotlin.Int = 31 * (31 * (31 * (31 * (this.first.hashCode()) + this.text.hashCode()) + this.bytes.contentHashCode()) + this.kind.hashCode()) + this.last.hashCode()
    override fun toString(): kotlin.String = "BulkReplayRow(first=${this.first}, text=${this.text}, bytes=${this.bytes.contentToString()}, kind=${this.kind}, last=${this.last})"
}
