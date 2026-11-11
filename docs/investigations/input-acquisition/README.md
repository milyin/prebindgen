# Make the conversion API describe the work it executes

Design proposal for [#898](https://github.com/milyin/prebindgen/pull/898), based
on merged #903 at `e78f4fd24ffaf85ccb342980f7dc33c377d39c34`.

**The architectural problem is an outdated execution model.** The public API
separates validation, acquisition and construction, while the declaration compiler
already emits argument-construction code into the phase called validation. That behavior
keeps temporary data alive and permits efficient collection conversion. The
contract should describe it directly.

The rejection of mixed converter implementations is evidence of the mismatch.
It is not specific to `&Item` and `Item`, or even to related types. The common
assembler chooses a schedule from resource-accounting flags instead of asking
for the actual operation: convert these incoming values into a Rust argument.

**Recommendation:** represent complete, scoped input conversion; keep transport
reading, value conversion and output delivery as their actual operations.
Use the existing wire tree as the authoritative description of types, access and
repetition. Remove the duplicate `Resource` inventory and global input-stage
requests. Reuse runtime ownership reporting where the boundary needs it.

This PR contains a proposal and a runnable investigation fixture. It changes no
generator or runtime implementation and completes no implementation checkbox.
Read this overview first, then the [API contract](contract.md). The
[evidence](evidence.md) distinguishes observed behavior from proposed behavior.

## 1. Generation and application execution are different programs

Prebindgen generates Rust code that will execute later. The generator and its
output both use Rust, but they run at different times:

| Time | Who runs | Work |
| --- | --- | --- |
| Binding build | The binding crate's `build.rs` calls prebindgen. | Read the annotated source API and binding declarations; emit Rust wrappers and C headers or Kotlin sources. |
| Application call | C or Kotlin invokes the generated binding. | Decode actual arguments, obtain Rust values, invoke the source function, deliver outputs and clean up. |

The binding author selects declarations and conversion rules. The library
traverses them and drives generation. A custom converter can implement the same
emission contract without using the library's concrete conversion-plan storage.

A **wire value** crosses the foreign boundary: for example, a primitive, pointer,
Java reference or bulk frame. A **wire tree** groups those definitions into
records, alternatives and repeated rows. It describes a representation; it is
not a list of runtime objects or a second execution schedule.

**Flat** stores the source API, binding metadata and prepared converters. A
**slot** holds the converter for one parameter or return occurrence. `Schema`
chooses the Rust types stored in those slots and metadata fields; it is not a
runtime layout. A **recipe** describes how to construct or project a value.
The declaration compiler walks the recipe and asks a language **codec** to
supply conversions and physical representations.

```text
annotated source -> captured items -> Flat
    -> binding declarations + declaration compiler + language codec
    -> prepared converters stored in Flat
    -> language emitter + common call assembler
    -> generated Rust wrapper and C/Kotlin surface
```

The common assembler combines emitted conversion code with the source function
call. The language emitter owns the foreign ABI, Kotlin preflight and receipt
delivery. Neither requires the application user to write the traversal.

## 2. The actual runtime pipelines

### Ordinary arguments

The foreign entry point already receives wire arguments. Some leaf conversions
read those arguments directly; others decode text, bytes or native handles.
Bulk transport may open one frame and expose its fields. Reading can remain
lazy or grouped where useful; there is no requirement to decode every argument
before constructing any Rust value.

```text
enter the C/JNI wrapper with wire arguments
    -> establish boundary-specific receipt handling, if needed
    -> convert an argument, retaining its supporting storage
        -> convert the next argument
            -> call the source Rust function
            -> prepare outputs -> deliver -> commit output ownership
```

A conversion can check its wire values, take incoming owners and call a source
constructor. Applicable checks precede use. A later failure may occur after an
earlier input has transferred. Already constructed Rust owners are then cleaned
up through ordinary Rust scope destruction.

### Collections

A collection adds an actual transport lifetime: the current row's decoded
fields and JNI local references may be valid only until that row is released.

```text
open the ordinary or bulk carrier once
    -> bind the next row's incoming fields
    -> convert those fields and construct the Rust element
    -> place the element in its destination
    -> release row-local transport state
    -> continue with the next row
```

C, ordinary JNI rows and bulk rows have different reading code. They can still
use one value-conversion operation. Keep grouped field reads, cached bulk reads
and construction directly into final storage. A borrowed view backed by the
whole carrier retains that carrier for its required lifetime instead of being
forced through an owning-element reconstruction loop.

### Values returned by foreign callbacks

A callback result enters Rust through input conversion, but its cleanup
responsibility differs from ordinary incoming arguments:

```text
invoke the foreign callback and receive its result carrier
    -> protect incoming owners before a failing read can lose them
    -> read/bind result values, using existing caches where appropriate
    -> convert the returned value into Rust
    -> release carrier storage and finish the result
```

On failure, constructed Rust values are dropped and unread callback-owned values
are reclaimed by the existing guards or failure-only scans. Ordinary function
arguments leave unread owners with the foreign caller. Those cleanup policies
remain boundary responsibilities; they are not alternate decoder modes.

### Outputs keep their real delivery boundary

Output preparation, foreign delivery and ownership commit remain distinct.
Preparing a result does not prove the foreign recipient has accepted it. If
preparation or delivery fails, uncommitted owners must still be reclaimable.
This proposal changes input conversion, not that output contract.

## 3. One scoped value-conversion operation

The assembler needs this operation:

> Construct the Rust argument as `target`, then execute `next` while the argument
> and its supporting storage remain valid.

This is the proposed **Convert** operation. `next` is the remaining Rust code
inserted inside that scope during generation; it is not a runtime closure.
The same operation serves an ordinary parameter, a field/constructor input,
a collection element and a callback result. Their transport and cleanup scopes
are composed around it.

The built-in plan must store conversion code according to that contract.
Merely renaming `validation` to `preparation` while retaining global
`acquire/construction` phases would preserve the architectural mismatch.
A plan explicitly exposes backing-storage declarations so codecs and the
declaration compiler can keep owners outside optional/selector branches. That
storage performs no validation or acquisition. Code generation can still read
fields together and specialize row release. Those are actual
lifetime and transport requirements, not per-converter stage classifications.

For example, in generated Rust for a C opaque pointer:

```rust,ignore
let owner = unsafe { Box::from_raw(packet_pointer) }; // Existing allocation.
// Record the incoming ownership take here if this boundary needs a receipt.
let packet = *owner;
let packet = source_constructor(packet);             // May panic.
// The remaining call runs in the scope of any required supporting storage.
```

`Box::from_raw` does not allocate another object or clone the packet. A borrow
creates a Rust reference without taking its owner. A constructor building a new
Rust record from scalars creates no incoming foreign-owner transfer to report.

## 4. Describe access once; keep runtime reporting for its real purpose

The adapters' wire descriptions already identify native allocation types and
move/shared/exclusive access. C additionally describes inline owners and
ownership-transferring callback contexts. JNI handles describe allocation type
and access; retaining a Java callback reference is not a native-owner transfer.

The language emitter should derive preflight and receipt needs from that
existing representation. A separate `Resource` list repeats part of the same
information and also carries scheduling/accounting flags. The proposal removes
that list and its query; it does not replace them with an effects registry,
phase table or a record of checks performed during generation.

Kotlin's existing prepared handle sources have a real execution purpose:
they retain the objects used for preflight, locking and post-call invalidation.
Keep those sources and their correspondence to the wire tree. C continues to
trust its caller's pointer, lifetime and aliasing obligations.

The **consumed-prefix counter** is different from diagnostic metadata. Existing
recoverable C calls, consuming JNI calls and callback-result cleanup use it at
runtime to know which incoming owners Rust has taken. Preserve those protocols.
For owners `[A, B, C]`, a prefix of two means A and B, not B and C. Borrows and
locally constructed objects do not advance it.

Record actual successful takes in the established ownership traversal order.
Keep ordinary function and callback-result counters separate. Do not generate
tracking for an otherwise unobserved C call solely to help a diagnostic; the
[contract](contract.md#runtime-ownership-reporting) specifies where reporting is
needed and how a no-report boundary emits no counter updates.

## 5. Why the current stage model causes the observed rejection

Today a low-level boxed leaf supplies separate pointer validation, ownership
acquisition and final argument construction. The declaration compiler combines
those pieces into its `validation` code so the constructed value stays alive.
For a collection, this can be necessary to finish the current row efficiently.

The assembler still composes all `Validate` requests, then all `Acquire`
requests, then all `Construct` requests. It infers early construction from
`Resource::registered`, which is also set on shared borrows. Scalar conversion
can construct early without having a resource descriptor at all.

| Example source parameters | Converter combination | Current generation |
| --- | --- | --- |
| `&Item, Item` | Both normally declaration-compiled | Succeeds. |
| `&Item, Item` | Low-level borrow plus compiled move | Rejected. |
| `&Item, Item` | Compiled borrow plus low-level move | Rejected. |
| `&Item, Item` | Both low-level | Succeeds. |
| `&Item, &Item` | Mixed preparation styles | Rejected despite no ownership moves. |
| `&Item, i64` | Low-level borrow plus compiled scalar | Succeeds despite early scalar construction. |
| `&Item, Packet` | Both normally declaration-compiled | Succeeds. |
| `&Item, Packet` | Low-level borrow plus compiled move | Rejected despite unrelated pointee types. |

The rejection is:

```text
cannot mix deferred resource conversions with eagerly constructed inputs
```

The [eight-case probe](probe/src/main.rs) verifies those generation outcomes.
This is unrelated to aliasing the same object. The examples assume independent
allocations; Kotlin preflight and C's caller obligations still apply.

Removing just the conditional is insufficient. With two owning inputs, a late
conversion of A and early conversion of B can produce a non-prefix transfer
order. Without the guard, the current assembler would also omit A's automatic count
in a mixed path.
Those are reasons to establish a consistent conversion/reporting contract, not
reasons to restore global validation or add another tracking structure.

## 6. Design and implementation order

First establish the transport, conversion, lifetime and delivery operations.
Migrate the existing built-ins and independent converters to that API, including
callback cached reads and row cleanup. Remove the obsolete public stages,
phased plan storage, `Resources` and converter-provided `Acknowledge` requests.
The emitter initializes its own boundary receipt instead.

Then verify mixed converters through the same contract, using both related and
unrelated types, shared-only inputs and owning inputs with partial failures.
The reproduction is an acceptance test for the architecture, not its organizing
principle. Do not consider the work complete after removing one rejection.

The normal Cbindgen/JniGen declaration syntax and explicit copy choices stay
unchanged. Error paths may consume earlier inputs before a later check fails;
correct destruction and truthful receipts are required, not state restoration.
No performance improvement has been measured. The
[acceptance plan](contract.md#acceptance-and-measurement) checks hot paths,
optional/temporary scopes, carrier caching and code size before the new contract
can be considered implemented.
