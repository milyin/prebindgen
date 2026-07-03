package io.prebindgen.covertest.model

/**
 * Hand-written surface for the native Rust `Freshness` enum — the binding
 * declares it `enum_class(...).suppress_kotlin_code()`, so no Kotlin file is
 * generated and this one takes over. It must stay wire-compatible with the
 * generated convention (see the generated [Priority]): an `Int` `value` per
 * variant and a `fromInt` companion, both of which the generated wrappers
 * call.
 */
public enum class Freshness(public val value: Int) {
    FRESH(0),
    STALE(1);

    public companion object {
        @JvmStatic
        public fun fromInt(value: Int): Freshness = entries.first { it.value == value }
    }
}
