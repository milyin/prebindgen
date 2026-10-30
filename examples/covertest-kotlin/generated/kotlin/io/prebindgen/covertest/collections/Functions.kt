@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.collections

public fun `collectionScalars`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.collections.SequenceScalars>, p1_w1: Double, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.collections.SequenceScalars>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.collections.SequenceScalars> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5.`flag`, p0_w5.`byte`, p0_w5.`short`, p0_w5.`number`, p0_w5.`long`, p0_w5.`float`, p0_w5.`double`, p0_w5.`wide`.toByteArray(), if (p0_w5.`text` != null) requireNotNull(requireNotNull(p0_w5.`text`)) { "invalid option payload" } else null) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_scalars_d9a5730159aaf1ce(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.collections.SequenceScalars>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
 try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>

            if (output_row[8] != null) {
            }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.collections.SequenceScalars>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {             if (output_row[8] != null) {
            }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
            if (output_row[8] != null) {
            }

outputlist.add(io.prebindgen.covertest.collections.SequenceScalars(output_row[0] as Boolean, output_row[1] as Byte, output_row[2] as Short, output_row[3] as Int, output_row[4] as Long, output_row[5] as Float, output_row[6] as Double, java.math.BigInteger(output_row[7] as ByteArray), if (output_row[8] != null) output_row[8] as String else null))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>

            if (output_row[8] != null) {
            }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `collectionU128`(p0_w1: kotlin.collections.List<java.math.BigInteger?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<java.math.BigInteger>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<java.math.BigInteger> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.toByteArray()) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_u128_04e9df44fc750d1f(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<java.math.BigInteger>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
 try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<java.math.BigInteger>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(java.math.BigInteger(1, output_row[0] as ByteArray))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `collectionConstructAll`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w6 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w6?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w6) { "null handle" } else null) }, { if (true) p0_w6!!.markConsumed(); kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_construct_all_2135887528f6e1b7(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `collectionMakeItems`(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_make_items_c6725210ed9bf318(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
 try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>
        var outputitem_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {             try { val owner = outputitem_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
            outputitem_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(output_row[0] as Long)

outputlist.add(requireNotNull(outputitem_g0_g0_0))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `collectionBundle`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>, Long) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_Array_Any___Long2_ErrorEmpty_9ac4dd96dd8aa1a2.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_bundle_f76991cc8c87924d(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputg0_g0_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputg0_g0_g0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
 try {
for (outputg0_g0_g0_cleanupAction in outputg0_g0_g0_cleanups.orEmpty()) { try { outputg0_g0_g0_cleanupAction(failure) } catch (outputg0_g0_g0_cleanupError: Throwable) { if (failure !== outputg0_g0_g0_cleanupError) failure.addSuppressed(outputg0_g0_g0_cleanupError) } }
for (outputg0_g0_g0_index in (outputg0_g0_g0_cleanups?.size ?: 0) until (returnCapture.s0 as kotlin.Array<*>).size) {
var outputg0_g0_g0_failure: kotlin.Throwable? = failure
try { val outputg0_g0_g0__row = (returnCapture.s0 as kotlin.Array<*>)[outputg0_g0_g0_index] as kotlin.Array<*>
        var outputg0_g0_g0_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputg0_g0_g0_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputg0_g0_g0__row[0] as Long) } catch (cleanup: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = cleanup else if (outputg0_g0_g0_failure !== cleanup) outputg0_g0_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg0_g0_g0_cleanupError: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = outputg0_g0_g0_cleanupError else if (outputg0_g0_g0_failure !== outputg0_g0_g0_cleanupError) outputg0_g0_g0_failure!!.addSuppressed(outputg0_g0_g0_cleanupError) }

}
} catch (outputg0_g0_g0_cleanupError: Throwable) { if (failure !== outputg0_g0_g0_cleanupError) failure.addSuppressed(outputg0_g0_g0_cleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputg0_g0_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputg0_g0_g0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputg0_g0_g0_cleanups = kotlin.collections.mutableListOf()
outputg0_g0_g0_list = kotlin.collections.mutableListOf()
for (outputg0_g0_g0_rawRow in (returnCapture.s0 as kotlin.Array<*>)) {
val outputg0_g0_g0__row = outputg0_g0_g0_rawRow as kotlin.Array<*>
        var outputg0_g0_g0_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputg0_g0_g0_cleanups.add { outputg0_g0_g0_initialFailure -> var outputg0_g0_g0_failure = outputg0_g0_g0_initialFailure; try {             try { val owner = outputg0_g0_g0_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputg0_g0_g0__row[0] as Long) } catch (cleanup: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = cleanup else if (outputg0_g0_g0_failure !== cleanup) outputg0_g0_g0_failure!!.addSuppressed(cleanup) }
 } catch (outputg0_g0_g0_cleanupError: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = outputg0_g0_g0_cleanupError else if (outputg0_g0_g0_failure !== outputg0_g0_g0_cleanupError) outputg0_g0_g0_failure!!.addSuppressed(outputg0_g0_g0_cleanupError) }; outputg0_g0_g0_failure }
            outputg0_g0_g0_item_0 = io.prebindgen.covertest.resources.OwnedResource(outputg0_g0_g0__row[0] as Long)

outputg0_g0_g0_list.add(requireNotNull(outputg0_g0_g0_item_0))
}
            build(requireNotNull(outputg0_g0_g0_list), returnCapture.s1)
        } catch (failure: Throwable) {
try {
for (outputg0_g0_g0_cleanupAction in outputg0_g0_g0_cleanups.orEmpty()) { try { outputg0_g0_g0_cleanupAction(failure) } catch (outputg0_g0_g0_cleanupError: Throwable) { if (failure !== outputg0_g0_g0_cleanupError) failure.addSuppressed(outputg0_g0_g0_cleanupError) } }
for (outputg0_g0_g0_index in (outputg0_g0_g0_cleanups?.size ?: 0) until (returnCapture.s0 as kotlin.Array<*>).size) {
var outputg0_g0_g0_failure: kotlin.Throwable? = failure
try { val outputg0_g0_g0__row = (returnCapture.s0 as kotlin.Array<*>)[outputg0_g0_g0_index] as kotlin.Array<*>
        var outputg0_g0_g0_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputg0_g0_g0_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputg0_g0_g0__row[0] as Long) } catch (cleanup: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = cleanup else if (outputg0_g0_g0_failure !== cleanup) outputg0_g0_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg0_g0_g0_cleanupError: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = outputg0_g0_g0_cleanupError else if (outputg0_g0_g0_failure !== outputg0_g0_g0_cleanupError) outputg0_g0_g0_failure!!.addSuppressed(outputg0_g0_g0_cleanupError) }

}
} catch (outputg0_g0_g0_cleanupError: Throwable) { if (failure !== outputg0_g0_g0_cleanupError) failure.addSuppressed(outputg0_g0_g0_cleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `collectionOptionalHandles`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>? = run {
    val p0_w3_elements = if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.map { p0_w7 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w7?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w7) { "null handle" } else null) }, { if (true) p0_w7!!.markConsumed(); kotlin.Unit })
} else null) else (null)
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (if (p0_w3 != null) (p0_w3_elements.orEmpty().flatMap { it.second() }) else kotlin.collections.emptyList())) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_optional_handles_d09dc27d024df24c(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0]) { if (p0_w3 != null) { p0_w3_elements.orEmpty().forEach { it.third() } } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
             if (nativeResult != null) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            }
 }
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
            if (nativeResult != null) {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {             try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
            outputitem_0 = io.prebindgen.covertest.resources.OwnedResource(output_row[0] as Long)

outputlist.add(requireNotNull(outputitem_0))
}
            }
            if (nativeResult != null) requireNotNull(outputlist) else null
        } catch (failure: Throwable) {
            if (nativeResult != null) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `collectionNestedHandles`(p0_w1: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
val p0_w5_elements = if (true) p0_w5?.toList()?.map { p0_w9 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w9?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w9) { "null handle" } else null) }, { if (true) p0_w9!!.markConsumed(); kotlin.Unit })
} else null
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) }, { (kotlin.collections.emptyList<Nothing>() + (p0_w5_elements.orEmpty().flatMap { it.second() })) }, { p0_w5_elements.orEmpty().forEach { it.third() }; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_nested_handles_a1c0a814a0de57b0(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
 try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputitem_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

try {
for (outputitem_cleanupAction in outputitem_cleanups.orEmpty()) { try { outputfailure = outputitem_cleanupAction(outputfailure) } catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) } }
for (outputitem_index in (outputitem_cleanups?.size ?: 0) until (output_row[0] as kotlin.Array<*>).size) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>
        var outputitem_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputitem__row[0] as Long) } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

} catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }
outputfailure = outputitem_failure
}
} catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputitem_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try { try {
for (outputitem_cleanupAction in outputitem_cleanups.orEmpty()) { try { outputfailure = outputitem_cleanupAction(outputfailure) } catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) } }
for (outputitem_index in (outputitem_cleanups?.size ?: 0) until (output_row[0] as kotlin.Array<*>).size) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>
        var outputitem_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputitem__row[0] as Long) } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

} catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }
outputfailure = outputitem_failure
}
} catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
outputitem_cleanups = kotlin.collections.mutableListOf()
outputitem_list = kotlin.collections.mutableListOf()
for (outputitem_rawRow in (output_row[0] as kotlin.Array<*>)) {
val outputitem__row = outputitem_rawRow as kotlin.Array<*>
        var outputitem_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputitem_cleanups.add { outputitem_initialFailure -> var outputitem_failure = outputitem_initialFailure; try {             try { val owner = outputitem_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputitem__row[0] as Long) } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
 } catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }; outputitem_failure }
            outputitem_item_0 = io.prebindgen.covertest.resources.OwnedResource(outputitem__row[0] as Long)

outputitem_list.add(requireNotNull(outputitem_item_0))
}

outputlist.add(requireNotNull(outputitem_list))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputitem_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

try {
for (outputitem_cleanupAction in outputitem_cleanups.orEmpty()) { try { outputfailure = outputitem_cleanupAction(outputfailure) } catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) } }
for (outputitem_index in (outputitem_cleanups?.size ?: 0) until (output_row[0] as kotlin.Array<*>).size) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>
        var outputitem_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputitem__row[0] as Long) } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

} catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }
outputfailure = outputitem_failure
}
} catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <R> `collectionNumbers`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (kotlin.collections.List<Long>, Long) -> R): R = run {
    val returnCapture = io.prebindgen.covertest.Return_ByteArray_Long2_ErrorEmpty_34705c584e211183.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_numbers_58708da43cac559a(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputg0_g0_g0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var outputg0_g0_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputg0_g0_g0_bulkList: kotlin.collections.MutableList<Long>? = null
outputg0_g0_g0_bulkReader = io.prebindgen.covertest.BulkReader(returnCapture.s0 as ByteArray)
outputg0_g0_g0_bulkRange = requireNotNull(outputg0_g0_g0_bulkReader).rows(((requireNotNull(outputg0_g0_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputg0_g0_g0_bulkList = kotlin.collections.ArrayList(outputg0_g0_g0_bulkRange!!.length)
for (outputg0_g0_g0_bulkIndex in 0 until outputg0_g0_g0_bulkRange!!.length) {
val outputg0_g0_g0_bulkBase = outputg0_g0_g0_bulkRange!!.offset + outputg0_g0_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputg0_g0_g0_bulkList!!.add((requireNotNull(outputg0_g0_g0_bulkReader).long((outputg0_g0_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

build(requireNotNull(outputg0_g0_g0_bulkList), returnCapture.s1)
    } finally { returnCapture.release() }
}

public fun `collectionResourceIds`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w6 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w6?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w6) { "null handle" } else null) }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_resource_ids_25588809310a31d6(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `collectionI64`(p0_w1: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_i64_90bde5aeb8c647ea(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `collectionText`(p0_w1: kotlin.collections.List<String?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<String>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<String> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_text_19d59571edb173e9(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<String>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
 try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<String>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(output_row[0] as String)
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `collectionBytes`(p0_w1: kotlin.collections.List<ByteArray?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<ByteArray>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<ByteArray> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_bytes_f8c8c3aaf2aa2be0(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<ByteArray>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
 try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<ByteArray>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(output_row[0] as ByteArray)
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `collectionNested`(p0_w1: kotlin.collections.List<kotlin.collections.List<Long>>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<Long>>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<Long>> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
val p0_w5_elements = if (true) p0_w5?.toList()?.map { p0_w9 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w9) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_nested_4394d3d5a3b0ca8f(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.collections.List<Long>>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r01c263066f8f1a73.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size
var outputbulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkItem_bulkList: kotlin.collections.MutableList<Long>? = null

outputbulkItem_bulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkItem_bulkList = kotlin.collections.ArrayList(outputbulkItem_bulkRange!!.length)
for (outputbulkItem_bulkIndex in 0 until outputbulkItem_bulkRange!!.length) {
val outputbulkItem_bulkBase = outputbulkItem_bulkRange!!.offset + outputbulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkItem_bulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

outputbulkList!!.add(requireNotNull(outputbulkItem_bulkList))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `collectionOptions`(p0_w1: kotlin.collections.List<Long?>, p1_w1: Short, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long?>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long?> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w7 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>((if (p0_w7 != null) 1 else 0), if (p0_w7 != null) requireNotNull(p0_w7) else 0L) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_options_a293ccc475eb588e(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long?>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.ra4ae347219cc8258.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r06ab1b823b7727ce.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r06ab1b823b7727ce.size

            if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r06ab1b823b7727ce.offsets[0])) == 1) {
            }

outputbulkList!!.add(if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r06ab1b823b7727ce.offsets[0])) == 1) (requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r06ab1b823b7727ce.offsets[1])) as Long else null)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `collectionOptional`(p0_w3: kotlin.collections.List<Long>?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long>? = run {
    val p0_w3_elements = if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.map { p0_w7 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w7) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null) else (null)
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_optional_1f78a6a6c734d45b(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.offsets[0])) == 1) {
            }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.offsets[0])) == 1) {
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}
            }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.offsets[0])) == 1) requireNotNull(outputbulkList) else null
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.rc2b72962102e7ef8.offsets[0])) == 1) {
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `collectionOptionalItems`(p0_w3: kotlin.collections.List<Long?>?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long?>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long?>? = run {
    val p0_w3_elements = if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.map { p0_w9 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>((if (p0_w9 != null) 1 else 0), if (p0_w9 != null) requireNotNull(p0_w9) else 0L) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null) else (null)
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_optional_items_a463b73617be6af0(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long?>? = null
 if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r8f12815ce53ffe57.offsets[0])) == 1) {
            }
 }
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long?>? = null
        try {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r8f12815ce53ffe57.offsets[0])) == 1) {
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r8f12815ce53ffe57.offsets[1])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r06ab1b823b7727ce.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r06ab1b823b7727ce.size

            if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r06ab1b823b7727ce.offsets[0])) == 1) {
            }

outputbulkList!!.add(if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r06ab1b823b7727ce.offsets[0])) == 1) (requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r06ab1b823b7727ce.offsets[1])) as Long else null)
}
            }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r8f12815ce53ffe57.offsets[0])) == 1) requireNotNull(outputbulkList) else null
        } catch (failure: Throwable) {
if (outputbulkReader == null) { outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
 }
            if ((requireNotNull(outputbulkReader).int((0) + io.prebindgen.covertest.BulkLayouts.r8f12815ce53ffe57.offsets[0])) == 1) {
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `collectionEnum`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.enums.Code>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.enums.Code>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.enums.Code> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5.wireValue) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_enum_a72cd6089a34be29(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.enums.Code>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
 try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
 }
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.enums.Code>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(io.prebindgen.covertest.enums.Code.fromWire(output_row[0] as Int))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `collectionProbe`(p0_w1: kotlin.collections.List<Int>, p1_w1: String?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Int>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Int> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_probe_cf67d7568b2f663c(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Int>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r5141b4657b63c2b4.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1233d7a0efd1d246.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r1233d7a0efd1d246.size


outputbulkList!!.add((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r1233d7a0efd1d246.offsets[0])) as Int)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `collectionBorrow`(p0_w1: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_borrow_0edafd22349f21ff(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `collectionSlice`(p0_w1: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_slice_00044f8d9833eed8(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `collectionResourceNew`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.OwnedResource> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.OwnedResource = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_resource_new_7274517961a66222(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: io.prebindgen.covertest.resources.OwnedResource? = null
             try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

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

public fun `collectionHandles`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, p1_w1: Int, p2_w1: Int, onBindingError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>>): kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w5) { "null handle" } else null) }, { if (true) p0_w5!!.markConsumed(); kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p2_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_handles_eceeff64e2628ef6(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
 try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
 }
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {             try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
            outputitem_0 = io.prebindgen.covertest.resources.OwnedResource(output_row[0] as Long)

outputlist.add(requireNotNull(outputitem_0))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `collectionAlias`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, p1_w1: io.prebindgen.covertest.resources.OwnedResource?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w5) { "null handle" } else null) }, { if (true) p0_w5!!.markConsumed(); kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handles = try { (listOfNotNull(if (true) requireNotNull(p1_w1) { "null handle" } else null) + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_alias_4512077905b51b90(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `collectionResult`(p0_w1: kotlin.collections.List<Long>, p1_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<kotlin.collections.List<Long>>): kotlin.collections.List<Long> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_result_be11b22c862519eb(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<Long>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputbulkList!!.add((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `collectionFail`(onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.CollectionErrorHandler<Unit>): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorArray_Any___ByteArray2_5ead62d97d876746.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_fail_e960b89bf99875af(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run { var outputg0_g0_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputg0_g0_g0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
var outputg0_g1_g0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var outputg0_g1_g0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputg0_g1_g0_bulkList: kotlin.collections.MutableList<kotlin.collections.List<Long>>? = null

var cleanupFailure: Throwable? = null
var callbackFailed = false
try {
outputg0_g0_g0_cleanups = kotlin.collections.mutableListOf()
outputg0_g0_g0_list = kotlin.collections.mutableListOf()
for (outputg0_g0_g0_rawRow in (returnCapture.d0 as kotlin.Array<*>)) {
val outputg0_g0_g0__row = outputg0_g0_g0_rawRow as kotlin.Array<*>
        var outputg0_g0_g0_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputg0_g0_g0_cleanups.add { outputg0_g0_g0_initialFailure -> var outputg0_g0_g0_failure = outputg0_g0_g0_initialFailure; try {             try { val owner = outputg0_g0_g0_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputg0_g0_g0__row[0] as Long) } catch (cleanup: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = cleanup else if (outputg0_g0_g0_failure !== cleanup) outputg0_g0_g0_failure!!.addSuppressed(cleanup) }
 } catch (outputg0_g0_g0_cleanupError: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = outputg0_g0_g0_cleanupError else if (outputg0_g0_g0_failure !== outputg0_g0_g0_cleanupError) outputg0_g0_g0_failure!!.addSuppressed(outputg0_g0_g0_cleanupError) }; outputg0_g0_g0_failure }
            outputg0_g0_g0_item_0 = io.prebindgen.covertest.resources.OwnedResource(outputg0_g0_g0__row[0] as Long)

outputg0_g0_g0_list.add(requireNotNull(outputg0_g0_g0_item_0))
}
outputg0_g1_g0_bulkReader = io.prebindgen.covertest.BulkReader(returnCapture.d1 as ByteArray)
outputg0_g1_g0_bulkRange = requireNotNull(outputg0_g1_g0_bulkReader).rows(((requireNotNull(outputg0_g1_g0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r01c263066f8f1a73.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size)
outputg0_g1_g0_bulkList = kotlin.collections.ArrayList(outputg0_g1_g0_bulkRange!!.length)
for (outputg0_g1_g0_bulkIndex in 0 until outputg0_g1_g0_bulkRange!!.length) {
val outputg0_g1_g0_bulkBase = outputg0_g1_g0_bulkRange!!.offset + outputg0_g1_g0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size
var outputg0_g1_g0_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var outputg0_g1_g0_bulkItem_bulkList: kotlin.collections.MutableList<Long>? = null

outputg0_g1_g0_bulkItem_bulkRange = requireNotNull(outputg0_g1_g0_bulkReader).rows(((requireNotNull(outputg0_g1_g0_bulkReader).range((outputg0_g1_g0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
outputg0_g1_g0_bulkItem_bulkList = kotlin.collections.ArrayList(outputg0_g1_g0_bulkItem_bulkRange!!.length)
for (outputg0_g1_g0_bulkItem_bulkIndex in 0 until outputg0_g1_g0_bulkItem_bulkRange!!.length) {
val outputg0_g1_g0_bulkItem_bulkBase = outputg0_g1_g0_bulkItem_bulkRange!!.offset + outputg0_g1_g0_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


outputg0_g1_g0_bulkItem_bulkList!!.add((requireNotNull(outputg0_g1_g0_bulkReader).long((outputg0_g1_g0_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

outputg0_g1_g0_bulkList!!.add(requireNotNull(outputg0_g1_g0_bulkItem_bulkList))
}

onError.run(requireNotNull(outputg0_g0_g0_list), requireNotNull(outputg0_g1_g0_bulkList))
} catch (failure: Throwable) { callbackFailed = true; cleanupFailure = failure; throw failure } finally {
try {
for (outputg0_g0_g0_cleanupAction in outputg0_g0_g0_cleanups.orEmpty()) { try { cleanupFailure = outputg0_g0_g0_cleanupAction(cleanupFailure) } catch (outputg0_g0_g0_cleanupError: Throwable) { if (cleanupFailure == null) cleanupFailure = outputg0_g0_g0_cleanupError else if (cleanupFailure !== outputg0_g0_g0_cleanupError) cleanupFailure!!.addSuppressed(outputg0_g0_g0_cleanupError) } }
for (outputg0_g0_g0_index in (outputg0_g0_g0_cleanups?.size ?: 0) until (returnCapture.d0 as kotlin.Array<*>).size) {
var outputg0_g0_g0_failure: kotlin.Throwable? = cleanupFailure
try { val outputg0_g0_g0__row = (returnCapture.d0 as kotlin.Array<*>)[outputg0_g0_g0_index] as kotlin.Array<*>
        var outputg0_g0_g0_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputg0_g0_g0_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(outputg0_g0_g0__row[0] as Long) } catch (cleanup: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = cleanup else if (outputg0_g0_g0_failure !== cleanup) outputg0_g0_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg0_g0_g0_cleanupError: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = outputg0_g0_g0_cleanupError else if (outputg0_g0_g0_failure !== outputg0_g0_g0_cleanupError) outputg0_g0_g0_failure!!.addSuppressed(outputg0_g0_g0_cleanupError) }
cleanupFailure = outputg0_g0_g0_failure
}
} catch (outputg0_g0_g0_cleanupError: Throwable) { if (cleanupFailure == null) cleanupFailure = outputg0_g0_g0_cleanupError else if (cleanupFailure !== outputg0_g0_g0_cleanupError) cleanupFailure!!.addSuppressed(outputg0_g0_g0_cleanupError) }

if (!callbackFailed) cleanupFailure?.let { throw it }
} }
    Unit
    } finally { returnCapture.release() }
}

public fun `collectionResultOptional`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, p1_w1: Int, onBindingError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>?>): kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>? = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w5) { "null handle" } else null) }, { if (true) p0_w5!!.markConsumed(); kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_result_optional_2bf5c4a387d195e5(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
             if (nativeResult != null) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            }
 }
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
            if (nativeResult != null) {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {             try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
            outputitem_0 = io.prebindgen.covertest.resources.OwnedResource(output_row[0] as Long)

outputlist.add(requireNotNull(outputitem_0))
}
            }
            if (nativeResult != null) requireNotNull(outputlist) else null
        } catch (failure: Throwable) {
            if (nativeResult != null) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `storagePutSlice`(p0_w1: io.prebindgen.covertest.Storage?, p1_w1: kotlin.collections.List<io.prebindgen.covertest.collections.Payload>, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val p1_w1_elements = if (true) p1_w1?.toList()?.map { p1_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p1_w5.`id`, p1_w5.`seq`, p1_w5.`value`, p1_w5.`flag`) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_storage_put_slice_0e1853c97ea92d0f(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun `storageGetVec`(p0_w1: io.prebindgen.covertest.Storage?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.collections.Payload>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.collections.Payload>? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_storage_get_vec_1f0f8386b7d2b147(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message); var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.collections.Payload>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
             if (nativeResult != null) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            }
 }
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.collections.Payload>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
            if (nativeResult != null) {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (nativeResult as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(io.prebindgen.covertest.collections.Payload(output_row[0] as Long, output_row[1] as Int, output_row[2] as Double, output_row[3] as Boolean))
}
            }
            if (nativeResult != null) requireNotNull(outputlist) else null
        } catch (failure: Throwable) {
            if (nativeResult != null) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (nativeResult as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionScalarsFold`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.collections.SequenceScalars>, p1_w1: Double, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.collections.SequenceScalars) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5.`flag`, p0_w5.`byte`, p0_w5.`short`, p0_w5.`number`, p0_w5.`long`, p0_w5.`float`, p0_w5.`double`, p0_w5.`wide`.toByteArray(), if (p0_w5.`text` != null) requireNotNull(requireNotNull(p0_w5.`text`)) { "invalid option payload" } else null) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_scalars_fold_70028742f64f0191 { v0, v1, v2, v3, v4, v5, v6, v7, v8 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1, v2, v3, v4, v5, v6, v7, v8)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>

            if (__pb_fold_row[8] != null) {
            }

} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_scalars_fold_3627b263ec4e572b(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->             if (__pb_fold_row[8] != null) {
            }
 }
            if (__pb_fold_row[8] != null) {
            }

current = fold(current, io.prebindgen.covertest.collections.SequenceScalars(__pb_fold_row[0] as Boolean, __pb_fold_row[1] as Byte, __pb_fold_row[2] as Short, __pb_fold_row[3] as Int, __pb_fold_row[4] as Long, __pb_fold_row[5] as Float, __pb_fold_row[6] as Double, java.math.BigInteger(__pb_fold_row[7] as ByteArray), if (__pb_fold_row[8] != null) __pb_fold_row[8] as String else null))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>

            if (__pb_fold_row[8] != null) {
            }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionU128Fold`(p0_w1: kotlin.collections.List<java.math.BigInteger?>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, java.math.BigInteger) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.toByteArray()) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_u128_fold_6f48cddbc993447a { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_u128_fold_5aa4076e5dce00fc(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, java.math.BigInteger(1, __pb_fold_row[0] as ByteArray))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionOptionalHandlesFold`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>?, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.resources.OwnedResource) -> A): A? = run {
    val p0_w3_elements = if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.map { p0_w7 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w7?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w7) { "null handle" } else null) }, { if (true) p0_w7!!.markConsumed(); kotlin.Unit })
} else null) else (null)
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_optional_handles_fold_0322a075b01009ab { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (if (p0_w3 != null) (p0_w3_elements.orEmpty().flatMap { it.second() }) else kotlin.collections.emptyList())) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_optional_handles_fold_f02c8c63b4591395(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0]) { if (p0_w3 != null) { p0_w3_elements.orEmpty().forEach { it.third() } } } }
    } } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

__pb_pending = { failure ->             try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
            foldElement0 = io.prebindgen.covertest.resources.OwnedResource(__pb_fold_row[0] as Long)

current = fold(current, requireNotNull(foldElement0))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionNestedHandlesFold`(p0_w1: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
val p0_w5_elements = if (true) p0_w5?.toList()?.map { p0_w9 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w9?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w9) { "null handle" } else null) }, { if (true) p0_w9!!.markConsumed(); kotlin.Unit })
} else null
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) }, { (kotlin.collections.emptyList<Nothing>() + (p0_w5_elements.orEmpty().flatMap { it.second() })) }, { p0_w5_elements.orEmpty().forEach { it.third() }; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_nested_handles_fold_0692fa392fcf7c3f { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
var foldElementlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var foldElementcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

try {
for (foldElementcleanupAction in foldElementcleanups.orEmpty()) { try { foldElementcleanupAction(failure) } catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) } }
for (foldElementindex in (foldElementcleanups?.size ?: 0) until (__pb_fold_row[0] as kotlin.Array<*>).size) {
var foldElementfailure: kotlin.Throwable? = failure
try { val foldElement_row = (__pb_fold_row[0] as kotlin.Array<*>)[foldElementindex] as kotlin.Array<*>
        var foldElementitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElementitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(foldElement_row[0] as Long) } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }

} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }

} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_nested_handles_fold_b65f6cde439e67ad(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
var foldElementlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var foldElementcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

__pb_pending = { failure -> try {
for (foldElementcleanupAction in foldElementcleanups.orEmpty()) { try { foldElementcleanupAction(failure) } catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) } }
for (foldElementindex in (foldElementcleanups?.size ?: 0) until (__pb_fold_row[0] as kotlin.Array<*>).size) {
var foldElementfailure: kotlin.Throwable? = failure
try { val foldElement_row = (__pb_fold_row[0] as kotlin.Array<*>)[foldElementindex] as kotlin.Array<*>
        var foldElementitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElementitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(foldElement_row[0] as Long) } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }

} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }
 }
foldElementcleanups = kotlin.collections.mutableListOf()
foldElementlist = kotlin.collections.mutableListOf()
for (foldElementrawRow in (__pb_fold_row[0] as kotlin.Array<*>)) {
val foldElement_row = foldElementrawRow as kotlin.Array<*>
        var foldElementitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

foldElementcleanups.add { foldElementinitialFailure -> var foldElementfailure = foldElementinitialFailure; try {             try { val owner = foldElementitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(foldElement_row[0] as Long) } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }
 } catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }; foldElementfailure }
            foldElementitem_0 = io.prebindgen.covertest.resources.OwnedResource(foldElement_row[0] as Long)

foldElementlist.add(requireNotNull(foldElementitem_0))
}

current = fold(current, requireNotNull(foldElementlist))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
var foldElementlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var foldElementcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

try {
for (foldElementcleanupAction in foldElementcleanups.orEmpty()) { try { foldElementcleanupAction(failure) } catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) } }
for (foldElementindex in (foldElementcleanups?.size ?: 0) until (__pb_fold_row[0] as kotlin.Array<*>).size) {
var foldElementfailure: kotlin.Throwable? = failure
try { val foldElement_row = (__pb_fold_row[0] as kotlin.Array<*>)[foldElementindex] as kotlin.Array<*>
        var foldElementitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElementitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(foldElement_row[0] as Long) } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }

} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionI64Fold`(p0_w1: kotlin.collections.List<Long>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Long) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_i64_fold_7481b0c247e22a81 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_i64_fold_808c7d955e03904b(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, __pb_fold_row[0] as Long)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionTextFold`(p0_w1: kotlin.collections.List<String?>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, String) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_text_fold_eee532ef372c2cc4 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_text_fold_a7b24a9c45b3e186(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, __pb_fold_row[0] as String)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionBytesFold`(p0_w1: kotlin.collections.List<ByteArray?>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, ByteArray) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_bytes_fold_6625bb31d7dc18eb { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_bytes_fold_9e9770856c6c4641(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, __pb_fold_row[0] as ByteArray)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionNestedFold`(p0_w1: kotlin.collections.List<kotlin.collections.List<Long>>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, kotlin.collections.List<Long>) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
val p0_w5_elements = if (true) p0_w5?.toList()?.map { p0_w9 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w9) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_nested_fold_63e97a79272db78e { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
var foldElementlist: kotlin.collections.MutableList<Long>? = null
var foldElementcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

try {
for (foldElementcleanupAction in foldElementcleanups.orEmpty()) { try { foldElementcleanupAction(failure) } catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) } }
for (foldElementindex in (foldElementcleanups?.size ?: 0) until (__pb_fold_row[0] as kotlin.Array<*>).size) {
var foldElementfailure: kotlin.Throwable? = failure
try { val foldElement_row = (__pb_fold_row[0] as kotlin.Array<*>)[foldElementindex] as kotlin.Array<*>


} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }

} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_nested_fold_b79bf8a96a5df66c(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
var foldElementlist: kotlin.collections.MutableList<Long>? = null
var foldElementcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

__pb_pending = { failure -> try {
for (foldElementcleanupAction in foldElementcleanups.orEmpty()) { try { foldElementcleanupAction(failure) } catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) } }
for (foldElementindex in (foldElementcleanups?.size ?: 0) until (__pb_fold_row[0] as kotlin.Array<*>).size) {
var foldElementfailure: kotlin.Throwable? = failure
try { val foldElement_row = (__pb_fold_row[0] as kotlin.Array<*>)[foldElementindex] as kotlin.Array<*>


} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }
 }
foldElementcleanups = kotlin.collections.mutableListOf()
foldElementlist = kotlin.collections.mutableListOf()
for (foldElementrawRow in (__pb_fold_row[0] as kotlin.Array<*>)) {
val foldElement_row = foldElementrawRow as kotlin.Array<*>

foldElementcleanups.add { foldElementinitialFailure -> var foldElementfailure = foldElementinitialFailure; try {  } catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }; foldElementfailure }

foldElementlist.add(foldElement_row[0] as Long)
}

current = fold(current, requireNotNull(foldElementlist))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
var foldElementlist: kotlin.collections.MutableList<Long>? = null
var foldElementcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

try {
for (foldElementcleanupAction in foldElementcleanups.orEmpty()) { try { foldElementcleanupAction(failure) } catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) } }
for (foldElementindex in (foldElementcleanups?.size ?: 0) until (__pb_fold_row[0] as kotlin.Array<*>).size) {
var foldElementfailure: kotlin.Throwable? = failure
try { val foldElement_row = (__pb_fold_row[0] as kotlin.Array<*>)[foldElementindex] as kotlin.Array<*>


} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionOptionsFold`(p0_w1: kotlin.collections.List<Long?>, p1_w1: Short, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Long?) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w7 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>((if (p0_w7 != null) 1 else 0), if (p0_w7 != null) requireNotNull(p0_w7) else 0L) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_options_fold_b6fa2828841c6e69 { v0, v1 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>

            if (__pb_fold_row[0] == 1) {
            }

} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_options_fold_ded63bd11a5c595b(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->             if (__pb_fold_row[0] == 1) {
            }
 }
            if (__pb_fold_row[0] == 1) {
            }

current = fold(current, if (__pb_fold_row[0] == 1) __pb_fold_row[1] as Long else null)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>

            if (__pb_fold_row[0] == 1) {
            }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionOptionalFold`(p0_w3: kotlin.collections.List<Long>?, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Long) -> A): A? = run {
    val p0_w3_elements = if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.map { p0_w7 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w7) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null) else (null)
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_optional_fold_78209d4e9082c96a { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_optional_fold_1efcbb0a1a187e64(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, __pb_fold_row[0] as Long)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionOptionalItemsFold`(p0_w3: kotlin.collections.List<Long?>?, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Long?) -> A): A? = run {
    val p0_w3_elements = if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.map { p0_w9 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>((if (p0_w9 != null) 1 else 0), if (p0_w9 != null) requireNotNull(p0_w9) else 0L) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null) else (null)
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_optional_items_fold_0dd26be8936deb87 { v0, v1 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>

            if (__pb_fold_row[0] == 1) {
            }

} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_optional_items_fold_912719d84a3df595(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->             if (__pb_fold_row[0] == 1) {
            }
 }
            if (__pb_fold_row[0] == 1) {
            }

current = fold(current, if (__pb_fold_row[0] == 1) __pb_fold_row[1] as Long else null)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>

            if (__pb_fold_row[0] == 1) {
            }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionEnumFold`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.enums.Code>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.enums.Code) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5.wireValue) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_enum_fold_c4e4e95a0f2dd06c { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_enum_fold_7a76112af7324a8e(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, io.prebindgen.covertest.enums.Code.fromWire(__pb_fold_row[0] as Int))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionBorrowFold`(p0_w1: kotlin.collections.List<Long>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Long) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_borrow_fold_b4856d1bed461622 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_borrow_fold_22d09716b260bda8(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, __pb_fold_row[0] as Long)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionSliceFold`(p0_w1: kotlin.collections.List<Long>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Long) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_slice_fold_2b262532b9b8b02f { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_slice_fold_6f331d2d69f04ead(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, __pb_fold_row[0] as Long)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionHandlesFold`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, p1_w1: Int, p2_w1: Int, initial: A, onBindingError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<A>, fold: (A, io.prebindgen.covertest.resources.OwnedResource) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w5) { "null handle" } else null) }, { if (true) p0_w5!!.markConsumed(); kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_handles_fold_94b94355358878bd { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p2_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument6 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_handles_fold_31a24f87629a2d5f(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5, nativeArgument6) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

__pb_pending = { failure ->             try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
            foldElement0 = io.prebindgen.covertest.resources.OwnedResource(__pb_fold_row[0] as Long)

current = fold(current, requireNotNull(foldElement0))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionResultFold`(p0_w1: kotlin.collections.List<Long>, p1_w1: Boolean, initial: A, onBindingError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<A>, fold: (A, Long) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_result_fold_8dc1da1d06d04f3a { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_result_fold_72fabc4d8e10f6c8(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, __pb_fold_row[0] as Long)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `collectionResultOptionalFold`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, p1_w1: Int, initial: A, onBindingError: io.prebindgen.covertest.JniErrorHandler<A?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<A?>, fold: (A, io.prebindgen.covertest.resources.OwnedResource) -> A): A? = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w5) { "null handle" } else null) }, { if (true) p0_w5!!.markConsumed(); kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_collection_result_optional_fold_522f3e931c5b4b1c { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_collection_result_optional_fold_a6c94691b7ce90de(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

__pb_pending = { failure ->             try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
            foldElement0 = io.prebindgen.covertest.resources.OwnedResource(__pb_fold_row[0] as Long)

current = fold(current, requireNotNull(foldElement0))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `sequenceLateDropFold`(initial: A, onBindingError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<A>, fold: (A, io.prebindgen.covertest.resources.OwnedResource) -> A): A = run {
    val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_sequence_late_drop_fold_c7e3bfa988c8796d { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sequence_late_drop_fold_7e360d29a6fc0d5f(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

__pb_pending = { failure ->             try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
            foldElement0 = io.prebindgen.covertest.resources.OwnedResource(__pb_fold_row[0] as Long)

current = fold(current, requireNotNull(foldElement0))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `sequenceScalarsFold`(initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Boolean, Byte, Short, Int, Long, Float, Double, java.math.BigInteger, String?) -> A): A = run {
    val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_sequence_scalars_fold_47a6ace8a9aad013 { v0, v1, v2, v3, v4, v5, v6, v7, v8 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1, v2, v3, v4, v5, v6, v7, v8)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>

            if (__pb_fold_row[8] != null) {
            }

} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sequence_scalars_fold_5108aab400f755b1(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->             if (__pb_fold_row[8] != null) {
            }
 }
            if (__pb_fold_row[8] != null) {
            }

current = fold(current, __pb_fold_row[0] as Boolean, __pb_fold_row[1] as Byte, __pb_fold_row[2] as Short, __pb_fold_row[3] as Int, __pb_fold_row[4] as Long, __pb_fold_row[5] as Float, __pb_fold_row[6] as Double, java.math.BigInteger(__pb_fold_row[7] as ByteArray), if (__pb_fold_row[8] != null) __pb_fold_row[8] as String else null)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>

            if (__pb_fold_row[8] != null) {
            }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `sequencePartsFold`(p0_w1: Int, initial: A, onBindingError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<A>, fold: (A, Long, String, io.prebindgen.covertest.resources.OwnedResource) -> A): A = run {
    val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_sequence_parts_fold_34d4f8eed2926471 { v0, v1, v2 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1, v2)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElementg0_g2_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElementg0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorLong1_5ff51b1265add9de.acquire()
    try {
val nativeResult = try { try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_sequence_parts_fold_aeb653ed535ad4ab(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onBindingError.run(returnCapture.message) }
    if (returnCapture.status == 3) return@run run {
onError.run(returnCapture.d0) }
    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElementg0_g2_0: io.prebindgen.covertest.resources.OwnedResource? = null

__pb_pending = { failure ->             try { val owner = foldElementg0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
            foldElementg0_g2_0 = io.prebindgen.covertest.resources.OwnedResource(__pb_fold_row[2] as Long)

current = fold(current, __pb_fold_row[0] as Long, __pb_fold_row[1] as String, requireNotNull(foldElementg0_g2_0))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator as A
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
        var foldElementg0_g2_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElementg0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop_OwnedResource_e39aa0af54304c88(__pb_fold_row[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

}
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun <A> `storageGetVecFold`(p0_w1: io.prebindgen.covertest.Storage?, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.collections.Payload) -> A): A? = run {
    val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink_storage_get_vec_fold_886389afd9ad2e6a { v0, v1, v2, v3 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1, v2, v3)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_storage_get_vec_fold_5d94b3f03ff97a04(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) }
    } } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(returnCapture.message));
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (nativeResult != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>

__pb_pending = { failure ->  }

current = fold(current, io.prebindgen.covertest.collections.Payload(__pb_fold_row[0] as Long, __pb_fold_row[1] as Int, __pb_fold_row[2] as Double, __pb_fold_row[3] as Boolean))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator
        } catch (failure: Throwable) {
__pb_pending?.invoke(failure)
if (nativeResult != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>


}
            throw failure
        }
    } finally { returnCapture.release() }
}
