package io.prebindgen.covertest

import io.prebindgen.covertest.canonical.*

fun checkCanonicalValues() {
    for (number in listOf(Long.MIN_VALUE, -1L, 0L, Long.MAX_VALUE)) {
        check(canonicalMillisEcho(number) == number)
        check(canonicalMillisOptional(number) == number)
        check(canonicalMillisBorrow(number) == number)
    }
    check(canonicalMillisOptional(null) == null)
    check(canonicalMillisStatic() == 31L)
    check(canonicalMillisViews { first, second -> check(first == 37L && second == first); first + second } == 74L)
    check(canonicalMillisLocal(13) == 13)
    check(canonicalMillisRows(listOf(3L, 7L, 11L)) == listOf(3L, 7L, 11L))
    check(canonicalMillisRows(emptyList()).isEmpty())
    check(canonicalCelsius(22) == 23)
    check(canonicalPercent(null) == null && canonicalPercent(100) == 100)
    check(canonicalPercent(101, onError = JniErrorHandler {
        check(it == "percent outside range: 101"); 42
    }) == 42)
    check(canonicalPercentInvalid(onError = JniErrorHandler {
        check(it == "invalid output percent: 101"); 43
    }) == 43)
    for (text in listOf("hello", "héllo 世界\u0000")) check(canonicalLabelEcho(text) == text)
    check(canonicalLabelEcho("", onError = JniErrorHandler {
        check(it == "empty canonical label"); "recovered"
    }) == "recovered")
    val settings = CanonSettings(3, null, listOf(7, 11), "settings")
    check(canonicalSettings(settings) == settings)
    check(canonicalSettingsRows(listOf(settings, settings.copy(optional = 17))) ==
        listOf(settings, settings.copy(optional = 17)))
    check(canonicalSettingsRows(emptyList()).isEmpty())
    check(canonicalSettingsRows(listOf(settings, settings.copy(label = ""), settings), onError = JniErrorHandler {
        check(it == "empty canonical label"); emptyList()
    }).isEmpty())
    check(canonicalSettings(settings.copy(label = ""), onError = JniErrorHandler { settings }) == settings)
    check(canonicalViewEcho(CanonView(41)) == CanonView(41))
    check(canonicalMillisCallback({ check(it == 19L); it + 1 }) == 20L)
    var notified = false
    canonicalMillisNotify({ check(it == 23L); notified = true })
    check(notified)
    check(canonicalMillisCallbackLocal({ value: Int -> check(value == 29); value + 1 }) == 30L)
    val callbackSettings = canonicalSettingsCallback({
        check(it == CanonSettings(3, null, listOf(7), "callback"))
        it.copy(delay = 5)
    })
    check(callbackSettings.delay == 5L)
    check(canonicalDomainError(7, onError = CanonErrorHandler { error("unexpected $it") }) == 7L)
    check(canonicalDomainError(-1, onBindingError = JniErrorHandler { error(it ?: "binding error") },
        onError = CanonErrorHandler { check(it == 17); 91L }) == 91L)

    check(canonicalPercentCallback({ 101 }))
    var invalidNotified = false
    canonicalPercentNotify({ invalidNotified = true })
    check(!invalidNotified)
    for (bulk in listOf(false, true)) {
        val tokens = listOf(canonicalTokenNew(1), canonicalTokenNew(-2), canonicalTokenNew(3))
        val recover = JniErrorHandler<Long> { check(it == "negative canonical token"); -1L }
        check((if (bulk) canonicalConsumeRowsBulk(tokens, recover) else canonicalConsumeRows(tokens, recover)) == -1L)
        check(tokens[0].isClosed && tokens[1].isClosed && !tokens[2].isClosed)
        check(canonicalLive() == 1L)
        tokens[2].close()
        val failed = if (bulk) canonicalOutputRows(true, JniErrorHandler { emptyList() }) else canonicalOutputRowsPlain(true, JniErrorHandler { emptyList() })
        check(failed.isEmpty() && canonicalLive() == 0L)
        val output = if (bulk) canonicalOutputRows(false) else canonicalOutputRowsPlain(false)
        check(output.map { canonicalTokenValue(it.token) } == listOf(0L,1L,2L))
        output.forEach { it.token.close() }
    }
    var outputCalled = false
    canonicalOutputNotify({ outputCalled = true })
    check(!outputCalled && canonicalLive() == 0L)
    check(canonicalLive() == 0L)
    val token = canonicalTokenNew(47)
    check(canonicalMillisFromToken(token) == 47L)
    check(!token.isClosed && canonicalTokenValue(token) == 47L)
    token.close()
    val first = canonicalTokenNew(1)
    val second = canonicalTokenNew(-2)
    val third = canonicalTokenNew(3)
    check(canonicalConsume(first, second, third,
        onError = CanonErrorHandler { error("unexpected domain error") },
        onBindingError = JniErrorHandler {
            check(it == "negative canonical token")
            check(first.isClosed && second.isClosed && !third.isClosed)
            // Recovery happens after the native call and after handle locks release.
            check(canonicalTokenValue(third) == 3L)
            -1L
        }) == -1L)
    check(canonicalLive() == 1L)
    third.close()
    var scoped: CanonToken? = null
    val returned = canonicalOwnedCallback({
        check(canonicalTokenValue(it) == 53L)
        scoped = it
        canonicalTokenNew(54)
    })
    check(scoped!!.isClosed && !returned.isClosed)
    check(canonicalTokenValue(returned) == 54L)
    returned.close()
    check(canonicalLive() == 0L)
}
