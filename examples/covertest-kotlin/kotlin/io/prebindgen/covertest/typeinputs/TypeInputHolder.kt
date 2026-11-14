package io.prebindgen.covertest.typeinputs

class TypeInputHolder(row: TypeInputRow?) {
    private val first = row
    val row: TypeInputRow? get() = if (readRow == null) first else readRow!!.invoke(first)
    companion object { internal var readRow: ((TypeInputRow?) -> TypeInputRow?)? = null }
}
