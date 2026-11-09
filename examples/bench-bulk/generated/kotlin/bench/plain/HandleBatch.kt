package bench.plain

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class HandleBatch(
    public val guard: bench.plain.InputHandle,
    public val rows: kotlin.collections.List<bench.plain.HandleRecord>
)
