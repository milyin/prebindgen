@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.resources

public fun `projectionPairBorrowed`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.ProjectionPair> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.ProjectionPair = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call259() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            outputg0_0 = io.prebindgen.covertest.resources.ProjectionHandle(result[1] as Long)
            outputg1_0 = io.prebindgen.covertest.resources.ProjectionHandle(result[2] as Long)
            io.prebindgen.covertest.resources.ProjectionPair(requireNotNull(outputg0_0), requireNotNull(outputg1_0))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `projectionPairOwned`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.ProjectionPair> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.ProjectionPair = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call260() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            outputg0_0 = io.prebindgen.covertest.resources.ProjectionHandle(result[1] as Long)
            outputg1_0 = io.prebindgen.covertest.resources.ProjectionHandle(result[2] as Long)
            io.prebindgen.covertest.resources.ProjectionPair(requireNotNull(outputg0_0), requireNotNull(outputg1_0))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `projectionPairMixed`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.ProjectionPair> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.ProjectionPair = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call261() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            outputg0_0 = io.prebindgen.covertest.resources.ProjectionHandle(result[1] as Long)
            outputg1_0 = io.prebindgen.covertest.resources.ProjectionHandle(result[2] as Long)
            io.prebindgen.covertest.resources.ProjectionPair(requireNotNull(outputg0_0), requireNotNull(outputg1_0))
        } catch (failure: Throwable) {
            try { val owner = outputg0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `projectionPairViews`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.resources.ProjectionPair, io.prebindgen.covertest.resources.ProjectionPair) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call262() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_g0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg0_g1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_g0_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var outputg1_g1_0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            outputg0_g0_0 = io.prebindgen.covertest.resources.ProjectionHandle(result[1] as Long)
            outputg0_g1_0 = io.prebindgen.covertest.resources.ProjectionHandle(result[2] as Long)
            outputg1_g0_0 = io.prebindgen.covertest.resources.ProjectionHandle(result[3] as Long)
            outputg1_g1_0 = io.prebindgen.covertest.resources.ProjectionHandle(result[4] as Long)
            build(io.prebindgen.covertest.resources.ProjectionPair(requireNotNull(outputg0_g0_0), requireNotNull(outputg0_g1_0)), io.prebindgen.covertest.resources.ProjectionPair(requireNotNull(outputg1_g0_0), requireNotNull(outputg1_g1_0)))
        } catch (failure: Throwable) {
            try { val owner = outputg0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[3] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[4] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `projectionClones`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call263() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { (result[1] as Long).toULong() }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `projectionDrops`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call264() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { (result[1] as Long).toULong() }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `resourceBundleNew`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.ResourceBundle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.ResourceBundle = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call265() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg1_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg1_g0_g1_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg2_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            outputg1_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(result[2] as Long)
            outputg1_g0_g1_0 = io.prebindgen.covertest.resources.OwnedResource(result[3] as Long)
            outputg2_0 = io.prebindgen.covertest.resources.OwnedResource(result[4] as Long)
            io.prebindgen.covertest.resources.ResourceBundle(result[1] as String, io.prebindgen.covertest.resources.ResourcePair(requireNotNull(outputg1_g0_g0_0), requireNotNull(outputg1_g0_g1_0)), requireNotNull(outputg2_0), io.prebindgen.covertest.values.DataFloats(result[5] as ByteArray, result[6] as Float, result[7] as Double))
        } catch (failure: Throwable) {
            try { val owner = outputg1_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_g0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[3] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[4] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `resourceBundleDefault`(onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.ResourceBundle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.ResourceBundle = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call266() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg1_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg1_g0_g1_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg2_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            outputg1_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(result[2] as Long)
            outputg1_g0_g1_0 = io.prebindgen.covertest.resources.OwnedResource(result[3] as Long)
            outputg2_0 = io.prebindgen.covertest.resources.OwnedResource(result[4] as Long)
            io.prebindgen.covertest.resources.ResourceBundle(result[1] as String, io.prebindgen.covertest.resources.ResourcePair(requireNotNull(outputg1_g0_g0_0), requireNotNull(outputg1_g0_g1_0)), requireNotNull(outputg2_0), io.prebindgen.covertest.values.DataFloats(result[5] as ByteArray, result[6] as Float, result[7] as Double))
        } catch (failure: Throwable) {
            try { val owner = outputg1_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg1_g0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[3] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[4] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `resourceBundleFields`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (String, io.prebindgen.covertest.resources.ResourcePair, io.prebindgen.covertest.resources.OwnedResource, io.prebindgen.covertest.values.DataFloats) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call267() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_g1_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg0_g1_g0_g1_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg0_g2_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            outputg0_g1_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(result[2] as Long)
            outputg0_g1_g0_g1_0 = io.prebindgen.covertest.resources.OwnedResource(result[3] as Long)
            outputg0_g2_0 = io.prebindgen.covertest.resources.OwnedResource(result[4] as Long)
            build(result[1] as String, io.prebindgen.covertest.resources.ResourcePair(requireNotNull(outputg0_g1_g0_g0_0), requireNotNull(outputg0_g1_g0_g1_0)), requireNotNull(outputg0_g2_0), io.prebindgen.covertest.values.DataFloats(result[5] as ByteArray, result[6] as Float, result[7] as Double))
        } catch (failure: Throwable) {
            try { val owner = outputg0_g1_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g1_g0_g1_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[3] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = outputg0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[4] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `apiRecordNew`(onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call268() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `apiRecordViews`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (`count`: Long, `label`: String) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call269() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { build(result[1] as Long, result[2] as String) }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `ownedResourceId`(p0_w1: io.prebindgen.covertest.resources.OwnedResource?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call270(nativeArgument0) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `ownedResourceDrops`(onError: io.prebindgen.covertest.JniErrorHandler<ULong> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call271() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { (result[1] as Long).toULong() }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `apiRecordRowsFold`(initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, `count`: Long, `label`: String) -> A): A = run {
    val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink303 { v0, v1 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val result = try { try { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call303(nativeArgument0, nativeArgument1) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (result[1] != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, __pb_fold_row[0] as Long, __pb_fold_row[1] as String)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (result[1] != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        } }
        
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

