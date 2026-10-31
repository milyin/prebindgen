package bench.plain

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class HandleRecord(
    public val `handle`: bench.plain.InputHandle,
    public val `tag`: kotlin.Long
)
