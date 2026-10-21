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
    check(optionBytes(null) == null)
    check(optionBytes(byteArrayOf(0, -1))!!.contentEquals(byteArrayOf(0, -1)))
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
    try {optionResourceId(returned); error("closed optional handle accepted")} catch (_: IllegalStateException) {}
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
