package io.prebindgen.covertest

import io.prebindgen.covertest.callbacks.*
import io.prebindgen.covertest.nativeoutputs.*

private fun unexpectedOutputError(code: Long): Nothing = error("unexpected source error: $code")

fun checkJniOutputOwnership() {
    fun close(rows: List<CallbackRow>) {
        rows.forEach { it.first.close(); it.second.close() }
    }
    for (fail in listOf(-1L, 0L, 1L, 2L)) {
        val operations: List<() -> Unit> = listOf(
            { close(jniOutputRows(fail, onError = ResultFailureHandler(::unexpectedOutputError))) },
            { jniOutputNested(fail, onError = ResultFailureHandler(::unexpectedOutputError)).forEach(::close) },
            { jniOutputPair(fail, onError = ResultFailureHandler(::unexpectedOutputError)) { first, second -> close(first); close(second) } },
            { jniOutputFold(fail, Unit, onError = ResultFailureHandler(::unexpectedOutputError)) { _, row -> close(listOf(row)) } },
            { jniOutputOptional(fail).filterNotNull().forEach(::close) },
            { jniOutputMixed(fail).forEach(::close) },
        )
        for (operation in operations) {
            var failed = false
            try { operation() } catch (failure: IllegalArgumentException) {
                check(failure.message == "Rust operation panicked")
                failed = true
            }
            check(failed == (fail >= 0))
            check(callbackRowResourceLive() == 0L) { "JNI output leaked after fail=$fail" }
        }
    }
    for (mode in 13..18) check(CallbackFailureProbe.exercise(mode) == 1)
    val drops = callbackRowResourceDrops()
    try { JniOutputRuntimeProbe.rollback(); error("rollback did not preserve exception") }
    catch (failure: java.io.IOException) { check(failure.message == "original output rollback") }
    check(callbackRowResourceLive() == 0L && callbackRowResourceDrops() == drops + 2L)
}

object JniOutputRuntimeProbe {
    @JvmStatic external fun rollback()
}

object JniOutputFailureEntryPoint {
    @JvmStatic fun run(mode: Int) {
        var failed = false
        try {
            when (mode) {
                13 -> jniOutputRows(-1, onError = ResultFailureHandler(::unexpectedOutputError)).forEach { it.first.close(); it.second.close() }
                14 -> jniOutputNested(-1, onError = ResultFailureHandler(::unexpectedOutputError)).flatten().forEach { it.first.close(); it.second.close() }
                15 -> jniOutputPair(-1, onError = ResultFailureHandler(::unexpectedOutputError)) { first, second -> (first + second).forEach { it.first.close(); it.second.close() } }
                16 -> jniOutputFold(-1, Unit, onError = ResultFailureHandler(::unexpectedOutputError)) { _, row -> row.first.close(); row.second.close() }
                17 -> jniOutputOptional(-1).filterNotNull().flatten().forEach { it.first.close(); it.second.close() }
                18 -> jniOutputMixed(-1).flatten().forEach { it.first.close(); it.second.close() }
            }
        } catch (failure: OutOfMemoryError) {
            check(failure.message == "injected callback allocation failure")
            failed = true
        }
        check(failed && callbackRowResourceLive() == 0L)
    }
}
