package io.prebindgen.covertest.typeinputs

class TypeInputRow(a: TypeInputA, val b: TypeInputB) {
    private val first = a
    val a: TypeInputA get() = readA?.invoke(first) ?: first
    companion object { internal var readA: ((TypeInputA) -> TypeInputA)? = null }
}
