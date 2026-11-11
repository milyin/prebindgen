# Proposed API for the actual conversion pipelines

Design for review, not implemented API. Read the [overview](README.md) first.
The proposal uses the existing Flat/converter model and concrete C/JNI wire
representations. It adds no registry for execution stages, effects or diagnostics.

## Responsibilities and authoritative information

| Owner | Responsibility and existing information |
| --- | --- |
| Flat and its parameter/result slots | Source API, binding declarations and the prepared converter for each occurrence. |
| Language emitter | Foreign ABI, wire transport, preflight/locking, boundary error policy and receipt delivery. It reads access information from its existing wire kinds. |
| Input converter | Convert available wire values into the requested Rust value, preserving supporting lifetimes and reporting ownership takes when required by the boundary. |
| Collection transport/composition | Open a carrier, expose rows, combine field conversions, retain/release row storage and initialize final Rust elements. |
| Callback-result boundary | Obtain the returned carrier, protect unread owners, supply cached/native reads and reclaim the unread suffix on failure. |
| Output converter/emitter | Prepare results, deliver them and commit output ownership according to the existing protocol. |

A wire tree describes transport, access and repetition, not when all arguments
must be validated. A converter's code describes execution. Do not infer that
execution from an auxiliary resource record.

## One input-conversion request

Replace `Validate`, `Acquire`, `Construct`, `Resources` and `Acknowledge` with
one scoped operation. The existing `InputFragment` request type can become a
struct rather than an enum with one variant:

```rust,ignore
pub struct InputFragment<'a, C> {
    pub context: &'a C,
    pub target: &'a Ident,
    pub next: TokenStream,
}
```

These are proposed fields, not a second interface alongside the old requests.
`Conversion::code_fragment_slot` and its caller-supplied expected type remain.
Input converters use this request type; the generic Flat `Conversion` contract
continues to permit other adapters' request types. The declaration `Codec`
interface retains its four required operations.

The returned Rust expression must:

1. Read/validate values as needed, construct the value as `target`, and perform
   any necessary ownership takes in the agreed traversal order.
2. Evaluate `next` with `target` and its required backing storage valid.
3. Destroy untransferred Rust temporaries on exit or unwind.

Insert `next` exactly once, in tail position of an unlabelled block, outside any
new closure, loop or labelled block. Preserve its existing `?`/`return` meaning
and tail-block flattening. A row boundary can already supply a row-scoped result
expression; the converter must not add another closure around its continuation.

The operation may fail after earlier takes. It does not promise whole-call
validation before acquisition. Applicable checks precede use according to the
adapter's policy: C's trusted-pointer contract does not gain native alias or
pointer checks.

The assembler generates nested requests in reverse parameter order so emitted
Rust executes complete conversions in source order. Non-consuming transport
reads can still be grouped where useful. Inside a value/row conversion, retain
existing grouped operations when their lifetime and failure constraints permit.
The ownership prefix constrains the order of takes, not a requirement to use
immediately every wire field just read.

This composition is performed during generation. `next` is inserted Rust code,
not a runtime callback or another closure per row. Preserve tail-block
flattening for wide signatures.

## Built-in plan storage follows the same operation

Do not implement `Convert` merely by wrapping the public
`validation/acquire/statements/argument` plan and leaving its old compiler
scheduling intact. Replace the stage representation with scoped conversion code.
Schematically, the plan retains:

```rust,ignore
pub struct InputPlan<W, C, N, B> {
    pub node: WireNode<W, N, B>,
    pub storage: TokenStream, // Enclosing backing-storage declarations, when needed.
    pub code: TokenStream, // One scoped conversion with target/continuation markers.
    pub fallible: bool,
    // Existing generic context marker as needed for the converter's Rust type.
}
```

The sketch omits ordinary type bounds. The code is a Rust template using the
binding/continuation mechanism, not a new interpreted runtime plan. `storage`
contains only backing-storage declarations and inert initializers such as
`None`. It performs no wire reads, source constructors, fallible work or ownership
takes. The conversion initializes that storage at the actual active use.
Most leaves need no separate storage.

This field is public because codecs produce leaves in another crate and the
declaration compiler composes them. For an optional/selector/record, the compiler
can move child storage declarations into the enclosing value's scope while
keeping the child's execution conditional. The storage is actual emitted Rust
needed to keep a borrow valid; it is not a stage, effects inventory or diagnostic
record. A built-in plan's complete request emits its storage and scoped code.
Independent converters may instead emit that whole expression themselves.

### Template composition and rendering order

Bind the requested target in the converter's own template, render that local
template with its emission context, then insert the already-rendered `next`.
During local rendering the reserved continuation marker remains intact. Only
after other markers are resolved does composition replace that marker and flatten
the tail block. The current `plug` rejects other markers, so passing a partly
rendered template to it is invalid. Use this defined pass order for both stored
plans and independent implementations.

Do not re-render the growing remaining call once per parameter. Each producer
renders its own code; the assembler composes the rendered expressions. In
particular, a nested callback-result decoder renders its own reporting before
its code becomes part of an outer template. No unresolved transfer marker may
cross that boundary. This avoids a new repeated whole-body rendering pass;
code-generation time and syntax depth still need measurement.

The recipe compiler must compose this code according to actual lifetime scopes.
For example, an explicitly reconstructed temporary backing `Option<&str>` needs
its owner outside the optional reference-producing expression. Record fields
containing borrows need their backing storage to survive through the enclosing
source call. Hoist the required Rust local storage when building the code; do
not let a helper return a reference to storage dropped at the end of its branch.

The compiler can share private code-building helpers for conditional construction,
row bindings and release. Cross-crate storage information uses the explicit field
above. These distinctions produce real Rust code and lifetimes; they are not
global validation/acquisition stages or a model maintained only for diagnostics.
Require a prototype
to demonstrate correct optional/selector/temporary composition, linear generated
code growth and no additional per-row closures or allocations before accepting
the storage rewrite. Preserve existing immediately invoked row expressions.
Do not duplicate the remaining call in every branch to hide a lifetime problem.

## Transport and value conversion compose at actual scopes

There is no requirement for one universal transport interface or a new transport
registry. Reuse the current operations where they represent the work:

- Native function parameters supply wire bindings at entry. Text/byte leaves
  can perform their existing semantic decoding in the value conversion.
- The sequence transport's open/bind/release code exposes row fields and bounds
  their lifetime. Row conversion constructs the Rust element while those fields
  are available, then stores it in final initialized storage.
- Bulk frame acquisition and cached reads remain carrier operations. Read a
  frame once where the existing path does, including nested ordinary/bulk rows.
- A borrowed collection view can retain its carrier for the source call rather
  than being reconstructed into an owning collection.

Finish row-local cleanup before propagating an ordinary row-conversion error.
A successful element may outlive its row's JNI references only if its declared
Rust representation permits that. Preserve the current supported-type limits;
do not introduce copying to make an unsupported lifetime appear to work.

### Callback results use the same conversion operation

The callback-result boundary first establishes the protections needed for the
returned representation. Some ownership-bearing fields must be read to install
a guard; failures during that reading must preserve the existing reclamation
behavior. Nonessential fallible decoding remains after protection is established.
Keep the current guards or failure-only scans, selected by the actual carrier.

Then request the complete conversion once. The current JNI
`callback_return_fragment` specialization can remain as a cache-aware emission
entry point, but it accepts the same complete request. It selects supplied
native/cached reads while emitting that operation; it does not replace a hidden
validation stage. Independent converters unable to use a required cached carrier
remain unsupported rather than silently decoding it twice. The default method
must return an error when the boundary supplies a required cache; an override
must use the supplied cached/native reads. Keep that existing rejection on the
complete request, with no stage test or new capability inventory.

Callback unread owners belong to the binding's cleanup scope. Ordinary unread
function inputs remain with the foreign caller. Conversion does not merge those
ownership responsibilities or add an eager owner journal on the successful path.
Rust-to-foreign callback arguments continue to use the output protocol.

## Derive access and receipt needs from the existing wire tree

Remove the `Resources` query and `InputPlan.resources` rather than replacing the
current `Resource` with another parallel list. Remove `Resource`, including its
registration/counting flags, once its consumers have migrated. Keep `Access`:
wire descriptions and Kotlin handle sources still use its shared/exclusive/move
concepts independently of the removed inventory.

The information already has a home:

| Existing input wire description | Information used by its adapter |
| --- | --- |
| JNI `Handle` | Allocation type, move/shared/exclusive access and active condition. |
| C `Pointer` | Pointee type and move/shared/exclusive access. |
| C `Owned` | Native inline owner type, access and by-value versus pointer representation. |
| C input `Callback` | Context ownership transfer and its destructor contract. |
| Optional/selector/sequence nodes | Selection and repetition of the contained values. |
| Ordinary primitives, text or byte carriers | Their conversion semantics; constructing a Rust-owned buffer does not itself transfer a foreign native owner. |

Use adapter-private traversal/views over that tree; do not make the generic
`WireKind` trait interpret C/JNI ownership. The common call assembler needs the
completed arguments, not an allocation-class inventory. Source type equality
plays no role in deciding when conversion happens.

Kotlin still needs its prepared handle sources: they retain the actual objects
used for locking and post-call invalidation. Keep their existing correspondence
to the wire tree, type/access grouping and sorted locks. Remove the comparison
between `Resource` metadata and wire metadata. Existing checks against actual
Kotlin projection/handle sources can inspect those existing structures directly;
there is no new retained effects table or per-parameter diagnostic ledger.

The generated native and Kotlin code must implement the same access/ownership
contract. Arbitrary emitted Rust/Kotlin cannot be proved correct by comparing
another copied list. A custom converter must describe its inputs truthfully in
its wire representation and supply the corresponding projection and code.

`Condition` and `InputLeaf.active` still drive optional/selector conversion.
Place takes and any reporting inside the existing active branch. Rows still
execute in their actual loop. No replacement for `Resource.active/count` is
needed in another inventory: activation is in the operations, and the loop's
actual takes determine the count.

## Runtime ownership reporting

The consumed-prefix counter is existing application behavior, not build-time
tracking for diagnostics. Preserve the C/JNI receipt ABI and callback cleanup.
Its authoritative order is the existing owner traversal: parameter order,
active alternatives and row/field order within parameters. It is independent
of Kotlin's type/address ordering for locks.

A count of two for `[A, B, C]` means A and B transferred. The code taking an owner
reports the take before later user code, fallible work or destruction can unwind.
Borrowing and locally constructing a Rust value do not advance the prefix.
Keep transfers grouped only where all intervening operations are proven
infallible and cannot expose a partial state. The implementation must measure
that choice; no one-update-per-handle rule should defeat a valid grouping.

The language emitter initializes receipt handling before any such take, using
the input wire representation and the boundary's actual delivery policy:

| Boundary | Reporting |
| --- | --- |
| Recoverable consuming C `Result` | Existing local count and ownership output. |
| Consuming JNI call | Existing local count, validated receipt array and Kotlin consumption projection. |
| Foreign callback result with native owners | Existing invocation-local count and unread-owner cleanup. |
| C call whose failure aborts and which exposes no input receipt | No count solely for this proposal; emit no transfer updates for this boundary. |
| Shared-only or resource-free conversion | No incoming native-owner reporting. |
| Collection row | Use the enclosing boundary's reporting; no per-row receipt object. |

Remove converter-provided `Acknowledge` fragments. The adapter owns its receipt
storage and initializes/validates it once before conversion. C receipt outputs
remain a trusted-caller contract; JNI's existing receipt-array readiness check
belongs in the JNI wrapper. The common assembler does not merge setup fragments
or classify converter timing to choose when setup runs.

The supported C boundary cases are exhaustive: source `Result` wrappers recover
and require the existing ownership output whenever the input wires can transfer
an owner, including optional/empty carriers and callback contexts. Result wrappers
with no possible incoming owner need no receipt. Non-Result wrappers abort on
binding errors/panics and expose no recoverable partial-consumption path. The
[current audit](evidence.md#c-recovery-and-receipt-coverage) identifies these branches.
Thus the built-in ABI already covers partial consumption; global validation is
not the basis for omitting a receipt. A future/custom recoverable consuming
boundary must provide a receipt or fail generation. The migration must not
accept it on an assumed all-or-nothing guarantee.

Retain one code-generation operation for reporting a completed take, such as:

```rust,ignore
pub fn record_input_transfer(count: TokenStream) -> TokenStream;
```

This proposed helper emits a reserved `__prebindgen_input_transfer!(count)`
marker. `count` is a side-effect-free `usize` expression: no-report rendering
never evaluates it. Keep named binding markers separate from this operation.
Extend the existing public renderer explicitly, schematically:

```rust,ignore
pub fn render_template(
    code: &TokenStream,
    parameter: &impl Fn(&str) -> Result<TokenStream>,
    transfer: &impl Fn(&TokenStream) -> Result<TokenStream>,
) -> Result<TokenStream>;
```

The renderer recognizes the transfer marker, resolves any binding markers in its
count, and gives that expression to `transfer`. Ordinary groups are traversed
normally. `TemplateContext::render` keeps its signature, but input contexts must
implement the new operation and preserve the reserved continuation marker until
composition. This is an explicit source/semantic change to the public rendering
contract, including custom contexts, not just private marker syntax.

A reporting C/JNI context supplies the existing local count binding and emits
`Cell` arithmetic. A no-report context, including `()`, emits `()` and declares no
count for this purpose. This is a generation decision, with no runtime flag or
reliance on the optimizer. Unknown operations or an input context unable to
implement them fail generation; custom contexts must not leave them as Rust
macros or silently ignore them. The final generated Rust contains no markers.

Each producer renders only its own template before inserting the continuation.
For callback results, including the C callback closure assembled during binding
preparation, create an explicit callback-local reporting context/binding and
render the decoder there. That local scope exists independently of whether the
enclosing wrapper has a receipt. An outer no-report C context therefore sees
ordinary rendered Rust and cannot erase inner reporting. Do not rely solely on
lexical shadowing of the old counter name to distinguish rendering policies.

Require consistent ownership delivery from the existing wire and Kotlin
projection, including a consuming parameter next to a non-consuming one. Perform
checks at the operation that needs the information using those existing values;
do not create native and Kotlin ownership inventories just to compare them.
The generator cannot statically prove arbitrary custom code's traversal or
cleanup. That remains part of the converter contract.

### Migration must preserve callback accounting

The current JNI `registered` gate also prevents an uncounted low-level callback
return from reaching cleanup that assumes a truthful prefix. Remove that gate
only after every ownership-taking helper and callback decoder uses the new
contract. A raw move followed by failure must never be reclaimed again as unread.
This dependency is a migration obligation, not a replacement diagnostic mode.

This is an explicit safety obligation for every custom input converter: after
an actual ownership take, report it exactly once, before any later fallible work,
user code or destruction can unwind. Never report a take not yet performed.
Failure to comply can cause callback cleanup or caller invalidation to destroy
an owner twice. The generation method need not be Rust `unsafe`, but generated
code safety depends on this contract, just as it depends on truthful wire types.
Migrate all take sites before removing the old gate; comparing a new copied list
would not prove arbitrary fragments comply.

## Lifetime and scope restrictions

Supporting storage must remain valid through the continuation that uses it.
JNI critical-array regions, or other states forbidding subsequent JNI calls,
cannot cross a continuation that performs those calls. The current built-ins
return Rust-owned text/bytes or read bulk bytes into Rust-owned storage; the
[audit](evidence.md) identifies those paths and found no call-wide critical pin.

A last parameter is not automatically safe for a custom critical pin: source
execution, nested callbacks and output delivery also belong to its continuation.
An unsupported lifetime must remain unsupported or require an explicit supported
call restriction. Do not introduce an implicit copy or runtime safety tracker
as a workaround.

## Compatibility and implementation order

The first deliverable is the actual conversion/scope API and its representation.
Migrate leaves, recipe composition, rows and callback results to it before
claiming the mixed-converter rejection is fixed. The reproduction is one test
of composability, not the reason to retain an obsolete phase system.

This breaks low-level request matching and direct plan construction: the old
phase requests/fields, `Resources`, `Resource` and `Acknowledge` go away. Ordinary
Cbindgen/JniGen declaration syntax, explicit copy permissions, Kotlin preflight,
C's trusted-caller policy and output/error delivery stay intact.

An earlier input can now transfer before a later input fails validation, and
receipt readiness may fail before ordinary wire decoding. Those error-path
ordering changes must be documented. Partial consumption/state changes are
allowed; correct destruction and reporting are required, not rollback.

Migrate the entire existing implementation, including independent-converter
fixtures and probes, to one production contract. Remove obsolete supporting
code rather than retain parallel phase and conversion protocols. The
[source inventory](evidence.md#source-inventory) identifies the relevant sites.

## Acceptance and measurement

Before treating the design as implemented, verify:

- Ordinary, nested row, borrowed view and callback-result scopes use the complete
  conversion operation. Temporary owners survive every supported optional,
  selector, constructor and borrowed-field use. No reference escapes its storage.
- Many optional/selector fields generate code growing with the fields, without
  copying the whole remaining call into each branch or adding per-row closures
  or allocations beyond the existing row-scoped expressions.
- C/JNI input wire descriptions are the access authority. `Resources`, the
  `Resource` type, phased input plan storage and converter setup queries are gone.
  Kotlin's existing prepared sources still drive correct locking/consumption.
- Port all eight reproduction configurations to the new API. Generate and execute
  them, including unrelated `Item`/`Packet` types. Borrow/move cases transfer and
  destroy the moved owner once and retain the independent borrowed owner.
  Run matching configurations through recoverable C `Result` and consuming JNI
  boundaries as applicable, not only the probe's aborting C boundary. Check the
  receipt and each owner being counted exactly once. Shared-only cases do not
  create an input receipt.
- Two owning inputs, constructors taking several owners, optional/selector
  inactivity and interleaved handle types preserve the consumed prefix on failure
  before/between/after takes, during constructors and after source entry.
- Callback results drop constructed values and reclaim unread suffixes once.
  Exercise a low-level move followed by failure, worker callbacks, ordinary/bulk
  cached reads and a no-report outer C call containing a callback-result decoder.
- Row conversion preserves grouped reads and direct final construction. Native
  reading/decoding is not repeated to support another apparent stage.
- JNI calls after earlier input conversion pass `-Xcheck:jni`; guard lifetimes
  remain compatible with the supported source call and output path. Reject an
  incompatible representation instead of silently copying it.
- Typed domain errors and binding failures retain their existing channels. No
  C alias validation or duplicate native JNI alias checking is introduced.
- Update generated review artifacts and run Rust tests/Clippy, C/Kotlin/JNI/Java
  boundaries, sanitizers and cross-target checks. Static diagnostics are not a
  substitute for execution and cleanup tests.

Measure the successful path independently from error-path cleanup:

| Case | What to check |
| --- | --- |
| Scalar/shared-only calls | No added reporting, allocation or JNI calls. |
| Normal compiled owners and constructors | Preserve the current hot-path costs. |
| Small and large groups of low-level moves | Compare receipt update grouping, generated code and timings. |
| Ordinary/bulk arrays and borrowed views | Preserve caches, grouped reads, direct storage and explicit copy choices. |
| Callback results | Preserve existing successful-path behavior and failure-only cleanup. |
| C calls without a receipt | Verify generated code actually omits unneeded tracking and retains nested callback-local reporting. |

Use matching functions, inputs and settings before/after. Keep Kotlin locking
enabled in both runs, record warmup/sampling conditions and inspect allocation
and JNI-call counts as well as timing. This proposal contains no measured
speedup or promise that runtime behavior already changed.
