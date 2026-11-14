package io.prebindgen.covertest

import io.prebindgen.covertest.callbacks.*
import io.prebindgen.covertest.callbackreturns.*

private object CallbackDiagnosticsProbe {
    init { NativeLibrary.ensureLoaded() }
    @JvmStatic external fun install()
    @JvmStatic external fun events(): String
}

fun checkCallbackDiagnostics() {
    CallbackDiagnosticsProbe.install()
    callbackConstructorFire({}, 7)
    check(callbackReturnI64({ it * 2 }) == 42L)
    check(CallbackDiagnosticsProbe.events().isEmpty())
    callbackConstructorFire({ throw IllegalStateException("expected diagnostic upcall failure") }, 7)
    var afterFailure = CallbackDiagnosticsProbe.events()
    check(afterFailure.contains("prebindgen::callback"))
    check(afterFailure.contains("callback=") && afterFailure.contains("io.prebindgen.covertest.i64Callback"))
    check(afterFailure.contains("signature=\"(J)V\"") || afterFailure.contains("signature=(J)V"))
    check(afterFailure.contains("Java exception was thrown"))
    try {
        callbackReturnChar({ 0xd800 })
        error("expected invalid callback return")
    } catch (_: IllegalArgumentException) { }
    afterFailure = CallbackDiagnosticsProbe.events()
    check(afterFailure.contains("charTocharCallback"))
    check(afterFailure.contains("invalid Unicode scalar"))
    val beforeSuccess = afterFailure.length
    check(callbackReturnI64({ it * 2 }) == 42L)
    callbackConstructorFire({}, 9)
    check(CallbackDiagnosticsProbe.events().length == beforeSuccess)
    println("callback diagnostics PASS: application subscriber, callback context, conversion failure and quiet success")
}
