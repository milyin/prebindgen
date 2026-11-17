package bench.bulk

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class HandleBatch(
    public val guard: bench.bulk.InputHandle,
    public val rows: kotlin.collections.List<bench.bulk.HandleRecord>
)
