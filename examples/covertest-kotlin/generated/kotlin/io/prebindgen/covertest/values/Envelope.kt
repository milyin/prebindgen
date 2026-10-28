package io.prebindgen.covertest.values

public data class Envelope(
    public val `label`: kotlin.String,
    public val `blob`: io.prebindgen.covertest.model.BlobValue,
    public val `unsigned`: io.prebindgen.covertest.model.Unsigned,
    public val `enabled`: kotlin.Boolean,
    public val `code`: kotlin.Int,
    public val `signed`: java.math.BigInteger,
    public val `wide`: java.math.BigInteger
)
