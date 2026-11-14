package io.prebindgen.covertest.payloadenums

public data class WireTimestampStackRecord(
    public val point: io.prebindgen.covertest.payloadenums.WireInterceptionPoint,
    public val timestamp: io.prebindgen.covertest.payloadenums.WireInstrumentationTimestamp
)
