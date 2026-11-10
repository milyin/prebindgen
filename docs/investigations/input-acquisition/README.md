# When does an input become a Rust value?

A proposal for the input-acquisition work in
[#898](https://github.com/milyin/prebindgen/pull/898), based on merged #903 at
`e78f4fd24ffaf85ccb342980f7dc33c377d39c34`.

**Current problem:** two individually supported input converters cannot always
be used in the same exported function. One converter may construct its Rust
argument early, while another constructs it later. The common call assembler
rejects the combination, including some combinations containing only borrows.
Its execution order and ownership accounting currently depend on the same
resource flags.

**Recommendation:** finish converting each parameter in source order, have the
code that takes an incoming owner record that transfer, and describe the
conversion phases according to their actual effects. Keep the existing compact
ownership receipt and the existing collection decoding strategies.

This PR documents the problem and proposes a contract. It changes no generator,
wrapper or runtime implementation. The small [probe](probe/src/main.rs) reproduces
the current restriction; it is not a prototype of the proposed fix. The six
remaining implementation checkboxes in #898 stay open.

Read this overview first. [The proposed contract](contract.md) specifies the API,
execution order, compatibility effects and acceptance criteria.
[Evidence and reproduction](evidence.md) provides the executable controls and
links to the current implementation.

## 1. Two different Rust programs run at different times

Prebindgen generates code that will later execute. Both the generator and the
code it generates happen to be written in Rust. Keeping those two programs
separate is essential to understanding this problem.

| Time | Program running | Its work |
| --- | --- | --- |
| Binding build | A binding crate's `build.rs` calls prebindgen. | Read the source API, apply binding declarations and write Rust wrappers and C headers or Kotlin sources. |
| Application call | A C or Kotlin caller invokes a generated wrapper. | Translate the actual arguments, call the source Rust function, translate its result and clean up temporary values. |

The generator does not acquire a user's packet while it builds a binding.
It prepares Rust code that will acquire the packet when the application calls
the generated wrapper.

The library drives generation. The binding author selects declarations and
conversion rules; the author does not manually traverse every function and
write its wrapper. Authors can also supply custom prepared converters. Those
custom converters are an intentional extension point and must compose with the
ordinary declaration-generated converters.

## 2. A concrete source function and its ownership contract

Consider this source Rust API:

```rust,ignore
pub struct Item { pub value: i64 }

pub fn use_both(first: &Item, second: Item) {
    // Observe first and take ownership of second.
}
```

For this example, declare `Item` as an opaque pointer type in C. The caller has
two independent allocations. The pointers must not alias in a way that violates
the source signature.

| Source parameter | C representation in this example | Ownership during the call |
| --- | --- | --- |
| `first: &Item` | Pointer to an existing `Item`, exposed as `const`. | Rust borrows it. The C caller keeps the owner alive throughout the call. |
| `second: Item` | Pointer to an existing owning allocation. | Rust takes the value. After the transfer, C must not reuse or free that allocation. |

Kotlin represents opaque owners with generated handle objects; a native address
crosses JNI as an integer. Kotlin performs handle preflight and locking before
entering Rust. The source ownership distinction remains the same: a borrow
keeps its owner, while a move makes the incoming handle unusable after transfer.
This proposal preserves that Kotlin preflight and C's trusted-caller policy.

Taking `second` does not require allocating another `Item` or cloning it.
For a boxed pointer, Rust reconstructs the existing owning `Box` from its address
and moves the value out. Rust then manages the value's destruction. Constructing
a new Rust value from scalar fields is a different operation: it creates no
incoming foreign-owner transfer to report.

## 3. How prebindgen gets from the source to a callable wrapper

The following are build-time steps:

```text
annotated source crate
    -> captured source items
    -> Flat: source API plus binding metadata and converter slots
    -> language builder + declaration compiler + language codec
    -> prepared Flat with a converter in each exported parameter/result slot
    -> language emitter + common call assembler
    -> generated Rust wrapper and C/Kotlin surface
```

A **slot** is the place on a parameter or result that stores its prepared
converter. Two parameters of the same source type can have different converters.
`Schema` chooses the Rust types of those stored converters and their metadata;
it is not a runtime packet layout.

A **recipe** describes a conversion: use a leaf, call a constructor, expand
record fields, decode a collection, and so on. The **declaration compiler**
walks that recipe. A language **codec** supplies the physical values and Rust
conversion fragments for the leaves and language-specific operations.
The compiler assembles those fragments into a prepared converter and stores it
in Flat. Recipes are preparation inputs, not a second runtime model.

A **wire value** is a value crossing the boundary, such as an integer, a native
pointer, a Java reference or a bulk buffer. One source argument can require
several wire values. A converter's **wire tree** describes their grouping,
including repeated collection rows; the tree is a description, not an array of
runtime objects.

A prepared `InputPlan` stores the wire tree, Rust code fragments and resource
metadata. Independent converters can implement the emission contracts without
using that concrete plan. The **call assembler** requests fragments from the
converters and places them around the source function call. C and JNI both use
this shared assembler for ordinary function calls.

The result is compiled into the binding library. Only then can a foreign caller
invoke it with real values.

## 4. What the current input phases actually do

`InputFragment` is a generation-time request enum. A request returns Rust code;
the enum is not interpreted on every native call.

| Current request / plan storage | Meaning in generated Rust today |
| --- | --- |
| `Validate` / `validation` | Check or decode wire values. For declaration-compiled inputs, also take owners, run constructors and construct the argument. |
| `Acquire` / `acquire` | Take owners still deferred by a low-level converter. Often empty for declaration-compiled inputs. |
| `Construct` / `statements` and `argument` | Bind the final Rust argument. A low-level converter may create the reference or move the owned value here; a compiled converter often retrieves a value built earlier. |
| `Acknowledge` / `acknowledgement` | Make the shared ownership-reporting channel ready. JNI checks its receipt array before permitting transfers. This operation does not itself mean that every input was consumed. |
| `Resources` / `resources` | Describe relevant native allocation classes/access modes and provide today's consumption-accounting metadata to the generator. |

The declaration compiler deliberately builds values inside `validation`.
For example, a collection decoder can read a row, validate its fields, construct
the final Rust element while row-local storage is alive, and append that element
to its destination. Moving everything into a later global phase would require
retaining extra row state. Earlier work removed such staging; this proposal
keeps that benefit.

There are therefore two current shapes:

```text
Low-level boxed leaf:
    Validate: decode the pointer and perform applicable encoding checks
    Acquire:  reconstruct an owning Box for a move; empty for a borrow
    Construct: move the value out, or create the Rust reference

Declaration-compiled input:
    Validate: decode + acquire + construct, retaining the value in scope
    Acquire:  usually empty
    Construct: retrieve the already constructed argument
```

For a borrowed low-level leaf, the actual `&Item` is created in `Construct`,
not `Acquire`. For C direct pointers, encoding validity is the caller's
responsibility; the diagram does not add native pointer or alias checks.

## 5. The restriction is observable through the supported C API

The probe first prepares the example through `Cbindgen`. It then replaces one
parameter slot with a low-level `CCodec::input_leaf` plan and completes that
plan's normal finalization. Source signature, ownership intent and C wire types
stay the same.

At the current baseline:

| First converter | Second converter | Generation |
| --- | --- | --- |
| Compiled borrow | Compiled move | Succeeds. |
| Low-level borrow | Compiled move | Rejected. |
| Compiled borrow | Low-level move | Rejected. |
| Low-level borrow | Low-level move | Succeeds. |
| Low-level borrow | Compiled borrow | Rejected, although neither input consumes an owner. |
| Low-level borrow | Compiled scalar | Succeeds; the scalar has no resource descriptor. |

The rejected cases report:

```text
cannot mix deferred resource conversions with eagerly constructed inputs
```

Ordinary functions with both borrowed and owned parameters already work through
the usual builder. The failure concerns combining converter implementations,
not the Rust signature itself. It happens during generation, before a foreign
application can call the wrapper.

JNI has another relevant restriction: its preflight verifier currently requires
`Resource::registered` on native resources. A raw JNI leaf can be rejected there
before the common assembler is reached. Fixing only the assembler would not
complete the JNI customization path.

## 6. Why ownership reporting constrains the order

When conversion fails after taking some incoming owners, Rust may already have
dropped them. The foreign caller must learn which owners it must no longer
reuse or destroy. An **ownership receipt** supplies that information.

The current compact receipt is a count of consumed owned wire values in the
established argument/row/field traversal order, with inactive alternatives and
borrowed values skipped. It counts incoming owners, not source parameters or
all objects created in Rust. A collection of 1,000 incoming handles can contribute
1,000 transfers; a constructor consuming two handles contributes two even if it
produces one Rust object.

For incoming owners `[A, B, C]`, a count of two means **A and B** transferred.
A count alone cannot mean **B and C**. C recoverable `Result` wrappers write the
count through their ownership output. Kotlin receives an encoded count and
marks the corresponding handle objects consumed after the native call.
Callback-result decoders also use a consumed prefix to distinguish Rust-owned
values from unread callback-owned values that still need reclamation. Receipt
order is independent of the type/address sorting used to acquire Kotlin locks;
that lock ordering stays unchanged.

Now consider two owning inputs, A converted late and B converted early. The
current assembler places all `Validate` fragments before all `Acquire` fragments.
If the early conversion takes B and a constructor then panics, A has not yet
transferred. A count of one would wrongly describe A. This is a hypothetical
execution if the current rejection were removed unconditionally; the generator
currently prevents it.

The guard exists for a real reason, but it rejects too much. A deferred shared
borrow contributes no incoming owner to that count. The same guard also rejects
the all-borrowing combination in the table above.

## 7. Why the flags are an architectural problem

`Resource::registered` currently says a resource was acquired during preparation
and that preparation accounts for its consumption. The declaration compiler also
sets it on shared borrows. The assembler tests whether **any** resource has that
flag to decide the execution/receipt path, then rejects a mixture of flag values.

The flag does not describe the complete behavior of a converter: a scalar or
resource-free constructor can execute early without any resource descriptor.
`records_consumption` adds another distinction: some acquisition code increments
the counter itself, while other acquisition code depends on a parent or the
assembler to insert the increment.

This spreads responsibility across the leaf helper, declaration compiler,
collection compiler, C row specialization and call assembler. The JNI preflight
verifier also reads `registered` for a different purpose. An author must know
which layer will eventually account for a move, not merely how to decode and
construct the requested argument.

The proposed rework gives each responsibility one place:

- Input converters implement defined preparation, acquisition and construction
  operations, composed to completion in parameter order.
- The code taking a foreign owner records that successful transfer.
- Resource metadata describes foreign access requirements for preflight.
- The assembler coordinates calls and receipt readiness without inferring
  execution timing from resource metadata.

## 8. Proposed behavior and practical limits

For each source parameter, run its preparation, remaining acquisition and final
construction before starting the next parameter. Preserve each converter's
scope around subsequent work so borrowed temporary storage remains alive.
Only call the source function after every argument has been constructed.

Acquisitions follow the receipt's owned-wire order within each parameter,
including nested fields and collection rows. Record transfers before subsequent
user code, fallible work or destruction can fail. This keeps a prefix count
truthful without an owner bitmap or per-handle journal.

A later failure may consume earlier inputs. Rust drops acquired values and
partially built collections through ordinary scope cleanup. Source constructors
and destructors may change state. The contract requires correct destruction and
ownership reporting, not restoration of the caller's original state.

This changes the ordering seen by old low-level converters: an earlier input may
now transfer before a later input fails validation. Declaration-compiled inputs
already permit that behavior. This is a deliberate proposed contract change,
and low-level converter authors need a migration and ordering tests.

Bulk and ordinary collection decoders retain their existing row construction,
resource grouping, caching and failure cleanup. C still trusts callers. Kotlin
still performs preflight and locks before native entry. Output conversions,
typed domain-error delivery and binding-error handlers keep their existing
contracts.

No acceleration has been measured for this proposal. The implementation must
check that built-in hot paths keep their allocation counts, JNI call counts and
throughput. There is a specific risk for low-level batches whose old automatic
receipt update used one aggregate count: precise per-take updates could cost
more. [The acceptance plan](contract.md#acceptance-and-measurement) includes that
case rather than assuming the refactoring is faster.
