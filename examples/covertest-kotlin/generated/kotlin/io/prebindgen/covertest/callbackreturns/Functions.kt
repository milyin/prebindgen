@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.callbackreturns

public fun callbackMixedScalars(callback: io.prebindgen.covertest.MixedScalarBatchesCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackScalarBatch>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.let { callback_return_w1_snapshot -> kotlin.Array<kotlin.Any?>(callback_return_w1_snapshot.size) { callback_return_w1_index -> val callback_return_w5 = callback_return_w1_snapshot[callback_return_w1_index]
val callback_return_w7_elements = if (true) callback_return_w5.rows?.toList() else null
val callback_return_w6_writer_state = if (true) run { val callback_return_w6_writer = io.prebindgen.covertest.BulkWriter(io.prebindgen.covertest.BulkLayouts.r5803381f2d3ce1a6.size)
val callback_return_w7_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w7_elements }
if (callback_return_w7_bulkList == null) { callback_return_w6_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r5803381f2d3ce1a6.offsets[0]) } else {
val callback_return_w7_bulkStart = callback_return_w6_writer.rows(callback_return_w7_bulkList.size, io.prebindgen.covertest.BulkLayouts.r0d4a0111cad6e1de)
callback_return_w6_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r5803381f2d3ce1a6.offsets[0], callback_return_w7_bulkStart, callback_return_w7_bulkList.size)
callback_return_w7_bulkList.forEachIndexed { callback_return_w7_bulkIndex, callback_return_w11 ->
callback_return_w6_writer.long((callback_return_w7_bulkStart + callback_return_w7_bulkIndex * io.prebindgen.covertest.BulkLayouts.r0d4a0111cad6e1de.size) + io.prebindgen.covertest.BulkLayouts.r0d4a0111cad6e1de.offsets[0], io.prebindgen.covertest.InputValidation.value { callback_return_w11.number })
callback_return_w6_writer.text((callback_return_w7_bulkStart + callback_return_w7_bulkIndex * io.prebindgen.covertest.BulkLayouts.r0d4a0111cad6e1de.size) + io.prebindgen.covertest.BulkLayouts.r0d4a0111cad6e1de.offsets[1], io.prebindgen.covertest.InputValidation.value { callback_return_w11.label })
}
}

callback_return_w6_writer } else null
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { callback_return_w5.stamp }, io.prebindgen.covertest.InputValidation.value { callback_return_w6_writer_state?.array }, io.prebindgen.covertest.InputValidation.value { callback_return_w6_writer_state?.length ?: 0 }) } } else null
 run { val callbackReturnWire = callback_return_w1_elements
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_mixed_scalars_b3ffdfdc953994c9(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnResourceAuditStart(onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_resource_audit_start_a5392cdfc5cff176(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun callbackReturnResourceAuditFinish(onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_resource_audit_finish_a13e6bda7dfc3d9b(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun callbackMixedBatch(callback: io.prebindgen.covertest.MixedBatchCallback, retry: Boolean, worker: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackNestedBatch
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val __pb_callback_return_w1_field0 = callback_return_w1.first
val __pb_callback_return_w1_field1 = callback_return_w1.observed
val __pb_callback_return_w1_field2 = callback_return_w1.groups
val __pb_callback_return_w1_field3 = callback_return_w1.marker
val __pb_callback_return_w1_field4 = callback_return_w1.tail
val __pb_callback_return_w1_field5 = callback_return_w1.last
val callback_return_w5_elements = if (true) __pb_callback_return_w1_field2?.toList() else null
val callback_return_w4_writer_state = if (true) run { val callback_return_w4_writer = io.prebindgen.covertest.BulkInputs.Input_Range_31afb2a04211128a()
val callback_return_w5_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w5_elements }
if (callback_return_w5_bulkList == null) { callback_return_w4_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r31afb2a04211128a.offsets[0]) } else {
val callback_return_w5_bulkStart = callback_return_w4_writer.rows(callback_return_w5_bulkList.size, io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2)
callback_return_w4_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r31afb2a04211128a.offsets[0], callback_return_w5_bulkStart, callback_return_w5_bulkList.size)
callback_return_w5_bulkList.forEachIndexed { callback_return_w5_bulkIndex, callback_return_w11 ->
val callback_return_w11_elements = if (callback_return_w11 != null) (if (true) requireNotNull(callback_return_w11).toList() else null) else (null)
callback_return_w4_writer.int((callback_return_w5_bulkStart + callback_return_w5_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.size) + io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (callback_return_w11 != null) 1 else 0) })
val callback_return_w11_bulkList = io.prebindgen.covertest.InputValidation.value { if (callback_return_w11 != null) callback_return_w11_elements else null }
if (callback_return_w11_bulkList == null) { callback_return_w4_writer.absent((callback_return_w5_bulkStart + callback_return_w5_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.size) + io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.offsets[1]) } else {
val callback_return_w11_bulkStart = callback_return_w4_writer.rows(callback_return_w11_bulkList.size, io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3)
callback_return_w4_writer.range((callback_return_w5_bulkStart + callback_return_w5_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.size) + io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.offsets[1], callback_return_w11_bulkStart, callback_return_w11_bulkList.size)
callback_return_w11_bulkList.forEachIndexed { callback_return_w11_bulkIndex, callback_return_w15 ->
val __pb_callback_return_w15_field0 = callback_return_w15.first
val __pb_callback_return_w15_field1 = callback_return_w15.optional
val __pb_callback_return_w15_field2 = callback_return_w15.character
val __pb_callback_return_w15_field3 = callback_return_w15.last
callback_return_w4_writer.handle((callback_return_w11_bulkStart + callback_return_w11_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w15_field0 }, true, 0)
callback_return_w4_writer.handle((callback_return_w11_bulkStart + callback_return_w11_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[1], io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w15_field1 != null) requireNotNull(if (true) requireNotNull(__pb_callback_return_w15_field1) else null) { "invalid option payload" } else null }, true, 0)
callback_return_w4_writer.int((callback_return_w11_bulkStart + callback_return_w11_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[2], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w15_field2 })
callback_return_w4_writer.handle((callback_return_w11_bulkStart + callback_return_w11_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[3], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w15_field3 }, true, 0)
}
}
}
}

callback_return_w4_writer } else null
val callback_return_w20_elements = if (true) __pb_callback_return_w1_field4?.toList()?.let { callback_return_w20_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w20_snapshot.size) { callback_return_w20_index -> val callback_return_w24 = callback_return_w20_snapshot[callback_return_w20_index]
val __pb_callback_return_w24_field0 = callback_return_w24.first
val __pb_callback_return_w24_field1 = callback_return_w24.optional
val __pb_callback_return_w24_field2 = callback_return_w24.character
val __pb_callback_return_w24_field3 = callback_return_w24.last
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w24_field0 }, io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w24_field1 != null) requireNotNull(if (true) requireNotNull(__pb_callback_return_w24_field1) else null) { "invalid option payload" } else null }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w24_field2 }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w24_field3 }) }, intArrayOf(2, 2, 0, 2), 1, null) } else null
 run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { __pb_callback_return_w1_field0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.covertest.NativeHandle? = try { __pb_callback_return_w1_field1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle2: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w4_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle3: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w20_elements?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle4: io.prebindgen.covertest.NativeHandle? = try { __pb_callback_return_w1_field5 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withMixedInputLocks(kotlin.collections.listOfNotNull(__pb_handle1), ((kotlin.collections.listOfNotNull(__pb_handle0, __pb_handle4) + (__pb_handle2)) + (__pb_handle3))) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(__pb_handle0?.raw ?: 0L, __pb_handle1?.raw ?: 0L, callback_return_w4_writer_state?.encode(), callback_return_w4_writer_state?.length ?: 0, __pb_callback_return_w1_field3, callback_return_w20_elements?.encode(), __pb_handle4?.raw ?: 0L)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
__pb_handle0?.let { __pb_consume(it) }; callback_return_w4_writer_state?.consume(__pb_consume); callback_return_w20_elements?.consume(__pb_consume); __pb_handle4?.let { __pb_consume(it) }
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? io.prebindgen.covertest.callbackreturns.CallbackNestedBatch)?.let { returnedValue0 -> (returnedValue0.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue2 -> returnedHandles.add(returnedValue2); returnedOwners.add(returnedValue2) }
(returnedValue0.observed as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue6 -> returnedHandles.add(returnedValue6);  }
(returnedValue0.groups as? Iterable<*>)?.forEach { returnedValue9 -> (returnedValue9 as? Iterable<*>)?.forEach { returnedValue11 -> (returnedValue11 as? io.prebindgen.covertest.callbackreturns.CallbackFixedResult)?.let { returnedValue12 -> (returnedValue12.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue14 -> returnedHandles.add(returnedValue14); returnedOwners.add(returnedValue14) }
(returnedValue12.optional as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue17 -> returnedHandles.add(returnedValue17); returnedOwners.add(returnedValue17) }
(returnedValue12.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue19 -> returnedHandles.add(returnedValue19); returnedOwners.add(returnedValue19) }
 }
 }
 }
(returnedValue0.tail as? Iterable<*>)?.forEach { returnedValue21 -> (returnedValue21 as? io.prebindgen.covertest.callbackreturns.CallbackFixedResult)?.let { returnedValue22 -> (returnedValue22.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue24 -> returnedHandles.add(returnedValue24); returnedOwners.add(returnedValue24) }
(returnedValue22.optional as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue27 -> returnedHandles.add(returnedValue27); returnedOwners.add(returnedValue27) }
(returnedValue22.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue29 -> returnedHandles.add(returnedValue29); returnedOwners.add(returnedValue29) }
 }
 }
(returnedValue0.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue31 -> returnedHandles.add(returnedValue31); returnedOwners.add(returnedValue31) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { retry } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { worker } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_mixed_batch_0e29c21fa132f86f(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackMixedBatchesEcho(batches: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackNestedBatch>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackNestedBatch>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackNestedBatch> = run {
    val p0_w3_elements = try { if (true) batches?.toList() else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_Range_d0f85d5de64becff()
val p0_w3_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w3_elements }
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.rd0f85d5de64becff.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89)
p0_w1_writer.range((0) + io.prebindgen.covertest.BulkLayouts.rd0f85d5de64becff.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
val __pb_p0_w7_field0 = p0_w7.first
val __pb_p0_w7_field1 = p0_w7.observed
val __pb_p0_w7_field2 = p0_w7.groups
val __pb_p0_w7_field3 = p0_w7.marker
val __pb_p0_w7_field4 = p0_w7.tail
val __pb_p0_w7_field5 = p0_w7.last
val p0_w8_elements = if (true) __pb_p0_w7_field2?.toList() else null
val p0_w23_elements = if (true) __pb_p0_w7_field4?.toList() else null
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.size) + io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_p0_w7_field0 }, true, 0)
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.size) + io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.offsets[1], io.prebindgen.covertest.InputValidation.value { __pb_p0_w7_field1 }, true, 0)
val p0_w8_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w8_elements }
if (p0_w8_bulkList == null) { p0_w1_writer.absent((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.size) + io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.offsets[2]) } else {
val p0_w8_bulkStart = p0_w1_writer.rows(p0_w8_bulkList.size, io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2)
p0_w1_writer.range((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.size) + io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.offsets[2], p0_w8_bulkStart, p0_w8_bulkList.size)
p0_w8_bulkList.forEachIndexed { p0_w8_bulkIndex, p0_w14 ->
val p0_w14_elements = if (p0_w14 != null) (if (true) requireNotNull(p0_w14).toList() else null) else (null)
p0_w1_writer.int((p0_w8_bulkStart + p0_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.size) + io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (p0_w14 != null) 1 else 0) })
val p0_w14_bulkList = io.prebindgen.covertest.InputValidation.value { if (p0_w14 != null) p0_w14_elements else null }
if (p0_w14_bulkList == null) { p0_w1_writer.absent((p0_w8_bulkStart + p0_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.size) + io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.offsets[1]) } else {
val p0_w14_bulkStart = p0_w1_writer.rows(p0_w14_bulkList.size, io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3)
p0_w1_writer.range((p0_w8_bulkStart + p0_w8_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.size) + io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.offsets[1], p0_w14_bulkStart, p0_w14_bulkList.size)
p0_w14_bulkList.forEachIndexed { p0_w14_bulkIndex, p0_w18 ->
val __pb_p0_w18_field0 = p0_w18.first
val __pb_p0_w18_field1 = p0_w18.optional
val __pb_p0_w18_field2 = p0_w18.character
val __pb_p0_w18_field3 = p0_w18.last
p0_w1_writer.handle((p0_w14_bulkStart + p0_w14_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_p0_w18_field0 }, true, 0)
p0_w1_writer.handle((p0_w14_bulkStart + p0_w14_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[1], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w18_field1 != null) requireNotNull(if (true) requireNotNull(__pb_p0_w18_field1) else null) { "invalid option payload" } else null }, true, 0)
p0_w1_writer.int((p0_w14_bulkStart + p0_w14_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[2], io.prebindgen.covertest.InputValidation.value { __pb_p0_w18_field2 })
p0_w1_writer.handle((p0_w14_bulkStart + p0_w14_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[3], io.prebindgen.covertest.InputValidation.value { __pb_p0_w18_field3 }, true, 0)
}
}
}
}
p0_w1_writer.int((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.size) + io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.offsets[3], io.prebindgen.covertest.InputValidation.value { __pb_p0_w7_field3 })
val p0_w23_bulkList = io.prebindgen.covertest.InputValidation.value { p0_w23_elements }
if (p0_w23_bulkList == null) { p0_w1_writer.absent((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.size) + io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.offsets[4]) } else {
val p0_w23_bulkStart = p0_w1_writer.rows(p0_w23_bulkList.size, io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3)
p0_w1_writer.range((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.size) + io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.offsets[4], p0_w23_bulkStart, p0_w23_bulkList.size)
p0_w23_bulkList.forEachIndexed { p0_w23_bulkIndex, p0_w27 ->
val __pb_p0_w27_field0 = p0_w27.first
val __pb_p0_w27_field1 = p0_w27.optional
val __pb_p0_w27_field2 = p0_w27.character
val __pb_p0_w27_field3 = p0_w27.last
p0_w1_writer.handle((p0_w23_bulkStart + p0_w23_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_p0_w27_field0 }, true, 0)
p0_w1_writer.handle((p0_w23_bulkStart + p0_w23_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[1], io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w27_field1 != null) requireNotNull(if (true) requireNotNull(__pb_p0_w27_field1) else null) { "invalid option payload" } else null }, true, 0)
p0_w1_writer.int((p0_w23_bulkStart + p0_w23_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[2], io.prebindgen.covertest.InputValidation.value { __pb_p0_w27_field2 })
p0_w1_writer.handle((p0_w23_bulkStart + p0_w23_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[3], io.prebindgen.covertest.InputValidation.value { __pb_p0_w27_field3 }, true, 0)
}
}
p0_w1_writer.handle((p0_w3_bulkStart + p0_w3_bulkIndex * io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.size) + io.prebindgen.covertest.BulkLayouts.rd54b28f5270f2e89.offsets[5], io.prebindgen.covertest.InputValidation.value { __pb_p0_w7_field5 }, true, 0)
}
}

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_mixed_batches_echo_b1e56234e999f1fa(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; p0_w1_writer_state?.consume(__pb_consume) } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; p0_w1_writer_state?.consume(__pb_consume) } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackNestedBatch>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r4e69bfcf823e0723.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputbulkItem_g2_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_g2_g0_bulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>?>? = null
var outputbulkItem_g4_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_g4_g0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>? = null
        var outputbulkItem_g5_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
try { if (outputbulkItem_g2_g0_bulkRange == null) outputbulkItem_g2_g0_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[2])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.size)
for (outputbulkItem_g2_g0_bulkIndex in 0 until outputbulkItem_g2_g0_bulkRange!!.length) {
val outputbulkItem_g2_g0_bulkBase = outputbulkItem_g2_g0_bulkRange!!.offset + outputbulkItem_g2_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.size
var outputbulkItem_g2_g0_bulkFailure: Throwable? = outputbulkFailure
try { var outputbulkItem_g2_g0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_g2_g0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>? = null

            if ((requireNotNull(outputbulkReader).int((outputbulkItem_g2_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.offsets[0])) == 1) {
try { if (outputbulkItem_g2_g0_bulkItem_bulkRange == null) outputbulkItem_g2_g0_bulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkItem_g2_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r49d713334761702d.size)
for (outputbulkItem_g2_g0_bulkItem_bulkIndex in 0 until outputbulkItem_g2_g0_bulkItem_bulkRange!!.length) {
val outputbulkItem_g2_g0_bulkItem_bulkBase = outputbulkItem_g2_g0_bulkItem_bulkRange!!.offset + outputbulkItem_g2_g0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r49d713334761702d.size
var outputbulkItem_g2_g0_bulkItem_bulkFailure: Throwable? = outputbulkItem_g2_g0_bulkFailure
try {         var outputbulkItem_g2_g0_bulkItem_bulkItem_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g2_g0_bulkItem_bulkItem_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g2_g0_bulkItem_bulkItem_g3_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_g2_g0_bulkItem_bulkFailure == null) outputbulkItem_g2_g0_bulkItem_bulkFailure = cleanup else if (outputbulkItem_g2_g0_bulkItem_bulkFailure !== cleanup) outputbulkItem_g2_g0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            if ((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_g2_g0_bulkItem_bulkFailure == null) outputbulkItem_g2_g0_bulkItem_bulkFailure = cleanup else if (outputbulkItem_g2_g0_bulkItem_bulkFailure !== cleanup) outputbulkItem_g2_g0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[3])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_g2_g0_bulkItem_bulkFailure == null) outputbulkItem_g2_g0_bulkItem_bulkFailure = cleanup else if (outputbulkItem_g2_g0_bulkItem_bulkFailure !== cleanup) outputbulkItem_g2_g0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_g2_g0_bulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_g2_g0_bulkItem_bulkFailure == null) outputbulkItem_g2_g0_bulkItem_bulkFailure = outputbulkItem_g2_g0_bulkItem_bulkCleanupError else if (outputbulkItem_g2_g0_bulkItem_bulkFailure !== outputbulkItem_g2_g0_bulkItem_bulkCleanupError) outputbulkItem_g2_g0_bulkItem_bulkFailure!!.addSuppressed(outputbulkItem_g2_g0_bulkItem_bulkCleanupError) }
outputbulkItem_g2_g0_bulkFailure = outputbulkItem_g2_g0_bulkItem_bulkFailure
} } catch (outputbulkItem_g2_g0_bulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_g2_g0_bulkFailure == null) outputbulkItem_g2_g0_bulkFailure = outputbulkItem_g2_g0_bulkItem_bulkCleanupError else if (outputbulkItem_g2_g0_bulkFailure !== outputbulkItem_g2_g0_bulkItem_bulkCleanupError) outputbulkItem_g2_g0_bulkFailure!!.addSuppressed(outputbulkItem_g2_g0_bulkItem_bulkCleanupError) }
            }
 } catch (outputbulkItem_g2_g0_bulkCleanupError: Throwable) { if (outputbulkItem_g2_g0_bulkFailure == null) outputbulkItem_g2_g0_bulkFailure = outputbulkItem_g2_g0_bulkCleanupError else if (outputbulkItem_g2_g0_bulkFailure !== outputbulkItem_g2_g0_bulkCleanupError) outputbulkItem_g2_g0_bulkFailure!!.addSuppressed(outputbulkItem_g2_g0_bulkCleanupError) }
outputbulkFailure = outputbulkItem_g2_g0_bulkFailure
} } catch (outputbulkItem_g2_g0_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_g2_g0_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_g2_g0_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_g2_g0_bulkCleanupError) }
try { if (outputbulkItem_g4_g0_bulkRange == null) outputbulkItem_g4_g0_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[4])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r49d713334761702d.size)
for (outputbulkItem_g4_g0_bulkIndex in 0 until outputbulkItem_g4_g0_bulkRange!!.length) {
val outputbulkItem_g4_g0_bulkBase = outputbulkItem_g4_g0_bulkRange!!.offset + outputbulkItem_g4_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r49d713334761702d.size
var outputbulkItem_g4_g0_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_g4_g0_bulkItem_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g4_g0_bulkItem_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g4_g0_bulkItem_g3_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_g4_g0_bulkFailure == null) outputbulkItem_g4_g0_bulkFailure = cleanup else if (outputbulkItem_g4_g0_bulkFailure !== cleanup) outputbulkItem_g4_g0_bulkFailure!!.addSuppressed(cleanup) }
            if ((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_g4_g0_bulkFailure == null) outputbulkItem_g4_g0_bulkFailure = cleanup else if (outputbulkItem_g4_g0_bulkFailure !== cleanup) outputbulkItem_g4_g0_bulkFailure!!.addSuppressed(cleanup) }
            }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[3])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_g4_g0_bulkFailure == null) outputbulkItem_g4_g0_bulkFailure = cleanup else if (outputbulkItem_g4_g0_bulkFailure !== cleanup) outputbulkItem_g4_g0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_g4_g0_bulkCleanupError: Throwable) { if (outputbulkItem_g4_g0_bulkFailure == null) outputbulkItem_g4_g0_bulkFailure = outputbulkItem_g4_g0_bulkCleanupError else if (outputbulkItem_g4_g0_bulkFailure !== outputbulkItem_g4_g0_bulkCleanupError) outputbulkItem_g4_g0_bulkFailure!!.addSuppressed(outputbulkItem_g4_g0_bulkCleanupError) }
outputbulkFailure = outputbulkItem_g4_g0_bulkFailure
} } catch (outputbulkItem_g4_g0_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_g4_g0_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_g4_g0_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_g4_g0_bulkCleanupError) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[5])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputowners: io.prebindgen.covertest.CollectionOwners? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackNestedBatch>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputowners = io.prebindgen.covertest.CollectionOwners()
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r4e69bfcf823e0723.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.size
        var outputbulkItem_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputbulkItem_g2_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_g2_g0_bulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>?>? = null
var outputbulkItem_g4_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_g4_g0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>? = null
        var outputbulkItem_g5_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            outputbulkItem_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[0])) as Long))
            outputbulkItem_g1_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[1])) as Long))
outputbulkItem_g2_g0_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[2])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.size)
outputbulkItem_g2_g0_bulkList = kotlin.collections.ArrayList(outputbulkItem_g2_g0_bulkRange!!.length)
for (outputbulkItem_g2_g0_bulkIndex in 0 until outputbulkItem_g2_g0_bulkRange!!.length) {
val outputbulkItem_g2_g0_bulkBase = outputbulkItem_g2_g0_bulkRange!!.offset + outputbulkItem_g2_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.size
var outputbulkItem_g2_g0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_g2_g0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>? = null

            if ((requireNotNull(outputbulkReader).int((outputbulkItem_g2_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.offsets[0])) == 1) {
outputbulkItem_g2_g0_bulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkItem_g2_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r49d713334761702d.size)
outputbulkItem_g2_g0_bulkItem_bulkList = kotlin.collections.ArrayList(outputbulkItem_g2_g0_bulkItem_bulkRange!!.length)
for (outputbulkItem_g2_g0_bulkItem_bulkIndex in 0 until outputbulkItem_g2_g0_bulkItem_bulkRange!!.length) {
val outputbulkItem_g2_g0_bulkItem_bulkBase = outputbulkItem_g2_g0_bulkItem_bulkRange!!.offset + outputbulkItem_g2_g0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r49d713334761702d.size
        var outputbulkItem_g2_g0_bulkItem_bulkItem_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g2_g0_bulkItem_bulkItem_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g2_g0_bulkItem_bulkItem_g3_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            outputbulkItem_g2_g0_bulkItem_bulkItem_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[0])) as Long))
            if ((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) != 0L) {
            outputbulkItem_g2_g0_bulkItem_bulkItem_g1_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) as Long))
            }
            outputbulkItem_g2_g0_bulkItem_bulkItem_g3_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[3])) as Long))

outputbulkItem_g2_g0_bulkItem_bulkList!!.add(io.prebindgen.covertest.callbackreturns.CallbackFixedResult(requireNotNull(outputbulkItem_g2_g0_bulkItem_bulkItem_g0_0), if ((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) != 0L) requireNotNull(outputbulkItem_g2_g0_bulkItem_bulkItem_g1_g0_0) else null, (requireNotNull(outputbulkReader).int((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[2])) as Int, requireNotNull(outputbulkItem_g2_g0_bulkItem_bulkItem_g3_0)))
}
            }

outputbulkItem_g2_g0_bulkList!!.add(if ((requireNotNull(outputbulkReader).int((outputbulkItem_g2_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.offsets[0])) == 1) requireNotNull(outputbulkItem_g2_g0_bulkItem_bulkList) else null)
}
outputbulkItem_g4_g0_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[4])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r49d713334761702d.size)
outputbulkItem_g4_g0_bulkList = kotlin.collections.ArrayList(outputbulkItem_g4_g0_bulkRange!!.length)
for (outputbulkItem_g4_g0_bulkIndex in 0 until outputbulkItem_g4_g0_bulkRange!!.length) {
val outputbulkItem_g4_g0_bulkBase = outputbulkItem_g4_g0_bulkRange!!.offset + outputbulkItem_g4_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r49d713334761702d.size
        var outputbulkItem_g4_g0_bulkItem_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g4_g0_bulkItem_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g4_g0_bulkItem_g3_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            outputbulkItem_g4_g0_bulkItem_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[0])) as Long))
            if ((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) != 0L) {
            outputbulkItem_g4_g0_bulkItem_g1_g0_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) as Long))
            }
            outputbulkItem_g4_g0_bulkItem_g3_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[3])) as Long))

outputbulkItem_g4_g0_bulkList!!.add(io.prebindgen.covertest.callbackreturns.CallbackFixedResult(requireNotNull(outputbulkItem_g4_g0_bulkItem_g0_0), if ((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) != 0L) requireNotNull(outputbulkItem_g4_g0_bulkItem_g1_g0_0) else null, (requireNotNull(outputbulkReader).int((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[2])) as Int, requireNotNull(outputbulkItem_g4_g0_bulkItem_g3_0)))
}
            outputbulkItem_g5_0 = requireNotNull(outputowners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[5])) as Long))

outputbulkList!!.add(io.prebindgen.covertest.callbackreturns.CallbackNestedBatch(requireNotNull(outputbulkItem_g0_0), requireNotNull(outputbulkItem_g1_0), requireNotNull(outputbulkItem_g2_g0_bulkList), (requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[3])) as Int, requireNotNull(outputbulkItem_g4_g0_bulkList), requireNotNull(outputbulkItem_g5_0)))
}

outputowners!!.complete = true
            requireNotNull(outputbulkList)
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
outputowners?.close(failure)
if (outputowners?.complete != true) {
try { if (outputbulkRange == null) outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r4e69bfcf823e0723.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.size)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.size
var outputbulkFailure: Throwable? = failure
try {         var outputbulkItem_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputbulkItem_g2_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_g2_g0_bulkList: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>?>? = null
var outputbulkItem_g4_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_g4_g0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>? = null
        var outputbulkItem_g5_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
try { if (outputbulkItem_g2_g0_bulkRange == null) outputbulkItem_g2_g0_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[2])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.size)
for (outputbulkItem_g2_g0_bulkIndex in 0 until outputbulkItem_g2_g0_bulkRange!!.length) {
val outputbulkItem_g2_g0_bulkBase = outputbulkItem_g2_g0_bulkRange!!.offset + outputbulkItem_g2_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.size
var outputbulkItem_g2_g0_bulkFailure: Throwable? = outputbulkFailure
try { var outputbulkItem_g2_g0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_g2_g0_bulkItem_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>? = null

            if ((requireNotNull(outputbulkReader).int((outputbulkItem_g2_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.offsets[0])) == 1) {
try { if (outputbulkItem_g2_g0_bulkItem_bulkRange == null) outputbulkItem_g2_g0_bulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkItem_g2_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r1bd5c2ae3f2b872e.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r49d713334761702d.size)
for (outputbulkItem_g2_g0_bulkItem_bulkIndex in 0 until outputbulkItem_g2_g0_bulkItem_bulkRange!!.length) {
val outputbulkItem_g2_g0_bulkItem_bulkBase = outputbulkItem_g2_g0_bulkItem_bulkRange!!.offset + outputbulkItem_g2_g0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.r49d713334761702d.size
var outputbulkItem_g2_g0_bulkItem_bulkFailure: Throwable? = outputbulkItem_g2_g0_bulkFailure
try {         var outputbulkItem_g2_g0_bulkItem_bulkItem_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g2_g0_bulkItem_bulkItem_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g2_g0_bulkItem_bulkItem_g3_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_g2_g0_bulkItem_bulkFailure == null) outputbulkItem_g2_g0_bulkItem_bulkFailure = cleanup else if (outputbulkItem_g2_g0_bulkItem_bulkFailure !== cleanup) outputbulkItem_g2_g0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            if ((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_g2_g0_bulkItem_bulkFailure == null) outputbulkItem_g2_g0_bulkItem_bulkFailure = cleanup else if (outputbulkItem_g2_g0_bulkItem_bulkFailure !== cleanup) outputbulkItem_g2_g0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
            }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkItem_g2_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[3])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_g2_g0_bulkItem_bulkFailure == null) outputbulkItem_g2_g0_bulkItem_bulkFailure = cleanup else if (outputbulkItem_g2_g0_bulkItem_bulkFailure !== cleanup) outputbulkItem_g2_g0_bulkItem_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_g2_g0_bulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_g2_g0_bulkItem_bulkFailure == null) outputbulkItem_g2_g0_bulkItem_bulkFailure = outputbulkItem_g2_g0_bulkItem_bulkCleanupError else if (outputbulkItem_g2_g0_bulkItem_bulkFailure !== outputbulkItem_g2_g0_bulkItem_bulkCleanupError) outputbulkItem_g2_g0_bulkItem_bulkFailure!!.addSuppressed(outputbulkItem_g2_g0_bulkItem_bulkCleanupError) }
outputbulkItem_g2_g0_bulkFailure = outputbulkItem_g2_g0_bulkItem_bulkFailure
} } catch (outputbulkItem_g2_g0_bulkItem_bulkCleanupError: Throwable) { if (outputbulkItem_g2_g0_bulkFailure == null) outputbulkItem_g2_g0_bulkFailure = outputbulkItem_g2_g0_bulkItem_bulkCleanupError else if (outputbulkItem_g2_g0_bulkFailure !== outputbulkItem_g2_g0_bulkItem_bulkCleanupError) outputbulkItem_g2_g0_bulkFailure!!.addSuppressed(outputbulkItem_g2_g0_bulkItem_bulkCleanupError) }
            }
 } catch (outputbulkItem_g2_g0_bulkCleanupError: Throwable) { if (outputbulkItem_g2_g0_bulkFailure == null) outputbulkItem_g2_g0_bulkFailure = outputbulkItem_g2_g0_bulkCleanupError else if (outputbulkItem_g2_g0_bulkFailure !== outputbulkItem_g2_g0_bulkCleanupError) outputbulkItem_g2_g0_bulkFailure!!.addSuppressed(outputbulkItem_g2_g0_bulkCleanupError) }
outputbulkFailure = outputbulkItem_g2_g0_bulkFailure
} } catch (outputbulkItem_g2_g0_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_g2_g0_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_g2_g0_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_g2_g0_bulkCleanupError) }
try { if (outputbulkItem_g4_g0_bulkRange == null) outputbulkItem_g4_g0_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[4])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r49d713334761702d.size)
for (outputbulkItem_g4_g0_bulkIndex in 0 until outputbulkItem_g4_g0_bulkRange!!.length) {
val outputbulkItem_g4_g0_bulkBase = outputbulkItem_g4_g0_bulkRange!!.offset + outputbulkItem_g4_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r49d713334761702d.size
var outputbulkItem_g4_g0_bulkFailure: Throwable? = outputbulkFailure
try {         var outputbulkItem_g4_g0_bulkItem_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g4_g0_bulkItem_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputbulkItem_g4_g0_bulkItem_g3_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[0])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_g4_g0_bulkFailure == null) outputbulkItem_g4_g0_bulkFailure = cleanup else if (outputbulkItem_g4_g0_bulkFailure !== cleanup) outputbulkItem_g4_g0_bulkFailure!!.addSuppressed(cleanup) }
            if ((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) != 0L) {
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[1])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_g4_g0_bulkFailure == null) outputbulkItem_g4_g0_bulkFailure = cleanup else if (outputbulkItem_g4_g0_bulkFailure !== cleanup) outputbulkItem_g4_g0_bulkFailure!!.addSuppressed(cleanup) }
            }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkItem_g4_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r49d713334761702d.offsets[3])) as Long) } } catch (cleanup: Throwable) { if (outputbulkItem_g4_g0_bulkFailure == null) outputbulkItem_g4_g0_bulkFailure = cleanup else if (outputbulkItem_g4_g0_bulkFailure !== cleanup) outputbulkItem_g4_g0_bulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkItem_g4_g0_bulkCleanupError: Throwable) { if (outputbulkItem_g4_g0_bulkFailure == null) outputbulkItem_g4_g0_bulkFailure = outputbulkItem_g4_g0_bulkCleanupError else if (outputbulkItem_g4_g0_bulkFailure !== outputbulkItem_g4_g0_bulkCleanupError) outputbulkItem_g4_g0_bulkFailure!!.addSuppressed(outputbulkItem_g4_g0_bulkCleanupError) }
outputbulkFailure = outputbulkItem_g4_g0_bulkFailure
} } catch (outputbulkItem_g4_g0_bulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkItem_g4_g0_bulkCleanupError else if (outputbulkFailure !== outputbulkItem_g4_g0_bulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkItem_g4_g0_bulkCleanupError) }
            try { if (outputowners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r9d90eb6e2d1173db.offsets[5])) as Long) } } catch (cleanup: Throwable) { if (outputbulkFailure == null) outputbulkFailure = cleanup else if (outputbulkFailure !== cleanup) outputbulkFailure!!.addSuppressed(cleanup) }
 } catch (outputbulkCleanupError: Throwable) { if (outputbulkFailure == null) outputbulkFailure = outputbulkCleanupError else if (outputbulkFailure !== outputbulkCleanupError) outputbulkFailure!!.addSuppressed(outputbulkCleanupError) }

} } catch (outputbulkCleanupError: Throwable) { if (failure !== outputbulkCleanupError) failure.addSuppressed(outputbulkCleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun callbackMixedBatches(callback: io.prebindgen.covertest.MixedBatchesCallback, retry: Boolean, worker: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackNestedBatch>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.let { callback_return_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w1_snapshot.size) { callback_return_w1_index -> val callback_return_w5 = callback_return_w1_snapshot[callback_return_w1_index]
val __pb_callback_return_w5_field0 = callback_return_w5.first
val __pb_callback_return_w5_field1 = callback_return_w5.observed
val __pb_callback_return_w5_field2 = callback_return_w5.groups
val __pb_callback_return_w5_field3 = callback_return_w5.marker
val __pb_callback_return_w5_field4 = callback_return_w5.tail
val __pb_callback_return_w5_field5 = callback_return_w5.last
val callback_return_w9_elements = if (true) __pb_callback_return_w5_field2?.toList() else null
val callback_return_w8_writer_state = if (true) run { val callback_return_w8_writer = io.prebindgen.covertest.BulkInputs.Input_Range_31afb2a04211128a()
val callback_return_w9_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w9_elements }
if (callback_return_w9_bulkList == null) { callback_return_w8_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r31afb2a04211128a.offsets[0]) } else {
val callback_return_w9_bulkStart = callback_return_w8_writer.rows(callback_return_w9_bulkList.size, io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2)
callback_return_w8_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r31afb2a04211128a.offsets[0], callback_return_w9_bulkStart, callback_return_w9_bulkList.size)
callback_return_w9_bulkList.forEachIndexed { callback_return_w9_bulkIndex, callback_return_w15 ->
val callback_return_w15_elements = if (callback_return_w15 != null) (if (true) requireNotNull(callback_return_w15).toList() else null) else (null)
callback_return_w8_writer.int((callback_return_w9_bulkStart + callback_return_w9_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.size) + io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.offsets[0], io.prebindgen.covertest.InputValidation.value { (if (callback_return_w15 != null) 1 else 0) })
val callback_return_w15_bulkList = io.prebindgen.covertest.InputValidation.value { if (callback_return_w15 != null) callback_return_w15_elements else null }
if (callback_return_w15_bulkList == null) { callback_return_w8_writer.absent((callback_return_w9_bulkStart + callback_return_w9_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.size) + io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.offsets[1]) } else {
val callback_return_w15_bulkStart = callback_return_w8_writer.rows(callback_return_w15_bulkList.size, io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3)
callback_return_w8_writer.range((callback_return_w9_bulkStart + callback_return_w9_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.size) + io.prebindgen.covertest.BulkLayouts.r1033cb21c6bf06e2.offsets[1], callback_return_w15_bulkStart, callback_return_w15_bulkList.size)
callback_return_w15_bulkList.forEachIndexed { callback_return_w15_bulkIndex, callback_return_w19 ->
val __pb_callback_return_w19_field0 = callback_return_w19.first
val __pb_callback_return_w19_field1 = callback_return_w19.optional
val __pb_callback_return_w19_field2 = callback_return_w19.character
val __pb_callback_return_w19_field3 = callback_return_w19.last
callback_return_w8_writer.handle((callback_return_w15_bulkStart + callback_return_w15_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w19_field0 }, true, 0)
callback_return_w8_writer.handle((callback_return_w15_bulkStart + callback_return_w15_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[1], io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w19_field1 != null) requireNotNull(if (true) requireNotNull(__pb_callback_return_w19_field1) else null) { "invalid option payload" } else null }, true, 0)
callback_return_w8_writer.int((callback_return_w15_bulkStart + callback_return_w15_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[2], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w19_field2 })
callback_return_w8_writer.handle((callback_return_w15_bulkStart + callback_return_w15_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[3], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w19_field3 }, true, 0)
}
}
}
}

callback_return_w8_writer } else null
val callback_return_w25_elements = if (true) __pb_callback_return_w5_field4?.toList() else null
val callback_return_w24_writer_state = if (true) run { val callback_return_w24_writer = io.prebindgen.covertest.BulkInputs.Input_Range_08a37f4477600073()
val callback_return_w25_bulkList = io.prebindgen.covertest.InputValidation.value { callback_return_w25_elements }
if (callback_return_w25_bulkList == null) { callback_return_w24_writer.absent((0) + io.prebindgen.covertest.BulkLayouts.r08a37f4477600073.offsets[0]) } else {
val callback_return_w25_bulkStart = callback_return_w24_writer.rows(callback_return_w25_bulkList.size, io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3)
callback_return_w24_writer.range((0) + io.prebindgen.covertest.BulkLayouts.r08a37f4477600073.offsets[0], callback_return_w25_bulkStart, callback_return_w25_bulkList.size)
callback_return_w25_bulkList.forEachIndexed { callback_return_w25_bulkIndex, callback_return_w29 ->
val __pb_callback_return_w29_field0 = callback_return_w29.first
val __pb_callback_return_w29_field1 = callback_return_w29.optional
val __pb_callback_return_w29_field2 = callback_return_w29.character
val __pb_callback_return_w29_field3 = callback_return_w29.last
callback_return_w24_writer.handle((callback_return_w25_bulkStart + callback_return_w25_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[0], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w29_field0 }, true, 0)
callback_return_w24_writer.handle((callback_return_w25_bulkStart + callback_return_w25_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[1], io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w29_field1 != null) requireNotNull(if (true) requireNotNull(__pb_callback_return_w29_field1) else null) { "invalid option payload" } else null }, true, 0)
callback_return_w24_writer.int((callback_return_w25_bulkStart + callback_return_w25_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[2], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w29_field2 })
callback_return_w24_writer.handle((callback_return_w25_bulkStart + callback_return_w25_bulkIndex * io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.size) + io.prebindgen.covertest.BulkLayouts.r3f18e0d568ad44e3.offsets[3], io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w29_field3 }, true, 0)
}
}

callback_return_w24_writer } else null
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w5_field0 }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w5_field1 }, io.prebindgen.covertest.InputValidation.value { callback_return_w8_writer_state }, io.prebindgen.covertest.InputValidation.value { callback_return_w8_writer_state?.length ?: 0 }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w5_field3 }, io.prebindgen.covertest.InputValidation.value { callback_return_w24_writer_state }, io.prebindgen.covertest.InputValidation.value { callback_return_w24_writer_state?.length ?: 0 }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w5_field5 }) }, intArrayOf(2, 1, 0, 0, 0, 0, 0, 2), 2, arrayOf(intArrayOf(1), intArrayOf(0), intArrayOf(1), intArrayOf(), intArrayOf(), intArrayOf(1), intArrayOf(), intArrayOf(1))) } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_elements?.handleGroup(0).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_elements?.handleGroup(1).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withMixedInputLocks((__pb_handle0), (__pb_handle1)) { val callbackReturnWire = callback_return_w1_elements?.encode()
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_elements?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? Iterable<*>)?.forEach { returnedValue0 -> (returnedValue0 as? io.prebindgen.covertest.callbackreturns.CallbackNestedBatch)?.let { returnedValue1 -> (returnedValue1.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue3 -> returnedHandles.add(returnedValue3); returnedOwners.add(returnedValue3) }
(returnedValue1.observed as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue7 -> returnedHandles.add(returnedValue7);  }
(returnedValue1.groups as? Iterable<*>)?.forEach { returnedValue10 -> (returnedValue10 as? Iterable<*>)?.forEach { returnedValue12 -> (returnedValue12 as? io.prebindgen.covertest.callbackreturns.CallbackFixedResult)?.let { returnedValue13 -> (returnedValue13.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue15 -> returnedHandles.add(returnedValue15); returnedOwners.add(returnedValue15) }
(returnedValue13.optional as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue18 -> returnedHandles.add(returnedValue18); returnedOwners.add(returnedValue18) }
(returnedValue13.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue20 -> returnedHandles.add(returnedValue20); returnedOwners.add(returnedValue20) }
 }
 }
 }
(returnedValue1.tail as? Iterable<*>)?.forEach { returnedValue23 -> (returnedValue23 as? io.prebindgen.covertest.callbackreturns.CallbackFixedResult)?.let { returnedValue24 -> (returnedValue24.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue26 -> returnedHandles.add(returnedValue26); returnedOwners.add(returnedValue26) }
(returnedValue24.optional as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue29 -> returnedHandles.add(returnedValue29); returnedOwners.add(returnedValue29) }
(returnedValue24.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue31 -> returnedHandles.add(returnedValue31); returnedOwners.add(returnedValue31) }
 }
 }
(returnedValue1.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue33 -> returnedHandles.add(returnedValue33); returnedOwners.add(returnedValue33) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { retry } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { worker } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_mixed_batches_2277ce12060893e9(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackNestedBatchEcho(batch: io.prebindgen.covertest.callbackreturns.CallbackNestedBatch, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbackreturns.CallbackNestedBatch> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.callbackreturns.CallbackNestedBatch = run {
    val __pb_p0_w1_field0 = try { batch.first } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val __pb_p0_w1_field1 = try { batch.observed } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val __pb_p0_w1_field2 = try { batch.groups } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val __pb_p0_w1_field3 = try { batch.marker } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val __pb_p0_w1_field4 = try { batch.tail } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val __pb_p0_w1_field5 = try { batch.last } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w2_elements = try { if (true) __pb_p0_w1_field2?.toList()?.let { p0_w2_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(p0_w2_snapshot.size) { p0_w2_index -> val p0_w8 = p0_w2_snapshot[p0_w2_index]
val p0_w8_elements = if (p0_w8 != null) (if (true) requireNotNull(p0_w8).toList()?.let { p0_w8_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(p0_w8_snapshot.size) { p0_w8_index -> val p0_w12 = p0_w8_snapshot[p0_w8_index]
val __pb_p0_w12_field0 = p0_w12.first
val __pb_p0_w12_field1 = p0_w12.optional
val __pb_p0_w12_field2 = p0_w12.character
val __pb_p0_w12_field3 = p0_w12.last
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { __pb_p0_w12_field0 }, io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w12_field1 != null) requireNotNull(if (true) requireNotNull(__pb_p0_w12_field1) else null) { "invalid option payload" } else null }, io.prebindgen.covertest.InputValidation.value { __pb_p0_w12_field2 }, io.prebindgen.covertest.InputValidation.value { __pb_p0_w12_field3 }) }, intArrayOf(2, 2, 0, 2), 1, null) } else null) else (null)
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { if (p0_w8 != null) requireNotNull(p0_w8_elements) { "invalid option payload" } else null }) }, null, 1, null) } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val p0_w15_elements = try { if (true) __pb_p0_w1_field4?.toList()?.let { p0_w15_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(p0_w15_snapshot.size) { p0_w15_index -> val p0_w19 = p0_w15_snapshot[p0_w15_index]
val __pb_p0_w19_field0 = p0_w19.first
val __pb_p0_w19_field1 = p0_w19.optional
val __pb_p0_w19_field2 = p0_w19.character
val __pb_p0_w19_field3 = p0_w19.last
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { __pb_p0_w19_field0 }, io.prebindgen.covertest.InputValidation.value { if (__pb_p0_w19_field1 != null) requireNotNull(if (true) requireNotNull(__pb_p0_w19_field1) else null) { "invalid option payload" } else null }, io.prebindgen.covertest.InputValidation.value { __pb_p0_w19_field2 }, io.prebindgen.covertest.InputValidation.value { __pb_p0_w19_field3 }) }, intArrayOf(2, 2, 0, 2), 1, null) } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Long_Long_Array_Any__6_ErrorEmpty_5b53ecd9dac487e2.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { __pb_p0_w1_field0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.covertest.NativeHandle? = try { __pb_p0_w1_field1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle2: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w2_elements?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle3: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w15_elements?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle4: io.prebindgen.covertest.NativeHandle? = try { __pb_p0_w1_field5 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks(((kotlin.collections.listOfNotNull(__pb_handle0, __pb_handle1, __pb_handle4) + (__pb_handle2)) + (__pb_handle3)), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_handle1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p0_w2_elements?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_p0_w1_field3 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { p0_w15_elements?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { __pb_handle4?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument7 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_nested_batch_echo_3dada1cd491c5035(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6, nativeArgument7) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; __pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) }; p0_w2_elements?.consume(__pb_consume); p0_w15_elements?.consume(__pb_consume); __pb_handle4?.let { __pb_consume(it) } } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; __pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) }; p0_w2_elements?.consume(__pb_consume); p0_w15_elements?.consume(__pb_consume); __pb_handle4?.let { __pb_consume(it) } } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputg2_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg2_g0_list: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>?>? = null
var outputg4_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg4_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>? = null
        var outputg5_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
             try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
outputg2_g0_owners?.close(failure)
if (outputg2_g0_owners?.complete != true) {
try {
for (outputg2_g0_index in (returnCapture.s2 as kotlin.Array<*>).indices) {
var outputg2_g0_failure: kotlin.Throwable? = failure
try { val outputg2_g0__row = (returnCapture.s2 as kotlin.Array<*>)[outputg2_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputg2_g0_item_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>? = null

            if (outputg2_g0__row.c0 != null) {
try {
for (outputg2_g0_item_index in (outputg2_g0__row.c0 as kotlin.Array<*>).indices) {
var outputg2_g0_item_failure: kotlin.Throwable? = outputg2_g0_failure
try { val outputg2_g0_item__row = (outputg2_g0__row.c0 as kotlin.Array<*>)[outputg2_g0_item_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Int_4_851daa809dd7a1b1
        var outputg2_g0_item_item_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg2_g0_item_item_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg2_g0_item_item_g3_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            try { if (outputg2_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(outputg2_g0_item__row.c0) } } catch (cleanup: Throwable) { if (outputg2_g0_item_failure == null) outputg2_g0_item_failure = cleanup else if (outputg2_g0_item_failure !== cleanup) outputg2_g0_item_failure!!.addSuppressed(cleanup) }
            if (outputg2_g0_item__row.c1 != 0L) {
            try { if (outputg2_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(outputg2_g0_item__row.c1) } } catch (cleanup: Throwable) { if (outputg2_g0_item_failure == null) outputg2_g0_item_failure = cleanup else if (outputg2_g0_item_failure !== cleanup) outputg2_g0_item_failure!!.addSuppressed(cleanup) }
            }
            try { if (outputg2_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(outputg2_g0_item__row.c3) } } catch (cleanup: Throwable) { if (outputg2_g0_item_failure == null) outputg2_g0_item_failure = cleanup else if (outputg2_g0_item_failure !== cleanup) outputg2_g0_item_failure!!.addSuppressed(cleanup) }

} catch (outputg2_g0_item_cleanupError: Throwable) { if (outputg2_g0_item_failure == null) outputg2_g0_item_failure = outputg2_g0_item_cleanupError else if (outputg2_g0_item_failure !== outputg2_g0_item_cleanupError) outputg2_g0_item_failure!!.addSuppressed(outputg2_g0_item_cleanupError) }
outputg2_g0_failure = outputg2_g0_item_failure
}
} catch (outputg2_g0_item_cleanupError: Throwable) { if (outputg2_g0_failure == null) outputg2_g0_failure = outputg2_g0_item_cleanupError else if (outputg2_g0_failure !== outputg2_g0_item_cleanupError) outputg2_g0_failure!!.addSuppressed(outputg2_g0_item_cleanupError) }
            }

} catch (outputg2_g0_cleanupError: Throwable) { if (outputg2_g0_failure == null) outputg2_g0_failure = outputg2_g0_cleanupError else if (outputg2_g0_failure !== outputg2_g0_cleanupError) outputg2_g0_failure!!.addSuppressed(outputg2_g0_cleanupError) }

}
} catch (outputg2_g0_cleanupError: Throwable) { if (failure !== outputg2_g0_cleanupError) failure.addSuppressed(outputg2_g0_cleanupError) }

}
outputg4_g0_owners?.close(failure)
if (outputg4_g0_owners?.complete != true) {
try {
for (outputg4_g0_index in (returnCapture.s4 as kotlin.Array<*>).indices) {
var outputg4_g0_failure: kotlin.Throwable? = failure
try { val outputg4_g0__row = (returnCapture.s4 as kotlin.Array<*>)[outputg4_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Int_4_851daa809dd7a1b1
        var outputg4_g0_item_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg4_g0_item_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg4_g0_item_g3_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            try { if (outputg4_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(outputg4_g0__row.c0) } } catch (cleanup: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = cleanup else if (outputg4_g0_failure !== cleanup) outputg4_g0_failure!!.addSuppressed(cleanup) }
            if (outputg4_g0__row.c1 != 0L) {
            try { if (outputg4_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(outputg4_g0__row.c1) } } catch (cleanup: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = cleanup else if (outputg4_g0_failure !== cleanup) outputg4_g0_failure!!.addSuppressed(cleanup) }
            }
            try { if (outputg4_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(outputg4_g0__row.c3) } } catch (cleanup: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = cleanup else if (outputg4_g0_failure !== cleanup) outputg4_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg4_g0_cleanupError: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = outputg4_g0_cleanupError else if (outputg4_g0_failure !== outputg4_g0_cleanupError) outputg4_g0_failure!!.addSuppressed(outputg4_g0_cleanupError) }

}
} catch (outputg4_g0_cleanupError: Throwable) { if (failure !== outputg4_g0_cleanupError) failure.addSuppressed(outputg4_g0_cleanupError) }

}
            try { val owner = outputg5_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s5) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg1_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
var outputg2_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg2_g0_list: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>?>? = null
var outputg4_g0_owners: io.prebindgen.covertest.CollectionOwners? = null
var outputg4_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>? = null
        var outputg5_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        try {
            outputg0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(returnCapture.s0)
            outputg1_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(returnCapture.s1)
outputg2_g0_owners = io.prebindgen.covertest.CollectionOwners()
outputg2_g0_list = kotlin.collections.ArrayList((returnCapture.s2 as kotlin.Array<*>).size)
for (outputg2_g0_index in (returnCapture.s2 as kotlin.Array<*>).indices) {
val outputg2_g0__row = (returnCapture.s2 as kotlin.Array<*>)[outputg2_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputg2_g0_item_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>? = null

            if (outputg2_g0__row.c0 != null) {
outputg2_g0_item_list = kotlin.collections.ArrayList((outputg2_g0__row.c0 as kotlin.Array<*>).size)
for (outputg2_g0_item_index in (outputg2_g0__row.c0 as kotlin.Array<*>).indices) {
val outputg2_g0_item__row = (outputg2_g0__row.c0 as kotlin.Array<*>)[outputg2_g0_item_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Int_4_851daa809dd7a1b1
        var outputg2_g0_item_item_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg2_g0_item_item_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg2_g0_item_item_g3_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            outputg2_g0_item_item_g0_0 = requireNotNull(outputg2_g0_owners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource(outputg2_g0_item__row.c0))
            if (outputg2_g0_item__row.c1 != 0L) {
            outputg2_g0_item_item_g1_g0_0 = requireNotNull(outputg2_g0_owners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource(outputg2_g0_item__row.c1))
            }
            outputg2_g0_item_item_g3_0 = requireNotNull(outputg2_g0_owners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource(outputg2_g0_item__row.c3))

outputg2_g0_item_list.add(io.prebindgen.covertest.callbackreturns.CallbackFixedResult(requireNotNull(outputg2_g0_item_item_g0_0), if (outputg2_g0_item__row.c1 != 0L) requireNotNull(outputg2_g0_item_item_g1_g0_0) else null, outputg2_g0_item__row.c2, requireNotNull(outputg2_g0_item_item_g3_0)))
}
            }

outputg2_g0_list.add(if (outputg2_g0__row.c0 != null) requireNotNull(outputg2_g0_item_list) else null)
}

outputg2_g0_owners!!.complete = true
outputg4_g0_owners = io.prebindgen.covertest.CollectionOwners()
outputg4_g0_list = kotlin.collections.ArrayList((returnCapture.s4 as kotlin.Array<*>).size)
for (outputg4_g0_index in (returnCapture.s4 as kotlin.Array<*>).indices) {
val outputg4_g0__row = (returnCapture.s4 as kotlin.Array<*>)[outputg4_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Int_4_851daa809dd7a1b1
        var outputg4_g0_item_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg4_g0_item_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg4_g0_item_g3_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            outputg4_g0_item_g0_0 = requireNotNull(outputg4_g0_owners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource(outputg4_g0__row.c0))
            if (outputg4_g0__row.c1 != 0L) {
            outputg4_g0_item_g1_g0_0 = requireNotNull(outputg4_g0_owners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource(outputg4_g0__row.c1))
            }
            outputg4_g0_item_g3_0 = requireNotNull(outputg4_g0_owners).own(io.prebindgen.covertest.callbackreturns.CallbackReturnResource(outputg4_g0__row.c3))

outputg4_g0_list.add(io.prebindgen.covertest.callbackreturns.CallbackFixedResult(requireNotNull(outputg4_g0_item_g0_0), if (outputg4_g0__row.c1 != 0L) requireNotNull(outputg4_g0_item_g1_g0_0) else null, outputg4_g0__row.c2, requireNotNull(outputg4_g0_item_g3_0)))
}

outputg4_g0_owners!!.complete = true
            outputg5_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(returnCapture.s5)
            io.prebindgen.covertest.callbackreturns.CallbackNestedBatch(requireNotNull(outputg0_0), requireNotNull(outputg1_0), requireNotNull(outputg2_g0_list), returnCapture.s3, requireNotNull(outputg4_g0_list), requireNotNull(outputg5_0))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
outputg2_g0_owners?.close(failure)
if (outputg2_g0_owners?.complete != true) {
try {
for (outputg2_g0_index in (returnCapture.s2 as kotlin.Array<*>).indices) {
var outputg2_g0_failure: kotlin.Throwable? = failure
try { val outputg2_g0__row = (returnCapture.s2 as kotlin.Array<*>)[outputg2_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var outputg2_g0_item_list: kotlin.collections.MutableList<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>? = null

            if (outputg2_g0__row.c0 != null) {
try {
for (outputg2_g0_item_index in (outputg2_g0__row.c0 as kotlin.Array<*>).indices) {
var outputg2_g0_item_failure: kotlin.Throwable? = outputg2_g0_failure
try { val outputg2_g0_item__row = (outputg2_g0__row.c0 as kotlin.Array<*>)[outputg2_g0_item_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Int_4_851daa809dd7a1b1
        var outputg2_g0_item_item_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg2_g0_item_item_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg2_g0_item_item_g3_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            try { if (outputg2_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(outputg2_g0_item__row.c0) } } catch (cleanup: Throwable) { if (outputg2_g0_item_failure == null) outputg2_g0_item_failure = cleanup else if (outputg2_g0_item_failure !== cleanup) outputg2_g0_item_failure!!.addSuppressed(cleanup) }
            if (outputg2_g0_item__row.c1 != 0L) {
            try { if (outputg2_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(outputg2_g0_item__row.c1) } } catch (cleanup: Throwable) { if (outputg2_g0_item_failure == null) outputg2_g0_item_failure = cleanup else if (outputg2_g0_item_failure !== cleanup) outputg2_g0_item_failure!!.addSuppressed(cleanup) }
            }
            try { if (outputg2_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(outputg2_g0_item__row.c3) } } catch (cleanup: Throwable) { if (outputg2_g0_item_failure == null) outputg2_g0_item_failure = cleanup else if (outputg2_g0_item_failure !== cleanup) outputg2_g0_item_failure!!.addSuppressed(cleanup) }

} catch (outputg2_g0_item_cleanupError: Throwable) { if (outputg2_g0_item_failure == null) outputg2_g0_item_failure = outputg2_g0_item_cleanupError else if (outputg2_g0_item_failure !== outputg2_g0_item_cleanupError) outputg2_g0_item_failure!!.addSuppressed(outputg2_g0_item_cleanupError) }
outputg2_g0_failure = outputg2_g0_item_failure
}
} catch (outputg2_g0_item_cleanupError: Throwable) { if (outputg2_g0_failure == null) outputg2_g0_failure = outputg2_g0_item_cleanupError else if (outputg2_g0_failure !== outputg2_g0_item_cleanupError) outputg2_g0_failure!!.addSuppressed(outputg2_g0_item_cleanupError) }
            }

} catch (outputg2_g0_cleanupError: Throwable) { if (outputg2_g0_failure == null) outputg2_g0_failure = outputg2_g0_cleanupError else if (outputg2_g0_failure !== outputg2_g0_cleanupError) outputg2_g0_failure!!.addSuppressed(outputg2_g0_cleanupError) }

}
} catch (outputg2_g0_cleanupError: Throwable) { if (failure !== outputg2_g0_cleanupError) failure.addSuppressed(outputg2_g0_cleanupError) }

}
outputg4_g0_owners?.close(failure)
if (outputg4_g0_owners?.complete != true) {
try {
for (outputg4_g0_index in (returnCapture.s4 as kotlin.Array<*>).indices) {
var outputg4_g0_failure: kotlin.Throwable? = failure
try { val outputg4_g0__row = (returnCapture.s4 as kotlin.Array<*>)[outputg4_g0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Long_Int_4_851daa809dd7a1b1
        var outputg4_g0_item_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg4_g0_item_g1_g0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        var outputg4_g0_item_g3_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null

            try { if (outputg4_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(outputg4_g0__row.c0) } } catch (cleanup: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = cleanup else if (outputg4_g0_failure !== cleanup) outputg4_g0_failure!!.addSuppressed(cleanup) }
            if (outputg4_g0__row.c1 != 0L) {
            try { if (outputg4_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(outputg4_g0__row.c1) } } catch (cleanup: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = cleanup else if (outputg4_g0_failure !== cleanup) outputg4_g0_failure!!.addSuppressed(cleanup) }
            }
            try { if (outputg4_g0_owners?.unread() != false) { io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(outputg4_g0__row.c3) } } catch (cleanup: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = cleanup else if (outputg4_g0_failure !== cleanup) outputg4_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg4_g0_cleanupError: Throwable) { if (outputg4_g0_failure == null) outputg4_g0_failure = outputg4_g0_cleanupError else if (outputg4_g0_failure !== outputg4_g0_cleanupError) outputg4_g0_failure!!.addSuppressed(outputg4_g0_cleanupError) }

}
} catch (outputg4_g0_cleanupError: Throwable) { if (failure !== outputg4_g0_cleanupError) failure.addSuppressed(outputg4_g0_cleanupError) }

}
            try { val owner = outputg5_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(returnCapture.s5) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun callbackNestedResults(callback: io.prebindgen.covertest.NestedResultsCallback, retry: Boolean, worker: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>?>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.let { callback_return_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w1_snapshot.size) { callback_return_w1_index -> val callback_return_w7 = callback_return_w1_snapshot[callback_return_w1_index]
val callback_return_w7_elements = if (callback_return_w7 != null) (if (true) requireNotNull(callback_return_w7).toList()?.let { callback_return_w7_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w7_snapshot.size) { callback_return_w7_index -> val callback_return_w11 = callback_return_w7_snapshot[callback_return_w7_index]
val __pb_callback_return_w11_field0 = callback_return_w11.first
val __pb_callback_return_w11_field1 = callback_return_w11.optional
val __pb_callback_return_w11_field2 = callback_return_w11.character
val __pb_callback_return_w11_field3 = callback_return_w11.last
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w11_field0 }, io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w11_field1 != null) requireNotNull(if (true) requireNotNull(__pb_callback_return_w11_field1) else null) { "invalid option payload" } else null }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w11_field2 }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w11_field3 }) }, intArrayOf(2, 2, 0, 2), 1, null) } else null) else (null)
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { if (callback_return_w7 != null) requireNotNull(callback_return_w7_elements) { "invalid option payload" } else null }) }, null, 1, null) } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_elements?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = callback_return_w1_elements?.encode()
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_elements?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? Iterable<*>)?.forEach { returnedValue0 -> (returnedValue0 as? Iterable<*>)?.forEach { returnedValue2 -> (returnedValue2 as? io.prebindgen.covertest.callbackreturns.CallbackFixedResult)?.let { returnedValue3 -> (returnedValue3.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue5 -> returnedHandles.add(returnedValue5); returnedOwners.add(returnedValue5) }
(returnedValue3.optional as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue8 -> returnedHandles.add(returnedValue8); returnedOwners.add(returnedValue8) }
(returnedValue3.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue10 -> returnedHandles.add(returnedValue10); returnedOwners.add(returnedValue10) }
 }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { retry } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { worker } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_nested_results_b9be5ae57009af02(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackNestedBatch(callback: io.prebindgen.covertest.NestedBatchCallback, retry: Boolean, worker: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackNestedBatch
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val __pb_callback_return_w1_field0 = callback_return_w1.first
val __pb_callback_return_w1_field1 = callback_return_w1.observed
val __pb_callback_return_w1_field2 = callback_return_w1.groups
val __pb_callback_return_w1_field3 = callback_return_w1.marker
val __pb_callback_return_w1_field4 = callback_return_w1.tail
val __pb_callback_return_w1_field5 = callback_return_w1.last
val callback_return_w4_elements = if (true) __pb_callback_return_w1_field2?.toList()?.let { callback_return_w4_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w4_snapshot.size) { callback_return_w4_index -> val callback_return_w10 = callback_return_w4_snapshot[callback_return_w4_index]
val callback_return_w10_elements = if (callback_return_w10 != null) (if (true) requireNotNull(callback_return_w10).toList()?.let { callback_return_w10_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w10_snapshot.size) { callback_return_w10_index -> val callback_return_w14 = callback_return_w10_snapshot[callback_return_w10_index]
val __pb_callback_return_w14_field0 = callback_return_w14.first
val __pb_callback_return_w14_field1 = callback_return_w14.optional
val __pb_callback_return_w14_field2 = callback_return_w14.character
val __pb_callback_return_w14_field3 = callback_return_w14.last
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w14_field0 }, io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w14_field1 != null) requireNotNull(if (true) requireNotNull(__pb_callback_return_w14_field1) else null) { "invalid option payload" } else null }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w14_field2 }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w14_field3 }) }, intArrayOf(2, 2, 0, 2), 1, null) } else null) else (null)
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { if (callback_return_w10 != null) requireNotNull(callback_return_w10_elements) { "invalid option payload" } else null }) }, null, 1, null) } else null
val callback_return_w19_elements = if (true) __pb_callback_return_w1_field4?.toList()?.let { callback_return_w19_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w19_snapshot.size) { callback_return_w19_index -> val callback_return_w23 = callback_return_w19_snapshot[callback_return_w19_index]
val __pb_callback_return_w23_field0 = callback_return_w23.first
val __pb_callback_return_w23_field1 = callback_return_w23.optional
val __pb_callback_return_w23_field2 = callback_return_w23.character
val __pb_callback_return_w23_field3 = callback_return_w23.last
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w23_field0 }, io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w23_field1 != null) requireNotNull(if (true) requireNotNull(__pb_callback_return_w23_field1) else null) { "invalid option payload" } else null }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w23_field2 }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w23_field3 }) }, intArrayOf(2, 2, 0, 2), 1, null) } else null
 run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { __pb_callback_return_w1_field0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.covertest.NativeHandle? = try { __pb_callback_return_w1_field1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle2: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w4_elements?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle3: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w19_elements?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle4: io.prebindgen.covertest.NativeHandle? = try { __pb_callback_return_w1_field5 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withMixedInputLocks(kotlin.collections.listOfNotNull(__pb_handle1), ((kotlin.collections.listOfNotNull(__pb_handle0, __pb_handle4) + (__pb_handle2)) + (__pb_handle3))) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(__pb_handle0?.raw ?: 0L, __pb_handle1?.raw ?: 0L, callback_return_w4_elements?.encode(), __pb_callback_return_w1_field3, callback_return_w19_elements?.encode(), __pb_handle4?.raw ?: 0L)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
__pb_handle0?.let { __pb_consume(it) }; callback_return_w4_elements?.consume(__pb_consume); callback_return_w19_elements?.consume(__pb_consume); __pb_handle4?.let { __pb_consume(it) }
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? io.prebindgen.covertest.callbackreturns.CallbackNestedBatch)?.let { returnedValue0 -> (returnedValue0.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue2 -> returnedHandles.add(returnedValue2); returnedOwners.add(returnedValue2) }
(returnedValue0.observed as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue6 -> returnedHandles.add(returnedValue6);  }
(returnedValue0.groups as? Iterable<*>)?.forEach { returnedValue8 -> (returnedValue8 as? Iterable<*>)?.forEach { returnedValue10 -> (returnedValue10 as? io.prebindgen.covertest.callbackreturns.CallbackFixedResult)?.let { returnedValue11 -> (returnedValue11.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue13 -> returnedHandles.add(returnedValue13); returnedOwners.add(returnedValue13) }
(returnedValue11.optional as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue16 -> returnedHandles.add(returnedValue16); returnedOwners.add(returnedValue16) }
(returnedValue11.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue18 -> returnedHandles.add(returnedValue18); returnedOwners.add(returnedValue18) }
 }
 }
 }
(returnedValue0.tail as? Iterable<*>)?.forEach { returnedValue20 -> (returnedValue20 as? io.prebindgen.covertest.callbackreturns.CallbackFixedResult)?.let { returnedValue21 -> (returnedValue21.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue23 -> returnedHandles.add(returnedValue23); returnedOwners.add(returnedValue23) }
(returnedValue21.optional as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue26 -> returnedHandles.add(returnedValue26); returnedOwners.add(returnedValue26) }
(returnedValue21.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue28 -> returnedHandles.add(returnedValue28); returnedOwners.add(returnedValue28) }
 }
 }
(returnedValue0.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue30 -> returnedHandles.add(returnedValue30); returnedOwners.add(returnedValue30) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { retry } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { worker } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_nested_batch_2c220f1124510108(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackFixedResult(callback: io.prebindgen.covertest.FixedResultCallback, retry: Boolean, worker: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackFixedResult
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val __pb_callback_return_w1_field0 = callback_return_w1.first
val __pb_callback_return_w1_field1 = callback_return_w1.optional
val __pb_callback_return_w1_field2 = callback_return_w1.character
val __pb_callback_return_w1_field3 = callback_return_w1.last
 run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { __pb_callback_return_w1_field0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: io.prebindgen.covertest.NativeHandle? = try { if (__pb_callback_return_w1_field1 != null) (if (true) requireNotNull(__pb_callback_return_w1_field1) else null) else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle2: io.prebindgen.covertest.NativeHandle? = try { __pb_callback_return_w1_field3 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; if (__pb_handle0 != null && __pb_handle1 != null && __pb_handle0.address == __pb_handle1.address) throw io.prebindgen.covertest.HandleValidationFailure("conflicting input aliases"); if (__pb_handle0 != null && __pb_handle2 != null && __pb_handle0.address == __pb_handle2.address) throw io.prebindgen.covertest.HandleValidationFailure("conflicting input aliases"); if (__pb_handle1 != null && __pb_handle2 != null && __pb_handle1.address == __pb_handle2.address) throw io.prebindgen.covertest.HandleValidationFailure("conflicting input aliases"); io.prebindgen.covertest.withHandleLocks(__pb_handle0, __pb_handle1, __pb_handle2) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(__pb_handle0?.raw ?: 0L, __pb_handle1?.raw ?: 0L, __pb_callback_return_w1_field2, __pb_handle2?.raw ?: 0L)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
__pb_handle0?.let { __pb_consume(it) }; __pb_handle1?.let { __pb_consume(it) }; __pb_handle2?.let { __pb_consume(it) }
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? io.prebindgen.covertest.callbackreturns.CallbackFixedResult)?.let { returnedValue0 -> (returnedValue0.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue2 -> returnedHandles.add(returnedValue2); returnedOwners.add(returnedValue2) }
(returnedValue0.optional as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue5 -> returnedHandles.add(returnedValue5); returnedOwners.add(returnedValue5) }
(returnedValue0.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue7 -> returnedHandles.add(returnedValue7); returnedOwners.add(returnedValue7) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { retry } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { worker } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_fixed_result_7b9617992129d1cc(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackFixedResults(callback: io.prebindgen.covertest.FixedResultsCallback, retry: Boolean, worker: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.let { callback_return_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w1_snapshot.size) { callback_return_w1_index -> val callback_return_w5 = callback_return_w1_snapshot[callback_return_w1_index]
val __pb_callback_return_w5_field0 = callback_return_w5.first
val __pb_callback_return_w5_field1 = callback_return_w5.optional
val __pb_callback_return_w5_field2 = callback_return_w5.character
val __pb_callback_return_w5_field3 = callback_return_w5.last
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w5_field0 }, io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w5_field1 != null) requireNotNull(if (true) requireNotNull(__pb_callback_return_w5_field1) else null) { "invalid option payload" } else null }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w5_field2 }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w5_field3 }) }, intArrayOf(2, 2, 0, 2), 1, null) } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_elements?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = callback_return_w1_elements?.encode()
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_elements?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? Iterable<*>)?.forEach { returnedValue0 -> (returnedValue0 as? io.prebindgen.covertest.callbackreturns.CallbackFixedResult)?.let { returnedValue1 -> (returnedValue1.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue3 -> returnedHandles.add(returnedValue3); returnedOwners.add(returnedValue3) }
(returnedValue1.optional as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue6 -> returnedHandles.add(returnedValue6); returnedOwners.add(returnedValue6) }
(returnedValue1.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue8 -> returnedHandles.add(returnedValue8); returnedOwners.add(returnedValue8) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { retry } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { worker } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_fixed_results_ee0852d3858ba775(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackFixedOptionalResults(callback: io.prebindgen.covertest.OptionalFixedResultsCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try { val callback_return_w3_elements = if (callback_return_w3 != null) (if (true) requireNotNull(callback_return_w3).toList()?.let { callback_return_w3_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w3_snapshot.size) { callback_return_w3_index -> val callback_return_w7 = callback_return_w3_snapshot[callback_return_w3_index]
val __pb_callback_return_w7_field0 = callback_return_w7.first
val __pb_callback_return_w7_field1 = callback_return_w7.optional
val __pb_callback_return_w7_field2 = callback_return_w7.character
val __pb_callback_return_w7_field3 = callback_return_w7.last
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w7_field0 }, io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w7_field1 != null) requireNotNull(if (true) requireNotNull(__pb_callback_return_w7_field1) else null) { "invalid option payload" } else null }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w7_field2 }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w7_field3 }) }, intArrayOf(2, 2, 0, 2), 1, null) } else null) else (null)
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { if (callback_return_w3 != null) (callback_return_w3_elements?.handles.orEmpty()) else kotlin.collections.emptyList() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = if (callback_return_w3 != null) requireNotNull(callback_return_w3_elements?.encode()) { "invalid option payload" } else null
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
if (callback_return_w3 != null) { callback_return_w3_elements?.consume(__pb_consume) }
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? Iterable<*>)?.forEach { returnedValue1 -> (returnedValue1 as? io.prebindgen.covertest.callbackreturns.CallbackFixedResult)?.let { returnedValue2 -> (returnedValue2.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue4 -> returnedHandles.add(returnedValue4); returnedOwners.add(returnedValue4) }
(returnedValue2.optional as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue7 -> returnedHandles.add(returnedValue7); returnedOwners.add(returnedValue7) }
(returnedValue2.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue9 -> returnedHandles.add(returnedValue9); returnedOwners.add(returnedValue9) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_fixed_optional_results_58d579f82e8266ff(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackFixedBorrowedResults(callback: io.prebindgen.covertest.BorrowedFixedResultsCallback, retry: Boolean, worker: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackFixedResult>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.let { callback_return_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w1_snapshot.size) { callback_return_w1_index -> val callback_return_w5 = callback_return_w1_snapshot[callback_return_w1_index]
val __pb_callback_return_w5_field0 = callback_return_w5.first
val __pb_callback_return_w5_field1 = callback_return_w5.optional
val __pb_callback_return_w5_field2 = callback_return_w5.character
val __pb_callback_return_w5_field3 = callback_return_w5.last
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w5_field0 }, io.prebindgen.covertest.InputValidation.value { if (__pb_callback_return_w5_field1 != null) requireNotNull(if (true) requireNotNull(__pb_callback_return_w5_field1) else null) { "invalid option payload" } else null }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w5_field2 }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w5_field3 }) }, intArrayOf(1, 2, 0, 2), 2, arrayOf(intArrayOf(0), intArrayOf(1), intArrayOf(), intArrayOf(1))) } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_elements?.handleGroup(0).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val __pb_handle1: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_elements?.handleGroup(1).orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withMixedInputLocks((__pb_handle0), (__pb_handle1)) { val callbackReturnWire = callback_return_w1_elements?.encode()
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_elements?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? Iterable<*>)?.forEach { returnedValue0 -> (returnedValue0 as? io.prebindgen.covertest.callbackreturns.CallbackFixedResult)?.let { returnedValue1 -> (returnedValue1.first as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue5 -> returnedHandles.add(returnedValue5);  }
(returnedValue1.optional as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue8 -> returnedHandles.add(returnedValue8); returnedOwners.add(returnedValue8) }
(returnedValue1.last as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue10 -> returnedHandles.add(returnedValue10); returnedOwners.add(returnedValue10) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { retry } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { worker } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_fixed_borrowed_results_e2d63138e2b54abf(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnWireRowsCatch(callback: io.prebindgen.covertest.VoidToCallbackWireRowVecCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackWireRow>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.let { callback_return_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w1_snapshot.size) { callback_return_w1_index -> val callback_return_w5 = callback_return_w1_snapshot[callback_return_w1_index]
val __pb_callback_return_w5_field0 = callback_return_w5.resource
val __pb_callback_return_w5_field1 = callback_return_w5.character
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w5_field0 }, io.prebindgen.covertest.InputValidation.value { __pb_callback_return_w5_field1 }) }, intArrayOf(2, 0), 1, null) } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_elements?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = callback_return_w1_elements?.encode()
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_elements?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? Iterable<*>)?.forEach { returnedValue0 -> (returnedValue0 as? io.prebindgen.covertest.callbackreturns.CallbackWireRow)?.let { returnedValue1 -> (returnedValue1.resource as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue3 -> returnedHandles.add(returnedValue3); returnedOwners.add(returnedValue3) }
 }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_wire_rows_catch_001b9a48fe8c7967(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnWireRowCatch(callback: io.prebindgen.covertest.VoidToCallbackWireRowCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackWireRow
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val __pb_callback_return_w1_field0 = callback_return_w1.resource
val __pb_callback_return_w1_field1 = callback_return_w1.character
 run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { __pb_callback_return_w1_field0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(__pb_handle0?.raw ?: 0L, __pb_callback_return_w1_field1)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
__pb_handle0?.let { __pb_consume(it) }
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? io.prebindgen.covertest.callbackreturns.CallbackWireRow)?.let { returnedValue0 -> (returnedValue0.resource as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue2 -> returnedHandles.add(returnedValue2); returnedOwners.add(returnedValue2) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_wire_row_catch_05921b2c51a5708c(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnI64(callback: io.prebindgen.covertest.i64Toi64Callback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
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
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_i64_c08776c9ce71e81d(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnI8(callback: io.prebindgen.covertest.i8Toi8Callback, onError: io.prebindgen.covertest.JniErrorHandler<Byte> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Byte = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Byte): Byte {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Byte = 0.toByte()
        var callbackReturned: Byte = 0.toByte()
        try {
            callbackArgument0_0 = raw0 as Byte
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_i8_753f4b94dff729a6(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnI16(callback: io.prebindgen.covertest.i16Toi16Callback, onError: io.prebindgen.covertest.JniErrorHandler<Short> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Short = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Short): Short {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Short = 0.toShort()
        var callbackReturned: Short = 0.toShort()
        try {
            callbackArgument0_0 = raw0 as Short
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_i16_9b06d1210ff8b2b7(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnF32(callback: io.prebindgen.covertest.f32Tof32Callback, onError: io.prebindgen.covertest.JniErrorHandler<Float> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Float = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Float): Float {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Float = 0.0f
        var callbackReturned: Float = 0.0f
        try {
            callbackArgument0_0 = raw0 as Float
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_f32_1d9d4691d73646c5(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnU128(callback: io.prebindgen.covertest.u128Tou128Callback, onError: io.prebindgen.covertest.JniErrorHandler<java.math.BigInteger> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.math.BigInteger = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): ByteArray? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: java.math.BigInteger
        lateinit var callbackReturned: java.math.BigInteger
        try {
            callbackArgument0_0 = java.math.BigInteger(1, raw0 as ByteArray)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1.toByteArray()
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_u128_54bd6e65c7018d0b(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    java.math.BigInteger(1, nativeResult as ByteArray)
    } finally { returnCapture.release() }
}

public fun callbackReturnZero(callback: io.prebindgen.covertest.VoidToi64Callback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: Long = 0L
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_zero_d965a5274955145f(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnBool(callback: io.prebindgen.covertest.boolToboolCallback, onError: io.prebindgen.covertest.JniErrorHandler<Boolean> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Boolean): Boolean {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Boolean = false
        var callbackReturned: Boolean = false
        try {
            callbackArgument0_0 = raw0 as Boolean
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_bool_468d68aae8a03d87(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnChar(callback: io.prebindgen.covertest.charTocharCallback, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
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
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_char_ac2c7988ad8d876f(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnFloat(callback: io.prebindgen.covertest.f64Tof64Callback, onError: io.prebindgen.covertest.JniErrorHandler<Double> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Double = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Double): Double {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Double = 0.0
        var callbackReturned: Double = 0.0
        try {
            callbackArgument0_0 = raw0 as Double
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_float_e8e49d9c6e5ee55e(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnUnsigned(callback: io.prebindgen.covertest.u64Tou64Callback, onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: ULong = 0UL
        var callbackReturned: ULong = 0UL
        try {
            callbackArgument0_0 = (raw0 as Long).toULong()
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1.toLong()
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_unsigned_fce893a55ce5733f(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun callbackReturnWide(callback: io.prebindgen.covertest.i128Toi128Callback, onError: io.prebindgen.covertest.JniErrorHandler<java.math.BigInteger> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.math.BigInteger = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): ByteArray? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: java.math.BigInteger
        lateinit var callbackReturned: java.math.BigInteger
        try {
            callbackArgument0_0 = java.math.BigInteger(raw0 as ByteArray)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1.toByteArray()
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_wide_3b195257768b36eb(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    java.math.BigInteger(nativeResult as ByteArray)
    } finally { returnCapture.release() }
}

public fun callbackReturnText(callback: io.prebindgen.covertest.StringToStringCallback, onError: io.prebindgen.covertest.JniErrorHandler<String> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: String?): String? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: String
        lateinit var callbackReturned: String
        try {
            callbackArgument0_0 = raw0 as String
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_text_b4839b1b734afa0b(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult as String
    } finally { returnCapture.release() }
}

public fun callbackReturnBytes(callback: io.prebindgen.covertest.u8VecTou8VecCallback, onError: io.prebindgen.covertest.JniErrorHandler<ByteArray> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ByteArray = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): ByteArray? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: ByteArray
        lateinit var callbackReturned: ByteArray
        try {
            callbackArgument0_0 = raw0 as ByteArray
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_bytes_8891244c4fbb186c(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult as ByteArray
    } finally { returnCapture.release() }
}

public fun callbackReturnOptional(callback: io.prebindgen.covertest.Optionali64ToOptionali64Callback, present: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Long?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int, raw1: Long): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        var callbackArgument0_0: Long? = null
        var callbackReturned: Long? = null
        try {
            if (raw0 == 1) {
            }
            callbackArgument0_0 = if (raw0 == 1) raw1 as Long else null
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w3 = callbackReturned
  run { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>((if (callback_return_w3 != null) 1 else 0), if (callback_return_w3 != null) requireNotNull(callback_return_w3) else 0L)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { present } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_optional_b061087be5b62673(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) returnCapture.s1 else null
    } finally { returnCapture.release() }
}

public fun callbackReturnVector(callback: io.prebindgen.covertest.i64VecToi64VecCallback, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): Array<Any?>? {
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
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
 val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.let { callback_return_w1_snapshot -> kotlin.Array<kotlin.Any?>(callback_return_w1_snapshot.size) { callback_return_w1_index -> val callback_return_w5 = callback_return_w1_snapshot[callback_return_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { callback_return_w5 }) } } else null
 run { val callbackReturnWire = callback_return_w1_elements
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_vector_f78658284aa91f49(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

public fun callbackReturnRecord(callback: io.prebindgen.covertest.callbackreturns.CallbackReturnRecordToCallbackReturnRecordCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long, raw1: String?): Array<Any?> {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnRecord
        lateinit var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackReturnRecord
        try {
            callbackArgument0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnRecord(raw0 as Long, raw1 as String)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = kotlin.arrayOf<kotlin.Any?>(callback_return_w1.number, callback_return_w1.label)
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_record_b21187a8bb28b01d(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnEnum(callback: io.prebindgen.covertest.callbackreturns.CallbackReturnChoiceToCallbackReturnChoiceCallback, onError: io.prebindgen.covertest.JniErrorHandler<Boolean> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Int): Int {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnChoice
        lateinit var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackReturnChoice
        try {
            callbackArgument0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnChoice.fromWire(raw0 as Int)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1.wireValue
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_enum_01ed3ed2bab330bf(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnResourceNew(`value`: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbackreturns.CallbackReturnResource> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.callbackreturns.CallbackReturnResource = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_resource_new_9e73b822affaedb3(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        try {
            output0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun callbackReturnResourceLive(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_resource_live_b530072d9d196364(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}

public fun callbackReturnOwned(callback: io.prebindgen.covertest.VoidToCallbackReturnResourceCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Long {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackReturnResource
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try {  run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { callback_return_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { val callbackReturnWire = __pb_handle0?.raw ?: 0L
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
__pb_handle0?.let { __pb_consume(it) }
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue0 -> returnedHandles.add(returnedValue0); returnedOwners.add(returnedValue0) }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_owned_5cfe5c76741495b8(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnTake(callback: io.prebindgen.covertest.callbackreturns.CallbackReturnResourceToCallbackReturnResourceCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Long): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackOutput0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        lateinit var callbackArgument0_0: io.prebindgen.covertest.callbackreturns.CallbackReturnResource
        lateinit var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackReturnResource
var callbackReturnReady = false
        try { var callbackFailure: Throwable? = null
        var callbackReady = false
        try {
            try {
            callbackOutput0_0 = io.prebindgen.covertest.callbackreturns.CallbackReturnResource(raw0 as Long)
            callbackArgument0_0 = requireNotNull(callbackOutput0_0)
        } catch (failure: Throwable) {
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
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
            try { val owner = callbackOutput0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnResource_289c5b28e5450a79(raw0 as Long) } catch (cleanup: Throwable) { if (callbackCleanupFailure == null) callbackCleanupFailure = cleanup else if (callbackCleanupFailure !== cleanup) callbackCleanupFailure!!.addSuppressed(cleanup) }
                callbackCleanupFailure?.let { cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }
            }
        } } catch (failure: Throwable) { if (callbackReturnReady) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callbackReturned as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue0 -> returnedHandles.add(returnedValue0); returnedOwners.add(returnedValue0) }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() } }; throw failure }
return kotlin.run { val callback_return_w1 = callbackReturned
 try {  run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { callback_return_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { val callbackReturnWire = __pb_handle0?.raw ?: 0L
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
__pb_handle0?.let { __pb_consume(it) }
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue0 -> returnedHandles.add(returnedValue0); returnedOwners.add(returnedValue0) }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_take_3ffb2a58590a9f8f(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnOwnedOptional(callback: io.prebindgen.covertest.VoidToOptionalCallbackReturnResourceCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Long {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: io.prebindgen.covertest.callbackreturns.CallbackReturnResource? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
 try {  run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { if (callback_return_w3 != null) (if (true) requireNotNull(callback_return_w3) else null) else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { val callbackReturnWire = __pb_handle0?.raw ?: 0L
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
__pb_handle0?.let { __pb_consume(it) }
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w3 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue1 -> returnedHandles.add(returnedValue1); returnedOwners.add(returnedValue1) }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_owned_optional_3f9c79bb9e2a1c84(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) returnCapture.s1 else null
    } finally { returnCapture.release() }
}

public fun callbackReturnOwnedVector(callback: io.prebindgen.covertest.VoidToCallbackReturnResourceVecCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackReturnResource>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.let { callback_return_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w1_snapshot.size) { callback_return_w1_index -> val callback_return_w5 = callback_return_w1_snapshot[callback_return_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { callback_return_w5 }) }, intArrayOf(2), 1, null) } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_elements?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = callback_return_w1_elements?.encode()
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_elements?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? Iterable<*>)?.forEach { returnedValue0 -> (returnedValue0 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue1 -> returnedHandles.add(returnedValue1); returnedOwners.add(returnedValue1) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_owned_vector_c955ff3794caa0f7(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnSpawn(callback: io.prebindgen.covertest.i64Toi64Callback, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbackreturns.CallbackReturnJob> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.callbackreturns.CallbackReturnJob = run {
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
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_spawn_7212e8a490c09a14(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.callbackreturns.CallbackReturnJob? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnJob_c59afe63a0d1957d(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.callbackreturns.CallbackReturnJob? = null
        try {
            output0 = io.prebindgen.covertest.callbackreturns.CallbackReturnJob(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnJob_c59afe63a0d1957d(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun callbackReturnJoin(job: io.prebindgen.covertest.callbackreturns.CallbackReturnJob, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { job } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_join_1a06270dc90abd5f(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; __pb_handle0?.let { __pb_consume(it) } } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; __pb_handle0?.let { __pb_consume(it) } } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnHandlerNew(callback: io.prebindgen.covertest.i64Toi64Callback, fail: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.callbackreturns.CallbackReturnHandler> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.StringMessageHandler<io.prebindgen.covertest.callbackreturns.CallbackReturnHandler>): io.prebindgen.covertest.callbackreturns.CallbackReturnHandler = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorString1_6b62d91b16305083.acquire()
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
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { fail } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_handler_new_773c695edbbbf41e(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.callbackreturns.CallbackReturnHandler? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnHandler_8a6b761b8bbde2f7(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0 as String) }
            var output0: io.prebindgen.covertest.callbackreturns.CallbackReturnHandler? = null
        try {
            output0 = io.prebindgen.covertest.callbackreturns.CallbackReturnHandler(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_CallbackReturnHandler_8a6b761b8bbde2f7(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun callbackReturnHandlerCall(handler: io.prebindgen.covertest.callbackreturns.CallbackReturnHandler, `value`: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { handler } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_handler_call_ea4b685d6ef94d77(nativeArgument0, nativeArgument1, nativeArgument2) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnCatch(callback: io.prebindgen.covertest.i64Toi64Callback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
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
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_catch_fb340e4965e59b90(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnHandlerMany(handlers: kotlin.collections.List<io.prebindgen.covertest.i64Toi64Callback>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) handlers?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w6 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { object {
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
        callbackReturned = p0_w6.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} }) } } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_handler_many_8518ace5a4249c8b(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnCharCatch(callback: io.prebindgen.covertest.VoidTocharCallback, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
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
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_char_catch_2304a076cc2d2d5d(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnOwnedVectorCatch(callback: io.prebindgen.covertest.VoidToCallbackReturnResourceVecCallback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): Array<Any?>? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: kotlin.collections.List<io.prebindgen.covertest.callbackreturns.CallbackReturnResource>
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
 try { val callback_return_w1_elements = if (true) callback_return_w1?.toList()?.let { callback_return_w1_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(callback_return_w1_snapshot.size) { callback_return_w1_index -> val callback_return_w5 = callback_return_w1_snapshot[callback_return_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { callback_return_w5 }) }, intArrayOf(2), 1, null) } else null
 run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { callback_return_w1_elements?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val callbackReturnWire = callback_return_w1_elements?.encode()
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }
callback_return_w1_elements?.consume(__pb_consume)
callbackReturnWire } } } catch (failure: Throwable) { val returnedOwners = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); val returnedHandles = kotlin.collections.ArrayList<io.prebindgen.covertest.NativeHandle>(); (callback_return_w1 as? Iterable<*>)?.forEach { returnedValue0 -> (returnedValue0 as? io.prebindgen.covertest.NativeHandle)?.let { returnedValue1 -> returnedHandles.add(returnedValue1); returnedOwners.add(returnedValue1) }
 }

val released = kotlin.collections.HashSet<Long>(); for (owner in returnedOwners) { if (owner.isClosed) continue; try { if (released.add(owner.address)) owner.close() else owner.markConsumed() } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) } }; for (handle in returnedHandles) { if (handle.address in released) handle.markConsumed() }; throw failure } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_owned_vector_catch_712110277489aed4(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun callbackReturnUnit(callback: io.prebindgen.covertest.i64Callback, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
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
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_unit_dd4499fffb863427(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun callbackReturnHandlerValue(handler: io.prebindgen.covertest.i64Toi64Callback, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
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
        callbackReturned = handler.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_callback_return_handler_value_a23aec1750dc3226(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}
