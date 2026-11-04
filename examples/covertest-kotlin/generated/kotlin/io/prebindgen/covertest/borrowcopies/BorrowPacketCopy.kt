package io.prebindgen.covertest.borrowcopies

/** Invoked by Rust, possibly from a native worker thread. */
public fun interface BorrowPacketCopy {
    public fun run(`arg0_0`: io.prebindgen.covertest.borrowcopies.BorrowPacket)
}
