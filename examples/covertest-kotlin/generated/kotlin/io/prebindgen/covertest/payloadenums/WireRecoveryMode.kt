package io.prebindgen.covertest.payloadenums

public sealed interface WireRecoveryMode {
public data class PeriodicQueries(
    public val v0: kotlin.ULong
) : io.prebindgen.covertest.payloadenums.WireRecoveryMode
public data object Heartbeat : io.prebindgen.covertest.payloadenums.WireRecoveryMode
}
