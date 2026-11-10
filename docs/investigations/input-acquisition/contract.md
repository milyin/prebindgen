# Proposed input execution and ownership contract

This is a design for review, not implemented API. Read the
[pipeline overview](README.md) before this document. Names in the sketches refer
to existing prebindgen types unless identified as a rename.

## Meaning of the operations

Keep the existing generation interfaces: `Flat` stores converters, `Conversion`
accepts generator-defined requests, and C/JNI opt into the shared staged protocol.
Other adapters remain free to use their own protocols. The recently simplified
`Codec` declaration interface needs no new required method.

The recommendation is to retain the three useful scoped input operations,
rename the misleading first one, and give every converter the same composition
order. Each request still produces a Rust expression, with its continuation
inside the scope of any locals introduced by the converter.

| Proposed operation | Contract |
| --- | --- |
| `Prepare` (renamed `Validate`) | Decode/check wire values and establish required temporary storage. May also take incoming owners and construct the argument, as compiled recipes already do. Every applicable wire check precedes that wire value's use. |
| `Acquire` | Acquire owners not already taken during preparation. Each transfer must be recorded by the acquisition code. It may fail or unwind between acquisitions if the receipt remains truthful. |
| `Construct { target, next }` | Bind the argument as `target` and run `next` while the argument and its supporting storage remain valid. May run source constructors or create references; any ownership take here has the same recording obligation. |
| `Acknowledge` | Return the once-per-call receipt-readiness code, or `()`. Run it before any input operation can transfer an owner. Preserve existing detection of conflicting readiness code. |
| `Resources` | Describe the converter's foreign allocation types and access modes for the language adapter. This query is generation metadata, not an execution schedule. |

`Prepare` does not promise to perform every check in the complete call before
any owner can move. Checks inside a repeated row can remain inside that row's
construction. Neither the compiler nor a converter should add an array of
prepared closures merely to simulate a global validation phase.

The prepared-plan field `InputPlan::validation` becomes `preparation` and follows
the same meaning as `Prepare`. `acquire`, `statements` and `argument` retain their
roles. These remain storage for implementing the protocol; independent
converters need not construct an `InputPlan`.

Schematically, the enum changes one variant:

```rust,ignore
pub enum InputFragment<'a, C> {
    Prepare { context: &'a C, next: TokenStream }, // Renamed Validate.
    Acquire { context: &'a C, next: TokenStream },
    Construct {
        context: &'a C,
        target: &'a Ident,
        next: TokenStream,
    },
    Acknowledge { context: &'a C },
    Resources { context: &'a C, resources: &'a mut Vec<Resource> },
}
```

## Composition order is independent of the storage used by a converter

The generated wrapper follows this order:

```text
make the receipt channel ready, when needed
Prepare(first) -> Acquire(first) -> Construct(first)
Prepare(second) -> Acquire(second) -> Construct(second)
...
call the source function
prepare outputs -> deliver outputs -> commit output ownership
```

An operation can be empty. For a compiled argument, most work stays in `Prepare`.
For a low-level borrowed leaf, `Prepare` can retain the raw pointer,
`Acquire` is empty, and `Construct` creates the reference. Both implementations
follow this one ordering rule.

The assembler builds the nested fragments in reverse while generating code so
that the resulting wrapper executes in forward parameter order. This is a
build-time traversal; it introduces no runtime loop over parameter descriptors.
Preserve the current flattening of tail blocks to avoid excessive nesting in
wide generated functions.

The previous schedule was generally all `Validate` fragments, then all
`Acquire` fragments, then all `Construct` fragments. Whether receipt readiness
ran before or after validation depended on `registered`. Replace those branches
with one schedule and put readiness before conversion. Preserve output ordering.

At the end of each operation, already acquired Rust owners must remain in scope
or have moved into another Rust owner. A failure unwinds those scopes normally.
A reference backed by a temporary, such as an explicitly reconstructed input
`String`, must retain that temporary through the source call.

## Ownership takes report themselves

The canonical receipt remains a consumed prefix. Its order follows the adapter's
existing traversal of **active owned wire values**, parameter by parameter,
then row/field order within each parameter. Shared/exclusive borrows and values
created locally without taking a foreign owner do not advance the prefix.

The helper that crosses an incoming ownership boundary must emit the matching
counter update. For example, this is generated Rust inside a wrapper, shown
schematically with error handling omitted:

```rust,ignore
let owner = unsafe { Box::from_raw(packet_pointer) }; // Existing allocation.
consumed.set(consumed.get() + 1);                    // Ownership now is Rust's.
let packet = *owner;
let packet = source_constructor(packet);           // May panic; prefix is set.
```

There is no allocation in `Box::from_raw`. Moving the value out releases its old
box allocation as part of normal Rust ownership handling. If a subsequent
constructor panics, Rust owns the value and destroys it; the caller must not
reconstruct ownership from the original pointer again.

| Acquisition | Receipt obligation |
| --- | --- |
| Reconstruct a foreign-owned `Box<T>` | Record after the owning `Box` exists and before later fallible/unwinding work. |
| Move an inline opaque owner out of C storage | Record after that owner has transferred. Preserve the existing empty replacement and its failure ordering. |
| Retain an incoming callback context as a Rust-owned closure | Record that context transfer once; preserve its destructor contract. |
| Borrow `&T` or `&mut T` | Do not count it. |
| Construct a Rust record from scalars | Do not count the locally created record. Count only any foreign owners its field conversions took. |
| Decode a repeated row | Record actual successful owner takes in that row. Completed Rust rows and partial row values remain under normal Rust cleanup. |

No observable failure or unwinding point may occur between an ownership take
and its report. Grouping updates is permissible only when all grouped takes
and intervening operations are proven infallible and cannot expose a partial
state to an observer. Treat the existing constructors, inline-owner replacement
factories and destructors as possible unwind points. Measure batching choices;
do not infer safety from a helper being called `Acquire`.

A converter must acquire owners in the same order as its declared foreign
ownership traversal. Self-reporting a count does not make arbitrary acquisition
orders representable. The generator cannot prove that arbitrary Rust fragments
obey this rule; it is part of the custom converter author's contract, just as
correct construction and cleanup are. Converters requiring a non-prefix transfer order still
need a distinct supported protocol; that is outside this proposal.

## Resource metadata stops deciding execution timing

Remove `Resource::registered` and `Resource::records_consumption` once every
owning acquisition path has been migrated. The assembler and compiler then
have no automatic count to insert on a leaf's behalf.

The current inventory also shows that `Resource::count` and `Resource::active`
feed that automatic counting and its template rendering. They do not drive
Kotlin's type/access coverage check. After replacing the counting consumers,
remove those fields rather than leave unused runtime expressions in metadata.
The intended resource description is therefore:

```rust,ignore
pub struct Resource {
    pub allocation_type: Option<Type>,
    pub access: Access, // Shared, Exclusive or Move.
}
```

This describes native access requirements, including repeated element schemas;
it is not a list of every runtime handle address. The existing wire tree and
Kotlin projection supply the actual handle sources and repetition.

Kotlin's `require_preflight` must be updated in the same migration. It currently
checks `registered`, but that flag is not a proof of live handles or correct
aliasing. Keep the real requirements: native resource allocation types and
access modes match the Kotlin handle wires, retained handle sources cover them,
carriers have their required preparation, and locks/preflight precede native use.
Prove the timing-flag removal against those checks. No native alias validator is
added. C continues to rely on its caller's pointer and aliasing obligations.

## Collections and callback results

Do not move collection conversion out of its row loop. Ordinary JNI rows can
retain row-local references; bulk rows can retain their existing cached reads;
C rows can retain their existing grouped buffer/enum/inline-owner handling.
The change is which code reports the take and how complete parameter fragments
are ordered, not a replacement transport or storage format.

Callbacks need two distinctions:

- Rust-to-foreign callback arguments are output conversions. Their ownership
  and cleanup protocols remain output protocols.
- A value returned by a foreign callback enters Rust through an input converter.
  Its decoder must use the same phase meaning and per-take accounting, with a
  receipt local to that invocation. It also retains the separate guards or
  failure-only scans that reclaim unread callback-owned values.

A callback may already have handed several owners to the binding before Rust
constructs the first source value. Rust must still reclaim an unread suffix on
failure. Ordinary incoming function arguments are different: unread owners
remain with the foreign caller, which learns the consumed prefix from the
receipt. Updating the input protocol does not merge these cleanup policies.

A callback-result decoder initializes its own local tracking and cleanup guards.
It does not reuse the ordinary JNI function-argument receipt array or run that
array's `Acknowledge` fragment. The shared requirement is truthful transfer
counting, not one receipt transport for every direction.

JNI callback decoders currently request the three phases directly and sometimes
substitute cached bulk reads. Migrate those sites alongside the assembler.
Moving cleanup guards after a potentially failing decode would leak callback
owners; keeping the guard/decoder ordering is an explicit acceptance criterion.

## Compatibility and migration

This is a breaking change for authors using the low-level staged protocol:
`Validate` and `validation` are renamed, resource accounting fields are removed,
and receipt updates become an acquisition helper's responsibility. The ordinary
Cbindgen/JniGen declaration syntax is unchanged.

Low-level conversion timing also changes: an earlier owner may transfer before
a later input is validated. A caller may therefore observe more consumed inputs
on failure. This is permitted by the chosen failure policy, but must be stated
in the migration guide and exercised in tests. Do not claim global prevalidation
or transactional restoration.

Migrate all built-in helpers and consumers on the implementation branch before
shipping the new protocol. Remove the old counting paths and old request variant;
keep one production contract. The work includes:

1. Boxed and inline-owner leaves, callback-context acquisition and C row helpers.
2. Declaration parameter/field/constructor composition and collection rows.
3. The common assembler and its receipt-readiness/ordering logic.
4. JNI forwarding conversions, preflight verification, callback-return decoders,
   and the independent-converter covertest fixtures.
5. Error-path cleanup, documentation, standalone probes and affected tests.

[The source inventory](evidence.md#source-inventory) identifies the current sites.
A rename alone does not complete any implementation checkbox. Supporting the
mixed cases requires execution and cleanup tests through the public emitters.
The implementation may be developed in stages, but each accepted combination
must already have a truthful receipt; retain rejection for any unproved case.

## Alternatives considered

| Alternative | Tradeoff |
| --- | --- |
| Exempt shared resources from the current mixed-mode rejection | A useful bounded fix, but leaves execution timing inferred from resource flags and leaves mixed owning conversions unresolved. |
| Add an explicit early/deferred mode to every converter | Makes today's classification explicit but retains two execution styles and the scheduling/receipt interaction. The proposed parameter order does not need that mode. |
| Require every converter to construct only after all validation | Would require revisiting row-local lifetimes and can reintroduce staging or repeat collection decoding. |
| Use a bitmap or journal instead of a prefix receipt | Can represent out-of-order ownership transfers, but changes the C/JNI protocol and can add storage or traversal. Source-ordered conversion avoids needing that for the documented cases. |
| One combined input-conversion request | An attractive later simplification, but current callback cached reads and row composition use the individual operations. This proposal first defines and migrates those operations consistently. |

## Acceptance and measurement

The implementation should satisfy these independently reviewable conditions:

- The six probe cases generate successfully under the new contract, with actual
  execution coverage for the mixed borrow/move cases and all-borrow cases.
- Two independently allocated owned inputs compose in either old converter style.
  Inject failure before acquisition, between takes, during construction and after
  source entry. The receipt identifies the transferred prefix exactly.
- Optional/selector inactivity skips both acquisition and counting. Constructors
  consuming multiple owners, nested collections and partially built rows clean up
  without a leak or duplicate destruction. Shared temporaries stay alive.
- Rust panics unwind through the supported boundary with the same ownership
  reporting obligations as ordinary errors. Abort configurations remain aborts.
- Callback-return failures drop constructed values and reclaim unread incoming
  owners once. Retain worker callbacks, cached bulk reads and nested row coverage.
- Kotlin preflight and locks still cover all declared native resources. C remains
  free of native alias checks. Required-input nullability and typed error delivery
  retain their existing behavior.
- Update generated review artifacts and review their code changes. Run the full
  C/JNI/Kotlin/Java boundary suite, including sanitizers and `-Xcheck:jni`, Rust
  workspace tests, Clippy, and the existing cross-target checks.

Measure the happy path separately from failure-cleanup correctness:

| Case | Reason to measure |
| --- | --- |
| Resource-free scalar calls and shared-only inputs | The new contract should not introduce receipt setup or handle bookkeeping for these cases. |
| Normal compiled inputs: one owner, several owners, constructors | These are the production default and already account for transfers locally. |
| Low-level deferred inputs: one, two and many owners | The old assembler could add one aggregate count; local updates may cost more. Inspect generated code and allocation/JNI call counts as well as timings. |
| Ordinary and bulk collections, small and large | Preserve grouped decoding, direct final construction and current caching. |
| Retained callback results with flat/nested owners | Preserve the optimized successful path and failure-only cleanup strategy. |

Use matching source functions and binding settings before/after; keep Kotlin
locking enabled in both runs. Resource ownership and input sizes must match.
Record process/warmup/sample settings with the results, and avoid interpreting
failure-path timings as successful-call performance. This document contains no
performance measurement or promise of a speedup.
