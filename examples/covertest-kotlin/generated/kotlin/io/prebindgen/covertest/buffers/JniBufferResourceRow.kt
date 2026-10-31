package io.prebindgen.covertest.buffers

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class JniBufferResourceRow(
    public val `resource`: io.prebindgen.covertest.callbacks.CallbackOwned,
    public val `text`: kotlin.String,
    public val `bytes`: kotlin.ByteArray
) {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is JniBufferResourceRow &&
            this.`resource` == other.`resource` &&
            this.`text` == other.`text` &&
            this.`bytes`.contentEquals(other.`bytes`))
    override fun hashCode(): kotlin.Int = 31 * (31 * (this.`resource`.hashCode()) + this.`text`.hashCode()) + this.`bytes`.contentHashCode()
    override fun toString(): kotlin.String = "JniBufferResourceRow(resource=${this.`resource`}, text=${this.`text`}, bytes=${this.`bytes`.contentToString()})"
}
