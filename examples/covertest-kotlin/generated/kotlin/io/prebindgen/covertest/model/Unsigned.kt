package io.prebindgen.covertest.model

public data class Unsigned(
    public val `byte`: kotlin.Int,
    public val `short`: kotlin.Int,
    public val `int`: kotlin.Long,
    public val `long`: kotlin.ULong,
    public val `maybe_long`: kotlin.ULong?
)
