@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package io.prebindgen.covertest.defaults

public fun defaultBulkEcho(values: kotlin.collections.List<io.prebindgen.covertest.defaults.DefaultBulkRow>, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.defaults.DefaultBulkRow>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.defaults.DefaultBulkRow> = run {
    val p0_w1_elements = try { if (true) values?.toList()?.let { p0_w1_snapshot -> kotlin.Array<kotlin.Any?>(p0_w1_snapshot.size) { p0_w1_index -> val p0_w5 = p0_w1_snapshot[p0_w1_index]
kotlin.arrayOf<kotlin.Any?>(io.prebindgen.covertest.InputValidation.value { p0_w5.number }, io.prebindgen.covertest.InputValidation.value { p0_w5.fraction }) } } else null } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { p0_w1_elements } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_echo_19271b2932c312bb(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.defaults.DefaultBulkRow>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rd59a56236bfd5111.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.size


outputbulkList!!.add(io.prebindgen.covertest.defaults.DefaultBulkRow((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.offsets[0])) as Long, (requireNotNull(outputbulkReader).double((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.offsets[1])) as Double))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun defaultBulkArray(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.defaults.DefaultBulkRow>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.defaults.DefaultBulkRow> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_array_f969bde7cc1c432a(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.defaults.DefaultBulkRow>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rd59a56236bfd5111.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.size


outputbulkList!!.add(io.prebindgen.covertest.defaults.DefaultBulkRow((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.offsets[0])) as Long, (requireNotNull(outputbulkReader).double((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.offsets[1])) as Double))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun defaultBulkOptional(mode: Long, onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.defaults.DefaultBulkRow>?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.defaults.DefaultBulkRow>? = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { mode } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_optional_ea20da6fc813acee(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.defaults.DefaultBulkRow>? = null
            if (nativeResult != null) {
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rd59a56236bfd5111.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.size


outputbulkList!!.add(io.prebindgen.covertest.defaults.DefaultBulkRow((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.offsets[0])) as Long, (requireNotNull(outputbulkReader).double((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.offsets[1])) as Double))
}
            }

if (nativeResult != null) requireNotNull(outputbulkList) else null
    } finally { returnCapture.release() }
}

public fun defaultBulkNested(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.collections.List<Long>>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.collections.List<Long>> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_nested_799f9a1fa2a2a9a1(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

public fun defaultBulkBorrowed(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.defaults.DefaultBulkRow>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.defaults.DefaultBulkRow> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_borrowed_7528ee78c961424e(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<io.prebindgen.covertest.defaults.DefaultBulkRow>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rd59a56236bfd5111.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.size


outputbulkList!!.add(io.prebindgen.covertest.defaults.DefaultBulkRow((requireNotNull(outputbulkReader).long((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.offsets[0])) as Long, (requireNotNull(outputbulkReader).double((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.offsets[1])) as Double))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun defaultBulkNumbers(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<Long>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<Long> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_numbers_677f84d2a60bc3b3(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
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

public fun defaultBulkUnits(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<kotlin.Unit>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<kotlin.Unit> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_units_12bee997ef805cee(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<kotlin.Unit>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rf41b5b157c049db1.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r904fefb3d01cb2ae.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r904fefb3d01cb2ae.size


outputbulkList!!.add(kotlin.Unit)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun defaultBulkTexts(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<String>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<String> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_texts_db6a5274075ed7aa(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<String>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r79427dd643a1b061.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.size


outputbulkList!!.add((requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.r963ae3ba49e0cb2b.offsets[0])) as String)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun defaultBulkOptionalTexts(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<String?>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<String?> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_optional_texts_53f1ccde499e0f96(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<String?>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rc248412fa02aa6e4.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.size

            if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[0])) == 1) {
            }

outputbulkList!!.add(if ((requireNotNull(outputbulkReader).int((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[0])) == 1) (requireNotNull(outputbulkReader).text((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.re5f8866547bdf41a.offsets[1])) as String else null)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun defaultBulkBuffers(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<ByteArray>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<ByteArray> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_buffers_c8800191206cf047(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: io.prebindgen.covertest.BulkReader? = null
var outputbulkRange: io.prebindgen.covertest.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<ByteArray>? = null
outputbulkReader = io.prebindgen.covertest.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rba88ec47a5d9bc85.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.size


outputbulkList!!.add((requireNotNull(outputbulkReader).bytes((outputbulkBase) + io.prebindgen.covertest.BulkLayouts.re048ef6766326a01.offsets[0])) as ByteArray)
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun defaultBulkPlain(onError: io.prebindgen.covertest.JniErrorHandler<kotlin.collections.List<io.prebindgen.covertest.defaults.DefaultBulkRow>> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<io.prebindgen.covertest.defaults.DefaultBulkRow> = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_plain_e53a41329b22bbda(nativeArgument0) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputlist: kotlin.collections.MutableList<io.prebindgen.covertest.defaults.DefaultBulkRow>? = null
outputlist = kotlin.collections.ArrayList((nativeResult as kotlin.Array<*>).size)
for (outputindex in (nativeResult as kotlin.Array<*>).indices) {
val output_row = (nativeResult as kotlin.Array<*>)[outputindex] as io.prebindgen.covertest.JNINative.SequenceRow_Long_Double_2_012db11776ed72c5


outputlist.add(io.prebindgen.covertest.defaults.DefaultBulkRow(output_row.c0, output_row.c1))
}

requireNotNull(outputlist)
    } finally { returnCapture.release() }
}

public fun defaultBulkCallback(callback: io.prebindgen.covertest.DefaultBulkRowVecCallback, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput0_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<io.prebindgen.covertest.defaults.DefaultBulkRow>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<io.prebindgen.covertest.defaults.DefaultBulkRow>
        try {
callbackOutput0_bulkReader = io.prebindgen.covertest.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.rd59a56236bfd5111.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.size


callbackOutput0_bulkList!!.add(io.prebindgen.covertest.defaults.DefaultBulkRow((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.offsets[0])) as Long, (requireNotNull(callbackOutput0_bulkReader).double((callbackOutput0_bulkBase) + io.prebindgen.covertest.BulkLayouts.r1962afe2ac010549.offsets[1])) as Double))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callback.run(callbackArgument0_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_callback_af0d489a7a6a5660(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun defaultBulkCallbackMixed(callback: io.prebindgen.covertest.DefaultBulkMixedCallback, onError: io.prebindgen.covertest.JniErrorHandler<Unit> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Unit = run {
    val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: Array<Any?>?, raw1: ByteArray?) {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_list: kotlin.collections.MutableList<kotlin.collections.List<Long>>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<kotlin.collections.List<Long>>
var callbackOutput1_bulkReader: io.prebindgen.covertest.BulkReader? = null
var callbackOutput1_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput1_bulkList: kotlin.collections.MutableList<kotlin.collections.List<Long>>? = null
        lateinit var callbackArgument1_0: kotlin.collections.List<kotlin.collections.List<Long>>
        try {
callbackOutput0_list = kotlin.collections.ArrayList((raw0 as kotlin.Array<*>).size)
for (callbackOutput0_index in (raw0 as kotlin.Array<*>).indices) {
val callbackOutput0__row = (raw0 as kotlin.Array<*>)[callbackOutput0_index] as io.prebindgen.covertest.JNINative.SequenceRow_Array_Any___1_00f39593b4ea53ea
var callbackOutput0_item_list: kotlin.collections.MutableList<Long>? = null

callbackOutput0_item_list = kotlin.collections.ArrayList((callbackOutput0__row.c0 as kotlin.Array<*>).size)
for (callbackOutput0_item_index in (callbackOutput0__row.c0 as kotlin.Array<*>).indices) {
val callbackOutput0_item__row = (callbackOutput0__row.c0 as kotlin.Array<*>)[callbackOutput0_item_index] as io.prebindgen.covertest.JNINative.SequenceRow_Long_1_127a3c24b600dbe0


callbackOutput0_item_list.add(callbackOutput0_item__row.c0)
}

callbackOutput0_list.add(requireNotNull(callbackOutput0_item_list))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_list)
callbackOutput1_bulkReader = io.prebindgen.covertest.BulkReader(raw1 as ByteArray)
callbackOutput1_bulkRange = requireNotNull(callbackOutput1_bulkReader).rows(((requireNotNull(callbackOutput1_bulkReader).range((0) + io.prebindgen.covertest.BulkLayouts.r01c263066f8f1a73.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size)
callbackOutput1_bulkList = kotlin.collections.ArrayList(callbackOutput1_bulkRange!!.length)
for (callbackOutput1_bulkIndex in 0 until callbackOutput1_bulkRange!!.length) {
val callbackOutput1_bulkBase = callbackOutput1_bulkRange!!.offset + callbackOutput1_bulkIndex * io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.size
var callbackOutput1_bulkItem_bulkRange: io.prebindgen.covertest.BulkRange? = null
var callbackOutput1_bulkItem_bulkList: kotlin.collections.MutableList<Long>? = null

callbackOutput1_bulkItem_bulkRange = requireNotNull(callbackOutput1_bulkReader).rows(((requireNotNull(callbackOutput1_bulkReader).range((callbackOutput1_bulkBase) + io.prebindgen.covertest.BulkLayouts.r1800d3dd3eceec6d.offsets[0])) as io.prebindgen.covertest.BulkRange?), io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size)
callbackOutput1_bulkItem_bulkList = kotlin.collections.ArrayList(callbackOutput1_bulkItem_bulkRange!!.length)
for (callbackOutput1_bulkItem_bulkIndex in 0 until callbackOutput1_bulkItem_bulkRange!!.length) {
val callbackOutput1_bulkItem_bulkBase = callbackOutput1_bulkItem_bulkRange!!.offset + callbackOutput1_bulkItem_bulkIndex * io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.size


callbackOutput1_bulkItem_bulkList!!.add((requireNotNull(callbackOutput1_bulkReader).long((callbackOutput1_bulkItem_bulkBase) + io.prebindgen.covertest.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long)
}

callbackOutput1_bulkList!!.add(requireNotNull(callbackOutput1_bulkItem_bulkList))
}
            callbackArgument1_0 = requireNotNull(callbackOutput1_bulkList)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callback.run(callbackArgument0_0, callbackArgument1_0)

    }
} } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_callback_mixed_d29393bf89d94532(nativeArgument0, nativeArgument1) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    Unit
    } finally { returnCapture.release() }
}

public fun <A> defaultBulkArrayFold(initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.defaults.DefaultBulkRow) -> A): A = run {
    val __pb_sink = io.prebindgen.covertest.JNINative.SequenceCollector_default_bulk_array_fold_385d020277360226()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_array_fold_8b682a05955403df(nativeArgument0, nativeArgument1, nativeArgument2) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
if (nativeResult == 0) {
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]


current = fold(current, io.prebindgen.covertest.defaults.DefaultBulkRow(__pb_fold_row.c0, __pb_fold_row.c1))
}
__pb_accumulator = current
}

__pb_accumulator as A
    } finally { returnCapture.release() }
}

public fun <A> defaultBulkOptionalFold(mode: Long, initial: A, onError: io.prebindgen.covertest.JniErrorHandler<A?> = io.prebindgen.covertest.JniErrorHandler { message -> throw IllegalArgumentException(message) }, fold: (A, io.prebindgen.covertest.defaults.DefaultBulkRow) -> A): A? = run {
    val __pb_sink = io.prebindgen.covertest.JNINative.SequenceCollector_default_bulk_optional_fold_514df038c7a4b1ee()
val __pb_rows = __pb_sink.rows
val __pb_outputAccepted = BooleanArray(1)
val returnCapture = io.prebindgen.covertest.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { run { val nativeArgument0 = try { mode } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { __pb_sink } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { __pb_outputAccepted } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw io.prebindgen.covertest.HandleValidationFailure(failure.message ?: "invalid input") }; io.prebindgen.covertest.JNINative.call_default_bulk_optional_fold_a9233ea21645798f(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) } } } catch (failure: io.prebindgen.covertest.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var __pb_accumulator: A? = null
if (nativeResult == 0) {
var current = initial
for (__pb_rowIndex in 0 until __pb_rows.size) {
val __pb_fold_row = __pb_rows[__pb_rowIndex]


current = fold(current, io.prebindgen.covertest.defaults.DefaultBulkRow(__pb_fold_row.c0, __pb_fold_row.c1))
}
__pb_accumulator = current
}

__pb_accumulator
    } finally { returnCapture.release() }
}
