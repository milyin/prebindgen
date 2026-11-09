@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.nullability

public fun nullabilityHandleNew(`value`: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.nullability.NullabilityHandle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.nullability.NullabilityHandle = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_handle_new_52d93c3d58bc9ddd(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.nullability.NullabilityHandle? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_NullabilityHandle_aefadfc6a0601b82(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.nullability.NullabilityHandle? = null
        try {
            output0 = io.prebindgen.covertest.nullability.NullabilityHandle(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_NullabilityHandle_aefadfc6a0601b82(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun nullabilityHandleRead(`value`: io.prebindgen.covertest.nullability.NullabilityHandle, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_handle_read_87c3a62f7392e4f0(nativeArgument0, nativeArgument1) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nullabilityHandleOptional(`value`: io.prebindgen.covertest.nullability.NullabilityHandle?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { if (`value` != null) (if (true) requireNotNull(`value`) else null) else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { run { val nativeArgument0 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_handle_optional_0d97fddf5e04f9dc(nativeArgument0, nativeArgument1) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nullabilityText(`value`: String, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_text_371469abc18caa34(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nullabilityTextOptional(`value`: String?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { if (`value` != null) requireNotNull(requireNotNull(`value`)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_text_optional_7c4699f3d1e235f0(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nullabilityBytes(`value`: ByteArray, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_bytes_ed1ce1214d744c8d(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nullabilityBytesOptional(`value`: ByteArray?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { if (`value` != null) requireNotNull(requireNotNull(`value`)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_bytes_optional_48809d71f31114f1(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nullabilityWide(`value`: java.math.BigInteger, onError: io.prebindgen.covertest.JniErrorHandler<java.math.BigInteger> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.math.BigInteger = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value`.toByteArray() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_wide_9d858f42bb045bf8(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    java.math.BigInteger(nativeResult as ByteArray)
    } finally { returnCapture.release() }
}

public fun nullabilityWideOptional(`value`: java.math.BigInteger?, onError: io.prebindgen.covertest.JniErrorHandler<java.math.BigInteger?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): java.math.BigInteger? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_ByteArray2_ErrorEmpty_a2aad6159c8c4c14.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { (if (`value` != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (`value` != null) requireNotNull(`value`).toByteArray() else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_wide_optional_675c57dd8593ffa4(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) java.math.BigInteger(returnCapture.s1 as ByteArray) else null
    } finally { returnCapture.release() }
}

public fun nullabilityValueSum(valueVariant: Int, valueVariant0: io.prebindgen.covertest.nullability.NullabilityValue?, valueVariant1Text: String?, valueVariant1Bytes: ByteArray?, valueVariant1Wide: java.math.BigInteger?, valueVariant1Choice: io.prebindgen.covertest.nullability.NullabilityChoice?, valueVariant1Handle: io.prebindgen.covertest.nullability.NullabilityHandle?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { if (valueVariant == 1) requireNotNull(valueVariant1Handle) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { run { val nativeArgument0 = try { valueVariant } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valueVariant == 0) requireNotNull(valueVariant0).text else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (valueVariant == 0) requireNotNull(valueVariant0).bytes else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { if (valueVariant == 0) requireNotNull(valueVariant0).wide.toByteArray() else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { if (valueVariant == 0) requireNotNull(valueVariant0).choice.wireValue else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { if (valueVariant == 0) requireNotNull(valueVariant0).handle_value else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { valueVariant1Text } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument7 = try { valueVariant1Bytes } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument8 = try { if (valueVariant == 1) requireNotNull(valueVariant1Wide).toByteArray() else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument9 = try { if (valueVariant == 1) requireNotNull(valueVariant1Choice).wireValue else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument10 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument11 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_value_sum_9ffd3f4f4ef95c92(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6, nativeArgument7, nativeArgument8, nativeArgument9, nativeArgument10, nativeArgument11) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nullabilityValueBulk(valueVariant: Int, valueVariant0: io.prebindgen.covertest.nullability.NullabilityValue?, valueVariant1Text: String?, valueVariant1Bytes: ByteArray?, valueVariant1Wide: java.math.BigInteger?, valueVariant1Choice: io.prebindgen.covertest.nullability.NullabilityChoice?, valueVariant1Handle: io.prebindgen.covertest.nullability.NullabilityHandle?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer_state = try { if (true) run { val p0_w1_writer = io.prebindgen.covertest.BulkInputs.Input_i32_Range_6217a02e84ef490c()
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r6217a02e84ef490c.offsets[0], io.prebindgen.covertest.InputValidation.value { valueVariant })
p0_w1_writer.text((0) + io.prebindgen.covertest.BulkLayouts.r6217a02e84ef490c.offsets[1], io.prebindgen.covertest.InputValidation.value { if (valueVariant == 0) requireNotNull(valueVariant0).text else null })
p0_w1_writer.bytes((0) + io.prebindgen.covertest.BulkLayouts.r6217a02e84ef490c.offsets[2], io.prebindgen.covertest.InputValidation.value { if (valueVariant == 0) requireNotNull(valueVariant0).bytes else null })
p0_w1_writer.wide((0) + io.prebindgen.covertest.BulkLayouts.r6217a02e84ef490c.offsets[3], io.prebindgen.covertest.InputValidation.value { if (valueVariant == 0) requireNotNull(valueVariant0).wide.toByteArray() else null }, true)
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r6217a02e84ef490c.offsets[4], io.prebindgen.covertest.InputValidation.value { if (valueVariant == 0) requireNotNull(valueVariant0).choice.wireValue else 0 })
p0_w1_writer.long((0) + io.prebindgen.covertest.BulkLayouts.r6217a02e84ef490c.offsets[5], io.prebindgen.covertest.InputValidation.value { if (valueVariant == 0) requireNotNull(valueVariant0).handle_value else 0L })
p0_w1_writer.text((0) + io.prebindgen.covertest.BulkLayouts.r6217a02e84ef490c.offsets[6], io.prebindgen.covertest.InputValidation.value { valueVariant1Text })
p0_w1_writer.bytes((0) + io.prebindgen.covertest.BulkLayouts.r6217a02e84ef490c.offsets[7], io.prebindgen.covertest.InputValidation.value { valueVariant1Bytes })
p0_w1_writer.wide((0) + io.prebindgen.covertest.BulkLayouts.r6217a02e84ef490c.offsets[8], io.prebindgen.covertest.InputValidation.value { if (valueVariant == 1) requireNotNull(valueVariant1Wide).toByteArray() else null }, true)
p0_w1_writer.int((0) + io.prebindgen.covertest.BulkLayouts.r6217a02e84ef490c.offsets[9], io.prebindgen.covertest.InputValidation.value { if (valueVariant == 1) requireNotNull(valueVariant1Choice).wireValue else 0 })
p0_w1_writer.handle((0) + io.prebindgen.covertest.BulkLayouts.r6217a02e84ef490c.offsets[10], io.prebindgen.covertest.InputValidation.value { if (valueVariant == 1) valueVariant1Handle else null }, false, 0)

p0_w1_writer } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { p0_w1_writer_state?.handles.orEmpty() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), false) { run { val nativeArgument0 = try { p0_w1_writer_state?.encode() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer_state?.length ?: 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_value_bulk_cde4e2ae3684af79(nativeArgument0, nativeArgument1, nativeArgument2) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nullabilityValueOptional(valuePresent: Boolean, valueText: String?, valueBytes: ByteArray?, valueWide: java.math.BigInteger?, valueChoice: io.prebindgen.covertest.nullability.NullabilityChoice?, valueHandle: io.prebindgen.covertest.nullability.NullabilityHandle?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: io.prebindgen.covertest.NativeHandle? = try { if (valuePresent) (if (true) requireNotNull(valueHandle) { "null handle" } else null) else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(__pb_handle0) { run { val nativeArgument0 = try { if (valuePresent) requireNotNull(valueText) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valuePresent) valueBytes else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (valuePresent) if (true) requireNotNull(valueWide).toByteArray() else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { if (valuePresent) if (true) requireNotNull(valueChoice).wireValue else 0 else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { __pb_handle0?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_value_optional_9875ca4fcdb053b1(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nullabilityValues(values: kotlin.collections.List<io.prebindgen.covertest.nullability.NullabilityValue>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w5.text }, io.prebindgen.covertest.InputValidation.value { p0_w5.bytes }, io.prebindgen.covertest.InputValidation.value { p0_w5.wide.toByteArray() }, io.prebindgen.covertest.InputValidation.value { p0_w5.choice.wireValue }, io.prebindgen.covertest.InputValidation.value { p0_w5.handle_value }) } } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_values_b3abc03a5e5fdbda(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nullabilityValuesOptional(values: kotlin.collections.List<io.prebindgen.covertest.nullability.NullabilityValue>?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (values != null) (if (true) requireNotNull(values).toList()?.let { p0_w3_snapshot -> kotlin.Array<kotlin.Any?>(p0_w3_snapshot.size) { p0_w3_index -> val p0_w7 = p0_w3_snapshot[p0_w3_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w7.text }, io.prebindgen.covertest.InputValidation.value { p0_w7.bytes }, io.prebindgen.covertest.InputValidation.value { p0_w7.wide.toByteArray() }, io.prebindgen.covertest.InputValidation.value { p0_w7.choice.wireValue }, io.prebindgen.covertest.InputValidation.value { p0_w7.handle_value }) } } else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { if (values != null) requireNotNull(p0_w3_elements) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_values_optional_bd4a7ceb471c6c54(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nullabilityHandlesOptional(values: kotlin.collections.List<io.prebindgen.covertest.nullability.NullabilityHandle>?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = try { if (values != null) (if (true) requireNotNull(values).toList()?.let { p0_w3_snapshot -> io.prebindgen.covertest.InputRows(kotlin.Array<kotlin.Any?>(p0_w3_snapshot.size) { p0_w3_index -> val p0_w7 = p0_w3_snapshot[p0_w3_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w7 }) }, intArrayOf(2), 1, null) } else null) else (null) } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val __pb_handle0: kotlin.collections.List<io.prebindgen.covertest.NativeHandle> = try { if (values != null) (p0_w3_elements?.handles.orEmpty()) else kotlin.collections.emptyList() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withInputHandleLocks((__pb_handle0), true) { val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { if (values != null) requireNotNull(p0_w3_elements?.encode()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_handles_optional_95fc2bf9cec835af(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (values != null) { p0_w3_elements?.consume(__pb_consume) } } else { var remaining = -1L - ownership[0]; val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { handle.markConsumed(); remaining-- } }; if (values != null) { p0_w3_elements?.consume(__pb_consume) } } } } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nullabilityTextCallback(callback: io.prebindgen.covertest.VoidToStringCallback, onError: io.prebindgen.covertest.JniErrorHandler<String> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): String? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: String
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
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_text_callback_42d80e83e23401a2(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult as String
    } finally { returnCapture.release() }
}

public fun nullabilityOptionalTextCallback(callback: io.prebindgen.covertest.VoidToOptionalStringCallback, onError: io.prebindgen.covertest.JniErrorHandler<String?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): String? {
        var callbackCleanupFailure: Throwable? = null
        var callbackReturned: String? = null
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = callback.run()
return kotlin.run { val callback_return_w3 = callbackReturned
  run { val callbackReturnWire = if (callback_return_w3 != null) requireNotNull(requireNotNull(callback_return_w3)) { "invalid option payload" } else null
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_optional_text_callback_426adf31f0aa74b4(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != null) {
            }

if (nativeResult != null) nativeResult as String else null
    } finally { returnCapture.release() }
}

public fun nullabilityCallbackChoose(valueVariant: Int, valueVariant0: io.prebindgen.covertest.nullability.NullabilityValue?, valueVariant1Callback: io.prebindgen.covertest.VoidToStringCallback?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { valueVariant } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valueVariant == 0) requireNotNull(valueVariant0).text else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (valueVariant == 0) requireNotNull(valueVariant0).bytes else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { if (valueVariant == 0) requireNotNull(valueVariant0).wide.toByteArray() else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { if (valueVariant == 0) requireNotNull(valueVariant0).choice.wireValue else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { if (valueVariant == 0) requireNotNull(valueVariant0).handle_value else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { if (valueVariant == 1) if (valueVariant1Callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(): String? {
        var callbackCleanupFailure: Throwable? = null
        lateinit var callbackReturned: String
        try {
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = valueVariant1Callback.run()
return kotlin.run { val callback_return_w1 = callbackReturned
  run { val callbackReturnWire = callback_return_w1
val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }

callbackReturnWire } }
    }
} else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument7 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nullability_callback_choose_6b3e84beed47a860(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6, nativeArgument7) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}
