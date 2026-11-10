package io.prebindgen.covertest

import io.prebindgen.covertest.inputcontracts.durationMillis
import java.time.Duration

fun checkInputContracts() {
    for (milliseconds in listOf(0L, 1L, -1L, 123456789L, Long.MIN_VALUE, Long.MAX_VALUE)) {
        check(durationMillis(Duration.ofMillis(milliseconds)) == milliseconds)
    }
    check(durationMillis(Duration.ofNanos(1_999_999)) == 1L)
    try {
        durationMillis(Duration.ofSeconds(Long.MAX_VALUE))
        error("expected Duration.toMillis overflow before JNI")
    } catch (_: ArithmeticException) {
        // The custom Kotlin projection keeps java.time.Duration semantics.
    }
}
