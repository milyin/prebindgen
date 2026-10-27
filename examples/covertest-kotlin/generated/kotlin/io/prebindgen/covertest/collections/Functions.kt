@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.collections

public fun `collectionScalars`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.collections.SequenceScalars>, p1_w1: Double, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.collections.SequenceScalars>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.collections.SequenceScalars> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5.`flag`, p0_w5.`byte`, p0_w5.`short`, p0_w5.`number`, p0_w5.`long`, p0_w5.`float`, p0_w5.`double`, p0_w5.`wide`.toByteArray(), if (p0_w5.`text` != null) requireNotNull(requireNotNull(p0_w5.`text`)) { "invalid option payload" } else null) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call115(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.collections.SequenceScalars>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
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
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>

            if (output_row[8] != null) {
            }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionU128`(p0_w1: kotlin.collections.List<java.math.BigInteger?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<java.math.BigInteger>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<java.math.BigInteger> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.toByteArray()) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call116(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<java.math.BigInteger>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(java.math.BigInteger(1, output_row[0] as ByteArray))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionConstructAll`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w6 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w6?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w6) { "null handle" } else null) }, { if (true) p0_w6!!.markConsumed(); kotlin.Unit })
} else null
val result = try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call117(nativeArgument0, nativeArgument1) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionMakeItems`(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource> = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call118() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>
        var outputitem_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {             try { val owner = outputitem_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
            outputitem_g0_g0_0 = io.prebindgen.covertest.resources.OwnedResource(output_row[0] as Long)

outputlist.add(requireNotNull(outputitem_g0_g0_0))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_g0_g0_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_g0_g0_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `collectionBundle`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>, Long) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call119() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputg0_g0_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputg0_g0_g0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputg0_g0_g0_cleanups = kotlin.collections.mutableListOf()
outputg0_g0_g0_list = kotlin.collections.mutableListOf()
for (outputg0_g0_g0_rawRow in (result[1] as kotlin.Array<*>)) {
val outputg0_g0_g0__row = outputg0_g0_g0_rawRow as kotlin.Array<*>
        var outputg0_g0_g0_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputg0_g0_g0_cleanups.add { outputg0_g0_g0_initialFailure -> var outputg0_g0_g0_failure = outputg0_g0_g0_initialFailure; try {             try { val owner = outputg0_g0_g0_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(outputg0_g0_g0__row[0] as Long) } catch (cleanup: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = cleanup else if (outputg0_g0_g0_failure !== cleanup) outputg0_g0_g0_failure!!.addSuppressed(cleanup) }
 } catch (outputg0_g0_g0_cleanupError: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = outputg0_g0_g0_cleanupError else if (outputg0_g0_g0_failure !== outputg0_g0_g0_cleanupError) outputg0_g0_g0_failure!!.addSuppressed(outputg0_g0_g0_cleanupError) }; outputg0_g0_g0_failure }
            outputg0_g0_g0_item_0 = io.prebindgen.covertest.resources.OwnedResource(outputg0_g0_g0__row[0] as Long)

outputg0_g0_g0_list.add(requireNotNull(outputg0_g0_g0_item_0))
}
            build(requireNotNull(outputg0_g0_g0_list), result[2] as Long)
        } catch (failure: Throwable) {
try {
for (outputg0_g0_g0_cleanupAction in outputg0_g0_g0_cleanups.orEmpty()) { try { outputg0_g0_g0_cleanupAction(failure) } catch (outputg0_g0_g0_cleanupError: Throwable) { if (failure !== outputg0_g0_g0_cleanupError) failure.addSuppressed(outputg0_g0_g0_cleanupError) } }
for (outputg0_g0_g0_index in (outputg0_g0_g0_cleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputg0_g0_g0_failure: kotlin.Throwable? = failure
try { val outputg0_g0_g0__row = (result[1] as kotlin.Array<*>)[outputg0_g0_g0_index] as kotlin.Array<*>
        var outputg0_g0_g0_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputg0_g0_g0_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(outputg0_g0_g0__row[0] as Long) } catch (cleanup: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = cleanup else if (outputg0_g0_g0_failure !== cleanup) outputg0_g0_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg0_g0_g0_cleanupError: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = outputg0_g0_g0_cleanupError else if (outputg0_g0_g0_failure !== outputg0_g0_g0_cleanupError) outputg0_g0_g0_failure!!.addSuppressed(outputg0_g0_g0_cleanupError) }

}
} catch (outputg0_g0_g0_cleanupError: Throwable) { if (failure !== outputg0_g0_g0_cleanupError) failure.addSuppressed(outputg0_g0_g0_cleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionOptionalHandles`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>? = run {
    val p0_w3_elements = if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.map { p0_w7 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w7?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w7) { "null handle" } else null) }, { if (true) p0_w7!!.markConsumed(); kotlin.Unit })
} else null) else (null)
val result = try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (if (p0_w3 != null) (p0_w3_elements.orEmpty().flatMap { it.second() }) else kotlin.collections.emptyList())) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call120(nativeArgument0, nativeArgument1) } } finally { if (ownership[0]) { if (p0_w3 != null) { p0_w3_elements.orEmpty().forEach { it.third() } } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
            if (result[1] != null) {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {             try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
            outputitem_0 = io.prebindgen.covertest.resources.OwnedResource(output_row[0] as Long)

outputlist.add(requireNotNull(outputitem_0))
}
            }
            if (result[1] != null) requireNotNull(outputlist) else null
        } catch (failure: Throwable) {
            if (result[1] != null) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionNestedHandles`(p0_w1: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
val p0_w5_elements = if (true) p0_w5?.toList()?.map { p0_w9 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w9?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w9) { "null handle" } else null) }, { if (true) p0_w9!!.markConsumed(); kotlin.Unit })
} else null
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) }, { (kotlin.collections.emptyList<Nothing>() + (p0_w5_elements.orEmpty().flatMap { it.second() })) }, { p0_w5_elements.orEmpty().forEach { it.third() }; kotlin.Unit })
} else null
val result = try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call121(nativeArgument0, nativeArgument1) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputitem_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try { try {
for (outputitem_cleanupAction in outputitem_cleanups.orEmpty()) { try { outputfailure = outputitem_cleanupAction(outputfailure) } catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) } }
for (outputitem_index in (outputitem_cleanups?.size ?: 0) until (output_row[0] as kotlin.Array<*>).size) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>
        var outputitem_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(outputitem__row[0] as Long) } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

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

outputitem_cleanups.add { outputitem_initialFailure -> var outputitem_failure = outputitem_initialFailure; try {             try { val owner = outputitem_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(outputitem__row[0] as Long) } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }
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
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputitem_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

try {
for (outputitem_cleanupAction in outputitem_cleanups.orEmpty()) { try { outputfailure = outputitem_cleanupAction(outputfailure) } catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) } }
for (outputitem_index in (outputitem_cleanups?.size ?: 0) until (output_row[0] as kotlin.Array<*>).size) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>
        var outputitem_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(outputitem__row[0] as Long) } catch (cleanup: Throwable) { if (outputitem_failure == null) outputitem_failure = cleanup else if (outputitem_failure !== cleanup) outputitem_failure!!.addSuppressed(cleanup) }

} catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }
outputfailure = outputitem_failure
}
} catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <R> `collectionNumbers`(onError: io.prebindgen.covertest.JniErrorHandler<R> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, build: (kotlin.collections.List<Long>, Long) -> R): R = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call122() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputg0_g0_g0_list: kotlin.collections.MutableList<Long>? = null
var outputg0_g0_g0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputg0_g0_g0_cleanups = kotlin.collections.mutableListOf()
outputg0_g0_g0_list = kotlin.collections.mutableListOf()
for (outputg0_g0_g0_rawRow in (result[1] as kotlin.Array<*>)) {
val outputg0_g0_g0__row = outputg0_g0_g0_rawRow as kotlin.Array<*>

outputg0_g0_g0_cleanups.add { outputg0_g0_g0_initialFailure -> var outputg0_g0_g0_failure = outputg0_g0_g0_initialFailure; try {  } catch (outputg0_g0_g0_cleanupError: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = outputg0_g0_g0_cleanupError else if (outputg0_g0_g0_failure !== outputg0_g0_g0_cleanupError) outputg0_g0_g0_failure!!.addSuppressed(outputg0_g0_g0_cleanupError) }; outputg0_g0_g0_failure }

outputg0_g0_g0_list.add(outputg0_g0_g0__row[0] as Long)
}
            build(requireNotNull(outputg0_g0_g0_list), result[2] as Long)
        } catch (failure: Throwable) {
try {
for (outputg0_g0_g0_cleanupAction in outputg0_g0_g0_cleanups.orEmpty()) { try { outputg0_g0_g0_cleanupAction(failure) } catch (outputg0_g0_g0_cleanupError: Throwable) { if (failure !== outputg0_g0_g0_cleanupError) failure.addSuppressed(outputg0_g0_g0_cleanupError) } }
for (outputg0_g0_g0_index in (outputg0_g0_g0_cleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputg0_g0_g0_failure: kotlin.Throwable? = failure
try { val outputg0_g0_g0__row = (result[1] as kotlin.Array<*>)[outputg0_g0_g0_index] as kotlin.Array<*>


} catch (outputg0_g0_g0_cleanupError: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = outputg0_g0_g0_cleanupError else if (outputg0_g0_g0_failure !== outputg0_g0_g0_cleanupError) outputg0_g0_g0_failure!!.addSuppressed(outputg0_g0_g0_cleanupError) }

}
} catch (outputg0_g0_g0_cleanupError: Throwable) { if (failure !== outputg0_g0_g0_cleanupError) failure.addSuppressed(outputg0_g0_g0_cleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionResourceIds`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w6 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w6?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w6) { "null handle" } else null) }, { ; kotlin.Unit })
} else null
val result = try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call123(nativeArgument0) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<Long>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(output_row[0] as Long)
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionI64`(p0_w1: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call124(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<Long>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(output_row[0] as Long)
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionText`(p0_w1: kotlin.collections.List<String?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<String>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<String> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call125(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<String>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(output_row[0] as String)
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionBytes`(p0_w1: kotlin.collections.List<ByteArray?>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<ByteArray>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<ByteArray> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call126(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<ByteArray>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(output_row[0] as ByteArray)
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionNested`(p0_w1: kotlin.collections.List<kotlin.collections.List<Long>>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<Long>>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<Long>> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
val p0_w5_elements = if (true) p0_w5?.toList()?.map { p0_w9 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w9) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call127(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<kotlin.collections.List<Long>>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<Long>? = null
var outputitem_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try { try {
for (outputitem_cleanupAction in outputitem_cleanups.orEmpty()) { try { outputfailure = outputitem_cleanupAction(outputfailure) } catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) } }
for (outputitem_index in (outputitem_cleanups?.size ?: 0) until (output_row[0] as kotlin.Array<*>).size) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>


} catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }
outputfailure = outputitem_failure
}
} catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
outputitem_cleanups = kotlin.collections.mutableListOf()
outputitem_list = kotlin.collections.mutableListOf()
for (outputitem_rawRow in (output_row[0] as kotlin.Array<*>)) {
val outputitem__row = outputitem_rawRow as kotlin.Array<*>

outputitem_cleanups.add { outputitem_initialFailure -> var outputitem_failure = outputitem_initialFailure; try {  } catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }; outputitem_failure }

outputitem_list.add(outputitem__row[0] as Long)
}

outputlist.add(requireNotNull(outputitem_list))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
var outputitem_list: kotlin.collections.MutableList<Long>? = null
var outputitem_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

try {
for (outputitem_cleanupAction in outputitem_cleanups.orEmpty()) { try { outputfailure = outputitem_cleanupAction(outputfailure) } catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) } }
for (outputitem_index in (outputitem_cleanups?.size ?: 0) until (output_row[0] as kotlin.Array<*>).size) {
var outputitem_failure: kotlin.Throwable? = outputfailure
try { val outputitem__row = (output_row[0] as kotlin.Array<*>)[outputitem_index] as kotlin.Array<*>


} catch (outputitem_cleanupError: Throwable) { if (outputitem_failure == null) outputitem_failure = outputitem_cleanupError else if (outputitem_failure !== outputitem_cleanupError) outputitem_failure!!.addSuppressed(outputitem_cleanupError) }
outputfailure = outputitem_failure
}
} catch (outputitem_cleanupError: Throwable) { if (outputfailure == null) outputfailure = outputitem_cleanupError else if (outputfailure !== outputitem_cleanupError) outputfailure!!.addSuppressed(outputitem_cleanupError) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionOptions`(p0_w1: kotlin.collections.List<Long?>, p1_w1: Short, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long?>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long?> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w7 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>((if (p0_w7 != null) 1 else 0), if (p0_w7 != null) requireNotNull(p0_w7) else 0L) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call128(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<Long?>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {             if (output_row[0] == 1) {
            }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
            if (output_row[0] == 1) {
            }

outputlist.add(if (output_row[0] == 1) output_row[1] as Long else null)
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>

            if (output_row[0] == 1) {
            }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionOptional`(p0_w3: kotlin.collections.List<Long>?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long>? = run {
    val p0_w3_elements = if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.map { p0_w7 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w7) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null) else (null)
val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call129(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<Long>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
            if (result[1] != null) {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(output_row[0] as Long)
}
            }
            if (result[1] != null) requireNotNull(outputlist) else null
        } catch (failure: Throwable) {
            if (result[1] != null) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionOptionalItems`(p0_w3: kotlin.collections.List<Long?>?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long?>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long?>? = run {
    val p0_w3_elements = if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.map { p0_w9 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>((if (p0_w9 != null) 1 else 0), if (p0_w9 != null) requireNotNull(p0_w9) else 0L) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null) else (null)
val result = try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call130(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<Long?>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
            if (result[1] != null) {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {             if (output_row[0] == 1) {
            }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
            if (output_row[0] == 1) {
            }

outputlist.add(if (output_row[0] == 1) output_row[1] as Long else null)
}
            }
            if (result[1] != null) requireNotNull(outputlist) else null
        } catch (failure: Throwable) {
            if (result[1] != null) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>

            if (output_row[0] == 1) {
            }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionEnum`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.enums.Code>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.enums.Code>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.enums.Code> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5.wireValue) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call131(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.enums.Code>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(io.prebindgen.covertest.enums.Code.fromWire(output_row[0] as Int))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionProbe`(p0_w1: kotlin.collections.List<Int>, p1_w1: String?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Int>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Int> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call132(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<Int>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(output_row[0] as Int)
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionBorrow`(p0_w1: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call133(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<Long>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(output_row[0] as Long)
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionSlice`(p0_w1: kotlin.collections.List<Long>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call134(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<Long>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(output_row[0] as Long)
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionResourceNew`(p0_w1: Long, onError: io.prebindgen.covertest.JniErrorHandler<io.prebindgen.covertest.resources.OwnedResource> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): io.prebindgen.covertest.resources.OwnedResource = run {
    val result = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call135(nativeArgument0) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> {         var output0: io.prebindgen.covertest.resources.OwnedResource? = null
        try {
            output0 = io.prebindgen.covertest.resources.OwnedResource(result[1] as Long)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(result[1] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionHandles`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, p1_w1: Int, p2_w1: Int, onBindingError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>>): kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w5) { "null handle" } else null) }, { if (true) p0_w5!!.markConsumed(); kotlin.Unit })
} else null
val result = try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p2_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call136(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {             try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
            outputitem_0 = io.prebindgen.covertest.resources.OwnedResource(output_row[0] as Long)

outputlist.add(requireNotNull(outputitem_0))
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        true -> { 
onError.run(result[1] as Long) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionAlias`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, p1_w1: io.prebindgen.covertest.resources.OwnedResource?, onError: io.prebindgen.covertest.JniErrorHandler<Long> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w5) { "null handle" } else null) }, { if (true) p0_w5!!.markConsumed(); kotlin.Unit })
} else null
val result = try { run { val handles = try { (listOfNotNull(if (true) requireNotNull(p1_w1) { "null handle" } else null) + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call137(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { result[1] as Long }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionResult`(p0_w1: kotlin.collections.List<Long>, p1_w1: Boolean, onBindingError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<kotlin.collections.List<Long>>): kotlin.collections.List<Long> = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call138(nativeArgument0, nativeArgument1) } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<Long>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(output_row[0] as Long)
}
            requireNotNull(outputlist)
        } catch (failure: Throwable) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            throw failure
        } }
        true -> { 
onError.run(result[1] as Long) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionFail`(onBindingError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.CollectionErrorHandler<Unit>): Unit = run {
    val result = try { run { ; io.prebindgen.covertest.JNINative.call139() } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        true -> { var outputg0_g0_g0_list: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputg0_g0_g0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
var outputg0_g1_g0_list: kotlin.collections.MutableList<kotlin.collections.List<Long>>? = null
var outputg0_g1_g0_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

var cleanupFailure: Throwable? = null
var callbackFailed = false
try {
outputg0_g0_g0_cleanups = kotlin.collections.mutableListOf()
outputg0_g0_g0_list = kotlin.collections.mutableListOf()
for (outputg0_g0_g0_rawRow in (result[1] as kotlin.Array<*>)) {
val outputg0_g0_g0__row = outputg0_g0_g0_rawRow as kotlin.Array<*>
        var outputg0_g0_g0_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputg0_g0_g0_cleanups.add { outputg0_g0_g0_initialFailure -> var outputg0_g0_g0_failure = outputg0_g0_g0_initialFailure; try {             try { val owner = outputg0_g0_g0_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(outputg0_g0_g0__row[0] as Long) } catch (cleanup: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = cleanup else if (outputg0_g0_g0_failure !== cleanup) outputg0_g0_g0_failure!!.addSuppressed(cleanup) }
 } catch (outputg0_g0_g0_cleanupError: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = outputg0_g0_g0_cleanupError else if (outputg0_g0_g0_failure !== outputg0_g0_g0_cleanupError) outputg0_g0_g0_failure!!.addSuppressed(outputg0_g0_g0_cleanupError) }; outputg0_g0_g0_failure }
            outputg0_g0_g0_item_0 = io.prebindgen.covertest.resources.OwnedResource(outputg0_g0_g0__row[0] as Long)

outputg0_g0_g0_list.add(requireNotNull(outputg0_g0_g0_item_0))
}
outputg0_g1_g0_cleanups = kotlin.collections.mutableListOf()
outputg0_g1_g0_list = kotlin.collections.mutableListOf()
for (outputg0_g1_g0_rawRow in (result[2] as kotlin.Array<*>)) {
val outputg0_g1_g0__row = outputg0_g1_g0_rawRow as kotlin.Array<*>
var outputg0_g1_g0_item_list: kotlin.collections.MutableList<Long>? = null
var outputg0_g1_g0_item_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

outputg0_g1_g0_cleanups.add { outputg0_g1_g0_initialFailure -> var outputg0_g1_g0_failure = outputg0_g1_g0_initialFailure; try { try {
for (outputg0_g1_g0_item_cleanupAction in outputg0_g1_g0_item_cleanups.orEmpty()) { try { outputg0_g1_g0_failure = outputg0_g1_g0_item_cleanupAction(outputg0_g1_g0_failure) } catch (outputg0_g1_g0_item_cleanupError: Throwable) { if (outputg0_g1_g0_failure == null) outputg0_g1_g0_failure = outputg0_g1_g0_item_cleanupError else if (outputg0_g1_g0_failure !== outputg0_g1_g0_item_cleanupError) outputg0_g1_g0_failure!!.addSuppressed(outputg0_g1_g0_item_cleanupError) } }
for (outputg0_g1_g0_item_index in (outputg0_g1_g0_item_cleanups?.size ?: 0) until (outputg0_g1_g0__row[0] as kotlin.Array<*>).size) {
var outputg0_g1_g0_item_failure: kotlin.Throwable? = outputg0_g1_g0_failure
try { val outputg0_g1_g0_item__row = (outputg0_g1_g0__row[0] as kotlin.Array<*>)[outputg0_g1_g0_item_index] as kotlin.Array<*>


} catch (outputg0_g1_g0_item_cleanupError: Throwable) { if (outputg0_g1_g0_item_failure == null) outputg0_g1_g0_item_failure = outputg0_g1_g0_item_cleanupError else if (outputg0_g1_g0_item_failure !== outputg0_g1_g0_item_cleanupError) outputg0_g1_g0_item_failure!!.addSuppressed(outputg0_g1_g0_item_cleanupError) }
outputg0_g1_g0_failure = outputg0_g1_g0_item_failure
}
} catch (outputg0_g1_g0_item_cleanupError: Throwable) { if (outputg0_g1_g0_failure == null) outputg0_g1_g0_failure = outputg0_g1_g0_item_cleanupError else if (outputg0_g1_g0_failure !== outputg0_g1_g0_item_cleanupError) outputg0_g1_g0_failure!!.addSuppressed(outputg0_g1_g0_item_cleanupError) }
 } catch (outputg0_g1_g0_cleanupError: Throwable) { if (outputg0_g1_g0_failure == null) outputg0_g1_g0_failure = outputg0_g1_g0_cleanupError else if (outputg0_g1_g0_failure !== outputg0_g1_g0_cleanupError) outputg0_g1_g0_failure!!.addSuppressed(outputg0_g1_g0_cleanupError) }; outputg0_g1_g0_failure }
outputg0_g1_g0_item_cleanups = kotlin.collections.mutableListOf()
outputg0_g1_g0_item_list = kotlin.collections.mutableListOf()
for (outputg0_g1_g0_item_rawRow in (outputg0_g1_g0__row[0] as kotlin.Array<*>)) {
val outputg0_g1_g0_item__row = outputg0_g1_g0_item_rawRow as kotlin.Array<*>

outputg0_g1_g0_item_cleanups.add { outputg0_g1_g0_item_initialFailure -> var outputg0_g1_g0_item_failure = outputg0_g1_g0_item_initialFailure; try {  } catch (outputg0_g1_g0_item_cleanupError: Throwable) { if (outputg0_g1_g0_item_failure == null) outputg0_g1_g0_item_failure = outputg0_g1_g0_item_cleanupError else if (outputg0_g1_g0_item_failure !== outputg0_g1_g0_item_cleanupError) outputg0_g1_g0_item_failure!!.addSuppressed(outputg0_g1_g0_item_cleanupError) }; outputg0_g1_g0_item_failure }

outputg0_g1_g0_item_list.add(outputg0_g1_g0_item__row[0] as Long)
}

outputg0_g1_g0_list.add(requireNotNull(outputg0_g1_g0_item_list))
}

onError.run(requireNotNull(outputg0_g0_g0_list), requireNotNull(outputg0_g1_g0_list))
} catch (failure: Throwable) { callbackFailed = true; cleanupFailure = failure; throw failure } finally {
try {
for (outputg0_g0_g0_cleanupAction in outputg0_g0_g0_cleanups.orEmpty()) { try { cleanupFailure = outputg0_g0_g0_cleanupAction(cleanupFailure) } catch (outputg0_g0_g0_cleanupError: Throwable) { if (cleanupFailure == null) cleanupFailure = outputg0_g0_g0_cleanupError else if (cleanupFailure !== outputg0_g0_g0_cleanupError) cleanupFailure!!.addSuppressed(outputg0_g0_g0_cleanupError) } }
for (outputg0_g0_g0_index in (outputg0_g0_g0_cleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputg0_g0_g0_failure: kotlin.Throwable? = cleanupFailure
try { val outputg0_g0_g0__row = (result[1] as kotlin.Array<*>)[outputg0_g0_g0_index] as kotlin.Array<*>
        var outputg0_g0_g0_item_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputg0_g0_g0_item_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(outputg0_g0_g0__row[0] as Long) } catch (cleanup: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = cleanup else if (outputg0_g0_g0_failure !== cleanup) outputg0_g0_g0_failure!!.addSuppressed(cleanup) }

} catch (outputg0_g0_g0_cleanupError: Throwable) { if (outputg0_g0_g0_failure == null) outputg0_g0_g0_failure = outputg0_g0_g0_cleanupError else if (outputg0_g0_g0_failure !== outputg0_g0_g0_cleanupError) outputg0_g0_g0_failure!!.addSuppressed(outputg0_g0_g0_cleanupError) }
cleanupFailure = outputg0_g0_g0_failure
}
} catch (outputg0_g0_g0_cleanupError: Throwable) { if (cleanupFailure == null) cleanupFailure = outputg0_g0_g0_cleanupError else if (cleanupFailure !== outputg0_g0_g0_cleanupError) cleanupFailure!!.addSuppressed(outputg0_g0_g0_cleanupError) }
try {
for (outputg0_g1_g0_cleanupAction in outputg0_g1_g0_cleanups.orEmpty()) { try { cleanupFailure = outputg0_g1_g0_cleanupAction(cleanupFailure) } catch (outputg0_g1_g0_cleanupError: Throwable) { if (cleanupFailure == null) cleanupFailure = outputg0_g1_g0_cleanupError else if (cleanupFailure !== outputg0_g1_g0_cleanupError) cleanupFailure!!.addSuppressed(outputg0_g1_g0_cleanupError) } }
for (outputg0_g1_g0_index in (outputg0_g1_g0_cleanups?.size ?: 0) until (result[2] as kotlin.Array<*>).size) {
var outputg0_g1_g0_failure: kotlin.Throwable? = cleanupFailure
try { val outputg0_g1_g0__row = (result[2] as kotlin.Array<*>)[outputg0_g1_g0_index] as kotlin.Array<*>
var outputg0_g1_g0_item_list: kotlin.collections.MutableList<Long>? = null
var outputg0_g1_g0_item_cleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

try {
for (outputg0_g1_g0_item_cleanupAction in outputg0_g1_g0_item_cleanups.orEmpty()) { try { outputg0_g1_g0_failure = outputg0_g1_g0_item_cleanupAction(outputg0_g1_g0_failure) } catch (outputg0_g1_g0_item_cleanupError: Throwable) { if (outputg0_g1_g0_failure == null) outputg0_g1_g0_failure = outputg0_g1_g0_item_cleanupError else if (outputg0_g1_g0_failure !== outputg0_g1_g0_item_cleanupError) outputg0_g1_g0_failure!!.addSuppressed(outputg0_g1_g0_item_cleanupError) } }
for (outputg0_g1_g0_item_index in (outputg0_g1_g0_item_cleanups?.size ?: 0) until (outputg0_g1_g0__row[0] as kotlin.Array<*>).size) {
var outputg0_g1_g0_item_failure: kotlin.Throwable? = outputg0_g1_g0_failure
try { val outputg0_g1_g0_item__row = (outputg0_g1_g0__row[0] as kotlin.Array<*>)[outputg0_g1_g0_item_index] as kotlin.Array<*>


} catch (outputg0_g1_g0_item_cleanupError: Throwable) { if (outputg0_g1_g0_item_failure == null) outputg0_g1_g0_item_failure = outputg0_g1_g0_item_cleanupError else if (outputg0_g1_g0_item_failure !== outputg0_g1_g0_item_cleanupError) outputg0_g1_g0_item_failure!!.addSuppressed(outputg0_g1_g0_item_cleanupError) }
outputg0_g1_g0_failure = outputg0_g1_g0_item_failure
}
} catch (outputg0_g1_g0_item_cleanupError: Throwable) { if (outputg0_g1_g0_failure == null) outputg0_g1_g0_failure = outputg0_g1_g0_item_cleanupError else if (outputg0_g1_g0_failure !== outputg0_g1_g0_item_cleanupError) outputg0_g1_g0_failure!!.addSuppressed(outputg0_g1_g0_item_cleanupError) }

} catch (outputg0_g1_g0_cleanupError: Throwable) { if (outputg0_g1_g0_failure == null) outputg0_g1_g0_failure = outputg0_g1_g0_cleanupError else if (outputg0_g1_g0_failure !== outputg0_g1_g0_cleanupError) outputg0_g1_g0_failure!!.addSuppressed(outputg0_g1_g0_cleanupError) }
cleanupFailure = outputg0_g1_g0_failure
}
} catch (outputg0_g1_g0_cleanupError: Throwable) { if (cleanupFailure == null) cleanupFailure = outputg0_g1_g0_cleanupError else if (cleanupFailure !== outputg0_g1_g0_cleanupError) cleanupFailure!!.addSuppressed(outputg0_g1_g0_cleanupError) }

if (!callbackFailed) cleanupFailure?.let { throw it }
} }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `collectionResultOptional`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, p1_w1: Int, onBindingError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>?>): kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>? = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w5) { "null handle" } else null) }, { if (true) p0_w5!!.markConsumed(); kotlin.Unit })
} else null
val result = try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call140(nativeArgument0, nativeArgument1, nativeArgument2) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
            if (result[1] != null) {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {             try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }
 } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }
            outputitem_0 = io.prebindgen.covertest.resources.OwnedResource(output_row[0] as Long)

outputlist.add(requireNotNull(outputitem_0))
}
            }
            if (result[1] != null) requireNotNull(outputlist) else null
        } catch (failure: Throwable) {
            if (result[1] != null) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>
        var outputitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = outputitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(output_row[0] as Long) } catch (cleanup: Throwable) { if (outputfailure == null) outputfailure = cleanup else if (outputfailure !== cleanup) outputfailure!!.addSuppressed(cleanup) }

} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            }
            throw failure
        } }
        true -> { 
onError.run(result[1] as Long) }
        is String -> { onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `storagePutSlice`(p0_w1: io.prebindgen.covertest.Storage?, p1_w1: kotlin.collections.List<io.prebindgen.covertest.collections.Payload>, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val p1_w1_elements = if (true) p1_w1?.toList()?.map { p1_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p1_w5.`id`, p1_w5.`seq`, p1_w5.`value`, p1_w5.`flag`) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val result = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call141(nativeArgument0, nativeArgument1) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { Unit }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun `storageGetVec`(p0_w1: io.prebindgen.covertest.Storage?, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.collections.Payload>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.collections.Payload>? = run {
    val result = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call142(nativeArgument0) }
    } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.collections.Payload>? = null
var outputcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null
        try {
            if (result[1] != null) {
outputcleanups = kotlin.collections.mutableListOf()
outputlist = kotlin.collections.mutableListOf()
for (outputrawRow in (result[1] as kotlin.Array<*>)) {
val output_row = outputrawRow as kotlin.Array<*>

outputcleanups.add { outputinitialFailure -> var outputfailure = outputinitialFailure; try {  } catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }; outputfailure }

outputlist.add(io.prebindgen.covertest.collections.Payload(output_row[0] as Long, output_row[1] as Int, output_row[2] as Double, output_row[3] as Boolean))
}
            }
            if (result[1] != null) requireNotNull(outputlist) else null
        } catch (failure: Throwable) {
            if (result[1] != null) {
try {
for (outputcleanupAction in outputcleanups.orEmpty()) { try { outputcleanupAction(failure) } catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) } }
for (outputindex in (outputcleanups?.size ?: 0) until (result[1] as kotlin.Array<*>).size) {
var outputfailure: kotlin.Throwable? = failure
try { val output_row = (result[1] as kotlin.Array<*>)[outputindex] as kotlin.Array<*>


} catch (outputcleanupError: Throwable) { if (outputfailure == null) outputfailure = outputcleanupError else if (outputfailure !== outputcleanupError) outputfailure!!.addSuppressed(outputcleanupError) }

}
} catch (outputcleanupError: Throwable) { if (failure !== outputcleanupError) failure.addSuppressed(outputcleanupError) }
            }
            throw failure
        } }
        
        is String -> { onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `collectionScalarsFold`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.collections.SequenceScalars>, p1_w1: Double, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.collections.SequenceScalars) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5.`flag`, p0_w5.`byte`, p0_w5.`short`, p0_w5.`number`, p0_w5.`long`, p0_w5.`float`, p0_w5.`double`, p0_w5.`wide`.toByteArray(), if (p0_w5.`text` != null) requireNotNull(requireNotNull(p0_w5.`text`)) { "invalid option payload" } else null) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink282 { v0, v1, v2, v3, v4, v5, v6, v7, v8 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1, v2, v3, v4, v5, v6, v7, v8)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>

            if (__pb_fold_row[8] != null) {
            }

} }
val result = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call282(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (result[1] != null) {
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
if (result[1] != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>

            if (__pb_fold_row[8] != null) {
            }

}
            throw failure
        } }
        
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `collectionU128Fold`(p0_w1: kotlin.collections.List<java.math.BigInteger?>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, java.math.BigInteger) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.toByteArray()) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink283 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val result = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call283(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

current = fold(current, java.math.BigInteger(1, __pb_fold_row[0] as ByteArray))
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

public fun <A> `collectionOptionalHandlesFold`(p0_w3: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>?, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.resources.OwnedResource) -> A): A? = run {
    val p0_w3_elements = if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.map { p0_w7 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w7?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w7) { "null handle" } else null) }, { if (true) p0_w7!!.markConsumed(); kotlin.Unit })
} else null) else (null)
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink284 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} }
val result = try { try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (if (p0_w3 != null) (p0_w3_elements.orEmpty().flatMap { it.second() }) else kotlin.collections.emptyList())) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call284(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0]) { if (p0_w3 != null) { p0_w3_elements.orEmpty().forEach { it.third() } } } }
    } } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (result[1] != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

__pb_pending = { failure ->             try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
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
if (result[1] != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

}
            throw failure
        } }
        
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `collectionNestedHandlesFold`(p0_w1: kotlin.collections.List<kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource>) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
val p0_w5_elements = if (true) p0_w5?.toList()?.map { p0_w9 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w9?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w9) { "null handle" } else null) }, { if (true) p0_w9!!.markConsumed(); kotlin.Unit })
} else null
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) }, { (kotlin.collections.emptyList<Nothing>() + (p0_w5_elements.orEmpty().flatMap { it.second() })) }, { p0_w5_elements.orEmpty().forEach { it.third() }; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink285 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
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

            try { val owner = foldElementitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(foldElement_row[0] as Long) } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }

} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }

} }
val result = try { try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call285(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (result[1] != null) {
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

            try { val owner = foldElementitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(foldElement_row[0] as Long) } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }

} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }
 }
foldElementcleanups = kotlin.collections.mutableListOf()
foldElementlist = kotlin.collections.mutableListOf()
for (foldElementrawRow in (__pb_fold_row[0] as kotlin.Array<*>)) {
val foldElement_row = foldElementrawRow as kotlin.Array<*>
        var foldElementitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

foldElementcleanups.add { foldElementinitialFailure -> var foldElementfailure = foldElementinitialFailure; try {             try { val owner = foldElementitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(foldElement_row[0] as Long) } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }
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
if (result[1] != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
var foldElementlist: kotlin.collections.MutableList<io.prebindgen.covertest.resources.OwnedResource>? = null
var foldElementcleanups: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null

try {
for (foldElementcleanupAction in foldElementcleanups.orEmpty()) { try { foldElementcleanupAction(failure) } catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) } }
for (foldElementindex in (foldElementcleanups?.size ?: 0) until (__pb_fold_row[0] as kotlin.Array<*>).size) {
var foldElementfailure: kotlin.Throwable? = failure
try { val foldElement_row = (__pb_fold_row[0] as kotlin.Array<*>)[foldElementindex] as kotlin.Array<*>
        var foldElementitem_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElementitem_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(foldElement_row[0] as Long) } catch (cleanup: Throwable) { if (foldElementfailure == null) foldElementfailure = cleanup else if (foldElementfailure !== cleanup) foldElementfailure!!.addSuppressed(cleanup) }

} catch (foldElementcleanupError: Throwable) { if (foldElementfailure == null) foldElementfailure = foldElementcleanupError else if (foldElementfailure !== foldElementcleanupError) foldElementfailure!!.addSuppressed(foldElementcleanupError) }

}
} catch (foldElementcleanupError: Throwable) { if (failure !== foldElementcleanupError) failure.addSuppressed(foldElementcleanupError) }

}
            throw failure
        } }
        
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `collectionI64Fold`(p0_w1: kotlin.collections.List<Long>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Long) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink286 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val result = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call286(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

current = fold(current, __pb_fold_row[0] as Long)
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

public fun <A> `collectionTextFold`(p0_w1: kotlin.collections.List<String?>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, String) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink287 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val result = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call287(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

current = fold(current, __pb_fold_row[0] as String)
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

public fun <A> `collectionBytesFold`(p0_w1: kotlin.collections.List<ByteArray?>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, ByteArray) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink288 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val result = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call288(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

current = fold(current, __pb_fold_row[0] as ByteArray)
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

public fun <A> `collectionNestedFold`(p0_w1: kotlin.collections.List<kotlin.collections.List<Long>>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, kotlin.collections.List<Long>) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
val p0_w5_elements = if (true) p0_w5?.toList()?.map { p0_w9 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w9) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink289 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
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
val result = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call289(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (result[1] != null) {
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
if (result[1] != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
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
        } }
        
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `collectionOptionsFold`(p0_w1: kotlin.collections.List<Long?>, p1_w1: Short, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Long?) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w7 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>((if (p0_w7 != null) 1 else 0), if (p0_w7 != null) requireNotNull(p0_w7) else 0L) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink290 { v0, v1 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>

            if (__pb_fold_row[0] == 1) {
            }

} }
val result = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call290(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (result[1] != null) {
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
if (result[1] != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>

            if (__pb_fold_row[0] == 1) {
            }

}
            throw failure
        } }
        
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `collectionOptionalFold`(p0_w3: kotlin.collections.List<Long>?, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Long) -> A): A? = run {
    val p0_w3_elements = if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.map { p0_w7 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w7) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null) else (null)
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink291 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val result = try { try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call291(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

current = fold(current, __pb_fold_row[0] as Long)
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator
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

public fun <A> `collectionOptionalItemsFold`(p0_w3: kotlin.collections.List<Long?>?, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Long?) -> A): A? = run {
    val p0_w3_elements = if (p0_w3 != null) (if (true) requireNotNull(p0_w3).toList()?.map { p0_w9 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>((if (p0_w9 != null) 1 else 0), if (p0_w9 != null) requireNotNull(p0_w9) else 0L) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null) else (null)
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink292 { v0, v1 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>

            if (__pb_fold_row[0] == 1) {
            }

} }
val result = try { try { run { val nativeArgument0 = try { if (p0_w3 != null) requireNotNull(p0_w3_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>()) { "invalid option payload" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call292(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (result[1] != null) {
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
if (result[1] != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>

            if (__pb_fold_row[0] == 1) {
            }

}
            throw failure
        } }
        
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `collectionEnumFold`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.enums.Code>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.enums.Code) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5.wireValue) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink293 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val result = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call293(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

current = fold(current, io.prebindgen.covertest.enums.Code.fromWire(__pb_fold_row[0] as Int))
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

public fun <A> `collectionBorrowFold`(p0_w1: kotlin.collections.List<Long>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Long) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink294 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val result = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call294(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

current = fold(current, __pb_fold_row[0] as Long)
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

public fun <A> `collectionSliceFold`(p0_w1: kotlin.collections.List<Long>, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Long) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink295 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val result = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call295(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

current = fold(current, __pb_fold_row[0] as Long)
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

public fun <A> `collectionHandlesFold`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, p1_w1: Int, p2_w1: Int, initial: A, onBindingError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<A>, fold: (A, io.prebindgen.covertest.resources.OwnedResource) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w5) { "null handle" } else null) }, { if (true) p0_w5!!.markConsumed(); kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink296 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} }
val result = try { try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p2_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument5 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call296(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4, nativeArgument5) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (result[1] != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

__pb_pending = { failure ->             try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
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
if (result[1] != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

}
            throw failure
        } }
        true -> { 
onError.run(result[1] as Long) }
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `collectionResultFold`(p0_w1: kotlin.collections.List<Long>, p1_w1: Boolean, initial: A, onBindingError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<A>, fold: (A, Long) -> A): A = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5) }, { kotlin.collections.emptyList<Nothing>() }, { ; kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink297 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val result = try { try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call297(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
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

current = fold(current, __pb_fold_row[0] as Long)
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
        true -> { 
onError.run(result[1] as Long) }
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `collectionResultOptionalFold`(p0_w1: kotlin.collections.List<io.prebindgen.covertest.resources.OwnedResource?>, p1_w1: Int, initial: A, onBindingError: io.prebindgen.covertest.JniErrorHandler<A?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<A?>, fold: (A, io.prebindgen.covertest.resources.OwnedResource) -> A): A? = run {
    val p0_w1_elements = if (true) p0_w1?.toList()?.map { p0_w5 ->
kotlin.Triple({ kotlin.arrayOf<kotlin.Any?>(p0_w5?.raw ?: 0L) }, { kotlin.collections.listOfNotNull(if (true) requireNotNull(p0_w5) { "null handle" } else null) }, { if (true) p0_w5!!.markConsumed(); kotlin.Unit })
} else null
val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink298 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} }
val result = try { try { run { val handles = try { (kotlin.collections.emptyList<Nothing>() + (p0_w1_elements.orEmpty().flatMap { it.second() })) } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handles) {
        val ownership = BooleanArray(1)
        try { run { val nativeArgument0 = try { p0_w1_elements?.map { it.first() }?.toTypedArray<kotlin.Any?>() } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { ownership } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument4 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call298(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3, nativeArgument4) } } finally { if (ownership[0]) { p0_w1_elements.orEmpty().forEach { it.third() } } }
    } } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (result[1] != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

__pb_pending = { failure ->             try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
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
if (result[1] != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

}
            throw failure
        } }
        true -> { 
onError.run(result[1] as Long) }
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `sequenceLateDropFold`(initial: A, onBindingError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<A>, fold: (A, io.prebindgen.covertest.resources.OwnedResource) -> A): A = run {
    val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink299 { v0 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} }
val result = try { try { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call299(nativeArgument0, nativeArgument1) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (result[1] != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

__pb_pending = { failure ->             try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
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
if (result[1] != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
        var foldElement0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElement0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[0] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

}
            throw failure
        } }
        true -> { 
onError.run(result[1] as Long) }
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `sequenceScalarsFold`(initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, Boolean, Byte, Short, Int, Long, Float, Double, java.math.BigInteger, String?) -> A): A = run {
    val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink300 { v0, v1, v2, v3, v4, v5, v6, v7, v8 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1, v2, v3, v4, v5, v6, v7, v8)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>

            if (__pb_fold_row[8] != null) {
            }

} }
val result = try { try { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call300(nativeArgument0, nativeArgument1) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    when (result[0]) {
        null -> { var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (result[1] != null) {
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
if (result[1] != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>

            if (__pb_fold_row[8] != null) {
            }

}
            throw failure
        } }
        
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `sequencePartsFold`(p0_w1: Int, initial: A, onBindingError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, onError: io.prebindgen.covertest.ResultFailureHandler<A>, fold: (A, Long, String, io.prebindgen.covertest.resources.OwnedResource) -> A): A = run {
    val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink301 { v0, v1, v2 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1, v2)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElementg0_g2_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElementg0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

} }
val result = try { try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call301(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onBindingError.run(failure.message) }
    when (result[0]) {
        null -> { var __pb_accumulator: A? = null
var __pb_processed = 0
var __pb_pending: ((Throwable) -> Unit)? = null
        try {
if (result[1] != null) {
var current = initial
for (rawRow in __pb_rows) {
val __pb_fold_row = rawRow as kotlin.Array<*>
        var foldElementg0_g2_0: io.prebindgen.covertest.resources.OwnedResource? = null

__pb_pending = { failure ->             try { val owner = foldElementg0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
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
if (result[1] != null) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until __pb_rows.size) {
val __pb_fold_row = __pb_rows[index] as kotlin.Array<*>
        var foldElementg0_g2_0: io.prebindgen.covertest.resources.OwnedResource? = null

            try { val owner = foldElementg0_g2_0; if (owner != null) owner.close() else io.prebindgen.covertest.JNINative.drop20(__pb_fold_row[2] as Long) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }

}
            throw failure
        } }
        true -> { 
onError.run(result[1] as Long) }
        is String -> { if (__pb_outputAccepted[0]) __pb_discard(IllegalArgumentException(result[0] as String)); onBindingError.run(result[0] as String) }
        else -> error("invalid native result channel")
    }
}

public fun <A> `storageGetVecFold`(p0_w1: io.prebindgen.covertest.Storage?, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.collections.Payload) -> A): A? = run {
    val __pb_rows = kotlin.collections.mutableListOf<kotlin.Array<kotlin.Any?>>()
val __pb_sink = io.prebindgen.covertest.JNINative.SequenceSink302 { v0, v1, v2, v3 -> __pb_rows.add(kotlin.arrayOf<kotlin.Any?>(v0, v1, v2, v3)); kotlin.Unit }
val __pb_outputAccepted = BooleanArray(1)
fun __pb_discard(failure: Throwable) { for (rawRow in __pb_rows) { val __pb_fold_row = rawRow as kotlin.Array<*>


} }
val result = try { try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid handle") }; io.prebindgen.covertest.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call302(nativeArgument0, nativeArgument1, nativeArgument2) }
    } } } catch (failure: Throwable) { if (__pb_outputAccepted[0]) __pb_discard(failure); throw failure } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

current = fold(current, io.prebindgen.covertest.collections.Payload(__pb_fold_row[0] as Long, __pb_fold_row[1] as Int, __pb_fold_row[2] as Double, __pb_fold_row[3] as Boolean))
__pb_pending = null
__pb_processed++
}
__pb_accumulator = current
}
            __pb_accumulator
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

