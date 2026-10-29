@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package bench.plain

public fun `scalar`(p0_w1: Long, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_scalar_a4a5da3f79ebbcaf(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `rowSum`(p0_w1: bench.plain.Row, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1.`a` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1.`b` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p0_w1.`c` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p0_w1.`d` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { p0_w1.`e` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { p0_w1.`f` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { p0_w1.`g` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument7 = try { p0_w1.`h` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument8 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_row_sum_db65517187cece4c(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6, nativeArgument7, nativeArgument8) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `numbersSum`(p0_w1: kotlin.collections.List<Long>, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_numbers_sum_47543406a5cdf588(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `rowsSum`(p0_w1: kotlin.collections.List<bench.plain.Row>, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5.`a`, p0_w5.`b`, p0_w5.`c`, p0_w5.`d`, p0_w5.`e`, p0_w5.`f`, p0_w5.`g`, p0_w5.`h`) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_rows_sum_d4a5ecfa43bf26b9(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `rowsMake`(p0_w1: Long, p1_w1: Long, onError: bench.plain.JniErrorHandler<kotlin.collections.List<bench.plain.Row>> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.plain.Row> = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_rows_make_6f7eb2a3ee93e0ae(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<bench.plain.Row>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
 try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<bench.plain.Row>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(bench.plain.Row(output_row[0] as Long, output_row[1] as Long, output_row[2] as Long, output_row[3] as Long, output_row[4] as Long, output_row[5] as Long, output_row[6] as Long, output_row[7] as Long))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `rowsEcho`(p0_w1: kotlin.collections.List<bench.plain.Row>, onError: bench.plain.JniErrorHandler<kotlin.collections.List<bench.plain.Row>> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.plain.Row> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5.`a`, p0_w5.`b`, p0_w5.`c`, p0_w5.`d`, p0_w5.`e`, p0_w5.`f`, p0_w5.`g`, p0_w5.`h`) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_rows_echo_5c26c8cf4751c454(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<bench.plain.Row>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
 try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<bench.plain.Row>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(bench.plain.Row(output_row[0] as Long, output_row[1] as Long, output_row[2] as Long, output_row[3] as Long, output_row[4] as Long, output_row[5] as Long, output_row[6] as Long, output_row[7] as Long))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `packetSum`(p0_w1: bench.plain.Packet, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1.`header`.`a` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1.`header`.`b` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p0_w1.`header`.`c` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p0_w1.`header`.`d` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { p0_w1.`header`.`e` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { p0_w1.`header`.`f` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { p0_w1.`header`.`g` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument7 = try { p0_w1.`header`.`h` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument8 = try { p0_w1.`bytes` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument9 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_packet_sum_c07ede9eb5de050f(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6, nativeArgument7, nativeArgument8, nativeArgument9) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `handlerNew`(p0_callback: bench.plain.RowsCallback?, onError: bench.plain.JniErrorHandler<bench.plain.Handler> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.plain.Handler = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?): Long {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<bench.plain.Row>? = null
var callbackOutput0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<bench.plain.Row>
        var callbackReturned: Long = 0L
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_list = kotlin.collections.mutableListOf()
for (callbackOutput0_rawRow in (raw0 as kotlin.Array<*>)) {
val callbackOutput0__row = callbackOutput0_rawRow as kotlin.Array<*>

callbackOutput0_cleanups.add { callbackOutput0_initialFailure -> var callbackOutput0_failure = callbackOutput0_initialFailure; try {  } catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }; callbackOutput0_failure }

callbackOutput0_list.add(bench.plain.Row(callbackOutput0__row[0] as Long, callbackOutput0__row[1] as Long, callbackOutput0__row[2] as Long, callbackOutput0__row[3] as Long, callbackOutput0__row[4] as Long, callbackOutput0__row[5] as Long, callbackOutput0__row[6] as Long, callbackOutput0__row[7] as Long))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
        } catch (failure: Throwable) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>


} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            callbackReturned = p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>


} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_handler_new_916a49ec8343b596(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: bench.plain.Handler? = null
             try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_Handler_bb814c4111561dda(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: bench.plain.Handler? = null
        try {
            output0 = bench.plain.Handler(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_Handler_bb814c4111561dda(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `fire`(p0_w1: bench.plain.Handler?, p1_w1: Long, p2_w1: Long, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p2_w1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_fire_6fdab6ebdaeffb11(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun <A> `rowsForReduceFold`(p0_w1: Long, p1_w1: Long, initial: A, onError: bench.plain.JniErrorHandler<A> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, bench.plain.Row) -> A): A = run {
    val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = bench.plain.JNINative.SequenceSink_rows_for_reduce_fold_6202aebdeb4d4c0b { v0, v1, v2, v3, v4, v5, v6, v7 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1, v2, v3, v4, v5, v6, v7)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_rows_for_reduce_fold_587a8aef1d9a0485(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, bench.plain.Row(__pb_fold_row[0] as Long, __pb_fold_row[1] as Long, __pb_fold_row[2] as Long, __pb_fold_row[3] as Long, __pb_fold_row[4] as Long, __pb_fold_row[5] as Long, __pb_fold_row[6] as Long, __pb_fold_row[7] as Long))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        }
    } finally { returnCapture.release() }
}
