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
import io.prebindgen.covertest.analytics.summaryMerge
import io.prebindgen.covertest.analytics.summaryPrefer
import io.prebindgen.covertest.analytics.summarySeries
import io.prebindgen.covertest.analytics.summarySeriesOpt
import io.prebindgen.covertest.analytics.summaryTotalOpt
import io.prebindgen.covertest.analytics.summaryTotalRaw
import io.prebindgen.covertest.errors.StorageErrorHandler
import io.prebindgen.covertest.esc_pkg.Esc_Probe
import io.prebindgen.covertest.model.Annotated
import io.prebindgen.covertest.model.Arrays
import io.prebindgen.covertest.model.CacheConfig
import io.prebindgen.covertest.model.RepliesConfig
import io.prebindgen.covertest.model.DurationBoundary
import io.prebindgen.covertest.model.ObjectBoundary
import io.prebindgen.covertest.model.ObjectBoundary2
import io.prebindgen.covertest.model.ObjectBoundary4
import io.prebindgen.covertest.model.ObjectBoundary8
import io.prebindgen.covertest.model.ObjectBoundary16
import io.prebindgen.covertest.model.ObjectBoundary32
import io.prebindgen.covertest.model.ObjectBoundary63
import io.prebindgen.covertest.model.ObjectBoundary64
import io.prebindgen.covertest.model.ObjectBoundaryLeaf
import io.prebindgen.covertest.model.Priority
import io.prebindgen.covertest.model.Hold
import io.prebindgen.covertest.model.HoldPolicy
import io.prebindgen.covertest.model.Lookup
import io.prebindgen.covertest.model.Reading
import io.prebindgen.covertest.model.Stamp
import io.prebindgen.covertest.model.Unsigned
import io.prebindgen.covertest.model.annotatedNew
import io.prebindgen.covertest.model.arraysEcho
import io.prebindgen.covertest.model.blobValueEcho
import io.prebindgen.covertest.Holder
import io.prebindgen.covertest.WrappedFields
import io.prebindgen.covertest.model.boxedElemIdSum
import io.prebindgen.covertest.model.boxedLatest
import io.prebindgen.covertest.model.boxedOptPayloadId
import io.prebindgen.covertest.model.boxedOptPriorityWeight
import io.prebindgen.covertest.model.boxedPayloadId
import io.prebindgen.covertest.model.boxedRunIdSum
import io.prebindgen.covertest.model.holderTagOr
import io.prebindgen.covertest.model.wrappedFieldsSum
import io.prebindgen.covertest.model.boxedNoteEcho
import io.prebindgen.covertest.model.plainNoteEcho
import io.prebindgen.covertest.model.blobValueNew
import io.prebindgen.covertest.model.annotatedAlternateValue
import io.prebindgen.covertest.model.celsiusDouble
import io.prebindgen.covertest.model.boxedDurationEcho
import io.prebindgen.covertest.model.durationOptional
import io.prebindgen.covertest.model.durationBoundaryEcho
import io.prebindgen.covertest.model.spanHolderNew
import io.prebindgen.covertest.model.durationEmit
import io.prebindgen.covertest.model.durationOutOfRange
import io.prebindgen.covertest.model.holdEcho
import io.prebindgen.covertest.model.holdPolicyEcho
import io.prebindgen.covertest.model.labelReverse
import io.prebindgen.covertest.model.labelSeriesEcho
import io.prebindgen.covertest.model.percentInvalidOutput
import io.prebindgen.covertest.model.percentOptional
import io.prebindgen.covertest.model.percentScale
import io.prebindgen.covertest.model.annotatedPayloadValue
import io.prebindgen.covertest.model.annotatedPriority
import io.prebindgen.covertest.model.annotatedTtl
import io.prebindgen.covertest.model.cacheConfigWeight
import io.prebindgen.covertest.model.objectBoundaryValue
import io.prebindgen.covertest.model.Observation
import io.prebindgen.covertest.model.observationNew
import io.prebindgen.covertest.model.observationWhich
import io.prebindgen.covertest.model.lookupEach
import io.prebindgen.covertest.model.ledgerEach
import io.prebindgen.covertest.model.ledgerNew
import io.prebindgen.covertest.model.reportEach
import io.prebindgen.covertest.model.lookupOf
import io.prebindgen.covertest.model.archiveReading
import io.prebindgen.covertest.model.archiveReadingMaybe
import io.prebindgen.covertest.model.archiveSetReading
import io.prebindgen.covertest.model.readingEach
import io.prebindgen.covertest.model.readingMaybe
import io.prebindgen.covertest.model.readingOf
import io.prebindgen.covertest.model.readingSeries
import io.prebindgen.covertest.model.Marker
import io.prebindgen.covertest.model.Tagged
import io.prebindgen.covertest.model.markerOf
import io.prebindgen.covertest.model.taggedNew
import io.prebindgen.covertest.model.taggedRank
import io.prebindgen.covertest.model.payloadPriority
import io.prebindgen.covertest.model.priorityOr
import io.prebindgen.covertest.model.priorityWeight
import io.prebindgen.covertest.model.stampNew
import io.prebindgen.covertest.model.stampSeries
import io.prebindgen.covertest.model.unsignedEmit
import io.prebindgen.covertest.model.unsignedDataMaybe
import io.prebindgen.covertest.model.unsignedOptional
import io.prebindgen.covertest.model.unsignedRoundTrip
import io.prebindgen.covertest.model.unsignedSeries
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
import io.prebindgen.covertest.storage.storageTryFromStamp
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

/** Same idea as [boom] for the typed domain `onError` channel (issue #45: the
 *  domain handler no longer carries `je` — that is the separate binding channel). */
private val boomStorage = StorageErrorHandler<Nothing> { message, handle ->
    handle.close()
    throw AssertionError("unexpected storage error: message=$message")
}

/** Thrown by the [StorageErrorHandler] used to probe the domain error channel. */
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

    // ── #108: fixed-width unsigned scalars. Small widths widen losslessly;
    // u64 keeps all bits through the public ULong ↔ raw Long projection. ─────
    section("fixed-width unsigned scalars") {
        val max = unsignedRoundTrip(
            UByte.MAX_VALUE.toInt(),
            UShort.MAX_VALUE.toInt(),
            UInt.MAX_VALUE.toLong(),
            ULong.MAX_VALUE,
            ULong.MAX_VALUE,
            boom,
        )
        check(
            max == Unsigned(
                UByte.MAX_VALUE.toInt(),
                UShort.MAX_VALUE.toInt(),
                UInt.MAX_VALUE.toLong(),
                ULong.MAX_VALUE,
                ULong.MAX_VALUE,
            ),
        ) { "unsigned max round trip mismatch: $max" }
        check(unsignedOptional(null, boom) == null)
        check(unsignedOptional(ULong.MAX_VALUE, boom) == ULong.MAX_VALUE)
        check(unsignedDataMaybe(max, boom) == ULong.MAX_VALUE)
        check(unsignedDataMaybe(max.copy(maybeLong = null), boom) == null)

        var emitted = 0uL
        unsignedEmit(ULong.MAX_VALUE, u64Callback { emitted = it }, boom)
        check(emitted == ULong.MAX_VALUE)
        check(unsignedSeries(boom) == listOf(0uL, ULong.MAX_VALUE))

        fun expectRangeError(
            byte: Int,
            short: Int,
            int: Long,
            expected: String,
        ) {
            var message: String? = null
            val fallback = Unsigned(0, 0, 0L, 0uL, null)
            val result = unsignedRoundTrip(byte, short, int, 0uL, null) { je ->
                message = je
                fallback
            }
            check(result == fallback)
            check(message?.contains(expected) == true) { "unexpected range error: $message" }
        }
        expectRangeError(-1, 0, 0L, "u8 input out of range: -1")
        expectRangeError(0, 65_536, 0L, "u16 input out of range: 65536")
        expectRangeError(0, 0, 4_294_967_296L, "u32 input out of range: 4294967296")
    }

    // ── bounded custom representation: Rust keeps Option<Duration>, Kotlin
    // sees ULong?, and JNI uses an invalid u64 bit pattern for null so the
    // native carrier remains primitive long rather than JObject/boxed Long. ─
    section("bounded Option<Duration> niche over raw Long") {
        val native = CovNative::class.java.getDeclaredMethod(
            "durationOptional",
            java.lang.Long.TYPE,
            Any::class.java,
        )
        check(native.parameterTypes[0] == java.lang.Long.TYPE)
        check(native.returnType == java.lang.Long.TYPE) {
            "bounded Option<Duration> must use a primitive Long JNI carrier"
        }

        check(durationOptional(null, boom) == null)
        check(durationOptional(0uL, boom) == 0uL)
        check(durationOptional(86_400_000uL, boom) == 86_400_000uL)

        // A `Box<Duration>` crosses as a bare `Duration` does — the wrapper is
        // invisible to Kotlin, which is why the model erases it. Both
        // directions run the full staged chain; skipping it put `Box::new`
        // around a `u64` and did not compile (#309).
        check(boxedDurationEcho(86_400_000uL, boom) == 86_400_000uL)
        check(boxedDurationEcho(0uL, boom) == 0uL)

        // The data-class properties are semantic `ULong` / `ULong?`, while the
        // native output factory receives primitive Longs (the optional one
        // niche-encoded). The echo's explicit object input also executes the
        // complete ULong -> Duration decoder.
        //
        // `required` and `delay` take DIFFERENT emitter paths: `delay` rides
        // the `Option<_>` wrapper, which composes its inner conversion chain
        // itself, while `required` is a bare leaf the whole-object decoder and
        // the leaf encoder each have to compose for. Both fields therefore
        // have to round-trip, not just the nullable one.
        val fromParts = DurationBoundary::class.java.getDeclaredMethod(
            "fromParts",
            java.lang.Long.TYPE,
            java.lang.Long.TYPE,
        )
        check(fromParts.parameterTypes.all { it == java.lang.Long.TYPE })
        check(
            durationBoundaryEcho(DurationBoundary(0uL, null), boom) ==
                DurationBoundary(0uL, null),
        )
        check(
            durationBoundaryEcho(DurationBoundary(7uL, 12_345uL), boom) ==
                DurationBoundary(7uL, 12_345uL),
        )
        // The required field carries its own value rather than mirroring the
        // optional one — a chain wired to the wrong binding would cross them.
        check(
            durationBoundaryEcho(DurationBoundary(86_400_000uL, 1uL), boom) ==
                DurationBoundary(86_400_000uL, 1uL),
        )

        // The same two leaves under an OPTIONAL ancestor (#142) — the
        // combination `DurationBoundary` cannot reach, since its fields are
        // never absent as a pair. A conditional value form makes both nullable,
        // which crosses the two facts that decide the wrap: does the leaf carry
        // a niche of its own (`delay` yes, `required` no), and can an ancestor
        // be absent (both). Only the first grants a sentinel.
        //
        // `0uL` is the value that makes this observable: it is legal in the
        // declared range AND is what a zero-defaulted slot would carry, so a
        // wrap that confused the two absences would report it as `null`.
        check(spanHolderNew(0L, 0uL, 0L, boom) { req, del -> "$req/$del" } == "0/0")
        // …and the niche-carrying leaf still reads its own `None`, while its
        // sibling — which has no niche — is unaffected.
        check(spanHolderNew(0L, 5uL, -1L, boom) { req, del -> "$req/$del" } == "5/null")
        // Ancestor absent: BOTH leaves are null, through the `?.` alone. The
        // `required` leaf has no sentinel to be confused by; before #142's fix
        // its wrap tested `-1L`, a value its own encoder can never produce.
        check(spanHolderNew(-1L, 9uL, 3L, boom) { req, del -> "$req/$del" } == "null/null")
        check(spanHolderNew(0L, 86_400_000uL, 12_345L, boom) { req, del -> "$req/$del" }
            == "86400000/12345")

        // A whole-value CALLBACK argument is a third encoder path, independent
        // of the data-class and sum emitters above: the trampoline encodes the
        // arg itself, with no leaf plan to carry the chain. The converted
        // analogue of `unsignedEmit`.
        var emittedDuration = 0uL
        durationEmit(12_345uL, DurationCallback { emittedDuration = it }, boom)
        check(emittedDuration == 12_345uL)
        durationEmit(86_400_000uL, DurationCallback { emittedDuration = it }, boom)
        check(emittedDuration == 86_400_000uL)

        var inputError: String? = null
        val inputFallback = durationOptional(86_400_001uL) { je ->
            inputError = je
            7uL
        }
        check(inputFallback == 7uL)
        check(inputError?.contains("outside its declared domain") == true) {
            "invalid duration input did not report its domain error: $inputError"
        }

        var outputError: String? = null
        val outputFallback = durationOutOfRange { je ->
            outputError = je
            null
        }
        check(outputFallback == null)
        check(outputError?.contains("outside its declared domain") == true) {
            "invalid duration output did not report its domain error: $outputError"
        }
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

    // ── sealed_class: a data-carrying enum as a Kotlin `sealed interface` ─────
    // The Kotlin surface only (the wire lowering is a separate stage): every
    // variant shape, the nested placement, the per-variant rename, and
    // `fromParts` picking the live group by tag.
    section("sealed_class Reading (sum surface + fromParts)") {
        // A payload-less alternative is a `data object`; the rest are `data
        // class`es, all nested inside the interface.
        val missing: Reading = Reading.Missing
        val exact: Reading = Reading.Exact(42L)
        val range: Reading = Reading.Range(1L, 9L)
        // `variant!(Labeled).name("Tagged")` renamed the class AND its slots.
        val tagged: Reading = Reading.Tagged("warm", Priority.HIGH)

        check(exact is Reading.Exact && (exact as Reading.Exact).v0 == 42L)
        check(range is Reading.Range && (range as Reading.Range).high == 9L)
        check(tagged is Reading.Tagged && (tagged as Reading.Tagged).v1 == Priority.HIGH)
        // `data object` is a singleton and `data class` gives structural equality.
        check(missing === Reading.Missing)
        check(Reading.Exact(42L) == exact)

        // `when` over the sealed hierarchy is exhaustive with no `else` — the
        // point of a sum: there is no "both set" or "neither set" case.
        fun describe(r: Reading): String =
            when (r) {
                is Reading.Missing -> "missing"
                is Reading.Exact -> "exact ${r.v0}"
                is Reading.Range -> "range ${r.low}..${r.high}"
                is Reading.Tagged -> "${r.v0}/${r.v1.value}"
                is Reading.Companion -> "companion ${r.v0}"
            }
        check(describe(missing) == "missing")
        check(describe(exact) == "exact 42")
        check(describe(range) == "range 1..9")
        check(describe(tagged) == "warm/2")

        // `fromParts(tag, …every group's slots side by side…)`: the tag picks
        // the live group; the inert slots are ignored.
        check(Reading.fromParts(0, 0L, 0L, 0L, "", Priority.LOW, 0L) === Reading.Missing)
        check(Reading.fromParts(1, 42L, 0L, 0L, "", Priority.LOW, 0L) == exact)
        check(Reading.fromParts(2, 0L, 1L, 9L, "", Priority.LOW, 0L) == range)
        check(Reading.fromParts(3, 0L, 0L, 0L, "warm", Priority.HIGH, 0L) == tagged)

        // A variant may legitimately be named `Companion`: that name is the
        // generator's own default for the `fromParts` holder, not a Kotlin
        // reserved word, so the generator renamed ITS companion (to
        // `Companion_`) rather than obliging the source crate to rename a
        // domain variant. `fromParts` is still reached through the interface.
        val companion: Reading = Reading.Companion(5L)
        check(companion is Reading.Companion && (companion as Reading.Companion).v0 == 5L)
        check(Reading.fromParts(4, 0L, 0L, 0L, "", Priority.LOW, 5L) == companion)
        check(Reading.Companion_.fromParts(4, 0L, 0L, 0L, "", Priority.LOW, 5L) == companion)

        // A tag outside 0..N-1 is an error, never a variant.
        var invalid: String? = null
        try {
            Reading.fromParts(5, 0L, 0L, 0L, "", Priority.LOW, 0L)
        } catch (e: IllegalArgumentException) {
            invalid = e.message
        }
        check(invalid == "Reading: invalid tag 5")
    }

    // ── a sum payload that is a CONVERTED type ───────────────────────────────
    // `Hold.For` carries a `Duration`, whose boundary conversion is the
    // `convert!` chain `Duration -> u64 -> jlong` rather than a single wire
    // converter. Every sum emitter has to run the whole chain: reading only the
    // wire-facing converter passes the semantic value where the representation
    // is expected. Exercised at all three positions — the function's own return
    // (`holdEcho`), a required data-class field and an optional one
    // (`holdPolicyEcho`), the last of which also decodes the sum back off a
    // Kotlin property.
    section("sum payload crossing a convert! chain") {
        check(holdEcho(Hold.Indefinite, boom) == Hold.Indefinite)
        check(holdEcho(Hold.For(12_345uL), boom) == Hold.For(12_345uL))
        // The domain bounds still apply inside a variant group: the payload
        // converter is the same one a bare `Duration` uses.
        check(holdEcho(Hold.For(86_400_000uL), boom) == Hold.For(86_400_000uL))

        val both = holdPolicyEcho(HoldPolicy(Hold.For(7uL), Hold.Indefinite), boom)
        check(both == HoldPolicy(Hold.For(7uL), Hold.Indefinite))
        val absent = holdPolicyEcho(HoldPolicy(Hold.Indefinite, null), boom)
        check(absent == HoldPolicy(Hold.Indefinite, null))
        // The optional group's payload must survive independently of the
        // required one's — a chain wired to the wrong binding would cross them.
        val onlyGrace = holdPolicyEcho(HoldPolicy(Hold.Indefinite, Hold.For(99uL)), boom)
        check(onlyGrace == HoldPolicy(Hold.Indefinite, Hold.For(99uL)))
    }

    // ── a sum as a data-class FIELD, crossing Rust → Kotlin ───────────────────
    // The tag and every variant's group ride the parent's single `fromParts`;
    // inert groups are wire-defaulted, which is why an inert object slot must
    // be nullable (`reading_tagged_v0: String?`) and re-asserted `!!` in its
    // own live arm. No JVM object is built for the sum itself.
    section("sum as a data-class field (tag-gated groups on one fromParts)") {
        // Every alternative survives the crossing, including the payload-less
        // one and the two whose groups sit beside object-shaped slots.
        check(observationNew(0, false, boom).reading == Reading.Missing)
        check(observationNew(1, false, boom).reading == Reading.Exact(42L))
        check(observationNew(2, false, boom).reading == Reading.Range(1L, 9L))
        check(observationNew(3, false, boom).reading == Reading.Tagged("warm", Priority.HIGH))
        check(observationNew(4, false, boom).reading == Reading.Companion(5L))

        // The sum sits beside ordinary flattened leaves — they must not be
        // disturbed by the tag-gated groups interleaved with them.
        val obs = observationNew(3, false, boom)
        check(obs.id == 7L && obs.note == "obs")

        // `Option<sum>`: the present flag and the tag are independent facts,
        // so an absent optional is null regardless of what its tag slot holds.
        check(observationNew(1, false, boom).fallback == null)
        check(observationNew(1, true, boom).fallback == Reading.Range(1L, 9L))
        // …and an object-payload variant round-trips through the optional too.
        check(observationNew(2, true, boom).fallback == Reading.Tagged("warm", Priority.HIGH))
        // Both sums live at once, each with its own tag.
        val both = observationNew(4, true, boom)
        check(both.reading == Reading.Companion(5L) && both.fallback == Reading.Missing)

        // …and back IN as part of a data-class parameter: every alternative
        // reconstructs the same Rust variant it came from.
        for (which in 0..4) {
            check(observationWhich(observationNew(which, false, boom), boom) == which)
        }
        // A Kotlin-constructed value (not one that came from Rust) crosses in
        // just the same.
        check(observationWhich(Observation(1L, Reading.Range(2L, 3L), null, "n"), boom) == 2)
        check(
            observationWhich(
                Observation(1L, Reading.Tagged("x", Priority.LOW), Reading.Missing, "n"),
                boom,
            ) == 3
        )

        // A tag outside 0..N-1 reaches the binding-error channel — never a
        // panic across the boundary (the locked rule in the design). The
        // generated wrapper computes the tag from an exhaustive `when` and so
        // can never produce one, which is exactly why this calls the extern
        // directly: it is the only way to exercise the guard.
        var invalidTag: String? = null
        val cap = JniErrorHandlerCapture.acquire()
        CovNative.observationWhich(
            1L,
            99, 0L, 0L, 0L, null, 0, 0L,
            false,
            0, 0L, 0L, 0L, null, 0, 0L,
            "n",
            cap,
        )
        if (cap.failed) invalidTag = cap.ze0
        check(invalidTag != null && invalidTag!!.contains("Reading: invalid tag")) {
            "expected the binding-error channel to carry the invalid tag, got: $invalidTag"
        }
    }

    // ── a sum as the function's OWN return / callback argument ────────────────
    // Nothing surrounds the value here, so the decomposition carries its own
    // tag: the wrapper hands the native side a hoisted builder (or folder)
    // singleton, and the live group is picked by a `when` over that tag. Still
    // no JVM object built on the Rust side — only the tag and the raw slots
    // cross.
    section("sum in return position (tag + groups through a fixed builder)") {
        // Bare `E` return: every variant shape survives, including the
        // payload-less one and the ones whose groups carry object slots.
        check(readingOf(0, boom) == Reading.Missing)
        check(readingOf(1, boom) == Reading.Exact(42L))
        check(readingOf(2, boom) == Reading.Range(1L, 9L))
        check(readingOf(3, boom) == Reading.Tagged("warm", Priority.HIGH))
        check(readingOf(4, boom) == Reading.Companion(5L))
        // The payload-less alternative is still the singleton `data object` —
        // the tag alone rebuilt it, no group was read.
        check(readingOf(0, boom) === Reading.Missing)

        // `Option<E>` return: the present layer nulls the whole result and is
        // independent of the tag (which is 0 — `Missing` — in the null case,
        // and must not be mistaken for one).
        check(readingMaybe(-1, boom) == null)
        check(readingMaybe(0, boom) == Reading.Missing)
        check(readingMaybe(3, boom) == Reading.Tagged("warm", Priority.HIGH))

        // `Vec<E>` return: each element's tag + groups cross raw and the
        // folder singleton appends the rebuilt alternative.
        check(readingSeries(0, boom).isEmpty())
        check(
            readingSeries(5, boom) == listOf(
                Reading.Missing,
                Reading.Exact(42L),
                Reading.Range(1L, 9L),
                Reading.Tagged("warm", Priority.HIGH),
                Reading.Companion(5L),
            )
        )

        // Callback argument: the user callback receives the whole reassembled
        // sum while the wire still carries decoupled slots.
        val seen = ArrayList<Reading>()
        readingEach(5, { r -> seen.add(r) }, boom)
        check(seen == readingSeries(5, boom))
    }

    // ── a tag-gated group that owns a native resource ─────────────────────────
    // `Lookup.Found` carries an opaque handle: the live group hands over a
    // freshly boxed pointer the caller owns, while an inert handle slot stays
    // the `0L` sentinel that is never wrapped (wrapping it would fabricate a
    // handle to nothing). `Failed`'s `String` group proves an inert OBJECT slot
    // arrives as JVM null and so must be nullable in the raw builder.
    section("sum return with a handle payload") {
        check(lookupOf(0L, 0.0, boom) === Lookup.Absent)

        val failed = lookupOf(-1L, 0.0, boom)
        check(failed is Lookup.Failed && (failed as Lookup.Failed).v0 == "negative count")

        val found = lookupOf(3L, 7.5, boom)
        check(found is Lookup.Found)
        val summary = (found as Lookup.Found).v0
        // The handle is live and owns its own copy of the native object.
        check(summary.count(boom) == 3L)
        check(summary.total(boom) == 7.5)
        summary.close()
        check(summary.isClosed())
    }

    // The same handle-carrying sum arriving through a CALLBACK rather than as a
    // return. A sum payload is a plan LEAF, so the generated proxy WRAPS the
    // pointer but does not `close()` it — unlike a plan-less `impl Fn(Handle)`
    // arg, which closes in a `finally`. So the handle is live for as long as the
    // receiver keeps it and is the caller's to close, exactly as for a returned
    // sum. That contract is what this pins (#161).
    section("sum with a handle payload delivered to a callback") {
        val seen = mutableListOf<String>()
        val kept = mutableListOf<Summary>()
        lookupEach(3L, 2.5, { lookup ->
            when (lookup) {
                is Lookup.Failed -> seen.add("failed:${lookup.v0}")
                Lookup.Absent -> seen.add("absent")
                is Lookup.Found -> {
                    val s = lookup.v0
                    // Live INSIDE the callback: the live group's handle is a real
                    // native object, not the inert `0L` sentinel.
                    check(!s.isClosed())
                    check(s.total(boom) == 2.5)
                    seen.add("found:${s.count(boom)}")
                    kept.add(s)
                }
            }
        }, boom)
        check(seen == listOf("failed:negative count", "absent", "found:1"))

        // Still live AFTER the call returns — the proxy did not close it — and
        // closeable by the receiver that kept it.
        check(kept.size == 1)
        val s = kept[0]
        check(!s.isClosed())
        check(s.count(boom) == 1L)
        s.close()
        check(s.isClosed())
    }

    // An output boundary DERIVED from the type's value form
    // (`expand_return!(Report).fields(fields!(report_to_struct))`) instead of a
    // restated field list — #213. The point is that deriving changes NOTHING
    // about the wire: each field still crosses by its own type's boundary, all
    // in ONE crossing, so a binding can swap a hand-written list (which drifts
    // when the struct gains a field) for the derived one and keep its shape.
    //
    // Each parameter below lands on a different rule, and the signature itself
    // is the assertion — it would not compile if a field had been derived
    // wrongly:
    //   summary__count/total  the field's type has its own expand_return! and
    //                         is spliced by it — NOT handed over as a handle
    //   taken                 Option<data class> stays ONE leaf
    //   origin__secs/nanos    a non-optional data class INLINES
    //   outcome               a sum, typed here, tag + groups on the wire
    //   label                 a plain leaf
    section("output boundary derived from a value form") {
        val rows = mutableListOf<String>()
        val kept = mutableListOf<Summary>()
        reportEach(3L, { sCount, sTotal, taken, oSecs, oNanos, outcome, label ->
            // The value form is called ONCE per delivery, so every leaf below
            // comes from the same snapshot.
            check(oSecs == 1L && oNanos == 2L)
            val stamped = if (taken != null) "@${taken.secs}" else "-"
            val which = when (outcome) {
                is Lookup.Failed -> "failed"
                Lookup.Absent -> "absent"
                is Lookup.Found -> {
                    val s = outcome.v0
                    // A handle carried by a sum group, reached through a value
                    // form: live, and the receiver's to close.
                    check(!s.isClosed())
                    kept.add(s)
                    "found:${s.count(boom)}"
                }
            }
            rows.add("$label|$sCount|$sTotal|$stamped|$which")
        }, boom)

        check(
            rows == listOf(
                "r0|0|0.0|@7|failed",
                "r1|0|10.0|-|absent",
                "r2|1|20.0|@7|found:1",
            )
        ) { "derived value-form leaves: $rows" }

        check(kept.size == 1)
        kept[0].close()
        check(kept[0].isClosed())
    }

    // The same derived boundary reached through an `Option` — a CONDITIONAL
    // hoist. `ledgerNew(n)` fills `filed` (bit 0) and `archived` (bit 1)
    // independently, so one call per n drives all four present/absent
    // combinations, and each report carries a sum whose `match` lives inside
    // the arm that binds it.
    //
    //   filed     Option<&Report> — a BORROW, so the by-value form clones
    //   archived  Option<Report>  — OWNED, so it moves in
    //
    // Absent is null in EVERY leaf, the sum's selector included: its tag boxes
    // so JVM null can mean "no value here", which tag 0 cannot — that would
    // alias a real variant (`Lookup.Absent`) and lose the Option.
    section("value form reached through an Option (conditional hoist)") {
        // At a BUILDER position the sum stays raw (tag + groups), so the tag is
        // readable directly — which is how the absent case is pinned below.
        val rows = mutableListOf<String>()
        for (n in 0L..3L) {
            val row = ledgerNew(n, boom) { fCount, _, _, _, _, fTag, fFound, _, fLabel,
                                           aCount, _, _, _, _, aTag, aFound, _, aLabel ->
                fFound?.close()
                aFound?.close()
                val filed = if (fLabel == null) "-|$fTag" else "$fLabel/$fCount/$fTag"
                val archived = if (aLabel == null) "-|$aTag" else "$aLabel/$aCount/$aTag"
                "$filed $archived"
            }
            rows.add(row)
        }
        // `-|null`: an absent report nulls the selector too, so a receiver can
        // tell it from a present report whose outcome IS the tag-0 variant.
        check(
            rows == listOf(
                "-|null -|null",
                "l1/1/1 -|null",
                "-|null l2/2/1",
                "l1/1/1 l2/2/1",
            )
        ) { "conditional value-form leaves: $rows" }

        // The callback position takes the same path, and there the sum arrives
        // TYPED — so this is where the absent case would silently become
        // `Lookup.Absent` if the selector could not be null.
        val seen = mutableListOf<String>()
        ledgerEach(4L, { _, _, _, _, _, fOutcome, fLabel, _, _, _, _, _, aOutcome, aLabel ->
            check((fOutcome == null) == (fLabel == null)) {
                "an absent report must reconstruct its sum as null, not as a variant"
            }
            check((aOutcome == null) == (aLabel == null))
            (fOutcome as? Lookup.Found)?.v0?.close()
            (aOutcome as? Lookup.Found)?.v0?.close()
            seen.add("${fLabel ?: "-"}|${aLabel ?: "-"}")
        }, boom)
        check(seen == listOf("-|-", "l1|-", "-|l2", "l1|l2")) {
            "conditional value form through a callback: $seen"
        }
    }

    // A sum returned BORROWED (`&Reading` / `Option<&Reading>`). The value stays
    // owned by the archive; the encoder matches THROUGH the reference and clones
    // what each live group needs, so Kotlin gets an ordinary value with no
    // borrow to track and the owner can be read again afterwards (#161).
    section("borrowed sum returns") {
        val vault = archiveNew(boom)

        // Every alternative, read back through `&Reading`.
        archiveSetReading(vault, 0, boom)
        check(archiveReading(vault, boom) === Reading.Missing)
        archiveSetReading(vault, 1, boom)
        check((archiveReading(vault, boom) as Reading.Exact).v0 == 42L)
        archiveSetReading(vault, 2, boom)
        val range = archiveReading(vault, boom) as Reading.Range
        check(range.low == 1L && range.high == 9L)
        archiveSetReading(vault, 3, boom)
        val tagged = archiveReading(vault, boom) as Reading.Tagged
        check(tagged.v0 == "warm" && tagged.v1 == Priority.HIGH)

        // Borrowing does not consume: the same archive answers again, and the
        // value it still owns is unchanged.
        check((archiveReading(vault, boom) as Reading.Tagged).v0 == "warm")

        // `Option<&Reading>`: the present layer nulls the whole result, staying
        // independent of the tag.
        check(archiveReadingMaybe(vault, boom) != null)
        archiveSetReading(vault, -1, boom)
        check(archiveReadingMaybe(vault, boom) == null)
        check(archiveReading(vault, boom) === Reading.Missing)

        vault.close()
    }

    // ── a sum whose payload is NOT leaf-shaped ────────────────────────────────
    // `Marker.Ranked` carries `Option<Priority>` — an enum object or null in the
    // JVM slot, which tag-gated groups cannot express. The sum degrades to a
    // whole-object crossing rather than failing the build, and that path is what
    // reads an enum property back (bare and optional read differently: the slot
    // holds the enum OBJECT, not a boxed Int).
    section("sum with a non-leaf payload (whole-object crossing, Option<enum>)") {
        check(taggedRank(taggedNew(0, boom), boom) == -1)
        check(taggedRank(taggedNew(1, boom), boom) == 0)
        check(taggedRank(taggedNew(2, boom), boom) == 10)

        // Kotlin-constructed values cross identically — including the two
        // `Ranked` shapes that differ only by the optional being present.
        check(taggedRank(Tagged(1L, Marker.None_), boom) == -1)
        check(taggedRank(Tagged(1L, Marker.Ranked(null)), boom) == 0)
        check(taggedRank(Tagged(1L, Marker.Ranked(Priority.HIGH)), boom) == 10)
    }

    // ── the same Option<enum> payload in RETURN position ──────────────────────
    // The field above degrades to the whole-object crossing, so it never reaches
    // `synth_sum_leaves` — which hardcodes `nullable: false` on every group leaf
    // and lets `plan_leaf_param` widen from the inert side. Two nullabilities
    // meet in this one slot and must not collapse into each other: the payload's
    // own `None`, and the slot being inert because the OTHER variant is live.
    // Both arrive as a JVM null, so only the tag tells `Ranked(null)` from
    // `None_`.
    section("Option<enum> payload in a returned sum") {
        check(markerOf(0, boom) === Marker.None_)

        val absent = markerOf(1, boom)
        check(absent is Marker.Ranked && absent.v0 == null)

        val present = markerOf(2, boom)
        check(present is Marker.Ranked && present.v0 == Priority.HIGH)

        // The two nulls are distinguishable in both directions: each value the
        // return path built crosses back in and reads as itself.
        check(taggedRank(Tagged(1L, markerOf(0, boom)), boom) == -1)
        check(taggedRank(Tagged(1L, markerOf(1, boom)), boom) == 0)
        check(taggedRank(Tagged(1L, markerOf(2, boom)), boom) == 10)
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

    // ── array-backed VALUE EQUALITY ─────────────────────────────────────────
    // The Rust types derive `Eq`, so the Kotlin mirrors must compare by
    // content. Kotlin arrays compare by IDENTITY, which silently breaks that
    // for every `ByteArray`-backed property — a value blob's `bytes`, a
    // `Vec<u8>` field, and any value carrying one of those. Kotlin's own
    // `data class` codegen is inconsistent here (its `hashCode`/`toString` DO
    // special-case arrays, its `equals` does not), so `==` is the assertion
    // that matters and `hashCode` alone would not have caught the defect.
    section("array-backed value equality (content, not identity)") {
        // The NEGATIVE case first: a data class with no array property must keep
        // the compiler's own equality. The generator emits nothing for it, so
        // this pins that the content operators do not churn ordinary classes.
        val s1 = stampNew(7L, 42L, boom)
        val s2 = stampNew(7L, 42L, boom)
        check(s1 == s2) { "scalar data class must compare by value: $s1 vs $s2" }
        check(s1.hashCode() == s2.hashCode())
        check(stampNew(8L, 42L, boom) != s1) { "different content must not compare equal" }
        check(hashSetOf(s1, s2).size == 1)
        check(s1.toString() == "Stamp(secs=7, nanos=42)") { "got $s1" }

        // A data class with a DIRECT `ByteArray` field plus a NESTED value
        // blob — the two shapes that broke downstream. The bytes sit LAST, so
        // this also covers the `31 * result + …contentHashCode()` fold form
        // that a real value (`Timestamp(ntp64, id)`) produces.
        fun blob(secs: Long, id: ByteArray, chunks: List<ByteArray>) =
            blobValueNew(secs, id, chunks, boom)

        val chunks = listOf(byteArrayOf(9), byteArrayOf(8, 7))
        val b1 = blob(7L, byteArrayOf(1, 2, 3), chunks)
        val b2 = blob(7L, byteArrayOf(1, 2, 3), listOf(byteArrayOf(9), byteArrayOf(8, 7)))
        check(b1 == b2) { "array-backed data class must compare by content: $b1 vs $b2" }
        check(b1.hashCode() == b2.hashCode())
        check(hashSetOf(b1, b2).size == 1)
        // Every component must actually participate — a comparison that ignored
        // any of them would still pass the equality checks above.
        check(blob(7L, byteArrayOf(1, 2, 4), chunks) != b1) { "id must matter" }
        check(blob(8L, byteArrayOf(1, 2, 3), chunks) != b1) { "nested blob must matter" }
        // A CONTAINER of arrays: `List<ByteArray>` inherits `ByteArray`'s
        // identity equality, so the operators must dig through the container.
        check(blob(7L, byteArrayOf(1, 2, 3), listOf(byteArrayOf(9), byteArrayOf(8, 6))) != b1) {
            "chunk contents must matter"
        }
        check(blob(7L, byteArrayOf(1, 2, 3), listOf(byteArrayOf(9))) != b1) {
            "chunk count must matter"
        }
        check(b1.toString().contains("id=[1, 2, 3]")) { "toString must render bytes, got $b1" }
        check(b1.toString().contains("chunks=[[9], [8, 7]]")) {
            "toString must render nested bytes, got $b1"
        }

        // ── fixed-size arrays -> Kotlin primitive arrays ─────────────────────
        // Every `[T; N]` crosses as the matching primitive array (bulk-copied,
        // nothing boxed), and every one of them compares by IDENTITY in Kotlin
        // — so each needs the content operators, not just the byte case.
        val a1 = Arrays(
            byteArrayOf(1, 2, 3, 4),
            shortArrayOf(-1, 2),
            intArrayOf(3, -4, 5),
            longArrayOf(6, -7),
            doubleArrayOf(0.5, -1.25),
            booleanArrayOf(true, false, true),
            longArrayOf(-1L, 0L), // [u64; 2] carries raw bits: -1L == u64::MAX
        )
        val a2 = arraysEcho(a1, boom)
        check(a2 == a1) { "fixed-size arrays must round-trip by content: $a2 vs $a1" }
        check(a2.hashCode() == a1.hashCode())
        check(hashSetOf(a1, a2).size == 1)
        // The round trip must preserve VALUES, not just shapes — a per-element
        // cast error would survive an equality-only check between two echoes.
        check(a2.bytes.contentEquals(byteArrayOf(1, 2, 3, 4)))
        check(a2.shorts.contentEquals(shortArrayOf(-1, 2)))
        check(a2.ints.contentEquals(intArrayOf(3, -4, 5)))
        check(a2.longs.contentEquals(longArrayOf(6, -7)))
        check(a2.doubles.contentEquals(doubleArrayOf(0.5, -1.25)))
        check(a2.flags.contentEquals(booleanArrayOf(true, false, true)))
        // `u64::MAX` survives as the raw bit pattern rather than saturating.
        check(a2.raw.contentEquals(longArrayOf(-1L, 0L))) { "raw bits: ${a2.raw.toList()}" }
        // Each component participates in equality.
        check(arraysEcho(a1.copy(ints = intArrayOf(3, -4, 6)), boom) != a1) { "ints must matter" }
        check(arraysEcho(a1.copy(flags = booleanArrayOf(true, true, true)), boom) != a1) {
            "flags must matter"
        }
        check(a1.toString().contains("flags=[true, false, true]")) { "got $a1" }

        // Wrong length is a BINDING ERROR, not a panic — the decode's `try_into`
        // is the length check (the fixed-size-array successor to the value
        // blob's byte-length guard).
        var lenErr: String? = null
        arraysEcho(a1.copy(ints = intArrayOf(1, 2))) { je -> lenErr = je; a1 }
        check(lenErr?.contains("fixed-size array decode") == true) {
            "wrong-length array must report a binding error, got: $lenErr"
        }

        // WHOLE-OBJECT input decode (`.jobject_input()`): the decoder reads each
        // field off the Kotlin object by JVM descriptor. A value-blob field's
        // slot is the wrapper class, not `[B` — reading the old descriptor threw
        // `NoSuchFieldError` on the first decode.
        check(blobValueEcho(b1, boom) == b1) { "jobject-input round trip must preserve the value" }
        check(blobValueEcho(blob(0L, ByteArray(0), emptyList()), boom).chunks.isEmpty())
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

        // #87: split × builder-delivered return. `summaryMerge` returns a
        // `Summary` decomposed through the trailing builder lambda, so its
        // wrapper — and EVERY split overload — is generic over `<R>`; before
        // the fix the overloads referenced `R` without declaring it and the
        // generated Kotlin did not compile.
        check(
            summaryMerge(2L, 40.0, 1L, 2.0, boom) { count, total -> count to total } ==
                (3L to 42.0),
        )                                                                          // build / build
        check(
            summaryMerge(2L, 40.0, Summary.of(1L, 2.0, boom), boom) { count, _ -> count } == 3L,
        )                                                                          // build / handle
        check(
            summaryMerge(Summary.of(2L, 40.0, boom), 1L, 2.0, boom) { _, total -> total } == 42.0,
        )                                                                          // handle / build
        check(
            summaryMerge(Summary.of(2L, 40.0, boom), Summary.of(1L, 2.0, boom), boom) {
                count, total ->
                count to total
            } == (3L to 42.0),
        )                                                                          // handle / handle

        // #189 ALIAS PREFLIGHT: two consumed handle params of one class can be
        // handed the SAME resource, which would consume one allocation twice.
        // The wrapper compares `ptr` before the lock and before any conversion,
        // so the rejection reaches `onError` and the handle is untouched —
        // still open, still usable, and still ours to close. A check inside the
        // converter could not offer that: by then the first argument is gone.
        run {
            val shared = Summary.of(5L, 55.0, boom)
            var aliasMessage: String? = null
            val rejected = summaryPrefer(shared, shared) { je ->
                aliasMessage = je
                -1L
            }
            check(rejected == -1L)
            check(aliasMessage?.contains("Aliasing arguments") == true) {
                "expected an alias rejection, got: $aliasMessage"
            }
            // Nothing was consumed: the handle survives and still reads.
            check(!shared.isClosed())
            check(shared.total(boom) == 55.0)

            // No false positives — two DISTINCT handles of the same class go
            // through untouched.
            check(summaryPrefer(shared, Summary.of(3L, 99.0, boom), boom) == 0L)
        }

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

    // ── Result<_, E> → two-caller error split (ok + domain error) ────────────
    // A fallible-typed wrapper takes TWO handlers: `onBindingError` (the binding
    // channel) and `onError` (the typed domain channel, no `je`). See #45.
    section("Result error channel storageTryWithLabel") {
        val ok = storageTryWithLabel("hi", boom, boomStorage)
        check(ok.len(boom) == 1L)
        ok.close()

        // Domain error: `onError` fires (NOT `onBindingError`). The StorageError's
        // flatten delivers its `message` field plus — via the type-level
        // `field_self` — the owned error handle itself, live and queryable.
        try {
            storageTryWithLabel("", boom, StorageErrorHandler<Storage> { message, handle ->
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

    // ── #45: both channels of ONE fallible wrapper, each fires independently ──
    section("two-caller split storageTryFromStamp") {
        // Happy path: neither channel fires.
        val ok = storageTryFromStamp(stampNew(5L, 0L, boom), byteArrayOf(1, 2), boom, boomStorage)
        check(ok.len(boom) == 1L)
        ok.close()

        // DOMAIN error (well-formed Stamp, rejected value): `onError` fires,
        // `onBindingError` must NOT. The handler returns a throwaway Storage.
        var domainMsg: String? = null
        val domainRet = storageTryFromStamp(
            stampNew(-1L, 0L, boom),
            byteArrayOf(1, 2),
            JniErrorHandler<Storage> { je ->
                throw AssertionError("binding channel must not fire on a domain error: $je")
            },
            StorageErrorHandler<Storage> { message, handle ->
                domainMsg = message
                check(handle.message(boom) == "stamp secs must be positive")
                handle.close()
                storageNew(boom)
            },
        )
        check(domainMsg == "stamp secs must be positive") { "domain onError did not fire: $domainMsg" }
        domainRet.close()

        // BINDING error (wrong-length `tag` array): `onBindingError` fires,
        // the domain `onError` must NOT.
        var bindingJe: String? = null
        val bindingRet = storageTryFromStamp(
            Stamp(1L, 0L),
            byteArrayOf(1, 2, 3),   // `tag` is [u8; 2]; 3 must be rejected on decode
            JniErrorHandler<Storage> { je ->
                bindingJe = je
                storageNew(boom)
            },
            StorageErrorHandler<Storage> { _, handle ->
                handle.close()
                throw AssertionError("domain channel must not fire on a binding error")
            },
        )
        check(bindingJe != null && bindingJe!!.contains("fixed-size array decode")) {
            "binding onBindingError did not fire: $bindingJe"
        }
        bindingRet.close()
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
    section("convert! fallible stages under Option (Percent -> Int?)") {
        check(percentScale(50, 2, boom) == 100)
        check(percentScale(30, 2, boom) == 60)
        check(percentOptional(null, boom) == null)
        check(percentOptional(25, boom) == 25)
        // Out-of-range input: the TryFrom impl's Err(String) routes to
        // onError through an Option-composed stage (je carries the Display'd
        // message after normalization to __JniErr).
        var msg: String? = null
        percentOptional(150) { je ->
            msg = je
            null
        }
        check(msg?.contains("percent out of range: 150") == true) {
            "percentOptional(150) must report the range error, got: $msg"
        }

        // The output stage has its own raw String error. It must normalize in
        // the opposite Option composition direction and use the same handler.
        msg = null
        percentInvalidOutput { je ->
            msg = je
            null
        }
        check(msg == "invalid Percent output: 101") {
            "percentInvalidOutput must report the output conversion error, got: $msg"
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

        // `Vec<Label>` — a collection whose ELEMENT is a converted type. The
        // `Vec` converters build the element conversion inline in both
        // directions, so each has to run the element's chain rather than its
        // wire-facing converter alone. (`Vec<Duration>` cannot probe this: a
        // `Vec` needs a JObject-shaped element wire and a bounded duration's
        // is a primitive `Long`, so it is refused at resolve time.)
        check(labelSeriesEcho(listOf("alpha", "beta"), boom) == listOf("alpha", "beta"))
        check(labelSeriesEcho(emptyList(), boom) == emptyList<String>())
    }

    // ── Vec<opaque-handle> return: the Kotlin-side handle fold ───────────────
    section("record-built <A> fold (summarySeries / summarySeriesOpt)") {
        // Bare Vec<Summary>: the caller threads the accumulator; each element
        // arrives as its decomposed (count, total) leaves.
        val pairs =
            summarySeries(3L, 10L, mutableListOf<Pair<Long, Double>>(), boom) { acc, count, total ->
                acc.add(count to total)
                acc
            }
        check(pairs == listOf(10L to 100.0, 11L to 110.0, 12L to 120.0))
        check(summarySeries(0L, 5L, 0L, boom) { acc, _, _ -> acc + 1 } == 0L)
        // Option<Vec<Summary>> (#105): null = None (the fold never invoked);
        // Some(empty) returns the untouched accumulator, distinguishable from
        // None by the caller.
        check(summarySeriesOpt(-1L, 0L, 0L, boom) { acc, _, _ -> acc + 1 } == null)
        check(summarySeriesOpt(0L, 0L, 7L, boom) { acc, _, _ -> acc + 1 } == 7L)
        check(summarySeriesOpt(2L, 1L, 0.0, boom) { acc, _, total -> acc + total } == 30.0)
    }

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
        check(annotatedAlternateValue(a, boom) == null)     // Option<nested> absent gate
        val none = annotatedNew(payload(1L, 0, 0.0, false, null), null, null, boom)
        check(annotatedTtl(none, boom) == null && annotatedPriority(none, boom) == null)
        // Kotlin-constructed instance crosses direct + optional recursive paths.
        val c = Annotated(
            payload(2L, 0, 9.0, false, null),
            payload(3L, 0, 11.0, false, "alternate"),
            5L,
            Priority.LOW,
        )
        check(annotatedTtl(c, boom) == 5L)
        check(annotatedPriority(c, boom) == Priority.LOW)
        check(annotatedPayloadValue(c, boom) == 9.0)
        check(annotatedAlternateValue(c, boom) == 11.0)
    }

    // ── #144: non-null enum field reached through Option<data_class> input ────
    // The outer `Option<CacheConfig>` propagates nullable-context into the
    // non-optional nested `RepliesConfig`, whose non-null `priority` enum field
    // must decode with a SINGLE Elvis default. Before the fix the generated
    // Kotlin was `cache?.replies?.priority?.value ?: 0 ?: 0` — a dead second
    // default that the Kotlin compiler warned about. This exercises the decode
    // (present + absent) and is the regression guard for that codegen.
    section("Option<data_class> with non-null nested enum field (#144)") {
        val cache = CacheConfig(RepliesConfig(Priority.HIGH, 4L), 7L)
        check(cacheConfigWeight(cache, boom) == 17)   // weight(HIGH)=10 + ttl 7
        check(cacheConfigWeight(null, boom) == -1)    // absent outer optional
        val low = CacheConfig(RepliesConfig(Priority.LOW, 0L), 3L)
        check(cacheConfigWeight(low, boom) == 4)      // weight(LOW)=1 + ttl 3
    }

    section("data_class JVM-slot-limited JObject input boundary") {
        val leaf = ObjectBoundaryLeaf(1L)
        val level2 = ObjectBoundary2(leaf, leaf)
        val level4 = ObjectBoundary4(level2, level2)
        val level8 = ObjectBoundary8(level4, level4)
        val level16 = ObjectBoundary16(level8, level8)
        val level32 = ObjectBoundary32(level16, level16)
        val level64 = ObjectBoundary64(level32, level32)
        val level63 = ObjectBoundary63(level32, level16, level8, level4, level2, leaf)
        check(objectBoundaryValue(ObjectBoundary(level64, level63), boom) == 127L)
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

    // ── transparent wrappers: the spelling changes, the crossing must not ───
    // The model erases `Box`/`Cow`, so a wrapped spelling and its unwrapped
    // twin are ONE type to Kotlin. Compiling this crate already proves the
    // generated Rust is well-typed; these assert the surfaces are the same and
    // the values actually make the round trip.
    section("transparent wrapper crossings") {
        // Converted return: the converter is selected for the spelling, so it
        // names `Box<Option<String>>` itself.
        for (note in listOf("wrapped", null)) {
            check(boxedNoteEcho(note, boom) == note)
            check(plainNoteEcho(note, boom) == note)
        }

        // DECOMPOSED return: no converter names the spelling — the extern binds
        // the value and matches it, so the `Box` has to come off first (#292).
        // Same delivery as `archiveLatest`, one wrapper apart.
        val a: SummaryVault = archiveNew(boom)
        check(boxedLatest(a, boom) { count, total -> count to total } == null)
        archiveStore(a, 0, 5L, 100.0, null, boom)
        check(boxedLatest(a, boom) { count, total -> count to total } == 5L to 100.0)
        a.close()

        // INPUT side (#292 item 3). These lowerings REBUILD their parameter, so
        // the wrapper has to go back on before the value reaches the signature.
        // Every surface below is the unwrapped one — a wrapper must not cost a
        // parameter its lowering, and must not show up in Kotlin either.
        val p = payload(7L, 1, 2.0, true, "w")
        check(boxedPayloadId(p, boom) == 7L)              // core wrap
        check(boxedOptPayloadId(p, boom) == 7L)           // core + optional wrap
        check(boxedOptPayloadId(null, boom) == -1L)       // …and the absent arm
        check(boxedOptPriorityWeight(Priority.HIGH, boom) == 10L)  // option-scalar
        check(boxedOptPriorityWeight(null, boom) == -1L)
        val many = listOf(payload(1L, 0, 0.0, false, null), payload(2L, 0, 0.0, false, null))
        check(boxedElemIdSum(many, boom) == 3L)           // wrapped element
        check(boxedRunIdSum(many, boom) == 3L)            // wrapped run, by value

        // FIELDS (#289). `boxed: Box<Option<Long>>` and `plain: Option<Long>`
        // are one type to the model, so both cross as `Long?` on the decoupled
        // `(present, value)` pair — the boxed one used to be read by path
        // segment as "not optional" and crossed as one boxed object.
        // `Priority.LOW` weighs 1, `HIGH` weighs 10 — see `priority_weight`.
        check(wrappedFieldsSum(WrappedFields(1L, 2L, 4L, Priority.LOW, Priority.LOW), boom) == 9L)
        check(wrappedFieldsSum(WrappedFields(1L, null, 4L, Priority.LOW, Priority.LOW), boom) == 7L)
        check(wrappedFieldsSum(WrappedFields(1L, 2L, null, Priority.LOW, Priority.LOW), boom) == 5L)
        check(wrappedFieldsSum(WrappedFields(1L, null, null, Priority.LOW, Priority.LOW), boom) == 3L)

        // And over a TERMINAL (#309): `Box<Priority>` had no outbound route at
        // all, where `Box<Option<Long>>` above rode the `Optional` layer arm.
        // Both enum fields are declared `Priority` in Kotlin — the wrapper is
        // invisible — and the pair differing only in spelling must weigh alike.
        check(wrappedFieldsSum(WrappedFields(0L, null, null, Priority.HIGH, Priority.LOW), boom) == 11L)
        check(wrappedFieldsSum(WrappedFields(0L, null, null, Priority.LOW, Priority.HIGH), boom) == 11L)

        // An absent `Option<data class>` must deliver `None`, not an error. Its
        // leaves are inert placeholders when the object is null, and a required
        // HANDLE field's placeholder is pointer 0 — which the direct-handle
        // decode reads as a closed handle. So this is the shape that proves the
        // field decodes stay inside the presence gate; a fixture whose fields
        // all decode successfully cannot tell the two orders apart.
        check(holderTagOr(null, -9L, boom) == -9L)
        val held = Summary.of(4L, 8.0, boom)
        check(holderTagOr(Holder(3L, held), -9L, boom) == 7L)  // 3 + count(4)
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

    // ── binding error: je != null (fixed-size array length guard) ───────────
    section("binding error je != null (wrong-length fixed-size array)") {
        var je: String? = null
        val fallback = storageTryFromStamp(
            stampNew(1L, 0L, boom),
            byteArrayOf(1, 2, 3),   // `tag` is [u8; 2]; 3 is rejected on decode
            JniErrorHandler { e ->
                je = e
                storageNew(boom)
            },
            StorageErrorHandler { _, handle ->
                throw AssertionError("domain channel must not fire on a decode failure")
            },
        )
        fallback.close()
        check(je != null && je!!.contains("fixed-size array decode")) { "unexpected je: $je" }
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

    // ── JNI native-symbol escaping (#86) ─────────────────────────────────────
    section("JNI native-symbol escaping (esc_pkg / Esc_Probe / snake extern)") {
        // Every call here resolves a Rust export whose symbol needs the JNI
        // spec's `_1` escaping — `esc_1pkg` + `Esc_1Probe` in the freePtr
        // destructor, `escape_1probe_1value` on the harness extern. A raw
        // dot-to-underscore symbol would throw UnsatisfiedLinkError.
        val p = Esc_Probe.escapeProbeNew(7L, boom)
        check(p.escapeProbeValue(boom) == 7L)
        p.close()
    }

    println("PASS - $sectionCount sections, every JniGen feature exercised")
}
