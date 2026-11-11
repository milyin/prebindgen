@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.canonical

public fun canonicalMillisBorrow(`value`: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_millis_borrow_785f06d63a7833ec(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalMillisFromToken(`value`: io.prebindgen.covertest.canonical.CanonToken, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_millis_from_token_29b5b9a7d4693cc6(nativeArgument0, nativeArgument1) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalMillisEcho(`value`: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_millis_echo_78944b36ef45859e(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalMillisOptional(`value`: Long?, onError: io.prebindgen.covertest.JniErrorHandler<Long?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { (if (`value` != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (`value` != null) requireNotNull(`value`) else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_millis_optional_a1a3b5cc3bee6c0c(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) returnCapture.s1 else null
    } finally { returnCapture.release() }
}

public fun canonicalMillisRows(`value`: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w1_elements = try { if (true) `value`?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w7 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w7 }) } } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_millis_rows_1a2c665200830afe(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun canonicalMillisCallback(callback: io.prebindgen.covertest.CanonMillisToCanonMillisCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackReturned: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w3 = callbackReturned
  run { val callbackReturnWire = callback_return_w3
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_millis_callback_7a6e8a46e55294f2(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalMillisNotify(callback: io.prebindgen.covertest.CanonMillisCallback, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_millis_notify_6b6d5b03e5d23260(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun canonicalMillisStatic(onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_millis_static_7c48cab6bf804c36(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun <R> canonicalMillisViews(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Long, Long) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long2_ErrorEmpty_951bc171243c8a73.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_millis_views_ada96198aef4e8f9(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    build(returnCapture.s0, returnCapture.s1)
    } finally { returnCapture.release() }
}

public fun canonicalCelsius(`value`: Int, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_celsius_7911a7eaf4d4cc9d(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalPercent(`value`: Int?, onError: io.prebindgen.covertest.JniErrorHandler<Int?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Int2_ErrorEmpty_1d2a9157b2d89875.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { (if (`value` != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (`value` != null) requireNotNull(`value`) else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_percent_e5139562f8847d8b(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) returnCapture.s1 else null
    } finally { returnCapture.release() }
}

public fun canonicalPercentInvalid(onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_percent_invalid_5b6e7123908f8a68(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalPercentCallback(callback: io.prebindgen.covertest.VoidToCanonPercentCallback, onError: io.prebindgen.covertest.JniErrorHandler<Boolean> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Int {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: Int = 0
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
  run { val callbackReturnWire = callback_return_w3
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_percent_callback_1cf4de9cee6bd4e1(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalPercentNotify(callback: io.prebindgen.covertest.CanonPercentCallback, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int) {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Int = 0
        try {
            callbackArgument0_0 = raw0 as Int
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_percent_notify_085bf1b1c0f0c84f(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun canonicalLabelEcho(`value`: String, onError: io.prebindgen.covertest.JniErrorHandler<String> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_label_echo_a05fdb0627a1339f(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult as String
    } finally { returnCapture.release() }
}

public fun canonicalSettings(`value`: io.prebindgen.covertest.canonical.CanonSettings, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.canonical.CanonSettings> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.canonical.CanonSettings = run {
    val p0_w2_elements = try { if (true) `value`.history?.toList()?.let { p0_w2_snapshot -> kotlin.Array<kotlin.Any?>(p0_w2_snapshot.size) { p0_w2_index -> val p0_w8 = p0_w2_snapshot[p0_w2_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w8 }) } } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Long_Int_Long5_ErrorEmpty_c47a63cbc2ce1435.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value`.delay } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { (if (`value`.optional != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (`value`.optional != null) requireNotNull(`value`.optional) else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p0_w2_elements } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { `value`.label } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_settings_83f8e071e261c29e(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputg2_g0_list: kotlin.collections.MutableList<Long>? = null
            if (returnCapture.s1 == 1) {
            }
outputg2_g0_list = kotlin.collections.ArrayList((returnCapture.s3 as kotlin.Array<*>).size)
for (outputg2_g0_index in (returnCapture.s3 as kotlin.Array<*>).indices) {
val outputg2_g0__row = (returnCapture.s3 as kotlin.Array<*>)[outputg2_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_1_127a3c24b600dbe0


outputg2_g0_list.add(outputg2_g0__row.c0)
}

io.prebindgen.covertest.canonical.CanonSettings(returnCapture.s0, if (returnCapture.s1 == 1) returnCapture.s2 else null, requireNotNull(outputg2_g0_list), returnCapture.s4 as String)
    } finally { returnCapture.release() }
}

public fun canonicalSettingsRows(`value`: kotlin.collections.List<io.prebindgen.covertest.canonical.CanonSettings>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.canonical.CanonSettings>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.canonical.CanonSettings> = run {
    val p0_w3_elements = try { if (true) `value`?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rd07d09e0f4e34169.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rd07d09e0f4e34169.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rd07d09e0f4e34169.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val p0_w8_elements = if (true) p0_w7.history?.toList() else null
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.size) + io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w7.delay })
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.size) + io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.offsets[1], io.prebindgen.covertest.InputValidation.value { (if (p0_w7.optional != null) 1 else 0) })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.size) + io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.offsets[2], io.prebindgen.covertest.InputValidation.value { if (p0_w7.optional != null) requireNotNull(p0_w7.optional) else 0L })
val p0_w8_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w8_elements }
if (p0_w8_bulkList == null) { p0_w1_writer.absent((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.size) + io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.offsets[3]) } else {
val p0_w8_bulkStart = p0_w1_writer.rows(p0_w8_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.size) + io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.offsets[3], p0_w8_bulkStart, p0_w8_bulkList.size)
p0_w8_bulkList.forEachIndexed { p0_w8_bulkIndex, p0_w14 ->
p0_w1_writer.long((p0_w8_bulkStart + p0_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w14 })
}
}
p0_w1_writer.text((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.size) + io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.offsets[4], io.prebindgen.covertest.InputValidation.value { p0_w7.label })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_settings_rows_0fc7aa0d7b8464de(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.canonical.CanonSettings>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rd07d09e0f4e34169.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.size
var outputbulkItem_g2_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_g2_g0_bulkList: kotlin.collections.MutableList<Long>? = null

            if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.offsets[1])) == 1) {
            }
outputbulkItem_g2_g0_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.offsets[3])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkItem_g2_g0_bulkList = kotlin.collections.ArrayList(outputbulkItem_g2_g0_bulkRange!!.length)
for (outputbulkItem_g2_g0_bulkIndex in 0 until outputbulkItem_g2_g0_bulkRange!!.length) {
val outputbulkItem_g2_g0_bulkBase = outputbulkItem_g2_g0_bulkRange!!.offset + outputbulkItem_g2_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkItem_g2_g0_bulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

outputbulkList!!.add(io.prebindgen.covertest.canonical.CanonSettings((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.offsets[0])) as Long, if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.offsets[1])) == 1) (requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.offsets[2])) as Long else null, requireNotNull(outputbulkItem_g2_g0_bulkList), (requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rf2e4d425e0f43581.offsets[4])) as String))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun canonicalSettingsCallback(callback: io.prebindgen.covertest.canonical.CanonSettingsToCanonSettingsCallback, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.canonical.CanonSettings> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.canonical.CanonSettings = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Int_Long5_ErrorEmpty_c47a63cbc2ce1435.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Int, raw2: Long, raw3: Array<Any?>?, raw4: String?): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_g2_g0_list: kotlin.collections.MutableList<Long>? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.canonical.CanonSettings
        lateinit var callbackReturned: io.prebindgen.covertest.canonical.CanonSettings
        try {
            if (raw1 == 1) {
            }
callbackOutput0_g2_g0_list = kotlin.collections.ArrayList((raw3 as kotlin.Array<*>).size)
for (callbackOutput0_g2_g0_index in (raw3 as kotlin.Array<*>).indices) {
val callbackOutput0_g2_g0__row = (raw3 as kotlin.Array<*>)[callbackOutput0_g2_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_1_127a3c24b600dbe0


callbackOutput0_g2_g0_list.add(callbackOutput0_g2_g0__row.c0)
}
            callbackArgument0_0 = io.prebindgen.covertest.canonical.CanonSettings(raw0 as Long, if (raw1 == 1) raw2 as Long else null, requireNotNull(callbackOutput0_g2_g0_list), raw4 as String)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
 val callback_return_w2_elements = if (true) callback_return_w1.history?.toList()?.let { callback_return_w2_snapshot -> kotlin.Array<kotlin.Any?>(callback_return_w2_snapshot.size) { callback_return_w2_index -> val callback_return_w8 = callback_return_w2_snapshot[callback_return_w2_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { callback_return_w8 }) } } else null
 run { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1.delay, (if (callback_return_w1.optional != null) 1 else 0), if (callback_return_w1.optional != null) requireNotNull(callback_return_w1.optional) else 0L, callback_return_w2_elements, callback_return_w1.label)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_settings_callback_5b78af5d7a99f7b2(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputg2_g0_list: kotlin.collections.MutableList<Long>? = null
            if (returnCapture.s1 == 1) {
            }
outputg2_g0_list = kotlin.collections.ArrayList((returnCapture.s3 as kotlin.Array<*>).size)
for (outputg2_g0_index in (returnCapture.s3 as kotlin.Array<*>).indices) {
val outputg2_g0__row = (returnCapture.s3 as kotlin.Array<*>)[outputg2_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_1_127a3c24b600dbe0


outputg2_g0_list.add(outputg2_g0__row.c0)
}

io.prebindgen.covertest.canonical.CanonSettings(returnCapture.s0, if (returnCapture.s1 == 1) returnCapture.s2 else null, requireNotNull(outputg2_g0_list), returnCapture.s4 as String)
    } finally { returnCapture.release() }
}

public fun canonicalViewEcho(`value`: io.prebindgen.covertest.canonical.CanonView, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.canonical.CanonView> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.canonical.CanonView = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value`.delay } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_view_echo_24a693eaade25f01(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    io.prebindgen.covertest.canonical.CanonView(nativeResult)
    } finally { returnCapture.release() }
}

public fun canonicalDomainError(`value`: Long, onBindingError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.CanonErrorHandler<Long>): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorInt1_5ff51a1265add82b.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_domain_error_d7d0cad0f0b11ef5(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalTokenNew(`value`: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.canonical.CanonToken> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.canonical.CanonToken = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_token_new_395cad9680466497(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.canonical.CanonToken? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.canonical.CanonToken? = null
        try {
            output0 = io.prebindgen.covertest.canonical.CanonToken(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun canonicalTokenValue(`value`: io.prebindgen.covertest.canonical.CanonToken, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_token_value_6611e447afc30b2c(nativeArgument0, nativeArgument1) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalLive(onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_live_a0e805b1e44c8892(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalOutputRows(fail: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.canonical.CanonOutput>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.canonical.CanonOutput> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { fail } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_output_rows_3923c3720cf2c318(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.canonical.CanonOutput>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r0981bbf0720dfabe.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: io.prebindgen.covertest.canonical.CanonToken? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.canonical.CanonOutput>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r0981bbf0720dfabe.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.size
        var outputbulkItem_g0_0: io.prebindgen.covertest.canonical.CanonToken? = null

            outputbulkItem_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.canonical.CanonToken((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.offsets[0])) as Long))

outputbulkList!!.add(io.prebindgen.covertest.canonical.CanonOutput(requireNotNull(outputbulkItem_g0_0), (requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.offsets[1])) as Int))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r0981bbf0720dfabe.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: io.prebindgen.covertest.canonical.CanonToken? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun canonicalOutputRowsPlain(fail: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.canonical.CanonOutput>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.canonical.CanonOutput> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { fail } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_output_rows_plain_08df4e0deb89a7bc(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.canonical.CanonOutput>? = null
 outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Int_2_cec315495ba8c6f9
        var outputitem_g0_0: io.prebindgen.covertest.canonical.CanonToken? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7(output_row.c0) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.canonical.CanonOutput>? = null
        try {
outputowners = io.prebindgen.covertest.CollectionOwners()
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Int_2_cec315495ba8c6f9
        var outputitem_g0_0: io.prebindgen.covertest.canonical.CanonToken? = null

            outputitem_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.canonical.CanonToken(output_row.c0))

outputlist.add(io.prebindgen.covertest.canonical.CanonOutput(requireNotNull(outputitem_g0_0), output_row.c1))
}

outputowners!!.complete = true
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
outputowners?.close(failure)
if (outputowners?.complete != true) {
try {
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Int_2_cec315495ba8c6f9
        var outputitem_g0_0: io.prebindgen.covertest.canonical.CanonToken? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7(output_row.c0) } } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun canonicalOutputNotify(callback: io.prebindgen.covertest.CanonOutputVecCallback, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_owners: io.prebindgen.covertest.CollectionOwners? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.canonical.CanonOutput>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.canonical.CanonOutput>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_owners = io.prebindgen.covertest.CollectionOwners()
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r0981bbf0720dfabe.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.size
        var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.canonical.CanonToken? = null

            callbackOutput0_bulkItem_g0_0 = requireNotNull(callbackOutput0_owners).own(io.prebindgen.covertest.canonical.CanonToken((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.offsets[0])) as Long))

callbackOutput0_bulkList!!.add(io.prebindgen.covertest.canonical.CanonOutput(requireNotNull(callbackOutput0_bulkItem_g0_0), (requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.offsets[1])) as Int))
}

callbackOutput0_owners!!.complete = true
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
callbackCleanupFailure = callbackOutput0_owners?.close(callbackCleanupFailure) ?: callbackCleanupFailure
if (callbackOutput0_owners?.complete != true) {
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r0981bbf0720dfabe.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.size)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.canonical.CanonToken? = null

            try { if (callbackOutput0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }

}
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
callbackCleanupFailure = callbackOutput0_owners?.close(callbackCleanupFailure) ?: callbackCleanupFailure
if (callbackOutput0_owners?.complete != true) {
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r0981bbf0720dfabe.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.size)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.canonical.CanonToken? = null

            try { if (callbackOutput0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r2d58718dc7f0bdda.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }

}
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_output_notify_66c5ebf5a1007e56(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun canonicalConsumeRows(values: kotlin.collections.List<io.prebindgen.covertest.canonical.CanonToken>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w7 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w7 }) }, intArrayOf(2), 1, null) } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_elements?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_consume_rows_c103dfd5f1f626c1(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_elements?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_elements?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalConsumeRowsBulk(values: kotlin.collections.List<io.prebindgen.covertest.canonical.CanonToken>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_ef0d998e3fda7d97()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.ref0d998e3fda7d97.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rd684c8ff2b4fcdbf)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.ref0d998e3fda7d97.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w9 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rd684c8ff2b4fcdbf.size) + io.prebindgen.covertest.BulkLayouts.rd684c8ff2b4fcdbf.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w9 }, true, 0)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_consume_rows_bulk_f50739fe8a8eb65e(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalConsume(first: io.prebindgen.covertest.canonical.CanonToken, second: io.prebindgen.covertest.canonical.CanonToken, third: io.prebindgen.covertest.canonical.CanonToken, onBindingError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.CanonErrorHandler<Long>): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorInt1_5ff51a1265add82b.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { first } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.covertest.NativeHandle? = try { second } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle2: io.prebindgen.covertest.NativeHandle? = try { third } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; if (__pb_handle0 != null && __pb_handle1 != null && __pb_handle0.address == __pb_handle1.address) throw io.prebindgen.covertest.HandleValidationFailure("conflicting input aliases"); if (__pb_handle0 != null && __pb_handle2 != null && __pb_handle0.address == __pb_handle2.address) throw io.prebindgen.covertest.HandleValidationFailure("conflicting input aliases"); if (__pb_handle1 != null && __pb_handle2 != null && __pb_handle1.address == __pb_handle2.address) throw io.prebindgen.covertest.HandleValidationFailure("conflicting input aliases"); io.prebindgen.covertest.withHandleLocks(__pb_handle0, __pb_handle1, __pb_handle2) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_handle1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_handle2?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_consume_315782551208bc05(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; __pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) }; __pb_handle2?.let { __pb_consume(it) } } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; __pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) }; __pb_handle2?.let { __pb_consume(it) } } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalOwnedCallback(callback: io.prebindgen.covertest.CanonOwnedToCanonOwnedCallback, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.canonical.CanonToken> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.canonical.CanonToken = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_0: io.prebindgen.covertest.canonical.CanonToken? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.canonical.CanonToken
        lateinit var callbackReturned: io.prebindgen.covertest.canonical.CanonToken
var callbackReturnReady = false
        try { var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackOutput0_0 = io.prebindgen.covertest.canonical.CanonToken(raw0 as Long)
            callbackArgument0_0 = requireNotNull(callbackOutput0_0)
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            callbackReturned = callback.run(callbackArgument0_0); callbackReturnReady = true
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        } } catch (failure: Throwable) { if (callbackReturnReady) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callbackReturned as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue0 -> returnedHandles.add(returnedValue0); returnedOwners.add(returnedValue0) }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() } }; throw failure }
return kotlin.run { val callback_return_w3 = callbackReturned
 try {  run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { callback_return_w3 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { val callbackReturnWire = __pb_handle0?.raw ?: 0L
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
__pb_handle0?.let { __pb_consume(it) }
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue0 -> returnedHandles.add(returnedValue0); returnedOwners.add(returnedValue0) }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_owned_callback_676d03635ad3da67(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.canonical.CanonToken? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.canonical.CanonToken? = null
        try {
            output0 = io.prebindgen.covertest.canonical.CanonToken(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CanonToken_67aeebcce8af1ce7(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun canonicalMillisCallbackLocal(callback: io.prebindgen.covertest.CanonicalIntCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int): Int {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Int = 0
        var callbackReturned: Int = 0
        try {
            callbackArgument0_0 = raw0 as Int
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w3 = callbackReturned
  run { val callbackReturnWire = callback_return_w3
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_millis_callback_local_0dc4639a5a3d89d9(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun canonicalMillisLocal(`value`: Int, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_canonical_millis_local_63762bf3eee320d5(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}
