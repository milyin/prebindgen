@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.rawinputs

public fun bulkReplayRows(rows: kotlin.collections.List<io.prebindgen.covertest.rawinputs.BulkReplayRow>, onError: io.prebindgen.covertest.rawinputs.JniErrorHandler<ULong> = io.prebindgen.covertest.rawinputs.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w3_elements = try { if (true) rows?.toList() else null } catch (failure: io.prebindgen.covertest.rawinputs.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.rawinputs.BulkInputs.Input_Range_9c778fd2ce1e4e75()
val p0_w3_bulkList = io.prebindgen.covertest.rawinputs.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r9c778fd2ce1e4e75.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181)
p0_w1_writer.range((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r9c778fd2ce1e4e75.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val __pb_p0_w7_field0 = p0_w7.first
val __pb_p0_w7_field1 = p0_w7.text
val __pb_p0_w7_field2 = p0_w7.bytes
val __pb_p0_w7_field3 = p0_w7.kind
val __pb_p0_w7_field4 = p0_w7.last
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.offsets[0], io.prebindgen.covertest.rawinputs.InputValidation.value { __pb_p0_w7_field0 }, true, 0)
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.offsets[1], io.prebindgen.covertest.rawinputs.InputValidation.value { (if (__pb_p0_w7_field1 != null) 1 else 0) })
p0_w1_writer.text((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.offsets[2], io.prebindgen.covertest.rawinputs.InputValidation.value { if (__pb_p0_w7_field1 != null) requireNotNull(__pb_p0_w7_field1) else null })
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.offsets[3], io.prebindgen.covertest.rawinputs.InputValidation.value { (if (__pb_p0_w7_field2 != null) 1 else 0) })
p0_w1_writer.bytes((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.offsets[4], io.prebindgen.covertest.rawinputs.InputValidation.value { if (__pb_p0_w7_field2 != null) requireNotNull(__pb_p0_w7_field2) else null })
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.offsets[5], io.prebindgen.covertest.rawinputs.InputValidation.value { if (__pb_p0_w7_field3 != null) run { val value = requireNotNull(__pb_p0_w7_field3).wireValue; require(value != -1) { "invalid option payload" }; value } else -1 })
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.re7fb16c195e57181.offsets[6], io.prebindgen.covertest.rawinputs.InputValidation.value { __pb_p0_w7_field4 }, true, 0)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.rawinputs.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.rawinputs.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.rawinputs.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.rawinputs.JNINative.call_bulk_replay_rows_647e979b1d69f1d8(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.rawinputs.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.rawinputs.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.rawinputs.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkNestedHandlesInput(values: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.rawinputs.BulkHandle?>?>, onError: io.prebindgen.covertest.rawinputs.JniErrorHandler<ULong> = io.prebindgen.covertest.rawinputs.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w4_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.rawinputs.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.rawinputs.BulkInputs.Input_Range_5019fbfb9686d1a8()
val p0_w4_bulkList = io.prebindgen.covertest.rawinputs.InputValidation.value { p0_w4_elements }
if (p0_w4_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r5019fbfb9686d1a8.offsets[0]) } else {
val p0_w4_bulkStart = p0_w1_writer.rows(p0_w4_bulkList.size, io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6)
p0_w1_writer.range((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r5019fbfb9686d1a8.offsets[0], p0_w4_bulkStart, p0_w4_bulkList.size)
p0_w4_bulkList.forEachIndexed { p0_w4_bulkIndex, p0_w11 ->
val p0_w11_elements = if (p0_w11 != null) (if (true) requireNotNull(p0_w11).toList() else null) else (null)
p0_w1_writer.int((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.offsets[0], io.prebindgen.covertest.rawinputs.InputValidation.value { (if (p0_w11 != null) 1 else 0) })
val p0_w11_bulkList = io.prebindgen.covertest.rawinputs.InputValidation.value { if (p0_w11 != null) p0_w11_elements else null }
if (p0_w11_bulkList == null) { p0_w1_writer.absent((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.offsets[1]) } else {
val p0_w11_bulkStart = p0_w1_writer.rows(p0_w11_bulkList.size, io.prebindgen.covertest.rawinputs.BulkLayouts.rcbe830243575e259)
p0_w1_writer.range((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.offsets[1], p0_w11_bulkStart, p0_w11_bulkList.size)
p0_w11_bulkList.forEachIndexed { p0_w11_bulkIndex, p0_w18 ->
p0_w1_writer.handle((p0_w11_bulkStart + p0_w11_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.rcbe830243575e259.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.rcbe830243575e259.offsets[0], io.prebindgen.covertest.rawinputs.InputValidation.value { if (p0_w18 != null) requireNotNull(if (true) requireNotNull(p0_w18) else null) { "invalid option payload" } else null }, true, 0)
}
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.rawinputs.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.rawinputs.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (p0_w1_writer_state?.handles.orEmpty()) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.rawinputs.withHandleLocks(handles) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.rawinputs.JNINative.call_bulk_nested_handles_input_20a8692eaca21fba(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.rawinputs.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.rawinputs.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume); handles.forEach { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.rawinputs.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkCleanupNestedCallback(callback: io.prebindgen.covertest.rawinputs.BulkCleanupNestedCallback, retry: Boolean, worker: Boolean, onError: io.prebindgen.covertest.rawinputs.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.rawinputs.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val returnCapture = io.prebindgen.covertest.rawinputs.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.rawinputs.BulkHandle?>?>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.rawinputs.BulkInputs.Input_Range_5019fbfb9686d1a8()
val callback_return_w3_bulkList = io.prebindgen.covertest.rawinputs.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r5019fbfb9686d1a8.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6)
callback_return_w1_writer.range((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r5019fbfb9686d1a8.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w9 ->
val callback_return_w9_elements = if (callback_return_w9 != null) (if (true) requireNotNull(callback_return_w9).toList() else null) else (null)
callback_return_w1_writer.int((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.offsets[0], io.prebindgen.covertest.rawinputs.InputValidation.value { (if (callback_return_w9 != null) 1 else 0) })
val callback_return_w9_bulkList = io.prebindgen.covertest.rawinputs.InputValidation.value { if (callback_return_w9 != null) callback_return_w9_elements else null }
if (callback_return_w9_bulkList == null) { callback_return_w1_writer.absent((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.offsets[1]) } else {
val callback_return_w9_bulkStart = callback_return_w1_writer.rows(callback_return_w9_bulkList.size, io.prebindgen.covertest.rawinputs.BulkLayouts.rcbe830243575e259)
callback_return_w1_writer.range((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.rf27ab2e6065597a6.offsets[1], callback_return_w9_bulkStart, callback_return_w9_bulkList.size)
callback_return_w9_bulkList.forEachIndexed { callback_return_w9_bulkIndex, callback_return_w15 ->
callback_return_w1_writer.handle((callback_return_w9_bulkStart + callback_return_w9_bulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.rcbe830243575e259.size) + io.prebindgen.covertest.rawinputs.BulkLayouts.rcbe830243575e259.offsets[0], io.prebindgen.covertest.rawinputs.InputValidation.value { if (callback_return_w15 != null) requireNotNull(if (true) requireNotNull(callback_return_w15) else null) { "invalid option payload" } else null }, true, 0)
}
}
}
}

callback_return_w1_writer } else null
 io.prebindgen.covertest.rawinputs.withHandleLocks((callback_return_w1_writer_state?.handles.orEmpty())) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.rawinputs.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.rawinputs.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.rawinputs.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? Iterable<*>)?.forEach { returnedValue3 -> (returnedValue3 as? io.prebindgen.covertest.rawinputs.NativeHandle)?.let { returnedValue5 -> returnedHandles.add(returnedValue5); returnedOwners.add(returnedValue5) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { retry } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { worker } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.rawinputs.JNINative.call_bulk_cleanup_nested_callback_1167603045ff0977(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: io.prebindgen.covertest.rawinputs.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.rawinputs.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.rawinputs.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
outputbulkReader = io.prebindgen.covertest.rawinputs.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.rawinputs.BulkRange?), io.prebindgen.covertest.rawinputs.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.rawinputs.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.rawinputs.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun bulkCleanupPairCallback(callback: io.prebindgen.covertest.rawinputs.BulkCleanupPairCallback, onError: io.prebindgen.covertest.rawinputs.JniErrorHandler<ULong> = io.prebindgen.covertest.rawinputs.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.rawinputs.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: io.prebindgen.covertest.rawinputs.BulkPairText
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val __pb_callback_return_w3_field0 = callback_return_w3.borrowed_handle
val __pb_callback_return_w3_field1 = callback_return_w3.owned_handle
val __pb_callback_return_w3_field2 = callback_return_w3.text
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.rawinputs.BulkInputs.Input_i64_i64_8afc2761c446bf98()
callback_return_w1_writer.handle((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r8afc2761c446bf98.offsets[0], io.prebindgen.covertest.rawinputs.InputValidation.value { __pb_callback_return_w3_field0 }, false, 0)
callback_return_w1_writer.handle((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r8afc2761c446bf98.offsets[1], io.prebindgen.covertest.rawinputs.InputValidation.value { __pb_callback_return_w3_field1 }, true, 1)
callback_return_w1_writer.text((0) + io.prebindgen.covertest.rawinputs.BulkLayouts.r8afc2761c446bf98.offsets[2], io.prebindgen.covertest.rawinputs.InputValidation.value { __pb_callback_return_w3_field2 })

callback_return_w1_writer } else null
 io.prebindgen.covertest.rawinputs.withHandleLocks((callback_return_w1_writer_state?.handles.orEmpty())) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.rawinputs.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.rawinputs.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.rawinputs.NativeHandle>(); (callback_return_w3 as? io.prebindgen.covertest.rawinputs.BulkPairText)?.let { returnedValue1 -> (returnedValue1.borrowed_handle as? io.prebindgen.covertest.rawinputs.NativeHandle)?.let { returnedValue5 -> returnedHandles.add(returnedValue5);  }
(returnedValue1.owned_handle as? io.prebindgen.covertest.rawinputs.NativeHandle)?.let { returnedValue7 -> returnedHandles.add(returnedValue7); returnedOwners.add(returnedValue7) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.rawinputs.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.rawinputs.JNINative.call_bulk_cleanup_pair_callback_75c0d45b83cad591(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.rawinputs.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}
