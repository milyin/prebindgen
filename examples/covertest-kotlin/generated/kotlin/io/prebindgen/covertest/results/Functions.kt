@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.results

public fun `cResultErrorDrop`(onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.CDropErrorHandler<Unit>): Unit = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call143() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        true -> {         var outputg0_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

var cleanupFailure: Throwable? = null
var callbackFailed = false
try {
            outputg0_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(result[1] as Long)

onError.run(requireNotNull(outputg0_g0_g0_0))
} catch (failure: Throwable) { callbackFailed = true; cleanupFailure = failure; throw failure } finally {
            try { val owner = outputg0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[1] as Long) } catch (cleanup: Throwable) { if (cleanupFailure == null) cleanupFailure = cleanup else if (cleanupFailure !== cleanup) cleanupFailure!!.addSuppressed(cleanup) }

if (!callbackFailed) cleanupFailure?.let { throw it }
} }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `cResultLateNiche`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.CResultErrorHandler<R>, build: (Long, Int) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call144(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { build(result[1] as Long, result[2] as Int) }
        true -> { 
onError.run(result[1] as Long, result[2] as String) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `cResultResourceError`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.CResourceErrorHandler<Unit>): Unit = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call145(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        true -> {         var outputg0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

var cleanupFailure: Throwable? = null
var callbackFailed = false
try {
            outputg0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(result[1] as Long)

onError.run(requireNotNull(outputg0_g0_0), result[2] as String)
} catch (failure: Throwable) { callbackFailed = true; cleanupFailure = failure; throw failure } finally {
            try { val owner = outputg0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[1] as Long) } catch (cleanup: Throwable) { if (cleanupFailure == null) cleanupFailure = cleanup else if (cleanupFailure !== cleanup) cleanupFailure!!.addSuppressed(cleanup) }

if (!callbackFailed) cleanupFailure?.let { throw it }
} }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `cResultRichError`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.CRichErrorHandler<Unit>): Unit = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call146(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        true -> { 
var cleanupFailure: Throwable? = null
var callbackFailed = false
try {
            if (result[2] != null) {
            }

onError.run(io.prebindgen.covertest.enums.Code.fromWire(result[1] as Int), if (result[2] != null) result[2] as String else null)
} catch (failure: Throwable) { callbackFailed = true; cleanupFailure = failure; throw failure } finally {
            if (result[2] != null) {
            }

if (!callbackFailed) cleanupFailure?.let { throw it }
} }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `resultScalar`(p0_w1: Long, onBindingError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<Long>): Long = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call147(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        true -> { 
onError.run(result[1] as Long) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `resultUnit`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<Unit>): Unit = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call148(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        true -> { 
onError.run(result[1] as Long) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `resultOptional`(p0_w1: Int, onBindingError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.Code?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<io.prebindgen.covertest.enums.Code?>): io.prebindgen.covertest.enums.Code? = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call149(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
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
        true -> { 
onError.run(result[1] as Long) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `resultText`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<String> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<String>): String = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call150(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as String }
        true -> { 
onError.run(result[1] as Long) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `resultRecord`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.enums.EnumRecord> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<io.prebindgen.covertest.enums.EnumRecord>): io.prebindgen.covertest.enums.EnumRecord = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call151(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
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
        true -> { 
onError.run(result[1] as Long) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `resultOwned`(p0_w1: io.prebindgen.covertest.resources.OwnedResource?, p1_w1: Int, p2_w1: Int, onBindingError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.OwnedResource> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<io.prebindgen.covertest.resources.OwnedResource>): io.prebindgen.covertest.resources.OwnedResource = run {
    val result = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p2_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call152(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0]) { if (true) p0_w1!!.markConsumed() } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            output0 = io.prebindgen.covertest.resources.OwnedResource(result[1] as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        true -> { 
onError.run(result[1] as Long) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `resultOther`(p0_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.StringMessageHandler<Long>): Long = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call153(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        true -> { 
onError.run(result[1] as String) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

@kotlin.jvm.Throws(java.lang.IllegalStateException::class)
public fun `storageTryWithLabel`(p0_w1: String?, onBindingError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.Storage> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.StorageErrorMessageHandler<io.prebindgen.covertest.Storage> = io.prebindgen.covertest.StorageErrorMessageHandler { message -> throw java.lang.IllegalStateException(message) }): io.prebindgen.covertest.Storage = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call154(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.Storage? = null
        try {
            output0 = io.prebindgen.covertest.Storage(result[1] as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop25(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        true -> { 
onError.run(result[1] as String) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `resultErrorDrops`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call155() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { (result[1] as Long).toULong() }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

