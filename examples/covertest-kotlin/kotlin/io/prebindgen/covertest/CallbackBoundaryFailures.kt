package io.prebindgen.covertest

import io.prebindgen.covertest.callbackboundaries.*
import io.prebindgen.covertest.callbacks.*
import java.lang.ref.WeakReference
import java.util.concurrent.atomic.AtomicReference

private object CallbackFailureProbe {
    init { NativeLibrary.ensureLoaded() }
    @JvmStatic external fun exercise(mode: Int): Int
}

object CallbackFailureEntryPoint {
    @JvmStatic fun run() {
        val failure = AtomicReference<Throwable?>()
        var calls = 0
        var delivered: List<CallbackOwned> = emptyList()
        callbackBoundaryFaults({ values ->
            try {
                calls++
                delivered = values
                check(values.map(::callbackOwnedId) == listOf(11L, 13L))
            } catch (error: Throwable) { failure.set(error) }
        })
        failure.get()?.let { throw it }
        check(calls == 1 && delivered.all { it.isClosed })
    }
}

private class RegistrationRejected : RuntimeException()

private fun rejectedRegistrationCapture(): WeakReference<Any> {
    val marker = Any()
    val weak = WeakReference(marker)
    var callbackCalls = 0
    var errors = 0
    try {
        callbackBoundaryRegister({ marker.hashCode(); callbackCalls++ }, true,
            onBindingError = { error("unexpected binding error: $it") },
            onError = { code, message ->
                check(code == 17L && message == "registration rejected")
                errors++
                throw RegistrationRejected()
            })
        error("expected rejected registration")
    } catch (_: RegistrationRejected) { }
    check(errors == 1 && callbackCalls == 0)
    return weak
}

private fun successfulRegistrationCapture(): Triple<CallbackBoundaryHandler, WeakReference<Any>, AtomicReference<Long?>> {
    val marker = Any()
    val observed = AtomicReference<Long?>()
    val handler = callbackBoundaryRegister({ marker.hashCode(); observed.set(it) }, false,
        onError = { _, _ -> error("unexpected registration error") })
    return Triple(handler, WeakReference(marker), observed)
}

private fun folderFailureCapture(): WeakReference<Any> {
    val marker = Any()
    val weak = WeakReference(marker)
    val original = IllegalStateException("expected folder failure")
    try {
        callbackBoundaryFoldFold({ marker.hashCode() }, 3, 0L) { _, _ -> throw original }
        error("expected folder failure")
    } catch (failure: IllegalStateException) { check(failure === original) }
    return weak
}

private fun collected(weak: WeakReference<Any>) {
    repeat(100) {
        System.gc()
        if (weak.get() == null) return
        Thread.sleep(10)
    }
    error("callback capture remained retained after release")
}

fun checkCallbackBoundaryFailures() {
    collected(rejectedRegistrationCapture())
    val (handler, weak, observed) = successfulRegistrationCapture()
    System.gc()
    check(weak.get() != null)
    callbackBoundaryFire(handler, 59)
    check(observed.get() == 59L)
    handler.close()
    collected(weak)

    val seen = mutableListOf<Long>()
    check(callbackBoundaryFoldFold({ seen.add(it); Unit }, 3, 10L) { sum, value ->
        check(seen == listOf(0L, 1L, 2L))
        sum + value
    } == 13L)
    collected(folderFailureCapture())

    val failure = AtomicReference<Throwable?>()
    var converted = 0
    callbackBoundaryConversion({ first, number, last ->
        try {
            converted++
            check(callbackOwnedId(first) == 11L && number == 7L && callbackOwnedId(last) == 13L)
        } catch (error: Throwable) { failure.set(error) }
    })
    failure.get()?.let { throw it }
    check(converted == 1)
    // Each probe fails exactly one allocation inside the generated Rust
    // callback, then proves the second invocation succeeds after OOME cleanup.
    for (mode in 0..1) check(CallbackFailureProbe.exercise(mode) == 1)
    println("callback boundary failures PASS: typed registration errors, captures, folds, conversion panic and JNI allocation failures")
}
