@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.enums

public fun <R> `enumBoundaryProbe`(`bytes`: ByteArray?, `valuePresent`: Boolean, `valueSome`: io.prebindgen.covertest.enums.Code?, `text`: String?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, io.prebindgen.covertest.enums.Code?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `bytes` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (`valuePresent`) run { val value = if (`valueSome` != null) run { val value = requireNotNull(`valueSome`).wireValue; require(value != -3) { "invalid option payload" }; value } else -3; require(value != -4) { "invalid option payload" }; value } else -4 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { `text` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_enum_boundary_probe_602c56993c15d467(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != -4) {
            if (nativeResult != -3) {
            }
            }

build(nativeResult != -4, if (nativeResult != -4) if (nativeResult != -3) io.prebindgen.covertest.enums.Code.fromWire(nativeResult) else null else null)
    } finally { returnCapture.release() }
}

public fun `enumPayloadNew`(`value`: Double, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.EnumPayload> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.EnumPayload = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `value` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_enum_payload_new_9a992a3f2e4e9d40(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.enums.EnumPayload? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_PayloadHandle_af81cbfac6e10b9d(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.enums.EnumPayload? = null
        try {
            output0 = io.prebindgen.covertest.enums.EnumPayload(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_PayloadHandle_af81cbfac6e10b9d(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `payloadPriority`(`p`: io.prebindgen.covertest.enums.EnumPayload?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.Priority> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.Priority = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(`p`) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { `p`?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_payload_priority_1865922a1dc262d4(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    io.prebindgen.covertest.enums.Priority.fromWire(nativeResult)
    } finally { returnCapture.release() }
}

public fun `priorityWeight`(`p`: io.prebindgen.covertest.enums.Priority, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `p`.wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_priority_weight_bcf2d93ac3538d33(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `priorityOr`(`p`: io.prebindgen.covertest.enums.Priority?, `fallback`: io.prebindgen.covertest.enums.Priority, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.Priority> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.Priority = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (`p` != null) run { val value = requireNotNull(`p`).wireValue; require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { `fallback`.wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_priority_or_8b7e9d67b98b36fd(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    io.prebindgen.covertest.enums.Priority.fromWire(nativeResult)
    } finally { returnCapture.release() }
}

public fun `sparseEcho`(`value`: io.prebindgen.covertest.enums.Code, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.Code> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.Code = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `value`.wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sparse_echo_ff112271a9f96bd5(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    io.prebindgen.covertest.enums.Code.fromWire(nativeResult)
    } finally { returnCapture.release() }
}

public fun `sparseOptional`(`value_c2da6115cd444ccd`: io.prebindgen.covertest.enums.Code?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.Code?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.Code? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (`value_c2da6115cd444ccd` != null) run { val value = requireNotNull(`value_c2da6115cd444ccd`).wireValue; require(value != -3) { "invalid option payload" }; value } else -3 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sparse_optional_d96a4fb0a1baa04f(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != -3) {
            }

if (nativeResult != -3) io.prebindgen.covertest.enums.Code.fromWire(nativeResult) else null
    } finally { returnCapture.release() }
}

public fun <R> `sparseNested`(`valuePresent`: Boolean, `valueSome`: io.prebindgen.covertest.enums.Code?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, io.prebindgen.covertest.enums.Code?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (`valuePresent`) run { val value = if (`valueSome` != null) run { val value = requireNotNull(`valueSome`).wireValue; require(value != -3) { "invalid option payload" }; value } else -3; require(value != -4) { "invalid option payload" }; value } else -4 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sparse_nested_80137c780e0dbc13(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != -4) {
            if (nativeResult != -3) {
            }
            }

build(nativeResult != -4, if (nativeResult != -4) if (nativeResult != -3) io.prebindgen.covertest.enums.Code.fromWire(nativeResult) else null else null)
    } finally { returnCapture.release() }
}

public fun `sparseRef`(`value`: io.prebindgen.covertest.enums.Code, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `value`.wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sparse_ref_d09b189574416726(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `sparseBorrowed`(`value`: io.prebindgen.covertest.enums.Code, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.Code> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.Code = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `value`.wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sparse_borrowed_2c5728a75c56fd13(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    io.prebindgen.covertest.enums.Code.fromWire(nativeResult)
    } finally { returnCapture.release() }
}

public fun `enumRecordEcho`(`value_c2da6115cd444ccd`: io.prebindgen.covertest.enums.EnumRecord, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.EnumRecord> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.EnumRecord = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Int2_ErrorEmpty_1d2a9157b2d89875.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `value_c2da6115cd444ccd`.`priority`.wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (`value_c2da6115cd444ccd`.`code` != null) run { val value = requireNotNull(`value_c2da6115cd444ccd`.`code`).wireValue; require(value != -3) { "invalid option payload" }; value } else -3 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_enum_record_echo_1fa3f892fbef60f0(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s1 != -3) {
            }

io.prebindgen.covertest.enums.EnumRecord(io.prebindgen.covertest.enums.Priority.fromWire(returnCapture.s0), if (returnCapture.s1 != -3) io.prebindgen.covertest.enums.Code.fromWire(returnCapture.s1) else null)
    } finally { returnCapture.release() }
}

public fun `enumNamesEcho`(`entry`: io.prebindgen.covertest.enums.EnumNames, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.EnumNames> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.EnumNames = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { `entry`._wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_enum_names_echo_45deb1696c7faead(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    io.prebindgen.covertest.enums.EnumNames._fromWire(nativeResult)
    } finally { returnCapture.release() }
}
