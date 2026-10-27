package io.prebindgen.covertest.enums

public enum class EnumNames(public val _wireValue: kotlin.Int) {
    `Companion`(4),
    `wireValue`(9),
    `fromWire`(12),
    `type`(14),
    `when`(16),
    `Name`(20),
    `Ordinal`(21),
    `Entries`(22),
    `value`(24),
    `Kotlin`(25);

    public companion object _Companion {
        public fun _fromWire(_value: kotlin.Int): EnumNames = when (_value) {
            4 -> `Companion`
            9 -> `wireValue`
            12 -> `fromWire`
            14 -> `type`
            16 -> `when`
            20 -> `Name`
            21 -> `Ordinal`
            22 -> `Entries`
            24 -> `value`
            25 -> `Kotlin`
            else -> throw kotlin.IllegalArgumentException("invalid enum discriminant: $_value")
        }
    }
}
