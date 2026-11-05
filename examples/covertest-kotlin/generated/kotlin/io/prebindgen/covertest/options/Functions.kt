@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.options

public fun nicheChoiceNew(value_c2da6115cd444ccd: Int?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.options.NicheChoiceHandle> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.options.NicheChoiceHandle = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (value_c2da6115cd444ccd != null) run { val value = requireNotNull(value_c2da6115cd444ccd); require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_niche_choice_new_a532294534010060(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.options.NicheChoiceHandle? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_NicheChoiceHandle_820b60fc3ef75039(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.options.NicheChoiceHandle? = null
        try {
            output0 = io.prebindgen.covertest.options.NicheChoiceHandle(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_NicheChoiceHandle_820b60fc3ef75039(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun nicheChoiceId(valuePresent: Boolean, valueSomeVariant: Int, valueSomeVariant0Value: Int?, valueSomeVariant1: io.prebindgen.covertest.options.NicheChoiceHandle?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (valuePresent) if (true && valueSomeVariant == 1) requireNotNull(valueSomeVariant1) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { if (valuePresent) run { val value = valueSomeVariant; require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valuePresent) if (valueSomeVariant0Value != null) run { val value = requireNotNull(valueSomeVariant0Value); require(value != -1) { "invalid option payload" }; value } else -1 else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (valuePresent) valueSomeVariant1?.raw ?: 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_niche_choice_id_ecb04a4cc855b867(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (valuePresent) { if (true && valueSomeVariant == 1) __pb_consume(valueSomeVariant1!!) } } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; if (valuePresent) { if (true && valueSomeVariant == 1) __pb_consume(valueSomeVariant1!!) }; handle0?.let { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun <R> optionNestedU32(valuePresent: Boolean, valueSome: Long?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Long?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (valuePresent) run { val value = if (valueSome != null) run { val value = requireNotNull(valueSome); require(value != -1L) { "invalid option payload" }; value } else -1L; require(value != -2L) { "invalid option payload" }; value } else -2L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_nested_u32_2e0792d512f0f24f(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != -2L) {
            if (nativeResult != -1L) {
            }
            }

build(nativeResult != -2L, if (nativeResult != -2L) if (nativeResult != -1L) nativeResult else null else null)
    } finally { returnCapture.release() }
}

public fun <R> optionTripleText(valuePresent: Boolean, valueSomePresent: Boolean, valueSomeSome: String?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Boolean?, String?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_String2_ErrorEmpty_7e68b805da700de5.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (valuePresent) run { val value = (if (valueSomePresent) 1 else 0); require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valuePresent) if (valueSomePresent) if (valueSomeSome != null) requireNotNull(requireNotNull(valueSomeSome)) { "invalid option payload" } else null else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_triple_text_cae6b50b503944e6(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 != -1) {
            if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != null) {
            }
            }
            }

build(returnCapture.s0 != -1, if (returnCapture.s0 != -1) returnCapture.s0 == 1 else null, if (returnCapture.s0 != -1) if (returnCapture.s0 == 1) if (returnCapture.s1 != null) returnCapture.s1 as String else null else null else null)
    } finally { returnCapture.release() }
}

public fun <R> optionTripleBytes(valuePresent: Boolean, valueSomePresent: Boolean, valueSomeSome: ByteArray?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Boolean?, ByteArray?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_ByteArray2_ErrorEmpty_a2aad6159c8c4c14.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (valuePresent) run { val value = (if (valueSomePresent) 1 else 0); require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valuePresent) if (valueSomePresent) if (valueSomeSome != null) requireNotNull(requireNotNull(valueSomeSome)) { "invalid option payload" } else null else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_triple_bytes_4f662b4b06709c31(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 != -1) {
            if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != null) {
            }
            }
            }

build(returnCapture.s0 != -1, if (returnCapture.s0 != -1) returnCapture.s0 == 1 else null, if (returnCapture.s0 != -1) if (returnCapture.s0 == 1) if (returnCapture.s1 != null) returnCapture.s1 as ByteArray else null else null else null)
    } finally { returnCapture.release() }
}

public fun <R> optionTripleI64(valuePresent: Boolean, valueSomePresent: Boolean, valueSomeSome: Long?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Boolean?, Long?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (valuePresent) run { val value = if (valueSomePresent) run { val value = (if (valueSomeSome != null) 1 else 0); require(value != -1) { "invalid option payload" }; value } else -1; require(value != -2) { "invalid option payload" }; value } else -2 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valuePresent) if (valueSomePresent) if (valueSomeSome != null) requireNotNull(valueSomeSome) else 0L else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_triple_i64_53dfd00875940117(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 != -2) {
            if (returnCapture.s0 != -1) {
            if (returnCapture.s0 == 1) {
            }
            }
            }

build(returnCapture.s0 != -2, if (returnCapture.s0 != -2) returnCapture.s0 != -1 else null, if (returnCapture.s0 != -2) if (returnCapture.s0 != -1) if (returnCapture.s0 == 1) returnCapture.s1 else null else null else null)
    } finally { returnCapture.release() }
}

public fun <R> optionTripleUnit(valuePresent: Boolean, valueSomePresent: Boolean, valueSomeSomePresent: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Boolean?, Boolean?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (valuePresent) run { val value = if (valueSomePresent) run { val value = (if (valueSomeSomePresent) 1 else 0); require(value != -1) { "invalid option payload" }; value } else -1; require(value != -2) { "invalid option payload" }; value } else -2 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_triple_unit_c0762ed5037f126a(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != -2) {
            if (nativeResult != -1) {
            if (nativeResult == 1) {
            }
            }
            }

build(nativeResult != -2, if (nativeResult != -2) nativeResult != -1 else null, if (nativeResult != -2) if (nativeResult != -1) nativeResult == 1 else null else null)
    } finally { returnCapture.release() }
}

public fun optionalTextInputLen(valuePresent: Boolean, valueText: String?, valueNumber: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (valuePresent) requireNotNull(valueText) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valuePresent) valueNumber else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_optional_text_input_len_5334cfb37cec76c0(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun optionalBytesInputLen(valuePresent: Boolean, valueBytes: ByteArray?, valueNumber: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (valuePresent) requireNotNull(valueBytes) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valuePresent) valueNumber else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_optional_bytes_input_len_7c4fd8ed8adbd4f7(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun nestedNicheChoiceId(valuePresent: Boolean, valueSomePresent: Boolean, valueSomeSomeVariant: Int, valueSomeSomeVariant0Value: Int?, valueSomeSomeVariant1: io.prebindgen.covertest.options.NicheChoiceHandle?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (valuePresent) if (valueSomePresent) if (true && true && valueSomeSomeVariant == 1) requireNotNull(valueSomeSomeVariant1) { "null handle" } else null else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { if (valuePresent) run { val value = if (valueSomePresent) run { val value = valueSomeSomeVariant; require(value != -1) { "invalid option payload" }; value } else -1; require(value != -2) { "invalid option payload" }; value } else -2 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valuePresent) if (valueSomePresent) if (valueSomeSomeVariant0Value != null) run { val value = requireNotNull(valueSomeSomeVariant0Value); require(value != -1) { "invalid option payload" }; value } else -1 else 0 else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (valuePresent) if (valueSomePresent) valueSomeSomeVariant1?.raw ?: 0L else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_nested_niche_choice_id_3b8d8d5f9bee3571(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (valuePresent) { if (valueSomePresent) { if (true && true && valueSomeSomeVariant == 1) __pb_consume(valueSomeSomeVariant1!!) } } } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; if (valuePresent) { if (valueSomePresent) { if (true && true && valueSomeSomeVariant == 1) __pb_consume(valueSomeSomeVariant1!!) } }; handle0?.let { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun <R> optionNestedProjectionViews(outer: Boolean, inner: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, io.prebindgen.covertest.resources.ProjectionHandle?, io.prebindgen.covertest.resources.ProjectionHandle?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long_Long3_ErrorEmpty_2e9e07b44c221bd1.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { outer } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { inner } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_nested_projection_views_d622fb44db024930(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var output1: io.prebindgen.covertest.resources.ProjectionHandle? = null
             if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = output1; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var output1: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != 0L) {
            output0 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s1)
            output1 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s2)
            }
            }
            build(returnCapture.s0 == 1, if (returnCapture.s0 == 1) if (returnCapture.s1 != 0L) requireNotNull(output0) else null else null, if (returnCapture.s0 == 1) if (returnCapture.s1 != 0L) requireNotNull(output1) else null else null)
        } catch (failure: Throwable) {
            if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = output1; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s2) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun optionCtorNew(value: Long?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.options.OptionCtor> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.options.OptionCtor = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { (if (value != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (value != null) requireNotNull(value) else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_ctor_new_3e8a52ff9fd43303(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.options.OptionCtor? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OptionCtor_9701b4353b1ab7b0(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.options.OptionCtor? = null
        try {
            output0 = io.prebindgen.covertest.options.OptionCtor(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OptionCtor_9701b4353b1ab7b0(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun optionCtorId(valueVariant: Int, valueVariant0Value: Long?, valueVariant1: io.prebindgen.covertest.options.OptionCtor?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (valueVariant == 1) requireNotNull(valueVariant1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { valueVariant } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { (if (valueVariant0Value != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (valueVariant0Value != null) requireNotNull(valueVariant0Value) else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { valueVariant1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_ctor_id_5d244c0e4c436a1a(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (valueVariant == 1) __pb_consume(valueVariant1!!) } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; if (valueVariant == 1) __pb_consume(valueVariant1!!); handle0?.let { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun <R> optionNestedResource(valuePresent: Boolean, valueSome: io.prebindgen.covertest.resources.OwnedResource?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, io.prebindgen.covertest.resources.OwnedResource?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (valuePresent) if (valueSome != null) if (true && true) requireNotNull(requireNotNull(valueSome)) { "null handle" } else null else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { (if (valuePresent) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valuePresent) if (valueSome != null) requireNotNull(valueSome).raw else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_nested_resource_b0bbbca600a8c975(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (valuePresent) { if (valueSome != null) { if (true && true) __pb_consume(requireNotNull(valueSome)) } } } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; if (valuePresent) { if (valueSome != null) { if (true && true) __pb_consume(requireNotNull(valueSome)) } }; handle0?.let { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.resources.OwnedResource? = null
             if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != 0L) {
            output0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s1)
            }
            }
            build(returnCapture.s0 == 1, if (returnCapture.s0 == 1) if (returnCapture.s1 != 0L) requireNotNull(output0) else null else null)
        } catch (failure: Throwable) {
            if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun optionChoiceId(valuePresent: Boolean, valueVariant: Int, valueVariant0Id: Long, valueVariant1: io.prebindgen.covertest.options.OptionChoice?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (valuePresent) if (true && valueVariant == 1) requireNotNull(valueVariant1) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { if (valuePresent) run { val value = valueVariant; require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valuePresent) valueVariant0Id else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (valuePresent) valueVariant1?.raw ?: 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_choice_id_4d12974690909134(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (valuePresent) { if (true && valueVariant == 1) __pb_consume(valueVariant1!!) } } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; if (valuePresent) { if (true && valueVariant == 1) __pb_consume(valueVariant1!!) }; handle0?.let { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun optionStrEcho(value: String?, onError: io.prebindgen.covertest.JniErrorHandler<String?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (value != null) requireNotNull(requireNotNull(value)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_str_echo_94daf2f0c810e1a7(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != null) {
            }

if (nativeResult != null) nativeResult as String else null
    } finally { returnCapture.release() }
}

public fun optionSliceEcho(value: ByteArray?, onError: io.prebindgen.covertest.JniErrorHandler<ByteArray?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ByteArray? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (value != null) requireNotNull(requireNotNull(value)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_slice_echo_d601bb0588850bdb(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != null) {
            }

if (nativeResult != null) nativeResult as ByteArray else null
    } finally { returnCapture.release() }
}

public fun optionStrLen(value: String?, onError: io.prebindgen.covertest.JniErrorHandler<ULong?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (value != null) requireNotNull(requireNotNull(value)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_str_len_e1179a8d3f3ccc82(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) (returnCapture.s1).toULong() else null
    } finally { returnCapture.release() }
}

public fun optionSliceLen(value: ByteArray?, onError: io.prebindgen.covertest.JniErrorHandler<ULong?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (value != null) requireNotNull(requireNotNull(value)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_slice_len_c45226bd8197ca90(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) (returnCapture.s1).toULong() else null
    } finally { returnCapture.release() }
}

public fun optionStringLen(value: String?, onError: io.prebindgen.covertest.JniErrorHandler<ULong?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (value != null) requireNotNull(requireNotNull(value)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_string_len_5af3d51e1e87974d(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) (returnCapture.s1).toULong() else null
    } finally { returnCapture.release() }
}

public fun optionI64Ref(value: Long?, onError: io.prebindgen.covertest.JniErrorHandler<Long?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { (if (value != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (value != null) requireNotNull(value) else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_i64_ref_fda4b6cd002e52b6(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) returnCapture.s1 else null
    } finally { returnCapture.release() }
}

public fun optionResourcePair(first: io.prebindgen.covertest.resources.OwnedResource?, second: io.prebindgen.covertest.resources.OwnedResource?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (first != null) if (true) requireNotNull(requireNotNull(first)) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; val handle1 = try { if (second != null) if (true) requireNotNull(requireNotNull(second)) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0, handle1) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { if (first != null) requireNotNull(first).raw else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (second != null) requireNotNull(second).raw else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_resource_pair_b954aa61fb636e9b(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (first != null) { if (true) __pb_consume(requireNotNull(first)) }; if (second != null) { if (true) __pb_consume(requireNotNull(second)) } } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; if (first != null) { if (true) __pb_consume(requireNotNull(first)) }; if (second != null) { if (true) __pb_consume(requireNotNull(second)) }; handle0?.let { if (it.address in taken) it.markConsumed() }; handle1?.let { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun resourceChoiceId(valuePresent: Boolean, valueResource: io.prebindgen.covertest.resources.OwnedResource?, valueOffset: Long, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (valuePresent) if (true) requireNotNull(valueResource) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { if (valuePresent) valueResource?.raw ?: 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valuePresent) valueOffset else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_resource_choice_id_7f8664c9adf8ecee(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (valuePresent) { if (true) __pb_consume(valueResource!!) } } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; if (valuePresent) { if (true) __pb_consume(valueResource!!) }; handle0?.let { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun <R> optionalPairNew(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.resources.OwnedResource?, io.prebindgen.covertest.resources.OwnedResource?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long2_ErrorEmpty_951bc171243c8a73.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_optional_pair_new_a373cf8922fb383f(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg0_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
             if (returnCapture.s0 != 0L) {
            try { val owner = outputg0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            if (returnCapture.s1 != 0L) {
            try { val owner = outputg0_g1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        var outputg0_g1_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            if (returnCapture.s0 != 0L) {
            outputg0_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s0)
            }
            if (returnCapture.s1 != 0L) {
            outputg0_g1_g0_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s1)
            }
            build(if (returnCapture.s0 != 0L) requireNotNull(outputg0_g0_g0_0) else null, if (returnCapture.s1 != 0L) requireNotNull(outputg0_g1_g0_0) else null)
        } catch (failure: Throwable) {
            if (returnCapture.s0 != 0L) {
            try { val owner = outputg0_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            if (returnCapture.s1 != 0L) {
            try { val owner = outputg0_g1_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> optionProjectionViews(present: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (io.prebindgen.covertest.resources.ProjectionHandle?, io.prebindgen.covertest.resources.ProjectionHandle?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long2_ErrorEmpty_951bc171243c8a73.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { present } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_projection_views_5f2b052a4aadad2a(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var output1: io.prebindgen.covertest.resources.ProjectionHandle? = null
             if (returnCapture.s0 != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = output1; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        var output1: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            if (returnCapture.s0 != 0L) {
            output0 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s0)
            output1 = io.prebindgen.covertest.resources.ProjectionHandle(returnCapture.s1)
            }
            build(if (returnCapture.s0 != 0L) requireNotNull(output0) else null, if (returnCapture.s0 != 0L) requireNotNull(output1) else null)
        } catch (failure: Throwable) {
            if (returnCapture.s0 != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s0) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            try { val owner = output1; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun optionalBytesEcho(value: io.prebindgen.covertest.options.OptionalBytes, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.options.OptionalBytes> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.options.OptionalBytes = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (value.data != null) requireNotNull(requireNotNull(value.data)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_optional_bytes_echo_16b56811763936ed(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != null) {
            }

io.prebindgen.covertest.options.OptionalBytes(if (nativeResult != null) nativeResult as ByteArray else null)
    } finally { returnCapture.release() }
}

public fun optionResourceMut(value: io.prebindgen.covertest.resources.OwnedResource?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (value != null) if (true) requireNotNull(requireNotNull(value)) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { if (value != null) requireNotNull(value).raw else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_resource_mut_7bbfc5f1d8d2d0c3(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun optionProjectionRef(value: io.prebindgen.covertest.resources.ProjectionHandle?, present: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.ProjectionHandle?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.ProjectionHandle? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(value) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { value?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { present } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_projection_ref_9f590934a25a729a(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.resources.ProjectionHandle? = null
             if (nativeResult != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.resources.ProjectionHandle? = null
        try {
            if (nativeResult != 0L) {
            output0 = io.prebindgen.covertest.resources.ProjectionHandle(nativeResult)
            }
            if (nativeResult != 0L) requireNotNull(output0) else null
        } catch (failure: Throwable) {
            if (nativeResult != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_ProjectionHandle_46b41135284839d8(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun optionI64(value: Long?, onError: io.prebindgen.covertest.JniErrorHandler<Long?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { (if (value != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (value != null) requireNotNull(value) else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_i64_351387af348bdd22(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) returnCapture.s1 else null
    } finally { returnCapture.release() }
}

public fun optionChar(value_c2da6115cd444ccd: Int?, onError: io.prebindgen.covertest.JniErrorHandler<Int?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (value_c2da6115cd444ccd != null) run { val value = requireNotNull(value_c2da6115cd444ccd); require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_char_8b0e6ac80fbfb499(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != -1) {
            }

if (nativeResult != -1) nativeResult else null
    } finally { returnCapture.release() }
}

public fun optionU8(value_c2da6115cd444ccd: Int?, onError: io.prebindgen.covertest.JniErrorHandler<Int?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Int? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (value_c2da6115cd444ccd != null) run { val value = requireNotNull(value_c2da6115cd444ccd); require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_u8_28e6cb3d73ac58b7(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != -1) {
            }

if (nativeResult != -1) nativeResult else null
    } finally { returnCapture.release() }
}

public fun optionBool(value: Boolean?, onError: io.prebindgen.covertest.JniErrorHandler<Boolean?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Boolean2_ErrorEmpty_2c463b65ab58550c.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { (if (value != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (value != null) requireNotNull(value) else false } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_bool_abd9a75cd1e6da39(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) returnCapture.s1 else null
    } finally { returnCapture.release() }
}

public fun optionText(value: String?, onError: io.prebindgen.covertest.JniErrorHandler<String?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (value != null) requireNotNull(requireNotNull(value)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_text_1006bf1c6da09727(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != null) {
            }

if (nativeResult != null) nativeResult as String else null
    } finally { returnCapture.release() }
}

public fun optionBytes(value: ByteArray?, onError: io.prebindgen.covertest.JniErrorHandler<ByteArray?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ByteArray? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (value != null) requireNotNull(requireNotNull(value)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_bytes_2c7d160b5c896230(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != null) {
            }

if (nativeResult != null) nativeResult as ByteArray else null
    } finally { returnCapture.release() }
}

public fun optionUnit(value: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<Boolean> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Boolean = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { (if (value) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_unit_d5c35f5bcf4cdd85(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult == 1) {
            }

nativeResult == 1
    } finally { returnCapture.release() }
}

public fun <R> optionNested(valuePresent: Boolean, valueSome: Long?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Long?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (valuePresent) run { val value = (if (valueSome != null) 1 else 0); require(value != -1) { "invalid option payload" }; value } else -1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (valuePresent) if (valueSome != null) requireNotNull(valueSome) else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_nested_3c7b2ba952e35029(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 != -1) {
            if (returnCapture.s0 == 1) {
            }
            }

build(returnCapture.s0 != -1, if (returnCapture.s0 != -1) if (returnCapture.s0 == 1) returnCapture.s1 else null else null)
    } finally { returnCapture.release() }
}

public fun <R> optionNestedChar(valuePresent: Boolean, valueSome: Int?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Int?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (valuePresent) run { val value = if (valueSome != null) run { val value = requireNotNull(valueSome); require(value != -1) { "invalid option payload" }; value } else -1; require(value != -2) { "invalid option payload" }; value } else -2 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_nested_char_0fbb5a6359c6a2ee(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != -2) {
            if (nativeResult != -1) {
            }
            }

build(nativeResult != -2, if (nativeResult != -2) if (nativeResult != -1) nativeResult else null else null)
    } finally { returnCapture.release() }
}

public fun optionResourceNew(present: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.OwnedResource?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.OwnedResource? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { present } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_resource_new_cd6aa3cab1e8ab93(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.resources.OwnedResource? = null
             if (nativeResult != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            if (nativeResult != 0L) {
            output0 = io.prebindgen.covertest.resources.OwnedResource(nativeResult)
            }
            if (nativeResult != 0L) requireNotNull(output0) else null
        } catch (failure: Throwable) {
            if (nativeResult != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun optionResourceEcho(value: io.prebindgen.covertest.resources.OwnedResource?, valid: Int, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.OwnedResource?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.OwnedResource? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (value != null) if (true) requireNotNull(requireNotNull(value)) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        val ownership = LongArray(1) { -1L }
        try { run { val nativeArgument0 = try { if (value != null) requireNotNull(value).raw else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { valid } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_resource_echo_92588c87a7537517(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0] >= 0L) { val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { it.markConsumed() }; if (value != null) { if (true) __pb_consume(requireNotNull(value)) } } else { var remaining = -1L - ownership[0]; val taken = kotlin.collections.HashSet<Long>(); val __pb_consume: (io.prebindgen.covertest.NativeHandle) -> Unit = { handle -> if (remaining > 0L) { taken.add(handle.address); handle.markConsumed(); remaining-- } else if (handle.address in taken) handle.markConsumed() }; if (value != null) { if (true) __pb_consume(requireNotNull(value)) }; handle0?.let { if (it.address in taken) it.markConsumed() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.resources.OwnedResource? = null
             if (nativeResult != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
 }
        return@run onError.run(returnCapture.message) }

            var output0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            if (nativeResult != 0L) {
            output0 = io.prebindgen.covertest.resources.OwnedResource(nativeResult)
            }
            if (nativeResult != 0L) requireNotNull(output0) else null
        } catch (failure: Throwable) {
            if (nativeResult != 0L) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun optionResourceId(value: io.prebindgen.covertest.resources.OwnedResource?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (value != null) if (true) requireNotNull(requireNotNull(value)) { "null handle" } else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { if (value != null) requireNotNull(value).raw else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_option_resource_id_3befc66c4766a89e(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun optionalFieldsEcho(value_c2da6115cd444ccd: io.prebindgen.covertest.options.OptionalFields?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.options.OptionalFields?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.options.OptionalFields? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Int_Long5_ErrorEmpty_072908d3b52dbcec.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { (if (value_c2da6115cd444ccd != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (value_c2da6115cd444ccd != null) if (true) (if (requireNotNull(requireNotNull(value_c2da6115cd444ccd)).count != null) 1 else 0) else 0 else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (value_c2da6115cd444ccd != null) if (true) if (requireNotNull(requireNotNull(value_c2da6115cd444ccd)).count != null) requireNotNull(requireNotNull(requireNotNull(value_c2da6115cd444ccd)).count) else 0L else 0L else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { if (value_c2da6115cd444ccd != null) if (true) if (requireNotNull(requireNotNull(value_c2da6115cd444ccd)).text != null) requireNotNull(requireNotNull(requireNotNull(requireNotNull(value_c2da6115cd444ccd)).text)) { "invalid option payload" } else null else null else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { if (value_c2da6115cd444ccd != null) if (true) if (requireNotNull(requireNotNull(value_c2da6115cd444ccd)).letter != null) run { val value = requireNotNull(requireNotNull(requireNotNull(value_c2da6115cd444ccd)).letter); require(value != -1) { "invalid option payload" }; value } else -1 else 0 else 0 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_optional_fields_echo_303b818252f2c60b(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            if (returnCapture.s1 == 1) {
            }
            if (returnCapture.s3 != null) {
            }
            if (returnCapture.s4 != -1) {
            }
            }

if (returnCapture.s0 == 1) io.prebindgen.covertest.options.OptionalFields(if (returnCapture.s1 == 1) returnCapture.s2 else null, if (returnCapture.s3 != null) returnCapture.s3 as String else null, if (returnCapture.s4 != -1) returnCapture.s4 else null) else null
    } finally { returnCapture.release() }
}

public fun <R> optionalFieldNew(present: Boolean, value: Long?, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Boolean, Long?) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { present } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { (if (value != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { if (value != null) requireNotNull(value) else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_optional_field_new_16797b4e790c764e(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 != -1) {
            if (returnCapture.s0 == 1) {
            }
            }

build(returnCapture.s0 != -1, if (returnCapture.s0 != -1) if (returnCapture.s0 == 1) returnCapture.s1 else null else null)
    } finally { returnCapture.release() }
}

public fun optionalResourceNew(present: Boolean, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.options.OptionalResource?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.options.OptionalResource? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long_Int7_ErrorEmpty_7a075b93a01b6645.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { present } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_optional_resource_new_79dab16fa38a4430(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var outputg0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
             if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != 0L) {
            try { val owner = outputg0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            }
 }
        return@run onError.run(returnCapture.message) }

            var outputg0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != 0L) {
            outputg0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(returnCapture.s1)
            }
            if (returnCapture.s2 == 1) {
            if (returnCapture.s3 == 1) {
            }
            if (returnCapture.s5 != null) {
            }
            if (returnCapture.s6 != -1) {
            }
            }
            }
            if (returnCapture.s0 == 1) io.prebindgen.covertest.options.OptionalResource(if (returnCapture.s1 != 0L) requireNotNull(outputg0_g0_0) else null, if (returnCapture.s2 == 1) io.prebindgen.covertest.options.OptionalFields(if (returnCapture.s3 == 1) returnCapture.s4 else null, if (returnCapture.s5 != null) returnCapture.s5 as String else null, if (returnCapture.s6 != -1) returnCapture.s6 else null) else null) else null
        } catch (failure: Throwable) {
            if (returnCapture.s0 == 1) {
            if (returnCapture.s1 != 0L) {
            try { val owner = outputg0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(returnCapture.s1) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}
