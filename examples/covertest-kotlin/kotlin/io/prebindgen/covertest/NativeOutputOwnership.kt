package io.prebindgen.covertest

import io.prebindgen.covertest.callbacks.*
import io.prebindgen.covertest.cleanup.*

fun checkNativeOutputOwnership() {
    fun close(rows: List<CallbackRow>) {
        rows.forEach { it.first.close(); it.second.close() }
    }
    check(callbackRowResourceLive() == 0L)
    for (fail in listOf(-1L, 0L, 1L, 2L)) {
        var built = false
        val operations: List<() -> Unit> = listOf(
            { close(nativeBulkRows(fail)) },
            { nativeBulkNested(fail).forEach(::close) },
            { nativeBulkPair(fail) { first, second -> built = true; close(first); close(second) } },
        )
        for (operation in operations) {
            var failed = false
            try { operation() } catch (failure: IllegalArgumentException) {
                check(failure.message == "Rust operation panicked")
                failed = true
            }
            check(failed == (fail >= 0))
            check(callbackRowResourceLive() == 0L)
        }
        check(built == (fail < 0))
        var calls = 0
        nativeBulkCallback(fail, NativeBulkCallback { groups ->
            calls++
            check(groups.size == 3 && groups.all { it.size == 3 })
            // Scoped callback handles are reclaimed by generated Kotlin.
        })
        check(calls == if (fail < 0) 1 else 0)
        check(callbackRowResourceLive() == 0L)
    }
    // Foreign callback failure after successful native encoding must retain the
    // existing report-and-clear protocol and reclaim scoped Kotlin wrappers.
    var calls = 0
    nativeBulkCallback(-1, NativeBulkCallback {
        calls++
        throw IllegalStateException("injected callback failure after bulk delivery")
    })
    check(calls == 1 && callbackRowResourceLive() == 0L)
    for (mode in 10..12) check(CallbackFailureProbe.exercise(mode) == 1)
}

object NativeOutputFailureEntryPoint {
    @JvmStatic fun run(mode: Int) {
        var built = false
        var failed = false
        try {
            when (mode) {
                10 -> nativeBulkRows(-1).forEach { it.first.close(); it.second.close() }
                11 -> nativeBulkPair(-1) { first, second ->
                    built = true
                    (first + second).forEach { it.first.close(); it.second.close() }
                }
                12 -> nativeBulkCallback(-1, NativeBulkCallback { built = true })
            }
        } catch (_: OutOfMemoryError) { failed = true }
        check(failed == (mode != 12))
        check(!built && callbackRowResourceLive() == 0L)
    }
}
