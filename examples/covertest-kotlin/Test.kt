import io.prebindgen.demo.*
import java.util.concurrent.CountDownLatch
import java.util.concurrent.atomic.AtomicInteger

private fun fails(action: () -> Unit) {
    try { action() } catch (_: IllegalArgumentException) { return }
    error("expected a binding error")
}
private fun payload(k: String, d: String) = payloadFromString(1, null, k, d)
private fun <R> invoke(k: String, x: Payload, y: Payload, build: (Payload, KeyExpr, String, Bytes) -> R): R =
    foo(1, null, k, 0, x, 0, null, null, null, 0, null, null, null, 0, null, null, null, y, build)

fun main() {
    var cases = 0
    for (kt in 0..1) for (xt in 0..3) for (xkt in 0 until (if (xt == 0) 1 else 2)) {
        val k = if (kt == 0) keyexprFromString("k") else null
        val x = if (xt == 0) payload("x", "A") else null
        val xk = if (xt != 0 && xkt == 0) keyexprFromString("x") else null
        val data = if (xt == 1) bytesFromSlice(byteArrayOf(65)) else null
        val y = payload("y", "B")
        foo(kt, k, if (kt == 1) "k" else null,
            xt, x,
            xkt, if (xt == 1) xk else null, if (xt == 1 && xkt == 1) "x" else null, data,
            xkt, if (xt == 2) xk else null, if (xt == 2 && xkt == 1) "x" else null, if (xt == 2) "A" else null,
            xkt, if (xt == 3) xk else null, if (xt == 3 && xkt == 1) "x" else null, if (xt == 3) byteArrayOf(65) else null,
            y) { whole, key, string, bytes ->
                check(string == "k:x:y")
                check(payloadText(whole) == "k:x:y:AB")
                check(whole.isClosed)
                check(keyexprText(key) == string)
                check(bytesRead(bytes).contentEquals(byteArrayOf(65, 66)))
            }
        check(y.isClosed)
        check(k == null || k.isClosed)
        check(x == null || x.isClosed)
        check(xk == null || xk.isClosed)
        check(data == null || data.isClosed)
        cases++
    }
    check(cases == 14)
    // An inactive, already closed handle must never be consumed or validated.
    val inactive = keyexprFromString("unused")
    inactive.close()
    payloadFromString(1, inactive, "safe", "A").close()
    val retained = keyexprFromString("retained")
    fails { payloadFromString(99, retained, null, "A") }
    check(!retained.isClosed)
    retained.close(); retained.close()
    fails { keyexprText(retained) }
    fails { keyexprFromString(null) }
    val wrong = bytesFromSlice(byteArrayOf(1))
    fails { JNINative.call2(wrong.raw) }
    check(!wrong.isClosed); wrong.close()
    // A later decoding failure does not resurrect earlier consumed handles.
    val early = keyexprFromString("early")
    fails { foo(0, early, null, 99, null, 0, null, null, null, 0, null, null, null, 0, null, null, null, null) { _, _, _, _ -> } }
    check(early.isClosed)
    val aliased = payload("a", "A")
    fails { invoke("k", aliased, aliased) { _, _, _, _ -> } }
    check(aliased.isClosed)
    fails { invoke("panic", payload("x", "A"), payload("y", "B")) { _, _, _, _ -> } }
    // One native resource can be consumed by exactly one concurrent caller.
    val shared = keyexprFromString("race")
    val go = CountDownLatch(1)
    val won = AtomicInteger()
    val threads = (0 until 8).map { Thread {
        go.await()
        try { check(keyexprText(shared) == "race"); won.incrementAndGet() }
        catch (_: IllegalArgumentException) { }
    }.also { it.start() } }
    go.countDown(); threads.forEach { it.join() }
    check(won.get() == 1 && shared.isClosed)
    // Existing declaration forms configure slots in the same pipeline.
    check(echoKey(1, null, "declared") == "declared")
    val echoInput = keyexprFromString("handle")
    check(echoKey(0, echoInput, null) == "handle" && echoInput.isClosed)
    val rawInput = keyexprFromString("raw")
    val rawOutput = echoRaw(rawInput)
    check(rawInput.isClosed && keyexprText(rawOutput) == "raw")
    var abandoned: KeyExpr? = null
    try {
        keyexprPair(1, null, "builder") { h, s ->
            check(s == "builder")
            abandoned = h
            error("builder failed")
        }
        error("expected builder failure")
    } catch (failure: IllegalStateException) {
        check(failure.message == "builder failed")
    }
    check(abandoned!!.isClosed)
    io.prebindgen.demo.extra.doNothing()
    noop()
    println("JNI PASS: 14 combinations, output ownership, inactive branches, failures and concurrent consumption")
}
