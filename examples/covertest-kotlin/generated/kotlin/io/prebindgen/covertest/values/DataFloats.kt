package io.prebindgen.covertest.values

public data class DataFloats(
    public val `bytes`: kotlin.ByteArray,
    public val `f`: kotlin.Float,
    public val `d`: kotlin.Double
) {
    override fun equals(other: kotlin.Any?): kotlin.Boolean =
        this === other || (other is DataFloats &&
            this.`bytes`.contentEquals(other.`bytes`) &&
            this.`f`.compareTo(other.`f`) == 0 &&
            this.`d`.compareTo(other.`d`) == 0)
    override fun hashCode(): kotlin.Int = 31 * (31 * (this.`bytes`.contentHashCode()) + this.`f`.hashCode()) + this.`d`.hashCode()
    override fun toString(): kotlin.String = "DataFloats(bytes=${this.`bytes`.contentToString()}, f=${this.`f`}, d=${this.`d`})"
}
