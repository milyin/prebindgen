package io.prebindgen.covertest

import io.prebindgen.covertest.buffers.*
import io.prebindgen.covertest.callbacks.*
import java.util.concurrent.atomic.AtomicReference

private object BufferAllocationProbe {
    init { NativeLibrary.ensureLoaded() }
    @JvmStatic external fun start()
    @JvmStatic external fun stop(): Long
    @JvmStatic external fun directText(): String
    @JvmStatic external fun directBytes(): ByteArray
}

private fun largeNativeAllocations(call: () -> Unit): Long {
    BufferAllocationProbe.start()
    try { call() } catch (failure: Throwable) { BufferAllocationProbe.stop(); throw failure }
    return BufferAllocationProbe.stop()
}

private val bufferUnicode = "héllo\u0000世界😀"
private val bufferBits = byteArrayOf(0, 127, -128, -1)

fun checkBorrowedBuffers() {
    // Warm source storage, class loading and runtime encoding before measurement.
    val text = BufferAllocationProbe.directText()
    val bytes = BufferAllocationProbe.directBytes()
    check(text.length == 65536 && bytes.size == 65536)
    check(jniBufferText() == text && jniBufferString() == text)
    check(jniBufferBytes().contentEquals(bytes) && jniBufferVec().contentEquals(bytes))
    jniBufferRows()
    jniBufferCallback({ _, _, _, _ -> })
    val textEncoding = largeNativeAllocations { check(BufferAllocationProbe.directText() == text) }
    val byteEncoding = largeNativeAllocations { check(BufferAllocationProbe.directBytes().contentEquals(bytes)) }
    check(byteEncoding == 0L) // Copying a slice into byte[] needs no native staging allocation.
    repeat(16) {
        check(largeNativeAllocations { check(jniBufferText() == text) } == textEncoding)
        check(largeNativeAllocations { check(jniBufferString() == text) } == textEncoding)
        check(largeNativeAllocations { check(jniBufferBytes().contentEquals(bytes)) } == byteEncoding)
        check(largeNativeAllocations { check(jniBufferVec().contentEquals(bytes)) } == byteEncoding)
        check(largeNativeAllocations { check(jniBufferRows().size == 3) } == textEncoding + byteEncoding)
        check(largeNativeAllocations { jniBufferCallback({ _, _, _, _ -> }) } == 2 * (textEncoding + byteEncoding))
    }
    for (mode in 0..3) {
        jniBufferOptionalText(mode) { present, value ->
            check(present == (mode != 0))
            check(value == when (mode) { 0, 1 -> null; 2 -> ""; else -> bufferUnicode })
        }
        jniBufferOptionalBytes(mode) { present, value ->
            check(present == (mode != 0))
            if (mode < 2) check(value == null)
            else check(requireNotNull(value).contentEquals(if (mode == 2) byteArrayOf() else bufferBits))
        }
        jniBufferOwnedOptional(mode) { present, row ->
            check(present == (mode != 0))
            if (mode < 2) check(row == null)
            else check(row!!.text == bufferUnicode && row.bytes.contentEquals(bufferBits))
        }
    }
    jniBufferWide { t0, t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11, t12, t13, t14, t15, t16, t17, t18, t19, t20, t21, t22, t23, t24, t25, t26, t27, t28, t29, t30, t31, t32, t33, t34, t35, t36, t37, t38, t39, t40, t41, t42, t43, t44, t45, t46, t47 ->
        check(listOf(t0, t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11, t12, t13, t14, t15, t16, t17, t18, t19, t20, t21, t22, t23, t24, t25, t26, t27, t28, t29, t30, t31, t32, t33, t34, t35, t36, t37, t38, t39, t40, t41, t42, t43, t44, t45, t46, t47).all { it == bufferUnicode })
    }
    // These JVM values survive destruction of their temporary Rust owner.
    val copiedViews = jniBufferOwned { name, data -> name to data }
    check(copiedViews.first == bufferUnicode && copiedViews.second.contentEquals(bufferBits))
    val borrowedRows = jniBufferRows()
    check(borrowedRows[0].text == text && borrowedRows[0].bytes.contentEquals(bytes))
    check(borrowedRows[1].text == bufferUnicode && borrowedRows[1].bytes.contentEquals(bufferBits))
    check(borrowedRows[2].text.isEmpty() && borrowedRows[2].bytes.isEmpty())

    val failure = AtomicReference<Throwable?>()
    var calls = 0
    val saved = mutableListOf<Pair<String, ByteArray>>()
    jniBufferCallback({ name, data, optionalName, optionalData ->
        try {
            val row = calls++
            if (row < 3) {
                check(name == borrowedRows[row].text && data.contentEquals(borrowedRows[row].bytes))
                check(optionalName == name && optionalData!!.contentEquals(data))
            } else check(name.isEmpty() && data.isEmpty() && optionalName == null && optionalData == null)
            saved.add(name to data)
        } catch (error: Throwable) { failure.set(error) }
    })
    failure.get()?.let { throw it }
    check(calls == 4 && saved[1].first == bufferUnicode && saved[1].second.contentEquals(bufferBits))
    calls = 0
    jniBufferOwnedCallback({ name, data ->
        try {
            if (calls++ == 0) check(name == null && data == null)
            else {
                check(name == bufferUnicode && data!!.contentEquals(bufferBits))
                saved.add(name to data)
            }
        } catch (error: Throwable) { failure.set(error) }
    })
    failure.get()?.let { throw it }
    check(calls == 2 && saved.last().first == bufferUnicode && saved.last().second.contentEquals(bufferBits))
    val workerValue = AtomicReference<Pair<String, ByteArray>?>()
    jniBufferWorker({ name, data -> workerValue.set(name to data) })
    val worker = requireNotNull(workerValue.get())
    check(worker.first == bufferUnicode && worker.second.contentEquals(bufferBits))
    println("borrowed buffers PASS: 64KiB allocation counts match direct encoders (text=$textEncoding, bytes=$byteEncoding); getters, nested options, records, rows and callbacks")
}

object BufferFailureEntryPoint {
    @JvmStatic fun run() {
        var calls = 0
        var delivered: CallbackOwned? = null
        val failure = AtomicReference<Throwable?>()
        jniBufferFailure({ handle, name, bytes ->
            try {
                calls++
                check(callbackOwnedId(handle) == 42L && name.length == 65536 && bytes.size == 65536)
                delivered = handle
            } catch (error: Throwable) { failure.set(error) }
        })
        failure.get()?.let { throw it }
        check(calls == 1 && delivered?.isClosed == true)
    }
}


object BufferReturnFailureEntryPoint {
    @JvmStatic fun run(mode: Int) {
        val before = callbackOwnedDrops()
        var domainInvoked = false
        var bindingInvoked = false
        try {
            when (mode) {
                4, 5 -> jniBufferResource().resource.close()
                6, 7 -> jniBufferDomainFailure(
                    onBindingError = { bindingInvoked = true },
                    onError = { row -> domainInvoked = true; row.resource.close() })
                8, 9 -> jniBufferResourceRows().forEach { it.resource.close() }
                else -> error("unknown buffer failure probe")
            }
            error("expected buffer allocation failure")
        } catch (failure: OutOfMemoryError) {
            check(failure.message == "injected callback allocation failure")
        }
        check(!domainInvoked && !bindingInvoked)
        check(callbackOwnedDrops() == before + if (mode >= 8) 3uL else 1uL)
    }
}
