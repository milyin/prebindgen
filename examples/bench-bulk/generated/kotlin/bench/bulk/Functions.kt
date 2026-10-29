@file:Suppress("UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION")
package bench.bulk

public fun `scalar`(p0_w3: Long, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.ra60d3960993b9a77.size)
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.ra60d3960993b9a77.offsets[0], p0_w3)

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_scalar_a4a5da3f79ebbcaf(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)

(requireNotNull(outputbulkReader).long((0) + bench.bulk.BulkLayouts.ra60d3960993b9a77.offsets[0])) as Long
    } finally { returnCapture.release() }
}

public fun `rowSum`(p0_w3: bench.bulk.Row, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.r789cd54eb117685f.size)
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0], p0_w3.`a`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1], p0_w3.`b`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2], p0_w3.`c`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3], p0_w3.`d`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4], p0_w3.`e`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5], p0_w3.`f`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6], p0_w3.`g`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7], p0_w3.`h`)

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_row_sum_db65517187cece4c(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `numbersSum`(p0_w3: kotlin.collections.List<Long>, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = if (true) p0_w3?.toList() else null
val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.r1800d3dd3eceec6d.size)
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = p0_w3_elements
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + bench.bulk.BulkLayouts.r1800d3dd3eceec6d.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, bench.bulk.BulkLayouts.ra60d3960993b9a77)
p0_w1_writer.range((0) + bench.bulk.BulkLayouts.r1800d3dd3eceec6d.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.ra60d3960993b9a77.size) + bench.bulk.BulkLayouts.ra60d3960993b9a77.offsets[0], p0_w7)
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_numbers_sum_47543406a5cdf588(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `rowsSum`(p0_w3: kotlin.collections.List<bench.bulk.Row>, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w3_elements = if (true) p0_w3?.toList() else null
val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.rf4d5524a08b9867b.size)
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = p0_w3_elements
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, bench.bulk.BulkLayouts.r789cd54eb117685f)
p0_w1_writer.range((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0], p0_w7.`a`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1], p0_w7.`b`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2], p0_w7.`c`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3], p0_w7.`d`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4], p0_w7.`e`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5], p0_w7.`f`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6], p0_w7.`g`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7], p0_w7.`h`)
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_rows_sum_d4a5ecfa43bf26b9(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `rowsMake`(p0_w1: Long, p1_w1: Long, onError: bench.bulk.JniErrorHandler<kotlin.collections.List<bench.bulk.Row>> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.bulk.Row> = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_rows_make_6f7eb2a3ee93e0ae(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.Row>? = null
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r789cd54eb117685f.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size


outputbulkList!!.add(bench.bulk.Row((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7])) as Long))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `rowsForReduce`(p0_w1: Long, p1_w1: Long, onError: bench.bulk.JniErrorHandler<kotlin.collections.List<bench.bulk.Row>> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.bulk.Row> = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { p0_w1 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_rows_for_reduce_3c2e82abbc965f1c(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.Row>? = null
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r789cd54eb117685f.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size


outputbulkList!!.add(bench.bulk.Row((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7])) as Long))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `rowsEcho`(p0_w3: kotlin.collections.List<bench.bulk.Row>, onError: bench.bulk.JniErrorHandler<kotlin.collections.List<bench.bulk.Row>> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): kotlin.collections.List<bench.bulk.Row> = run {
    val p0_w3_elements = if (true) p0_w3?.toList() else null
val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.rf4d5524a08b9867b.size)
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
val p0_w3_bulkList = p0_w3_elements
if (p0_w3_bulkList == null) { p0_w1_writer.absent((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0]) } else {
val p0_w3_bulkStart = p0_w1_writer.rows(p0_w3_bulkList.size, bench.bulk.BulkLayouts.r789cd54eb117685f)
p0_w1_writer.range((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0], p0_w3_bulkStart, p0_w3_bulkList.size)
p0_w3_bulkList.forEachIndexed { p0_w3_bulkIndex, p0_w7 ->
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0], p0_w7.`a`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1], p0_w7.`b`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2], p0_w7.`c`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3], p0_w7.`d`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4], p0_w7.`e`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5], p0_w7.`f`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6], p0_w7.`g`)
p0_w1_writer.long((p0_w3_bulkStart + p0_w3_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7], p0_w7.`h`)
}
}

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_rows_echo_5c26c8cf4751c454(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    var outputbulkReader: bench.bulk.BulkReader? = null
var outputbulkRange: bench.bulk.BulkRange? = null
var outputbulkList: kotlin.collections.MutableList<bench.bulk.Row>? = null
outputbulkReader = bench.bulk.BulkReader(nativeResult as ByteArray)
outputbulkRange = requireNotNull(outputbulkReader).rows(((requireNotNull(outputbulkReader).range((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r789cd54eb117685f.size)
outputbulkList = kotlin.collections.ArrayList(outputbulkRange!!.length)
for (outputbulkIndex in 0 until outputbulkRange!!.length) {
val outputbulkBase = outputbulkRange!!.offset + outputbulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size


outputbulkList!!.add(bench.bulk.Row((requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6])) as Long, (requireNotNull(outputbulkReader).long((outputbulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7])) as Long))
}

requireNotNull(outputbulkList)
    } finally { returnCapture.release() }
}

public fun `packetSum`(p0_w3: bench.bulk.Packet, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val p0_w1_writer = bench.bulk.BulkWriter(bench.bulk.BulkLayouts.re5733c882bf9c317.size)
val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { run {
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[0], p0_w3.`header`.`a`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[1], p0_w3.`header`.`b`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[2], p0_w3.`header`.`c`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[3], p0_w3.`header`.`d`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[4], p0_w3.`header`.`e`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[5], p0_w3.`header`.`f`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[6], p0_w3.`header`.`g`)
p0_w1_writer.long((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[7], p0_w3.`header`.`h`)
p0_w1_writer.bytes((0) + bench.bulk.BulkLayouts.re5733c882bf9c317.offsets[8], p0_w3.`bytes`)

p0_w1_writer.array } } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p0_w1_writer.length } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_packet_sum_c07ede9eb5de050f(nativeArgument0, nativeArgument1, nativeArgument2) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}

public fun `handlerNew`(p0_callback: bench.bulk.RowsCallback?, onError: bench.bulk.JniErrorHandler<bench.bulk.Handler> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): bench.bulk.Handler = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val nativeArgument0 = try { if (p0_callback == null) null else object {
    @Suppress("UNUSED_PARAMETER", "UNUSED_VARIABLE", "UNCHECKED_CAST", "UNNECESSARY_NOT_NULL_ASSERTION", "USELESS_CAST", "VARIABLE_WITH_REDUNDANT_INITIALIZER")
    fun run(raw0: ByteArray?): Long {
        var callbackCleanupFailure: Throwable? = null
var callbackOutput0_bulkReader: bench.bulk.BulkReader? = null
var callbackOutput0_bulkRange: bench.bulk.BulkRange? = null
var callbackOutput0_bulkList: kotlin.collections.MutableList<bench.bulk.Row>? = null
        lateinit var callbackArgument0_0: kotlin.collections.List<bench.bulk.Row>
        var callbackReturned: Long = 0L
        try {
callbackOutput0_bulkReader = bench.bulk.BulkReader(raw0 as ByteArray)
callbackOutput0_bulkRange = requireNotNull(callbackOutput0_bulkReader).rows(((requireNotNull(callbackOutput0_bulkReader).range((0) + bench.bulk.BulkLayouts.rf4d5524a08b9867b.offsets[0])) as bench.bulk.BulkRange?), bench.bulk.BulkLayouts.r789cd54eb117685f.size)
callbackOutput0_bulkList = kotlin.collections.ArrayList(callbackOutput0_bulkRange!!.length)
for (callbackOutput0_bulkIndex in 0 until callbackOutput0_bulkRange!!.length) {
val callbackOutput0_bulkBase = callbackOutput0_bulkRange!!.offset + callbackOutput0_bulkIndex * bench.bulk.BulkLayouts.r789cd54eb117685f.size


callbackOutput0_bulkList!!.add(bench.bulk.Row((requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[0])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[1])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[2])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[3])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[4])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[5])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[6])) as Long, (requireNotNull(callbackOutput0_bulkReader).long((callbackOutput0_bulkBase) + bench.bulk.BulkLayouts.r789cd54eb117685f.offsets[7])) as Long))
}
            callbackArgument0_0 = requireNotNull(callbackOutput0_bulkList)
        } catch (failure: Throwable) {
            callbackCleanupFailure?.let { if (failure !== it) failure.addSuppressed(it) }
            throw failure
        }
        callbackReturned = p0_callback.run(callbackArgument0_0)
return kotlin.run { val callback_return_w1 = callbackReturned
  val callbackReturnWire = callback_return_w1

callbackReturnWire }
    }
} } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_handler_new_916a49ec8343b596(nativeArgument0, nativeArgument1) } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) { if (returnCapture.discardSuccess) { val failure = IllegalArgumentException(returnCapture.message);         var output0: bench.bulk.Handler? = null
             try { val owner = output0; if (owner != null) owner.close() else bench.bulk.JNINative.drop_Handler_bb814c4111561dda(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
 }
        return@run onError.run(returnCapture.message) }

            var output0: bench.bulk.Handler? = null
        try {
            output0 = bench.bulk.Handler(nativeResult)
            requireNotNull(output0)
        } catch (failure: Throwable) {
            try { val owner = output0; if (owner != null) owner.close() else bench.bulk.JNINative.drop_Handler_bb814c4111561dda(nativeResult) } catch (cleanup: Throwable) { if (failure !== cleanup) failure.addSuppressed(cleanup) }
            throw failure
        }
    } finally { returnCapture.release() }
}

public fun `fire`(p0_w1: bench.bulk.Handler?, p1_w1: Long, p2_w1: Long, onError: bench.bulk.JniErrorHandler<Long> = bench.bulk.JniErrorHandler { message -> throw IllegalArgumentException(message) }): Long = run {
    val returnCapture = bench.bulk.Return_Empty_ErrorEmpty_a7ee757e32992e01.acquire()
    try {
val nativeResult = try { run { val handle0 = try { if (true) requireNotNull(p0_w1) { "null handle" } else null } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid handle") }; bench.bulk.withHandleLocks(handle0) {
        run { val nativeArgument0 = try { p0_w1?.raw ?: 0L } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument1 = try { p1_w1 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument2 = try { p2_w1 } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; val nativeArgument3 = try { returnCapture } catch (failure: IllegalArgumentException) { throw bench.bulk.HandleValidationFailure(failure.message ?: "invalid input") }; bench.bulk.JNINative.call_fire_6fdab6ebdaeffb11(nativeArgument0, nativeArgument1, nativeArgument2, nativeArgument3) }
    } } } catch (failure: bench.bulk.HandleValidationFailure) { return@run onError.run(failure.message) }
    if (returnCapture.status == 2) {
        return@run onError.run(returnCapture.message) }

    nativeResult
    } finally { returnCapture.release() }
}
