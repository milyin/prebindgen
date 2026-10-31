@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package bench.plain

public fun `rowMake`(p0_w1: Long, onError: bench.plain.JniErrorHandler<bench.plain.Row> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.plain.Row = run {
    val returnCapture = bench.plain.Return_Long_Long_Long8_ErrorEmpty_f77363a35e75c509.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_row_make_bcd43e7b136151dd(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    bench.plain.Row(returnCapture.s0, returnCapture.s1, returnCapture.s2, returnCapture.s3, returnCapture.s4, returnCapture.s5, returnCapture.s6, returnCapture.s7)
    } finally { returnCapture.release() }
}

public fun `wideRowMake`(p0_w1: Long, onError: bench.plain.JniErrorHandler<bench.plain.WideRow> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.plain.WideRow = run {
    val returnCapture = bench.plain.Return_Long_Long_Long32_ErrorEmpty_6fba61a9625e2221.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_wide_row_make_1dbd8ca2fd4616ac(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    bench.plain.WideRow(returnCapture.s0, returnCapture.s1, returnCapture.s2, returnCapture.s3, returnCapture.s4, returnCapture.s5, returnCapture.s6, returnCapture.s7, returnCapture.s8, returnCapture.s9, returnCapture.s10, returnCapture.s11, returnCapture.s12, returnCapture.s13, returnCapture.s14, returnCapture.s15, returnCapture.s16, returnCapture.s17, returnCapture.s18, returnCapture.s19, returnCapture.s20, returnCapture.s21, returnCapture.s22, returnCapture.s23, returnCapture.s24, returnCapture.s25, returnCapture.s26, returnCapture.s27, returnCapture.s28, returnCapture.s29, returnCapture.s30, returnCapture.s31)
    } finally { returnCapture.release() }
}

public fun `wideRowSum`(p0_w1: bench.plain.WideRow, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1.`f0` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1.`f1` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p0_w1.`f2` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p0_w1.`f3` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { p0_w1.`f4` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { p0_w1.`f5` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { p0_w1.`f6` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument7 = try { p0_w1.`f7` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument8 = try { p0_w1.`f8` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument9 = try { p0_w1.`f9` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument10 = try { p0_w1.`f10` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument11 = try { p0_w1.`f11` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument12 = try { p0_w1.`f12` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument13 = try { p0_w1.`f13` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument14 = try { p0_w1.`f14` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument15 = try { p0_w1.`f15` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument16 = try { p0_w1.`f16` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument17 = try { p0_w1.`f17` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument18 = try { p0_w1.`f18` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument19 = try { p0_w1.`f19` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument20 = try { p0_w1.`f20` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument21 = try { p0_w1.`f21` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument22 = try { p0_w1.`f22` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument23 = try { p0_w1.`f23` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument24 = try { p0_w1.`f24` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument25 = try { p0_w1.`f25` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument26 = try { p0_w1.`f26` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument27 = try { p0_w1.`f27` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument28 = try { p0_w1.`f28` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument29 = try { p0_w1.`f29` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument30 = try { p0_w1.`f30` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument31 = try { p0_w1.`f31` } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument32 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_wide_row_sum_861bdfef124a25ad(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6, nativeArgument7, nativeArgument8, nativeArgument9, nativeArgument10, nativeArgument11, nativeArgument12, nativeArgument13, nativeArgument14, nativeArgument15, nativeArgument16, nativeArgument17, nativeArgument18, nativeArgument19, nativeArgument20, nativeArgument21, nativeArgument22, nativeArgument23, nativeArgument24, nativeArgument25, nativeArgument26, nativeArgument27, nativeArgument28, nativeArgument29, nativeArgument30, nativeArgument31, nativeArgument32) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `rowReturnHandlerNew`(p0_callback: bench.plain.RowReturnCallback?, onError: bench.plain.JniErrorHandler<bench.plain.RowReturnHandler> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.plain.RowReturnHandler = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        lateinit var callbackReturned: bench.plain.Row
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1.`a`, callback_return_w1.`b`, callback_return_w1.`c`, callback_return_w1.`d`, callback_return_w1.`e`, callback_return_w1.`f`, callback_return_w1.`g`, callback_return_w1.`h`)
val __pb_consume: (bench.plain.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_row_return_handler_new_9a8fcadcab7b9640(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: bench.plain.RowReturnHandler? = null
             try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_RowReturnHandler_0972ff7cb1556afa(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: bench.plain.RowReturnHandler? = null
        try {
            output0 = bench.plain.RowReturnHandler(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_RowReturnHandler_0972ff7cb1556afa(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `wideRowReturnHandlerNew`(p0_callback: bench.plain.WideRowReturnCallback?, onError: bench.plain.JniErrorHandler<bench.plain.WideRowReturnHandler> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.plain.WideRowReturnHandler = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        lateinit var callbackReturned: bench.plain.WideRow
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1.`f0`, callback_return_w1.`f1`, callback_return_w1.`f2`, callback_return_w1.`f3`, callback_return_w1.`f4`, callback_return_w1.`f5`, callback_return_w1.`f6`, callback_return_w1.`f7`, callback_return_w1.`f8`, callback_return_w1.`f9`, callback_return_w1.`f10`, callback_return_w1.`f11`, callback_return_w1.`f12`, callback_return_w1.`f13`, callback_return_w1.`f14`, callback_return_w1.`f15`, callback_return_w1.`f16`, callback_return_w1.`f17`, callback_return_w1.`f18`, callback_return_w1.`f19`, callback_return_w1.`f20`, callback_return_w1.`f21`, callback_return_w1.`f22`, callback_return_w1.`f23`, callback_return_w1.`f24`, callback_return_w1.`f25`, callback_return_w1.`f26`, callback_return_w1.`f27`, callback_return_w1.`f28`, callback_return_w1.`f29`, callback_return_w1.`f30`, callback_return_w1.`f31`)
val __pb_consume: (bench.plain.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_wide_row_return_handler_new_67f38d7168020371(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: bench.plain.WideRowReturnHandler? = null
             try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_WideRowReturnHandler_ed750f04931eb9ca(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: bench.plain.WideRowReturnHandler? = null
        try {
            output0 = bench.plain.WideRowReturnHandler(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else bench.plain.JNINative.drop_WideRowReturnHandler_ed750f04931eb9ca(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `rowReturnFire`(p0_w1: bench.plain.RowReturnHandler?, p1_w1: Long, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_row_return_fire_7c8a1183dcdd277c(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `wideRowReturnFire`(p0_w1: bench.plain.WideRowReturnHandler?, p1_w1: Long, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.plain.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_wide_row_return_fire_88ad5bbf9c8c73e2(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `control`(p0_w1: Long, onError: bench.plain.JniErrorHandler<Long> = bench.plain.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.plain.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.plain.HandleValidationFailure(failure.message ?: "invalid input") }; bench.plain.JNINative.call_control_b0073ee32096caea(nativeArgument0, nativeArgument1) } } catch (failure: bench.plain.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

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
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { __pb_consume: (bench.plain.NativeHandle) -> Unit -> ; kotlin.Unit })
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
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5.`a`, p0_w5.`b`, p0_w5.`c`, p0_w5.`d`, p0_w5.`e`, p0_w5.`f`, p0_w5.`g`, p0_w5.`h`) }, { kotlin.collections.emptyList<Nothing>() }, { __pb_consume: (bench.plain.NativeHandle) -> Unit -> ; kotlin.Unit })
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
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5.`a`, p0_w5.`b`, p0_w5.`c`, p0_w5.`d`, p0_w5.`e`, p0_w5.`f`, p0_w5.`g`, p0_w5.`h`) }, { kotlin.collections.emptyList<Nothing>() }, { __pb_consume: (bench.plain.NativeHandle) -> Unit -> ; kotlin.Unit })
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
val __pb_consume: (bench.plain.NativeHandle) -> Unit = { it.markConsumed() }

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
