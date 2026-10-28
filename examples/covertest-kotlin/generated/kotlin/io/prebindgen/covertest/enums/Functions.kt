@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.enums

public fun <R> `enumBoundaryProbe`(p0_w1: ByteArray?, p1_w1: Boolean, p1_w5: io.prebindgen.covertest.enums.Code?, p2_w1: String?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, io.prebindgen.covertest.enums.Code?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_w1) run { val value = if (p1_w5 != null) run { val value = requireNotNull(p1_w5).wireValue; require(value != -3) { "invalid option payload" }; value } else -3; require(value != -4) { "invalid option payload" }; value } else -4 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p2_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_enum_boundary_probe_602c56993c15d467(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -4) {
            if (result[1] != -3) {
            }
            }
            build(result[1] != -4, if (result[1] != -4) if (result[1] != -3) io.prebindgen.covertest.enums.Code.fromWire(result[1] as Int) else null else null)
        } catch (failure: Throwable) {
            if (result[1] != -4) {
            if (result[1] != -3) {
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `enumPayloadNew`(p0_w1: Double, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.EnumPayload> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.EnumPayload = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_enum_payload_new_9a992a3f2e4e9d40(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.enums.EnumPayload? = null
        try {
            output0 = io.prebindgen.covertest.enums.EnumPayload(result[1] as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_PayloadHandle_af81cbfac6e10b9d(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `payloadPriority`(p0_w1: io.prebindgen.covertest.enums.EnumPayload?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.Priority> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.Priority = run {
    val result = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_payload_priority_1865922a1dc262d4(nativeArgument0) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { io.prebindgen.covertest.enums.Priority.fromWire(result[1] as Int) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `priorityWeight`(p0_w1: io.prebindgen.covertest.enums.Priority, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val result = try { run { val nativeArgument0 = try { p0_w1.wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_priority_weight_bcf2d93ac3538d33(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Int }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `priorityOr`(p0_w3: io.prebindgen.covertest.enums.Priority?, p1_w1: io.prebindgen.covertest.enums.Priority, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.Priority> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.Priority = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) run { val value = requireNotNull(p0_w3).wireValue; require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1.wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_priority_or_8b7e9d67b98b36fd(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { io.prebindgen.covertest.enums.Priority.fromWire(result[1] as Int) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `sparseEcho`(p0_w1: io.prebindgen.covertest.enums.Code, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.Code> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.Code = run {
    val result = try { run { val nativeArgument0 = try { p0_w1.wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sparse_echo_ff112271a9f96bd5(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { io.prebindgen.covertest.enums.Code.fromWire(result[1] as Int) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `sparseOptional`(p0_w3: io.prebindgen.covertest.enums.Code?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.Code?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.Code? = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) run { val value = requireNotNull(p0_w3).wireValue; require(value != -3) { "invalid option payload" }; value } else -3 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sparse_optional_d96a4fb0a1baa04f(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -3) {
            }
            if (result[1] != -3) io.prebindgen.covertest.enums.Code.fromWire(result[1] as Int) else null
        } catch (failure: Throwable) {
            if (result[1] != -3) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `sparseNested`(p0_w1: Boolean, p0_w5: io.prebindgen.covertest.enums.Code?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, io.prebindgen.covertest.enums.Code?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w1) run { val value = if (p0_w5 != null) run { val value = requireNotNull(p0_w5).wireValue; require(value != -3) { "invalid option payload" }; value } else -3; require(value != -4) { "invalid option payload" }; value } else -4 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sparse_nested_80137c780e0dbc13(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -4) {
            if (result[1] != -3) {
            }
            }
            build(result[1] != -4, if (result[1] != -4) if (result[1] != -3) io.prebindgen.covertest.enums.Code.fromWire(result[1] as Int) else null else null)
        } catch (failure: Throwable) {
            if (result[1] != -4) {
            if (result[1] != -3) {
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `sparseRef`(p0_w1: io.prebindgen.covertest.enums.Code, onError: io.prebindgen.covertest.JniErrorHandler<Int> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int = run {
    val result = try { run { val nativeArgument0 = try { p0_w1.wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sparse_ref_d09b189574416726(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Int }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `sparseBorrowed`(p0_w1: io.prebindgen.covertest.enums.Code, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.Code> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.Code = run {
    val result = try { run { val nativeArgument0 = try { p0_w1.wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sparse_borrowed_2c5728a75c56fd13(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { io.prebindgen.covertest.enums.Code.fromWire(result[1] as Int) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `enumRecordEcho`(p0_w1: io.prebindgen.covertest.enums.EnumRecord, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.EnumRecord> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.EnumRecord = run {
    val result = try { run { val nativeArgument0 = try { p0_w1.`priority`.wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w1.`code` != null) run { val value = requireNotNull(p0_w1.`code`).wireValue; require(value != -3) { "invalid option payload" }; value } else -3 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_enum_record_echo_1fa3f892fbef60f0(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[2] != -3) {
            }
            io.prebindgen.covertest.enums.EnumRecord(io.prebindgen.covertest.enums.Priority.fromWire(result[1] as Int), if (result[2] != -3) io.prebindgen.covertest.enums.Code.fromWire(result[2] as Int) else null)
        } catch (failure: Throwable) {
            if (result[2] != -3) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `enumNamesEcho`(p0_w1: io.prebindgen.covertest.enums.EnumNames, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.EnumNames> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.enums.EnumNames = run {
    val result = try { run { val nativeArgument0 = try { p0_w1._wireValue } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_enum_names_echo_45deb1696c7faead(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { io.prebindgen.covertest.enums.EnumNames._fromWire(result[1] as Int) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

