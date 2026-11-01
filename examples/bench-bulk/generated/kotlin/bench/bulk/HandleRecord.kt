package bench.bulk

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class HandleRecord(
    public val `handle`: bench.bulk.InputHandle,
    public val `tag`: kotlin.Long
)
