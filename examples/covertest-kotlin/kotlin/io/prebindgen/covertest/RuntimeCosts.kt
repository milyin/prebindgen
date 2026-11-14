package io.prebindgen.covertest
import io.prebindgen.covertest.runtimecosts.*

fun checkRuntimeCosts() {
    bufferViewsText { text, length -> check(text == "move text" && length == 9uL) }
    bufferViewsBytes { bytes, length ->
        check(bytes.contentEquals(byteArrayOf(1, 2, 3, 4)) && length == 4uL)
    }
    for (present in listOf(false, true)) {
        bufferViewsOptionalText(present) { text, length ->
            check(text == (if (present) "move text" else null) && length == (if (present) 9uL else 0uL))
        }
        bufferViewsOptionalBytes(present) { bytes, length ->
            check((bytes == null) == !present && length == (if (present) 4uL else 0uL))
            if (present) check(checkNotNull(bytes).contentEquals(byteArrayOf(1, 2, 3, 4)))
        }
    }
    println("runtime costs PASS: owned buffer identities coexist with getters")
}
