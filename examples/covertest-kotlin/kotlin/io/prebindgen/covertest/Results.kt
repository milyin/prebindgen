package io.prebindgen.covertest

import io.prebindgen.covertest.results.*
import io.prebindgen.covertest.options.optionResourceNew
import io.prebindgen.covertest.options.optionResourceId
import io.prebindgen.covertest.resources.ownedResourceDrops
import io.prebindgen.covertest.enums.Code
import io.prebindgen.covertest.enums.Priority
import java.io.IOException

private inline fun <reified T : Throwable> resultFailure(message: String, action: () -> Unit) {
    try { action() } catch (failure: Throwable) {
        check(failure is T) { "expected ${T::class}, got $failure" }
        check(failure.message == message) { "unexpected message: ${failure.message}" }
        return
    }
    error("expected ${T::class}")
}

fun checkResults() {
    val errorStart = resultErrorDrops()
    check(resultScalar(42) == 42L)
    resultFailure<IOException>("failure -7 世界") { resultScalar(-7) }
    resultUnit(true)
    resultFailure<IOException>("failure 1 世界") { resultUnit(false) }
    check(resultOptional(0) == null)
    check(resultOptional(1) == Code.Seven)
    resultFailure<IOException>("failure 2 世界") { resultOptional(2) }
    check(resultText(true) == "héllo 世界")
    resultFailure<IOException>("failure 3 世界") { resultText(false) }
    val record = resultRecord(true)
    check(record.priority == Priority.High && record.code == Code.MinusOne)
    resultFailure<IOException>("failure 4 世界") { resultRecord(false) }
    check(resultErrorDrops() == errorStart + 5uL)
    check(resultOther(true) == 42L)
    resultFailure<ResultExceptions.Nested>("other error") { resultOther(false) }
    storageTryWithLabel("label").close()
    resultFailure<IllegalStateException>("label must not be empty") { storageTryWithLabel("") }
    try { storageTryWithLabel(null); error("null input accepted") }
    catch (_: IllegalArgumentException) {}

    val start = ownedResourceDrops()
    for (mode in listOf(0, 1, 2, 3)) {
        val value = checkNotNull(optionResourceNew(true))
        try { resultOwned(value, mode, 0xd800); error("invalid char accepted") }
        catch (_: IllegalArgumentException) {}
        check(!value.isClosed && optionResourceId(value) == 77L)
        when (mode) {
            0 -> {
                val moved = resultOwned(value, mode, 65)
                check(value.isClosed && !moved.isClosed)
                check(optionResourceId(moved) == 77L)
                moved.close()
            }
            1 -> resultFailure<IOException>("failure 5 世界") { resultOwned(value, mode, 65) }
            else -> resultFailure<IllegalArgumentException>("Rust operation panicked") { resultOwned(value, mode, 65) }
        }
        check(value.isClosed)
        value.close() // acknowledgement prevents a second native free
    }
    check(ownedResourceDrops() == start + 4uL)
    check(resultErrorDrops() == errorStart + 7uL)
    try { ResultRuntimeProbe.missingClass(); error("missing exception class accepted") }
    catch (_: NoClassDefFoundError) {}
    resultFailure<IOException>("original") { ResultRuntimeProbe.pendingException() }
}

internal object ResultRuntimeProbe {
    @JvmStatic external fun missingClass()
    @JvmStatic external fun pendingException()
}

class ResultExceptions {
    class Nested(message: String) : RuntimeException(message)
}
