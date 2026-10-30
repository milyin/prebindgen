package io.prebindgen.covertest

import io.prebindgen.covertest.results.*
import io.prebindgen.covertest.options.optionResourceNew
import io.prebindgen.covertest.options.optionResourceId
import io.prebindgen.covertest.options.optionalTextInputLen
import io.prebindgen.covertest.resources.ownedResourceDrops
import io.prebindgen.covertest.enums.Code
import io.prebindgen.covertest.enums.Priority
import java.io.IOException
import io.prebindgen.covertest.ResultFailureHandler
import io.prebindgen.covertest.StringMessageHandler
import io.prebindgen.covertest.CResourceErrorHandler
import io.prebindgen.covertest.CRichErrorHandler

private inline fun <reified T : Throwable> resultFailure(message: String, action: () -> Unit) {
    try { action() } catch (failure: Throwable) {
        check(failure is T) { "expected ${T::class}, got $failure" }
        check(failure.message == message) { "unexpected message: ${failure.message}" }
        return
    }
    error("expected ${T::class}")
}

fun checkResults() {
    check(resultUnitError(false, onError = { error("unexpected unit error") }) == 42L)
    check(resultUnitError(true, onError = { 17L }) == 17L)
    // The getter's .name("code") survives preparation into the handler API.
    check(ResultFailureHandler<Long> { it }.run(code = 7L) == 7L)
    check(optionalTextInputLen(true, null, 0, onError = JniErrorHandler { 123L }) == 123L)
    val throwResult = ResultFailureHandler<Nothing> { code -> throw IOException("failure $code 世界") }
    val errorStart = resultErrorDrops()
    check(resultScalar(42, onError = throwResult) == 42L)
    resultFailure<IOException>("failure -7 世界") { resultScalar(-7, onError = throwResult) }
    resultUnit(true, onError = throwResult)
    resultFailure<IOException>("failure 1 世界") { resultUnit(false, onError = throwResult) }
    check(resultOptional(0, onError = throwResult) == null)
    check(resultOptional(1, onError = throwResult) == Code.Seven)
    resultFailure<IOException>("failure 2 世界") { resultOptional(2, onError = throwResult) }
    check(resultText(true, onError = throwResult) == "héllo 世界")
    resultFailure<IOException>("failure 3 世界") { resultText(false, onError = throwResult) }
    val record = resultRecord(true, onError = throwResult)
    check(record.priority == Priority.High && record.code == Code.MinusOne)
    resultFailure<IOException>("failure 4 世界") { resultRecord(false, onError = throwResult) }
    check(resultErrorDrops() == errorStart + 5uL)
    check(resultOther(true, onError = StringMessageHandler { error(it) }) == 42L)
    resultFailure<ResultExceptions.Nested>("other error") { resultOther(false, onError = StringMessageHandler { throw ResultExceptions.Nested(it) }) }
    storageTryWithLabel("label").close()
    resultFailure<IllegalStateException>("label must not be empty") { storageTryWithLabel("") }
    try { storageTryWithLabel(null); error("null input accepted") }
    catch (_: IllegalArgumentException) {}

    val start = ownedResourceDrops()
    for (mode in listOf(0, 1, 2, 3)) {
        val invalid = checkNotNull(optionResourceNew(true))
        try { resultOwned(invalid, mode, 0xd800, onError = throwResult); error("invalid char accepted") }
        catch (_: IllegalArgumentException) {}
        check(invalid.isClosed)
        val value = checkNotNull(optionResourceNew(true))
        when (mode) {
            0 -> {
                val moved = resultOwned(value, mode, 65, onError = throwResult)
                check(value.isClosed && !moved.isClosed)
                check(optionResourceId(moved) == 77L)
                moved.close()
            }
            1 -> resultFailure<IOException>("failure 5 世界") { resultOwned(value, mode, 65, onError = throwResult) }
            else -> resultFailure<IllegalArgumentException>("Rust operation panicked") { resultOwned(value, mode, 65, onError = throwResult) }
        }
        check(value.isClosed)
        value.close() // acknowledgement prevents a second native free
    }
    check(ownedResourceDrops() == start + 8uL)
    check(resultErrorDrops() == errorStart + 7uL)
    // Typed callbacks can recover instead of throwing; success never invokes them.
    check(resultScalar(-11) { code -> -code } == 11L)
    check(resultScalar(-17, onError = ResultFailureHandler { code -> -code }) == 17L)
    check(resultUnit(false, onError = ResultFailureHandler { code -> check(code == 1L) }) == Unit)
    val fallback = checkNotNull(optionResourceNew(true))
    check(resultOwned(null, 0, 65, onError = throwResult,
        onBindingError = JniErrorHandler { check(it?.contains("null") == true); fallback }) === fallback)
    val closed = checkNotNull(optionResourceNew(true)); closed.close()
    check(resultOwned(closed, 0, 65, onError = throwResult,
        onBindingError = JniErrorHandler { check(it == "null or misaligned allocation pointer"); fallback }) === fallback)
    fallback.close()
    val borrowed = checkNotNull(optionResourceNew(true))
    check(resultOwned(borrowed, 0, 0xd800, onError = throwResult,
        onBindingError = JniErrorHandler {
            val checked = java.util.concurrent.atomic.AtomicBoolean(false)
            val worker = Thread { checked.set(synchronized(borrowed) { borrowed.isClosed }) }.also { it.isDaemon = true; it.start() }
            worker.join(3000)
            check(!worker.isAlive && checked.get()) { "binding callback ran while the handle monitor was held" }
            borrowed
        }) === borrowed)
    check(borrowed.isClosed)
    borrowed.close()
    check(cResultLateNiche(false, onError = CResultErrorHandler { _, _ -> "error" }) { _, _ -> "success" } == "success")
    check(cResultLateNiche(true, onError = CResultErrorHandler { code, message -> "$code:$message" }) { _, _ -> "success" } == "8:late niche")
    val beforeErrorHandle = ownedResourceDrops()
    var scoped: io.prebindgen.covertest.resources.OwnedResource? = null
    var retained: io.prebindgen.covertest.resources.OwnedResource? = null
    cResultResourceError(true, onError = CResourceErrorHandler { resource, message ->
        check(optionResourceId(resource) == 91L && message == "resource error")
        scoped = resource
    })
    check(scoped!!.isClosed && ownedResourceDrops() == beforeErrorHandle + 1uL)
    cResultResourceError(true, onError = CResourceErrorHandler { resource, _ -> retained = resource.take() })
    check(!retained!!.isClosed && optionResourceId(retained!!) == 91L)
    retained!!.close()
    val original = IOException("callback identity")
    try { cResultResourceError(true, onError = CResourceErrorHandler { _, _ -> throw original }); error("callback did not throw") }
    catch (failure: IOException) { check(failure === original) }
    check(ownedResourceDrops() == beforeErrorHandle + 3uL)
    cResultErrorDrop(onError = CDropErrorHandler { resource -> check(optionResourceId(resource) == 12L) })
    check(ownedResourceDrops() == beforeErrorHandle + 4uL)
    for (present in listOf(false, true)) cResultRichError(present, onError = CRichErrorHandler { kind, message ->
        check(kind == Code.Seven)
        check(message == if (present) "rich error" else null)
    })
    resultFailure<IOException>("original") { ResultRuntimeProbe.pendingException() }
}

internal object ResultRuntimeProbe {
    @JvmStatic external fun pendingException()
}

class ResultExceptions {
    class Nested(message: String) : RuntimeException(message)
}
