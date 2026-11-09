package io.prebindgen.covertest

import java.lang.reflect.Method

/** Adapt raw JNI validation probes to the generated typed return capture. */
internal fun Method.invokePacket(vararg arguments: Any?): Array<Any?> {
    val type = parameterTypes.last()
    val companion = type.getField("Companion").get(null)
    val capture = companion.javaClass.getMethod("acquire").invoke(companion)
    fun get(name: String): Any? = type.getMethod(name).invoke(capture)
    try {
        val value = invoke(null, *arguments, capture)
        val status = get("getStatus") as Int
        if (status == 2) return arrayOf(get("getMessage"))
        val prefix = if (status == 3) "getD" else "getS"
        val fields = type.declaredMethods.map { it.name }
            .filter { it.startsWith(prefix) && it.removePrefix(prefix).toIntOrNull() != null }
            .sortedBy { it.removePrefix(prefix).toInt() }
        val values = if (status != 3 && returnType != Void.TYPE) listOf(value)
                     else fields.map(::get)
        return arrayOf(if (status == 3) true else null, *values.toTypedArray())
    } finally { type.getMethod("release").invoke(capture) }
}
