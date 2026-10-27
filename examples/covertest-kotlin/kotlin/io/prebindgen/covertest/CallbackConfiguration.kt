package io.prebindgen.covertest

import io.prebindgen.covertest.callbackconfig.*

fun checkCallbackConfiguration() {
    check(callbackConfigTypeDefault(p0_callback = { it * 2 }) == 42L)
    check(callbackConfigDefault({ number, extra -> number + extra == 28L }) == 42L)
    check(callbackConfigArgument({ positive, extra -> positive && extra == 7L }) == 42L)
    check(callbackConfigReturn({ number, extra -> number + extra }) == 28L)
    check(callbackConfigPair({ number, _ -> number == 21L }, { positive, _ -> !positive }) == 41L)
    check(callbackConfigNested(listOf(
        NestedHandler { positive, extra -> if (positive) extra else 0L },
        NestedHandler { _, extra -> extra * 2 },
    )) == 21L)
    check(callbackConfigOptional(ConfiguredHandler { number, extra -> number + extra == 28L }) == 42L)
    check(callbackConfigOptional(null) == -1L)
    check(callbackConfigOrdinary(19) == 19L)
    val unused = UnusedHandler { if (it) 1.toShort() else 0.toShort() }
    check(unused.run(true) == 1.toShort())
    println("callback configuration PASS: defaults, argument/result overrides, constructors, collections, options and workers")
}
