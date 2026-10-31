package io.prebindgen.covertest

import io.prebindgen.covertest.callbackreturns.*
import io.prebindgen.covertest.callbacks.callbackRowReturnCleanupFailure
import io.prebindgen.covertest.callbacks.callbackRowResourceLive
import java.math.BigInteger
import java.util.concurrent.atomic.AtomicInteger

private class RawReturnedFields(private val body: () -> Array<Any?>) {
    fun run(): Array<Any?> = body()
}

private fun invalidCallbackReturn(action: () -> Unit) {
    try { action() } catch (_: IllegalArgumentException) { return }
    error("expected callback result failure")
}

fun checkCallbackReturns() {
    check(callbackReturnI64({ it * 2 }) == 42L)
    check(callbackReturnI8({ (it + 1).toByte() }) == (-7).toByte())
    check(callbackReturnI16({ (it + 1).toShort() }) == (-1599).toShort())
    check(callbackReturnF32({ it * 2 }) == 2.5f)
    check(callbackReturnU128({ it - BigInteger.ONE }) == BigInteger.ONE.shiftLeft(128) - BigInteger.TWO)
    check(callbackReturnZero({ 42L }) == 42L)
    check(!callbackReturnBool({ !it }))
    check(callbackReturnChar({ it + 1 }) == 0x1f981)
    check(callbackReturnFloat({ it * 2 }) == 2.5)
    check(callbackReturnUnsigned({ it - 1UL }) == ULong.MAX_VALUE - 1UL)
    check(callbackReturnWide({ it - BigInteger.ONE }) == BigInteger.ONE.shiftLeft(127) - BigInteger.TWO)
    check(callbackReturnText({ check(it == "héllo\u0000世界"); "returned\u0000🦀" }) == "returned\u0000🦀")
    check(callbackReturnBytes({ it.reversedArray() }).contentEquals(byteArrayOf(-1, -128, 0)))
    check(callbackReturnOptional({ it?.times(2) }, true) == 42L)
    check(callbackReturnOptional({ check(it == null); null }, false) == null)
    check(callbackReturnVector({ it.map { value -> value * 2 } }) == listOf(2L, 4L, 6L))
    check(callbackReturnRecord({ check(it.label == "row"); CallbackReturnRecord(42, "answer") }) == 42L)
    check(callbackReturnEnum({ check(it == CallbackReturnChoice.First); CallbackReturnChoice.Second }))

    check(callbackReturnResourceLive() == 0UL)
    check(callbackReturnTake({ it.take() }) == 91L && callbackReturnResourceLive() == 0UL)
    invalidCallbackReturn { callbackReturnTake({ it }) }
    check(callbackReturnResourceLive() == 0UL)
    val owned = callbackReturnResourceNew(71)
    check(callbackReturnOwned({ owned }) == 71L && owned.isClosed)
    check(callbackReturnResourceLive() == 0UL)
    check(callbackReturnOwnedOptional({ null }) == null)
    val optional = callbackReturnResourceNew(72)
    check(callbackReturnOwnedOptional({ optional }) == 72L && optional.isClosed)
    check(callbackReturnResourceLive() == 0UL)
    val rows = listOf(callbackReturnResourceNew(11), callbackReturnResourceNew(12))
    check(callbackReturnOwnedVector({ rows }) == 23L && rows.all { it.isClosed })
    check(callbackReturnResourceLive() == 0UL)
    val duplicate = callbackReturnResourceNew(73)
    invalidCallbackReturn { callbackReturnOwnedVector({ listOf(duplicate, duplicate) }) }
    check(duplicate.isClosed && callbackReturnResourceLive() == 0UL)
    val partial = listOf(callbackReturnResourceNew(74), null, callbackReturnResourceNew(75))
    invalidCallbackReturn { callbackReturnOwnedVector({ partial }) }
    check(partial.filterNotNull().all { it.isClosed } && callbackReturnResourceLive() == 0UL)

    var unpublished: CallbackReturnResource? = null
    invalidCallbackReturn { callbackRowReturnCleanupFailure({ callbackReturnResourceNew(76).also { unpublished = it } }) }
    check(unpublished!!.isClosed && callbackReturnResourceLive() == 0UL && callbackRowResourceLive() == 0L)
    val workers = AtomicInteger()
    check(callbackReturnJoin(callbackReturnSpawn({ workers.incrementAndGet(); it * 2 })) == 19800L)
    check(workers.get() == 200)
    val handler = callbackReturnHandlerNew({ it + 21 }, false, onError = { throw IllegalArgumentException(it) })
    check(callbackReturnHandlerCall(handler, 21) == 42L)
    handler.close()
    check(callbackReturnHandlerValue({ it * 2 }) == 42L)
    check(callbackReturnHandlerMany(listOf(i64Toi64Callback { it * 2 }, i64Toi64Callback { it + 1 })) == 64L)
    invalidCallbackReturn { callbackReturnHandlerNew({ it }, true, onError = { throw IllegalArgumentException(it) }) }

    invalidCallbackReturn { callbackReturnI64({ throw IllegalStateException("returned callback failure") }) }
    invalidCallbackReturn { callbackReturnChar({ 0xd800 }) }
    check(callbackReturnCatch({ if (it == 0L) throw IllegalStateException("first invocation fails"); it * 2 }) == 42L)
    check(callbackReturnI64({ it * 2 }) == 42L)
    var first = true
    check(callbackReturnCharCatch({ if (first) { first = false; 0xd800 } else 'c'.code }) == 'c'.code)
    var vectorCalls = 0
    check(callbackReturnOwnedVectorCatch({
        vectorCalls++
        if (vectorCalls == 1) {
            val value = callbackReturnResourceNew(81)
            listOf(value, value, callbackReturnResourceNew(82))
        } else {
            check(callbackReturnResourceLive() == 0UL)
            listOf(callbackReturnResourceNew(83))
        }
    }) == 83L)
    check(vectorCalls == 2 && callbackReturnResourceLive() == 0UL)
    // A malformed non-owning wire must not bypass cleanup of the returned owner.
    val rawMethod = JNINative::class.java.declaredMethods.single { it.name.startsWith("call_callback_return_wire_row_catch_") }.also { it.isAccessible = true }
    var rawCalls = 0
    val rawBridge = RawReturnedFields {
        if (rawCalls > 0) check(callbackReturnResourceLive() == 0UL)
        val resource = callbackReturnResourceNew(84)
        val address = resource.raw
        resource.markConsumed()
        rawCalls++
        arrayOf<Any?>(address, if (rawCalls == 1) "wrong boxed class" else 'c'.code)
    }
    val rawPacket = rawMethod.invokePacket(rawBridge)
    check(rawPacket[0] == null && rawPacket[1] == 84L)
    check(rawCalls == 2 && callbackReturnResourceLive() == 0UL)
    val rawRowsMethod = JNINative::class.java.declaredMethods.single { it.name.startsWith("call_callback_return_wire_rows_catch_") }.also { it.isAccessible = true }
    var rowsCalls = 0
    val rowsBridge = RawReturnedFields {
        check(callbackReturnResourceLive() == 0UL)
        rowsCalls++
        Array<Any?>(if (rowsCalls == 1) 3 else 1) { index ->
            val resource = callbackReturnResourceNew(87)
            val address = resource.raw
            resource.markConsumed()
            arrayOf<Any?>(address, if (rowsCalls == 1 && index == 1) "wrong boxed class" else 'c'.code)
        }
    }
    val rowsPacket = rawRowsMethod.invokePacket(rowsBridge)
    check(rowsPacket[0] == null && rowsPacket[1] == 87L)
    check(rowsCalls == 2 && callbackReturnResourceLive() == 0UL)
    var unit = 0L
    callbackReturnUnit({ unit = it })
    check(unit == 21L)
    println("callback returns PASS: scalars, records, collections, owned handles, errors and retained native workers")
}
