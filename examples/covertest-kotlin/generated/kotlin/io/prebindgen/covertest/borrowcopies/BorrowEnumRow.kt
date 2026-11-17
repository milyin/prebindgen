package io.prebindgen.covertest.borrowcopies

/** Close each handle member explicitly. Copies share the same handle objects. */
public data class BorrowEnumRow(
    public val packet: io.prebindgen.covertest.borrowcopies.BorrowPacket,
    public val mode: io.prebindgen.covertest.borrowcopies.BorrowMode
)
