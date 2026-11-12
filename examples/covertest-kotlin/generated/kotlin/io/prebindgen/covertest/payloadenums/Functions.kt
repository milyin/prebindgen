@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.payloadenums

public fun wireEventBorrowed(callback: io.prebindgen.covertest.payloadenums.WireEventCallback, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int, raw1: Long, raw2: Long, raw3: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_enumValue: io.prebindgen.covertest.payloadenums.WireEvent? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.payloadenums.WireEvent
        try {
callbackOutput0_enumValue = when (raw0 as Int) { 0 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Idle }
1 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Count((raw1 as Long).toULong()) }
2 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Packet((raw2 as Long).toULong(), raw3 as ByteArray) }
3 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyTuple }
4 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyRecord }
 else -> error("invalid native enum tag") }
            callbackArgument0_0 = requireNotNull(callbackOutput0_enumValue)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_event_borrowed_09bf8d605d27949b(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun wireSelfNameEcho(value_c2da6115cd444ccd: io.prebindgen.covertest.payloadenums.WireSelfName, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireSelfName> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.payloadenums.WireSelfName = run {
    val p0_w1_tag = try { when (val value = requireNotNull(value_c2da6115cd444ccd)) { is io.prebindgen.covertest.payloadenums.WireSelfName.WireSelfName -> 0
is io.prebindgen.covertest.payloadenums.WireSelfName.Other -> 1 } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_tag } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { (if (p0_w1_tag == 0) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireSelfName.WireSelfName).v0 else 0uL).toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_self_name_echo_5724c1bd8adc5e81(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputenumValue: io.prebindgen.covertest.payloadenums.WireSelfName? = null
outputenumValue = when (returnCapture.s0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireSelfName.WireSelfName((returnCapture.s1).toULong()) }
1 -> {  io.prebindgen.covertest.payloadenums.WireSelfName.Other }
 else -> error("invalid native enum tag") }

requireNotNull(outputenumValue)
    } finally { returnCapture.release() }
}

public fun wireNestedEcho(value_c2da6115cd444ccd: io.prebindgen.covertest.payloadenums.WireNestedDecision, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireNestedDecision> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.payloadenums.WireNestedDecision = run {
    val p0_w1_tag = try { when (val value = requireNotNull(value_c2da6115cd444ccd)) { is io.prebindgen.covertest.payloadenums.WireNestedDecision.Ok -> 0
is io.prebindgen.covertest.payloadenums.WireNestedDecision.Err -> 1 } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_a0_f0_tag = try { if (p0_w1_tag == 0) when (val value = requireNotNull((if (p0_w1_tag == 0) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireNestedDecision.Ok).v0 else null))) { is io.prebindgen.covertest.payloadenums.WireDecision.Ok -> 0
is io.prebindgen.covertest.payloadenums.WireDecision.Err -> 1 } else 0 } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Int_Int_Long5_ErrorEmpty_b90cba981b507bba.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_tag } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_a0_f0_tag } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { (if ((p0_w1_tag == 0) && p0_w1_a0_f0_tag == 0) (requireNotNull((if (p0_w1_tag == 0) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireNestedDecision.Ok).v0 else null)) as io.prebindgen.covertest.payloadenums.WireDecision.Ok).v0 else 0L) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { (if ((p0_w1_tag == 0) && p0_w1_a0_f0_tag == 1) (requireNotNull((if (p0_w1_tag == 0) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireNestedDecision.Ok).v0 else null)) as io.prebindgen.covertest.payloadenums.WireDecision.Err).v0 else null) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { (if (p0_w1_tag == 1) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireNestedDecision.Err).v0 else null) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_nested_echo_76d3305595ab5ac4(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputa0_g0_g0_enumValue: io.prebindgen.covertest.payloadenums.WireDecision? = null
var outputenumValue: io.prebindgen.covertest.payloadenums.WireNestedDecision? = null
outputenumValue = when (returnCapture.s0) { 0 -> { outputa0_g0_g0_enumValue = when (returnCapture.s1) { 0 -> {  io.prebindgen.covertest.payloadenums.WireDecision.Ok(returnCapture.s2) }
1 -> {  io.prebindgen.covertest.payloadenums.WireDecision.Err(returnCapture.s3 as String) }
 else -> error("invalid native enum tag") }
 io.prebindgen.covertest.payloadenums.WireNestedDecision.Ok(requireNotNull(outputa0_g0_g0_enumValue)) }
1 -> {  io.prebindgen.covertest.payloadenums.WireNestedDecision.Err(returnCapture.s4 as String) }
 else -> error("invalid native enum tag") }

requireNotNull(outputenumValue)
    } finally { returnCapture.release() }
}

public fun wireNestedOutcome(kind: Int, onBindingError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireDecision> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.StringMessageHandler<io.prebindgen.covertest.payloadenums.WireDecision>): io.prebindgen.covertest.payloadenums.WireDecision = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long_String3_ErrorString1_72706b10c821cc64.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { kind } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_nested_outcome_267045d1d6aade55(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0 as String) }
    var outputenumValue: io.prebindgen.covertest.payloadenums.WireDecision? = null
outputenumValue = when (returnCapture.s0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireDecision.Ok(returnCapture.s1) }
1 -> {  io.prebindgen.covertest.payloadenums.WireDecision.Err(returnCapture.s2 as String) }
 else -> error("invalid native enum tag") }

requireNotNull(outputenumValue)
    } finally { returnCapture.release() }
}

public fun wireErrorOutcome(onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.payloadenums.WireErrorKindHandler<Unit>): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorInt_Long_String3_b35d95e7a73a9cfe.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_error_outcome_a68fd9e000421da6(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run { var outputenumValue: io.prebindgen.covertest.payloadenums.WireErrorKind? = null
outputenumValue = when (returnCapture.d0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireErrorKind.Code(returnCapture.d1) }
1 -> {  io.prebindgen.covertest.payloadenums.WireErrorKind.Message(returnCapture.d2 as String) }
 else -> error("invalid native enum tag") }

onError.run(requireNotNull(outputenumValue)) }
    Unit
    } finally { returnCapture.release() }
}

public fun wireEmptyEcho(value_c2da6115cd444ccd: io.prebindgen.covertest.payloadenums.WireEmpty, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireEmpty> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.payloadenums.WireEmpty = run {
    val p0_w1_tag = try { when (val value = requireNotNull(value_c2da6115cd444ccd)) { is io.prebindgen.covertest.payloadenums.WireEmpty.Unit -> 0
is io.prebindgen.covertest.payloadenums.WireEmpty.Tuple -> 1
is io.prebindgen.covertest.payloadenums.WireEmpty.Named -> 2 } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_tag } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_empty_echo_2615f17ea250b8db(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputenumValue: io.prebindgen.covertest.payloadenums.WireEmpty? = null
outputenumValue = when (nativeResult) { 0 -> {  io.prebindgen.covertest.payloadenums.WireEmpty.Unit }
1 -> {  io.prebindgen.covertest.payloadenums.WireEmpty.Tuple }
2 -> {  io.prebindgen.covertest.payloadenums.WireEmpty.Named }
 else -> error("invalid native enum tag") }

requireNotNull(outputenumValue)
    } finally { returnCapture.release() }
}

public fun wireOwnedBench(callback: io.prebindgen.covertest.payloadenums.WireOwnedEventToi64Callback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int, raw1: Long, raw2: Long, raw3: String?): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_a1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var callbackOutput0_a1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var callbackOutput0_enumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.payloadenums.WireOwnedEvent
        var callbackReturned: Long = 0L
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_enumValue = when (raw0 as Int) { 0 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Empty }
1 -> {             callbackOutput0_a1_g0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(raw1 as Long)
            callbackOutput0_a1_g1_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(raw2 as Long)
 io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair(requireNotNull(callbackOutput0_a1_g0_0), requireNotNull(callbackOutput0_a1_g1_0)) }
2 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text(raw3 as String) }
 else -> error("invalid native enum tag") }
            callbackArgument0_0 = requireNotNull(callbackOutput0_enumValue)
        } catch (failure: Throwable) {
if (raw0 as Int == 1) {             try { val owner = callbackOutput0_a1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw1 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_a1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw2 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
 }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            callbackReturned = callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (raw0 as Int == 1) {             try { val owner = callbackOutput0_a1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw1 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_a1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw2 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
 }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_owned_bench_43d5cbd62302dd8c(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun wireManualOwnedBench(callback: io.prebindgen.covertest.callbackreturns.CallbackReturnResourceCallbackReturnResourceToi64Callback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource
        var callbackOutput1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        lateinit var callbackArgument1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource
        var callbackReturned: Long = 0L
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackOutput0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(raw0 as Long)
            callbackArgument0_0 = requireNotNull(callbackOutput0_0)
            callbackOutput1_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(raw1 as Long)
            callbackArgument1_0 = requireNotNull(callbackOutput1_0)
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw1 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            callbackReturned = callback.run(callbackArgument0_0, callbackArgument1_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw1 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_manual_owned_bench_0b32cc235edec714(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun wireBenchTotal(value_c2da6115cd444ccd: io.prebindgen.covertest.payloadenums.WireBench, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w1_tag = try { when (val value = requireNotNull(value_c2da6115cd444ccd)) { is io.prebindgen.covertest.payloadenums.WireBench.Count -> 0
is io.prebindgen.covertest.payloadenums.WireBench.Pair -> 1 } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_tag } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { (if (p0_w1_tag == 0) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireBench.Count).v0 else 0uL).toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { (if (p0_w1_tag == 1) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireBench.Pair).left else 0uL).toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { (if (p0_w1_tag == 1) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireBench.Pair).right else 0uL).toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_bench_total_26bd797a8f4f3130(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun wireBenchScalarBulk(value_c2da6115cd444ccd: io.prebindgen.covertest.payloadenums.WireBench, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w3_tag = try { when (val value = requireNotNull(value_c2da6115cd444ccd)) { is io.prebindgen.covertest.payloadenums.WireBench.Count -> 0
is io.prebindgen.covertest.payloadenums.WireBench.Pair -> 1 } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.size)
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w3_tag })
if ((p0_w3_tag) == 0) {
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.offsets[1], io.prebindgen.covertest.InputValidation.value { (if (p0_w3_tag == 0) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireBench.Count).v0 else 0uL).toLong() })
}
if ((p0_w3_tag) == 1) {
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.offsets[2], io.prebindgen.covertest.InputValidation.value { (if (p0_w3_tag == 1) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireBench.Pair).left else 0uL).toLong() })
}
if ((p0_w3_tag) == 1) {
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.offsets[3], io.prebindgen.covertest.InputValidation.value { (if (p0_w3_tag == 1) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireBench.Pair).right else 0uL).toLong() })
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_bench_scalar_bulk_272bea75a88427d5(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun wireManualTotal(tag: Int, count: ULong, left: ULong, right: ULong, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { tag } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { count.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { left.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { right.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_manual_total_d00ad560a4738dfd(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun wireBenchBulk(values: kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireBench>, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r1a03ae7b416459bc.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r1a03ae7b416459bc.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r1a03ae7b416459bc.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val p0_w7_tag = when (val value = requireNotNull(p0_w7)) { is io.prebindgen.covertest.payloadenums.WireBench.Count -> 0
is io.prebindgen.covertest.payloadenums.WireBench.Pair -> 1 }
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.size) + io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w7_tag })
if ((p0_w7_tag) == 0) {
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.size) + io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.offsets[1], io.prebindgen.covertest.InputValidation.value { (if (p0_w7_tag == 0) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireBench.Count).v0 else 0uL).toLong() })
}
if ((p0_w7_tag) == 1) {
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.size) + io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.offsets[2], io.prebindgen.covertest.InputValidation.value { (if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireBench.Pair).left else 0uL).toLong() })
}
if ((p0_w7_tag) == 1) {
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.size) + io.prebindgen.covertest.BulkLayouts.ra76ccbbc825e1396.offsets[3], io.prebindgen.covertest.InputValidation.value { (if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireBench.Pair).right else 0uL).toLong() })
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_bench_bulk_6cebf744a52d53cf(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun wireBenchObjects(values: kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireBench>, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
val p0_w5_tag = when (val value = requireNotNull(p0_w5)) { is io.prebindgen.covertest.payloadenums.WireBench.Count -> 0
is io.prebindgen.covertest.payloadenums.WireBench.Pair -> 1 }
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w5_tag }, io.prebindgen.covertest.InputValidation.value { (if (p0_w5_tag == 0) (requireNotNull(p0_w5) as io.prebindgen.covertest.payloadenums.WireBench.Count).v0 else 0uL).toLong() }, io.prebindgen.covertest.InputValidation.value { (if (p0_w5_tag == 1) (requireNotNull(p0_w5) as io.prebindgen.covertest.payloadenums.WireBench.Pair).left else 0uL).toLong() }, io.prebindgen.covertest.InputValidation.value { (if (p0_w5_tag == 1) (requireNotNull(p0_w5) as io.prebindgen.covertest.payloadenums.WireBench.Pair).right else 0uL).toLong() }) } } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_bench_objects_30b3c3482b78fce2(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun wireCheckedNew(marker: Long, onBindingError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireCheckedEvent> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.StringMessageHandler<io.prebindgen.covertest.payloadenums.WireCheckedEvent>): io.prebindgen.covertest.payloadenums.WireCheckedEvent = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long_Long4_ErrorString1_3b413a8eca251644.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { marker } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_checked_new_622c9abaf30fd818(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputa1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputa1_g2_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputenumValue: io.prebindgen.covertest.payloadenums.WireCheckedEvent? = null
 if (returnCapture.s0 == 1) {             try { val owner = outputa1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputa1_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s3) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
 }
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0 as String) }
            var outputa1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputa1_g2_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputenumValue: io.prebindgen.covertest.payloadenums.WireCheckedEvent? = null
        try {
outputenumValue = when (returnCapture.s0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireCheckedEvent.Empty }
1 -> {             outputa1_g0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(returnCapture.s1)
            outputa1_g2_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(returnCapture.s3)
 io.prebindgen.covertest.payloadenums.WireCheckedEvent.Data(requireNotNull(outputa1_g0_0), returnCapture.s2, requireNotNull(outputa1_g2_0)) }
 else -> error("invalid native enum tag") }
            requireNotNull(outputenumValue)
        } catch (failure: Throwable) {
if (returnCapture.s0 == 1) {             try { val owner = outputa1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputa1_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s3) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun wireCheckedCallback(callback: io.prebindgen.covertest.VoidToWireCheckedEventCallback, onBindingError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.StringMessageHandler<Long>): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorString1_6b62d91b16305083.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: io.prebindgen.covertest.payloadenums.WireCheckedEvent
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val callback_return_w1_tag = when (val value = requireNotNull(callback_return_w1)) { is io.prebindgen.covertest.payloadenums.WireCheckedEvent.Empty -> 0
is io.prebindgen.covertest.payloadenums.WireCheckedEvent.Data -> 1 }
 run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { if (callback_return_w1_tag == 1) requireNotNull((if (callback_return_w1_tag == 1) (requireNotNull(callback_return_w1) as io.prebindgen.covertest.payloadenums.WireCheckedEvent.Data).first else null)) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.covertest.NativeHandle? = try { if (callback_return_w1_tag == 1) requireNotNull((if (callback_return_w1_tag == 1) (requireNotNull(callback_return_w1) as io.prebindgen.covertest.payloadenums.WireCheckedEvent.Data).last else null)) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; if (__pb_handle0 != null && __pb_handle1 != null && __pb_handle0.address == __pb_handle1.address) throw io.prebindgen.covertest.HandleValidationFailure("conflicting input aliases"); io.prebindgen.covertest.withHandleLocks(__pb_handle0, __pb_handle1) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_tag, __pb_handle0?.raw ?: 0L, (if (callback_return_w1_tag == 1) (requireNotNull(callback_return_w1) as io.prebindgen.covertest.payloadenums.WireCheckedEvent.Data).marker else 0L), __pb_handle1?.raw ?: 0L)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
__pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) }
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); when(val returnedValue0=callback_return_w1) { is io.prebindgen.covertest.payloadenums.WireCheckedEvent.Empty -> {  }
is io.prebindgen.covertest.payloadenums.WireCheckedEvent.Data -> { (returnedValue0.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue2 -> returnedHandles.add(returnedValue2); returnedOwners.add(returnedValue2) }
(returnedValue0.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue4 -> returnedHandles.add(returnedValue4); returnedOwners.add(returnedValue4) }
 }
 else -> {} }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_checked_callback_edc4df34d483e102(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0 as String) }
    nativeResult
    } finally { returnCapture.release() }
}

public fun wireReplyEmit(error_f6778bffb5e00e22: Boolean, callback: io.prebindgen.covertest.WireReplyCallback, onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.StringMessageHandler<Unit>): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorString1_6b62d91b16305083.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { error_f6778bffb5e00e22 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int, raw1: Long, raw2: ByteArray?, raw3: ByteArray?, raw4: Int, raw5: Int, raw6: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_g0_g0_g0_a0_g0_g0_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var callbackOutput0_g0_g0_g0_a0_value0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var callbackOutput0_g0_g0_g0_a0_value1: ByteArray? = null
var callbackOutput0_g0_g0_g0_a1_value0: ByteArray? = null
var callbackOutput0_g0_g0_g0_a1_value1: Int = 0
        var callbackArgument0_0: Int = 0
        var callbackArgument0_1: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var callbackArgument0_2: ByteArray? = null
        var callbackArgument0_3: ByteArray? = null
        var callbackArgument0_4: Int = 0
        var callbackArgument0_5: Long? = null
        try {
if (raw0 as Int == 0) {             callbackOutput0_g0_g0_g0_a0_g0_g0_g0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(raw1 as Long)
 callbackOutput0_g0_g0_g0_a0_value0 = requireNotNull(callbackOutput0_g0_g0_g0_a0_g0_g0_g0_0)
callbackOutput0_g0_g0_g0_a0_value1 = raw2 as ByteArray
 }
if (raw0 as Int == 1) {  callbackOutput0_g0_g0_g0_a1_value0 = raw3 as ByteArray
callbackOutput0_g0_g0_g0_a1_value1 = raw4 as Int
 }
            if (raw5 == 1) {
            }
            callbackArgument0_0 = raw0 as Int
            callbackArgument0_1 = callbackOutput0_g0_g0_g0_a0_value0
            callbackArgument0_2 = callbackOutput0_g0_g0_g0_a0_value1
            callbackArgument0_3 = callbackOutput0_g0_g0_g0_a1_value0
            callbackArgument0_4 = callbackOutput0_g0_g0_g0_a1_value1
            callbackArgument0_5 = if (raw5 == 1) raw6 as Long else null
        } catch (failure: Throwable) {
if (raw0 as Int == 0) {             try { val owner = callbackOutput0_g0_g0_g0_a0_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw1 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
 }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callback.run(callbackArgument0_0, callbackArgument0_1, callbackArgument0_2, callbackArgument0_3, callbackArgument0_4, callbackArgument0_5)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_reply_emit_07254f2bed536861(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0 as String) }
    Unit
    } finally { returnCapture.release() }
}

public fun wireCompositeOwnedNew(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireCompositeOwnedEvent> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.payloadenums.WireCompositeOwnedEvent = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long_Int6_ErrorEmpty_3c077f222803a2a9.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_composite_owned_new_c307a6a2a390617b(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputa1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputa1_g1_g0_enumValue: io.prebindgen.covertest.payloadenums.WireEvent? = null
var outputenumValue: io.prebindgen.covertest.payloadenums.WireCompositeOwnedEvent? = null
 if (returnCapture.s0 == 1) {             try { val owner = outputa1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
 }
        return@run onError.run(returnCapture.message) }

            var outputa1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputa1_g1_g0_enumValue: io.prebindgen.covertest.payloadenums.WireEvent? = null
var outputenumValue: io.prebindgen.covertest.payloadenums.WireCompositeOwnedEvent? = null
        try {
outputenumValue = when (returnCapture.s0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireCompositeOwnedEvent.Empty }
1 -> {             outputa1_g0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(returnCapture.s1)
outputa1_g1_g0_enumValue = when (returnCapture.s2) { 0 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Idle }
1 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Count((returnCapture.s3).toULong()) }
2 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Packet((returnCapture.s4).toULong(), returnCapture.s5 as ByteArray) }
3 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyTuple }
4 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyRecord }
 else -> error("invalid native enum tag") }
 io.prebindgen.covertest.payloadenums.WireCompositeOwnedEvent.Data(requireNotNull(outputa1_g0_0), requireNotNull(outputa1_g1_g0_enumValue)) }
 else -> error("invalid native enum tag") }
            requireNotNull(outputenumValue)
        } catch (failure: Throwable) {
if (returnCapture.s0 == 1) {             try { val owner = outputa1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun wireNicheRows(values: kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireNicheEvent>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireNicheEvent>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireNicheEvent> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_e9cbead200b49a58()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.re9cbead200b49a58.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r9667197aedac566a)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.re9cbead200b49a58.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val p0_w7_tag = when (val value = requireNotNull(p0_w7)) { is io.prebindgen.covertest.payloadenums.WireNicheEvent.Empty -> 0
is io.prebindgen.covertest.payloadenums.WireNicheEvent.Data -> 1 }
val __pb_p0_w7_a1_f0_some_field0 = if ((p0_w7_tag == 1) && ((if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireNicheEvent.Data).small else null) != null)) (if (p0_w7_tag == 1 && true) requireNotNull((if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireNicheEvent.Data).small else null)).enabled else false) else (false)
val __pb_p0_w7_a1_f0_some_field1 = if ((p0_w7_tag == 1) && ((if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireNicheEvent.Data).small else null) != null)) (if (p0_w7_tag == 1 && true) requireNotNull((if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireNicheEvent.Data).small else null)).resource else null) else (null)
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.size) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w7_tag })
if ((p0_w7_tag) == 1) {
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.size) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[1], io.prebindgen.covertest.InputValidation.value { (if ((p0_w7_tag == 1) && ((if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireNicheEvent.Data).small else null) != null)) 1 else 0) })
}
if ((p0_w7_tag) == 1 && ((if ((p0_w7_tag == 1) && ((if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireNicheEvent.Data).small else null) != null)) 1 else 0)) == 1) {
p0_w1_writer.byte((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.size) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[2], (if (io.prebindgen.covertest.InputValidation.value { if ((p0_w7_tag == 1) && ((if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireNicheEvent.Data).small else null) != null)) if (p0_w7_tag == 1 && true) __pb_p0_w7_a1_f0_some_field0 else false else false }) 1 else 0).toByte())
}
if ((p0_w7_tag) == 1 && ((if ((p0_w7_tag == 1) && ((if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireNicheEvent.Data).small else null) != null)) 1 else 0)) == 1) {
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.size) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[3], io.prebindgen.covertest.InputValidation.value { if ((p0_w7_tag == 1) && ((if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireNicheEvent.Data).small else null) != null)) if (p0_w7_tag == 1 && true) if (p0_w7_tag == 1 && true) requireNotNull(__pb_p0_w7_a1_f0_some_field1) else null else null else null }, true, 0)
}
if ((p0_w7_tag) == 1) {
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.size) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[4], io.prebindgen.covertest.InputValidation.value { if ((p0_w7_tag == 1) && ((if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireNicheEvent.Data).wide else null) != null)) requireNotNull(if (p0_w7_tag == 1 && true) requireNotNull((if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireNicheEvent.Data).wide else null)) else null) { "invalid option payload" } else null }, true, 0)
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_niche_rows_04e85916466ca1ef(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.payloadenums.WireNicheEvent>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.re9cbead200b49a58.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_a1_g0_g0_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_a1_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputbulkItem_enumValue: io.prebindgen.covertest.payloadenums.WireNicheEvent? = null

if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[0])) as Int == 1) {             if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[1])) == 1) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[3])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[4])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[4])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
 }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.payloadenums.WireNicheEvent>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.re9cbead200b49a58.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.size
        var outputbulkItem_a1_g0_g0_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_a1_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputbulkItem_enumValue: io.prebindgen.covertest.payloadenums.WireNicheEvent? = null

outputbulkItem_enumValue = when ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[0])) as Int) { 0 -> {  io.prebindgen.covertest.payloadenums.WireNicheEvent.Empty }
1 -> {             if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[1])) == 1) {
            outputbulkItem_a1_g0_g0_g1_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[3])) as Long))
            }
            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[4])) != 0L) {
            outputbulkItem_a1_g1_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[4])) as Long))
            }
 io.prebindgen.covertest.payloadenums.WireNicheEvent.Data(if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[1])) == 1) io.prebindgen.covertest.payloadenums.WireOptionalOwner(((requireNotNull(outputbulkReader).byte((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[2]).toInt() != 0)) as Boolean, requireNotNull(outputbulkItem_a1_g0_g0_g1_0)) else null, if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[4])) != 0L) requireNotNull(outputbulkItem_a1_g1_g0_0) else null) }
 else -> error("invalid native enum tag") }

outputbulkList!!.add(requireNotNull(outputbulkItem_enumValue))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.re9cbead200b49a58.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_a1_g0_g0_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_a1_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputbulkItem_enumValue: io.prebindgen.covertest.payloadenums.WireNicheEvent? = null

if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[0])) as Int == 1) {             if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[1])) == 1) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[3])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[4])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9667197aedac566a.offsets[4])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
 }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun wireOwnedNew(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireOwnedEvent> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.payloadenums.WireOwnedEvent = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long_Long4_ErrorEmpty_1b22ee7c534cba27.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_owned_new_02f3a76c6198a50a(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputa1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputa1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputenumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null
 if (returnCapture.s0 == 1) {             try { val owner = outputa1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputa1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
 }
        return@run onError.run(returnCapture.message) }

            var outputa1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputa1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputenumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null
        try {
outputenumValue = when (returnCapture.s0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Empty }
1 -> {             outputa1_g0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(returnCapture.s1)
            outputa1_g1_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(returnCapture.s2)
 io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair(requireNotNull(outputa1_g0_0), requireNotNull(outputa1_g1_0)) }
2 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text(returnCapture.s3 as String) }
 else -> error("invalid native enum tag") }
            requireNotNull(outputenumValue)
        } catch (failure: Throwable) {
if (returnCapture.s0 == 1) {             try { val owner = outputa1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputa1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun wireOwnedEcho(value_c2da6115cd444ccd: io.prebindgen.covertest.payloadenums.WireOwnedEvent, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireOwnedEvent> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.payloadenums.WireOwnedEvent = run {
    val p0_w1_tag = try { when (val value = requireNotNull(value_c2da6115cd444ccd)) { is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Empty -> 0
is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair -> 1
is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text -> 2 } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Int_Long_Long4_ErrorEmpty_1b22ee7c534cba27.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { if (p0_w1_tag == 1) requireNotNull((if (p0_w1_tag == 1) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair).first else null)) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.covertest.NativeHandle? = try { if (p0_w1_tag == 1) requireNotNull((if (p0_w1_tag == 1) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair).second else null)) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; if (__pb_handle0 != null && __pb_handle1 != null && __pb_handle0.address == __pb_handle1.address) throw io.prebindgen.covertest.HandleValidationFailure("conflicting input aliases"); io.prebindgen.covertest.withHandleLocks(__pb_handle0, __pb_handle1) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_tag } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_handle1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { (if (p0_w1_tag == 2) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text).v0 else null) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_owned_echo_8afdb4c8af3ccb4b(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; __pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) } } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; __pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) } } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputa1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputa1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputenumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null
 if (returnCapture.s0 == 1) {             try { val owner = outputa1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputa1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
 }
        return@run onError.run(returnCapture.message) }

            var outputa1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputa1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputenumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null
        try {
outputenumValue = when (returnCapture.s0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Empty }
1 -> {             outputa1_g0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(returnCapture.s1)
            outputa1_g1_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(returnCapture.s2)
 io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair(requireNotNull(outputa1_g0_0), requireNotNull(outputa1_g1_0)) }
2 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text(returnCapture.s3 as String) }
 else -> error("invalid native enum tag") }
            requireNotNull(outputenumValue)
        } catch (failure: Throwable) {
if (returnCapture.s0 == 1) {             try { val owner = outputa1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputa1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun wireOwnedCallback(callback: io.prebindgen.covertest.payloadenums.WireOwnedEventToWireOwnedEventCallback, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireOwnedEvent> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.payloadenums.WireOwnedEvent = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long_Long4_ErrorEmpty_1b22ee7c534cba27.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int, raw1: Long, raw2: Long, raw3: String?): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_a1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var callbackOutput0_a1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var callbackOutput0_enumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.payloadenums.WireOwnedEvent
        lateinit var callbackReturned: io.prebindgen.covertest.payloadenums.WireOwnedEvent
var callbackReturnReady = false
        try { var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_enumValue = when (raw0 as Int) { 0 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Empty }
1 -> {             callbackOutput0_a1_g0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(raw1 as Long)
            callbackOutput0_a1_g1_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(raw2 as Long)
 io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair(requireNotNull(callbackOutput0_a1_g0_0), requireNotNull(callbackOutput0_a1_g1_0)) }
2 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text(raw3 as String) }
 else -> error("invalid native enum tag") }
            callbackArgument0_0 = requireNotNull(callbackOutput0_enumValue)
        } catch (failure: Throwable) {
if (raw0 as Int == 1) {             try { val owner = callbackOutput0_a1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw1 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_a1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw2 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
 }
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
if (raw0 as Int == 1) {             try { val owner = callbackOutput0_a1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw1 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_a1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw2 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
 }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        } } catch (failure: Throwable) { if (callbackReturnReady) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); when(val returnedValue0=callbackReturned) { is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Empty -> {  }
is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair -> { (returnedValue0.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue2 -> returnedHandles.add(returnedValue2); returnedOwners.add(returnedValue2) }
(returnedValue0.second as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue4 -> returnedHandles.add(returnedValue4); returnedOwners.add(returnedValue4) }
 }
is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text -> {  }
 else -> {} }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() } }; throw failure }
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val callback_return_w1_tag = when (val value = requireNotNull(callback_return_w1)) { is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Empty -> 0
is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair -> 1
is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text -> 2 }
 run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { if (callback_return_w1_tag == 1) requireNotNull((if (callback_return_w1_tag == 1) (requireNotNull(callback_return_w1) as io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair).first else null)) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.covertest.NativeHandle? = try { if (callback_return_w1_tag == 1) requireNotNull((if (callback_return_w1_tag == 1) (requireNotNull(callback_return_w1) as io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair).second else null)) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; if (__pb_handle0 != null && __pb_handle1 != null && __pb_handle0.address == __pb_handle1.address) throw io.prebindgen.covertest.HandleValidationFailure("conflicting input aliases"); io.prebindgen.covertest.withHandleLocks(__pb_handle0, __pb_handle1) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_tag, __pb_handle0?.raw ?: 0L, __pb_handle1?.raw ?: 0L, (if (callback_return_w1_tag == 2) (requireNotNull(callback_return_w1) as io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text).v0 else null))
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
__pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) }
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); when(val returnedValue0=callback_return_w1) { is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Empty -> {  }
is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair -> { (returnedValue0.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue2 -> returnedHandles.add(returnedValue2); returnedOwners.add(returnedValue2) }
(returnedValue0.second as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue4 -> returnedHandles.add(returnedValue4); returnedOwners.add(returnedValue4) }
 }
is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text -> {  }
 else -> {} }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_owned_callback_45a85256857f9fb7(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputa1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputa1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputenumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null
 if (returnCapture.s0 == 1) {             try { val owner = outputa1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputa1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
 }
        return@run onError.run(returnCapture.message) }

            var outputa1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputa1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputenumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null
        try {
outputenumValue = when (returnCapture.s0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Empty }
1 -> {             outputa1_g0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(returnCapture.s1)
            outputa1_g1_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(returnCapture.s2)
 io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair(requireNotNull(outputa1_g0_0), requireNotNull(outputa1_g1_0)) }
2 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text(returnCapture.s3 as String) }
 else -> error("invalid native enum tag") }
            requireNotNull(outputenumValue)
        } catch (failure: Throwable) {
if (returnCapture.s0 == 1) {             try { val owner = outputa1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputa1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun wireOwnedRows(values: kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireOwnedEvent>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireOwnedEvent>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireOwnedEvent> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_c0f3ea7ca8d547d0()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rc0f3ea7ca8d547d0.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rc0f3ea7ca8d547d0.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val p0_w7_tag = when (val value = requireNotNull(p0_w7)) { is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Empty -> 0
is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair -> 1
is io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text -> 2 }
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w7_tag })
if ((p0_w7_tag) == 1) {
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[1], io.prebindgen.covertest.InputValidation.value { if (p0_w7_tag == 1) (if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair).first else null) else null }, true, 0)
}
if ((p0_w7_tag) == 1) {
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[2], io.prebindgen.covertest.InputValidation.value { if (p0_w7_tag == 1) (if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair).second else null) else null }, true, 0)
}
if ((p0_w7_tag) == 2) {
p0_w1_writer.text((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[3], io.prebindgen.covertest.InputValidation.value { (if (p0_w7_tag == 2) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text).v0 else null) })
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_owned_rows_a0a5a7fb0fa9441b(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.payloadenums.WireOwnedEvent>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rc0f3ea7ca8d547d0.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_a1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_a1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputbulkItem_enumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null

if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[0])) as Int == 1) {             try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.payloadenums.WireOwnedEvent>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rc0f3ea7ca8d547d0.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size
        var outputbulkItem_a1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_a1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputbulkItem_enumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null

outputbulkItem_enumValue = when ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[0])) as Int) { 0 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Empty }
1 -> {             outputbulkItem_a1_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[1])) as Long))
            outputbulkItem_a1_g1_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[2])) as Long))
 io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair(requireNotNull(outputbulkItem_a1_g0_0), requireNotNull(outputbulkItem_a1_g1_0)) }
2 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text((requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[3])) as String) }
 else -> error("invalid native enum tag") }

outputbulkList!!.add(requireNotNull(outputbulkItem_enumValue))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rc0f3ea7ca8d547d0.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_a1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_a1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputbulkItem_enumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null

if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[0])) as Int == 1) {             try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun wireOwnedRowsNew(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireOwnedEvent>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireOwnedEvent> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_owned_rows_new_a77a450e1aea3cc5(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.payloadenums.WireOwnedEvent>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rc0f3ea7ca8d547d0.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_a1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_a1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputbulkItem_enumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null

if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[0])) as Int == 1) {             try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.payloadenums.WireOwnedEvent>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rc0f3ea7ca8d547d0.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size
        var outputbulkItem_a1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_a1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputbulkItem_enumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null

outputbulkItem_enumValue = when ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[0])) as Int) { 0 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Empty }
1 -> {             outputbulkItem_a1_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[1])) as Long))
            outputbulkItem_a1_g1_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[2])) as Long))
 io.prebindgen.covertest.payloadenums.WireOwnedEvent.Pair(requireNotNull(outputbulkItem_a1_g0_0), requireNotNull(outputbulkItem_a1_g1_0)) }
2 -> {  io.prebindgen.covertest.payloadenums.WireOwnedEvent.Text((requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[3])) as String) }
 else -> error("invalid native enum tag") }

outputbulkList!!.add(requireNotNull(outputbulkItem_enumValue))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rc0f3ea7ca8d547d0.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_a1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_a1_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputbulkItem_enumValue: io.prebindgen.covertest.payloadenums.WireOwnedEvent? = null

if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[0])) as Int == 1) {             try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r5e9e672aef1dc190.offsets[2])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun wireRecoveryEcho(value_c2da6115cd444ccd: io.prebindgen.covertest.payloadenums.WireRecoveryConfig, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireRecoveryConfig> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.payloadenums.WireRecoveryConfig = run {
    val p0_w1_f0_some_tag = try { if (value_c2da6115cd444ccd.mode != null) (when (val value = requireNotNull(requireNotNull(value_c2da6115cd444ccd.mode))) { is io.prebindgen.covertest.payloadenums.WireRecoveryMode.PeriodicQueries -> 0
is io.prebindgen.covertest.payloadenums.WireRecoveryMode.Heartbeat -> 1 }) else (0) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Int_Long_Int4_ErrorEmpty_36006a8dacac5e29.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { if (value_c2da6115cd444ccd.mode != null) run { val value = p0_w1_f0_some_tag; require(value != -2) { "invalid option payload" }; value } else -2 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (value_c2da6115cd444ccd.mode != null) (if (p0_w1_f0_some_tag == 0) (requireNotNull(requireNotNull(value_c2da6115cd444ccd.mode)) as io.prebindgen.covertest.payloadenums.WireRecoveryMode.PeriodicQueries).v0 else 0uL).toLong() else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { (if (value_c2da6115cd444ccd.retention_period != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { if (value_c2da6115cd444ccd.retention_period != null) requireNotNull(value_c2da6115cd444ccd.retention_period).toLong() else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_recovery_echo_9cff7a81f910e51a(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputg0_g0_enumValue: io.prebindgen.covertest.payloadenums.WireRecoveryMode? = null
            if (returnCapture.s0 != -2) {
outputg0_g0_enumValue = when (returnCapture.s0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireRecoveryMode.PeriodicQueries((returnCapture.s1).toULong()) }
1 -> {  io.prebindgen.covertest.payloadenums.WireRecoveryMode.Heartbeat }
 else -> error("invalid native enum tag") }
            }
            if (returnCapture.s2 == 1) {
            }

io.prebindgen.covertest.payloadenums.WireRecoveryConfig(if (returnCapture.s0 != -2) requireNotNull(outputg0_g0_enumValue) else null, if (returnCapture.s2 == 1) (returnCapture.s3).toULong() else null)
    } finally { returnCapture.release() }
}

public fun wireTimestamps(values: kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireTimestampStackRecord>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireTimestampStackRecord>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireTimestampStackRecord> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rb9c3ec472814fefd.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rb9c3ec472814fefd.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rb9c3ec472814fefd.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val p0_w7_f1_tag = when (val value = requireNotNull(p0_w7.timestamp)) { is io.prebindgen.covertest.payloadenums.WireInstrumentationTimestamp.Uhlc -> 0
is io.prebindgen.covertest.payloadenums.WireInstrumentationTimestamp.Custom -> 1 }
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.size) + io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w7.point.wireValue })
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.size) + io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w7_f1_tag })
if ((p0_w7_f1_tag) == 0) {
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.size) + io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.offsets[2], io.prebindgen.covertest.InputValidation.value { if (p0_w7_f1_tag == 0) requireNotNull((if (p0_w7_f1_tag == 0) (requireNotNull(p0_w7.timestamp) as io.prebindgen.covertest.payloadenums.WireInstrumentationTimestamp.Uhlc).v0 else null)).ntp64.toLong() else 0L })
}
if ((p0_w7_f1_tag) == 0) {
p0_w1_writer.bytes((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.size) + io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.offsets[3], io.prebindgen.covertest.InputValidation.value { if (p0_w7_f1_tag == 0) requireNotNull((if (p0_w7_f1_tag == 0) (requireNotNull(p0_w7.timestamp) as io.prebindgen.covertest.payloadenums.WireInstrumentationTimestamp.Uhlc).v0 else null)).id else null })
}
if ((p0_w7_f1_tag) == 1) {
p0_w1_writer.bytes((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.size) + io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.offsets[4], io.prebindgen.covertest.InputValidation.value { (if (p0_w7_f1_tag == 1) (requireNotNull(p0_w7.timestamp) as io.prebindgen.covertest.payloadenums.WireInstrumentationTimestamp.Custom).v0 else null) })
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_timestamps_ef5a88b190afbdc7(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.payloadenums.WireTimestampStackRecord>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rb9c3ec472814fefd.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.size
var outputbulkItem_g1_g0_enumValue: io.prebindgen.covertest.payloadenums.WireInstrumentationTimestamp? = null

outputbulkItem_g1_g0_enumValue = when ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.offsets[1])) as Int) { 0 -> {  io.prebindgen.covertest.payloadenums.WireInstrumentationTimestamp.Uhlc(io.prebindgen.covertest.payloadenums.WireTimestamp(((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.offsets[2])) as Long).toULong(), (requireNotNull(outputbulkReader).bytes((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.offsets[3])) as ByteArray)) }
1 -> {  io.prebindgen.covertest.payloadenums.WireInstrumentationTimestamp.Custom((requireNotNull(outputbulkReader).bytes((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.offsets[4])) as ByteArray) }
 else -> error("invalid native enum tag") }

outputbulkList!!.add(io.prebindgen.covertest.payloadenums.WireTimestampStackRecord(io.prebindgen.covertest.payloadenums.WireInterceptionPoint.fromWire((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r8d54fa6b8204159f.offsets[0])) as Int), requireNotNull(outputbulkItem_g1_g0_enumValue)))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun wireEventRows(values: kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireEvent>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireEvent>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireEvent> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r6535e286a16daa73.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r6535e286a16daa73.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r6535e286a16daa73.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val p0_w7_tag = when (val value = requireNotNull(p0_w7)) { is io.prebindgen.covertest.payloadenums.WireEvent.Idle -> 0
is io.prebindgen.covertest.payloadenums.WireEvent.Count -> 1
is io.prebindgen.covertest.payloadenums.WireEvent.Packet -> 2
is io.prebindgen.covertest.payloadenums.WireEvent.EmptyTuple -> 3
is io.prebindgen.covertest.payloadenums.WireEvent.EmptyRecord -> 4 }
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.size) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w7_tag })
if ((p0_w7_tag) == 1) {
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.size) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[1], io.prebindgen.covertest.InputValidation.value { (if (p0_w7_tag == 1) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireEvent.Count).v0 else 0uL).toLong() })
}
if ((p0_w7_tag) == 2) {
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.size) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[2], io.prebindgen.covertest.InputValidation.value { (if (p0_w7_tag == 2) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireEvent.Packet).id else 0uL).toLong() })
}
if ((p0_w7_tag) == 2) {
p0_w1_writer.bytes((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.size) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[3], io.prebindgen.covertest.InputValidation.value { (if (p0_w7_tag == 2) (requireNotNull(p0_w7) as io.prebindgen.covertest.payloadenums.WireEvent.Packet).bytes else null) })
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_event_rows_9b5b5f63438b5d4d(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.payloadenums.WireEvent>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r6535e286a16daa73.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.size
var outputbulkItem_enumValue: io.prebindgen.covertest.payloadenums.WireEvent? = null

outputbulkItem_enumValue = when ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[0])) as Int) { 0 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Idle }
1 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Count(((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[1])) as Long).toULong()) }
2 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Packet(((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[2])) as Long).toULong(), (requireNotNull(outputbulkReader).bytes((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[3])) as ByteArray) }
3 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyTuple }
4 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyRecord }
 else -> error("invalid native enum tag") }

outputbulkList!!.add(requireNotNull(outputbulkItem_enumValue))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun wireEventRowsCallback(callback: io.prebindgen.covertest.WireEventVecToWireEventVecCallback, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireEvent>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireEvent> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.payloadenums.WireEvent>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireEvent>
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.payloadenums.WireEvent>
        try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r6535e286a16daa73.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.size
var callbackOutput0_bulkItem_enumValue: io.prebindgen.covertest.payloadenums.WireEvent? = null

callbackOutput0_bulkItem_enumValue = when ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[0])) as Int) { 0 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Idle }
1 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Count(((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[1])) as Long).toULong()) }
2 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Packet(((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[2])) as Long).toULong(), (requireNotNull(callbackOutput0_bulkReader).bytes((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[3])) as ByteArray) }
3 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyTuple }
4 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyRecord }
 else -> error("invalid native enum tag") }

callbackOutput0_bulkList!!.add(requireNotNull(callbackOutput0_bulkItem_enumValue))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
 val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.let { callback_return_w1_snapshot -> kotlin.Array<kotlin.Any?>(callback_return_w1_snapshot.size) { callback_return_w1_index -> val callback_return_w5 = callback_return_w1_snapshot[callback_return_w1_index]
val callback_return_w5_tag = when (val value = requireNotNull(callback_return_w5)) { is io.prebindgen.covertest.payloadenums.WireEvent.Idle -> 0
is io.prebindgen.covertest.payloadenums.WireEvent.Count -> 1
is io.prebindgen.covertest.payloadenums.WireEvent.Packet -> 2
is io.prebindgen.covertest.payloadenums.WireEvent.EmptyTuple -> 3
is io.prebindgen.covertest.payloadenums.WireEvent.EmptyRecord -> 4 }
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { callback_return_w5_tag }, io.prebindgen.covertest.InputValidation.value { (if (callback_return_w5_tag == 1) (requireNotNull(callback_return_w5) as io.prebindgen.covertest.payloadenums.WireEvent.Count).v0 else 0uL).toLong() }, io.prebindgen.covertest.InputValidation.value { (if (callback_return_w5_tag == 2) (requireNotNull(callback_return_w5) as io.prebindgen.covertest.payloadenums.WireEvent.Packet).id else 0uL).toLong() }, io.prebindgen.covertest.InputValidation.value { (if (callback_return_w5_tag == 2) (requireNotNull(callback_return_w5) as io.prebindgen.covertest.payloadenums.WireEvent.Packet).bytes else null) }) } } else null
 run { val callbackReturnWire = callback_return_w1_elements
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_event_rows_callback_5bf05a04cceefdff(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.payloadenums.WireEvent>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r6535e286a16daa73.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.size
var outputbulkItem_enumValue: io.prebindgen.covertest.payloadenums.WireEvent? = null

outputbulkItem_enumValue = when ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[0])) as Int) { 0 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Idle }
1 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Count(((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[1])) as Long).toULong()) }
2 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Packet(((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[2])) as Long).toULong(), (requireNotNull(outputbulkReader).bytes((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r97c4ef67c98cf0bf.offsets[3])) as ByteArray) }
3 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyTuple }
4 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyRecord }
 else -> error("invalid native enum tag") }

outputbulkList!!.add(requireNotNull(outputbulkItem_enumValue))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun <R> wireBareEcho(valueVariant: Int, valueCountV0: Long, valuePacketX: ULong, valuePacketY: Double, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (tag: Int, count: Long, packetX: ULong, packetY: Double) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long_Long4_ErrorEmpty_ae385d2af0c9940c.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { valueVariant } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { valueCountV0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { valuePacketX.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { valuePacketY } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_bare_echo_e62a438d887d3bb2(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputa0_value0: Long = 0L
var outputa1_value0: ULong = 0uL
var outputa1_value1: Double = 0.0
if (returnCapture.s0 == 0) {  outputa0_value0 = returnCapture.s1
 }
if (returnCapture.s0 == 1) {  outputa1_value0 = (returnCapture.s2).toULong()
outputa1_value1 = returnCapture.s3
 }

build(returnCapture.s0, outputa0_value0, outputa1_value0, outputa1_value1)
    } finally { returnCapture.release() }
}

public fun wireEventEcho(value_c2da6115cd444ccd: io.prebindgen.covertest.payloadenums.WireEvent, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireEvent> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.payloadenums.WireEvent = run {
    val p0_w1_tag = try { when (val value = requireNotNull(value_c2da6115cd444ccd)) { is io.prebindgen.covertest.payloadenums.WireEvent.Idle -> 0
is io.prebindgen.covertest.payloadenums.WireEvent.Count -> 1
is io.prebindgen.covertest.payloadenums.WireEvent.Packet -> 2
is io.prebindgen.covertest.payloadenums.WireEvent.EmptyTuple -> 3
is io.prebindgen.covertest.payloadenums.WireEvent.EmptyRecord -> 4 } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Int_Long_Long4_ErrorEmpty_26bdbe3fb576cb52.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_tag } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { (if (p0_w1_tag == 1) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireEvent.Count).v0 else 0uL).toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { (if (p0_w1_tag == 2) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireEvent.Packet).id else 0uL).toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { (if (p0_w1_tag == 2) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireEvent.Packet).bytes else null) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_event_echo_9995546291e8a79d(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputenumValue: io.prebindgen.covertest.payloadenums.WireEvent? = null
outputenumValue = when (returnCapture.s0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Idle }
1 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Count((returnCapture.s1).toULong()) }
2 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Packet((returnCapture.s2).toULong(), returnCapture.s3 as ByteArray) }
3 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyTuple }
4 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyRecord }
 else -> error("invalid native enum tag") }

requireNotNull(outputenumValue)
    } finally { returnCapture.release() }
}

public fun wireEventCallback(callback: io.prebindgen.covertest.payloadenums.WireEventToWireEventCallback, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireEvent> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.payloadenums.WireEvent = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long_Long4_ErrorEmpty_26bdbe3fb576cb52.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int, raw1: Long, raw2: Long, raw3: ByteArray?): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_enumValue: io.prebindgen.covertest.payloadenums.WireEvent? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.payloadenums.WireEvent
        lateinit var callbackReturned: io.prebindgen.covertest.payloadenums.WireEvent
        try {
callbackOutput0_enumValue = when (raw0 as Int) { 0 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Idle }
1 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Count((raw1 as Long).toULong()) }
2 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Packet((raw2 as Long).toULong(), raw3 as ByteArray) }
3 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyTuple }
4 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyRecord }
 else -> error("invalid native enum tag") }
            callbackArgument0_0 = requireNotNull(callbackOutput0_enumValue)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
 val callback_return_w1_tag = when (val value = requireNotNull(callback_return_w1)) { is io.prebindgen.covertest.payloadenums.WireEvent.Idle -> 0
is io.prebindgen.covertest.payloadenums.WireEvent.Count -> 1
is io.prebindgen.covertest.payloadenums.WireEvent.Packet -> 2
is io.prebindgen.covertest.payloadenums.WireEvent.EmptyTuple -> 3
is io.prebindgen.covertest.payloadenums.WireEvent.EmptyRecord -> 4 }
 run { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_tag, (if (callback_return_w1_tag == 1) (requireNotNull(callback_return_w1) as io.prebindgen.covertest.payloadenums.WireEvent.Count).v0 else 0uL).toLong(), (if (callback_return_w1_tag == 2) (requireNotNull(callback_return_w1) as io.prebindgen.covertest.payloadenums.WireEvent.Packet).id else 0uL).toLong(), (if (callback_return_w1_tag == 2) (requireNotNull(callback_return_w1) as io.prebindgen.covertest.payloadenums.WireEvent.Packet).bytes else null))
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_event_callback_a4bd07b5c8093f25(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputenumValue: io.prebindgen.covertest.payloadenums.WireEvent? = null
outputenumValue = when (returnCapture.s0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Idle }
1 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Count((returnCapture.s1).toULong()) }
2 -> {  io.prebindgen.covertest.payloadenums.WireEvent.Packet((returnCapture.s2).toULong(), returnCapture.s3 as ByteArray) }
3 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyTuple }
4 -> {  io.prebindgen.covertest.payloadenums.WireEvent.EmptyRecord }
 else -> error("invalid native enum tag") }

requireNotNull(outputenumValue)
    } finally { returnCapture.release() }
}

public fun wireResultEcho(value_c2da6115cd444ccd: io.prebindgen.covertest.payloadenums.WireDecision, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireDecision> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.payloadenums.WireDecision = run {
    val p0_w1_tag = try { when (val value = requireNotNull(value_c2da6115cd444ccd)) { is io.prebindgen.covertest.payloadenums.WireDecision.Ok -> 0
is io.prebindgen.covertest.payloadenums.WireDecision.Err -> 1 } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Int_Long_String3_ErrorEmpty_88f96e73723eb19e.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_tag } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { (if (p0_w1_tag == 0) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireDecision.Ok).v0 else 0L) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { (if (p0_w1_tag == 1) (requireNotNull(value_c2da6115cd444ccd) as io.prebindgen.covertest.payloadenums.WireDecision.Err).v0 else null) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_result_echo_c1fb4e39160aa164(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputenumValue: io.prebindgen.covertest.payloadenums.WireDecision? = null
outputenumValue = when (returnCapture.s0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireDecision.Ok(returnCapture.s1) }
1 -> {  io.prebindgen.covertest.payloadenums.WireDecision.Err(returnCapture.s2 as String) }
 else -> error("invalid native enum tag") }

requireNotNull(outputenumValue)
    } finally { returnCapture.release() }
}

public fun wireResultCallback(callback: io.prebindgen.covertest.payloadenums.ResultToResultCallback, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.payloadenums.WireDecision> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.payloadenums.WireDecision = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long_String3_ErrorEmpty_88f96e73723eb19e.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int, raw1: String?): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_enumValue: io.prebindgen.covertest.payloadenums.WireNotice? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.payloadenums.WireNotice
        lateinit var callbackReturned: io.prebindgen.covertest.payloadenums.WireDecision
        try {
callbackOutput0_enumValue = when (raw0 as Int) { 0 -> {  io.prebindgen.covertest.payloadenums.WireNotice.Ok(kotlin.Unit) }
1 -> {  io.prebindgen.covertest.payloadenums.WireNotice.Err(raw1 as String) }
 else -> error("invalid native enum tag") }
            callbackArgument0_0 = requireNotNull(callbackOutput0_enumValue)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
 val callback_return_w1_tag = when (val value = requireNotNull(callback_return_w1)) { is io.prebindgen.covertest.payloadenums.WireDecision.Ok -> 0
is io.prebindgen.covertest.payloadenums.WireDecision.Err -> 1 }
 run { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_tag, (if (callback_return_w1_tag == 0) (requireNotNull(callback_return_w1) as io.prebindgen.covertest.payloadenums.WireDecision.Ok).v0 else 0L), (if (callback_return_w1_tag == 1) (requireNotNull(callback_return_w1) as io.prebindgen.covertest.payloadenums.WireDecision.Err).v0 else null))
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_wire_result_callback_5f00a3d27cff8113(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputenumValue: io.prebindgen.covertest.payloadenums.WireDecision? = null
outputenumValue = when (returnCapture.s0) { 0 -> {  io.prebindgen.covertest.payloadenums.WireDecision.Ok(returnCapture.s1) }
1 -> {  io.prebindgen.covertest.payloadenums.WireDecision.Err(returnCapture.s2 as String) }
 else -> error("invalid native enum tag") }

requireNotNull(outputenumValue)
    } finally { returnCapture.release() }
}
