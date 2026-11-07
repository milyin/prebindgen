@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.model

public fun <R> stampNew(secs: Long, nanos: Long, onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (Long, Long) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long2_ErrorEmpty_951bc171243c8a73.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { secs } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { nanos } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_stamp_new_e7d8c1d49f7ad8b1(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    build(returnCapture.s0, returnCapture.s1)
    } finally { returnCapture.release() }
}

public fun plainNoteEcho(note: String?, onError: io.prebindgen.covertest.JniErrorHandler<String?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): String? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { if (note != null) requireNotNull(requireNotNull(note)) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_plain_note_echo_04effec8337b6f4f(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (nativeResult != null) {
            }

if (nativeResult != null) nativeResult as String else null
    } finally { returnCapture.release() }
}

public fun unsignedRoundTrip(byte: Int, short: Int, int: Long, long: ULong, maybeLong: ULong?, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.model.Unsigned> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.model.Unsigned = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Int_Long6_ErrorEmpty_b3ecce38dd00826e.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { byte } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { short } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { int } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { long.toLong() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { (if (maybeLong != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { if (maybeLong != null) requireNotNull(maybeLong).toLong() else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_unsigned_round_trip_b59be7bd92f5b575(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s4 == 1) {
            }

io.prebindgen.covertest.model.Unsigned(returnCapture.s0, returnCapture.s1, returnCapture.s2, (returnCapture.s3).toULong(), if (returnCapture.s4 == 1) (returnCapture.s5).toULong() else null)
    } finally { returnCapture.release() }
}

public fun unsignedOptional(`value`: ULong?, onError: io.prebindgen.covertest.JniErrorHandler<ULong?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): ULong? = run {
    val returnCapture = io.prebindgen.covertest.Return_Int_Long2_ErrorEmpty_62eb913ef507b63c.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { (if (`value` != null) 1 else 0) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { if (`value` != null) requireNotNull(`value`).toLong() else 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_unsigned_optional_7f17c540c43bf08f(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

                if (returnCapture.s0 == 1) {
            }

if (returnCapture.s0 == 1) (returnCapture.s1).toULong() else null
    } finally { returnCapture.release() }
}

public fun blobValueNew(secs: Long, id: ByteArray, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.model.BlobValue> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.model.BlobValue = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long_ByteArray3_ErrorEmpty_aa6ead24a4efe458.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { secs } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { id } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_blob_value_new_34260fb08c70acda(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    io.prebindgen.covertest.model.BlobValue(io.prebindgen.covertest.model.Stamp(returnCapture.s0, returnCapture.s1), returnCapture.s2 as ByteArray)
    } finally { returnCapture.release() }
}

public fun blobValueEcho(`value`: io.prebindgen.covertest.model.BlobValue, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.model.BlobValue> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.model.BlobValue = run {
    val returnCapture = io.prebindgen.covertest.Return_Long_Long_ByteArray3_ErrorEmpty_aa6ead24a4efe458.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { `value`.stamp.secs } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { `value`.stamp.nanos } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { `value`.id } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_blob_value_echo_711e78e87dab6447(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    io.prebindgen.covertest.model.BlobValue(io.prebindgen.covertest.model.Stamp(returnCapture.s0, returnCapture.s1), returnCapture.s2 as ByteArray)
    } finally { returnCapture.release() }
}
