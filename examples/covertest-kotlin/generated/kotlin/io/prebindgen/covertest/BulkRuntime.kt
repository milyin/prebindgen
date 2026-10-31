package io.prebindgen.covertest

internal class BulkLayout(val size: Int, val alignment: Int, val offsets: IntArray)
internal class BulkRange(val offset: Int, val length: Int)

internal open class BulkWriter(initial: Int) {
    var array = ByteArray(initial.coerceAtLeast(16)); private set
    var length = initial; private set
    private var view = java.nio.ByteBuffer.wrap(array).order(BulkLayouts.byteOrder)
    private fun grow(end: Long) {
        InputValidation.require(end >= 0 && end <= Int.MAX_VALUE) { "bulk frame exceeds JVM array limit" }
        if (end > array.size) {
            val capacity = maxOf(end, minOf(Int.MAX_VALUE.toLong(), array.size.toLong() * 2)).toInt()
            array = array.copyOf(capacity)
            view = java.nio.ByteBuffer.wrap(array).order(BulkLayouts.byteOrder)
        }
    }
    fun rows(count: Int, layout: BulkLayout): Int {
        val start = (length.toLong() + layout.alignment - 1) and (layout.alignment.toLong() - 1).inv()
        val end = start + count.toLong() * layout.size
        grow(end); length = end.toInt(); return start.toInt()
    }
    fun range(at: Int, start: Int, count: Int) { view.putLong(at, start.toLong()); view.putLong(at + 8, count.toLong()) }
    fun absent(at: Int) { view.putLong(at, -1L); view.putLong(at + 8, 0L) }
    fun bytes(at: Int, value: ByteArray?) {
        if (value == null) { absent(at); return }
        val start = length; grow(start.toLong() + value.size)
        value.copyInto(array, start); length += value.size; range(at, start, value.size)
    }
    fun text(at: Int, value: String?) {
        val encoded = try { value?.encodeToByteArray(throwOnInvalidSequence = true) }
        catch (failure: java.nio.charset.CharacterCodingException) { throw HandleValidationFailure("invalid UTF-16 in bulk text") }
        bytes(at, encoded)
    }
    fun byte(at: Int, value: Byte) { view.put(at, value) }
    fun short(at: Int, value: Short) { view.putShort(at, value) }
    fun int(at: Int, value: Int) { view.putInt(at, value) }
    fun long(at: Int, value: Long) { view.putLong(at, value) }
    fun float(at: Int, value: Float) { view.putFloat(at, value) }
    fun double(at: Int, value: Double) { view.putDouble(at, value) }
    fun wide(at: Int, input: ByteArray?, signed: Boolean) {
        val value = input ?: run { java.util.Arrays.fill(array, at, at + 16, 0.toByte()); return }
        InputValidation.require(value.isNotEmpty()) { "empty 128-bit integer" }
        val negative = value[0] < 0
        InputValidation.require(signed || !negative) { "negative unsigned 128-bit integer" }
        val pad = if (negative) (-1).toByte() else 0.toByte()
        var start = 0
        while (value.size - start > 16 && value[start] == pad && (!signed || ((value[start + 1] < 0) == negative))) start++
        InputValidation.require(value.size - start <= 16) { "128-bit integer out of range" }
        java.util.Arrays.fill(array, at, at + 16, pad)
        value.copyInto(array, at + 16 - (value.size - start), start)
    }
}
internal class BulkReader(val array: ByteArray) {
    private val view = java.nio.ByteBuffer.wrap(array).order(BulkLayouts.byteOrder)
    fun byte(at: Int): Byte = view.get(at)
    fun short(at: Int): Short = view.getShort(at)
    fun int(at: Int): Int = view.getInt(at)
    fun long(at: Int): Long = view.getLong(at)
    fun float(at: Int): Float = view.getFloat(at)
    fun double(at: Int): Double = view.getDouble(at)
    fun wide(at: Int): ByteArray = array.copyOfRange(at, at + 16)
    fun range(at: Int): BulkRange? {
        val start = view.getLong(at); val count = view.getLong(at + 8)
        if (start == -1L) return null
        require(start >= 0 && start <= Int.MAX_VALUE && count >= 0 && count <= Int.MAX_VALUE) { "invalid bulk range" }
        return BulkRange(start.toInt(), count.toInt())
    }
    fun rows(range: BulkRange?, stride: Int): BulkRange {
        val r = requireNotNull(range) { "null bulk collection" }
        require(stride > 0 && r.offset.toLong() + r.length.toLong() * stride <= array.size) { "bulk rows outside frame" }
        return r
    }
    private fun buffer(at: Int): BulkRange? {
        val r = range(at) ?: return null
        require(r.offset.toLong() + r.length <= array.size) { "bulk buffer outside frame" }
        return r
    }
    fun bytes(at: Int): ByteArray? = buffer(at)?.let { array.copyOfRange(it.offset, it.offset + it.length) }
    fun text(at: Int): String? = buffer(at)?.let { array.decodeToString(it.offset, it.offset + it.length, throwOnInvalidSequence = true) }
}

/** Pack values once; resolve only native addresses after the wrapper acquires locks. */
internal class BulkHandleWriter(initial: Int) : BulkWriter(initial), PreparedInput {
    private var savedHandles: ArrayList<NativeHandle>? = null
    private var savedOwners: ArrayList<NativeHandle>? = null
    private var offsets = EMPTY_OFFSETS
    private companion object { val EMPTY_OFFSETS = IntArray(0) }
    override val handles: List<NativeHandle> get() = savedHandles ?: emptyList()
    override val owners: List<NativeHandle> get() = savedOwners ?: emptyList()
    fun handle(at: Int, value: NativeHandle?, owned: Boolean) {
        long(at, 0L)
        if (value == null) return
        if (savedHandles == null) savedHandles = ArrayList()
        val index = savedHandles!!.size
        if (index == offsets.size) offsets = offsets.copyOf(maxOf(8, index * 2))
        offsets[index] = at
        savedHandles!!.add(value)
        if (owned) {
            if (savedOwners == null) savedOwners = ArrayList()
            savedOwners!!.add(value)
        }
    }
    override fun encode(): ByteArray {
        for (index in handles.indices) long(offsets[index], handles[index].raw)
        return array
    }
}
