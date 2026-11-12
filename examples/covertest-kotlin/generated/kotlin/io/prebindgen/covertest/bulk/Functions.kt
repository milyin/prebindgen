@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.bulk

public fun bulkReplayRows(rows: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkReplayRow>, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w3_elements = try { if (true) rows?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_55fb9dde8a946716()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val __pb_p0_w7_field0 = p0_w7.first
val __pb_p0_w7_field1 = p0_w7.text
val __pb_p0_w7_field2 = p0_w7.bytes
val __pb_p0_w7_field3 = p0_w7.kind
val __pb_p0_w7_field4 = p0_w7.last
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_p0_w7_field0 }, true, 0)
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[1], io.prebindgen.covertest.InputValidation.value { (if (__pb_p0_w7_field1 != null) 1 else 0) })
if (((if (__pb_p0_w7_field1 != null) 1 else 0)) == 1) {
p0_w1_writer.text((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[2], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w7_field1 != null) requireNotNull(__pb_p0_w7_field1) else null })
}
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[3], io.prebindgen.covertest.InputValidation.value { (if (__pb_p0_w7_field2 != null) 1 else 0) })
if (((if (__pb_p0_w7_field2 != null) 1 else 0)) == 1) {
p0_w1_writer.bytes((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[4], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w7_field2 != null) requireNotNull(__pb_p0_w7_field2) else null })
}
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[5], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w7_field3 != null) run { val value = requireNotNull(__pb_p0_w7_field3).wireValue; require(value != -1) { "invalid option payload" }; value } else -1 })
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[6], io.prebindgen.covertest.InputValidation.value { __pb_p0_w7_field4 }, true, 0)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_replay_rows_aafaa0d2a20132d8(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkReplayRedundantRows(rows: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkReplayRow>, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w4_elements = try { if (true) rows?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_55fb9dde8a946716()
val p0_w4_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w4_elements }
if (p0_w4_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.offsets[0]) } else {
val p0_w4_bulkStart = p0_w1_writer.rows(p0_w4_bulkList.size, io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.offsets[0], p0_w4_bulkStart, p0_w4_bulkList.size)
p0_w4_bulkList.forEachIndexed { p0_w4_bulkIndex, p0_w8 ->
val __pb_p0_w8_field0 = p0_w8.first
val __pb_p0_w8_field1 = p0_w8.text
val __pb_p0_w8_field2 = p0_w8.bytes
val __pb_p0_w8_field3 = p0_w8.kind
val __pb_p0_w8_field4 = p0_w8.last
p0_w1_writer.handle((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_p0_w8_field0 }, true, 0)
p0_w1_writer.int((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[1], io.prebindgen.covertest.InputValidation.value { (if (__pb_p0_w8_field1 != null) 1 else 0) })
if (((if (__pb_p0_w8_field1 != null) 1 else 0)) == 1) {
p0_w1_writer.text((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[2], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w8_field1 != null) requireNotNull(__pb_p0_w8_field1) else null })
}
p0_w1_writer.int((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[3], io.prebindgen.covertest.InputValidation.value { (if (__pb_p0_w8_field2 != null) 1 else 0) })
if (((if (__pb_p0_w8_field2 != null) 1 else 0)) == 1) {
p0_w1_writer.bytes((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[4], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w8_field2 != null) requireNotNull(__pb_p0_w8_field2) else null })
}
p0_w1_writer.int((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[5], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w8_field3 != null) run { val value = requireNotNull(__pb_p0_w8_field3).wireValue; require(value != -1) { "invalid option payload" }; value } else -1 })
p0_w1_writer.handle((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[6], io.prebindgen.covertest.InputValidation.value { __pb_p0_w8_field4 }, true, 0)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_replay_redundant_rows_53c9ef64704ee61d(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkReplayNestedRows(rows: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkReplayRow>>, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w4_elements = try { if (true) rows?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_3087f4c8799259e2()
val p0_w4_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w4_elements }
if (p0_w4_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r3087f4c8799259e2.offsets[0]) } else {
val p0_w4_bulkStart = p0_w1_writer.rows(p0_w4_bulkList.size, io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r3087f4c8799259e2.offsets[0], p0_w4_bulkStart, p0_w4_bulkList.size)
p0_w4_bulkList.forEachIndexed { p0_w4_bulkIndex, p0_w10 ->
val p0_w10_elements = if (true) p0_w10?.toList() else null
val p0_w10_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w10_elements }
if (p0_w10_bulkList == null) { p0_w1_writer.absent((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.size) + io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.offsets[0]) } else {
val p0_w10_bulkStart = p0_w1_writer.rows(p0_w10_bulkList.size, io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde)
p0_w1_writer.range((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.size) + io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.offsets[0], p0_w10_bulkStart, p0_w10_bulkList.size)
p0_w10_bulkList.forEachIndexed { p0_w10_bulkIndex, p0_w14 ->
val __pb_p0_w14_field0 = p0_w14.first
val __pb_p0_w14_field1 = p0_w14.text
val __pb_p0_w14_field2 = p0_w14.bytes
val __pb_p0_w14_field3 = p0_w14.kind
val __pb_p0_w14_field4 = p0_w14.last
p0_w1_writer.handle((p0_w10_bulkStart + p0_w10_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_p0_w14_field0 }, true, 0)
p0_w1_writer.int((p0_w10_bulkStart + p0_w10_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[1], io.prebindgen.covertest.InputValidation.value { (if (__pb_p0_w14_field1 != null) 1 else 0) })
if (((if (__pb_p0_w14_field1 != null) 1 else 0)) == 1) {
p0_w1_writer.text((p0_w10_bulkStart + p0_w10_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[2], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w14_field1 != null) requireNotNull(__pb_p0_w14_field1) else null })
}
p0_w1_writer.int((p0_w10_bulkStart + p0_w10_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[3], io.prebindgen.covertest.InputValidation.value { (if (__pb_p0_w14_field2 != null) 1 else 0) })
if (((if (__pb_p0_w14_field2 != null) 1 else 0)) == 1) {
p0_w1_writer.bytes((p0_w10_bulkStart + p0_w10_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[4], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w14_field2 != null) requireNotNull(__pb_p0_w14_field2) else null })
}
p0_w1_writer.int((p0_w10_bulkStart + p0_w10_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[5], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w14_field3 != null) run { val value = requireNotNull(__pb_p0_w14_field3).wireValue; require(value != -1) { "invalid option payload" }; value } else -1 })
p0_w1_writer.handle((p0_w10_bulkStart + p0_w10_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[6], io.prebindgen.covertest.InputValidation.value { __pb_p0_w14_field4 }, true, 0)
}
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_replay_nested_rows_fd3636f6cc750f4e(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkReplayNestedCallback(callback: io.prebindgen.covertest.BulkReplayNestedCallback, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkReplayRow>>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_3087f4c8799259e2()
val callback_return_w3_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r3087f4c8799259e2.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716)
callback_return_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r3087f4c8799259e2.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w8 ->
val callback_return_w8_elements = if (true) callback_return_w8?.toList() else null
val callback_return_w8_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w8_elements }
if (callback_return_w8_bulkList == null) { callback_return_w1_writer.absent((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.size) + io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.offsets[0]) } else {
val callback_return_w8_bulkStart = callback_return_w1_writer.rows(callback_return_w8_bulkList.size, io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde)
callback_return_w1_writer.range((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.size) + io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.offsets[0], callback_return_w8_bulkStart, callback_return_w8_bulkList.size)
callback_return_w8_bulkList.forEachIndexed { callback_return_w8_bulkIndex, callback_return_w12 ->
val __pb_callback_return_w12_field0 = callback_return_w12.first
val __pb_callback_return_w12_field1 = callback_return_w12.text
val __pb_callback_return_w12_field2 = callback_return_w12.bytes
val __pb_callback_return_w12_field3 = callback_return_w12.kind
val __pb_callback_return_w12_field4 = callback_return_w12.last
callback_return_w1_writer.handle((callback_return_w8_bulkStart + callback_return_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w12_field0 }, true, 0)
callback_return_w1_writer.int((callback_return_w8_bulkStart + callback_return_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[1], io.prebindgen.covertest.InputValidation.value { (if (__pb_callback_return_w12_field1 != null) 1 else 0) })
if (((if (__pb_callback_return_w12_field1 != null) 1 else 0)) == 1) {
callback_return_w1_writer.text((callback_return_w8_bulkStart + callback_return_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[2], io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w12_field1 != null) requireNotNull(__pb_callback_return_w12_field1) else null })
}
callback_return_w1_writer.int((callback_return_w8_bulkStart + callback_return_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[3], io.prebindgen.covertest.InputValidation.value { (if (__pb_callback_return_w12_field2 != null) 1 else 0) })
if (((if (__pb_callback_return_w12_field2 != null) 1 else 0)) == 1) {
callback_return_w1_writer.bytes((callback_return_w8_bulkStart + callback_return_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[4], io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w12_field2 != null) requireNotNull(__pb_callback_return_w12_field2) else null })
}
callback_return_w1_writer.int((callback_return_w8_bulkStart + callback_return_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[5], io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w12_field3 != null) run { val value = requireNotNull(__pb_callback_return_w12_field3).wireValue; require(value != -1) { "invalid option payload" }; value } else -1 })
callback_return_w1_writer.handle((callback_return_w8_bulkStart + callback_return_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[6], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w12_field4 }, true, 0)
}
}
}
}

callback_return_w1_writer } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? Iterable<*>)?.forEach { returnedValue2 -> (returnedValue2 as? io.prebindgen.covertest.bulk.BulkReplayRow)?.let { returnedValue3 -> (returnedValue3.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue5 -> returnedHandles.add(returnedValue5); returnedOwners.add(returnedValue5) }
(returnedValue3.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue7 -> returnedHandles.add(returnedValue7); returnedOwners.add(returnedValue7) }
 }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_replay_nested_callback_43b7f2dcc306c446(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkReplayObjectRows(rows: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkReplayRow>, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w3_elements = try { if (true) rows?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_55fb9dde8a946716()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val __pb_p0_w7_field0 = p0_w7.first
val __pb_p0_w7_field1 = p0_w7.text
val __pb_p0_w7_field2 = p0_w7.bytes
val __pb_p0_w7_field3 = p0_w7.kind
val __pb_p0_w7_field4 = p0_w7.last
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_p0_w7_field0 }, true, 0)
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[1], io.prebindgen.covertest.InputValidation.value { (if (__pb_p0_w7_field1 != null) 1 else 0) })
if (((if (__pb_p0_w7_field1 != null) 1 else 0)) == 1) {
p0_w1_writer.text((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[2], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w7_field1 != null) requireNotNull(__pb_p0_w7_field1) else null })
}
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[3], io.prebindgen.covertest.InputValidation.value { (if (__pb_p0_w7_field2 != null) 1 else 0) })
if (((if (__pb_p0_w7_field2 != null) 1 else 0)) == 1) {
p0_w1_writer.bytes((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[4], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w7_field2 != null) requireNotNull(__pb_p0_w7_field2) else null })
}
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[5], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w7_field3 != null) run { val value = requireNotNull(__pb_p0_w7_field3).wireValue; require(value != -1) { "invalid option payload" }; value } else -1 })
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[6], io.prebindgen.covertest.InputValidation.value { __pb_p0_w7_field4 }, true, 0)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_replay_object_rows_be4c714a838acb22(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkReplayCallback(callback: io.prebindgen.covertest.BulkReplayCallback, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkReplayRow>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_55fb9dde8a946716()
val callback_return_w3_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde)
callback_return_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r55fb9dde8a946716.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w7 ->
val __pb_callback_return_w7_field0 = callback_return_w7.first
val __pb_callback_return_w7_field1 = callback_return_w7.text
val __pb_callback_return_w7_field2 = callback_return_w7.bytes
val __pb_callback_return_w7_field3 = callback_return_w7.kind
val __pb_callback_return_w7_field4 = callback_return_w7.last
callback_return_w1_writer.handle((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w7_field0 }, true, 0)
callback_return_w1_writer.int((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[1], io.prebindgen.covertest.InputValidation.value { (if (__pb_callback_return_w7_field1 != null) 1 else 0) })
if (((if (__pb_callback_return_w7_field1 != null) 1 else 0)) == 1) {
callback_return_w1_writer.text((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[2], io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w7_field1 != null) requireNotNull(__pb_callback_return_w7_field1) else null })
}
callback_return_w1_writer.int((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[3], io.prebindgen.covertest.InputValidation.value { (if (__pb_callback_return_w7_field2 != null) 1 else 0) })
if (((if (__pb_callback_return_w7_field2 != null) 1 else 0)) == 1) {
callback_return_w1_writer.bytes((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[4], io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w7_field2 != null) requireNotNull(__pb_callback_return_w7_field2) else null })
}
callback_return_w1_writer.int((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[5], io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w7_field3 != null) run { val value = requireNotNull(__pb_callback_return_w7_field3).wireValue; require(value != -1) { "invalid option payload" }; value } else -1 })
callback_return_w1_writer.handle((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.size) + io.prebindgen.covertest.BulkLayouts.r01496d6ef6ccdfde.offsets[6], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w7_field4 }, true, 0)
}
}

callback_return_w1_writer } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? io.prebindgen.covertest.bulk.BulkReplayRow)?.let { returnedValue2 -> (returnedValue2.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue4 -> returnedHandles.add(returnedValue4); returnedOwners.add(returnedValue4) }
(returnedValue2.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue6 -> returnedHandles.add(returnedValue6); returnedOwners.add(returnedValue6) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_replay_callback_929e98d2cfd600ef(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkNestedHandlesInput(values: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>?>, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val p0_w4_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_5019fbfb9686d1a8()
val p0_w4_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w4_elements }
if (p0_w4_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r5019fbfb9686d1a8.offsets[0]) } else {
val p0_w4_bulkStart = p0_w1_writer.rows(p0_w4_bulkList.size, io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r5019fbfb9686d1a8.offsets[0], p0_w4_bulkStart, p0_w4_bulkList.size)
p0_w4_bulkList.forEachIndexed { p0_w4_bulkIndex, p0_w11 ->
val p0_w11_elements = if (p0_w11 != null) (if (true) requireNotNull(p0_w11).toList() else null) else (null)
p0_w1_writer.int((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (p0_w11 != null) 1 else 0) })
if (((if (p0_w11 != null) 1 else 0)) == 1) {
val p0_w11_bulkList = io.prebindgen.covertest.InputValidation.value { if (p0_w11 != null) p0_w11_elements else null }
if (p0_w11_bulkList == null) { p0_w1_writer.absent((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1]) } else {
val p0_w11_bulkStart = p0_w1_writer.rows(p0_w11_bulkList.size, io.prebindgen.covertest.BulkLayouts.rcbe830243575e259)
p0_w1_writer.range((p0_w4_bulkStart + p0_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1], p0_w11_bulkStart, p0_w11_bulkList.size)
p0_w11_bulkList.forEachIndexed { p0_w11_bulkIndex, p0_w18 ->
p0_w1_writer.handle((p0_w11_bulkStart + p0_w11_bulkIndex * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0], io.prebindgen.covertest.InputValidation.value { if (p0_w18 != null) requireNotNull(if (true) requireNotNull(p0_w18) else null) { "invalid option payload" } else null }, true, 0)
}
}
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_nested_handles_input_dba24072c6255486(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkCleanupNestedCallback(callback: io.prebindgen.covertest.BulkCleanupNestedCallback, retry: Boolean, worker: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>?>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_5019fbfb9686d1a8()
val callback_return_w3_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r5019fbfb9686d1a8.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6)
callback_return_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r5019fbfb9686d1a8.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w9 ->
val callback_return_w9_elements = if (callback_return_w9 != null) (if (true) requireNotNull(callback_return_w9).toList() else null) else (null)
callback_return_w1_writer.int((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (callback_return_w9 != null) 1 else 0) })
if (((if (callback_return_w9 != null) 1 else 0)) == 1) {
val callback_return_w9_bulkList = io.prebindgen.covertest.InputValidation.value { if (callback_return_w9 != null) callback_return_w9_elements else null }
if (callback_return_w9_bulkList == null) { callback_return_w1_writer.absent((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1]) } else {
val callback_return_w9_bulkStart = callback_return_w1_writer.rows(callback_return_w9_bulkList.size, io.prebindgen.covertest.BulkLayouts.rcbe830243575e259)
callback_return_w1_writer.range((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1], callback_return_w9_bulkStart, callback_return_w9_bulkList.size)
callback_return_w9_bulkList.forEachIndexed { callback_return_w9_bulkIndex, callback_return_w16 ->
callback_return_w1_writer.handle((callback_return_w9_bulkStart + callback_return_w9_bulkIndex * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0], io.prebindgen.covertest.InputValidation.value { if (callback_return_w16 != null) requireNotNull(if (true) requireNotNull(callback_return_w16) else null) { "invalid option payload" } else null }, true, 0)
}
}
}
}
}

callback_return_w1_writer } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? Iterable<*>)?.forEach { returnedValue3 -> (returnedValue3 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue5 -> returnedHandles.add(returnedValue5); returnedOwners.add(returnedValue5) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { retry } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { worker } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_cleanup_nested_callback_5f8da21ad356132b(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

public fun bulkCleanupPairCallback(callback: io.prebindgen.covertest.BulkCleanupPairCallback, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: io.prebindgen.covertest.bulk.BulkPairText
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
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.BulkInputs.Input_i64_i64_8afc2761c446bf98()
callback_return_w1_writer.handle((0) + io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w3_field0 }, false, 0)
callback_return_w1_writer.handle((0) + io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.offsets[1], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w3_field1 }, true, 1)
callback_return_w1_writer.text((0) + io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.offsets[2], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w3_field2 })

callback_return_w1_writer } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handleGroup(0).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handleGroup(1).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withMixedInputLocks((__pb_handle0), (__pb_handle1)) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? io.prebindgen.covertest.bulk.BulkPairText)?.let { returnedValue1 -> (returnedValue1.borrowed_handle as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue5 -> returnedHandles.add(returnedValue5);  }
(returnedValue1.owned_handle as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue7 -> returnedHandles.add(returnedValue7); returnedOwners.add(returnedValue7) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_cleanup_pair_callback_f2f2462e96f2da09(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkReplayReset(failText: ULong, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { failText.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_replay_reset_ec26adbfb8cc43d7(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun bulkReplayCount(which: ULong, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { which.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_replay_count_0e31779505af7ccf(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkPatchRows(rowFirst: io.prebindgen.covertest.bulk.BulkHandle?, rowChildren: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>?>?, rowLast: io.prebindgen.covertest.bulk.BulkHandle, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w10_elements = try { if (rowChildren != null) (if (true) requireNotNull(rowChildren).toList() else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_i64_i32_7fe138bb3730f8e7()
p0_w1_writer.handle((0) + io.prebindgen.covertest.BulkLayouts.r7fe138bb3730f8e7.offsets[0], io.prebindgen.covertest.InputValidation.value { if (rowFirst != null) requireNotNull(if (true) requireNotNull(rowFirst) else null) { "invalid option payload" } else null }, true, 0)
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r7fe138bb3730f8e7.offsets[1], io.prebindgen.covertest.InputValidation.value { (if (rowChildren != null) 1 else 0) })
if (((if (rowChildren != null) 1 else 0)) == 1) {
val p0_w10_bulkList = io.prebindgen.covertest.InputValidation.value { if (rowChildren != null) p0_w10_elements else null }
if (p0_w10_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r7fe138bb3730f8e7.offsets[2]) } else {
val p0_w10_bulkStart = p0_w1_writer.rows(p0_w10_bulkList.size, io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r7fe138bb3730f8e7.offsets[2], p0_w10_bulkStart, p0_w10_bulkList.size)
p0_w10_bulkList.forEachIndexed { p0_w10_bulkIndex, p0_w16 ->
val p0_w16_elements = if (p0_w16 != null) (if (true) requireNotNull(p0_w16).toList() else null) else (null)
p0_w1_writer.int((p0_w10_bulkStart + p0_w10_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (p0_w16 != null) 1 else 0) })
if (((if (p0_w16 != null) 1 else 0)) == 1) {
val p0_w16_bulkList = io.prebindgen.covertest.InputValidation.value { if (p0_w16 != null) p0_w16_elements else null }
if (p0_w16_bulkList == null) { p0_w1_writer.absent((p0_w10_bulkStart + p0_w10_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1]) } else {
val p0_w16_bulkStart = p0_w1_writer.rows(p0_w16_bulkList.size, io.prebindgen.covertest.BulkLayouts.rcbe830243575e259)
p0_w1_writer.range((p0_w10_bulkStart + p0_w10_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1], p0_w16_bulkStart, p0_w16_bulkList.size)
p0_w16_bulkList.forEachIndexed { p0_w16_bulkIndex, p0_w22 ->
p0_w1_writer.handle((p0_w16_bulkStart + p0_w16_bulkIndex * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0], io.prebindgen.covertest.InputValidation.value { if (p0_w22 != null) requireNotNull(if (true) requireNotNull(p0_w22) else null) { "invalid option payload" } else null }, true, 0)
}
}
}
}
}
}
p0_w1_writer.handle((0) + io.prebindgen.covertest.BulkLayouts.r7fe138bb3730f8e7.offsets[3], io.prebindgen.covertest.InputValidation.value { rowLast }, true, 0)

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_patch_rows_d32fa8a44880666b(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

public fun bulkPatchCallback(callback: io.prebindgen.covertest.BulkPatchCallback, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>?>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_5019fbfb9686d1a8()
val callback_return_w3_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r5019fbfb9686d1a8.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6)
callback_return_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r5019fbfb9686d1a8.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w9 ->
val callback_return_w9_elements = if (callback_return_w9 != null) (if (true) requireNotNull(callback_return_w9).toList() else null) else (null)
callback_return_w1_writer.int((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (callback_return_w9 != null) 1 else 0) })
if (((if (callback_return_w9 != null) 1 else 0)) == 1) {
val callback_return_w9_bulkList = io.prebindgen.covertest.InputValidation.value { if (callback_return_w9 != null) callback_return_w9_elements else null }
if (callback_return_w9_bulkList == null) { callback_return_w1_writer.absent((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1]) } else {
val callback_return_w9_bulkStart = callback_return_w1_writer.rows(callback_return_w9_bulkList.size, io.prebindgen.covertest.BulkLayouts.rcbe830243575e259)
callback_return_w1_writer.range((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.size) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1], callback_return_w9_bulkStart, callback_return_w9_bulkList.size)
callback_return_w9_bulkList.forEachIndexed { callback_return_w9_bulkIndex, callback_return_w16 ->
callback_return_w1_writer.handle((callback_return_w9_bulkStart + callback_return_w9_bulkIndex * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0], io.prebindgen.covertest.InputValidation.value { if (callback_return_w16 != null) requireNotNull(if (true) requireNotNull(callback_return_w16) else null) { "invalid option payload" } else null }, true, 0)
}
}
}
}
}

callback_return_w1_writer } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? Iterable<*>)?.forEach { returnedValue3 -> (returnedValue3 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue5 -> returnedHandles.add(returnedValue5); returnedOwners.add(returnedValue5) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_patch_callback_22fbb64046f78357(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

public fun bulkNestedEnvelope(values: kotlin.collections.List<kotlin.collections.List<kotlin.collections.List<Long>>>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r10bed0e0898eed85.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r10bed0e0898eed85.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r83cf990f0be55ad5)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r10bed0e0898eed85.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w9 ->
val p0_w9_elements = if (true) p0_w9?.toList() else null
val p0_w9_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w9_elements }
if (p0_w9_bulkList == null) { p0_w1_writer.absent((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r83cf990f0be55ad5.size) + io.prebindgen.covertest.BulkLayouts.r83cf990f0be55ad5.offsets[0]) } else {
val p0_w9_bulkStart = p0_w1_writer.rows(p0_w9_bulkList.size, io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d)
p0_w1_writer.range((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r83cf990f0be55ad5.size) + io.prebindgen.covertest.BulkLayouts.r83cf990f0be55ad5.offsets[0], p0_w9_bulkStart, p0_w9_bulkList.size)
p0_w9_bulkList.forEachIndexed { p0_w9_bulkIndex, p0_w13 ->
val p0_w13_elements = if (true) p0_w13?.toList() else null
val p0_w13_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w13_elements }
if (p0_w13_bulkList == null) { p0_w1_writer.absent((p0_w9_bulkStart + p0_w9_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val p0_w13_bulkStart = p0_w1_writer.rows(p0_w13_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((p0_w9_bulkStart + p0_w9_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], p0_w13_bulkStart, p0_w13_bulkList.size)
p0_w13_bulkList.forEachIndexed { p0_w13_bulkIndex, p0_w17 ->
p0_w1_writer.long((p0_w13_bulkStart + p0_w13_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w17 })
}
}
}
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_nested_envelope_fac9f0f836298b23(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun bulkPairTextCallback(callback: io.prebindgen.covertest.BulkPairTextCallback, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPairText>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_2997fc5de2832bba()
val callback_return_w3_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r2997fc5de2832bba.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98)
callback_return_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r2997fc5de2832bba.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w7 ->
val __pb_callback_return_w7_field0 = callback_return_w7.borrowed_handle
val __pb_callback_return_w7_field1 = callback_return_w7.owned_handle
val __pb_callback_return_w7_field2 = callback_return_w7.text
callback_return_w1_writer.handle((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.size) + io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w7_field0 }, false, 0)
callback_return_w1_writer.handle((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.size) + io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.offsets[1], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w7_field1 }, true, 1)
callback_return_w1_writer.text((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.size) + io.prebindgen.covertest.BulkLayouts.r8afc2761c446bf98.offsets[2], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w7_field2 })
}
}

callback_return_w1_writer } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handleGroup(0).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handleGroup(1).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withMixedInputLocks((__pb_handle0), (__pb_handle1)) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? io.prebindgen.covertest.bulk.BulkPairText)?.let { returnedValue2 -> (returnedValue2.borrowed_handle as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue6 -> returnedHandles.add(returnedValue6);  }
(returnedValue2.owned_handle as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue8 -> returnedHandles.add(returnedValue8); returnedOwners.add(returnedValue8) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_pair_text_callback_45fa8b29f215e390(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkHandleTexts(values: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandleText>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandleText>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandleText> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_ff46ac585710e08a()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val __pb_p0_w7_field0 = p0_w7.handle
val __pb_p0_w7_field1 = p0_w7.text
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_p0_w7_field0 }, true, 0)
p0_w1_writer.text((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[1], io.prebindgen.covertest.InputValidation.value { __pb_p0_w7_field1 })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_texts_415eab299e560837(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandleText>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandleText>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size
        var outputbulkItem_g0_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            outputbulkItem_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[0])) as Long))

outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkHandleText(requireNotNull(outputbulkItem_g0_0), (requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[1])) as String))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun bulkCollectionLockGroups(first: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>?, second: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>, probe: io.prebindgen.covertest.CollectionLocksProbe, onError: io.prebindgen.covertest.JniErrorHandler<Boolean> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val p0_w3_elements = try { if (first != null) (if (true) requireNotNull(first).toList()?.let { p0_w3_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(p0_w3_snapshot.size) { p0_w3_index -> val p0_w7 = p0_w3_snapshot[p0_w3_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w7 }) }, intArrayOf(2), 1, null) } else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p1_w1_elements = try { if (true) second?.toList()?.let { p1_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(p1_w1_snapshot.size) { p1_w1_index -> val p1_w5 = p1_w1_snapshot[p1_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p1_w5 }) }, intArrayOf(2), 1, null) } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { if (first != null) (p0_w3_elements?.handles.orEmpty()) else kotlin.collections.emptyList() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p1_w1_elements?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks(((__pb_handle0) + (__pb_handle1)), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { if (first != null) requireNotNull(p0_w3_elements?.encode()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1_elements?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Boolean {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: Boolean = false
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = probe.run()
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_collection_lock_groups_7fe8555600f1dcf6(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (first != null) { p0_w3_elements?.consume(__pb_consume) }; p1_w1_elements?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; if (first != null) { p0_w3_elements?.consume(__pb_consume) }; p1_w1_elements?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun bulkHandleNested(values: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_46147a291c22570d()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r42eab86179b36709)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
val p0_w8_elements = if (true) p0_w8?.toList() else null
val p0_w8_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w8_elements }
if (p0_w8_bulkList == null) { p0_w1_writer.absent((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0]) } else {
val p0_w8_bulkStart = p0_w1_writer.rows(p0_w8_bulkList.size, io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09)
p0_w1_writer.range((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0], p0_w8_bulkStart, p0_w8_bulkList.size)
p0_w8_bulkList.forEachIndexed { p0_w8_bulkIndex, p0_w12 ->
p0_w1_writer.handle((p0_w8_bulkStart + p0_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w12 }, true, 0)
}
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_nested_ee0f317e439c05be(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null

try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkItem_bulkFailure !== outputbulkItem_bulkCleanupError) outputbulkItem_bulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
outputbulkFailure = outputbulkItem_bulkFailure
} } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size
var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null

outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
outputbulkItem_bulkList = kotlin.collections.ArrayList(outputbulkItem_bulkRange!!.length)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
        var outputbulkItem_bulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            outputbulkItem_bulkItem_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long))

outputbulkItem_bulkList!!.add(requireNotNull(outputbulkItem_bulkItem_0))
}

outputbulkList!!.add(requireNotNull(outputbulkItem_bulkList))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r46147a291c22570d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.size
var outputbulkFailure: Throwable? = failure
try { var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null

try { if (outputbulkItem_bulkRange == null) outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkItem_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_bulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = cleanup else if (outputbulkItem_bulkFailure !== cleanup) outputbulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_bulkFailure == null) outputbulkItem_bulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkItem_bulkFailure !== outputbulkItem_bulkCleanupError) outputbulkItem_bulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
outputbulkFailure = outputbulkItem_bulkFailure
} } catch (outputbulkItem_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_bulkCleanupError) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun bulkHandleTextCallback(callback: io.prebindgen.covertest.BulkHandleTextCallback, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandleText>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (true) callback_return_w3?.toList() else null
val callback_return_w1_writer_state = if (true) run { val callback_return_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_ff46ac585710e08a()
val callback_return_w3_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w3_elements }
if (callback_return_w3_bulkList == null) { callback_return_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0]) } else {
val callback_return_w3_bulkStart = callback_return_w1_writer.rows(callback_return_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a)
callback_return_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rff46ac585710e08a.offsets[0], callback_return_w3_bulkStart, callback_return_w3_bulkList.size)
callback_return_w3_bulkList.forEachIndexed { callback_return_w3_bulkIndex, callback_return_w7 ->
val __pb_callback_return_w7_field0 = callback_return_w7.handle
val __pb_callback_return_w7_field1 = callback_return_w7.text
callback_return_w1_writer.handle((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w7_field0 }, true, 0)
callback_return_w1_writer.text((callback_return_w3_bulkStart + callback_return_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.size) + io.prebindgen.covertest.BulkLayouts.r4927f4d69593185a.offsets[1], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w7_field1 })
}
}

callback_return_w1_writer } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1_writer_state?.encode(), callback_return_w1_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_writer_state?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? io.prebindgen.covertest.bulk.BulkHandleText)?.let { returnedValue2 -> (returnedValue2.handle as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue4 -> returnedHandles.add(returnedValue4); returnedOwners.add(returnedValue4) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_text_callback_bf41b6097b186476(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkOptionalBorrowPair(valuePresent: Boolean, valueHandle: io.prebindgen.covertest.bulk.BulkHandle?, valueNumber: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_i64_i64_fd1551214c82cfe3()
p0_w1_writer.handle((0) + io.prebindgen.covertest.BulkLayouts.rfd1551214c82cfe3.offsets[0], io.prebindgen.covertest.InputValidation.value { if (valuePresent) requireNotNull(if (true) valueHandle else null) { "invalid option payload" } else null }, false, 0)
if ((if (valuePresent) requireNotNull(if (true) valueHandle else null) { "invalid option payload" } else null) != null) {
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.rfd1551214c82cfe3.offsets[1], io.prebindgen.covertest.InputValidation.value { if (valuePresent) valueNumber else 0L })
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), false) { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_optional_borrow_pair_11a84638f9db14d4(nativeArgument0, nativeArgument1, nativeArgument2) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun bulkSelect(valueVariant: Int, valueVariant0Value: Long, valueVariant1: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rdafeaae0e426fdd5.size)
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.rdafeaae0e426fdd5.offsets[0], io.prebindgen.covertest.InputValidation.value { valueVariant })
if ((valueVariant) == 0) {
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.rdafeaae0e426fdd5.offsets[1], io.prebindgen.covertest.InputValidation.value { valueVariant0Value })
}
if ((valueVariant) == 1) {
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.rdafeaae0e426fdd5.offsets[2], io.prebindgen.covertest.InputValidation.value { valueVariant1 })
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_select_b3dae360ea0eecfb(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

(requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long
    } finally { returnCapture.release() }
}

public fun bulkKinds(`value`: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkKind>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkKind>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkKind> = run {
    val p0_w3_elements = try { if (true) `value`?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r49be3e7148c9ff8f.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r49be3e7148c9ff8f.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rdc22244d89c2587b)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r49be3e7148c9ff8f.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rdc22244d89c2587b.size) + io.prebindgen.covertest.BulkLayouts.rdc22244d89c2587b.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.wireValue })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_kinds_9b033197b22af4fe(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkKind>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r49be3e7148c9ff8f.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rdc22244d89c2587b.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rdc22244d89c2587b.size


outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkKind.fromWire((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rdc22244d89c2587b.offsets[0])) as Int))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun bulkUnits(`value`: kotlin.collections.List<kotlin.Unit>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.Unit>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.Unit> = run {
    val p0_w3_elements = try { if (true) `value`?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rf41b5b157c049db1.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rf41b5b157c049db1.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r904fefb3d01cb2ae)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rf41b5b157c049db1.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, bulkUnit ->
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_units_1560a205c63d4c8a(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.Unit>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf41b5b157c049db1.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r904fefb3d01cb2ae.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r904fefb3d01cb2ae.size


outputbulkList!!.add(kotlin.Unit)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun bulkFlags(`value`: kotlin.collections.List<Boolean>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Boolean>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Boolean> = run {
    val p0_w3_elements = try { if (true) `value`?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rf4f41f2e97b83c0c.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rf4f41f2e97b83c0c.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rc958cabd18f5a53a)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rf4f41f2e97b83c0c.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.byte((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rc958cabd18f5a53a.size) + io.prebindgen.covertest.BulkLayouts.rc958cabd18f5a53a.offsets[0], (if (io.prebindgen.covertest.InputValidation.value { p0_w8 }) 1 else 0).toByte())
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_flags_fe0ac76234b14846(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Boolean>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf4f41f2e97b83c0c.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rc958cabd18f5a53a.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rc958cabd18f5a53a.size


outputbulkList!!.add(((requireNotNull(outputbulkReader).byte((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rc958cabd18f5a53a.offsets[0]).toInt() != 0)) as Boolean)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun bulkEnvelope(`value`: io.prebindgen.covertest.bulk.BulkEnvelope?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkEnvelope?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkEnvelope? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long_Double4_ErrorEmpty_4244d173b136f580.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { (if (`value` != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (`value` != null) if (true) if (true) requireNotNull(`value`).point.x else 0L else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (`value` != null) if (true) if (true) requireNotNull(`value`).point.y else 0.0 else 0.0 else 0.0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { if (`value` != null) if (true) requireNotNull(`value`).tag else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_envelope_c78cdfdfd18c33f1(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) io.prebindgen.covertest.bulk.BulkEnvelope(io.prebindgen.covertest.bulk.BulkPoint(returnCapture.s1, returnCapture.s2), returnCapture.s3) else null
    } finally { returnCapture.release() }
}

public fun bulkPrimitives(`value`: io.prebindgen.covertest.bulk.BulkPrimitives, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkPrimitives> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkPrimitives = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.size)
p0_w1_writer.byte((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[0], (if (io.prebindgen.covertest.InputValidation.value { `value`.flag }) 1 else 0).toByte())
p0_w1_writer.byte((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[1], io.prebindgen.covertest.InputValidation.value { `value`.byte })
p0_w1_writer.short((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[2], io.prebindgen.covertest.InputValidation.value { `value`.short })
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[3], io.prebindgen.covertest.InputValidation.value { `value`.octet })
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[4], io.prebindgen.covertest.InputValidation.value { `value`.word })
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[5], io.prebindgen.covertest.InputValidation.value { `value`.number })
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[6], io.prebindgen.covertest.InputValidation.value { `value`.scalar })
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[7], io.prebindgen.covertest.InputValidation.value { `value`.address.toLong() })

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_primitives_be9bffc80840ca87(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

io.prebindgen.covertest.bulk.BulkPrimitives(((requireNotNull(outputbulkReader).byte((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[0]).toInt() != 0)) as Boolean, (requireNotNull(outputbulkReader).byte((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[1])) as Byte, (requireNotNull(outputbulkReader).short((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[2])) as Short, (requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[3])) as Int, (requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[4])) as Int, (requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[5])) as Long, (requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[6])) as Int, ((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r8a5c993ab6685b67.offsets[7])) as Long).toULong())
    } finally { returnCapture.release() }
}

public fun bulkOptionalWide(`value`: java.math.BigInteger?, onError: io.prebindgen.covertest.JniErrorHandler<java.math.BigInteger?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.math.BigInteger? = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.size)
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (`value` != null) 1 else 0) })
if (((if (`value` != null) 1 else 0)) == 1) {
p0_w1_writer.wide((0) + io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.offsets[1], io.prebindgen.covertest.InputValidation.value { if (`value` != null) requireNotNull(`value`).toByteArray() else null }, true)
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_optional_wide_0dbde2915189c95e(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.offsets[0])) == 1) {
            }

if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.offsets[0])) == 1) java.math.BigInteger((requireNotNull(outputbulkReader).wide((0) + io.prebindgen.covertest.BulkLayouts.r807a81a0a53fa8e0.offsets[1])) as ByteArray) else null
    } finally { returnCapture.release() }
}

public fun bulkOptionalCallback(`value`: io.prebindgen.covertest.BulkOptionalCallback, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = `value`.run()
return kotlin.run { val callback_return_w6 = callbackReturned
 try { val callback_return_w6_elements = if (callback_return_w6 != null) (if (true) requireNotNull(callback_return_w6).toList() else null) else (null)
val callback_return_w4_writer_state = if (callback_return_w6 != null) (if (true) run { val callback_return_w4_writer = io.prebindgen.covertest.BulkInputs.Input_Range_42eab86179b36709()
val callback_return_w6_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w6_elements }
if (callback_return_w6_bulkList == null) { callback_return_w4_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0]) } else {
val callback_return_w6_bulkStart = callback_return_w4_writer.rows(callback_return_w6_bulkList.size, io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09)
callback_return_w4_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0], callback_return_w6_bulkStart, callback_return_w6_bulkList.size)
callback_return_w6_bulkList.forEachIndexed { callback_return_w6_bulkIndex, callback_return_w10 ->
callback_return_w4_writer.handle((callback_return_w6_bulkStart + callback_return_w6_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], io.prebindgen.covertest.InputValidation.value { callback_return_w10 }, true, 0)
}
}

callback_return_w4_writer } else null) else (null)
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { if (callback_return_w6 != null) (callback_return_w4_writer_state?.handles.orEmpty()) else kotlin.collections.emptyList() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>((if (callback_return_w6 != null) 1 else 0), if (callback_return_w6 != null) callback_return_w4_writer_state?.encode() else null, if (callback_return_w6 != null) callback_return_w4_writer_state?.length ?: 0 else 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
if (callback_return_w6 != null) { callback_return_w4_writer_state?.consume(__pb_consume) }
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w6 as? Iterable<*>)?.forEach { returnedValue2 -> (returnedValue2 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue3 -> returnedHandles.add(returnedValue3); returnedOwners.add(returnedValue3) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_optional_callback_3ffeb9ae490fb489(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkNative(values: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkNative>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkNative>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkNative> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r87bd2f61e8a69677.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r87bd2f61e8a69677.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r87bd2f61e8a69677.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.size) + io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.prefix })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.size) + io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w8.number })
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.size) + io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.offsets[2], io.prebindgen.covertest.InputValidation.value { p0_w8.unsigned.toLong() })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_native_939b6cd52b96a019(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkNative>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r87bd2f61e8a69677.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.size


outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkNative((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.offsets[0])) as Int, (requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.offsets[1])) as Long, ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r220da08e0b780b69.offsets[2])) as Long).toULong()))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun <R> bulkExpanded(valueNumber: Long, valueText: String, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Long, String) -> R): R = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r37f6d28aa836bfb7.size)
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.r37f6d28aa836bfb7.offsets[0], io.prebindgen.covertest.InputValidation.value { valueNumber })
p0_w1_writer.text((0) + io.prebindgen.covertest.BulkLayouts.r37f6d28aa836bfb7.offsets[1], io.prebindgen.covertest.InputValidation.value { valueText })

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_expanded_65b04628441e244b(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

build((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r0b9b7f90ad764db3.offsets[0])) as Long, (requireNotNull(outputbulkReader).text((0) + io.prebindgen.covertest.BulkLayouts.r0b9b7f90ad764db3.offsets[1])) as String)
    } finally { returnCapture.release() }
}

public fun bulkExpandedRows(values: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkExpanded>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkExpanded>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkExpanded> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.ra5f90e11a63369ed.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.ra5f90e11a63369ed.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.ra5f90e11a63369ed.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.size) + io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.number })
p0_w1_writer.text((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.size) + io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w8.text })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_expanded_rows_8611c0e3cec0b75a(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkExpanded>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ra5f90e11a63369ed.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.size


outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkExpanded((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.offsets[0])) as Long, (requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r535f9e324da6eaa9.offsets[1])) as String))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun <R> bulkResult(valueNumber: Long, valueText: String, success: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.BulkFailureHandler<R>, build: (Long, String) -> R): R = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r37f6d28aa836bfb7.size)
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.r37f6d28aa836bfb7.offsets[0], io.prebindgen.covertest.InputValidation.value { valueNumber })
p0_w1_writer.text((0) + io.prebindgen.covertest.BulkLayouts.r37f6d28aa836bfb7.offsets[1], io.prebindgen.covertest.InputValidation.value { valueText })

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorByteArray1_056b654d5078f47e.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { success } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_result_22b6c65613a56b35(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run { var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(returnCapture.d0 as ByteArray)

onError.run((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r97145e0ef6a1193d.offsets[0])) as Long) }
    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

build((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r0b9b7f90ad764db3.offsets[0])) as Long, (requireNotNull(outputbulkReader).text((0) + io.prebindgen.covertest.BulkLayouts.r0b9b7f90ad764db3.offsets[1])) as String)
    } finally { returnCapture.release() }
}

public fun bulkHandleNew(`value`: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkHandle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkHandle = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_new_e5a9aa77fa861745(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.bulk.BulkHandle? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.bulk.BulkHandle? = null
        try {
            output0 = io.prebindgen.covertest.bulk.BulkHandle(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun bulkHandleValue(`value`: io.prebindgen.covertest.bulk.BulkHandle, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_value_4d14e8e3b89bd1bd(nativeArgument0, nativeArgument1) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun bulkDrops(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_drops_0c93c5a0b1492a12(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun bulkLate(panic: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkHandle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkHandle = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { panic } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_late_051f1ea6dcd2506f(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        var output0: io.prebindgen.covertest.bulk.BulkHandle? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r3711b7e845f5d8b9.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        var output0: io.prebindgen.covertest.bulk.BulkHandle? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            output0 = io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r3711b7e845f5d8b9.offsets[0])) as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r3711b7e845f5d8b9.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> bulkHandleViews(`value`: Long, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.bulk.BulkHandle, Long) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_views_c1f9da5de20e3fc3(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        var outputg0_0: io.prebindgen.covertest.bulk.BulkHandle? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.ree997f40a731b1db.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        var outputg0_0: io.prebindgen.covertest.bulk.BulkHandle? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            outputg0_0 = io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.ree997f40a731b1db.offsets[0])) as Long)
            build(requireNotNull(outputg0_0), (requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.ree997f40a731b1db.offsets[1])) as Long)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.ree997f40a731b1db.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun bulkHandles(values: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_42eab86179b36709()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8 }, true, 0)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handles_16b087b5f7f79484(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
        var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            outputbulkItem_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long))

outputbulkList!!.add(requireNotNull(outputbulkItem_0))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun bulkBorrow(`value`: io.prebindgen.covertest.bulk.BulkHandle, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_i64_d515d9783303b819()
p0_w1_writer.handle((0) + io.prebindgen.covertest.BulkLayouts.rd515d9783303b819.offsets[0], io.prebindgen.covertest.InputValidation.value { `value` }, false, 0)

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), false) { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_borrow_034d937ec5b796d9(nativeArgument0, nativeArgument1, nativeArgument2) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun bulkChecked(`value`: io.prebindgen.covertest.bulk.BulkHandle, number: Int, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkHandle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkHandle = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_i64_638fef00b45d8f09()
p0_w1_writer.handle((0) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], io.prebindgen.covertest.InputValidation.value { `value` }, true, 0)

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p1_w1_writer_state = try { if (true) run { val p1_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r062153e85348d725.size)
p1_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r062153e85348d725.offsets[0], io.prebindgen.covertest.InputValidation.value { number })

p1_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p1_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p1_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_checked_b7e284c31e7e5a38(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        var output0: io.prebindgen.covertest.bulk.BulkHandle? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
        var output0: io.prebindgen.covertest.bulk.BulkHandle? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            output0 = io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun bulkWide(values: kotlin.collections.List<java.math.BigInteger>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<java.math.BigInteger>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<java.math.BigInteger> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.re0c6fa25bb7482b5.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.re0c6fa25bb7482b5.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r659298f90cfa7005)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.re0c6fa25bb7482b5.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.wide((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r659298f90cfa7005.size) + io.prebindgen.covertest.BulkLayouts.r659298f90cfa7005.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.toByteArray() }, true)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_wide_6b955d7b4ee7dc0d(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<java.math.BigInteger>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.re0c6fa25bb7482b5.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r659298f90cfa7005.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r659298f90cfa7005.size


outputbulkList!!.add(java.math.BigInteger((requireNotNull(outputbulkReader).wide((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r659298f90cfa7005.offsets[0])) as ByteArray))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun bulkUnsignedWide(values: kotlin.collections.List<java.math.BigInteger>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<java.math.BigInteger>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<java.math.BigInteger> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r31d2b5a17a152d89.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r31d2b5a17a152d89.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r98c254983b527011)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r31d2b5a17a152d89.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.wide((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r98c254983b527011.size) + io.prebindgen.covertest.BulkLayouts.r98c254983b527011.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.toByteArray() }, false)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_unsigned_wide_407b0f069bb7ed7a(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<java.math.BigInteger>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r31d2b5a17a152d89.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r98c254983b527011.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r98c254983b527011.size


outputbulkList!!.add(java.math.BigInteger(1, (requireNotNull(outputbulkReader).wide((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r98c254983b527011.offsets[0])) as ByteArray))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun bulkCallbackWorker(`value`: io.prebindgen.covertest.BulkWorkerCallback, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<Long>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<Long>
        lateinit var callbackReturned: kotlin.collections.List<Long>
        try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


callbackOutput0_bulkList!!.add((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = `value`.run(callbackArgument0_0)
return kotlin.run { val callback_return_w4 = callbackReturned
 val callback_return_w4_elements = if (true) callback_return_w4?.toList() else null
val callback_return_w2_writer_state = if (true) run { val callback_return_w2_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size)
val callback_return_w4_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w4_elements }
if (callback_return_w4_bulkList == null) { callback_return_w2_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val callback_return_w4_bulkStart = callback_return_w2_writer.rows(callback_return_w4_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
callback_return_w2_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], callback_return_w4_bulkStart, callback_return_w4_bulkList.size)
callback_return_w4_bulkList.forEachIndexed { callback_return_w4_bulkIndex, callback_return_w8 ->
callback_return_w2_writer.long((callback_return_w4_bulkStart + callback_return_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { callback_return_w8 })
}
}

callback_return_w2_writer } else null
 run { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w2_writer_state?.array, callback_return_w2_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_callback_worker_4498defd15d5c1bf(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

public fun bulkCallback(`value`: io.prebindgen.covertest.BulkNumbersCallback, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<Long>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<Long>
        lateinit var callbackReturned: kotlin.collections.List<Long>
        try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


callbackOutput0_bulkList!!.add((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = `value`.run(callbackArgument0_0)
return kotlin.run { val callback_return_w4 = callbackReturned
 val callback_return_w4_elements = if (true) callback_return_w4?.toList() else null
val callback_return_w2_writer_state = if (true) run { val callback_return_w2_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size)
val callback_return_w4_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w4_elements }
if (callback_return_w4_bulkList == null) { callback_return_w2_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val callback_return_w4_bulkStart = callback_return_w2_writer.rows(callback_return_w4_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
callback_return_w2_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], callback_return_w4_bulkStart, callback_return_w4_bulkList.size)
callback_return_w4_bulkList.forEachIndexed { callback_return_w4_bulkIndex, callback_return_w8 ->
callback_return_w2_writer.long((callback_return_w4_bulkStart + callback_return_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { callback_return_w8 })
}
}

callback_return_w2_writer } else null
 run { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w2_writer_state?.array, callback_return_w2_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_callback_63ae4882756ac2cf(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

public fun bulkHandleCallback(`value`: io.prebindgen.covertest.BulkHandlesCallback, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_owners: io.prebindgen.covertest.CollectionOwners? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle>
        try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_owners = io.prebindgen.covertest.CollectionOwners()
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
        var callbackOutput0_bulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            callbackOutput0_bulkItem_0 = requireNotNull(callbackOutput0_owners).own(io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long))

callbackOutput0_bulkList!!.add(requireNotNull(callbackOutput0_bulkItem_0))
}

callbackOutput0_owners!!.complete = true
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
if (callbackOutput0_bulkReader == null) { callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
 }
callbackCleanupFailure = callbackOutput0_owners?.close(callbackCleanupFailure) ?: callbackCleanupFailure
if (callbackOutput0_owners?.complete != true) {
try { if (callbackOutput0_bulkRange == null) callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var callbackOutput0_bulkFailure: Throwable? = callbackCleanupFailure
try {         var callbackOutput0_bulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { if (callbackOutput0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = cleanup else if (callbackOutput0_bulkFailure !== cleanup) callbackOutput0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackOutput0_bulkFailure == null) callbackOutput0_bulkFailure = callbackOutput0_bulkCleanupError else if (callbackOutput0_bulkFailure !== callbackOutput0_bulkCleanupError) callbackOutput0_bulkFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }
callbackCleanupFailure = callbackOutput0_bulkFailure
} } catch (callbackOutput0_bulkCleanupError: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = callbackOutput0_bulkCleanupError else if (callbackCleanupFailure !== callbackOutput0_bulkCleanupError) callbackCleanupFailure!!.addSuppressed(callbackOutput0_bulkCleanupError) }

}
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = `value`.run(callbackArgument0_0)
return kotlin.run { val callback_return_w4 = callbackReturned
 try { val callback_return_w4_elements = if (true) callback_return_w4?.toList() else null
val callback_return_w2_writer_state = if (true) run { val callback_return_w2_writer = io.prebindgen.covertest.BulkInputs.Input_Range_42eab86179b36709()
val callback_return_w4_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w4_elements }
if (callback_return_w4_bulkList == null) { callback_return_w2_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0]) } else {
val callback_return_w4_bulkStart = callback_return_w2_writer.rows(callback_return_w4_bulkList.size, io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09)
callback_return_w2_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0], callback_return_w4_bulkStart, callback_return_w4_bulkList.size)
callback_return_w4_bulkList.forEachIndexed { callback_return_w4_bulkIndex, callback_return_w8 ->
callback_return_w2_writer.handle((callback_return_w4_bulkStart + callback_return_w4_bulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0], io.prebindgen.covertest.InputValidation.value { callback_return_w8 }, true, 0)
}
}

callback_return_w2_writer } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w2_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w2_writer_state?.encode(), callback_return_w2_writer_state?.length ?: 0)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w2_writer_state?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w4 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue2 -> returnedHandles.add(returnedValue2); returnedOwners.add(returnedValue2) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_handle_callback_39d3540632be44a7(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
        var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            outputbulkItem_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long))

outputbulkList!!.add(requireNotNull(outputbulkItem_0))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r42eab86179b36709.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r638fef00b45d8f09.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun bulkPoint(`value`: io.prebindgen.covertest.bulk.BulkPoint, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkPoint> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkPoint = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size)
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0], io.prebindgen.covertest.InputValidation.value { `value`.x })
p0_w1_writer.double((0) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1], io.prebindgen.covertest.InputValidation.value { `value`.y })

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_point_e770ca9da9c12852(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

io.prebindgen.covertest.bulk.BulkPoint((requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0])) as Long, (requireNotNull(outputbulkReader).double((0) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1])) as Double)
    } finally { returnCapture.release() }
}

public fun bulkPoints(values: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.x })
p0_w1_writer.double((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w8.y })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_points_6101af74290565c5(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkPoint>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size


outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkPoint((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0])) as Long, (requireNotNull(outputbulkReader).double((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1])) as Double))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun bulkPointArray(`value`: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint> = run {
    val p0_w3_elements = try { if (true) `value`?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.x })
p0_w1_writer.double((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w8.y })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_point_array_aa3dcf2590ca3a28(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkPoint>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size


outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkPoint((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0])) as Long, (requireNotNull(outputbulkReader).double((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1])) as Double))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun bulkSlice(`value`: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkPoint> = run {
    val p0_w3_elements = try { if (true) `value`?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8.x })
p0_w1_writer.double((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1], io.prebindgen.covertest.InputValidation.value { p0_w8.y })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_slice_c30fbd330fdb06e6(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkPoint>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ra8ec97310260b4fd.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.size


outputbulkList!!.add(io.prebindgen.covertest.bulk.BulkPoint((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[0])) as Long, (requireNotNull(outputbulkReader).double((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r554fa1d6f8692db9.offsets[1])) as Double))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun bulkPointPlain(`value`: io.prebindgen.covertest.bulk.BulkPoint, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.bulk.BulkPoint> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.bulk.BulkPoint = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Double2_ErrorEmpty_a4ec6c8f44b5ff7d.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value`.x } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { `value`.y } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_point_plain_53e7bd65e4464218(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    io.prebindgen.covertest.bulk.BulkPoint(returnCapture.s0, returnCapture.s1)
    } finally { returnCapture.release() }
}

public fun bulkOptionalHandles(`value`: kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.bulk.BulkHandle?>? = run {
    val p0_w5_elements = try { if (`value` != null) (if (true) requireNotNull(`value`).toList() else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_i32_Range_f27ab2e6065597a6()
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (`value` != null) 1 else 0) })
if (((if (`value` != null) 1 else 0)) == 1) {
val p0_w5_bulkList = io.prebindgen.covertest.InputValidation.value { if (`value` != null) p0_w5_elements else null }
if (p0_w5_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1]) } else {
val p0_w5_bulkStart = p0_w1_writer.rows(p0_w5_bulkList.size, io.prebindgen.covertest.BulkLayouts.rcbe830243575e259)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1], p0_w5_bulkStart, p0_w5_bulkList.size)
p0_w5_bulkList.forEachIndexed { p0_w5_bulkIndex, p0_w12 ->
p0_w1_writer.handle((p0_w5_bulkStart + p0_w5_bulkIndex * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0], io.prebindgen.covertest.InputValidation.value { if (p0_w12 != null) requireNotNull(if (true) requireNotNull(p0_w12) else null) { "invalid option payload" } else null }, true, 0)
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_optional_handles_44d1ca20cf3dbdbe(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle?>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0])) == 1) {
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.bulk.BulkHandle?>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0])) == 1) {
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size
        var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) != 0L) {
            outputbulkItem_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.bulk.BulkHandle((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) as Long))
            }

outputbulkList!!.add(if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) != 0L) requireNotNull(outputbulkItem_0) else null)
}

outputowners!!.complete = true
            }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0])) == 1) requireNotNull(outputbulkList) else null
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[0])) == 1) {
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf27ab2e6065597a6.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_0: io.prebindgen.covertest.bulk.BulkHandle? = null

            if ((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_BulkHandle_393b82dfa5eaa2d5((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.rcbe830243575e259.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun bulkArray(`value`: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w3_elements = try { if (true) `value`?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8 })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_array_b64f34d955cfbc66(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

public fun bulkArrayBorrow(`value`: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (true) `value`?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8 })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_array_borrow_08a33cfde93f0897(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun bulkScalar(`value`: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { `value` })

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_scalar_859e15b07fe6305f(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

(requireNotNull(outputbulkReader).long((0) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long
    } finally { returnCapture.release() }
}

public fun bulkNumbers(values: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8 })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_numbers_a3f5c3da005fa8ec(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

public fun bulkNested(values: kotlin.collections.List<kotlin.collections.List<Long>>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<Long>>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<Long>> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r01c263066f8f1a73.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r01c263066f8f1a73.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r01c263066f8f1a73.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
val p0_w8_elements = if (true) p0_w8?.toList() else null
val p0_w8_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w8_elements }
if (p0_w8_bulkList == null) { p0_w1_writer.absent((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val p0_w8_bulkStart = p0_w1_writer.rows(p0_w8_bulkList.size, io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0], p0_w8_bulkStart, p0_w8_bulkList.size)
p0_w8_bulkList.forEachIndexed { p0_w8_bulkIndex, p0_w13 ->
p0_w1_writer.long((p0_w8_bulkStart + p0_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w13 })
}
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_nested_0e44e904f1fd8209(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<Long>>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r01c263066f8f1a73.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size
var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<Long>? = null

outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkItem_bulkList = kotlin.collections.ArrayList(outputbulkItem_bulkRange!!.length)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkItem_bulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

outputbulkList!!.add(requireNotNull(outputbulkItem_bulkList))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun bulkText(`value`: String, onError: io.prebindgen.covertest.JniErrorHandler<String> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.size)
p0_w1_writer.text((0) + io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.offsets[0], io.prebindgen.covertest.InputValidation.value { `value` })

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_text_af334975f2297859(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)

(requireNotNull(outputbulkReader).text((0) + io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.offsets[0])) as String
    } finally { returnCapture.release() }
}

public fun bulkStrings(values: kotlin.collections.List<String>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<String>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<String> = run {
    val p0_w3_elements = try { if (true) values?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r79427dd643a1b061.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r79427dd643a1b061.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r79427dd643a1b061.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.text((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.size) + io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8 })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_strings_106737903dcecf60(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<String>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r79427dd643a1b061.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.size


outputbulkList!!.add((requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.offsets[0])) as String)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun bulkOptional(`value`: kotlin.collections.List<String?>?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<String?>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<String?>? = run {
    val p0_w5_elements = try { if (`value` != null) (if (true) requireNotNull(`value`).toList() else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.size)
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (`value` != null) 1 else 0) })
if (((if (`value` != null) 1 else 0)) == 1) {
val p0_w5_bulkList = io.prebindgen.covertest.InputValidation.value { if (`value` != null) p0_w5_elements else null }
if (p0_w5_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[1]) } else {
val p0_w5_bulkStart = p0_w1_writer.rows(p0_w5_bulkList.size, io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[1], p0_w5_bulkStart, p0_w5_bulkList.size)
p0_w5_bulkList.forEachIndexed { p0_w5_bulkIndex, p0_w12 ->
p0_w1_writer.int((p0_w5_bulkStart + p0_w5_bulkIndex * io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.size) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (p0_w12 != null) 1 else 0) })
if (((if (p0_w12 != null) 1 else 0)) == 1) {
p0_w1_writer.text((p0_w5_bulkStart + p0_w5_bulkIndex * io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.size) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[1], io.prebindgen.covertest.InputValidation.value { if (p0_w12 != null) requireNotNull(p0_w12) else null })
}
}
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_optional_4abfbe9a71e24339(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<String?>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[0])) == 1) {
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.size

            if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[0])) == 1) {
            }

outputbulkList!!.add(if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[0])) == 1) (requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[1])) as String else null)
}
            }

if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rc8d6433a01ad4efb.offsets[0])) == 1) requireNotNull(outputbulkList) else null
    } finally { returnCapture.release() }
}

public fun bulkBytes(`value`: kotlin.collections.List<ByteArray>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<ByteArray>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<ByteArray> = run {
    val p0_w3_elements = try { if (true) `value`?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.rba88ec47a5d9bc85.size)
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rba88ec47a5d9bc85.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.re048ef6766326a01)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rba88ec47a5d9bc85.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w8 ->
p0_w1_writer.bytes((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.size) + io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.offsets[0], io.prebindgen.covertest.InputValidation.value { p0_w8 })
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_writer_state?.array } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_bulk_bytes_711128b36c976142(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<ByteArray>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rba88ec47a5d9bc85.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.size


outputbulkList!!.add((requireNotNull(outputbulkReader).bytes((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.offsets[0])) as ByteArray)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}
