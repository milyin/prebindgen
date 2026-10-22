package io.prebindgen.covertest

import io.prebindgen.covertest.options.*
import io.prebindgen.covertest.resources.ownedResourceDrops

fun checkOptions() {
    for (v in listOf(-1L, -2L, -3L)) {
        try { optionNestedU32(true, v) { _, _ -> Unit }; error("invalid nested u32 accepted") }
        catch (_: IllegalArgumentException) {}
    }
    check(optionNestedU32(false, 99) { present, v -> !present && v == null })
    check(optionNestedU32(true, null) { present, v -> present && v == null })
    check(optionNestedU32(true, 4294967295L) { present, v -> present && v == 4294967295L })
    check(nicheChoiceId(false, 0, -2, null) == -2L)
    check(nicheChoiceId(true, 0, null, null) == -1L)
    check(nicheChoiceId(true, 0, 65, null) == 65L)
    val nicheHandle = nicheChoiceNew(66)
    check(nicheChoiceId(true, 1, -2, nicheHandle) == 66L && nicheHandle.isClosed)
    optionNestedProjectionViews(false, false) { present, a, b -> check(!present && a == null && b == null) }
    optionNestedProjectionViews(true, false) { present, a, b -> check(present && a == null && b == null) }
    optionNestedProjectionViews(true, true) { present, a, b -> check(present); requireNotNull(a).close(); requireNotNull(b).close() }

    for (v in listOf(null, "", "héllo\u0000世界")) check(io.prebindgen.covertest.model.plainNoteEcho(v) == v)
    for (v in listOf(null, 0UL, ULong.MAX_VALUE)) {
        check(io.prebindgen.covertest.model.unsignedOptional(v) == v)
        check(io.prebindgen.covertest.model.unsignedRoundTrip(0, 0, 0, 0UL, v).maybe_long == v)
    }
    val stored = io.prebindgen.covertest.storage.storageNew()
    io.prebindgen.covertest.storage.storageGet(stored) { id, seq, value, flag ->
        check(id == null && seq == null && value == null && flag == null)
    }
    check(!io.prebindgen.covertest.storage.storagePutOpt(stored, false, 0, 0, 0.0, false))
    check(io.prebindgen.covertest.storage.storagePutOpt(stored, true, 42, 7, 1.5, false))
    io.prebindgen.covertest.storage.storageGet(stored) { id, seq, value, flag ->
        check(id == 42L && seq == 7 && value == 1.5 && flag == false)
    }
    stored.close()
    val archive = io.prebindgen.covertest.analytics.archiveNew()
    check(io.prebindgen.covertest.analytics.archiveLatest(archive) == null)
    io.prebindgen.covertest.analytics.archiveStore(archive, 0, 2, 40.0, null)
    val first = io.prebindgen.covertest.analytics.archiveLatest(archive)!!
    val second = io.prebindgen.covertest.analytics.archiveLatest(archive)!!
    archive.close()
    check(summaryTotalRaw(1, 0, 0.0, first) == 40.0)
    check(first.isClosed && !second.isClosed)
    check(summaryTotalRaw(1, 0, 0.0, second) == 40.0)

    for (v in listOf(null, 0L, Long.MIN_VALUE, Long.MAX_VALUE)) check(optionI64(v) == v)
    for (v in listOf(null, 0, 0x10ffff)) check(optionChar(v) == v)
    for (v in listOf(null, 0, 255)) check(optionU8(v) == v)
    for (v in listOf(null, false, true)) check(optionBool(v) == v)
    for (v in listOf(null, "", "héllo\u0000世界")) check(optionText(v) == v)
    // Null and empty must remain different both in parameters and results.
    check(optionBytes(null) == null)
    check(optionBytes(byteArrayOf())!!.isEmpty())
    check(optionBytes(byteArrayOf(0, -1))!!.contentEquals(byteArrayOf(0, -1)))
    check(optionStrEcho("") == "")
    check(optionSliceEcho(byteArrayOf())!!.isEmpty())
    // Explicit presence must agree with a non-null discriminator even when
    // an expanded constructor exposes nullable raw reference parameters.
    check(optionalTextInputLen(false, null, 9) == -1L)
    check(optionalTextInputLen(false, "ignored", 9) == -1L)
    check(optionalTextInputLen(true, "", 5) == 5L)
    check(optionalTextInputLen(true, "é", 5) == 7L)
    try { optionalTextInputLen(true, null, 5); error("null present text accepted") }
    catch (_: IllegalArgumentException) {}
    check(optionalBytesInputLen(false, null, 9) == -1L)
    check(optionalBytesInputLen(false, byteArrayOf(1), 9) == -1L)
    check(optionalBytesInputLen(true, byteArrayOf(), 5) == 5L)
    check(optionalBytesInputLen(true, byteArrayOf(0, -1), 5) == 7L)
    try { optionalBytesInputLen(true, null, 5); error("null present bytes accepted") }
    catch (_: IllegalArgumentException) {}
    // Call the two reference fixtures directly to check the native ABI and
    // rejection of malformed tags without Kotlin's argument guards. Their
    // (Int, reference) signatures are unique in this fixture manifest; selecting
    // by signature keeps the check independent of generated call numbering.
    for (referenceType in listOf(String::class.java, ByteArray::class.java)) {
        val native = JNINative::class.java.declaredMethods.single {
            it.name.startsWith("call") && it.parameterTypes.contentEquals(arrayOf(Int::class.javaPrimitiveType, referenceType))
        }
        val empty: Any = if (referenceType == String::class.java) "" else byteArrayOf()
        for ((tag, payload) in listOf(-1 to empty, 0 to empty, 1 to null, 1 to empty)) {
            val packet = native.invoke(null, tag, payload) as Array<*>
            check(packet[0] == null)
            val result = packet.drop(1)
            check(result.size == 2 && result[0] == tag)
            if (tag != 1 || payload == null) check(result[1] == null)
            else if (payload is String) check(result[1] == "")
            else check((result[1] as ByteArray).isEmpty())
        }
        for (tag in listOf(-2, Int.MIN_VALUE, 2, Int.MAX_VALUE)) {
            val packet = native.invoke(null, tag, empty) as Array<*>
            check(packet[0] is String)
        }
    }
    check(optionTripleText(false, true, "ignored") { outer, inner, v -> !outer && inner == null && v == null })
    check(optionTripleText(true, false, "ignored") { outer, inner, v -> outer && inner == false && v == null })
    check(optionTripleText(true, true, null) { outer, inner, v -> outer && inner == true && v == null })
    for (text in listOf("", "héllo\u0000世界")) {
        check(optionTripleText(true, true, text) { outer, inner, v -> outer && inner == true && v == text })
    }
    check(optionTripleBytes(false, true, byteArrayOf(9)) { outer, inner, v -> !outer && inner == null && v == null })
    check(optionTripleBytes(true, false, byteArrayOf(9)) { outer, inner, v -> outer && inner == false && v == null })
    check(optionTripleBytes(true, true, null) { outer, inner, v -> outer && inner == true && v == null })
    for (bytes in listOf(byteArrayOf(), byteArrayOf(0, -1, 127))) {
        check(optionTripleBytes(true, true, bytes) { outer, inner, v -> outer && inner == true && v!!.contentEquals(bytes) })
    }
    check(optionTripleI64(false, true, 9) { outer, inner, v -> !outer && inner == null && v == null })
    check(optionTripleI64(true, false, 9) { outer, inner, v -> outer && inner == false && v == null })
    check(optionTripleI64(true, true, null) { outer, inner, v -> outer && inner == true && v == null })
    for (v in listOf(Long.MIN_VALUE, 0L, Long.MAX_VALUE)) {
        check(optionTripleI64(true, true, v) { outer, inner, value -> outer && inner == true && value == v })
    }
    check(optionTripleUnit(false, true, true) { outer, inner, unit -> !outer && inner == null && unit == null })
    check(optionTripleUnit(true, false, true) { outer, inner, unit -> outer && inner == false && unit == null })
    check(optionTripleUnit(true, true, false) { outer, inner, unit -> outer && inner == true && unit == false })
    check(optionTripleUnit(true, true, true) { outer, inner, unit -> outer && inner == true && unit == true })
    check(nestedNicheChoiceId(false, true, 0, 65, null) == -3L)
    check(nestedNicheChoiceId(true, false, 0, 65, null) == -2L)
    check(nestedNicheChoiceId(true, true, 0, null, null) == -1L)
    check(nestedNicheChoiceId(true, true, 0, 65, null) == 65L)
    val selectorHandle = nicheChoiceNew(66)
    check(nestedNicheChoiceId(false, true, 1, null, selectorHandle) == -3L && !selectorHandle.isClosed)
    check(nestedNicheChoiceId(true, false, 1, null, selectorHandle) == -2L && !selectorHandle.isClosed)
    check(nestedNicheChoiceId(true, true, 1, null, selectorHandle) == 66L && selectorHandle.isClosed)
    for (tag in listOf(-1, -2, Int.MIN_VALUE, 2, Int.MAX_VALUE)) {
        try { nestedNicheChoiceId(true, true, tag, 65, null); error("invalid selector accepted") }
        catch (_: IllegalArgumentException) {}
    }
    check(!optionUnit(false)); check(optionUnit(true))
    check(optionNested(false, 7) { present, v -> !present && v == null })
    check(optionNested(true, null) { present, v -> present && v == null })
    check(optionNested(true, 7) { present, v -> present && v == 7L })
    check(optionNestedChar(false, 65) { present, v -> !present && v == null })
    check(optionNestedChar(true, null) { present, v -> present && v == null })
    check(optionNestedChar(true, 65) { present, v -> present && v == 65 })
    for (v in listOf(-1, 256)) {
        try {optionU8(v); error("invalid u8 accepted")} catch (_: IllegalArgumentException) {}
    }
    for (v in listOf(-1, 0xd800, 0x110000)) {
        try {optionChar(v); error("invalid char accepted")} catch (_: IllegalArgumentException) {}
    }
    val bytesRecord = OptionalBytes(byteArrayOf(1,2))
    check(optionalBytesEcho(bytesRecord) == bytesRecord)
    check(optionalBytesEcho(bytesRecord).hashCode() == bytesRecord.hashCode())
    check(optionalBytesEcho(OptionalBytes(null)) == OptionalBytes(null))
    // An unannotated field expansion must preserve the outer presence bit.
    check(optionalFieldNew(false, 7) { present, value -> !present && value == null })
    check(optionalFieldNew(true, null) { present, value -> present && value == null })
    check(optionalFieldNew(true, 7) { present, value -> present && value == 7L })
    check(optionalFieldsEcho(null) == null)
    for (v in listOf(OptionalFields(null, null, null), OptionalFields(8, "", 0), OptionalFields(9, "é", 65))) {
        check(optionalFieldsEcho(v) == v)
    }
    check(optionStrEcho(null) == null); check(optionStrEcho("héllo") == "héllo")
    check(optionSliceEcho(null) == null); check(optionSliceEcho(byteArrayOf(0, -1))!!.contentEquals(byteArrayOf(0, -1)))
    check(optionStrLen(null) == null); check(optionStrLen("é") == 2uL)
    check(optionSliceLen(null) == null); check(optionSliceLen(byteArrayOf(1, 2)) == 2uL)
    check(optionStringLen(null) == null); check(optionStringLen("é") == 2uL)
    check(optionI64Ref(null) == null); check(optionI64Ref(9) == 9L)
    check(optionChoiceId(false, 0, 8, null) == -1L)
    check(optionChoiceId(true, 0, 8, null) == 8L)
    val borrowedSource = io.prebindgen.covertest.resources.projectionPairOwned()
    check(optionProjectionRef(borrowedSource.first, false) == null)
    optionProjectionRef(borrowedSource.first, true)!!.close()
    borrowedSource.first.close(); borrowedSource.second.close()
    val viewDrops = io.prebindgen.covertest.resources.projectionDrops()
    optionProjectionViews(false) { a, b -> check(a == null && b == null) }
    try {optionProjectionViews(true) { _, _ -> error("delivery") }} catch (e: IllegalStateException) {check(e.message == "delivery");check(e.suppressed.isEmpty())}
    check(io.prebindgen.covertest.resources.projectionDrops() == viewDrops + 2uL)
    check(optionResourcePair(null, null) == 0L)
    check(optionCtorId(0,null,null) == -10L)
    check(optionCtorId(0,7,null) == 7L)
    val ctor = optionCtorNew(null)
    check(optionCtorId(1,99,ctor) == -10L && ctor.isClosed)
    val nestedStart=ownedResourceDrops()
    val untouched=optionResourceNew(true)!!
    optionNestedResource(false,untouched) {present,value->check(!present && value==null)}
    check(!untouched.isClosed)
    optionNestedResource(true,null) {present,value->check(present && value==null)}
    val nestedMoved=optionNestedResource(true,untouched) {present,value->check(present);requireNotNull(value)}
    check(untouched.isClosed);nestedMoved.close()
    check(ownedResourceDrops()==nestedStart+1uL)
    val resourceChoice = optionResourceNew(true)!!
    check(resourceChoiceId(false, resourceChoice, 7) == -1L && !resourceChoice.isClosed)
    check(resourceChoiceId(true, resourceChoice, 7) == 84L && resourceChoice.isClosed)
    val start = ownedResourceDrops()
    check(optionResourceNew(false) == null)
    check(optionResourceEcho(null, 65) == null)
    check(optionResourceId(null) == -1L)
    check(ownedResourceDrops() == start)
    val handle = optionResourceNew(true)!!
    check(optionResourceId(handle) == 77L)
    try {optionResourcePair(handle, handle); error("aliased optional owners accepted")} catch (_: IllegalArgumentException) {}
    check(!handle.isClosed)
    try { optionResourceEcho(handle, 0xd800); error("invalid scalar accepted") } catch (_: IllegalArgumentException) {}
    check(!handle.isClosed)
    val returned = optionResourceEcho(handle, 65)!!
    check(handle.isClosed)
    check(optionResourceId(returned) == 77L)
    check(optionResourceMut(null) == -1L)
    check(optionResourceMut(returned) == 78L)
    returned.close()
    check(ownedResourceDrops() == start + 1uL)
    try {optionResourceId(returned); error("closed optional handle accepted")} catch (_: IllegalArgumentException) {}
    val none = optionalResourceNew(false)!!
    check(none.resource == null && none.details == null)
    val record = optionalResourceNew(true)!!
    check(record.details == OptionalFields(9, null, 233))
    record.resource!!.close()
    check(ownedResourceDrops() == start + 2uL)
    try { optionalPairNew { absent, present -> check(absent == null && present != null); error("mixed delivery") } }
    catch (e: IllegalStateException) {check(e.message == "mixed delivery");check(e.suppressed.isEmpty())}
    check(ownedResourceDrops() == start + 3uL)
}
