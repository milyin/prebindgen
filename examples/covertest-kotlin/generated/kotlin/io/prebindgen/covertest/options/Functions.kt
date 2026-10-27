@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.options

public fun `nicheChoiceNew`(p0_w3: Int?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.options.NicheChoiceHandle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.options.NicheChoiceHandle = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) run { val value = requireNotNull(p0_w3); require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call168(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.options.NicheChoiceHandle? = null
        try {
            output0 = io.prebindgen.covertest.options.NicheChoiceHandle(result[1] as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop17(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `nicheChoiceId`(p0_w1: Boolean, p0_w3: Int, p0_w8: Int?, p0_w10: io.prebindgen.covertest.options.NicheChoiceHandle?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (p0_w1) if (true && p0_w3 == 1) requireNotNull(p0_w10) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { if (p0_w1) run { val value = p0_w3; require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w1) if (p0_w8 != null) run { val value = requireNotNull(p0_w8); require(value != -1) { "invalid option payload" }; value } else -1 else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (p0_w1) p0_w10?.raw ?: 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call169(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0]) { if (p0_w1) { if (true && p0_w3 == 1) p0_w10!!.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `optionNestedU32`(p0_w1: Boolean, p0_w5: Long?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Long?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w1) run { val value = if (p0_w5 != null) run { val value = requireNotNull(p0_w5); require(value != -1L) { "invalid option payload" }; value } else -1L; require(value != -2L) { "invalid option payload" }; value } else -2L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call170(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -2L) {
            if (result[1] != -1L) {
            }
            }
            build(result[1] != -2L, if (result[1] != -2L) if (result[1] != -1L) result[1] as Long else null else null)
        } catch (failure: Throwable) {
            if (result[1] != -2L) {
            if (result[1] != -1L) {
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `optionTripleText`(p0_w1: Boolean, p0_w3: Boolean, p0_w7: String?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Boolean?, String?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w1) run { val value = (if (p0_w3) 1 else 0); require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w1) if (p0_w3) if (p0_w7 != null) requireNotNull(requireNotNull(p0_w7)) { "invalid option payload" } else null else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call171(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -1) {
            if (result[1] == 1) {
            if (result[2] != null) {
            }
            }
            }
            build(result[1] != -1, if (result[1] != -1) result[1] == 1 else null, if (result[1] != -1) if (result[1] == 1) if (result[2] != null) result[2] as String else null else null else null)
        } catch (failure: Throwable) {
            if (result[1] != -1) {
            if (result[1] == 1) {
            if (result[2] != null) {
            }
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `optionTripleBytes`(p0_w1: Boolean, p0_w3: Boolean, p0_w7: ByteArray?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Boolean?, ByteArray?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w1) run { val value = (if (p0_w3) 1 else 0); require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w1) if (p0_w3) if (p0_w7 != null) requireNotNull(requireNotNull(p0_w7)) { "invalid option payload" } else null else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call172(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -1) {
            if (result[1] == 1) {
            if (result[2] != null) {
            }
            }
            }
            build(result[1] != -1, if (result[1] != -1) result[1] == 1 else null, if (result[1] != -1) if (result[1] == 1) if (result[2] != null) result[2] as ByteArray else null else null else null)
        } catch (failure: Throwable) {
            if (result[1] != -1) {
            if (result[1] == 1) {
            if (result[2] != null) {
            }
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `optionTripleI64`(p0_w1: Boolean, p0_w3: Boolean, p0_w7: Long?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Boolean?, Long?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w1) run { val value = if (p0_w3) run { val value = (if (p0_w7 != null) 1 else 0); require(value != -1) { "invalid option payload" }; value } else -1; require(value != -2) { "invalid option payload" }; value } else -2 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w1) if (p0_w3) if (p0_w7 != null) requireNotNull(p0_w7) else 0L else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call173(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -2) {
            if (result[1] != -1) {
            if (result[1] == 1) {
            }
            }
            }
            build(result[1] != -2, if (result[1] != -2) result[1] != -1 else null, if (result[1] != -2) if (result[1] != -1) if (result[1] == 1) result[2] as Long else null else null else null)
        } catch (failure: Throwable) {
            if (result[1] != -2) {
            if (result[1] != -1) {
            if (result[1] == 1) {
            }
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `optionTripleUnit`(p0_w1: Boolean, p0_w3: Boolean, p0_w5: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Boolean?, Boolean?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w1) run { val value = if (p0_w3) run { val value = (if (p0_w5) 1 else 0); require(value != -1) { "invalid option payload" }; value } else -1; require(value != -2) { "invalid option payload" }; value } else -2 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call174(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -2) {
            if (result[1] != -1) {
            if (result[1] == 1) {
            }
            }
            }
            build(result[1] != -2, if (result[1] != -2) result[1] != -1 else null, if (result[1] != -2) if (result[1] != -1) result[1] == 1 else null else null)
        } catch (failure: Throwable) {
            if (result[1] != -2) {
            if (result[1] != -1) {
            if (result[1] == 1) {
            }
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionalTextInputLen`(p0_w1: Boolean, p0_w4: String?, p0_w6: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w1) requireNotNull(p0_w4) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w1) p0_w6 else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call175(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionalBytesInputLen`(p0_w1: Boolean, p0_w4: ByteArray?, p0_w6: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w1) requireNotNull(p0_w4) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w1) p0_w6 else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call176(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `nestedNicheChoiceId`(p0_w1: Boolean, p0_w3: Boolean, p0_w5: Int, p0_w10: Int?, p0_w12: io.prebindgen.covertest.options.NicheChoiceHandle?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (p0_w1) if (p0_w3) if (true && true && p0_w5 == 1) requireNotNull(p0_w12) { "null handle" } else null else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { if (p0_w1) run { val value = if (p0_w3) run { val value = p0_w5; require(value != -1) { "invalid option payload" }; value } else -1; require(value != -2) { "invalid option payload" }; value } else -2 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w1) if (p0_w3) if (p0_w10 != null) run { val value = requireNotNull(p0_w10); require(value != -1) { "invalid option payload" }; value } else -1 else 0 else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (p0_w1) if (p0_w3) p0_w12?.raw ?: 0L else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call177(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0]) { if (p0_w1) { if (p0_w3) { if (true && true && p0_w5 == 1) p0_w12!!.markConsumed() } } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `optionNestedProjectionViews`(p0_w1: Boolean, p1_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, io.prebindgen.covertest.resources.ProjectionHandle?, io.prebindgen.covertest.resources.ProjectionHandle?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call178(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var output1: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            if (result[1] == 1) {
            if (result[2] != 0L) {
            output0 = io.prebindgen.covertest.resources.ProjectionHandle(result[2] as Long)
            output1 = io.prebindgen.covertest.resources.ProjectionHandle(result[3] as Long)
            }
            }
            build(result[1] == 1, if (result[1] == 1) if (result[2] != 0L) requireNotNull(output0) else null else null, if (result[1] == 1) if (result[2] != 0L) requireNotNull(output1) else null else null)
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            if (result[2] != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = output1; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[3] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionCtorNew`(p0_w3: Long?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.options.OptionCtor> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.options.OptionCtor = run {
    val result = try { run { val nativeArgument0 = try { (if (p0_w3 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w3 != null) requireNotNull(p0_w3) else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call179(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.options.OptionCtor? = null
        try {
            output0 = io.prebindgen.covertest.options.OptionCtor(result[1] as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop19(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionCtorId`(p0_w1: Int, p0_w6: Long?, p0_w8: io.prebindgen.covertest.options.OptionCtor?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (p0_w1 == 1) requireNotNull(p0_w8) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { (if (p0_w6 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (p0_w6 != null) requireNotNull(p0_w6) else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p0_w8?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call180(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0]) { if (p0_w1 == 1) p0_w8!!.markConsumed() } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `optionNestedResource`(p0_w1: Boolean, p0_w5: io.prebindgen.covertest.resources.OwnedResource?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, io.prebindgen.covertest.resources.OwnedResource?) -> R): R = run {
    val result = try { run { val handle0 = try { if (p0_w1) if (p0_w5 != null) if (true && true) requireNotNull(requireNotNull(p0_w5)) { "null handle" } else null else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { (if (p0_w1) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w1) if (p0_w5 != null) requireNotNull(p0_w5).raw else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call181(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0]) { if (p0_w1) { if (p0_w5 != null) { if (true && true) requireNotNull(p0_w5).markConsumed() } } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            if (result[1] == 1) {
            if (result[2] != 0L) {
            output0 = io.prebindgen.covertest.resources.OwnedResource(result[2] as Long)
            }
            }
            build(result[1] == 1, if (result[1] == 1) if (result[2] != 0L) requireNotNull(output0) else null else null)
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            if (result[2] != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionChoiceId`(p0_w1: Boolean, p0_w3: Int, p0_w6: Long, p0_w8: io.prebindgen.covertest.options.OptionChoice?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (p0_w1) if (true && p0_w3 == 1) requireNotNull(p0_w8) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { if (p0_w1) run { val value = p0_w3; require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w1) p0_w6 else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (p0_w1) p0_w8?.raw ?: 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call182(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0]) { if (p0_w1) { if (true && p0_w3 == 1) p0_w8!!.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionStrEcho`(p0_w3: String?, onError: io.prebindgen.covertest.JniErrorHandler<String?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String? = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(requireNotNull(p0_w3)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call183(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != null) {
            }
            if (result[1] != null) result[1] as String else null
        } catch (failure: Throwable) {
            if (result[1] != null) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionSliceEcho`(p0_w3: ByteArray?, onError: io.prebindgen.covertest.JniErrorHandler<ByteArray?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ByteArray? = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(requireNotNull(p0_w3)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call184(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != null) {
            }
            if (result[1] != null) result[1] as ByteArray else null
        } catch (failure: Throwable) {
            if (result[1] != null) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionStrLen`(p0_w3: String?, onError: io.prebindgen.covertest.JniErrorHandler<ULong?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong? = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(requireNotNull(p0_w3)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call185(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            }
            if (result[1] == 1) (result[2] as Long).toULong() else null
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionSliceLen`(p0_w3: ByteArray?, onError: io.prebindgen.covertest.JniErrorHandler<ULong?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong? = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(requireNotNull(p0_w3)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call186(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            }
            if (result[1] == 1) (result[2] as Long).toULong() else null
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionStringLen`(p0_w3: String?, onError: io.prebindgen.covertest.JniErrorHandler<ULong?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong? = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(requireNotNull(p0_w3)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call187(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            }
            if (result[1] == 1) (result[2] as Long).toULong() else null
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionI64Ref`(p0_w3: Long?, onError: io.prebindgen.covertest.JniErrorHandler<Long?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long? = run {
    val result = try { run { val nativeArgument0 = try { (if (p0_w3 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w3 != null) requireNotNull(p0_w3) else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call188(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            }
            if (result[1] == 1) result[2] as Long else null
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionResourcePair`(p0_w3: io.prebindgen.covertest.resources.OwnedResource?, p1_w3: io.prebindgen.covertest.resources.OwnedResource?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (p0_w3 != null) if (true) requireNotNull(requireNotNull(p0_w3)) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val handle1 = try { if (p1_w3 != null) if (true) requireNotNull(requireNotNull(p1_w3)) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0, handle1) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3).raw else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p1_w3 != null) requireNotNull(p1_w3).raw else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call189(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0]) { if (p0_w3 != null) { if (true) requireNotNull(p0_w3).markConsumed() }; if (p1_w3 != null) { if (true) requireNotNull(p1_w3).markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `resourceChoiceId`(p0_w1: Boolean, p0_w4: io.prebindgen.covertest.resources.OwnedResource?, p0_w6: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (p0_w1) if (true) requireNotNull(p0_w4) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { if (p0_w1) p0_w4?.raw ?: 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w1) p0_w6 else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call190(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0]) { if (p0_w1) { if (true) p0_w4!!.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `optionalPairNew`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.resources.OwnedResource?, io.prebindgen.covertest.resources.OwnedResource?) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call191() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg0_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            if (result[1] != 0L) {
            outputg0_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(result[1] as Long)
            }
            if (result[2] != 0L) {
            outputg0_g1_g0_0 = io.prebindgen.covertest.resources.OwnedResource(result[2] as Long)
            }
            build(if (result[1] != 0L) requireNotNull(outputg0_g0_g0_0) else null, if (result[2] != 0L) requireNotNull(outputg0_g1_g0_0) else null)
        } catch (failure: Throwable) {
            if (result[1] != 0L) {
            try { val owner = outputg0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            if (result[2] != 0L) {
            try { val owner = outputg0_g1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `optionProjectionViews`(p0_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.resources.ProjectionHandle?, io.prebindgen.covertest.resources.ProjectionHandle?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call192(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var output1: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            if (result[1] != 0L) {
            output0 = io.prebindgen.covertest.resources.ProjectionHandle(result[1] as Long)
            output1 = io.prebindgen.covertest.resources.ProjectionHandle(result[2] as Long)
            }
            build(if (result[1] != 0L) requireNotNull(output0) else null, if (result[1] != 0L) requireNotNull(output1) else null)
        } catch (failure: Throwable) {
            if (result[1] != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = output1; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionalBytesEcho`(p0_w1: io.prebindgen.covertest.options.OptionalBytes, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.options.OptionalBytes> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.options.OptionalBytes = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w1.`data` != null) requireNotNull(requireNotNull(p0_w1.`data`)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call193(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != null) {
            }
            io.prebindgen.covertest.options.OptionalBytes(if (result[1] != null) result[1] as ByteArray else null)
        } catch (failure: Throwable) {
            if (result[1] != null) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionResourceMut`(p0_w3: io.prebindgen.covertest.resources.OwnedResource?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (p0_w3 != null) if (true) requireNotNull(requireNotNull(p0_w3)) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3).raw else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call194(nativeArgument0) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionProjectionRef`(p0_w1: io.prebindgen.covertest.resources.ProjectionHandle?, p1_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.ProjectionHandle?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.ProjectionHandle? = run {
    val result = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call195(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            if (result[1] != 0L) {
            output0 = io.prebindgen.covertest.resources.ProjectionHandle(result[1] as Long)
            }
            if (result[1] != 0L) requireNotNull(output0) else null
        } catch (failure: Throwable) {
            if (result[1] != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop22(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionI64`(p0_w3: Long?, onError: io.prebindgen.covertest.JniErrorHandler<Long?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long? = run {
    val result = try { run { val nativeArgument0 = try { (if (p0_w3 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w3 != null) requireNotNull(p0_w3) else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call196(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            }
            if (result[1] == 1) result[2] as Long else null
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionChar`(p0_w3: Int?, onError: io.prebindgen.covertest.JniErrorHandler<Int?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int? = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) run { val value = requireNotNull(p0_w3); require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call197(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -1) {
            }
            if (result[1] != -1) result[1] as Int else null
        } catch (failure: Throwable) {
            if (result[1] != -1) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionU8`(p0_w3: Int?, onError: io.prebindgen.covertest.JniErrorHandler<Int?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int? = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) run { val value = requireNotNull(p0_w3); require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call198(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -1) {
            }
            if (result[1] != -1) result[1] as Int else null
        } catch (failure: Throwable) {
            if (result[1] != -1) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionBool`(p0_w3: Boolean?, onError: io.prebindgen.covertest.JniErrorHandler<Boolean?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean? = run {
    val result = try { run { val nativeArgument0 = try { (if (p0_w3 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w3 != null) requireNotNull(p0_w3) else false } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call199(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            }
            if (result[1] == 1) result[2] as Boolean else null
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionText`(p0_w3: String?, onError: io.prebindgen.covertest.JniErrorHandler<String?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String? = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(requireNotNull(p0_w3)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call200(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != null) {
            }
            if (result[1] != null) result[1] as String else null
        } catch (failure: Throwable) {
            if (result[1] != null) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionBytes`(p0_w3: ByteArray?, onError: io.prebindgen.covertest.JniErrorHandler<ByteArray?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ByteArray? = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(requireNotNull(p0_w3)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call201(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != null) {
            }
            if (result[1] != null) result[1] as ByteArray else null
        } catch (failure: Throwable) {
            if (result[1] != null) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionUnit`(p0_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Boolean> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val result = try { run { val nativeArgument0 = try { (if (p0_w1) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call202(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            }
            result[1] == 1
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `optionNested`(p0_w1: Boolean, p0_w5: Long?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Long?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w1) run { val value = (if (p0_w5 != null) 1 else 0); require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w1) if (p0_w5 != null) requireNotNull(p0_w5) else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call203(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -1) {
            if (result[1] == 1) {
            }
            }
            build(result[1] != -1, if (result[1] != -1) if (result[1] == 1) result[2] as Long else null else null)
        } catch (failure: Throwable) {
            if (result[1] != -1) {
            if (result[1] == 1) {
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `optionNestedChar`(p0_w1: Boolean, p0_w5: Int?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Int?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { if (p0_w1) run { val value = if (p0_w5 != null) run { val value = requireNotNull(p0_w5); require(value != -1) { "invalid option payload" }; value } else -1; require(value != -2) { "invalid option payload" }; value } else -2 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call204(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -2) {
            if (result[1] != -1) {
            }
            }
            build(result[1] != -2, if (result[1] != -2) if (result[1] != -1) result[1] as Int else null else null)
        } catch (failure: Throwable) {
            if (result[1] != -2) {
            if (result[1] != -1) {
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionResourceNew`(p0_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.OwnedResource?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.OwnedResource? = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call205(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            if (result[1] != 0L) {
            output0 = io.prebindgen.covertest.resources.OwnedResource(result[1] as Long)
            }
            if (result[1] != 0L) requireNotNull(output0) else null
        } catch (failure: Throwable) {
            if (result[1] != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionResourceEcho`(p0_w3: io.prebindgen.covertest.resources.OwnedResource?, p1_w1: Int, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.OwnedResource?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.OwnedResource? = run {
    val result = try { run { val handle0 = try { if (p0_w3 != null) if (true) requireNotNull(requireNotNull(p0_w3)) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3).raw else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call206(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0]) { if (p0_w3 != null) { if (true) requireNotNull(p0_w3).markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            if (result[1] != 0L) {
            output0 = io.prebindgen.covertest.resources.OwnedResource(result[1] as Long)
            }
            if (result[1] != 0L) requireNotNull(output0) else null
        } catch (failure: Throwable) {
            if (result[1] != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionResourceId`(p0_w3: io.prebindgen.covertest.resources.OwnedResource?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val result = try { run { val handle0 = try { if (p0_w3 != null) if (true) requireNotNull(requireNotNull(p0_w3)) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3).raw else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call207(nativeArgument0) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionalFieldsEcho`(p0_w3: io.prebindgen.covertest.options.OptionalFields?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.options.OptionalFields?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.options.OptionalFields? = run {
    val result = try { run { val nativeArgument0 = try { (if (p0_w3 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w3 != null) if (true) (if (requireNotNull(requireNotNull(p0_w3)).`count` != null) 1 else 0) else 0 else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (p0_w3 != null) if (true) if (requireNotNull(requireNotNull(p0_w3)).`count` != null) requireNotNull(requireNotNull(requireNotNull(p0_w3)).`count`) else 0L else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { if (p0_w3 != null) if (true) if (requireNotNull(requireNotNull(p0_w3)).`text` != null) requireNotNull(requireNotNull(requireNotNull(requireNotNull(p0_w3)).`text`)) { "invalid option payload" } else null else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { if (p0_w3 != null) if (true) if (requireNotNull(requireNotNull(p0_w3)).`letter` != null) run { val value = requireNotNull(requireNotNull(requireNotNull(p0_w3)).`letter`); require(value != -1) { "invalid option payload" }; value } else -1 else 0 else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call208(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] == 1) {
            if (result[2] == 1) {
            }
            if (result[4] != null) {
            }
            if (result[5] != -1) {
            }
            }
            if (result[1] == 1) io.prebindgen.covertest.options.OptionalFields(if (result[2] == 1) result[3] as Long else null, if (result[4] != null) result[4] as String else null, if (result[5] != -1) result[5] as Int else null) else null
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            if (result[2] == 1) {
            }
            if (result[4] != null) {
            }
            if (result[5] != -1) {
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `optionalFieldNew`(p0_w1: Boolean, p1_w3: Long?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Long?) -> R): R = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { (if (p1_w3 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (p1_w3 != null) requireNotNull(p1_w3) else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call209(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         try {
            if (result[1] != -1) {
            if (result[1] == 1) {
            }
            }
            build(result[1] != -1, if (result[1] != -1) if (result[1] == 1) result[2] as Long else null else null)
        } catch (failure: Throwable) {
            if (result[1] != -1) {
            if (result[1] == 1) {
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `optionalResourceNew`(p0_w1: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.options.OptionalResource?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.options.OptionalResource? = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call210(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var outputg0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            if (result[1] == 1) {
            if (result[2] != 0L) {
            outputg0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(result[2] as Long)
            }
            if (result[3] == 1) {
            if (result[4] == 1) {
            }
            if (result[6] != null) {
            }
            if (result[7] != -1) {
            }
            }
            }
            if (result[1] == 1) io.prebindgen.covertest.options.OptionalResource(if (result[2] != 0L) requireNotNull(outputg0_g0_0) else null, if (result[3] == 1) io.prebindgen.covertest.options.OptionalFields(if (result[4] == 1) result[5] as Long else null, if (result[6] != null) result[6] as String else null, if (result[7] != -1) result[7] as Int else null) else null) else null
        } catch (failure: Throwable) {
            if (result[1] == 1) {
            if (result[2] != 0L) {
            try { val owner = outputg0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            if (result[3] == 1) {
            if (result[4] == 1) {
            }
            if (result[6] != null) {
            }
            if (result[7] != -1) {
            }
            }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

