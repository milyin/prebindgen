@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.results

public fun `cResultErrorDrop`(onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.CDropErrorHandler<Unit>): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_c_result_error_drop_8c05c7fda32d0f81(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {         var outputg0_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

var cleanupFailure: Throwable? = null
var callbackFailed = false
try {
            outputg0_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.d0)

onError.run(requireNotNull(outputg0_g0_g0_0))
} catch (failure: Throwable) { callbackFailed = true; cleanupFailure = failure; throw failure } finally {
            try { val owner = outputg0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.d0) } catch (cleanup: Throwable) { if (cleanupFailure == null) cleanupFailure = cleanup else if (cleanupFailure !== cleanup) cleanupFailure!!.addSuppressed(cleanup) }

if (!callbackFailed) cleanupFailure?.let { throw it }
} }
    Unit
    } finally { returnCapture.release() }
}

public fun <R> `cResultLateNiche`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.CResultErrorHandler<R>, build: (Long, Int) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Int2_ErrorLong_String2_d7386b6fbdbb3a71.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_c_result_late_niche_9d6cee99c69463b1(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0, returnCapture.d1 as String) }
    build(returnCapture.s0, returnCapture.s1)
    } finally { returnCapture.release() }
}

public fun `cResultResourceError`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.CResourceErrorHandler<Unit>): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong_String2_a3fb4f88f2014cb0.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_c_result_resource_error_f29a434ac3a17580(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {         var outputg0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

var cleanupFailure: Throwable? = null
var callbackFailed = false
try {
            outputg0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.d0)

onError.run(requireNotNull(outputg0_g0_0), returnCapture.d1 as String)
} catch (failure: Throwable) { callbackFailed = true; cleanupFailure = failure; throw failure } finally {
            try { val owner = outputg0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.d0) } catch (cleanup: Throwable) { if (cleanupFailure == null) cleanupFailure = cleanup else if (cleanupFailure !== cleanup) cleanupFailure!!.addSuppressed(cleanup) }

if (!callbackFailed) cleanupFailure?.let { throw it }
} }
    Unit
    } finally { returnCapture.release() }
}

public fun `cResultRichError`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.CRichErrorHandler<Unit>): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorInt_String2_b786195c4ed75c25.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_c_result_rich_error_e97eeeaedd2fcd23(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
var cleanupFailure: Throwable? = null
var callbackFailed = false
try {
            if (returnCapture.d1 != null) {
            }

onError.run(io.prebindgen.covertest.enums.Code.fromWire(returnCapture.d0), if (returnCapture.d1 != null) returnCapture.d1 as String else null)
} catch (failure: Throwable) { callbackFailed = true; cleanupFailure = failure; throw failure } finally {
            if (returnCapture.d1 != null) {
            }

if (!callbackFailed) cleanupFailure?.let { throw it }
} }
    Unit
    } finally { returnCapture.release() }
}

public fun `resultScalar`(p0_w1: Long, onBindingError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<Long>): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_result_scalar_717ad7e5f39fb113(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    nativeResult
    } finally { returnCapture.release() }
}

public fun `resultUnitError`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ErrorHandler_Unit_typed_f92819398b43e125<Long>): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_result_unit_error_6007a70d185fe955(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run() }
    nativeResult
    } finally { returnCapture.release() }
}

public fun `resultUnit`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<Unit>): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_result_unit_fc0177d655b15009(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    Unit
    } finally { returnCapture.release() }
}

public fun `resultOptional`(p0_w1: Int, onBindingError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.Code?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<io.prebindgen.covertest.enums.Code?>): io.prebindgen.covertest.enums.Code? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_result_optional_01608adeaf654d25(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);              if (nativeResult != -3) {
            }
 }
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
            try {
            if (nativeResult != -3) {
            }
            if (nativeResult != -3) io.prebindgen.covertest.enums.Code.fromWire(nativeResult) else null
        } catch (failure: Throwable) {
            if (nativeResult != -3) {
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `resultText`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<String> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<String>): String = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_result_text_d4e50fb117070bd3(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    nativeResult as String
    } finally { returnCapture.release() }
}

public fun `resultRecord`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.EnumRecord> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<io.prebindgen.covertest.enums.EnumRecord>): io.prebindgen.covertest.enums.EnumRecord = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Int2_ErrorLong1_d482e1a7516fbcfa.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_result_record_1be5deeed131a35d(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);              if (returnCapture.s1 != -3) {
            }
 }
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
            try {
            if (returnCapture.s1 != -3) {
            }
            io.prebindgen.covertest.enums.EnumRecord(io.prebindgen.covertest.enums.Priority.fromWire(returnCapture.s0), if (returnCapture.s1 != -3) io.prebindgen.covertest.enums.Code.fromWire(returnCapture.s1) else null)
        } catch (failure: Throwable) {
            if (returnCapture.s1 != -3) {
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `resultOwned`(p0_w1: io.prebindgen.covertest.resources.OwnedResource?, p1_w1: Int, p2_w1: Int, onBindingError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.OwnedResource> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<io.prebindgen.covertest.resources.OwnedResource>): io.prebindgen.covertest.resources.OwnedResource = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p2_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_result_owned_ed97ae30ee5fa2ec(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0]) { if (true) p0_w1!!.markConsumed() } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.resources.OwnedResource? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
            var output0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            output0 = io.prebindgen.covertest.resources.OwnedResource(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `resultOther`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.StringMessageHandler<Long>): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorString1_6b62d91b16305083.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_result_other_bf6ac73e7d98d05c(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0 as String) }
    nativeResult
    } finally { returnCapture.release() }
}

@kotlin.jvm.Throws(java.lang.IllegalStateException::class)
public fun `storageTryWithLabel`(p0_w1: String?, onBindingError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.Storage> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.StorageErrorMessageHandler<io.prebindgen.covertest.Storage> = io.prebindgen.covertest.StorageErrorMessageHandler { message -> throw java.lang.IllegalStateException(message) }): io.prebindgen.covertest.Storage = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorString1_6b62d91b16305083.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_storage_try_with_label_1db3fb1ae12e2bed(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.Storage? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_Storage_9651261943550368(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0 as String) }
            var output0: io.prebindgen.covertest.Storage? = null
        try {
            output0 = io.prebindgen.covertest.Storage(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_Storage_9651261943550368(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `resultErrorDrops`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_result_error_drops_37b19328e81c2636(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    (nativeResult).toULong()
    } finally { returnCapture.release() }
}
