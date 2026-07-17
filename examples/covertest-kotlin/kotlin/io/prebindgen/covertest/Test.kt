package io.prebindgen.covertest

import io.prebindgen.covertest.analytics.Summary
import io.prebindgen.covertest.analytics.SummaryVault
import io.prebindgen.covertest.analytics.archiveLatest
import io.prebindgen.covertest.analytics.archiveNew
import io.prebindgen.covertest.analytics.archiveStore
import io.prebindgen.covertest.analytics.storageExpectSummary
import io.prebindgen.covertest.analytics.storageMatchesSummary
import io.prebindgen.covertest.analytics.storageSummary
import io.prebindgen.covertest.analytics.storageSummaryProbe
import io.prebindgen.covertest.analytics.describeSummary
import io.prebindgen.covertest.analytics.storageSummaryFull
import io.prebindgen.covertest.analytics.storageSummaryHandle
import io.prebindgen.covertest.analytics.summaryPrefer
import io.prebindgen.covertest.analytics.summaryTotalOpt
import io.prebindgen.covertest.analytics.summaryTotalRaw
import io.prebindgen.covertest.errors.StorageErrorHandler
import io.prebindgen.covertest.model.Annotated
import io.prebindgen.covertest.model.Priority
import io.prebindgen.covertest.model.Stamp
import io.prebindgen.covertest.model.annotatedNew
import io.prebindgen.covertest.model.celsiusDouble
import io.prebindgen.covertest.model.labelReverse
import io.prebindgen.covertest.model.percentScale
import io.prebindgen.covertest.model.annotatedPayloadValue
import io.prebindgen.covertest.model.annotatedPriority
import io.prebindgen.covertest.model.annotatedTtl
import io.prebindgen.covertest.model.payloadPriority
import io.prebindgen.covertest.model.priorityOr
import io.prebindgen.covertest.model.priorityWeight
import io.prebindgen.covertest.model.stampNew
import io.prebindgen.covertest.model.stampSeries
import io.prebindgen.covertest.storage.addMillis
import io.prebindgen.covertest.storage.payloadHandlerNew
import io.prebindgen.covertest.storage.payloadVecHandlerNew
import io.prebindgen.covertest.storage.storageCallback
import io.prebindgen.covertest.storage.storageCallbackVec
import io.prebindgen.covertest.storage.storageEmit
import io.prebindgen.covertest.storage.storageGet
import io.prebindgen.covertest.storage.storageGetVec
import io.prebindgen.covertest.storage.storageHandlerNew
import io.prebindgen.covertest.storage.storageLabels
import io.prebindgen.covertest.storage.storageNew
import io.prebindgen.covertest.storage.storagePutByRead
import io.prebindgen.covertest.storage.storagePutByTake
import io.prebindgen.covertest.storage.storagePutOpt
import io.prebindgen.covertest.storage.storagePutSlice
import io.prebindgen.covertest.storage.storageShards
import io.prebindgen.covertest.storage.storageShardsOpt
import io.prebindgen.covertest.storage.storageTotalLen
import io.prebindgen.covertest.storage.storageTryWithLabel
import java.util.concurrent.atomic.AtomicInteger
import kotlin.concurrent.thread

/**
 * Correctness test for `covertest-kotlin`: drives **every** JniGen feature the
 * binding exercises (see `build.rs`) and asserts the native result. Unlike
 * `perftest-kotlin` (a benchmark), this is a pass/fail coverage harness — any
 * failed [check] aborts with a non-zero exit so `./gradlew run` surfaces it.
 *
 * Generic onError handler that never expects to fire on the happy paths.
 * `JniErrorHandler<out R>` is covariant, so a single `<Nothing>` instance is
 * assignable everywhere an error handler of any `R` is required.
 */
private val boom = JniErrorHandler<Nothing> { je ->
    throw AssertionError("unexpected native error: $je")
}

/** Same idea as [boom] for the `Result` error channel's dedicated handler type. */
private val boomStorage = StorageErrorHandler<Nothing> { je, message, handle ->
    handle.close()
    throw AssertionError("unexpected storage error: je=$je message=$message")
}

/** Thrown by the [StorageErrorHandler] used to probe the `Result` error channel. */
private class LabelError(val detail: String) : RuntimeException(detail)

private var sectionCount = 0

private inline fun section(name: String, body: () -> Unit) {
    body()
    sectionCount++
    println("ok   - $name")
}

private fun payload(id: Long, seq: Int, value: Double, flag: Boolean, label: String?) =
    Payload(id, seq, value, flag, label)

fun main() {
    println("covertest-kotlin: exercising every JniGen feature")

    // ── consts: eagerly-initialized top-level vals, one per value source —
    // #[prebindgen] const (bare constant!), nullary #[prebindgen] fn (.fun),
    // binding-local fn by path (.with), binding-defined expression (.expr) ────
    section("top-level const vals (all four value sources)") {
        check(COVER_MAGIC == 0xC0FFEE.toLong())
        check(COVER_TAG == "covertest")
        check(COVER_TAG_RUNTIME == "covertest-runtime")
        check(COVER_VERSION.startsWith("cover-"))
        check(COVER_BANNER == "covertest:0xc0ffee")
    }

    // ── data_class: fields cross as leaves, reassembled via fromParts ─────────
    section("data_class Payload") {
        val p = Payload(1L, 2, 3.5, true, "hello")
        check(p.id == 1L && p.seq == 2 && p.value == 3.5 && p.flag && p.label == "hello")
        check(Payload.fromParts(9L, 9, 9.0, false, null).label == null)
    }

    // ── enum_class: return / by-value param / Option<enum> param ─────────────
    section("enum_class Priority") {
        check(payloadPriority(payload(1L, 0, 3.0, false, null), boom) == Priority.LOW)
        check(payloadPriority(payload(1L, 0, 50.0, false, null), boom) == Priority.HIGH)
        check(payloadPriority(payload(1L, 0, 500.0, false, null), boom) == Priority.NORMAL)
        check(priorityWeight(Priority.LOW, boom) == 1)
        check(priorityWeight(Priority.NORMAL, boom) == 5)
        check(priorityWeight(Priority.HIGH, boom) == 10)
        // Option<enum>: null falls back, present overrides.
        check(priorityOr(null, Priority.NORMAL, boom) == Priority.NORMAL)
        check(priorityOr(Priority.LOW, Priority.HIGH, boom) == Priority.LOW)
        // enum_class surface: value + fromInt round-trip.
        check(Priority.HIGH.value == 2)
        check(Priority.fromInt(0) == Priority.LOW)
    }

    // ── value_class: by-value bytes, instance accessors, Vec<value> → List ────
    section("value_class Stamp") {
        val st: Stamp = stampNew(7L, 42L, boom)
        check(st.secs(boom) == 7L)
        check(st.nanos(boom) == 42L)
        val series: List<Stamp> = stampSeries(3L, boom)
        check(series.size == 3)
        check(series[0].secs(boom) == 0L)
        check(series[2].secs(boom) == 2L && series[2].nanos(boom) == 0L)
        check(stampSeries(0L, boom).isEmpty())
    }

    // ── Option<scalar> nullable primitive return + data_class instance
    // member (I5): the receiver crosses as `this`'s field leaves ────────────
    section("Option<i64> Payload.labelLen") {
        check(payload(1L, 0, 0.0, false, "abcd").labelLen(boom) == 4L)
        check(payload(1L, 0, 0.0, false, null).labelLen(boom) == null)
    }

    // ── ptr_class members + Option<Payload>/Option<Vec>/Vec round-trips ──────
    section("Storage members + Option/Vec round-trips") {
        val s = storageNew(boom)
        check(s.len(boom) == 0L)

        storagePutByTake(s, payload(42L, 1, 1.0, false, "a"), boom)
        check(s.len(boom) == 1L)                       // accessor
        check(s.contains(42L, boom))                   // method (true)
        check(!s.contains(7L, boom))                   // method (false)
        check(storageGet(s, boom) == payload(42L, 1, 1.0, false, "a")) // Option<Payload> Some

        storagePutByRead(s, payload(43L, 2, 2.0, true, null), boom)
        check(storageGet(s, boom)?.id == 43L)

        val batch = listOf(payload(1L, 1, 10.0, false, "x"), payload(2L, 2, 30.0, true, null))
        storagePutSlice(s, batch, boom)               // Vec<Payload> / &[Payload] input
        check(storageGetVec(s, boom) == batch)        // Option<Vec<Payload>> Some
        check(s.len(boom) == 2L)

        storagePutSlice(s, emptyList(), boom)
        check(storageGetVec(s, boom) == null)         // Option<Vec> None
        check(storageGet(s, boom) == null)            // Option<Payload> None
        s.close()
    }

    // ── constructor (companion factory) ──────────────────────────────────────
    section("constructor Storage.withPayload") {
        val s = Storage.withPayload(payload(99L, 0, 0.0, false, "z"), boom)
        check(s.len(boom) == 1L)
        check(s.contains(99L, boom))
        s.close()
    }

    // ── .interface() hatch (#54): each generated class emits a `<Name>Api`
    // interface; the HAND-WRITTEN CovResource/Timestamped/Ranked interfaces
    // EXTEND those and add default members that call the class's real
    // generated members — used here polymorphically, no generated-code edits ──
    section(".interface() hatch (Api interfaces extended by SDK interfaces)") {
        // ptr class: Storage implements StorageApi; CovResource : StorageApi.
        val s = storageNew(boom)
        val r: CovResource = s
        check(r.live)                     // default over inherited peek()/isClosed()
        check(r.isEmpty())                // default over class-specific len()
        check(r.len(boom) == 0L)          // generated member through the interface
        storagePutByTake(s, payload(7L, 0, 0.0, false, null), boom)
        check(!r.isEmpty())
        check(r.len(boom) == 1L)
        s.close()
        check(!r.live)
        check(r.isClosed() && r.peek() == 0L)

        // data class: Payload implements PayloadApi; Timestamped : PayloadApi.
        val fresh: Timestamped = payload(1L, 5, 0.0, false, null)
        val stale: Timestamped = payload(1L, 0, 0.0, false, null)
        check(fresh.fresh && !stale.fresh)
        check(fresh.seq == 5)             // generated field through the interface

        // enum class: Priority implements PriorityKind + Ranked.
        val hi: Ranked = Priority.HIGH
        check(hi.outranks(Priority.LOW))  // default over generated `value`
        check(!Priority.LOW.outranks(Priority.HIGH))
    }

    // ── impl Fn callbacks: single-payload + whole-batch ──────────────────────
    section("callbacks (impl Fn single + slice)") {
        val s = storageNew(boom)
        storagePutSlice(
            s,
            listOf(payload(1L, 0, 0.0, false, null), payload(2L, 0, 0.0, false, null), payload(3L, 0, 0.0, false, null)),
            boom,
        )

        // payload_handler_new: closure decoded once, fires once per payload.
        var perElem = 0L
        val h = payloadHandlerNew(PayloadCallback { p -> perElem += p.id }, boom)
        storageCallback(s, h, boom)
        check(perElem == 6L)
        h.close()

        // payload_vec_handler_new: whole batch delivered once as List<Payload>.
        var batchSize = -1
        var batchSum = 0L
        val vh: PayloadVecHandler = payloadVecHandlerNew(
            PayloadListCallback { list -> batchSize = list.size; batchSum = list.sumOf { it.id } },
            boom,
        )
        storageCallbackVec(s, vh, boom)
        check(batchSize == 3)
        check(batchSum == 6L)
        vh.close()
        s.close()
    }

    // ── flatten matrix on Summary: output (default/suppress/with) ────────────
    section("flatten_output (default / suppress / with)") {
        val s = storageNew(boom)
        storagePutSlice(s, listOf(payload(1L, 0, 10.0, false, null), payload(2L, 0, 30.0, false, null)), boom)

        // flatten_output DEFAULT: decompose into (count, total) leaves via builder.
        val pair = storageSummary(s, boom) { count, total -> count to total }
        check(pair.first == 2L && pair.second == 40.0)

        // flatten_output_suppress: keep the raw opaque handle.
        val raw: Summary = storageSummaryHandle(s, boom)
        check(raw.count(boom) == 2L)          // accessor on handle (non-consuming)
        check(raw.total(boom) == 40.0)
        check(raw.scaled(2.0, boom) == 80.0)  // method on handle
        // flatten_input_suppress: consume the raw handle to read its total.
        check(summaryTotalRaw(raw, boom) == 40.0)

        // flatten_output_with: custom field set that ALSO keeps the self handle.
        var fullHandle: Summary? = null
        val full = storageSummaryFull(s, boom) { count, total, handle ->
            fullHandle = handle
            count to total
        }
        check(full.first == 2L && full.second == 40.0)
        check(fullHandle!!.total(boom) == 40.0)
        fullHandle!!.close()
        s.close()
    }

    // ── binding-local field: fun!(crate::…).sig(sig!).name("handle") ────────
    // A CUSTOM field computed by a fn defined in THIS binding crate
    // (crate::summary_if_nonempty, src/lib.rs) — no source-crate item behind
    // it, declared with the same fun!+sig! vocabulary as every binding-local
    // fn. This exercise uses it for CONDITIONAL delivery (one use among
    // many): the handle leaf is gated by the binding-side predicate — the
    // zenoh "Encoding handle only when schema-carrying" idiom. Condition
    // fails ⇒ the leaf is null (no native clone, no wrapper); holds ⇒ a live
    // owned handle arrives with the values.
    section("binding-local field (fun! + sig!)") {
        val s = storageNew(boom)

        // Empty storage: count == 0 ⇒ the predicate fails ⇒ null handle.
        val emptyProbe = storageSummaryProbe(s, boom) { count, total, handle ->
            Triple(count, total, handle)
        }
        check(emptyProbe.first == 0L && emptyProbe.second == 0.0)
        check(emptyProbe.third == null) { "empty summary must arrive value-only" }

        // Non-empty: the handle arrives live alongside the decomposed values.
        storagePutSlice(s, listOf(payload(1L, 0, 10.0, false, null), payload(2L, 0, 30.0, false, null)), boom)
        val probe = storageSummaryProbe(s, boom) { count, total, handle ->
            Triple(count, total, handle)
        }
        check(probe.first == 2L && probe.second == 40.0)
        val h = probe.third ?: error("non-empty summary must deliver its handle")
        check(h.count(boom) == 2L && h.total(boom) == 40.0)
        h.close()
        s.close()
    }

    // ── binding-local FUNCTIONS: fun!(crate::…).sig(sig!(…)) ─────────────────
    // Full fns defined in the BINDING crate (covertest-kotlin/src/lib.rs),
    // exported through the ordinary FunctionDecl surface — free package fn,
    // instance method, companion constructor. No source-crate item exists for
    // any of them, yet converters, expansion defaults (describeSummary's `s`
    // param carries the Summary selector form), members and naming all apply
    // exactly as for #[prebindgen] fns.
    section("binding-local functions (fun!(crate::…) + sig!)") {
        // `mean` and `fromMean` carry NO .name(): the strip-class-prefix
        // method hook derives them from each path's LAST segment — automatic
        // mangling covers binding-local fns exactly like registry fns.
        // FALLIBLE companion constructor: the sig's `Result<Summary, String>`
        // return is the error channel — happy path first…
        val m = Summary.fromMean(4L, 2.5, boom)
        check(m.count(boom) == 4L && m.total(boom) == 10.0)
        // Instance method.
        check(m.mean(boom) == 2.5)
        // …then the Err arm: a negative count routes the Err's Display to
        // onError (a String error has no domain decomposition, so it arrives
        // as the je message), exactly like a #[prebindgen] fn's Result.
        var fromMeanErr: String? = null
        Summary.fromMean(-1L, 2.5) { je -> fromMeanErr = je; m }
        check(fromMeanErr == "summary count must be non-negative, got -1") {
            "unexpected fromMean error: $fromMeanErr"
        }
        // Free fn, selector form: build-arm (0) and handle-arm (1) both reach
        // the same binding-local Rust fn.
        check(describeSummary(0, 2L, 8.0, null, false, boom) == "2/8")
        check(describeSummary(1, null, null, m, true, boom) == "summary of 4 payloads totalling 10")
        m.close()
    }

    // ── flatten input on Summary: default + with, both selectors ─────────────
    section("flatten_input (default / with), leaves + handle") {
        val s = storageNew(boom)
        storagePutSlice(s, listOf(payload(1L, 0, 10.0, false, null), payload(2L, 0, 30.0, false, null)), boom)

        // constructor + accessors + method on the analytics handle.
        val sum = Summary.of(2L, 40.0, boom)
        check(sum.count(boom) == 2L && sum.total(boom) == 40.0 && sum.scaled(0.5, boom) == 20.0)
        sum.close()

        // #52 single-param `.split_on_param("expected")` on the CLASS-DEFAULT
        // `Summary` variants: idiomatic typed forms delegating to the selector.
        check(storageMatchesSummary(s, 2L, 40.0, boom))       // build-from-leaves arm
        check(!storageMatchesSummary(s, 1L, 40.0, boom))
        val h0 = Summary.of(2L, 40.0, boom)
        check(storageMatchesSummary(s, h0, boom))             // pass-handle arm
        // The selector form stays public underneath (raw arm dispatch).
        check(storageMatchesSummary(s, 0, 2L, 40.0, null, boom))

        // #52 single-param split via a per-fn `.expand_param` override.
        check(storageExpectSummary(s, 2L, 40.0, boom))        // build-from-leaves arm
        val h1 = Summary.of(2L, 40.0, boom)
        check(storageExpectSummary(s, h1, boom))              // pass-handle arm

        // #52 CARTESIAN PRODUCT: two split params → the 2×2 grid of typed
        // overloads, all four combinations distinct. Build args are prefixed
        // with the origin parameter name (`primaryCount`, `fallbackTotal`); the
        // handle arm consumes its `Summary`, so each is a fresh handle.
        check(summaryPrefer(2L, 40.0, 1L, 1.0, boom) == 1L)                       // build / build
        check(summaryPrefer(1L, 1.0, Summary.of(3L, 99.0, boom), boom) == 0L)     // build / handle
        check(summaryPrefer(Summary.of(3L, 99.0, boom), 1L, 1.0, boom) == 1L)     // handle / build
        check(
            summaryPrefer(Summary.of(1L, 1.0, boom), Summary.of(3L, 99.0, boom), boom) == 0L,
        )                                                                          // handle / handle

        // Optional combined-selector expansion: `Option<&Summary>` under the
        // dual-arm type default. The selector also encodes absence (-1 = None);
        // the borrow-identity arm CLONES, so the handle survives the call.
        check(summaryTotalOpt(-1, null, null, null, boom) == -1.0)     // absent
        check(summaryTotalOpt(0, 2L, 40.0, null, boom) == 40.0)        // build arm
        val hOpt = Summary.of(3L, 99.0, boom)
        check(summaryTotalOpt(1, null, null, hOpt, boom) == 99.0)      // borrow-identity arm
        check(hOpt.total(boom) == 99.0)                                // handle still live
        hOpt.close()

        // Auto-generated overloads coexist with a HAND-WRITTEN same-named one
        // (issue #52's manual path): `ManualOverloads.kt` adds another
        // `storageExpectSummary` — an `Int`-typed arm — in the analytics
        // package; Kotlin resolves it by signature alongside the generated ones.
        check(storageExpectSummary(s, 2, 40.0, boom))         // manual Int overload
        s.close()
    }

    // ── Result<_, E> → onError channel (ok + domain error) ───────────────────
    section("Result error channel storageTryWithLabel") {
        val ok = storageTryWithLabel("hi", boomStorage)
        check(ok.len(boom) == 1L)
        ok.close()

        // Domain error: `je` is null (no JNI exception); the StorageError's
        // flatten delivers its `message` field plus — via the type-level
        // `field_self` — the owned error handle itself, live and queryable.
        try {
            storageTryWithLabel("", StorageErrorHandler<Storage> { je, message, handle ->
                check(je == null) { "domain error should have a null jni exception, got $je" }
                check(!handle.isClosed())
                check(handle.message(boom) == "label must not be empty")
                handle.close()
                throw LabelError(message)
            })
            check(false) { "storageTryWithLabel(\"\") must fail" }
        } catch (e: LabelError) {
            check(e.detail == "label must not be empty")
        }
    }

    // ── input_wrapper / output_wrapper: Millis ⇄ Long ────────────────────────
    // `addMillis` is `millis_add` renamed via the per-fn `.name()` override.
    section("input/output wrapper Millis -> Long (+ .name rename)") {
        check(addMillis(100L, 50L, boom) == 150L)
        check(addMillis(0L, 0L, boom) == 0L)
    }

    // ── convert! source kinds: trait impls and binding-local fns ────────────
    section("convert! via From/Into impls (Celsius -> Int)") {
        check(celsiusDouble(21, boom) == 42)
        check(celsiusDouble(-5, boom) == -10)
    }
    section("convert! via TryFrom (Percent -> Int, fallible input)") {
        check(percentScale(50, 2, boom) == 100)
        check(percentScale(30, 2, boom) == 60)
        // Out-of-range input: the TryFrom impl's Err(String) routes to
        // onError through the converter's error slot (je carries the
        // Display'd message).
        var msg: String? = null
        percentScale(150, 1) { je ->
            msg = je
            0
        }
        check(msg?.contains("percent out of range: 150") == true) {
            "percentScale(150) must report the range error, got: $msg"
        }
    }
    section("convert! via binding-local fns (Label -> String, fallible input)") {
        check(labelReverse("abc", boom) == "cba")
        // Empty label: the local fn's Err(String) routes to onError.
        var msg: String? = null
        labelReverse("") { je ->
            msg = je
            ""
        }
        check(msg?.contains("label must not be empty") == true) {
            "labelReverse(\"\") must report the empty-label error, got: $msg"
        }
    }

    // ── Vec<opaque-handle> return: the Kotlin-side handle fold ───────────────
    section("Vec<Storage> handle fold (storageShards / storageShardsOpt)") {
        val shards = storageShards(3L, 2L, boom)
        check(shards.size == 3)
        check(shards.all { it.len(boom) == 2L })
        check(shards[2].contains(2001L, boom))   // distinct, correctly-typed handles
        check(!shards[0].contains(2001L, boom))
        shards.forEach { it.close() }
        check(storageShards(0L, 2L, boom).isEmpty())
        // Option<Vec<handle>>: the same fold under the null niche.
        check(storageShardsOpt(0L, 2L, boom) == null)
        val some = storageShardsOpt(2L, 1L, boom)!!
        check(some.size == 2 && some.all { it.len(boom) == 1L })
        some.forEach { it.close() }
    }

    // ── owned-handle callback: raw jlong + Kotlin wrap-and-close proxy ───────
    section("owned-handle callback (impl Fn(Storage))") {
        var seenLen = -1L
        var openInRun = false
        var escaped: Storage? = null
        val h = storageHandlerNew(
            StorageCallback { st ->
                openInRun = !st.isClosed()
                seenLen = st.len(boom)
                escaped = st
            },
            boom,
        )
        storageEmit(5L, h, boom)
        check(openInRun && seenLen == 5L)
        // close-unless-taken: the proxy closed the handle after run.
        check(escaped!!.isClosed())
        h.close()
    }

    // ── nested data_class + Option<prim>/Option<enum> FIELDS ─────────────────
    section("nested data_class Annotated + Option fields") {
        val p = payload(7L, 1, 2.5, true, "x")
        val a = annotatedNew(p, 30L, Priority.HIGH, boom)   // output: nested fromParts
        check(a.payload == p && a.ttl == 30L && a.priority == Priority.HIGH)
        check(annotatedTtl(a, boom) == 30L)                 // input: (present, value) pair
        check(annotatedPriority(a, boom) == Priority.HIGH)  // Option<enum> return
        check(annotatedPayloadValue(a, boom) == 2.5)        // nested field survived decode
        val none = annotatedNew(payload(1L, 0, 0.0, false, null), null, null, boom)
        check(annotatedTtl(none, boom) == null && annotatedPriority(none, boom) == null)
        // Kotlin-constructed instance crosses the input path too.
        val c = Annotated(payload(2L, 0, 9.0, false, null), 5L, Priority.LOW)
        check(annotatedTtl(c, boom) == 5L)
        check(annotatedPriority(c, boom) == Priority.LOW)
        check(annotatedPayloadValue(c, boom) == 9.0)
    }

    // ── borrowed-opaque output: Option<&Summary> → cloned owned handle ───────
    // `Archive` is renamed to `SummaryVault` via the per-class `.name()`
    // override — the explicit type annotation asserts the rename.
    section("borrowed-opaque output archiveLatest") {
        val a: SummaryVault = archiveNew(boom)
        check(archiveLatest(a, boom) == null)               // None → null
        val s = Summary.of(2L, 40.0, boom)
        archiveStore(a, 1, null, null, s, boom)             // flatten-input, handle arm
        val first = archiveLatest(a, boom)!!
        val second = archiveLatest(a, boom)!!
        check(first.count(boom) == 2L && first.total(boom) == 40.0)
        first.close()                                       // clones are independent…
        check(second.total(boom) == 40.0)                   // …of each other
        second.close()
        val third = archiveLatest(a, boom)!!                // …and of the archived value
        check(third.total(boom) == 40.0)
        third.close()
        archiveStore(a, 0, 3L, 60.0, null, boom)            // flatten-input, leaves arm
        val fourth = archiveLatest(a, boom)!!
        check(fourth.count(boom) == 3L && fourth.total(boom) == 60.0)
        fourth.close()
        a.close()
    }

    // ── Vec<String> fold + Option<data-class> input + plain String return ────
    section("Vec<String> storageLabels + Option<Payload> input + String return") {
        val s = storageNew(boom)
        check(storageLabels(s, boom).isEmpty())
        storagePutSlice(
            s,
            listOf(payload(1L, 0, 0.0, false, "a"), payload(2L, 0, 0.0, false, null), payload(3L, 0, 0.0, false, "c")),
            boom,
        )
        check(storageLabels(s, boom) == listOf("a", "c"))
        check(storagePutOpt(s, payload(4L, 0, 0.0, false, "d"), boom))   // Some → pushed
        check(!storagePutOpt(s, null, boom))                              // None → not
        check(s.len(boom) == 4L)
        check(storageLabels(s, boom) == listOf("a", "c", "d"))
        check(stringNew("hello", boom) == "hello")
        check(stringNew("", boom) == "")
        s.close()
    }

    // ── binding error: je != null (value-blob length guard) ──────────────────
    section("binding error je != null (malformed Stamp bytes)") {
        val bogus = Stamp(ByteArray(3))   // Stamp is 16 bytes; 3 must be rejected
        var je: String? = null
        val fallback = bogus.secs(JniErrorHandler { e ->
            je = e
            -1L
        })
        check(fallback == -1L)
        check(je != null && je!!.contains("wrong byte length")) { "unexpected je: $je" }
    }

    // ── callback exceptions: swallowed per upcall (no-throw contract) ────────
    // A callback that throws must not corrupt the surrounding native call: the
    // trampoline describes + clears the pending exception per upcall (the stack
    // trace printed below is EXPECTED output) and delivery continues.
    section("callback exceptions are swallowed (no-throw contract)") {
        val s = storageNew(boom)
        storagePutSlice(s, listOf(payload(1L, 0, 0.0, false, null), payload(2L, 0, 0.0, false, null)), boom)
        var fired = 0
        val h = payloadHandlerNew(
            PayloadCallback { fired++; throw RuntimeException("deliberate covertest exception") },
            boom,
        )
        storageCallback(s, h, boom)   // must not throw at the call site
        check(fired == 2) { "every payload must still be delivered, got $fired" }
        storageCallback(s, h, boom)   // the handler stays usable
        check(fired == 4)
        h.close()
        s.close()
    }

    // ── 3-handle sorted locking + concurrent smoke ───────────────────────────
    section("3-handle locking + 2-thread smoke") {
        val s1 = Storage.withPayload(payload(1L, 0, 0.0, false, null), boom)
        val s2 = Storage.withPayload(payload(2L, 0, 0.0, false, null), boom)
        val s3 = storageNew(boom)
        check(storageTotalLen(s1, s2, s3, boom) == 2L)
        check(storageTotalLen(s3, s2, s1, boom) == 2L)   // argument order irrelevant
        // Opposite lock-acquisition orders + a writer on a shared handle: the
        // sorted N-ary locking must neither deadlock nor tear.
        val iterations = 2_000
        val errs = AtomicInteger()
        val s4 = storageNew(boom)
        val workers = listOf(
            thread { repeat(iterations) { if (storageTotalLen(s1, s2, s3, boom) != 2L) errs.incrementAndGet() } },
            thread { repeat(iterations) { if (storageTotalLen(s3, s2, s1, boom) != 2L) errs.incrementAndGet() } },
            thread { repeat(iterations) { storagePutByTake(s4, payload(9L, 0, 0.0, false, null), boom) } },
            thread { repeat(iterations) { if (storageTotalLen(s4, s1, s2, boom) > 3L) errs.incrementAndGet() } },
        )
        workers.forEach { it.join(30_000) }
        check(workers.none { it.isAlive }) { "deadlock: worker threads still alive" }
        check(errs.get() == 0) { "${errs.get()} inconsistent reads under concurrency" }
        check(s4.len(boom) == 1L)   // put_by_take always leaves a 1-element batch
        listOf(s1, s2, s3, s4).forEach { it.close() }
    }

    // ── close/take storm vs N-ary locking: lock-order stability + closed-race ─
    // Regression test for prebindgen#35 (lock ordering keyed by a MUTABLE ptr:
    // a concurrent close() moved a handle across the sort order, letting two
    // threads acquire the same pair of monitors in opposite orders — AB/BA
    // deadlock) and prebindgen#34 (a close between the wrapper's pre-lock guard
    // and the native call passed a dead pointer into Rust — UB/SIGSEGV).
    // Readers hammer the 3-handle storageTotalLen over a shared pool while
    // stormers close()/take() the same handles and swap in fresh ones. With the
    // tag-bit lifecycle the sort key (ptr and -2) is immutable, so no deadlock
    // (watchdog); a closed handle racing a call must surface via onError as
    // "closed native handle" — never a crash, never any other error.
    section("close/take storm (lock-order stability + closed-handle race)") {
        val slots = 4
        val pool = java.util.concurrent.atomic.AtomicReferenceArray<Storage>(slots)
        for (i in 0 until slots) pool.set(i, storageNew(boom))
        val stop = java.util.concurrent.atomic.AtomicBoolean(false)
        val closedRaces = AtomicInteger()
        val unexpected = java.util.concurrent.atomic.AtomicReference<String?>(null)
        val tolerant = JniErrorHandler<Long> { je ->
            if (je != null && je.contains("closed native handle")) closedRaces.incrementAndGet()
            else unexpected.compareAndSet(null, je ?: "je == null")
            -1L
        }
        val readers = List(4) {
            thread {
                val rnd = java.util.concurrent.ThreadLocalRandom.current()
                while (!stop.get()) {
                    val a = pool.get(rnd.nextInt(slots))
                    val b = pool.get(rnd.nextInt(slots))
                    val c = pool.get(rnd.nextInt(slots))
                    storageTotalLen(a, b, c, tolerant)
                }
            }
        }
        val stormers = List(2) {
            thread {
                val rnd = java.util.concurrent.ThreadLocalRandom.current()
                repeat(3_000) { n ->
                    val i = rnd.nextInt(slots)
                    val old = pool.getAndSet(i, storageNew(boom))
                    when (n % 3) {
                        0 -> old.close()
                        // take(): the twin shares the old handle's masked
                        // address (an intentional sort-key tie) — the old
                        // object is closed before the twin exists.
                        1 -> old.take().close()
                        else -> { old.close(); old.close() }   // idempotent
                    }
                }
            }
        }
        stormers.forEach { it.join(60_000) }
        stop.set(true)
        readers.forEach { it.join(60_000) }
        check((stormers + readers).none { it.isAlive }) { "deadlock: storm threads still alive" }
        check(unexpected.get() == null) { "unexpected native error: ${unexpected.get()}" }
        check(closedRaces.get() > 0) { "storm never observed a closed handle — test is not racing" }
        for (i in 0 until slots) pool.get(i).close()
    }

    // ── high-volume callback: per-upcall local-frame hygiene ─────────────────
    // 20k upcalls, half carrying a fresh String local each — leaked JNI local
    // refs (the historical daemon-thread OOM) would accumulate here.
    section("high-volume callback (localref pressure)") {
        val s = storageNew(boom)
        val n = 5_000
        storagePutSlice(
            s,
            List(n) { payload(it.toLong(), it, it.toDouble(), false, if (it % 2 == 0) "L$it" else null) },
            boom,
        )
        var count = 0L
        var sum = 0L
        val h = payloadHandlerNew(PayloadCallback { p -> count++; sum += p.id }, boom)
        repeat(4) { storageCallback(s, h, boom) }
        check(count == 4L * n)
        check(sum == 4L * (n.toLong() - 1L) * n.toLong() / 2L)
        h.close()
        s.close()
    }

    // ── .gc_managed() lifecycle: release ticket + Cleaner backstop ───────────
    // Summary is gc_managed: its pointer lives in an atomic cell and every
    // release path (close / take / by-value consumption / the GC action)
    // settles the once-only untagged→tagged CAS ticket. The explicit paths
    // must behave exactly like a plain handle's; a use-after-free by any
    // double-settled ticket would crash the JVM in the churn loop below.
    section(".gc_managed() lifecycle (ticket + Cleaner backstop)") {
        // Explicit close stays primary and is idempotent.
        val a = Summary.of(2L, 40.0, boom)
        check(a.total(boom) == 40.0)
        a.close()
        check(a.isClosed())
        a.close() // double close: ticket already settled — no double free
        var closedErr: String? = null
        a.total { je -> closedErr = je; -1.0 }
        check(closedErr != null && closedErr!!.contains("closed native handle"))

        // take(): ticket moves into the fresh wrapper; the source is closed.
        val b = Summary.of(3L, 60.0, boom)
        val c = b.take()
        check(b.isClosed() && !c.isClosed())
        check(c.total(boom) == 60.0)
        b.close() // settled ticket: no-op
        c.close()

        // By-value consumption settles the ticket (markConsumed): the summary
        // is freed by Rust, and neither close nor the Cleaner may free again.
        val d = Summary.of(2L, 40.0, boom)
        check(summaryTotalRaw(d, boom) == 40.0)
        check(d.isClosed())
        d.close()

        // Cleaner backstop: churn unreachable handles through every state —
        // never-released (GC action must free), explicitly closed, consumed —
        // then force GC so the cleaner thread settles the survivors. Any
        // double free or free-under-use aborts the JVM here.
        repeat(2_000) { i ->
            val s = Summary.of(i.toLong(), i.toDouble(), boom)
            when (i % 3) {
                0 -> {} // dropped live: the Cleaner frees it
                1 -> s.close()
                2 -> check(summaryTotalRaw(s, boom) == i.toDouble())
            }
        }
        repeat(3) {
            System.gc()
            Thread.sleep(50)
        }
        // The world is still sane after the cleaner ran.
        val e = Summary.of(5L, 50.0, boom)
        check(e.count(boom) == 5L)
        e.close()
    }

    println("PASS - $sectionCount sections, every JniGen feature exercised")
}
