# Proposed input execution and ownership contract

This is a design for review, not implemented API. Read the
[pipeline overview](README.md) before this document. Names in the sketches refer
to existing prebindgen types unless identified as a rename.

## Meaning of the operations

Keep the existing generation interfaces: `Flat` stores converters, `Conversion`
accepts generator-defined requests, and C/JNI opt into the shared staged protocol.
Other adapters remain free to use their own protocols. The recently simplified
`Codec` declaration interface needs no new required method.

The recommendation is one scoped **`Convert` operation per parameter**. The
assembler needs the completed Rust argument, with supporting storage kept alive;
it does not need to interleave the converter's internal phases with other
parameters. This gives the public operation a concrete purpose.

| Proposed operation | Contract |
| --- | --- |
| `Convert { target, next }` | Decode/check wire values, acquire owners and construct the argument as `target`. Run `next` with that argument and its permitted supporting storage alive. Each applicable check precedes that wire value's use; every ownership take is recorded. |
| `Acknowledge` | Return the ordinary function call's once-per-call receipt-readiness code, or `()`. Run it before conversion can transfer an owner. Preserve detection of conflicting readiness code. |
| `Resources` | Describe native allocation classes and access modes. These determine preflight requirements and whether ownership tracking/receipt delivery is needed, not the converter's execution schedule. |

`Convert` replaces `Validate`, `Acquire` and `Construct`. It returns one Rust
expression containing its continuation, as the current protocol already does.
It may fail after consuming some inputs. It does not promise to perform every
check in the complete call before any owner can move.

```rust,ignore
pub enum InputFragment<'a, C> {
    Convert {
        context: &'a C,
        target: &'a Ident,
        next: TokenStream,
    },
    Acknowledge { context: &'a C },
    Resources { context: &'a C, resources: &'a mut Vec<Resource> },
}
```

`InputPlan` may retain separate code buffers for recipe composition and cached
reads: `preparation` (renamed `validation`), `acquire`, `statements` and `argument`.
Its `Convert` implementation composes those buffers once and binds `target`.
These buffers support existing row specialization; they are not a second public
execution protocol. Independent converters need not construct an `InputPlan`.

For JNI callback results, the existing `callback_return_fragment` hook receives
`Convert` once. A plan-based override substitutes cached reads in its preparation
buffer, then emits the complete conversion. Preserve the current rejection of a
custom converter that cannot honor a required cache; do not silently decode a
carrier again. This removes the need to retain three public requests merely to
expose the built-in plan's buffers.

## Complete parameters in source order

The generated wrapper follows this order:

```text
establish local ownership tracking when an incoming owner may transfer
make the ordinary call's receipt channel ready, when needed
Convert(first)
    Convert(second)
        ...
            call the source function
            prepare outputs -> deliver outputs -> commit output ownership
```

For a compiled argument, most existing work stays in its preparation buffer.
For a low-level borrowed leaf, conversion decodes the pointer and creates the
reference. Both provide the same complete-argument operation.

The assembler requests nested fragments in reverse while generating code so the
wrapper executes in forward parameter order. This is a build-time traversal;
it introduces no runtime loop over parameter descriptors. Preserve tail-block
flattening for wide generated functions.

The old schedule generally ran all `Validate` fragments, then all `Acquire`
fragments, then all `Construct` fragments. Whether receipt readiness ran before
or after validation depended on `registered`. Replace those branches with one
schedule; preserve output preparation, delivery and commit ordering.

Acquired Rust owners must remain in scope or move into another Rust owner.
A failure unwinds those scopes. A reference backed by a temporary, such as an
explicitly reconstructed input `String`, retains that temporary through the call.

Completing one argument before the next extends some lifetimes relative to the
old protocol. In particular, a JNI converter must not hold a critical-array
region, or another resource that forbids subsequent JNI calls, across `next`.
Keep only storage compatible with later conversions, callbacks and the source
call. Release temporary JNI access before continuing when the representation
allows it. A representation requiring a call-wide JNI prohibition needs an
explicit supported restriction or a generation error; do not add a fallback
copy merely to make it fit this contract. Arbitrary custom code must uphold this
scope obligation; the generator cannot infer it from tokens.

## Establish and address the ownership counter

A `Move` resource descriptor means the conversion may take an incoming foreign
owner, even when an optional value is absent on a particular call. Every counted
take must declare such a resource, including a C callback context whose ownership
transfers. Retaining a JVM callback object through a shared global reference is
not by itself an incoming native-owner transfer.

For a boundary that may take owners, the adapter establishes a local
`Cell<usize>` initialized to zero before conversion and supplies its reference
through the existing `TemplateContext` parameter **`input_consumed`**. C and JNI
already create the underlying cells in their wrappers. A custom adapter using
this protocol must provide the same binding; unsupported context rendering is
a generation error. An ordinary call and each callback-result invocation have
separate counters. The counter is not shared between threads or invocations.

The assembler uses the presence of a `Move` descriptor to require this binding
and arrange readiness. It does not use `registered`, a runtime input count or
the converter's choice of code buffers. The counter exists before any `Convert`
fragment can execute.

Provide one generation helper for ownership-taking code:

```rust,ignore
pub fn record_input_transfer(count: TokenStream) -> TokenStream;
```

It emits an increment of the cell referenced by `template_parameter("input_consumed")`.
`count` is normally `quote!(1usize)` and must describe actual completed takes.
This is generated local arithmetic, not a runtime helper call or JNI call.
The helper standardizes how boxed, inline-owner, callback and custom acquisitions
report transfers without depending on a private generated identifier.

The counter and its delivery have different responsibilities:

| Boundary | Local counter | Delivery |
| --- | --- | --- |
| Recoverable C `Result` call with possible moves | Required. | Existing ownership output receives the count. |
| C call without a `Result` | Required by the conversion contract when moves are possible. | Preserve the existing success/abort policy and ABI; no new ownership output. Unobserved counting may optimize away. |
| Ordinary consuming JNI call | Required. | Existing receipt array and Kotlin ownership updates. Validate readiness before any take. |
| Foreign callback result entering Rust | Required when moves are possible. | Local cleanup guards use the prefix to avoid reclaiming Rust-owned values; no ordinary function-argument receipt array. |
| Collection rows | Reuse their enclosing call/invocation counter. | No per-row receipt object. |
| Shared-only or resource-free boundary | Not required by this protocol. | No ownership receipt introduced. |

Today compiled shared-only inputs can select the assembler's eager counter setup
because their resources are `registered`. Removing that unnecessary internal
setup is a proposed simplification, not a claim about current code. Preserve the
foreign ABI and measure the generated result.

## Ownership takes report themselves

The canonical receipt remains a consumed prefix. Its authority is the adapter's
existing traversal of **active owned wire values**, not a lock/type bucket.
For the built-in converters this is parameter order and row/field order within
each parameter. Keep Rust acquisition, Kotlin consumption and callback cleanup
in agreement with that traversal. Shared/exclusive borrows and values
created locally without taking a foreign owner do not advance the prefix.

The helper that crosses an incoming ownership boundary must emit the matching
counter update. For example, this is generated Rust inside a wrapper, shown
schematically with error handling omitted:

```rust,ignore
let owner = unsafe { Box::from_raw(packet_pointer) }; // Existing allocation.
// Emit record_input_transfer(quote!(1usize)) here: ownership now is Rust's.
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

The [source inventory](evidence.md#source-inventory) identifies Kotlin's
`KotlinInput.consumed` statements and the carriers' ordered owner lists, as well
as the independent type/address grouping used for locks. Add an interleaved
two-handle-type case; the shared prefix must follow owner order across types.

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

Kotlin's `require_preflight` must be updated in the same migration. Its
`registered` gate currently does important additional work for callback results:
their decoder invokes conversion fragments directly, without the assembler's
automatic counting. Accepting an uncounted low-level take there can make unread
owner cleanup reclaim a value Rust already took, causing duplicate destruction.
For existing built-ins, registration stands in for accounting already being
present; it is not merely an unnecessary scheduling restriction.

Migrate and verify **every** ownership-taking helper, including low-level JNI
leaves used in callback results, before removing that gate. Replace its implied
accounting guarantee with the universal per-take reporting contract. The flag
alone is not a proof of live handles or correct aliasing. Keep the real requirements: native resource allocation types and
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
  Its decoder must use the same complete-conversion and per-take accounting contract, with a
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
counting; receipt delivery remains specific to each boundary.

JNI callback decoders currently request the three phases directly and sometimes
substitute cached bulk reads. Migrate those sites to the complete `Convert`
request alongside the assembler.
Moving cleanup guards after a potentially failing decode would leak callback
owners; keeping the guard/decoder ordering is an explicit acceptance criterion.

## Compatibility and migration

This is a breaking change for authors using the low-level staged protocol:
`Validate`/`Acquire`/`Construct` become `Convert`, the plan's `validation` buffer
becomes `preparation`, resource accounting fields are removed, and receipt updates
become an acquisition helper's responsibility. The ordinary
Cbindgen/JniGen declaration syntax is unchanged.

Low-level conversion timing also changes: an earlier owner may transfer before
a later input is validated. A caller may therefore observe more consumed inputs
on failure. This is permitted by the chosen failure policy, but must be stated
in the migration guide and exercised in tests. Receipt-readiness errors may also
precede wire-validation errors on a formerly deferred path. Do not claim global
prevalidation or transactional restoration.

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
| Keep three public phase requests, always invoked together | Retains implementation details in the assembler contract. Row specialization can keep its code buffers, and callback caching can specialize the complete `Convert` operation instead. |

## Acceptance and measurement

The implementation should satisfy these independently reviewable conditions:

- The six probe cases generate successfully under the new contract, with actual
  execution coverage for the mixed borrow/move cases and all-borrow cases.
- Two independently allocated owned inputs compose in either old converter style.
  Inject failure before acquisition, between takes, during construction and after
  source entry. The receipt identifies the transferred prefix exactly.
- Optional/selector inactivity skips both acquisition and counting. Constructors
  consuming multiple owners, nested collections and partially built rows clean up
  without a leak or duplicate destruction. Shared temporaries stay alive. An
  interleaved two-type carrier preserves the adapter's original owner order
  regardless of Kotlin lock grouping.
- Rust panics unwind through the supported boundary with the same ownership
  reporting obligations as ordinary errors. Abort configurations remain aborts.
- A callback result using a low-level JNI move leaf fails after taking an owner:
  the counter advances, Rust destroys the acquired value once, and unread-owner
  cleanup never takes it again. Extend to a remaining suffix and constructor
  panic. Retain worker callbacks, cached bulk reads and nested row coverage.
- A low-level JNI array input followed by a parameter doing JNI decoding passes
  `-Xcheck:jni`; temporary access regions end at the required point. A known
  representation requiring incompatible critical-region lifetimes is rejected
  rather than silently copied.
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
