package io.prebindgen.covertest

import io.prebindgen.covertest.outputcontracts.*
import java.time.Duration

fun checkOutputContracts() {
    for (millis in listOf(0L, -1L, 123456L, Long.MIN_VALUE, Long.MAX_VALUE)) {
        check(durationEcho(millis) == Duration.ofMillis(millis))
        check(durationPacket(millis) == Duration.ofMillis(millis))
    }
    check(durationPair { a, b -> listOf(a, b) } == listOf(Duration.ofMillis(17), Duration.ofMillis(-29)))
    val expectedSeries = listOf(3L, -5L, 8L).map { Duration.ofMillis(it) }
    check(durationSeries() == expectedSeries)
    check(durationSeriesFold(emptyList<Duration>()) { accumulated, value -> accumulated + value } == expectedSeries)
    check(durationOptional(true) == Duration.ofMillis(37))
    check(durationOptional(false) == null)
    check(durationResult(19, onError = { failure -> error("unexpected $failure") }) == Duration.ofMillis(19))
    check(durationResult(-7, onError = { error ->
        check(error == Duration.ofMillis(7))
        Duration.ofMillis(123)
    }) == Duration.ofMillis(123))
    check(plainResult(-7, onError = { error -> error + 1 }) == 8L)
    check(durationCallback(callback = { value ->
        check(value == Duration.ofMillis(43))
        value.plusMillis(5)
    }) == 48L)
    check(plainCallback(callback = { value -> value + 2 }) == 45L)
}
