package io.prebindgen.covertest

import io.prebindgen.covertest.collections.*
import io.prebindgen.covertest.enums.Code
import io.prebindgen.covertest.resources.ownedResourceDrops
import io.prebindgen.covertest.resources.ownedResourceId
import java.lang.reflect.InvocationTargetException

private fun collectionInvalid(action: () -> Unit) {
    try { action() } catch (_: IllegalArgumentException) { return }
    error("expected collection validation failure")
}

fun checkCollections() {
    collectionNumbers { values, tag -> check(values == listOf(7L, 9L) && tag == 42L) }
    val numbers = listOf(Long.MIN_VALUE, 0L, Long.MAX_VALUE)
    check(collectionI64(numbers) == numbers)
    check(collectionI64(emptyList()).isEmpty())
    collectionInvalid { collectionI64(null) }
    check(collectionSlice(numbers) == numbers)
    check(collectionBorrow(numbers) == numbers)
    val text = listOf("", "héllo 世界", "embedded\u0000nul")
    check(collectionText(text) == text)
    val bytes = listOf(byteArrayOf(), byteArrayOf(0, -1, 127))
    val byteResult = collectionBytes(bytes)
    check(bytes.indices.all { bytes[it].contentEquals(byteResult[it]) })
    val nested = listOf(numbers, emptyList(), listOf(7L))
    check(collectionNested(nested) == nested)
    check(collectionOptions(listOf(null, Long.MIN_VALUE, null, Long.MAX_VALUE), 0) == listOf(null, Long.MIN_VALUE, null, Long.MAX_VALUE))
    check(collectionOptional(null) == null)
    check(collectionOptional(emptyList()) == emptyList<Long>())
    check(collectionOptional(numbers) == numbers)
    check(collectionEnum(listOf(Code.Bottom, Code.Seven, Code.Top)) == listOf(Code.Bottom, Code.Seven, Code.Top))
    check(collectionProbe(listOf(0, 65, 0x10ffff), "") == listOf(0, 65, 0x10ffff))
    collectionInvalid { collectionProbe(listOf(65, 0xd800), "") }
    check(collectionResult(numbers, true) == numbers)
    try { collectionResult(numbers, false); error("expected domain error") } catch (_: java.io.IOException) {}
    val storage = io.prebindgen.covertest.storage.storageNew()
    check(storageGetVec(storage) == null)
    val payloads = listOf(Payload(5, 2, 1.5, true), Payload(9, -1, -2.0, false))
    storagePutSlice(storage, payloads)
    check(storageGetVec(storage) == payloads)
    storage.close()

    val start = ownedResourceDrops()
    val handles = listOf(collectionResourceNew(1), collectionResourceNew(2))
    collectionInvalid { collectionHandles(handles, 0xd800, 0) }
    check(handles.none { it.isClosed })
    collectionInvalid { collectionHandles(listOf(handles[0], handles[0]), 65, 0) }
    collectionInvalid { collectionAlias(handles, handles[1]) }
    check(handles.none { it.isClosed })
    check(collectionResourceIds(listOf(handles[0], handles[0])) == listOf(1L, 1L))
    check(handles.none { it.isClosed })
    val moved = collectionHandles(handles, 65, 0)
    check(handles.all { it.isClosed })
    check(moved.map(::ownedResourceId) == listOf(1L, 2L))
    moved.forEach { it.close() }
    for (mode in listOf(1, 2)) {
        val inputs = listOf(collectionResourceNew(3), collectionResourceNew(4))
        try { collectionHandles(inputs, 65, mode); error("expected collection failure") }
        catch (_: java.io.IOException) { check(mode == 1) }
        catch (_: IllegalArgumentException) { check(mode == 2) }
        check(inputs.all { it.isClosed })
        inputs.forEach { it.close() }
    }
    check(ownedResourceDrops() == start + 8uL)
    val panicStart = ownedResourceDrops()
    val pending = (1L..3L).map(::collectionResourceNew)
    collectionInvalid { collectionConstructAll(pending) }
    check(pending.all { it.isClosed })
    check(ownedResourceDrops() == panicStart + 3uL)
    collectionInvalid { collectionMakeItems() }
    check(ownedResourceDrops() == panicStart + 6uL)
    var escaped: List<io.prebindgen.covertest.resources.OwnedResource>? = null
    val failure = IllegalStateException("collection builder failure")
    try {
        collectionBundle { values, tag ->
            check(tag == 42L)
            escaped = values
            throw failure
        }
    } catch (caught: IllegalStateException) { check(caught === failure) }
    check(checkNotNull(escaped).all { it.isClosed })
    check(ownedResourceDrops() == panicStart + 9uL)
    check(collectionOptionalHandles(null) == null)
    check(collectionOptionalHandles(emptyList()) == emptyList<Any>())
    val optionalHandle = collectionResourceNew(10)
    checkNotNull(collectionOptionalHandles(listOf(optionalHandle))).single().close()
    check(optionalHandle.isClosed)
    val nestedHandle = collectionResourceNew(11)
    collectionInvalid { collectionNestedHandles(listOf(listOf(nestedHandle), listOf(nestedHandle))) }
    check(!nestedHandle.isClosed)
    collectionNestedHandles(listOf(emptyList(), listOf(nestedHandle))).flatten().single().close()
    check(nestedHandle.isClosed)
    // Dynamic handle lists must not add one JVM stack frame per lock.
    val many = List(4000) { collectionResourceNew(it.toLong()) }
    collectionHandles(many, 65, 0).forEach { it.close() }
    check(many.all { it.isClosed })
    // A mutable/custom input list is read once, before locks are chosen.
    val first = collectionResourceNew(7)
    val second = collectionResourceNew(8)
    var reads = 0
    val changing = object : AbstractList<io.prebindgen.covertest.resources.OwnedResource>() {
        override val size: Int get() = 1
        override fun get(index: Int) = if (reads++ == 0) first else second
    }
    val snapshot = collectionHandles(changing, 65, 0)
    check(reads == 1 && first.isClosed && !second.isClosed)
    check(ownedResourceId(snapshot.single()) == 7L)
    snapshot.single().close(); second.close()
    check(collectionI64(List(20000) { it.toLong() }).size == 20000)

    // Exercise raw JNI validation, including an invalid but inactive payload.
    val native = Class.forName("io.prebindgen.covertest.JNINative")
    fun method(second: Class<*>) = native.declaredMethods.single {
        it.parameterTypes.toList() == listOf(Array<Any?>::class.java, second)
    }.also { it.isAccessible = true }
    val probe = method(String::class.java)
    for (bad in listOf(null, arrayOf<Any?>(), arrayOf<Any?>(null), arrayOf<Any?>("bad"), arrayOf<Any?>(0xd800))) {
        collectionInvalid {
            try { probe.invoke(null, arrayOf<Any?>(bad), "") }
            catch (e: InvocationTargetException) { throw e.cause!! }
        }
    }
    val optional = method(Short::class.javaPrimitiveType!!)
    optional.invoke(null, arrayOf<Any?>(arrayOf<Any?>(0, "ignored")), 0.toShort())
    collectionInvalid {
        try { optional.invoke(null, arrayOf<Any?>(arrayOf<Any?>(1, "invalid active")), 0.toShort()) }
        catch (e: InvocationTargetException) { throw e.cause!! }
    }
}
