@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.callbacks

public fun `callbackRowsRef`(p0_callback: io.prebindgen.covertest.CallbackObservedRowVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackObservedRow>? = null
var callbackOutput0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackObservedRow>
        try {
callbackOutput0_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_list = kotlin.collections.mutableListOf()
for (callbackOutput0_rawRow in (raw0 as kotlin.Array<*>)) {
val callbackOutput0__row = callbackOutput0_rawRow as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_cleanups.add { callbackOutput0_initialFailure -> var callbackOutput0_failure = callbackOutput0_initialFailure; try {             try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }; callbackOutput0_failure }
            callbackOutput0_item_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[0] as Long)

callbackOutput0_list.add(io.prebindgen.covertest.callbacks.CallbackObservedRow(requireNotNull(callbackOutput0_item_g0_0), callbackOutput0__row[1] as String))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
        } catch (failure: Throwable) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }

} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_ref_e8fc186edb0f16d5(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowReturnCleanupFailure`(p0_callback: io.prebindgen.covertest.CallbackUniqueRowVecToCallbackReturnResourceCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): Long {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackUniqueRow>? = null
var callbackOutput0_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackUniqueRow>
        var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var callbackReturnReady = false
        try { var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1466a57b41740b0e.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
callbackOutput0_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size
        var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackUniqueResource? = null

callbackOutput0_bulkActions!!.add { initial -> var callbackOutput0_bulkFailure = initial; try {             try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackUniqueResource_d31d0c3f2ae0419c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }; callbackOutput0_bulkFailure }
            callbackOutput0_bulkItem_g0_0 = io.prebindgen.covertest.callbacks.CallbackUniqueResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[0])) as Long)

callbackOutput0_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackUniqueRow(requireNotNull(callbackOutput0_bulkItem_g0_0), (requireNotNull(callbackOutput0_bulkReader).text((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[1])) as String))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1466a57b41740b0e.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackUniqueResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackUniqueResource_d31d0c3f2ae0419c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            callbackReturned = p0_callback.run(callbackArgument0_0); callbackReturnReady = true
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1466a57b41740b0e.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackUniqueResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackUniqueResource_d31d0c3f2ae0419c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        } } catch (failure: Throwable) { if (callbackReturnReady) { fun releaseReturned(value: Any?) { when (value) { is io.prebindgen.covertest.NativeHandle -> try { value.close() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }; is Iterable<*> -> value.forEach { releaseReturned(it) } } }; releaseReturned(callbackReturned) }; throw failure }
return kotlin.run { val callback_return_w1 = callbackReturned
 try {  io.prebindgen.covertest.withHandleLocks(if (true) requireNotNull(callback_return_w1) { "null handle" } else null) { val callbackReturnWire = callback_return_w1?.raw ?: 0L
if (true) callback_return_w1!!.markConsumed()
callbackReturnWire } } catch (failure: Throwable) { fun releaseReturned(value: Any?) { when (value) { is io.prebindgen.covertest.NativeHandle -> try { value.close() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }; is Iterable<*> -> value.forEach { releaseReturned(it) } } }; releaseReturned(callback_return_w1); throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_row_return_cleanup_failure_f029b8a3edf61e39(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackUniqueResourceId`(p0_w1: io.prebindgen.covertest.callbacks.CallbackUniqueResource?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_unique_resource_id_f503b662b364158e(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackRowsUnique`(p0_callback: io.prebindgen.covertest.CallbackUniqueRowVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackUniqueRow>? = null
var callbackOutput0_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackUniqueRow>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1466a57b41740b0e.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
callbackOutput0_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size
        var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackUniqueResource? = null

callbackOutput0_bulkActions!!.add { initial -> var callbackOutput0_bulkFailure = initial; try {             try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackUniqueResource_d31d0c3f2ae0419c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }; callbackOutput0_bulkFailure }
            callbackOutput0_bulkItem_g0_0 = io.prebindgen.covertest.callbacks.CallbackUniqueResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[0])) as Long)

callbackOutput0_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackUniqueRow(requireNotNull(callbackOutput0_bulkItem_g0_0), (requireNotNull(callbackOutput0_bulkReader).text((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[1])) as String))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1466a57b41740b0e.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackUniqueResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackUniqueResource_d31d0c3f2ae0419c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1466a57b41740b0e.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackUniqueResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackUniqueResource_d31d0c3f2ae0419c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_unique_019d8d474de1bdc3(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsUniqueError`(p0_callback: io.prebindgen.covertest.CallbackUniqueRowVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackUniqueRow>? = null
var callbackOutput0_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackUniqueRow>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1466a57b41740b0e.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
callbackOutput0_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size
        var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackUniqueResource? = null

callbackOutput0_bulkActions!!.add { initial -> var callbackOutput0_bulkFailure = initial; try {             try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackUniqueResource_d31d0c3f2ae0419c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }; callbackOutput0_bulkFailure }
            callbackOutput0_bulkItem_g0_0 = io.prebindgen.covertest.callbacks.CallbackUniqueResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[0])) as Long)

callbackOutput0_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackUniqueRow(requireNotNull(callbackOutput0_bulkItem_g0_0), (requireNotNull(callbackOutput0_bulkReader).text((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[1])) as String))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1466a57b41740b0e.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackUniqueResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackUniqueResource_d31d0c3f2ae0419c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1466a57b41740b0e.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackUniqueResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackUniqueResource_d31d0c3f2ae0419c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r06f0f78f562bfc88.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_unique_error_40154160e9803c99(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowResourceId`(p0_w1: io.prebindgen.covertest.callbacks.CallbackRowResource?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_row_resource_id_754894c5c9affc9b(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackRowResourceLive`(onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_row_resource_live_510223cb2d40d9e1(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackRowResourceDrops`(onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_row_resource_drops_01de29492e0e46ba(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackRowResourceHold`(p0_w1: io.prebindgen.covertest.callbacks.CallbackRowResource?, p1_callback: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p1_callback.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_row_resource_hold_5cf985322f526efd(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsOwnedProbe`(p0_callback: io.prebindgen.covertest.OrdinaryCallbackRowVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var callbackOutput0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>
        try {
callbackOutput0_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_list = kotlin.collections.mutableListOf()
for (callbackOutput0_rawRow in (raw0 as kotlin.Array<*>)) {
val callbackOutput0__row = callbackOutput0_rawRow as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_cleanups.add { callbackOutput0_initialFailure -> var callbackOutput0_failure = callbackOutput0_initialFailure; try {             try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }; callbackOutput0_failure }
            callbackOutput0_item_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[0] as Long)
            callbackOutput0_item_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[2] as Long)

callbackOutput0_list.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput0_item_g0_0), callbackOutput0__row[1] as String, requireNotNull(callbackOutput0_item_g2_0)))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
        } catch (failure: Throwable) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }

} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_owned_probe_18cebe9ab6081eb8(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsOptionalProbe`(p0_callback: io.prebindgen.covertest.OrdinaryOptionalOptionalCallbackRowVecCallback?, p1_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow?>? = null
var callbackOutput0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow?>? = null
        try {
            if (raw0 != null) {
callbackOutput0_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_list = kotlin.collections.mutableListOf()
for (callbackOutput0_rawRow in (raw0 as kotlin.Array<*>)) {
val callbackOutput0__row = callbackOutput0_rawRow as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_cleanups.add { callbackOutput0_initialFailure -> var callbackOutput0_failure = callbackOutput0_initialFailure; try {             if (callbackOutput0__row[0] == 1) {
            try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[1] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[3] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            }
 } catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }; callbackOutput0_failure }
            if (callbackOutput0__row[0] == 1) {
            callbackOutput0_item_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[1] as Long)
            callbackOutput0_item_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[3] as Long)
            }

callbackOutput0_list.add(if (callbackOutput0__row[0] == 1) io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput0_item_g0_0), callbackOutput0__row[2] as String, requireNotNull(callbackOutput0_item_g2_0)) else null)
}
            }
            callbackArgument0_0 = if (raw0 != null) requireNotNull(callbackOutput0_list) else null
        } catch (failure: Throwable) {
            if (raw0 != null) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            if (callbackOutput0__row[0] == 1) {
            try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[1] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[3] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            }

} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
            }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_optional_probe_bed3ad3142d5a05b(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsNestedProbe`(p0_callback: io.prebindgen.covertest.OrdinaryOptionalCallbackNestedRowVecVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackNestedRow?>>? = null
var callbackOutput0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackNestedRow?>>
        try {
callbackOutput0_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_list = kotlin.collections.mutableListOf()
for (callbackOutput0_rawRow in (raw0 as kotlin.Array<*>)) {
val callbackOutput0__row = callbackOutput0_rawRow as kotlin.Array<*>
var callbackOutput0_item_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackNestedRow?>? = null
var callbackOutput0_item_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

callbackOutput0_cleanups.add { callbackOutput0_initialFailure -> var callbackOutput0_failure = callbackOutput0_initialFailure; try { try {
for (callbackOutput0_item_cleanupAction in callbackOutput0_item_cleanups.orEmpty()) { try { callbackOutput0_failure = callbackOutput0_item_cleanupAction(callbackOutput0_failure) } catch (callbackOutput0_item_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_item_cleanupError else if (callbackOutput0_failure !== callbackOutput0_item_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_item_cleanupError) } }
for (callbackOutput0_item_index in (callbackOutput0_item_cleanups?.size ?: 0) until (callbackOutput0__row[0] as kotlin.Array<*>).size) {
var callbackOutput0_item_failure: kotlin.Throwable? = callbackOutput0_failure
try { val callbackOutput0_item__row = (callbackOutput0__row[0] as kotlin.Array<*>)[callbackOutput0_item_index] as kotlin.Array<*>
        var callbackOutput0_item_item_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_item_g0_g0_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_item_g1_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            if (callbackOutput0_item__row[0] == 1) {
            try { val owner = callbackOutput0_item_item_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0_item__row[1] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = cleanup else if (callbackOutput0_item_failure !== cleanup) callbackOutput0_item_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_item_g0_g0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0_item__row[3] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = cleanup else if (callbackOutput0_item_failure !== cleanup) callbackOutput0_item_failure!!.addSuppressed(cleanup) }
            if (callbackOutput0_item__row[4] == 1) {
            if (callbackOutput0_item__row[5] != 0L) {
            try { val owner = callbackOutput0_item_item_g1_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0_item__row[5] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = cleanup else if (callbackOutput0_item_failure !== cleanup) callbackOutput0_item_failure!!.addSuppressed(cleanup) }
            }
            }
            }

} catch (callbackOutput0_item_cleanupError: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = callbackOutput0_item_cleanupError else if (callbackOutput0_item_failure !== callbackOutput0_item_cleanupError) callbackOutput0_item_failure!!.addSuppressed(callbackOutput0_item_cleanupError) }
callbackOutput0_failure = callbackOutput0_item_failure
}
} catch (callbackOutput0_item_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_item_cleanupError else if (callbackOutput0_failure !== callbackOutput0_item_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_item_cleanupError) }
 } catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }; callbackOutput0_failure }
callbackOutput0_item_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_item_list = kotlin.collections.mutableListOf()
for (callbackOutput0_item_rawRow in (callbackOutput0__row[0] as kotlin.Array<*>)) {
val callbackOutput0_item__row = callbackOutput0_item_rawRow as kotlin.Array<*>
        var callbackOutput0_item_item_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_item_g0_g0_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_item_g1_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_item_cleanups.add { callbackOutput0_item_initialFailure -> var callbackOutput0_item_failure = callbackOutput0_item_initialFailure; try {             if (callbackOutput0_item__row[0] == 1) {
            try { val owner = callbackOutput0_item_item_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0_item__row[1] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = cleanup else if (callbackOutput0_item_failure !== cleanup) callbackOutput0_item_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_item_g0_g0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0_item__row[3] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = cleanup else if (callbackOutput0_item_failure !== cleanup) callbackOutput0_item_failure!!.addSuppressed(cleanup) }
            if (callbackOutput0_item__row[4] == 1) {
            if (callbackOutput0_item__row[5] != 0L) {
            try { val owner = callbackOutput0_item_item_g1_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0_item__row[5] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = cleanup else if (callbackOutput0_item_failure !== cleanup) callbackOutput0_item_failure!!.addSuppressed(cleanup) }
            }
            }
            }
 } catch (callbackOutput0_item_cleanupError: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = callbackOutput0_item_cleanupError else if (callbackOutput0_item_failure !== callbackOutput0_item_cleanupError) callbackOutput0_item_failure!!.addSuppressed(callbackOutput0_item_cleanupError) }; callbackOutput0_item_failure }
            if (callbackOutput0_item__row[0] == 1) {
            callbackOutput0_item_item_g0_g0_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0_item__row[1] as Long)
            callbackOutput0_item_item_g0_g0_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0_item__row[3] as Long)
            if (callbackOutput0_item__row[4] == 1) {
            if (callbackOutput0_item__row[5] != 0L) {
            callbackOutput0_item_item_g1_g0_g0_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0_item__row[5] as Long)
            }
            }
            }

callbackOutput0_item_list.add(if (callbackOutput0_item__row[0] == 1) io.prebindgen.covertest.callbacks.CallbackNestedRow(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput0_item_item_g0_g0_g0_0), callbackOutput0_item__row[2] as String, requireNotNull(callbackOutput0_item_item_g0_g0_g2_0)), if (callbackOutput0_item__row[4] == 1) io.prebindgen.covertest.callbacks.CallbackRowDetail(if (callbackOutput0_item__row[5] != 0L) requireNotNull(callbackOutput0_item_item_g1_g0_g0_g0_0) else null, callbackOutput0_item__row[6] as String) else null) else null)
}

callbackOutput0_list.add(requireNotNull(callbackOutput0_item_list))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
        } catch (failure: Throwable) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>
var callbackOutput0_item_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackNestedRow?>? = null
var callbackOutput0_item_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

try {
for (callbackOutput0_item_cleanupAction in callbackOutput0_item_cleanups.orEmpty()) { try { callbackOutput0_failure = callbackOutput0_item_cleanupAction(callbackOutput0_failure) } catch (callbackOutput0_item_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_item_cleanupError else if (callbackOutput0_failure !== callbackOutput0_item_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_item_cleanupError) } }
for (callbackOutput0_item_index in (callbackOutput0_item_cleanups?.size ?: 0) until (callbackOutput0__row[0] as kotlin.Array<*>).size) {
var callbackOutput0_item_failure: kotlin.Throwable? = callbackOutput0_failure
try { val callbackOutput0_item__row = (callbackOutput0__row[0] as kotlin.Array<*>)[callbackOutput0_item_index] as kotlin.Array<*>
        var callbackOutput0_item_item_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_item_g0_g0_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_item_g1_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            if (callbackOutput0_item__row[0] == 1) {
            try { val owner = callbackOutput0_item_item_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0_item__row[1] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = cleanup else if (callbackOutput0_item_failure !== cleanup) callbackOutput0_item_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_item_g0_g0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0_item__row[3] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = cleanup else if (callbackOutput0_item_failure !== cleanup) callbackOutput0_item_failure!!.addSuppressed(cleanup) }
            if (callbackOutput0_item__row[4] == 1) {
            if (callbackOutput0_item__row[5] != 0L) {
            try { val owner = callbackOutput0_item_item_g1_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0_item__row[5] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = cleanup else if (callbackOutput0_item_failure !== cleanup) callbackOutput0_item_failure!!.addSuppressed(cleanup) }
            }
            }
            }

} catch (callbackOutput0_item_cleanupError: Throwable) { if (callbackOutput0_item_failure == null) callbackOutput0_item_failure = callbackOutput0_item_cleanupError else if (callbackOutput0_item_failure !== callbackOutput0_item_cleanupError) callbackOutput0_item_failure!!.addSuppressed(callbackOutput0_item_cleanupError) }
callbackOutput0_failure = callbackOutput0_item_failure
}
} catch (callbackOutput0_item_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_item_cleanupError else if (callbackOutput0_failure !== callbackOutput0_item_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_item_cleanupError) }

} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_nested_probe_88c40199386d8891(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsPairProbe`(p0_callback: io.prebindgen.covertest.OrdinaryCallbackRowVecCallbackRowVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?, raw1: Array<Any?>?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var callbackOutput0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>
var callbackOutput1_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var callbackOutput1_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        lateinit var callbackArgument1_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>
        try {
callbackOutput0_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_list = kotlin.collections.mutableListOf()
for (callbackOutput0_rawRow in (raw0 as kotlin.Array<*>)) {
val callbackOutput0__row = callbackOutput0_rawRow as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_cleanups.add { callbackOutput0_initialFailure -> var callbackOutput0_failure = callbackOutput0_initialFailure; try {             try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }; callbackOutput0_failure }
            callbackOutput0_item_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[0] as Long)
            callbackOutput0_item_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[2] as Long)

callbackOutput0_list.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput0_item_g0_0), callbackOutput0__row[1] as String, requireNotNull(callbackOutput0_item_g2_0)))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
callbackOutput1_cleanups = kotlin.collections.mutableListOf()
callbackOutput1_list = kotlin.collections.mutableListOf()
for (callbackOutput1_rawRow in (raw1 as kotlin.Array<*>)) {
val callbackOutput1__row = callbackOutput1_rawRow as kotlin.Array<*>
        var callbackOutput1_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput1_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput1_cleanups.add { callbackOutput1_initialFailure -> var callbackOutput1_failure = callbackOutput1_initialFailure; try {             try { val owner = callbackOutput1_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput1__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput1_failure == null) callbackOutput1_failure = cleanup else if (callbackOutput1_failure !== cleanup) callbackOutput1_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput1_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput1__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput1_failure == null) callbackOutput1_failure = cleanup else if (callbackOutput1_failure !== cleanup) callbackOutput1_failure!!.addSuppressed(cleanup) }
 } catch (callbackOutput1_cleanupError: Throwable) { if (callbackOutput1_failure == null) callbackOutput1_failure = callbackOutput1_cleanupError else if (callbackOutput1_failure !== callbackOutput1_cleanupError) callbackOutput1_failure!!.addSuppressed(callbackOutput1_cleanupError) }; callbackOutput1_failure }
            callbackOutput1_item_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput1__row[0] as Long)
            callbackOutput1_item_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput1__row[2] as Long)

callbackOutput1_list.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput1_item_g0_0), callbackOutput1__row[1] as String, requireNotNull(callbackOutput1_item_g2_0)))
}
            callbackArgument1_0 = requireNotNull(callbackOutput1_list)
        } catch (failure: Throwable) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }

} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
try {
for (callbackOutput1_cleanupAction in callbackOutput1_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput1_cleanupAction(callbackCleanupFailure) } catch (callbackOutput1_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput1_cleanupError else if (callbackCleanupFailure !== callbackOutput1_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput1_cleanupError) } }
for (callbackOutput1_index in (callbackOutput1_cleanups?.size ?: 0) until (raw1 as kotlin.Array<*>).size) {
var callbackOutput1_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput1__row = (raw1 as kotlin.Array<*>)[callbackOutput1_index] as kotlin.Array<*>
        var callbackOutput1_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput1_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput1_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput1__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput1_failure == null) callbackOutput1_failure = cleanup else if (callbackOutput1_failure !== cleanup) callbackOutput1_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput1_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput1__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput1_failure == null) callbackOutput1_failure = cleanup else if (callbackOutput1_failure !== cleanup) callbackOutput1_failure!!.addSuppressed(cleanup) }

} catch (callbackOutput1_cleanupError: Throwable) { if (callbackOutput1_failure == null) callbackOutput1_failure = callbackOutput1_cleanupError else if (callbackOutput1_failure !== callbackOutput1_cleanupError) callbackOutput1_failure!!.addSuppressed(callbackOutput1_cleanupError) }
callbackCleanupFailure = callbackOutput1_failure
}
} catch (callbackOutput1_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput1_cleanupError else if (callbackCleanupFailure !== callbackOutput1_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput1_cleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0, callbackArgument1_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_pair_probe_de30e0e31a7df3e7(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsTransferredProbe`(p0_callback: io.prebindgen.covertest.OrdinaryCallbackTransferredRowVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackTransferredRow>? = null
var callbackOutput0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackTransferredRow>
        try {
callbackOutput0_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_list = kotlin.collections.mutableListOf()
for (callbackOutput0_rawRow in (raw0 as kotlin.Array<*>)) {
val callbackOutput0__row = callbackOutput0_rawRow as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_cleanups.add { callbackOutput0_initialFailure -> var callbackOutput0_failure = callbackOutput0_initialFailure; try {             try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }; callbackOutput0_failure }
            callbackOutput0_item_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[0] as Long)
            callbackOutput0_item_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[2] as Long)

callbackOutput0_list.add(io.prebindgen.covertest.callbacks.CallbackTransferredRow(requireNotNull(callbackOutput0_item_g0_0), callbackOutput0__row[1] as String, requireNotNull(callbackOutput0_item_g2_0)))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
        } catch (failure: Throwable) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }

} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_transferred_probe_a4ac030f6ad6a92d(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsTransferredPairProbe`(p0_callback: io.prebindgen.covertest.OrdinaryCallbackTransferredRowVecCallbackRowVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?, raw1: Array<Any?>?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackTransferredRow>? = null
var callbackOutput0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackTransferredRow>
var callbackOutput1_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var callbackOutput1_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        lateinit var callbackArgument1_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>
        try {
callbackOutput0_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_list = kotlin.collections.mutableListOf()
for (callbackOutput0_rawRow in (raw0 as kotlin.Array<*>)) {
val callbackOutput0__row = callbackOutput0_rawRow as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_cleanups.add { callbackOutput0_initialFailure -> var callbackOutput0_failure = callbackOutput0_initialFailure; try {             try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }; callbackOutput0_failure }
            callbackOutput0_item_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[0] as Long)
            callbackOutput0_item_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[2] as Long)

callbackOutput0_list.add(io.prebindgen.covertest.callbacks.CallbackTransferredRow(requireNotNull(callbackOutput0_item_g0_0), callbackOutput0__row[1] as String, requireNotNull(callbackOutput0_item_g2_0)))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
callbackOutput1_cleanups = kotlin.collections.mutableListOf()
callbackOutput1_list = kotlin.collections.mutableListOf()
for (callbackOutput1_rawRow in (raw1 as kotlin.Array<*>)) {
val callbackOutput1__row = callbackOutput1_rawRow as kotlin.Array<*>
        var callbackOutput1_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput1_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput1_cleanups.add { callbackOutput1_initialFailure -> var callbackOutput1_failure = callbackOutput1_initialFailure; try {             try { val owner = callbackOutput1_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput1__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput1_failure == null) callbackOutput1_failure = cleanup else if (callbackOutput1_failure !== cleanup) callbackOutput1_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput1_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput1__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput1_failure == null) callbackOutput1_failure = cleanup else if (callbackOutput1_failure !== cleanup) callbackOutput1_failure!!.addSuppressed(cleanup) }
 } catch (callbackOutput1_cleanupError: Throwable) { if (callbackOutput1_failure == null) callbackOutput1_failure = callbackOutput1_cleanupError else if (callbackOutput1_failure !== callbackOutput1_cleanupError) callbackOutput1_failure!!.addSuppressed(callbackOutput1_cleanupError) }; callbackOutput1_failure }
            callbackOutput1_item_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput1__row[0] as Long)
            callbackOutput1_item_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput1__row[2] as Long)

callbackOutput1_list.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput1_item_g0_0), callbackOutput1__row[1] as String, requireNotNull(callbackOutput1_item_g2_0)))
}
            callbackArgument1_0 = requireNotNull(callbackOutput1_list)
        } catch (failure: Throwable) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }

} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
try {
for (callbackOutput1_cleanupAction in callbackOutput1_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput1_cleanupAction(callbackCleanupFailure) } catch (callbackOutput1_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput1_cleanupError else if (callbackCleanupFailure !== callbackOutput1_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput1_cleanupError) } }
for (callbackOutput1_index in (callbackOutput1_cleanups?.size ?: 0) until (raw1 as kotlin.Array<*>).size) {
var callbackOutput1_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput1__row = (raw1 as kotlin.Array<*>)[callbackOutput1_index] as kotlin.Array<*>
        var callbackOutput1_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput1_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput1_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput1__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput1_failure == null) callbackOutput1_failure = cleanup else if (callbackOutput1_failure !== cleanup) callbackOutput1_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput1_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput1__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput1_failure == null) callbackOutput1_failure = cleanup else if (callbackOutput1_failure !== cleanup) callbackOutput1_failure!!.addSuppressed(cleanup) }

} catch (callbackOutput1_cleanupError: Throwable) { if (callbackOutput1_failure == null) callbackOutput1_failure = callbackOutput1_cleanupError else if (callbackOutput1_failure !== callbackOutput1_cleanupError) callbackOutput1_failure!!.addSuppressed(callbackOutput1_cleanupError) }
callbackCleanupFailure = callbackOutput1_failure
}
} catch (callbackOutput1_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput1_cleanupError else if (callbackCleanupFailure !== callbackOutput1_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput1_cleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0, callbackArgument1_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_transferred_pair_probe_43ce9ece795705de(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsOwned`(p0_callback: io.prebindgen.covertest.CallbackRowVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var callbackOutput0_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
callbackOutput0_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
        var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_bulkActions!!.add { initial -> var callbackOutput0_bulkFailure = initial; try {             try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }; callbackOutput0_bulkFailure }
            callbackOutput0_bulkItem_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long)
            callbackOutput0_bulkItem_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long)

callbackOutput0_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput0_bulkItem_g0_0), (requireNotNull(callbackOutput0_bulkReader).text((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[1])) as String, requireNotNull(callbackOutput0_bulkItem_g2_0)))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_owned_3e990348ad64d75b(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsBorrowed`(p0_callback: io.prebindgen.covertest.CallbackRowListCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var callbackOutput0_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
callbackOutput0_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
        var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_bulkActions!!.add { initial -> var callbackOutput0_bulkFailure = initial; try {             try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }; callbackOutput0_bulkFailure }
            callbackOutput0_bulkItem_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long)
            callbackOutput0_bulkItem_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long)

callbackOutput0_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput0_bulkItem_g0_0), (requireNotNull(callbackOutput0_bulkReader).text((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[1])) as String, requireNotNull(callbackOutput0_bulkItem_g2_0)))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_borrowed_57b23a1fb6669005(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsOptional`(p0_callback: io.prebindgen.covertest.OptionalOptionalCallbackRowVecCallback?, p1_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow?>? = null
var callbackOutput0_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow?>? = null
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            if (raw0 != null) {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r11e62943be787ed5.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
callbackOutput0_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.size
        var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_bulkActions!!.add { initial -> var callbackOutput0_bulkFailure = initial; try {             if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[0])) == 1) {
            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[1])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[3])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }; callbackOutput0_bulkFailure }
            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[0])) == 1) {
            callbackOutput0_bulkItem_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[1])) as Long)
            callbackOutput0_bulkItem_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[3])) as Long)
            }

callbackOutput0_bulkList!!.add(if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[0])) == 1) io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput0_bulkItem_g0_0), (requireNotNull(callbackOutput0_bulkReader).text((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[2])) as String, requireNotNull(callbackOutput0_bulkItem_g2_0)) else null)
}
            }
            callbackArgument0_0 = if (raw0 != null) requireNotNull(callbackOutput0_bulkList) else null
        } catch (failure: Throwable) {
            if (raw0 != null) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r11e62943be787ed5.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[0])) == 1) {
            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[1])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[3])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
            }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            if (raw0 != null) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r11e62943be787ed5.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[0])) == 1) {
            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[1])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.re6369465b61481d1.offsets[3])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
            }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_optional_0d52c7d730c41d75(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsNested`(p0_callback: io.prebindgen.covertest.OptionalCallbackNestedRowVecVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackNestedRow?>>? = null
var callbackOutput0_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackNestedRow?>>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r257bda408ee32755.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra15a90b5b0cdaba7.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
callbackOutput0_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra15a90b5b0cdaba7.size
var callbackOutput0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackNestedRow?>? = null
var callbackOutput0_bulkItem_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null

callbackOutput0_bulkActions!!.add { initial -> var callbackOutput0_bulkFailure = initial; try { try { if (callbackOutput0_bulkItem_bulkRange == null) callbackOutput0_bulkItem_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra15a90b5b0cdaba7.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.size)
for (action in callbackOutput0_bulkItem_bulkActions.orEmpty()) { try { callbackOutput0_bulkFailure = action(callbackOutput0_bulkFailure) } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) } }
for (callbackOutput0_bulkItem_bulkIndex in (callbackOutput0_bulkItem_bulkActions?.size ?: 0) until callbackOutput0_bulkItem_bulkRange!!.length) {
val callbackOutput0_bulkItem_bulkBase = callbackOutput0_bulkItem_bulkRange!!.offset + callbackOutput0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.size
var callbackOutput0_bulkItem_bulkFailure: Throwable? = callbackOutput0_bulkFailure
try {         var callbackOutput0_bulkItem_bulkItem_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_bulkItem_g0_g0_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_bulkItem_g1_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[0])) == 1) {
            try { val owner = callbackOutput0_bulkItem_bulkItem_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[1])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_bulkItem_g0_g0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[3])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[4])) == 1) {
            if ((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[5])) != 0L) {
            try { val owner = callbackOutput0_bulkItem_bulkItem_g1_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[5])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
            }
            }
 } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkItem_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }
callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkFailure
} } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }; callbackOutput0_bulkFailure }
callbackOutput0_bulkItem_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra15a90b5b0cdaba7.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.size)
callbackOutput0_bulkItem_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkItem_bulkRange!!.length)
callbackOutput0_bulkItem_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkItem_bulkIndex in 0 until callbackOutput0_bulkItem_bulkRange!!.length) {
val callbackOutput0_bulkItem_bulkBase = callbackOutput0_bulkItem_bulkRange!!.offset + callbackOutput0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.size
        var callbackOutput0_bulkItem_bulkItem_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_bulkItem_g0_g0_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_bulkItem_g1_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_bulkItem_bulkActions!!.add { initial -> var callbackOutput0_bulkItem_bulkFailure = initial; try {             if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[0])) == 1) {
            try { val owner = callbackOutput0_bulkItem_bulkItem_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[1])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_bulkItem_g0_g0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[3])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[4])) == 1) {
            if ((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[5])) != 0L) {
            try { val owner = callbackOutput0_bulkItem_bulkItem_g1_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[5])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
            }
            }
 } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkItem_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }; callbackOutput0_bulkItem_bulkFailure }
            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[0])) == 1) {
            callbackOutput0_bulkItem_bulkItem_g0_g0_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[1])) as Long)
            callbackOutput0_bulkItem_bulkItem_g0_g0_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[3])) as Long)
            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[4])) == 1) {
            if ((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[5])) != 0L) {
            callbackOutput0_bulkItem_bulkItem_g1_g0_g0_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[5])) as Long)
            }
            }
            }

callbackOutput0_bulkItem_bulkList!!.add(if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[0])) == 1) io.prebindgen.covertest.callbacks.CallbackNestedRow(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput0_bulkItem_bulkItem_g0_g0_g0_0), (requireNotNull(callbackOutput0_bulkReader).text((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[2])) as String, requireNotNull(callbackOutput0_bulkItem_bulkItem_g0_g0_g2_0)), if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[4])) == 1) io.prebindgen.covertest.callbacks.CallbackRowDetail(if ((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[5])) != 0L) requireNotNull(callbackOutput0_bulkItem_bulkItem_g1_g0_g0_g0_0) else null, (requireNotNull(callbackOutput0_bulkReader).text((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[6])) as String) else null) else null)
}

callbackOutput0_bulkList!!.add(requireNotNull(callbackOutput0_bulkItem_bulkList))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r257bda408ee32755.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra15a90b5b0cdaba7.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra15a90b5b0cdaba7.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try { var callbackOutput0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackNestedRow?>? = null
var callbackOutput0_bulkItem_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null

try { if (callbackOutput0_bulkItem_bulkRange == null) callbackOutput0_bulkItem_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra15a90b5b0cdaba7.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.size)
for (action in callbackOutput0_bulkItem_bulkActions.orEmpty()) { try { callbackOutput0_bulkFailure = action(callbackOutput0_bulkFailure) } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) } }
for (callbackOutput0_bulkItem_bulkIndex in (callbackOutput0_bulkItem_bulkActions?.size ?: 0) until callbackOutput0_bulkItem_bulkRange!!.length) {
val callbackOutput0_bulkItem_bulkBase = callbackOutput0_bulkItem_bulkRange!!.offset + callbackOutput0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.size
var callbackOutput0_bulkItem_bulkFailure: Throwable? = callbackOutput0_bulkFailure
try {         var callbackOutput0_bulkItem_bulkItem_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_bulkItem_g0_g0_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_bulkItem_g1_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[0])) == 1) {
            try { val owner = callbackOutput0_bulkItem_bulkItem_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[1])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_bulkItem_g0_g0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[3])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[4])) == 1) {
            if ((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[5])) != 0L) {
            try { val owner = callbackOutput0_bulkItem_bulkItem_g1_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[5])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
            }
            }
 } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkItem_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }
callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkFailure
} } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r257bda408ee32755.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra15a90b5b0cdaba7.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra15a90b5b0cdaba7.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try { var callbackOutput0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackNestedRow?>? = null
var callbackOutput0_bulkItem_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null

try { if (callbackOutput0_bulkItem_bulkRange == null) callbackOutput0_bulkItem_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra15a90b5b0cdaba7.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.size)
for (action in callbackOutput0_bulkItem_bulkActions.orEmpty()) { try { callbackOutput0_bulkFailure = action(callbackOutput0_bulkFailure) } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) } }
for (callbackOutput0_bulkItem_bulkIndex in (callbackOutput0_bulkItem_bulkActions?.size ?: 0) until callbackOutput0_bulkItem_bulkRange!!.length) {
val callbackOutput0_bulkItem_bulkBase = callbackOutput0_bulkItem_bulkRange!!.offset + callbackOutput0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.size
var callbackOutput0_bulkItem_bulkFailure: Throwable? = callbackOutput0_bulkFailure
try {         var callbackOutput0_bulkItem_bulkItem_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_bulkItem_g0_g0_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_bulkItem_g1_g0_g0_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[0])) == 1) {
            try { val owner = callbackOutput0_bulkItem_bulkItem_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[1])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_bulkItem_g0_g0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[3])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            if ((requireNotNull(callbackOutput0_bulkReader).int((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[4])) == 1) {
            if ((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[5])) != 0L) {
            try { val owner = callbackOutput0_bulkItem_bulkItem_g1_g0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.re730ef3861c078bd.offsets[5])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = cleanup else if (callbackOutput0_bulkItem_bulkFailure !== cleanup) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
            }
            }
 } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkItem_bulkFailure == null) callbackOutput0_bulkItem_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkItem_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkItem_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }
callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkFailure
} } catch (callbackOutput0_bulkItem_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkItem_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkItem_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkItem_bulkCleanupError) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_nested_5da56d6464038743(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsPair`(p0_callback: io.prebindgen.covertest.CallbackRowVecCallbackRowVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?, raw1: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var callbackOutput0_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>
var callbackOutput1_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput1_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput1_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var callbackOutput1_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument1_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
callbackOutput0_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
        var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_bulkActions!!.add { initial -> var callbackOutput0_bulkFailure = initial; try {             try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }; callbackOutput0_bulkFailure }
            callbackOutput0_bulkItem_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long)
            callbackOutput0_bulkItem_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long)

callbackOutput0_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput0_bulkItem_g0_0), (requireNotNull(callbackOutput0_bulkReader).text((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[1])) as String, requireNotNull(callbackOutput0_bulkItem_g2_0)))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
callbackOutput1_bulkReader = io.prebindgen.covertest.BulkReader(raw1 as ByteArray)
callbackOutput1_bulkRange = requireNotNull(callbackOutput1_bulkReader).rows(((requireNotNull(callbackOutput1_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
callbackOutput1_bulkList = kotlin.collections.ArrayList(callbackOutput1_bulkRange!!.length)
callbackOutput1_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput1_bulkIndex in 0 until callbackOutput1_bulkRange!!.length) {
val callbackOutput1_bulkBase = callbackOutput1_bulkRange!!.offset + callbackOutput1_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
        var callbackOutput1_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput1_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput1_bulkActions!!.add { initial -> var callbackOutput1_bulkFailure = initial; try {             try { val owner = callbackOutput1_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = cleanup else if (callbackOutput1_bulkFailure !== cleanup) callbackOutput1_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput1_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = cleanup else if (callbackOutput1_bulkFailure !== cleanup) callbackOutput1_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = callbackOutput1_bulkCleanupError else if (callbackOutput1_bulkFailure !== callbackOutput1_bulkCleanupError) callbackOutput1_bulkFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) }; callbackOutput1_bulkFailure }
            callbackOutput1_bulkItem_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long)
            callbackOutput1_bulkItem_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long)

callbackOutput1_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput1_bulkItem_g0_0), (requireNotNull(callbackOutput1_bulkReader).text((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[1])) as String, requireNotNull(callbackOutput1_bulkItem_g2_0)))
}
            callbackArgument1_0 = requireNotNull(callbackOutput1_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
if (callbackOutput1_bulkReader == null) { callbackOutput1_bulkReader = io.prebindgen.covertest.BulkReader(raw1 as ByteArray)
 }
try { if (callbackOutput1_bulkRange == null) callbackOutput1_bulkRange = requireNotNull(callbackOutput1_bulkReader).rows(((requireNotNull(callbackOutput1_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput1_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput1_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput1_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) } }
for (callbackOutput1_bulkIndex in (callbackOutput1_bulkActions?.size ?: 0) until callbackOutput1_bulkRange!!.length) {
val callbackOutput1_bulkBase = callbackOutput1_bulkRange!!.offset + callbackOutput1_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput1_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput1_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput1_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput1_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = cleanup else if (callbackOutput1_bulkFailure !== cleanup) callbackOutput1_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput1_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = cleanup else if (callbackOutput1_bulkFailure !== cleanup) callbackOutput1_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = callbackOutput1_bulkCleanupError else if (callbackOutput1_bulkFailure !== callbackOutput1_bulkCleanupError) callbackOutput1_bulkFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) }
callbackCleanupFailure = callbackOutput1_bulkFailure
} } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput1_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput1_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0, callbackArgument1_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
if (callbackOutput1_bulkReader == null) { callbackOutput1_bulkReader = io.prebindgen.covertest.BulkReader(raw1 as ByteArray)
 }
try { if (callbackOutput1_bulkRange == null) callbackOutput1_bulkRange = requireNotNull(callbackOutput1_bulkReader).rows(((requireNotNull(callbackOutput1_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput1_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput1_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput1_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) } }
for (callbackOutput1_bulkIndex in (callbackOutput1_bulkActions?.size ?: 0) until callbackOutput1_bulkRange!!.length) {
val callbackOutput1_bulkBase = callbackOutput1_bulkRange!!.offset + callbackOutput1_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput1_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput1_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput1_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput1_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = cleanup else if (callbackOutput1_bulkFailure !== cleanup) callbackOutput1_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput1_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = cleanup else if (callbackOutput1_bulkFailure !== cleanup) callbackOutput1_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = callbackOutput1_bulkCleanupError else if (callbackOutput1_bulkFailure !== callbackOutput1_bulkCleanupError) callbackOutput1_bulkFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) }
callbackCleanupFailure = callbackOutput1_bulkFailure
} } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput1_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput1_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_pair_a961f029753aca2e(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowHandles`(p0_callback: io.prebindgen.covertest.CallbackRowResourceVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRowResource>? = null
var callbackOutput0_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRowResource>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
callbackOutput0_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size
        var callbackOutput0_bulkItem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_bulkActions!!.add { initial -> var callbackOutput0_bulkFailure = initial; try {             try { val owner = callbackOutput0_bulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }; callbackOutput0_bulkFailure }
            callbackOutput0_bulkItem_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long)

callbackOutput0_bulkList!!.add(requireNotNull(callbackOutput0_bulkItem_0))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_bulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf673a6355cb08de9.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_bulkItem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r59a3202a71549093.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_row_handles_0883c9c06120fdc3(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsTransferred`(p0_callback: io.prebindgen.covertest.CallbackTransferredRowVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackTransferredRow>? = null
var callbackOutput0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackTransferredRow>
        try {
callbackOutput0_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_list = kotlin.collections.mutableListOf()
for (callbackOutput0_rawRow in (raw0 as kotlin.Array<*>)) {
val callbackOutput0__row = callbackOutput0_rawRow as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_cleanups.add { callbackOutput0_initialFailure -> var callbackOutput0_failure = callbackOutput0_initialFailure; try {             try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }; callbackOutput0_failure }
            callbackOutput0_item_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[0] as Long)
            callbackOutput0_item_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[2] as Long)

callbackOutput0_list.add(io.prebindgen.covertest.callbacks.CallbackTransferredRow(requireNotNull(callbackOutput0_item_g0_0), callbackOutput0__row[1] as String, requireNotNull(callbackOutput0_item_g2_0)))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
        } catch (failure: Throwable) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }

} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_transferred_bab23b0a8a8f4840(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsTransferredPair`(p0_callback: io.prebindgen.covertest.CallbackTransferredRowVecCallbackRowVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?, raw1: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackTransferredRow>? = null
var callbackOutput0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackTransferredRow>
var callbackOutput1_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput1_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput1_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var callbackOutput1_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument1_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_cleanups = kotlin.collections.mutableListOf()
callbackOutput0_list = kotlin.collections.mutableListOf()
for (callbackOutput0_rawRow in (raw0 as kotlin.Array<*>)) {
val callbackOutput0__row = callbackOutput0_rawRow as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_cleanups.add { callbackOutput0_initialFailure -> var callbackOutput0_failure = callbackOutput0_initialFailure; try {             try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }; callbackOutput0_failure }
            callbackOutput0_item_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[0] as Long)
            callbackOutput0_item_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource(callbackOutput0__row[2] as Long)

callbackOutput0_list.add(io.prebindgen.covertest.callbacks.CallbackTransferredRow(requireNotNull(callbackOutput0_item_g0_0), callbackOutput0__row[1] as String, requireNotNull(callbackOutput0_item_g2_0)))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
callbackOutput1_bulkReader = io.prebindgen.covertest.BulkReader(raw1 as ByteArray)
callbackOutput1_bulkRange = requireNotNull(callbackOutput1_bulkReader).rows(((requireNotNull(callbackOutput1_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
callbackOutput1_bulkList = kotlin.collections.ArrayList(callbackOutput1_bulkRange!!.length)
callbackOutput1_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput1_bulkIndex in 0 until callbackOutput1_bulkRange!!.length) {
val callbackOutput1_bulkBase = callbackOutput1_bulkRange!!.offset + callbackOutput1_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
        var callbackOutput1_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput1_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput1_bulkActions!!.add { initial -> var callbackOutput1_bulkFailure = initial; try {             try { val owner = callbackOutput1_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = cleanup else if (callbackOutput1_bulkFailure !== cleanup) callbackOutput1_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput1_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = cleanup else if (callbackOutput1_bulkFailure !== cleanup) callbackOutput1_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = callbackOutput1_bulkCleanupError else if (callbackOutput1_bulkFailure !== callbackOutput1_bulkCleanupError) callbackOutput1_bulkFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) }; callbackOutput1_bulkFailure }
            callbackOutput1_bulkItem_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long)
            callbackOutput1_bulkItem_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long)

callbackOutput1_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput1_bulkItem_g0_0), (requireNotNull(callbackOutput1_bulkReader).text((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[1])) as String, requireNotNull(callbackOutput1_bulkItem_g2_0)))
}
            callbackArgument1_0 = requireNotNull(callbackOutput1_bulkList)
        } catch (failure: Throwable) {
try {
for (callbackOutput0_cleanupAction in callbackOutput0_cleanups.orEmpty()) { try { callbackCleanupFailure = callbackOutput0_cleanupAction(callbackCleanupFailure) } catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) } }
for (callbackOutput0_index in (callbackOutput0_cleanups?.size ?: 0) until (raw0 as kotlin.Array<*>).size) {
var callbackOutput0_failure: kotlin.Throwable? = callbackCleanupFailure
try { val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as kotlin.Array<*>
        var callbackOutput0_item_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_item_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_item_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[0] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_item_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c(callbackOutput0__row[2] as Long) } catch (cleanup: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = cleanup else if (callbackOutput0_failure !== cleanup) callbackOutput0_failure!!.addSuppressed(cleanup) }

} catch (callbackOutput0_cleanupError: Throwable) { if (callbackOutput0_failure == null) callbackOutput0_failure = callbackOutput0_cleanupError else if (callbackOutput0_failure !== callbackOutput0_cleanupError) callbackOutput0_failure!!.addSuppressed(callbackOutput0_cleanupError) }
callbackCleanupFailure = callbackOutput0_failure
}
} catch (callbackOutput0_cleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_cleanupError else if (callbackCleanupFailure !== callbackOutput0_cleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_cleanupError) }
if (callbackOutput1_bulkReader == null) { callbackOutput1_bulkReader = io.prebindgen.covertest.BulkReader(raw1 as ByteArray)
 }
try { if (callbackOutput1_bulkRange == null) callbackOutput1_bulkRange = requireNotNull(callbackOutput1_bulkReader).rows(((requireNotNull(callbackOutput1_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput1_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput1_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput1_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) } }
for (callbackOutput1_bulkIndex in (callbackOutput1_bulkActions?.size ?: 0) until callbackOutput1_bulkRange!!.length) {
val callbackOutput1_bulkBase = callbackOutput1_bulkRange!!.offset + callbackOutput1_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput1_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput1_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput1_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput1_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = cleanup else if (callbackOutput1_bulkFailure !== cleanup) callbackOutput1_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput1_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = cleanup else if (callbackOutput1_bulkFailure !== cleanup) callbackOutput1_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = callbackOutput1_bulkCleanupError else if (callbackOutput1_bulkFailure !== callbackOutput1_bulkCleanupError) callbackOutput1_bulkFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) }
callbackCleanupFailure = callbackOutput1_bulkFailure
} } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput1_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput1_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0, callbackArgument1_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput1_bulkReader == null) { callbackOutput1_bulkReader = io.prebindgen.covertest.BulkReader(raw1 as ByteArray)
 }
try { if (callbackOutput1_bulkRange == null) callbackOutput1_bulkRange = requireNotNull(callbackOutput1_bulkReader).rows(((requireNotNull(callbackOutput1_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput1_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput1_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput1_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) } }
for (callbackOutput1_bulkIndex in (callbackOutput1_bulkActions?.size ?: 0) until callbackOutput1_bulkRange!!.length) {
val callbackOutput1_bulkBase = callbackOutput1_bulkRange!!.offset + callbackOutput1_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput1_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput1_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput1_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput1_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = cleanup else if (callbackOutput1_bulkFailure !== cleanup) callbackOutput1_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput1_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = cleanup else if (callbackOutput1_bulkFailure !== cleanup) callbackOutput1_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackOutput1_bulkFailure == null) callbackOutput1_bulkFailure = callbackOutput1_bulkCleanupError else if (callbackOutput1_bulkFailure !== callbackOutput1_bulkCleanupError) callbackOutput1_bulkFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) }
callbackCleanupFailure = callbackOutput1_bulkFailure
} } catch (callbackOutput1_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput1_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput1_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput1_bulkCleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_transferred_pair_e141e947123b4d58(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsConversionFailure`(p0_callback: io.prebindgen.covertest.CallbackRowVecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var callbackOutput0_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
callbackOutput0_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
        var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_bulkActions!!.add { initial -> var callbackOutput0_bulkFailure = initial; try {             try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }; callbackOutput0_bulkFailure }
            callbackOutput0_bulkItem_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long)
            callbackOutput0_bulkItem_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long)

callbackOutput0_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput0_bulkItem_g0_0), (requireNotNull(callbackOutput0_bulkReader).text((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[1])) as String, requireNotNull(callbackOutput0_bulkItem_g2_0)))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_conversion_failure_9ae85f90d1f5c256(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackRowsSpawn`(p0_callback: io.prebindgen.covertest.CallbackRowVecCallback?, p1_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbacks.CallbackJob> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.callbacks.CallbackJob = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbacks.CallbackRow>? = null
var callbackOutput0_bulkActions: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.callbacks.CallbackRow>
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
callbackOutput0_bulkActions = kotlin.collections.mutableListOf()
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
        var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

callbackOutput0_bulkActions!!.add { initial -> var callbackOutput0_bulkFailure = initial; try {             try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }; callbackOutput0_bulkFailure }
            callbackOutput0_bulkItem_g0_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long)
            callbackOutput0_bulkItem_g2_0 = io.prebindgen.covertest.callbacks.CallbackRowResource((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long)

callbackOutput0_bulkList!!.add(io.prebindgen.covertest.callbacks.CallbackRow(requireNotNull(callbackOutput0_bulkItem_g0_0), (requireNotNull(callbackOutput0_bulkReader).text((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[1])) as String, requireNotNull(callbackOutput0_bulkItem_g2_0)))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r070bbcf38f849ff2.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size)
for (action in callbackOutput0_bulkActions.orEmpty()) { try { callbackCleanupFailure = action(callbackCleanupFailure) } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) } }
for (callbackOutput0_bulkIndex in (callbackOutput0_bulkActions?.size ?: 0) until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_g0_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null
        var callbackOutput0_bulkItem_g2_0: io.prebindgen.covertest.callbacks.CallbackRowResource? = null

            try { val owner = callbackOutput0_bulkItem_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[0])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
            try { val owner = callbackOutput0_bulkItem_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackRowResource_abd70bbf025eb35c((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.rf508a557893415ce.offsets[2])) as Long) } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_rows_spawn_4aac2c41997a9b73(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.callbacks.CallbackJob? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackJob_7f5ec1b2a514af2f(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.callbacks.CallbackJob? = null
        try {
            output0 = io.prebindgen.covertest.callbacks.CallbackJob(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackJob_7f5ec1b2a514af2f(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `callbackRowJobFinished`(p0_w1: io.prebindgen.covertest.callbacks.CallbackJob?, onError: io.prebindgen.covertest.JniErrorHandler<Boolean> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_row_job_finished_f2cd7260ebbf9dfa(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackReentryNew`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbacks.CallbackReentry> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.callbacks.CallbackReentry = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_new_7c866d733e6695ab(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.callbacks.CallbackReentry? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReentry_6c6c0fff370f6efb(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.callbacks.CallbackReentry? = null
        try {
            output0 = io.prebindgen.covertest.callbacks.CallbackReentry(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReentry_6c6c0fff370f6efb(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `callbackReentryRead`(p0_w1: io.prebindgen.covertest.callbacks.CallbackReentry?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_read_3eac5f6ba179e3a0(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackReentryWrite`(p0_w1: io.prebindgen.covertest.callbacks.CallbackReentry?, p1_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_write_d51c3e7985c5d5b1(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackReentryTake`(p0_w1: io.prebindgen.covertest.callbacks.CallbackReentry?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_take_784cf035dcf4a6fe(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0]) { if (true) p0_w1!!.markConsumed() } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackReentryShared`(p0_w1: io.prebindgen.covertest.callbacks.CallbackReentry?, p1_callback: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p1_callback.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_shared_eb391d1c7013a7e0(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackReentryExclusive`(p0_w1: io.prebindgen.covertest.callbacks.CallbackReentry?, p1_callback: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p1_callback.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_exclusive_c4c879f41d91e189(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackReentryConsuming`(p0_w1: io.prebindgen.covertest.callbacks.CallbackReentry?, p1_callback: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p1_callback.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_consuming_ba9f8f27ccfa4a2d(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0]) { if (true) p0_w1!!.markConsumed() } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackReentryPair`(p0_w1: io.prebindgen.covertest.callbacks.CallbackReentry?, p1_w1: io.prebindgen.covertest.callbacks.CallbackReentry?, p2_callback: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val handle1 = try { if (true) requireNotNull(p1_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0, handle1) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (p2_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p2_callback.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_pair_6ab40de513e04e6c(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackReentryMixed`(p0_w1: io.prebindgen.covertest.callbacks.CallbackReentry?, p1_w1: io.prebindgen.covertest.callbacks.CallbackReentry?, p2_callback: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val handle1 = try { if (true) requireNotNull(p1_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0, handle1) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (p2_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p2_callback.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_mixed_78bee5923db373fd(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackReentryWorker`(p0_w1: io.prebindgen.covertest.callbacks.CallbackReentry?, p1_callback: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p1_callback.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_worker_c9405aa7f035bc96(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackReentryProbe`(p0_w1: io.prebindgen.covertest.callbacks.CallbackReentry?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_probe_b759005342e62645(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackReentryOptional`(p0_w3: io.prebindgen.covertest.callbacks.CallbackReentry?, p1_callback: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (p0_w3 != null) if (true) requireNotNull(requireNotNull(p0_w3)) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3).raw else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p1_callback.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_optional_23c1984d5bdcc548(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackReentryConstructed`(p0_w2: io.prebindgen.covertest.callbacks.CallbackReentry?, p1_callback: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w2) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w2?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p1_callback.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_reentry_constructed_7dd7a899513e45f3(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackConstructorNew`(p0_callback: io.prebindgen.covertest.i64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbacks.CallbackConstructor> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.callbacks.CallbackConstructor = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
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
        p0_callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_new_4d91f17ba5ed43a7(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.callbacks.CallbackConstructor? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackConstructor_65ec2182d775ceb8(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.callbacks.CallbackConstructor? = null
        try {
            output0 = io.prebindgen.covertest.callbacks.CallbackConstructor(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackConstructor_65ec2182d775ceb8(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `callbackConstructorFire`(p0_w2: io.prebindgen.covertest.i64Callback?, p1_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_w2 == null) null else object {
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
        p0_w2.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_fire_f870340c7995bc02(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackConstructorOwnedFire`(p0_w2: io.prebindgen.covertest.callbacks.CallbackOwnedCallback?, p1_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_w2 == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbacks.CallbackOwned
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackOutput0_0 = io.prebindgen.covertest.callbacks.CallbackOwned(raw0 as Long)
            callbackArgument0_0 = requireNotNull(callbackOutput0_0)
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_w2.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_owned_fire_5025d0be7b00c9b9(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackConstructorPair`(p0_w2: io.prebindgen.covertest.i64Callback?, p1_w2: io.prebindgen.covertest.i64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_w2 == null) null else object {
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
        p0_w2.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_w2 == null) null else object {
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
        p1_w2.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_pair_9c4ca5ba547ab06e(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackConstructorMixed`(p0_callback: io.prebindgen.covertest.i64Callback?, p1_w2: io.prebindgen.covertest.i64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
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
        p0_callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_w2 == null) null else object {
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
        p1_w2.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_mixed_0d8af3de66dbc771(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackConstructorSetFire`(p0_w2: io.prebindgen.covertest.i64CallbackRecordCallback?, p0_w4: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_w2 == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Long, raw2: String?) {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        lateinit var callbackArgument1_0: io.prebindgen.covertest.callbacks.CallbackRecord
        try {
            callbackArgument0_0 = raw0 as Long
            callbackArgument1_0 = io.prebindgen.covertest.callbacks.CallbackRecord(raw1 as Long, raw2 as String)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_w2.run(callbackArgument0_0, callbackArgument1_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w4 == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_w4.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_set_fire_66ee90d277a83d28(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackConstructorNested`(p0_w3: io.prebindgen.covertest.i64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_w3 == null) null else object {
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
        p0_w3.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_nested_cb4b380338cdfee2(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackConstructorSpawn`(p0_w2: io.prebindgen.covertest.i64Callback?, p1_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbacks.CallbackConstructorJob> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.callbacks.CallbackConstructorJob = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_w2 == null) null else object {
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
        p0_w2.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_spawn_34c96d991b52d675(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.callbacks.CallbackConstructorJob? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackConstructorJob_c88b6ee076189420(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.callbacks.CallbackConstructorJob? = null
        try {
            output0 = io.prebindgen.covertest.callbacks.CallbackConstructorJob(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackConstructorJob_c88b6ee076189420(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `callbackConstructorRun`(p0_w1: io.prebindgen.covertest.callbacks.CallbackConstructorJob?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_run_a024114588f63dc3(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0]) { if (true) p0_w1!!.markConsumed() } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackConstructorSelected`(p0_w1: Int, p0_w4: io.prebindgen.covertest.i64Callback?, p0_w6: io.prebindgen.covertest.callbacks.CallbackConstructor?, p1_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (p0_w1 == 1) requireNotNull(p0_w6) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w4 == null) null else object {
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
        p0_w4.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p0_w6?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_selected_e1407ac02615a674(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } finally { if (ownership[0]) { if (p0_w1 == 1) p0_w6!!.markConsumed() } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackConstructorOptional`(p0_w4: io.prebindgen.covertest.i64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Boolean> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { (if (p0_w4 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w4 != null) if (requireNotNull(p0_w4) == null) null else object {
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
        requireNotNull(p0_w4).run(callbackArgument0_0)

    }
} else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_optional_7e8221b306fdc76a(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackConstructorList`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.i64Callback?>, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w6 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(if (p0_w6 == null) null else object {
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
        p0_w6.run(callbackArgument0_0)

    }
}) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_list_9de391e0ca3c5896(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackConstructorChecked`(p0_w2: io.prebindgen.covertest.i64Callback?, p1_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_w2 == null) null else object {
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
        p0_w2.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_constructor_checked_68d8204b4d954b4b(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackSetNew`(p0_callback: io.prebindgen.covertest.i64CallbackRecordCallback?, p1_callback: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbacks.CallbackSet> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.callbacks.CallbackSet = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Long, raw2: String?) {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        lateinit var callbackArgument1_0: io.prebindgen.covertest.callbacks.CallbackRecord
        try {
            callbackArgument0_0 = raw0 as Long
            callbackArgument1_0 = io.prebindgen.covertest.callbacks.CallbackRecord(raw1 as Long, raw2 as String)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0, callbackArgument1_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p1_callback.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_set_new_84d26d4b7902b874(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.callbacks.CallbackSet? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackSet_42869ab2828f84d4(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.callbacks.CallbackSet? = null
        try {
            output0 = io.prebindgen.covertest.callbacks.CallbackSet(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackSet_42869ab2828f84d4(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `callbackSetFire`(p0_w1: io.prebindgen.covertest.callbacks.CallbackSet?, p1_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_set_fire_c9c8320f5ea50f3d(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackSetSpawn`(p0_w1: io.prebindgen.covertest.callbacks.CallbackSet?, p1_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbacks.CallbackJob> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.callbacks.CallbackJob = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_set_spawn_b320f0fcb65848c6(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.callbacks.CallbackJob? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackJob_7f5ec1b2a514af2f(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.callbacks.CallbackJob? = null
        try {
            output0 = io.prebindgen.covertest.callbacks.CallbackJob(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackJob_7f5ec1b2a514af2f(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `callbackJobJoin`(p0_w1: io.prebindgen.covertest.callbacks.CallbackJob?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_job_join_c037874626d44faf(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0]) { if (true) p0_w1!!.markConsumed() } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackOwnedNew`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbacks.CallbackOwned> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.callbacks.CallbackOwned = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_owned_new_1ad5877d98b31f02(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.callbacks.CallbackOwned? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.callbacks.CallbackOwned? = null
        try {
            output0 = io.prebindgen.covertest.callbacks.CallbackOwned(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `callbackOwnedId`(p0_w1: io.prebindgen.covertest.callbacks.CallbackOwned?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_owned_id_03a7cb65fe6b1309(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackOwnedDrops`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_owned_drops_d617a886f0baab03(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun `callbackOwnedFire`(p0_callback: io.prebindgen.covertest.callbacks.CallbackOwnedCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbacks.CallbackOwned
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackOutput0_0 = io.prebindgen.covertest.callbacks.CallbackOwned(raw0 as Long)
            callbackArgument0_0 = requireNotNull(callbackOutput0_0)
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_owned_fire_5ac0f47b60fb0ab1(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackOwnedBorrow`(p0_w1: io.prebindgen.covertest.callbacks.CallbackOwned?, p1_callback: io.prebindgen.covertest.callbacks.CallbackOwnedCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbacks.CallbackOwned
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackOutput0_0 = io.prebindgen.covertest.callbacks.CallbackOwned(raw0 as Long)
            callbackArgument0_0 = requireNotNull(callbackOutput0_0)
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p1_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_owned_borrow_37039e3928d2d226(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackTransferredId`(p0_w1: io.prebindgen.covertest.callbacks.CallbackTransferred?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_transferred_id_eed01c8238b82f9a(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackTransferredDrops`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_transferred_drops_4ff92471b79e1575(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun `callbackTransferredFire`(p0_callback: io.prebindgen.covertest.callbacks.CallbackTransferredOptionalCallbackTransferredCallback?, p1_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_0: io.prebindgen.covertest.callbacks.CallbackTransferred? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbacks.CallbackTransferred
        var callbackOutput1_0: io.prebindgen.covertest.callbacks.CallbackTransferred? = null
        var callbackArgument1_0: io.prebindgen.covertest.callbacks.CallbackTransferred? = null
        try {
            callbackOutput0_0 = io.prebindgen.covertest.callbacks.CallbackTransferred(raw0 as Long)
            callbackArgument0_0 = requireNotNull(callbackOutput0_0)
            if (raw1 != 0L) {
            callbackOutput1_0 = io.prebindgen.covertest.callbacks.CallbackTransferred(raw1 as Long)
            }
            callbackArgument1_0 = if (raw1 != 0L) requireNotNull(callbackOutput1_0) else null
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackTransferred_eab9ef9641575576(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            if (raw1 != 0L) {
            try { val owner = callbackOutput1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackTransferred_eab9ef9641575576(raw1 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0, callbackArgument1_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_transferred_fire_b5720cc21e6a881a(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackProjectionFire`(p0_callback: io.prebindgen.covertest.callbacks.CallbackProjectedCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_g1_0: io.prebindgen.covertest.callbacks.CallbackProjected? = null
        var callbackArgument0_0: Long = 0L
        lateinit var callbackArgument0_1: io.prebindgen.covertest.callbacks.CallbackProjected
        try {
            callbackOutput0_g1_0 = io.prebindgen.covertest.callbacks.CallbackProjected(raw1 as Long)
            callbackArgument0_0 = raw0 as Long
            callbackArgument0_1 = requireNotNull(callbackOutput0_g1_0)
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackProjected_74e71cedcb164780(raw1 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0, callbackArgument0_1)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_projection_fire_fab0fb05d382a2b3(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackProjectedId`(p0_w1: io.prebindgen.covertest.callbacks.CallbackProjected?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_projected_id_79c245701232b63c(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `callbackPreflight`(p0_w1: io.prebindgen.covertest.callbacks.CallbackOwned?, p1_callback: io.prebindgen.covertest.VoidCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run() {
        var callbackCleanupFailure: Throwable? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p1_callback.run()

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_preflight_a67d235ef587d6e2(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0]) { if (true) p0_w1!!.markConsumed() } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackScalars`(p0_callback: io.prebindgen.covertest.i64i64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long = 0L
        var callbackArgument1_0: Long = 0L
        try {
            callbackArgument0_0 = raw0 as Long
            callbackArgument1_0 = raw1 as Long
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0, callbackArgument1_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_scalars_107e667aaf4d1540(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackTexts`(p0_callback: io.prebindgen.covertest.stru8Listi64VecCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: String?, raw1: ByteArray?, raw2: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: String
        lateinit var callbackArgument1_0: ByteArray
var callbackOutput2_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput2_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput2_bulkList: kotlin.collections.MutableList<Long>? = null
        lateinit var callbackArgument2_0: kotlin.collections.List<Long>
        try {
            callbackArgument0_0 = raw0 as String
            callbackArgument1_0 = raw1 as ByteArray
callbackOutput2_bulkReader = io.prebindgen.covertest.BulkReader(raw2 as ByteArray)
callbackOutput2_bulkRange = requireNotNull(callbackOutput2_bulkReader).rows(((requireNotNull(callbackOutput2_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
callbackOutput2_bulkList = kotlin.collections.ArrayList(callbackOutput2_bulkRange!!.length)
for (callbackOutput2_bulkIndex in 0 until callbackOutput2_bulkRange!!.length) {
val callbackOutput2_bulkBase = callbackOutput2_bulkRange!!.offset + callbackOutput2_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


callbackOutput2_bulkList!!.add((requireNotNull(callbackOutput2_bulkReader).long((callbackOutput2_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}
            callbackArgument2_0 = requireNotNull(callbackOutput2_bulkList)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0, callbackArgument1_0, callbackArgument2_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_texts_c776f9ba3bb1e5fa(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackOptionalOwned`(p0_callback: io.prebindgen.covertest.callbacks.OptionalCallbackOwnedCallback?, p1_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long) {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null
        var callbackArgument0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            if (raw0 != 0L) {
            callbackOutput0_0 = io.prebindgen.covertest.callbacks.CallbackOwned(raw0 as Long)
            }
            callbackArgument0_0 = if (raw0 != 0L) requireNotNull(callbackOutput0_0) else null
        } catch (failure: Throwable) {
            if (raw0 != 0L) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            if (raw0 != 0L) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_optional_owned_c339b71b13e68762(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackEnvelope`(p0_callback: io.prebindgen.covertest.callbacks.CallbackEnvelopeCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: String?) {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_g0_0: io.prebindgen.covertest.callbacks.CallbackOwned? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbacks.CallbackEnvelope
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackOutput0_g0_0 = io.prebindgen.covertest.callbacks.CallbackOwned(raw0 as Long)
            callbackArgument0_0 = io.prebindgen.covertest.callbacks.CallbackEnvelope(requireNotNull(callbackOutput0_g0_0), raw1 as String)
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            try { val owner = callbackOutput0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackOwned_d936a95539d532a5(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_envelope_bede25767fde1a47(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackPrimitives`(p0_callback: io.prebindgen.covertest.boolcharu8u16u32u64i128u128f32f64Callback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Boolean, raw1: Int, raw2: Int, raw3: Int, raw4: Long, raw5: Long, raw6: ByteArray?, raw7: ByteArray?, raw8: Float, raw9: Double) {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Boolean = false
        var callbackArgument1_0: Int = 0
        var callbackArgument2_0: Int = 0
        var callbackArgument3_0: Int = 0
        var callbackArgument4_0: Long = 0L
        var callbackArgument5_0: ULong = 0UL
        lateinit var callbackArgument6_0: java.math.BigInteger
        lateinit var callbackArgument7_0: java.math.BigInteger
        var callbackArgument8_0: Float = 0.0f
        var callbackArgument9_0: Double = 0.0
        try {
            callbackArgument0_0 = raw0 as Boolean
            callbackArgument1_0 = raw1 as Int
            callbackArgument2_0 = raw2 as Int
            callbackArgument3_0 = raw3 as Int
            callbackArgument4_0 = raw4 as Long
            callbackArgument5_0 = (raw5 as Long).toULong()
            callbackArgument6_0 = java.math.BigInteger(raw6 as ByteArray)
            callbackArgument7_0 = java.math.BigInteger(1, raw7 as ByteArray)
            callbackArgument8_0 = raw8 as Float
            callbackArgument9_0 = raw9 as Double
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        p0_callback.run(callbackArgument0_0, callbackArgument1_0, callbackArgument2_0, callbackArgument3_0, callbackArgument4_0, callbackArgument5_0, callbackArgument6_0, callbackArgument7_0, callbackArgument8_0, callbackArgument9_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_primitives_ff126f0005bfa2b5(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `callbackEnums`(p0_callback: io.prebindgen.covertest.enums.PriorityOptionalPriorityCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int, raw1: Int) {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.enums.Priority
        var callbackArgument1_0: io.prebindgen.covertest.enums.Priority? = null
        var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackArgument0_0 = io.prebindgen.covertest.enums.Priority.fromWire(raw0 as Int)
            if (raw1 != -1) {
            }
            callbackArgument1_0 = if (raw1 != -1) io.prebindgen.covertest.enums.Priority.fromWire(raw1 as Int) else null
        } catch (failure: Throwable) {
            if (raw1 != -1) {
            }
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
            callbackReady = true
            p0_callback.run(callbackArgument0_0, callbackArgument1_0)
        } catch (failure: Throwable) {
            callbackFailure = failure
            throw failure
        } finally {
            if (callbackReady) {
            if (raw1 != -1) {
            }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        }

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_enums_b6723b6e70db554a(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}
