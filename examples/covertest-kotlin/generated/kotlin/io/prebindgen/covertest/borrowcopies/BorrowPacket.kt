package io.prebindgen.covertest.borrowcopies

public class BorrowPacket internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    override val handleType: Long get() = 3826124145257993298L
    public fun take(): BorrowPacket = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = BorrowPacket(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_BorrowPacket_969214a55d92aefe(raw) }
}
