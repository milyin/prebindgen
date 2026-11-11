package io.prebindgen.covertest

import io.prebindgen.covertest.inputcontracts.durationMillis
import java.time.Duration
import io.prebindgen.covertest.scopedinputs.*

fun checkInputContracts() {
    checkScopedInputs()
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

private fun checkScopedInputs() {
    val read = scopedInputNew(10)
    val moved = scopedInputNew(20)
    val before = scopedInputDrops()
    check(scopedInputPair(read, moved) == 30L)
    check(scopedInputDrops() == before + 1uL)
    check(scopedInputId(read) == 10L)
    check(runCatching { scopedInputId(moved) }.isFailure)
    read.close()
    for (mode in 0L..3L) {
        val first = scopedInputNew(if (mode == 3L) 3 else 10)
        val second = scopedInputNew(20)
        val drops = scopedInputDrops()
        val result = runCatching { scopedInputFailure(first, mode, second) { error("unexpected domain error") } }
        if (mode == 0L) check(result.getOrThrow() == 30L) else check(result.isFailure)
        check(runCatching { scopedInputId(first) }.isFailure)
        if (mode == 1L) {
            check(scopedInputDrops() == drops + 1uL)
            check(scopedInputId(second) == 20L)
            second.close()
        } else {
            check(scopedInputDrops() == drops + 2uL)
            check(runCatching { scopedInputId(second) }.isFailure)
        }
    }
}
