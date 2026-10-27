package io.prebindgen.covertest

import io.prebindgen.covertest.callbackresults.*
import io.prebindgen.covertest.callbackreturns.*
import java.util.concurrent.atomic.AtomicReference

fun checkCallbackResultArguments() {
    val numbers = mutableListOf<Long>()
    callbackResultValues({ numbers.add(it); Unit })
    check(numbers == listOf(11L, 13L))
    var units = 0
    callbackResultUnit({ units++; Unit })
    check(units == 2)
    var returns = 0
    check(callbackResultReturn({ returns++; it * 2 }) == 42L && returns == 1)
    var mapped = 0L
    callbackResultMapped({ mapped += it })
    check(mapped == 17L)
    callbackResultMapperPanic({ mapped += it })
    check(mapped == 34L)
    var positive = false
    callbackResultConfigured({ positive = it })
    check(positive)

    val failure = AtomicReference<Throwable?>()
    var left: CallbackReturnResource? = null
    var right: CallbackReturnResource? = null
    var handleCalls = 0
    callbackResultHandles({ first, value, last ->
        try {
            handleCalls++
            left = first
            right = last
            check(callbackReturnResourceId(first) == 11L && value == 7L)
            check(callbackReturnResourceId(last) == 13L)
        } catch (error: Throwable) { failure.set(error) }
    })
    failure.get()?.let { throw it }
    check(handleCalls == 1 && left!!.isClosed && right!!.isClosed)
    check(callbackReturnResourceLive() == 0UL)
    val caller = Thread.currentThread()
    var workerValue = 0L
    val handler = callbackResultRegister({
        try { check(Thread.currentThread() !== caller); workerValue = it }
        catch (error: Throwable) { failure.set(error) }
    })
    callbackResultWorker(handler)
    handler.close()
    failure.get()?.let { throw it }
    check(workerValue == 59L)
    println("callback Result arguments PASS: Ok delivery, Err suppression, mapper failure, cleanup, return values and workers")
}
