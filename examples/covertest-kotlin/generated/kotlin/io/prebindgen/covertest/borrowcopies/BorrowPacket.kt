package io.prebindgen.covertest.borrowcopies

public class BorrowPacket internal constructor(raw: Long) : io.prebindgen.covertest.NativeHandle(raw) {
    public fun take(): BorrowPacket = synchronized(this) { check(!isClosed) { "closed or consumed handle" }; val owner = BorrowPacket(raw); markConsumed(); owner }
    override fun freeNative(raw: Long) { io.prebindgen.covertest.JNINative.drop_BorrowPacket_969214a55d92aefe(raw) }
}
