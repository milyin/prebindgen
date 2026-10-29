@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.model

public fun <R> `stampNew`(p0_w1: Long, p1_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Long, Long) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long2_ErrorEmpty_951bc171243c8a73.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_stamp_new_e7d8c1d49f7ad8b1(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    build(returnCapture.s0, returnCapture.s1)
    } finally { returnCapture.release() }
}

public fun `plainNoteEcho`(p0_w3: String?, onError: io.prebindgen.covertest.JniErrorHandler<String?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(requireNotNull(p0_w3)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_plain_note_echo_04effec8337b6f4f(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);              if (nativeResult != null) {
            }
 }
        return@run onError.run(returnCapture.message) }

            try {
            if (nativeResult != null) {
            }
            if (nativeResult != null) nativeResult as String else null
        } catch (failure: Throwable) {
            if (nativeResult != null) {
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `unsignedRoundTrip`(p0_w1: Int, p1_w1: Int, p2_w1: Long, p3_w1: ULong, p4_w3: ULong?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.model.Unsigned> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.model.Unsigned = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Int_Long6_ErrorEmpty_b3ecce38dd00826e.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p2_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { p3_w1.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { (if (p4_w3 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { if (p4_w3 != null) requireNotNull(p4_w3).toLong() else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_unsigned_round_trip_b59be7bd92f5b575(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);              if (returnCapture.s4 == 1) {
            }
 }
        return@run onError.run(returnCapture.message) }

            try {
            if (returnCapture.s4 == 1) {
            }
            io.prebindgen.covertest.model.Unsigned(returnCapture.s0, returnCapture.s1, returnCapture.s2, (returnCapture.s3).toULong(), if (returnCapture.s4 == 1) (returnCapture.s5).toULong() else null)
        } catch (failure: Throwable) {
            if (returnCapture.s4 == 1) {
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `unsignedOptional`(p0_w3: ULong?, onError: io.prebindgen.covertest.JniErrorHandler<ULong?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { (if (p0_w3 != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (p0_w3 != null) requireNotNull(p0_w3).toLong() else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_unsigned_optional_7f17c540c43bf08f(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);              if (returnCapture.s0 == 1) {
            }
 }
        return@run onError.run(returnCapture.message) }

            try {
            if (returnCapture.s0 == 1) {
            }
            if (returnCapture.s0 == 1) (returnCapture.s1).toULong() else null
        } catch (failure: Throwable) {
            if (returnCapture.s0 == 1) {
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `blobValueNew`(p0_w1: Long, p1_w1: ByteArray?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.model.BlobValue> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.model.BlobValue = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long_ByteArray3_ErrorEmpty_aa6ead24a4efe458.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_blob_value_new_34260fb08c70acda(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    io.prebindgen.covertest.model.BlobValue(io.prebindgen.covertest.model.Stamp(returnCapture.s0, returnCapture.s1), returnCapture.s2 as ByteArray)
    } finally { returnCapture.release() }
}

public fun `blobValueEcho`(p0_w1: io.prebindgen.covertest.model.BlobValue, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.model.BlobValue> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.model.BlobValue = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long_ByteArray3_ErrorEmpty_aa6ead24a4efe458.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1.`stamp`.`secs` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1.`stamp`.`nanos` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p0_w1.`id` } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_blob_value_echo_711e78e87dab6447(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    io.prebindgen.covertest.model.BlobValue(io.prebindgen.covertest.model.Stamp(returnCapture.s0, returnCapture.s1), returnCapture.s2 as ByteArray)
    } finally { returnCapture.release() }
}
