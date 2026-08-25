package io.prebindgen.covertest

import io.prebindgen.covertest.analytics.Summary
import io.prebindgen.covertest.analytics.SummaryVault
import io.prebindgen.covertest.analytics.SelectorCode
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
import io.prebindgen.covertest.analytics.selectorCodeScore
import io.prebindgen.covertest.analytics.summaryEnvelopeScore
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
import io.prebindgen.covertest.model.Ingot
import io.prebindgen.covertest.model.ingotOptionalGrams
import io.prebindgen.covertest.model.ObjectBoundary64
import io.prebindgen.covertest.model.ObjectBoundaryLeaf
import io.prebindgen.covertest.model.Priority
import io.prebindgen.covertest.model.Hold
import io.prebindgen.covertest.model.HoldPolicy
import io.prebindgen.covertest.model.Layered
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
import io.prebindgen.covertest.model.refVecIdSum
import io.prebindgen.covertest.model.sliceIdSum
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
import io.prebindgen.covertest.model.ticksEmit
import io.prebindgen.covertest.model.vaultHolderNew
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
import io.prebindgen.covertest.model.probeEach
import io.prebindgen.covertest.model.probeNew
import io.prebindgen.covertest.model.lookupOf
import io.prebindgen.covertest.model.layeredOf
import io.prebindgen.covertest.model.verdictNew
import io.prebindgen.covertest.model.dossierNew
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
import io.prebindgen.covertest.model.maybeHolderNew
import io.prebindgen.covertest.model.taggedNew
import io.prebindgen.covertest.model.taggedRank
import io.prebindgen.covertest.model.payloadPriority
import io.prebindgen.covertest.model.priorityOr
import io.prebindgen.covertest.model.priorityNested
import io.prebindgen.covertest.model.priorityNestedState
import io.prebindgen.covertest.model.priorityWeight
import io.prebindgen.covertest.model.stampNew
import io.prebindgen.covertest.model.stampSeries
import io.prebindgen.covertest.model.unsignedEmit
import io.prebindgen.covertest.model.unsignedDataMaybe
import io.prebindgen.covertest.model.unsignedOptional
import io.prebindgen.covertest.model.unsignedRoundTrip
import io.prebindgen.covertest.model.unsignedSeries
import io.prebindgen.covertest.storage.payloadOptionalEmit
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

/** Assert that a reference-returning call with a throwing handler succeeded. */
private fun <T : Any> T?.orThrow(): T =
    this ?: throw AssertionError("a throwing error handler returned instead of throwing")

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
    // `fromParts` takes the raw wire slots, so it is `@UnsafeNativeApi` (#37);
    // exercising it from hand-written Kotlin means opting in explicitly.
    @OptIn(UnsafeNativeApi::class)
    section("data_class Payload") {
        val p = Payload(1L, 2, 3.5, true, "hello")
        check(p.id == 1L && p.seq == 2 && p.value == 3.5 && p.flag && p.label == "hello")
        check(Payload.fromParts(9L, 9, 9.0, false, null).label == null)
    }

    // ── borrowed Option<data_class>: null/present are deconstructed on the
    // Kotlin side, cross as primitive leaves, and are recomposed before the
    // source call borrows the registry-owned carrier ─────────────────────────
    section("borrowed Option<&data_class> uses the shared chain") {
        check(payloadOptionalBorrowId(Payload(17L, 2, 3.5, true, "hello"), boom) == 17L)
        check(payloadOptionalBorrowId(null, boom) == -1L)
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
        ).orThrow()
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
        // Nested Option layers share one primitive wire. Kotlin deliberately
        // collapses both absent Rust states to null, while preserving values.
        check(priorityNested(0, boom) == null)
        check(priorityNested(1, boom) == null)
        check(priorityNested(2, boom) == Priority.HIGH)
        // On input, null selects the outer None sentinel. The collapsed
        // Priority? surface cannot construct Rust's inner Some(None) state.
        check(priorityNestedState(null, boom) == 0)
        check(priorityNestedState(Priority.HIGH, boom) == 2)
        // enum_class surface: value + fromInt round-trip.
        check(Priority.HIGH.value == 2)
        check(Priority.fromInt(0) == Priority.LOW)
    }

    // ── sealed_class: a data-carrying enum as a Kotlin `sealed interface` ─────
    // The Kotlin surface only (the wire lowering is a separate stage): every
    // variant shape, the nested placement, the per-variant rename, and
    // `fromParts` picking the live group by tag.
    @OptIn(UnsafeNativeApi::class)
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
        check(observationNew(0, false, boom).orThrow().reading == Reading.Missing)
        check(observationNew(1, false, boom).orThrow().reading == Reading.Exact(42L))
        check(observationNew(2, false, boom).orThrow().reading == Reading.Range(1L, 9L))
        check(observationNew(3, false, boom).orThrow().reading == Reading.Tagged("warm", Priority.HIGH))
        check(observationNew(4, false, boom).orThrow().reading == Reading.Companion(5L))

        // The sum sits beside ordinary flattened leaves — they must not be
        // disturbed by the tag-gated groups interleaved with them.
        val obs = observationNew(3, false, boom).orThrow()
        check(obs.id == 7L && obs.note == "obs")

        // `Option<sum>`: the present flag and the tag are independent facts,
        // so an absent optional is null regardless of what its tag slot holds.
        check(observationNew(1, false, boom).orThrow().fallback == null)
        check(observationNew(1, true, boom).orThrow().fallback == Reading.Range(1L, 9L))
        // …and an object-payload variant round-trips through the optional too.
        check(observationNew(2, true, boom).orThrow().fallback == Reading.Tagged("warm", Priority.HIGH))
        // Both sums live at once, each with its own tag.
        val both = observationNew(4, true, boom).orThrow()
        check(both.reading == Reading.Companion(5L) && both.fallback == Reading.Missing)

        // …and back IN as part of a data-class parameter: every alternative
        // reconstructs the same Rust variant it came from.
        for (which in 0..4) {
            check(observationWhich(observationNew(which, false, boom).orThrow(), boom) == which)
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
        check(readingSeries(0, boom).orThrow().isEmpty())
        check(
            readingSeries(5, boom).orThrow() == listOf(
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
        check(seen == readingSeries(5, boom).orThrow())
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
    // return. The proxy binds the reassembled value to a local and `close()`s it
    // in a `finally` after `run` — the SAME close-unless-taken contract a
    // plan-less `impl Fn(Handle)` arg has always had, so the payload's lifetime
    // does not depend on whether it arrived bare or inside a sum (#218,
    // originally pinned the other way by #161).
    section("sum with a handle payload delivered to a callback") {
        val seen = mutableListOf<String>()
        val escaped = mutableListOf<Summary>()
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
                    escaped.add(s)
                }
            }
        }, boom)
        check(seen == listOf("failed:negative count", "absent", "found:1"))

        // Closed once `run` returned: a body that merely kept the reference gets
        // a closed handle, which is exactly what the bare-handle arg does.
        check(escaped.size == 1)
        check(escaped[0].isClosed())

        // `take()` is how a body means to outlive the call — it moves the
        // pointer out, so the proxy's `close()` finds nothing to free.
        val kept = mutableListOf<Summary>()
        lookupEach(3L, 4.0, { lookup ->
            if (lookup is Lookup.Found) kept.add(lookup.v0.take())
        }, boom)
        check(kept.size == 1)
        val s = kept[0]
        check(!s.isClosed())
        check(s.count(boom) == 1L)
        s.close()
        check(s.isClosed())
    }

    // The THIRD position the same handle can be reached through: a data-class
    // FIELD whose type is the sum. `Holder` (a plain handle field) and `Verdict`
    // (a sum field) must behave alike — that is the whole of #218. The container
    // is `AutoCloseable` either way and its `close()` cascades either way; the
    // walk into the alternatives lives in `Lookup`, not in `Verdict`.
    section("a data-class field reaching a handle through a sum cascades") {
        val v = verdictNew(7L, 3L, 1.5, boom).orThrow()
        check(v.id == 7L)
        val found = v.outcome
        check(found is Lookup.Found)
        val summary = (found as Lookup.Found).v0
        check(!summary.isClosed())
        check(summary.count(boom) == 3L)

        // Closing the CONTAINER closes the handle the sum holds — no
        // `(v.outcome as Lookup.Found).v0.close()` at the call site.
        v.close()
        check(summary.isClosed())

        // …and an alternative owning nothing native closes to a no-op, so the
        // cascade is safe for every value of the field, not just the live-handle
        // one.
        val absent = verdictNew(8L, 0L, 0.0, boom).orThrow()
        check(absent.outcome === Lookup.Absent)
        absent.close()
    }

    // The FOURTH position, and the row an emission test cannot cover: the field
    // is a plain data class that itself carries the handle. `Dossier`'s cascade
    // is the one-liner `holder.close()`, which only frees anything because
    // `Holder` was independently rendered `AutoCloseable` by its own pass —
    // two decisions, in two places, that nothing but a compiled run ties
    // together. This section IS that tie: it would not compile if `Holder` had
    // no `close()`, and the last check fails if `Dossier` had none.
    section("a data-class field reaching a handle through a nested data class cascades") {
        val d = dossierNew(5L, 3L, 4L, 2.0, boom).orThrow()
        check(d.note == 5L)
        check(d.holder.tag == 3L)
        val summary = d.holder.summary
        check(!summary.isClosed())
        check(summary.count(boom) == 4L)

        // Two levels down, closed by one `close()` at the top.
        d.close()
        check(summary.isClosed())
    }

    // The same handle field with an `Option` in front of it. The factory that
    // rebuilds the class takes a different arm per case, and the present arm has
    // to MINT a handle — through the generated factory, since #404 made the
    // constructor private. The optional arm went on naming the constructor, so
    // this class did not compile at all (#430), which no emission test could
    // say: the Rust half is identical either way. This section is the tie —
    // it does not compile if the arm names something private, and the checks
    // fail if either case rebuilds the wrong thing.
    section("an optional handle field is minted through the factory, present and absent") {
        val present = maybeHolderNew(3L, 4L, 8.0, true, boom).orThrow()
        check(present.tag == 3L)
        val held = present.summary ?: error("the present arm dropped the handle")
        check(!held.isClosed())
        check(held.count(boom) == 4L)
        present.close()
        check(held.isClosed())

        // …and the absent arm produces `null` rather than a handle over pointer
        // 0, so closing the container is a no-op with nothing to free.
        val absentHolder = maybeHolderNew(7L, 4L, 8.0, false, boom).orThrow()
        check(absentHolder.tag == 7L)
        check(absentHolder.summary == null)
        absentHolder.close()
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

    // An `Option<sum>` FIELD of a value form (#220) — the shape that used to be
    // refused here while a `data_class` field of the same type was accepted.
    //
    // The gate is the SEGMENT's, not the form's: `seq` beside it is an ordinary
    // leaf that crosses whether or not the sum is there. And absence rides the
    // selector's own nullability rather than a present flag beside it, which is
    // what this pins: `null` and `Lookup.Absent` are different answers, and a
    // raw `jint` tag could not tell them apart because `Absent` IS tag 0.
    section("Option<sum> as a value-form field") {
        val rows = mutableListOf<String>()
        val kept = mutableListOf<Summary>()
        probeEach(4L, 5.0, { seq, outcome ->
            val which = when (outcome) {
                null -> "none"
                is Lookup.Failed -> "failed:${outcome.v0}"
                Lookup.Absent -> "absent"
                is Lookup.Found -> {
                    val s = outcome.v0
                    check(!s.isClosed())
                    kept.add(s)
                    "found:${s.count(boom)}"
                }
            }
            rows.add("$seq|$which")
        }, boom)

        // i = 0 → count -2 → the field itself is absent; the rest walk `Lookup`.
        // `0|none` vs `1|failed…` is the whole point: the first has no sum at
        // all, the second has one whose alternative happens to be tag 0's
        // neighbour. A present flag would have said the same thing; the boxed
        // selector says it for free.
        check(
            rows == listOf(
                "0|none",
                "1|failed:negative count",
                "2|absent",
                "3|found:1",
            )
        ) { "Option<sum> value-form leaves: $rows" }

        check(kept.size == 1)
        kept[0].close()
        check(kept[0].isClosed())

        // The same field at a BUILDER position, where the sum stays raw — so the
        // absent case is readable as a null TAG, ahead of any variant.
        val absentTag = probeNew(9L, -2L, 0.0, boom) { seq, tag, _, _ -> "$seq:$tag" }
        check(absentTag == "9:null") { "an absent sum nulls its selector: $absentTag" }
        // …and the exact collision the boxing exists to prevent: a PRESENT sum
        // whose alternative is `Lookup.Absent` is tag `0`. A raw `jint` selector
        // would have made these two calls indistinguishable.
        val presentTag = probeNew(9L, 0L, 0.0, boom) { seq, tag, _, _ -> "$seq:$tag" }
        check(presentTag == "9:0") { "a present `Lookup.Absent` is tag 0, not null: $presentTag" }
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
            }.orThrow()
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
            // No `(fOutcome as? Lookup.Found)?.v0?.close()` here any more: an
            // OPTIONAL sum arg is closed by the proxy too, under the `?.` its
            // nullability earns (#218).
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
        val vault = archiveNew(boom).orThrow()

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

    // ── a sum whose payload uses an enum niche ────────────────────────────────
    // `Marker.Ranked` carries `Option<Priority>` in one primitive jint. An
    // unused discriminant represents null, while the independent sum tag keeps
    // `Marker.None_` distinct from `Marker.Ranked(null)`.
    section("sum with a niche-backed Option<enum> payload") {
        check(taggedRank(taggedNew(0, boom).orThrow(), boom) == -1)
        check(taggedRank(taggedNew(1, boom).orThrow(), boom) == 0)
        check(taggedRank(taggedNew(2, boom).orThrow(), boom) == 10)

        // Kotlin-constructed values cross identically — including the two
        // `Ranked` shapes that differ only by the optional being present.
        check(taggedRank(Tagged(1L, Marker.None_), boom) == -1)
        check(taggedRank(Tagged(1L, Marker.Ranked(null)), boom) == 0)
        check(taggedRank(Tagged(1L, Marker.Ranked(Priority.HIGH)), boom) == 10)
    }

    // ── the same Option<enum> payload in RETURN position ──────────────────────
    // The optional payload uses its reserved enum discriminant for null. The
    // sum tag separately identifies the active variant, so `Ranked(null)` and
    // `None_` remain distinct in both directions.
    section("Option<enum> payload in a returned sum") {
        check(markerOf(0, boom).orThrow() === Marker.None_)

        val absent = markerOf(1, boom).orThrow()
        check(absent is Marker.Ranked && absent.v0 == null)

        val present = markerOf(2, boom).orThrow()
        check(present is Marker.Ranked && present.v0 == Priority.HIGH)

        // The two nulls are distinguishable in both directions: each value the
        // return path built crosses back in and reads as itself.
        check(taggedRank(Tagged(1L, markerOf(0, boom).orThrow()), boom) == -1)
        check(taggedRank(Tagged(1L, markerOf(1, boom).orThrow()), boom) == 0)
        check(taggedRank(Tagged(1L, markerOf(2, boom).orThrow()), boom) == 10)
    }

    // The same two axes as the bounded-leaf matrix above, on an opaque HANDLE
    // leaf — whose `None` rides `0L`, because a `Box` pointer is never zero,
    // rather than a declared sentinel.
    //
    // The fourth row is why this runs rather than being asserted as text: an
    // `Option<handle>` under an absent ancestor collapses two absences into one
    // nullable typed view, and the two halves that carry them — the JNI
    // descriptor and the encoder's `jvalue` — can disagree while both still
    // compile. That is #433, and the first fix for it broke this row while
    // fixing the second one.
    section("an optional handle leaf reads its own niche and its ancestor's null") {
        // Ancestor absent: both leaves are null through the `?.` alone, and the
        // one WITH a niche must not report its sentinel as anything else.
        check(vaultHolderNew(-1L, 5L, 7L, boom) { always, maybe ->
            "${always == null}/${maybe == null}"
        } == "true/true")

        // Ancestor present, leaf absent: only the leaf with a niche of its own
        // can be null here, and it is — through `0L`, not through a JVM null.
        // The live one is a freshly minted OWNING handle, so it is closed after
        // reading; `Ingot` is a plain `NativeHandle` with no Cleaner backstop,
        // and dropping the only reference would leak the allocation.
        check(vaultHolderNew(0L, 5L, -1L, boom) { always, maybe ->
            check(maybe == null)
            val a = always ?: error("the ancestor-nullable leaf was dropped")
            a.use { it.grams(boom) }
        } == 5L)

        // Both present: each handle points at its own object, and each is the
        // JVM's to close.
        check(vaultHolderNew(0L, 5L, 7L, boom) { always, maybe ->
            val a = always ?: error("the ancestor-nullable leaf was dropped")
            val m = maybe ?: error("the niche-carrying leaf was dropped")
            val total = a.grams(boom) + m.grams(boom)
            // Each is the JVM's to close, and closing one leaves the other
            // alone: they are distinct objects, not one pointer delivered twice.
            a.close()
            check(a.isClosed() && !m.isClosed())
            m.close()
            check(m.isClosed())
            total
        } == 12L)
    }

    // The same layers as a CALLBACK ARGUMENT — the direction #429's fix did not
    // reach.
    //
    // A sum payload and a callback argument are converted by different
    // emitters, and each peeled the layers its own way: #432 taught the first,
    // and the `asRaw` proxy went on applying the leaf's conversion to the whole
    // value until #438. Both run one walk now, and this is the half a compiler
    // cannot check — that the list arrives element by element, absences and all,
    // rather than as one converted thing.
    section("a callback argument carries its Option and collection layers") {
        val seen = mutableListOf<List<ULong?>>()
        ticksEmit({ vec -> seen.add(vec) }, boom)

        check(seen.size == 2)
        // A mixed list: each element converted, the absence preserved in place
        // rather than collapsing the list or the value.
        check(seen[0] == listOf(4uL, null, 6uL))
        // …and an empty one is empty, not null.
        check(seen[1].isEmpty())
    }

    // The sum whose payloads have LAYERS. The class that reassembles a variant
    // from wire slots is Kotlin, and between a slot and the property there can
    // be a collection, an `Option`, and the leaf conversion. Applying the leaf's
    // conversion straight to the slot compiled nothing at all (#429), and the
    // Rust half is identical either way — so a compiled run is the only thing
    // that holds this line. The two controls at the end are half of it: a run
    // that needs no element conversion must NOT be distributed over, or a
    // `ByteArray` property comes back a `List<Byte>`.
    section("a sum payload carries its Option and collection layers") {
        // `Option<u64>`: JVM null is the absent case, not an error.
        check((layeredOf(0, boom).orThrow() as Layered.Count).v0 == null)
        check((layeredOf(1, boom).orThrow() as Layered.Count).v0 == 4uL)

        // `Option<handle>`: the absence rides the handle's own niche, so the
        // absent case is `0L` in a primitive slot rather than a JVM null in a
        // boxed one (#433). Both arms run, and closing the present one closes
        // the handle it minted.
        check((layeredOf(2, boom).orThrow() as Layered.Held).v0 == null)
        val held = layeredOf(3, boom).orThrow() as Layered.Held
        val summary = held.v0 ?: error("the present arm dropped the handle")
        check(summary.count(boom) == 4L)
        held.close()
        check(summary.isClosed())

        // `Vec<Option<u64>>`: the absences are inside the list, so a mixed one
        // arrives element by element rather than as one null.
        check((layeredOf(4, boom).orThrow() as Layered.Many).v0 == listOf(1uL, null, 3uL))

        // …and the same two layers in the other order. An absent run is null
        // rather than an empty list, and a present one still converts element by
        // element.
        check((layeredOf(5, boom).orThrow() as Layered.Values).v0 == null)
        check((layeredOf(6, boom).orThrow() as Layered.Values).v0 == listOf(5uL, null))

        // Layers nest, and the conversion belongs at the bottom of the stack.
        check(
            (layeredOf(7, boom).orThrow() as Layered.Nested).v0 ==
                listOf(listOf(6uL, null), emptyList())
        )

        // The controls. A `Vec<u8>` payload is a `ByteArray`, and a payload with
        // no layer is passed straight through.
        check((layeredOf(8, boom).orThrow() as Layered.Blob).v0.toList() == listOf<Byte>(1, 2, 3))
        check((layeredOf(9, boom).orThrow() as Layered.Plain).v0 == 7L)
    }

    // ── value_class: by-value bytes, instance accessors, Vec<value> → List ────
    section("value_class Stamp") {
        val st: Stamp = stampNew(7L, 42L, boom).orThrow()
        check(st.secs(boom) == 7L)
        check(st.nanos(boom) == 42L)
        val series: List<Stamp> = stampSeries(3L, boom).orThrow()
        check(series.size == 3)
        check(series[0].secs(boom) == 0L)
        check(series[2].secs(boom) == 2L && series[2].nanos(boom) == 0L)
        check(stampSeries(0L, boom).orThrow().isEmpty())
    }

    // ── array-backed VALUE EQUALITY ─────────────────────────────────────────
    // The Rust types derive `Eq`, so the Kotlin mirrors must compare by
    // content. Kotlin arrays compare by IDENTITY, which silently breaks that
    // for every `ByteArray`-backed property — a `Vec<u8>` field, a fixed-size
    // `[u8; N]` field, and any value carrying one of those. Kotlin's own
    // `data class` codegen is inconsistent here (its `hashCode`/`toString` DO
    // special-case arrays, its `equals` does not), so `==` is the assertion
    // that matters and `hashCode` alone would not have caught the defect.
    section("array-backed value equality (content, not identity)") {
        // The NEGATIVE case first: a data class with no array property must keep
        // the compiler's own equality. The generator emits nothing for it, so
        // this pins that the content operators do not churn ordinary classes.
        val s1 = stampNew(7L, 42L, boom).orThrow()
        val s2 = stampNew(7L, 42L, boom).orThrow()
        check(s1 == s2) { "scalar data class must compare by value: $s1 vs $s2" }
        check(s1.hashCode() == s2.hashCode())
        check(stampNew(8L, 42L, boom).orThrow() != s1) { "different content must not compare equal" }
        check(hashSetOf(s1, s2).size == 1)
        check(s1.toString() == "Stamp(secs=7, nanos=42)") { "got $s1" }

        // A data class with a DIRECT `ByteArray` field plus a NESTED data
        // class — the two shapes that broke downstream. The bytes sit LAST, so
        // this also covers the `31 * result + …contentHashCode()` fold form
        // that a real value (`Timestamp(ntp64, id)`) produces.
        fun blob(secs: Long, id: ByteArray, chunks: List<ByteArray>) =
            blobValueNew(secs, id, chunks, boom).orThrow()

        val chunks = listOf(byteArrayOf(9), byteArrayOf(8, 7))
        val b1 = blob(7L, byteArrayOf(1, 2, 3), chunks)
        val b2 = blob(7L, byteArrayOf(1, 2, 3), listOf(byteArrayOf(9), byteArrayOf(8, 7)))
        check(b1 == b2) { "array-backed data class must compare by content: $b1 vs $b2" }
        check(b1.hashCode() == b2.hashCode())
        check(hashSetOf(b1, b2).size == 1)
        // Every component must actually participate — a comparison that ignored
        // any of them would still pass the equality checks above.
        check(blob(7L, byteArrayOf(1, 2, 4), chunks) != b1) { "id must matter" }
        check(blob(8L, byteArrayOf(1, 2, 3), chunks) != b1) { "nested stamp must matter" }
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
        // is the fixed-size-array length check.
        var lenErr: String? = null
        arraysEcho(a1.copy(ints = intArrayOf(1, 2))) { je -> lenErr = je; a1 }
        check(lenErr?.contains("fixed-size array decode") == true) {
            "wrong-length array must report a binding error, got: $lenErr"
        }

        // WHOLE-OBJECT input decode (`.jobject_input()`): the decoder reads the
        // nested data class, direct byte array, and list of byte arrays from the
        // Kotlin object by their JVM descriptors.
        check(blobValueEcho(b1, boom) == b1) { "jobject-input round trip must preserve the value" }
        check(blobValueEcho(blob(0L, ByteArray(0), emptyList()), boom).orThrow().chunks.isEmpty())
    }

    // ── Option<scalar> nullable primitive return + data_class instance
    // member (I5): the receiver crosses as `this`'s field leaves ────────────
    section("Option<i64> Payload.labelLen") {
        check(payload(1L, 0, 0.0, false, "abcd").labelLen(boom) == 4L)
        check(payload(1L, 0, 0.0, false, null).labelLen(boom) == null)
    }

    // ── ptr_class members + Option<Payload>/Option<Vec>/Vec round-trips ──────
    section("Storage members + Option/Vec round-trips") {
        val s = storageNew(boom).orThrow()
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
        val s = Storage.withPayload(payload(99L, 0, 0.0, false, "z"), boom).orThrow()
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
        val s = storageNew(boom).orThrow()
        val r: CovResource = s
        check(r.live)                     // default over inherited peek()/isClosed()
        check(r.isEmpty())                // default over class-specific len()
        check(r.len(boom) == 0L)          // generated member through the interface
        storagePutByTake(s, payload(7L, 0, 0.0, false, null), boom)
        check(!r.isEmpty())
        check(r.len(boom) == 1L)
        s.close()
        check(!r.live)
        // `peek()` is opt-in (#37); a closed handle reads back as 0.
        @OptIn(UnsafeNativeApi::class)
        val closedPtr = r.peek()
        check(r.isClosed() && closedPtr == 0L)

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
        val s = storageNew(boom).orThrow()
        storagePutSlice(
            s,
            listOf(payload(1L, 0, 0.0, false, null), payload(2L, 0, 0.0, false, null), payload(3L, 0, 0.0, false, null)),
            boom,
        )

        // payload_handler_new: closure decoded once, fires once per payload.
        var perElem = 0L
        val h = payloadHandlerNew(PayloadCallback { p -> perElem += p.id }, boom).orThrow()
        storageCallback(s, h, boom)
        check(perElem == 6L)
        h.close()

        // Option<Payload> is deconstructed by one registry chain: presence plus
        // the Product intermediate, then reassembled as a nullable Payload here.
        val optionalSeen = mutableListOf<Payload?>()
        val optionalCb = PayloadOptionalCallback { optionalSeen += it }
        payloadOptionalEmit(true, optionalCb, boom)
        payloadOptionalEmit(false, optionalCb, boom)
        check(optionalSeen == listOf(payload(91L, 7, 2.5, true, "optional-callback"), null))

        // The optional chain still owns the handles nested in a delivered data
        // class. The callback may use them, but the bridge closes untaken
        // ownership after the callback returns. The absent arm owns nothing.
        val escapedTokens = mutableListOf<Ingot>()
        val holderSeen = mutableListOf<Long?>()
        callbackHolderOptionalEmit(true, CallbackHolderOptionalCallback { holder ->
            val value = holder ?: error("present CallbackHolder was delivered as null")
            check(value.token.grams(boom) == 23L)
            holderSeen += value.tag
            escapedTokens += value.token
        }, boom)
        check(escapedTokens.single().isClosed())
        callbackHolderOptionalEmit(false, CallbackHolderOptionalCallback { holder ->
            check(holder == null)
            holderSeen += null
        }, boom)
        check(holderSeen == listOf(17L, null))

        // payload_vec_handler_new: whole batch delivered once as List<Payload>.
        var batchSize = -1
        var batchSum = 0L
        val vh: PayloadVecHandler = payloadVecHandlerNew(
            PayloadListCallback { list -> batchSize = list.size; batchSum = list.sumOf { it.id } },
            boom,
        ).orThrow()
        storageCallbackVec(s, vh, boom)
        check(batchSize == 3)
        check(batchSum == 6L)
        vh.close()
        s.close()
    }

    // ── flatten matrix on Summary: output (default/suppress/with) ────────────
    section("flatten_output (default / suppress / with)") {
        val s = storageNew(boom).orThrow()
        storagePutSlice(s, listOf(payload(1L, 0, 10.0, false, null), payload(2L, 0, 30.0, false, null)), boom)

        // flatten_output DEFAULT: decompose into (count, total) leaves via builder.
        val pair = storageSummary(s, boom) { count, total -> count to total }.orThrow()
        check(pair.first == 2L && pair.second == 40.0)

        // flatten_output_suppress: keep the raw opaque handle.
        val raw: Summary = storageSummaryHandle(s, boom).orThrow()
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
        }.orThrow()
        check(full.first == 2L && full.second == 40.0)
        check(fullHandle!!.total(boom) == 40.0)
        fullHandle!!.close()
        s.close()

        // Generic R? recovery: a binding failure may deliberately decline to
        // fabricate an R by returning null from the handler.
        var genericRecoveryError: String? = null
        val recovered: String? = storageSummary(
            s,
            JniErrorHandler<String?> { je ->
                genericRecoveryError = je
                null
            },
        ) { count, total -> "$count:$total" }
        check(recovered == null)
        check(genericRecoveryError?.contains("closed native handle") == true) {
            "unexpected generic recovery error: $genericRecoveryError"
        }
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
        val s = storageNew(boom).orThrow()

        // Empty storage: count == 0 ⇒ the predicate fails ⇒ null handle.
        val emptyProbe = storageSummaryProbe(s, boom) { count, total, handle ->
            Triple(count, total, handle)
        }.orThrow()
        check(emptyProbe.first == 0L && emptyProbe.second == 0.0)
        check(emptyProbe.third == null) { "empty summary must arrive value-only" }

        // Non-empty: the handle arrives live alongside the decomposed values.
        storagePutSlice(s, listOf(payload(1L, 0, 10.0, false, null), payload(2L, 0, 30.0, false, null)), boom)
        val probe = storageSummaryProbe(s, boom) { count, total, handle ->
            Triple(count, total, handle)
        }.orThrow()
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
        val m = Summary.fromMean(4L, 2.5, boom).orThrow()
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

    // ── expanded nullable u16 leaf: primitive native pair, never JObject ────
    section("expanded selector keeps nullable scalar off JObject") {
        // -1 skips every arm; 0 rebuilds from the nullable constructor slots.
        // The latter crosses JNI as Boolean + Int and executes the registry
        // Optional chain, without allocating/unboxing java.lang.Integer.
        check(selectorCodeScore(-1, null, null, null, boom) == -1L)
        check(selectorCodeScore(0, 41, byteArrayOf(1, 2), null, boom) == 43L)

        // The identity arm uses the same expansion and proves its handle slot
        // remains independent of the primitive build-arm leaves.
        val code = SelectorCode.new(50, byteArrayOf(1, 2, 3), boom).orThrow()
        check(selectorCodeScore(1, null, null, code, boom) == 53L)
        code.close()
    }

    // ── owned Option<handle>: shared registry Optional chain ──────────────
    section("owned optional handle input consumes only the present arm") {
        check(ingotOptionalGrams(null, boom) == -1L)

        val ingot = Ingot.new(37L, boom).orThrow()
        check(ingotOptionalGrams(ingot, boom) == 37L)
        check(ingot.isClosed())
    }

    // ── recursive constructor expansion: nested build and identity arms ─────
    section("recursive constructor expansion freezes nested constructors") {
        // The outer SummaryEnvelope constructor receives a Summary. Exercise
        // both variants of that nested Summary build before the outer call.
        check(summaryEnvelopeScore(0, 4L, 10.0, null, 3L, boom) == 17L)
        val nested = Summary.of(5L, 12.0, boom).orThrow()
        check(summaryEnvelopeScore(1, null, null, nested, 2L, boom) == 19L)
    }

    // ── flatten input on Summary: default + with, both selectors ─────────────
    section("flatten_input (default / with), leaves + handle") {
        val s = storageNew(boom).orThrow()
        storagePutSlice(s, listOf(payload(1L, 0, 10.0, false, null), payload(2L, 0, 30.0, false, null)), boom)

        // constructor + accessors + method on the analytics handle.
        val sum = Summary.of(2L, 40.0, boom).orThrow()
        check(sum.count(boom) == 2L && sum.total(boom) == 40.0 && sum.scaled(0.5, boom) == 20.0)
        sum.close()

        // #52 single-param `.split_on_param("expected")` on the CLASS-DEFAULT
        // `Summary` variants: idiomatic typed forms delegating to the selector.
        check(storageMatchesSummary(s, 2L, 40.0, boom))       // build-from-leaves arm
        check(!storageMatchesSummary(s, 1L, 40.0, boom))
        val h0 = Summary.of(2L, 40.0, boom).orThrow()
        check(storageMatchesSummary(s, h0, boom))             // pass-handle arm
        // The selector form stays public underneath (raw arm dispatch).
        check(storageMatchesSummary(s, 0, 2L, 40.0, null, boom))

        // #52 single-param split via a per-fn `.expand_param` override.
        check(storageExpectSummary(s, 2L, 40.0, boom))        // build-from-leaves arm
        val h1 = Summary.of(2L, 40.0, boom).orThrow()
        check(storageExpectSummary(s, h1, boom))              // pass-handle arm

        // #52 CARTESIAN PRODUCT: two split params → the 2×2 grid of typed
        // overloads, all four combinations distinct. Build args are prefixed
        // with the origin parameter name (`primaryCount`, `fallbackTotal`); the
        // handle arm consumes its `Summary`, so each is a fresh handle.
        check(summaryPrefer(2L, 40.0, 1L, 1.0, boom) == 1L)                       // build / build
        check(summaryPrefer(1L, 1.0, Summary.of(3L, 99.0, boom).orThrow(), boom) == 0L)     // build / handle
        check(summaryPrefer(Summary.of(3L, 99.0, boom).orThrow(), 1L, 1.0, boom) == 1L)     // handle / build
        check(
            summaryPrefer(Summary.of(1L, 1.0, boom).orThrow(), Summary.of(3L, 99.0, boom).orThrow(), boom) == 0L,
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
            summaryMerge(2L, 40.0, Summary.of(1L, 2.0, boom).orThrow(), boom) { count, _ -> count } == 3L,
        )                                                                          // build / handle
        check(
            summaryMerge(Summary.of(2L, 40.0, boom).orThrow(), 1L, 2.0, boom) { _, total -> total } == 42.0,
        )                                                                          // handle / build
        check(
            summaryMerge(Summary.of(2L, 40.0, boom).orThrow(), Summary.of(1L, 2.0, boom).orThrow(), boom) {
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
            val shared = Summary.of(5L, 55.0, boom).orThrow()
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
            check(summaryPrefer(shared, Summary.of(3L, 99.0, boom).orThrow(), boom) == 0L)
        }

        // Optional combined-selector expansion: `Option<&Summary>` under the
        // dual-arm type default. The selector also encodes absence (-1 = None);
        // the borrow-identity arm CLONES, so the handle survives the call.
        check(summaryTotalOpt(-1, null, null, null, boom) == -1.0)     // absent
        check(summaryTotalOpt(0, 2L, 40.0, null, boom) == 40.0)        // build arm
        val hOpt = Summary.of(3L, 99.0, boom).orThrow()
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
        val ok = storageTryWithLabel("hi", boom, boomStorage).orThrow()
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
        val ok = storageTryFromStamp(stampNew(5L, 0L, boom).orThrow(), byteArrayOf(1, 2), boom, boomStorage).orThrow()
        check(ok.len(boom) == 1L)
        ok.close()

        // DOMAIN error (well-formed Stamp, rejected value): `onError` fires,
        // `onBindingError` must NOT. The handler declines to fabricate a Storage.
        var domainMsg: String? = null
        val domainRet = storageTryFromStamp(
            stampNew(-1L, 0L, boom).orThrow(),
            byteArrayOf(1, 2),
            JniErrorHandler<Storage?> { je ->
                throw AssertionError("binding channel must not fire on a domain error: $je")
            },
            StorageErrorHandler<Storage?> { message, handle ->
                domainMsg = message
                check(handle.message(boom) == "stamp secs must be positive")
                handle.close()
                null
            },
        )
        check(domainMsg == "stamp secs must be positive") { "domain onError did not fire: $domainMsg" }
        check(domainRet == null)

        // BINDING error (wrong-length `tag` array): `onBindingError` fires,
        // the domain `onError` must NOT.
        var bindingJe: String? = null
        val bindingRet = storageTryFromStamp(
            Stamp(1L, 0L),
            byteArrayOf(1, 2, 3),   // `tag` is [u8; 2]; 3 must be rejected on decode
            JniErrorHandler<Storage?> { je ->
                bindingJe = je
                null
            },
            StorageErrorHandler<Storage?> { _, handle ->
                handle.close()
                throw AssertionError("domain channel must not fire on a binding error")
            },
        )
        check(bindingJe != null && bindingJe!!.contains("fixed-size array decode")) {
            "binding onBindingError did not fire: $bindingJe"
        }
        check(bindingRet == null)
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
        val shards = storageShards(3L, 2L, boom).orThrow()
        check(shards.size == 3)
        check(shards.all { it.len(boom) == 2L })
        check(shards[2].contains(2001L, boom))   // distinct, correctly-typed handles
        check(!shards[0].contains(2001L, boom))
        shards.forEach { it.close() }
        check(storageShards(0L, 2L, boom).orThrow().isEmpty())
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
        ).orThrow()
        storageEmit(5L, h, boom)
        check(openInRun && seenLen == 5L)
        // close-unless-taken: the proxy closed the handle after run.
        check(escaped!!.isClosed())
        h.close()
    }

    // ── nested data_class + Option<prim>/Option<enum> FIELDS ─────────────────
    section("nested data_class Annotated + Option fields") {
        val p = payload(7L, 1, 2.5, true, "x")
        val a = annotatedNew(p, 30L, Priority.HIGH, boom).orThrow() // output: nested fromParts
        check(a.payload == p && a.ttl == 30L && a.priority == Priority.HIGH)
        check(annotatedTtl(a, boom) == 30L)                 // input: (present, value) pair
        check(annotatedPriority(a, boom) == Priority.HIGH)  // Option<enum> return
        check(annotatedPayloadValue(a, boom) == 2.5)        // nested field survived decode
        check(annotatedAlternateValue(a, boom) == null)     // Option<nested> absent gate
        val none = annotatedNew(payload(1L, 0, 0.0, false, null), null, null, boom).orThrow()
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
        val a: SummaryVault = archiveNew(boom).orThrow()
        check(archiveLatest(a, boom) == null)               // None → null
        val s = Summary.of(2L, 40.0, boom).orThrow()
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
        val a: SummaryVault = archiveNew(boom).orThrow()
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
        // Both now take the push-helper path through the ONE `payloadVec` trio
        // (#296): the wrapped element is keyed on the canonical `Payload`, and
        // its `Box` goes back on where the Vec is consumed. Weighed against each
        // other rather than each against a literal — the claim is that a `Box`
        // the model erases changes neither the surface nor the answer, and
        // `boxedRunIdSum` is the bare-element control that always took this path.
        check(boxedElemIdSum(many, boom) == boxedRunIdSum(many, boom))
        check(boxedElemIdSum(emptyList(), boom) == 0L)    // …and the empty run

        // The two spellings of a BORROWED run (#384). `&[Payload]` and
        // `&Vec<Payload>` are one type to the model, so both take the Vec-build
        // path and must come out identical — the `&Vec` one used to be handed a
        // `&[Payload]`, which is an E0308 in the generated crate rather than a
        // wrong answer. Weighed against each other, not against a literal.
        check(refVecIdSum(many, boom) == sliceIdSum(many, boom))
        check(sliceIdSum(many, boom) == 3L)
        check(refVecIdSum(emptyList(), boom) == 0L)       // …and the empty run

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
        val held = Summary.of(4L, 8.0, boom).orThrow()
        check(holderTagOr(Holder(3L, held), -9L, boom) == 7L)  // 3 + count(4)
    }

    // ── Vec<String> fold + Option<data-class> input + plain String return ────
    section("Vec<String> storageLabels + Option<Payload> input + String return") {
        val s = storageNew(boom).orThrow()
        check(storageLabels(s, boom).orThrow().isEmpty())
        storagePutSlice(
            s,
            listOf(payload(1L, 0, 0.0, false, "a"), payload(2L, 0, 0.0, false, null), payload(3L, 0, 0.0, false, "c")),
            boom,
        )
        check(storageLabels(s, boom).orThrow() == listOf("a", "c"))
        check(storagePutOpt(s, payload(4L, 0, 0.0, false, "d"), boom))   // Some → pushed
        check(!storagePutOpt(s, null, boom))                              // None → not
        check(s.len(boom) == 4L)
        check(storageLabels(s, boom).orThrow() == listOf("a", "c", "d"))
        check(stringNew("hello", boom) == "hello")
        check(stringNew("", boom) == "")
        s.close()
    }

    // ── binding error: je != null (fixed-size array length guard) ───────────
    section("binding error je != null (wrong-length fixed-size array)") {
        var je: String? = null
        val fallback = storageTryFromStamp(
            stampNew(1L, 0L, boom).orThrow(),
            byteArrayOf(1, 2, 3),   // `tag` is [u8; 2]; 3 is rejected on decode
            JniErrorHandler { e ->
                je = e
                storageNew(boom).orThrow()
            },
            StorageErrorHandler { _, handle ->
                throw AssertionError("domain channel must not fire on a decode failure")
            },
        ).orThrow()
        fallback.close()
        check(je != null && je!!.contains("fixed-size array decode")) { "unexpected je: $je" }
    }

    // ── callback exceptions: swallowed per upcall (no-throw contract) ────────
    // A callback that throws must not corrupt the surrounding native call: the
    // trampoline describes + clears the pending exception per upcall (the stack
    // trace printed below is EXPECTED output) and delivery continues.
    section("callback exceptions are swallowed (no-throw contract)") {
        val s = storageNew(boom).orThrow()
        storagePutSlice(s, listOf(payload(1L, 0, 0.0, false, null), payload(2L, 0, 0.0, false, null)), boom)
        var fired = 0
        val h = payloadHandlerNew(
            PayloadCallback { fired++; throw RuntimeException("deliberate covertest exception") },
            boom,
        ).orThrow()
        storageCallback(s, h, boom)   // must not throw at the call site
        check(fired == 2) { "every payload must still be delivered, got $fired" }
        storageCallback(s, h, boom)   // the handler stays usable
        check(fired == 4)
        h.close()
        s.close()
    }

    // ── 3-handle sorted locking + concurrent smoke ───────────────────────────
    section("3-handle locking + 2-thread smoke") {
        val s1 = Storage.withPayload(payload(1L, 0, 0.0, false, null), boom).orThrow()
        val s2 = Storage.withPayload(payload(2L, 0, 0.0, false, null), boom).orThrow()
        val s3 = storageNew(boom).orThrow()
        check(storageTotalLen(s1, s2, s3, boom) == 2L)
        check(storageTotalLen(s3, s2, s1, boom) == 2L)   // argument order irrelevant
        // Opposite lock-acquisition orders + a writer on a shared handle: the
        // sorted N-ary locking must neither deadlock nor tear.
        val iterations = 2_000
        val errs = AtomicInteger()
        val s4 = storageNew(boom).orThrow()
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
        for (i in 0 until slots) pool.set(i, storageNew(boom).orThrow())
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
                    val old = pool.getAndSet(i, storageNew(boom).orThrow())
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
        val s = storageNew(boom).orThrow()
        val n = 5_000
        storagePutSlice(
            s,
            List(n) { payload(it.toLong(), it, it.toDouble(), false, if (it % 2 == 0) "L$it" else null) },
            boom,
        )
        var count = 0L
        var sum = 0L
        val h = payloadHandlerNew(PayloadCallback { p -> count++; sum += p.id }, boom).orThrow()
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
        val a = Summary.of(2L, 40.0, boom).orThrow()
        check(a.total(boom) == 40.0)
        a.close()
        check(a.isClosed())
        a.close() // double close: ticket already settled — no double free
        var closedErr: String? = null
        a.total { je -> closedErr = je; -1.0 }
        check(closedErr != null && closedErr!!.contains("closed native handle"))

        // take(): ticket moves into the fresh wrapper; the source is closed.
        val b = Summary.of(3L, 60.0, boom).orThrow()
        val c = b.take()
        check(b.isClosed() && !c.isClosed())
        check(c.total(boom) == 60.0)
        b.close() // settled ticket: no-op
        c.close()

        // By-value consumption settles the ticket (markConsumed): the summary
        // is freed by Rust, and neither close nor the Cleaner may free again.
        val d = Summary.of(2L, 40.0, boom).orThrow()
        check(summaryTotalRaw(d, boom) == 40.0)
        check(d.isClosed())
        d.close()

        // Cleaner backstop: churn unreachable handles through every state —
        // never-released (GC action must free), explicitly closed, consumed —
        // then force GC so the cleaner thread settles the survivors. Any
        // double free or free-under-use aborts the JVM here.
        repeat(2_000) { i ->
            val s = Summary.of(i.toLong(), i.toDouble(), boom).orThrow()
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
        val e = Summary.of(5L, 50.0, boom).orThrow()
        check(e.count(boom) == 5L)
        e.close()
    }

    // ── JNI native-symbol escaping (#86) ─────────────────────────────────────
    section("JNI native-symbol escaping (esc_pkg / Esc_Probe / snake extern)") {
        // Every call here resolves a Rust export whose symbol needs the JNI
        // spec's `_1` escaping — `esc_1pkg` + `Esc_1Probe` in the freePtr
        // destructor, `escape_1probe_1value` on the harness extern. A raw
        // dot-to-underscore symbol would throw UnsatisfiedLinkError.
        val p = Esc_Probe.escapeProbeNew(7L, boom).orThrow()
        check(p.escapeProbeValue(boom) == 7L)
        p.close()
    }

    // ── The raw-pointer surface is closed to Java (#37) ──────────────────────
    section("raw-pointer surface is invisible to javac (bytecode check)") {
        // `internal` and `@RequiresOptIn` are Kotlin-source constructs: the
        // JVM sees a public member under a mangled name, and javac enforces
        // no opt-in. What actually stops a Java caller is `private` (for the
        // constructors, which `@JvmSynthetic` cannot target) and
        // ACC_SYNTHETIC, which javac skips during resolution. Assert the
        // flags on the emitted bytecode rather than trusting the source.
        fun ctorsHidden(c: Class<*>) = c.declaredConstructors.all {
            java.lang.reflect.Modifier.isPrivate(it.modifiers) || it.isSynthetic
        }
        for (c in listOf(Storage::class.java, Summary::class.java, Esc_Probe::class.java)) {
            check(ctorsHidden(c)) { "${'$'}{c.name} has a Java-callable constructor" }
        }
        // peek() keeps its name — Rust calls it through JNI — but not its
        // visibility to javac.
        check(NativeHandle::class.java.getDeclaredMethod("peek").isSynthetic)
        // Every extern, and the per-class static free. `internal object` is a
        // public JVM class, so the object's own visibility guards nothing.
        val externs = CovNative::class.java.declaredMethods.filter { java.lang.reflect.Modifier.isNative(it.modifiers) }
        check(externs.isNotEmpty())
        check(externs.all { it.isSynthetic }) {
            "Java-callable externs: ${'$'}{externs.filterNot { it.isSynthetic }.map { it.name }}"
        }
        check(Storage::class.java.declaredMethods.filter { java.lang.reflect.Modifier.isNative(it.modifiers) }.all { it.isSynthetic })
        // Mutable pointer state: a visible setter would let a caller repoint a
        // live handle and have the next generated call free that address.
        val ptrAccessors = NativeHandle::class.java.declaredMethods.filter {
            it.name.startsWith("getPtr") || it.name.startsWith("setPtr")
        }
        check(ptrAccessors.isNotEmpty())
        check(ptrAccessors.all { it.isSynthetic })

        // The callback adapters. `X.asRaw()` returns the generated proxy whose
        // `run` takes handle leaves as bare `Long`s and calls `fromRawPtr` on
        // them inside a file that holds the blanket opt-in — so a public
        // `asRaw` handed any caller a forged-pointer route with no opt-in of
        // its own. They are extension functions, hence statics on the file
        // facade class.
        val facades = listOf("io.prebindgen.covertest.CovertestKt", "io.prebindgen.covertest.model.ModelKt")
            .mapNotNull { runCatching { Class.forName(it) }.getOrNull() }
        // `$lambda$N` bodies share the prefix and are `private`, so javac
        // cannot resolve them either way; the adapters themselves are what
        // must carry the flag.
        val asRaws = facades.flatMap { it.declaredMethods.toList() }
            .filter { it.name.startsWith("asRaw") && !java.lang.reflect.Modifier.isPrivate(it.modifiers) }
        check(asRaws.isNotEmpty()) { "no asRaw adapters found to check" }
        check(asRaws.all { it.isSynthetic }) {
            "Java-callable asRaw in " + asRaws.filterNot { it.isSynthetic }.map { it.declaringClass.name }
        }
        // The hoisted folder singletons are the same route without the
        // extension: `internal object` is a public JVM class and `@JvmField` a
        // public static, so `__StorageFolderRawHolder.instance.run(list,
        // 0xdeadbeefL)` would mint a handle from an invented pointer.
        val holder = Class.forName("io.prebindgen.covertest.__StorageFolderRawHolder")
        val instance = holder.getDeclaredField("instance")
        check(instance.isSynthetic) { "__StorageFolderRawHolder.instance is Java-readable" }
        // And the hoisted builder singletons: a top-level `internal val` has a
        // private backing field but a facade getter javac resolves like any
        // other static, so `ModelKt.get__LookupBuilderRaw().run(0, 0xdeadbeefL,
        // null)` would mint a handle from an invented pointer.
        val getters = facades.flatMap { it.declaredMethods.toList() }
            .filter { it.name.startsWith("get__") }
        check(getters.isNotEmpty()) { "no builder singletons found to check" }
        check(getters.all { it.isSynthetic }) {
            "Java-callable builder singleton: " + getters.filterNot { it.isSynthetic }.map { it.name }
        }
    }

    println("PASS - $sectionCount sections, every JniGen feature exercised")
}
