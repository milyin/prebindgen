package io.prebindgen.covertest

import io.prebindgen.covertest.enums.*

fun checkEnums() {
    for ((value, expected) in listOf(1.0 to Priority.Low, 50.0 to Priority.High, 1000.0 to Priority.Normal)) {
        val payload = enumPayloadNew(value)
        check(payloadPriority(payload) == expected)
        payload.close()
    }
    check(priorityWeight(Priority.Low) == 1)
    check(priorityWeight(Priority.Normal) == 5)
    check(priorityWeight(Priority.High) == 10)
    check(priorityOr(null, Priority.High) == Priority.High)
    check(priorityOr(Priority.Low, Priority.High) == Priority.Low)
    for (code in Code.entries) {
        check(sparseEcho(code) == code)
        check(sparseOptional(code) == code)
        check(sparseRef(code) == code.wireValue)
        check(sparseBorrowed(code) == code)
        check(sparseNested(true, code) { present, value -> present && value == code })
        val record = EnumRecord(Priority.High, code)
        check(enumRecordEcho(record) == record)
    }
    check(sparseOptional(null) == null)
    check(sparseNested(false, Code.MinusOne) { present, value -> !present && value == null })
    check(sparseNested(true, null) { present, value -> present && value == null })
    check(enumRecordEcho(EnumRecord(Priority.Low, null)) == EnumRecord(Priority.Low, null))
    check(Code.MinusTwo.wireValue == -2)
    check(Code.MinusOne.wireValue == -1)
    check(Code.Zero.wireValue == 0)
    check(Code.Seven.wireValue == 7)
    check(Code.Eight.wireValue == 8)
    check(Code.Bottom.wireValue == Int.MIN_VALUE)
    check(Code.Top.wireValue == Int.MAX_VALUE)
    for (bad in listOf(-4, -3, 1, 6, 9)) {
        try { Code.fromWire(bad); error("invalid enum discriminant accepted") }
        catch (_: IllegalArgumentException) {}
    }
    // This probe has a unique (byte[], int, String) native signature. Bypass
    // the typed wrapper to check actual native discriminant validation and
    // nested sentinels, independently of Kotlin's enum factory.
    val native = JNINative::class.java.declaredMethods.single {
        it.name.startsWith("call") && it.parameterTypes.contentEquals(
            arrayOf(ByteArray::class.java, Int::class.javaPrimitiveType, String::class.java))
    }
    for (tag in listOf(-4, -3, -2, -1, 0, 7, 8, Int.MIN_VALUE, Int.MAX_VALUE)) {
        val packet = native.invoke(null, byteArrayOf(), tag, "") as Array<*>
        check(packet[0] == null)
        val result = packet.drop(1)
        check(result.size == 1 && result[0] == tag)
    }
    for (bad in listOf(-5, 1, 6, 9)) {
        val packet = native.invoke(null, byteArrayOf(), bad, "") as Array<*>
        check(packet[0] is String)
    }
    check(enumBoundaryProbe(byteArrayOf(), false, Code.Seven, "") { present, code -> !present && code == null })
    check(enumBoundaryProbe(byteArrayOf(), true, null, "") { present, code -> present && code == null })
    check(enumBoundaryProbe(byteArrayOf(), true, Code.Seven, "") { present, code -> present && code == Code.Seven })
    for (name in EnumNames.entries) check(enumNamesEcho(name) == name)
    check(EnumNames.Companion._wireValue == 4)
    check(EnumNames.`type`._wireValue == 14)
    check(EnumNames.`when`._wireValue == 16)
    check(EnumNames.Name._wireValue == 20)
    check(EnumNames.Ordinal._wireValue == 21)
    check(EnumNames.Entries._wireValue == 22)
    check(EnumNames.value._wireValue == 24)
    check(EnumNames.Kotlin._wireValue == 25)
}
