# Payload enums: one variant model for C and Kotlin

**Status: proposal for human review; no generator implementation in this PR.**
This is the payload-enum task from [#808](https://github.com/milyin/prebindgen/pull/808).
The runnable [baseline probe](probe/src/main.rs) demonstrates the present restrictions.
The [Zenoh usage audit](zenoh-usage.md) checks this plan against pinned source,
JNI declarations and both SDKs. It makes expanded reply delivery, nested optional
configuration and lists of instrumentation records concrete acceptance cases.

## What a caller should be able to do

A payload enum is a Rust enum whose variants can contain values:

```rust
pub enum Event {
    Idle,
    Count(u64),
    Packet { id: u64, bytes: Vec<u8> },
}

pub fn exchange(event: Event) -> Event { event }
pub fn publish(events: Vec<Event>, receive: impl Fn(Event)) { /* application */ }
```

The binding should represent the **active variant and its fields**. `Packet`
should carry its original bytes through the existing byte-vector conversion;
neither the enum nor its payload needs a new native heap object just because
the enum crossed the boundary. The same conversion must work for a function
argument, a record property, a collection element and a callback value.

Proposed declarations in the consumer's `build.rs`:

```rust,ignore
// Kotlin/JNI: proposed sealed_class! addition, existing package/function API.
.package(package!()
    .class(sealed_class!(Event))
    .fun(fun!(exchange))
    .fun(fun!(publish)))

// C: proposed tagged_union addition, following enum_type(name, type).
.tagged_union("event", syn::parse_quote!(Event))
.function(fun!(exchange))
.function(fun!(publish))
```

The Kotlin surface follows the existing production generator's convention:

```kotlin
sealed interface Event {
    data object Idle : Event
    data class Count(val v0: ULong) : Event
    data class Packet(val id: ULong, val bytes: ByteArray) : Event
}
```

Integer and byte-array mappings keep the existing JNI rules. Tuple fields use
`v0`, `v1`, etc.; named fields use the existing property naming rules. Variant
renaming uses `.variant_name("Packet", "Message")`, matching #808's
`enum_class!` naming override. Names do not determine variant tags.
Generate content-based byte-array equality/hash helpers as for current data
classes, rather than inheriting Kotlin's reference equality for array properties.
The source fields determine the Kotlin properties, even when a field needs no
wire storage: `Ok(())` becomes `data class Ok(val v0: Unit)`, constructed with
`Unit`. A genuinely fieldless variant, including `B()` or `C {}`, is a data
object. Never generate an illegal zero-property Kotlin data class.

The C surface is a tag plus a union of converted payloads, conceptually:

```c
typedef int32_t pb_event_tag;
#define PB_EVENT_IDLE   0
#define PB_EVENT_COUNT  1
#define PB_EVENT_PACKET 2
/* -1 is an empty carrier, usable by generated lifecycle operations only. */

typedef struct {
    pb_event_tag tag;
    union {
        struct { uint64_t v0; } count;
        struct { uint64_t id; pb_vec_u8 bytes; } packet;
    } payload;
} pb_event;

void pb_drop_event(pb_event *event);
```

`pb_vec_u8` above denotes the existing generated inline vector owner, including
its target-specific storage and accessors. This is an illustrative header, not
a proposed handwritten replacement for generated layout definitions. `Idle`
needs no payload storage. A scalar-only enum needs no destructor.

## What is missing today

| Current component | Existing capability | Missing operation |
| --- | --- | --- |
| Flat source model | Named records and fieldless enums | Variant payload fields, including tuple positions |
| Shared declarations | Record construction/projection, optional and sequence conversion | Match a source enum and construct its selected variant |
| Input selector | Choose among constructor functions for the same target type | Represent the target enum's actual variants |
| Kotlin adapter | Data classes, enum classes, handles and callbacks | Sealed values with converted payload properties |
| C adapter | Fieldless integer enums, inline owners and opaque pointers | Tagged carriers with active-payload ownership |
| Bulk layout | Target-measured named structures | Union payload layouts and tag-first decoding |

The existing input-expansion API already uses the word `variant`, for a choice
of constructor functions:

```rust,ignore
expand_param!(Event)
    .variant(fun!(event_from_bytes))
    .variant(fun!(event_from_text))
```

This configuration lets the foreign caller choose which function constructs an
Event. Internally, that choice is represented by `Input::Selector`. Both helpers
could return `Event::Packet`; a helper could also choose `Event::Idle` depending
on its arguments. Their signatures therefore do not describe the Rust enum's
variants or payload fields.

The proposed enum model records the actual Rust alternatives—`Idle`, `Count`
and `Packet`—and their fields. Output conversion needs this information to match
an existing Event and extract its active payload, including when Event is a
record property. Constructor selection remains a separate input operation;
the existing `.variant(fun!(...))` API keeps its current meaning.

The baseline probe rejects tuple and named payload enums with "only fieldless
enum variants are supported". It also demonstrates that callback `Result`
arguments and results are rejected, even when explicitly requested as values.
Fieldless enum generation is a positive control for both adapters.

## Lessons from the production implementation

Comparison reference: `main` at
`5e149f869710dfb71162c5af854c20e3ccb44cef`.

* Its Flat model distinguishes fieldless enums from sums and stores ordered
  alternatives with their payload fields. Payload tags are positions in that
  list, independent of source Rust discriminants. Keep these semantics.
* Its `sealed_class!` generates a sealed interface with nested data objects or
  classes. Its C `.tagged_union(type)` generates a tag/union and active-arm drop.
  Keep those destination-language concepts; use #808's declaration conventions.
* Its JNI expanded output implementation starts a payload conversion from
  `bind.clone()` in `encode_group_leaf`. Do not inherit that ownership strategy:
  #808's owned output should move the active fields, with copying explicitly
  requested by the binding author.
* Its C payload policy allocates a C string for `String` and has a narrower
  supported field set. #808 already has inline String/Vec owners and recursive
  conversion plans; use those rather than adding a second allocation policy.
* Its JNI fixed-layout property bridge explicitly rejects a `Vec` of sums.
  The new implementation must compose sequences and variants instead of
  inheriting this restriction or silently switching transport.
* Its C incoming-tag checks are outside #808's trusted-C contract. The new C
  adapter assumes valid tags and fields, as it assumes valid pointers today.

Relevant source locations are listed at the end for checking these claims.

## One source description, shared recursive compilation

Here **Flat** is the source model. A **wire** is an individual native field after
conversion, such as an integer or buffer descriptor. The shared declaration
compiler walks conversion recipes and emits their Rust construction/matching;
an adapter's **codec** supplies the foreign storage and language-specific pieces.

The proposed pipeline is:

```text
captured Rust enum
  -> Flat: source variants and their fields
  -> declaration resolution: value or explicit field expansion
  -> shared compiler: selected-variant input/output conversion
  -> adapter: Kotlin sealed value / C tagged carrier / expanded wires
  -> existing ordinary, collection, callback or bulk delivery
```

### 1. Flat describes the Rust value

Retain the existing fieldless enum description and its integer/niche mapping.
Add a payload-enum description containing ordered variants. Each variant has a
source name, field style (unit, tuple or named), payload fields and the existing
schema metadata slots for variants/fields. A field identifies its source member
by `syn::Member`, so a named member and a tuple position use the same recursive
field machinery. Refactor shared field operations accordingly; this does not
claim general tuple-struct support as part of this task.

Keep the existing integer-enum path for unit-syntax variants only. An enum with
any tuple/named constructor syntax uses the new variant model, even if that
constructor has zero fields. Preserve `A`, `B()` and `C {}` spelling when
generating Rust construction/matching. Thus an all-empty `enum E { A(), B {} }`
uses sealed/tagged variants. This differs from production's field-count
classification deliberately: numeric casts are not uniformly legal for these
forms (Rust 1.85 rejects them with explicit non-unit discriminants). Do not
weaken existing integer-enum guards or guess source discriminant layout.
Do not read or reproduce the memory layout of the source Rust enum, even if the
source uses `#[repr(C)]`. The foreign carrier contains *converted* field types.
Payload wire tags are signed 32-bit indices `0..N-1`; generation rejects overflow.
Existing fieldless discriminants and ABI remain unchanged.
This is an explicitly different numbering rule for payload enums: their source
discriminants, if present, do not define wire tags. Reordering variants changes
the foreign ABI. C headers and native libraries must be regenerated/distributed
together; there is no C runtime compatibility check. JNI verifies its tag mapping
at initialization. Stable user-selected tags can be considered separately.

Source selection must follow types inside variant fields, just as it follows
record fields. Unsupported field types fail generation with an enum/variant/field
path. Keep the current restrictions on generic user declarations, explicit
lifetimes, conditional variants and non-exhaustive enums. A recursive payload
requiring unsupported indirection also remains a generation error.

### 2. Resolve values and expansions before emission

`sealed_class!(Event)` and `.tagged_union("event", Event)` declare the ordinary
foreign value. Payload properties use canonical value conversions, just like
record properties after #906. A type-wide parameter expansion must not silently
change the declared Kotlin type of `Event.Packet.id`.
Whenever the ordinary representation of an enum or concrete Result needs a
foreign type, require its explicit `sealed_class!`/`tagged_union` declaration.
Missing declarations are build errors naming the required type; do not invent
public domain class names. An explicit enum-field expansion does not require
that ordinary declaration: its children and tag come from Flat, and it creates
no sealed object. This is necessary for expanded values whose payload types
have no foreign class, including Rust-only error types. Do not infer unused
ordinary payload-property types while preparing such an expansion.

The expanded tag is an integer with generated named constants, even without a
sealed/tagged declaration. For example, emit Kotlin top-level
`REPLY_RESULT_SAMPLE_TAG` and `REPLY_RESULT_ERROR_TAG` constants in the
occurrence's destination package and corresponding C prefixed macros. The SDK
adapter uses those constants, not integer literals. Derive names from source
type/variant names; concrete generic types use readable type arguments plus the
existing stable type hash, independent of variant order. Deduplicate per package
and reject name collisions. These constants create no domain object or enum
instance. Include every used enum's tag mapping in JNI initialization checks,
including field-only and callback occurrences, not just sealed declarations.
Constants are emitted per generated function/callback package, deduplicated
within that package; there is no mandatory global tag package. Their names
follow source variants even if `.variant_name(...)` renames a sealed class.
JNI initialization checks the generated Kotlin/native pair, not a separately
compiled SDK. Kotlin `const val` values are inlined into SDK bytecode, so rebuild
the SDK with its binding when the tag ABI changes; no SDK runtime check is claimed.
C retains its header/library compatibility contract with no runtime check.
Bulk/row storage helpers retain the existing stable naming scheme. A complete
canonical conversion to another representation can also avoid the declaration.
Concrete generic types such as Result require an explicit Kotlin `.name(...)`;
do not derive a potentially conflicting class named `Result`.

Add explicit enum-field expansions to the existing Input/Output tree API:

```rust,ignore
// Proposed operation: flatten tag + variant fields instead of passing Event.
fun!(exchange).expand_param("event", Input::enum_fields([
    ("Count", vec![("0", Input::value())]),
]))

// JNI example: bulk grouping also applies to a single expanded value.
fun!(exchange).expand_return(Output::enum_fields([
    ("Packet", vec![("bytes", Output::value())]),
]).bulk())
```

Keys are **source** variant names and source field members, independent of
foreign renaming. A member is a field name or a tuple index such as `"0"`, resolved
once to `syn::Member`. In an explicit enum-field expansion, omitted fields use
`Input::auto()` / `Output::auto()`, inheriting configured boundary defaults just
like existing explicit record-field expansions. Ordinary sealed/tagged value
properties continue using canonical conversions.
Reject unknown/duplicate keys; named recipes must not silently change meaning
when source fields are reordered. An output recipe may explicitly discard a
field; an input recipe must still construct every Rust field. An empty override
list inherits defaults for all variants. `Input::value()` / `Output::value()`
can explicitly select a field's canonical representation instead. Explicit discard uses the
existing `Output::Discard` node. Here "enum variant" always
means a source Rust variant; `.variant(fun!(...))` still means a constructor
alternative and is unchanged.

Keep builder inference predictable: add `enum_fields` on `Input<InputRequest>`
and `Output<OutputRequest>` with the fixed item shape
`(&str, Vec<(&str, Self)>)`, copying keys during resolution. An arbitrary
`IntoIterator` of this concrete item shape is accepted; keys are not generic
`Into<String>` arguments. `Input::enum_fields([])` and its Output equivalent
must compile without type annotations.
Output enum-field expansion follows the source occurrence's ownership: owned
values move active fields, while `&Enum` observes them by reference. Propagate
that access to child recipes. If a borrowed occurrence inherits an owned-only
helper such as Sample's `fields_into`, fail preparation with the variant/field
path; do not clone to make it fit. Input enum-field expansion constructs an owned
enum; borrowing a reconstructed enum still requires explicit reconstruction.

Type-wide boundary defaults apply in the same places as today's expansion
defaults, including recursively expanded optional/sequence occurrences. Ordinary
sealed properties and ordinary enum payloads request canonical value conversions,
not boundary defaults. Explicit enum-field expansions inherit boundary defaults;
their field overrides can select a different expansion or a canonical value.
A local function/callback override replaces that occurrence's
tree and does not mutate the foreign type declaration.
The explicit Kotlin field form exposes a tag `Int` followed by separately named
variant-field values, like existing constructor-selector expansions. Inactive
reference slots are nullable and ignored; an active required value must be
present. Expanded outputs use the existing Kotlin builder callback with those
tag/field arguments. `.bulk()` changes native storage, not those public
arguments. Only the ordinary value form accepts/returns one sealed Event.
Derive expanded public field names from their structural path: enclosing field,
source variant, payload member, then nested view/field names. A reply can expose
`resultSamplePayload`, `resultSampleEncodingId`, `resultErrorPayload` and
`resultErrorEncodingId` alongside `replierId`. Apply existing explicit naming
overrides and reject remaining collisions for both adapters; wire identifiers
retain the existing stable internal naming rules.
Omit the tuple position for a single-field tuple variant, so Sample(Sample)
adds `Sample`, not `Sample0`, to that path. Multi-field tuple variants include
their positions. Collision checking runs after this abbreviation and overrides.
For example, an expanded ReplyResult can inherit Sample's configured consuming
field expansion instead of producing a Sample handle. Expanding a surrounding
ReplyStruct must preserve its sibling replier id and branch over only the
result field; see the [real reply example](zenoh-usage.md#2-reply-delivery-must-keep-the-expanded-callback-path).

An ordinary sealed value nested in a data class uses tag-plus-field wires too:
Kotlin extracts its selected properties on input and reconstructs the selected
sealed value on output. Public grouping does not make the enum a JNI object
leaf. This permits RecoveryConfig's input to retain the current flattened native
shape. JNI reserves `-1` for payload-enum carrier emptiness as C does, although
JNI has no C carrier to drop; it is invalid as an ordinary value. Optional
absence can use another reserved tag (normally `-2`), falling back to the existing
explicit presence flag when a niche cannot be used. Inactive raw primitive slots
have zero defaults and reference slots null defaults; only selected fields are
decoded. Bulk changes physical storage, not this logical grouping.
If a public field form exposes an optional absence through a reserved tag,
generate its named constant too, using the control's structured occurrence path
and optional nesting (for example, `_NONE_TAG` / `_SOME_NONE_TAG`). Clients need
not hardcode reservations. An explicit presence-flag form exposes the flag
instead. Emit a C `_EMPTY_TAG` lifecycle constant for a grouped owning carrier.

Enum-field nodes support `.bulk()`, type-wide expansion registration, function
overrides and callback arguments through the existing entry points. JNI callback
results normally use the ordinary sealed value or another conversion exposing
one Kotlin value; general ungrouped tag-plus-payload fields do not satisfy that
existing rule. Diagnose such result overrides. Physical bulk packing does not
invent a public Kotlin tuple; generalized grouping remains the separate #830
task. C callback results can use their existing carrier/result-record protocol.
These nodes do not add another bulk or callback configuration API. Existing
constructor expansions remain useful as alternatives to enum-field input.

The resolved form retains the enum identity, selected representation (ordinary
value or expanded fields) and per-variant child trees. The shared compiler owns
Rust patterns/construction and recursive ownership. A codec operation for **enum
values/variant fields** supplies the physical tag, arm grouping and adapter
metadata around those children. Reuse shared child compilation for ordinary
and expanded forms; do not create fake captured constructor/getter functions or
an adapter-specific copy of the source variants.

Code generation consumes the prepared tree as today. Input remains one scoped
conversion operation; output retains prepare, delivery and commit. No additional
template pass, runtime conversion registry or per-type dispatch map is needed.

### 3. Give `Result` a data meaning without changing function errors

| Occurrence | Default meaning |
| --- | --- |
| Outermost result of an exported source function | Existing success/domain-error routing |
| Exported function parameter of type Result | Ordinary enum data |
| Callback argument or callback result | Enum data: `Ok(T)` or `Err(E)` |
| Record property, enum payload, optional or collection element | Ordinary enum data |
| Explicit function `.result_as_value()` | Whole Result is enum data instead of function-error routing |

Recognize concrete standard `Result<T, E>` in the same source-type model, with
two tuple variants, `Ok(T)` and `Err(E)`. It is a known standard type shape,
not a fabricated captured enum or a second adapter registry. Generic user enum
monomorphization remains separate work. Both adapters can explicitly name a
concrete Result value through the same declaration:

```rust,ignore
.class(sealed_class!(Result<Packet, DomainError>).name("PacketResult"))
// or:
.tagged_union("packet_result", syn::parse_quote!(Result<Packet, DomainError>))
```

The caller chooses the representation of `DomainError`, including a handle or a
canonical value conversion. There is no mandatory string conversion. A callback
returning `Err(error)` successfully returns a Rust Result value; it does not
trigger the binding-error handler. A failed conversion or foreign callback
failure keeps the existing callback failure policy.

Propose the explicit function setting `fun!(lookup).result_as_value()`. The
existing `.expand_return(...)` then configures the whole declared Result value;
without this setting it keeps configuring the successful `T`, as today.
Combining `.result_as_value()` with `.on_error(...)` is a declaration error.
The setting is order-independent and requires a top-level standard Result.
`Output::value()` alone keeps its current meaning and does not opt in.
This also deliberately changes which representation is selected *when that
setting is chosen*: ordinary Result data uses canonical payloads, whereas an
ordinary function outcome applies return-boundary defaults to its successful
`T`. Specify a local enum-field recipe when the data payload should instead
use an expansion. Type-wide error mappers are not invoked in data mode.
Here "error mappers" means the function outcome's `.on_error(...)` or
`.error_mapper(...)` selection. A type-wide `expand_return!(Error)` is a normal
boundary expansion, so an explicit enum-field expansion can inherit it for an
Err payload like any other output default. An ordinary canonical enum property
does not automatically inherit that boundary expansion.

Apply the function-error rule once at the exported source call. For
`Result<Result<T, E>, F>`, only the outer layer routes `F`; the inner Result is
data. A helper whose declared result exactly matches the expected data type
returns that value; `Result<Expected, ConversionError>` is a fallible helper.
Resolve against the expected type before treating a syntactic Result as failure.
Do not recursively reject every type containing Result.

`Option<T>` keeps its current optional conversion and niche optimization. Do not
replace it with a general two-arm carrier or change existing wire signatures.
Optional payload enums may reserve an unused tag through the existing niche
mechanism only when storage/lifecycle meanings remain unambiguous.

## Ownership and failure rules

Only the selected variant participates in conversion. Its children follow the
same move, borrow, canonical conversion and cleanup rules as record fields.

| Situation | Required behavior |
| --- | --- |
| Owned Rust enum output | Move active fields; do not clone to simplify matching |
| Borrowed enum output | Observe fields through existing borrowed conversions; reject unsupported owning projections |
| Foreign input becomes Rust enum | Validate as required by the adapter, take selected fields, construct that variant |
| Failure after some fields/rows were taken | Drop created Rust values and remaining owned resources according to actual transfers; no rollback |
| Inactive variant storage | Never convert, dereference, acquire or drop it |

Rust's restrictions on moving fields out of a type with a custom `Drop` remain
in force. Flat does not capture Drop implementations: an unsupported direct
move fails when rustc compiles the generated Rust (E0509), without another
build-time tracking registry. Such a type needs an explicit consuming helper if
direct field moves
are unavailable. A borrowed enum does not authorize a temporary owning copy:
borrowed inputs use a native reference representation or explicit reconstruction.
General lifetime-bearing enum views are outside this task.
The existing `convert!(Event).output(fun!(into_representation))` or explicit
constructor/getter expansions provide those helpers; no new consuming-helper
registry is needed. An ordinary C owning carrier cannot represent `&Event`
whose payload needs ownership. Reject that projection. An explicit field
expansion may borrow individual fields using existing view/pointer converters;
it is not an owning `pb_event` and has no owning enum destructor. A general
borrowed-view carrier can be designed with the lifetime work later.

Output preparation remains transactional with respect to foreign delivery:
existing RAII guards own prepared handles, buffers and inline owners until
commit. If any later active field/row fails, drop those guards and undelivered
Rust values. Extend those guards to enum branches instead of releasing ownership
merely because a carrier field has been written. Commit only at the delivery's
existing ownership-transfer point. This output cleanup is distinct from input
rollback, which is not promised.

### C carriers

Use raw integer tags and union payload storage in the generated Rust ABI
carrier, with `ManuallyDrop`/`MaybeUninit` where necessary. Do not form a typed
Rust source enum from unvalidated carrier bytes. C callers promise valid tags,
active fields and ownership; invalid input violates the function contract.
The empty lifecycle tag is valid for drop, not a source-function argument.

Pass a resource-owning input carrier by mutable pointer, following #808's inline
owner convention. Each actual take empties that slot in the caller's storage.
If a later field fails, the active tag stays and the caller can drop the
remaining fields safely. Once the entire value is taken, mark the carrier empty.
`pb_drop_event` drops only the active payload and empties the carrier; a second
drop is a no-op. This requires no `Event: Default`. Pure data carriers may pass
by value. Ownership is derived from converted fields, not guessed from the
source enum's `needs_drop` alone.
Consequently, adding an owning payload changes an input signature from a value
to a mutable carrier pointer; this is a documented ABI change requiring caller
regeneration. The header states the selected passing mode and ownership.

Every owning wire used inside such a carrier must have the existing idempotent
empty-after-take operation: null for owned pointer handles, configured empty
storage for inline owners, and the empty lifecycle tag for nested tagged values.
Records compose their children's empty states; optional payloads retain a
drop-safe state for any already-taken child. Unsupported ownership forms fail
generation. Retain actual-transfer reporting so callback/array cleanup skips
the taken prefix. Reserve `-1` exclusively for enum lifecycle emptiness;
the existing niche allocator must exclude that reservation for payload enums.
Optional absence uses another unused negative tag, normally `-2`, with further
nested optional reservations allocated distinctly. Existing fieldless enums
have no lifecycle tag and keep their current niche allocation. A consumed enum
must never be mistaken for an ordinary `None` argument.

For C callbacks, an owned Rust enum argument transfers as a by-value carrier:
Rust commits its argument guards immediately before invoking `run`, as today.
The C callback must consume or drop the active payload. A by-value enum returned
by the C callback transfers to Rust; C must not subsequently reuse/drop that
owner. Rust decodes mutable local carrier storage and reclaims untaken active
fields on failure through its callback-result guard. Existing `release_result`
releases only returned row/buffer storage, never moved owners. Borrowed arguments
require explicit nonowning field/view conversions and live only during `run`.
Borrowed callback returns remain unsupported.
There is no separate C `run` binding-failure return: a returned Result's `Err`
is data. A Rust-side failure decoding a returned carrier invokes the untaken
active-field cleanup above before applying the existing callback failure policy.
These are #808's current C transfer defaults. Zenoh's existing C consumer uses
scoped, takeable pointer callbacks, with cleanup by the trampoline after return.
That adapter-wide ownership-policy gap is a separate migration prerequisite;
enum support must not claim that those callers can be used unchanged. Keep
variant conversion independent of the outer callback's scope/transfer policy.
The carrier must provide the operations needed by that future policy: pass its
address during a scoped callback, take the whole carrier into caller storage
while leaving the source tag `-1`, and drop the active remainder afterward.
A take destination must be uninitialized or resource-free; a typed move
preserves payload ownership without interpreting inactive bytes/padding. Adding
the outer scoped/takeable callback setting remains separate work.
For ungrouped expanded arguments, that policy uses the existing take/empty
operations of individual owning leaves; there is no whole-enum carrier to take.

For JNI callbacks, argument handles remain scoped unless the existing explicit
transfer/retention mechanism is used. Returned handles transfer into Rust as
today. Kotlin construction failure cleans completed and partial active payloads,
including unread owning rows, using the existing static cleanup helpers. Emit
active-variant `AutoCloseable` behavior for sealed values reaching native handles,
consistent with production's sealed surface; closing a variant delegates to
its owned wrapper fields and does not invent a native enum allocation.
Moving a handle marks that **handle wrapper** consumed using today's mechanism;
the sealed container has no separate native identity or live bit. Its `copy()`
shares wrapper objects, as current data classes do: repeated close delegates to
the same idempotent handle close. Copying the wrapper object does not clone Rust
ownership. Kotlin preflight visits only the selected variant's reachable handles
and keeps existing type grouping, locking and actual-take semantics.

## Ordinary expansion and bulk share the variant tree

An explicit parameter expansion can keep a fixed native signature: one tag and
side-by-side slots for all variants. Only the selected slots are meaningful;
generated callers supply ABI-safe zero/null defaults for unused slots. This
does not require constructing inactive domain objects.
Expanded outputs retain existing delivery conventions: a non-Result C function
returns its first wire (the tag) and writes further wires through output
pointers; a function outcome keeps its existing success/error delivery. The
ordinary C enum representation is instead one carrier, returned by value. With
the initial example, its non-fallible signature is conceptually
`pb_event pb_exchange(pb_event *event)`.
For C field expansions, **only active payload output slots are written and may
be read/dropped**. The header documents those tag-dependent obligations; unused
output pointers are not dereferenced. On input, C callers pass the tag and
side-by-side raw slots themselves; inactive slots are ignored and may use
zero/null raw values. The Kotlin wrapper supplies its raw inactive defaults.
Neither adapter constructs inactive owners just to obtain defaults.
Expanded C callback arguments follow the same active-arm ownership rule. All
by-value raw ABI arguments have initialized neutral storage in inactive slots
(zero/null or a nested carrier's empty tag), without constructing inactive Rust
owners. Only active slots represent transferred resources and may be decoded or
dropped by the callback. Grouped tagged carriers still expose only their active
union payload.

A grouped C carrier or JNI bulk row can instead store the largest payload in a
union. This saves space when many variants have different fields. The logical
variant tree is the same; the physical storage is the codec's decision. Existing
JNI's `.bulk()` groups the prepared expansion, including enum nodes. Arrays contain
fixed-stride carrier rows; nested dynamic payloads keep existing range/owner
representations instead of embedding Java objects or Rust Vec internals.
C keeps its existing whole-Vec owner by default; element conversion applies
when a collection is explicitly expanded. Adding enums must not silently replace
that owner with a row array. JNI retains its current eligible-array bulk defaults.
An ordinary C `Vec<Event>` owner can be passed/returned as an opaque whole value;
this task adds no implicit element getter or vector constructor. C callers that
need to build/read individual Event rows use an explicit sequence expansion with
the enum-value element recipe. Required C sequence coverage uses that expansion,
including owning elements, while checking that whole-vector round trips remain
unchanged. It does not imply that a borrowed owning enum can be copied out.
This task does not enable the `.bulk()` modifier in C; C continues using its
native carrier/row conventions. When a C row's enum recipes are canonical, reuse
the declared tagged carrier rather than generating a second equivalent layout.
Only genuinely different explicit field recipes need a distinct carrier shape.
Such ABI helper names follow the existing readable type/shape plus stable-hash
scheme; they are not new unnamed public domain classes. Derived C tag/body/field
names must pass current identifier/keyword checks and be unique after mangling.
Reject collisions with source paths in the diagnostic. Per-variant C renaming
remains with the naming-policy backlog; do not silently emit invalid identifiers.

Extend `prebindgen-reprc` with target-measured tagged/union layout generation,
reusing its field/layout infrastructure. Both adapters use that definition.
The loaded native library supplies Kotlin the tag offset, each variant field
offset, total size/alignment and existing nested layout/transport information.
Include tag mapping in initialization compatibility checks. Kotlin never
calculates offsets from host assumptions; native compilation determines them,
including during cross-compilation. C's separate opaque layout artifact rules
continue to apply to opaque payload storage.
Measure component record/union layouts separately and add offsets as constants;
do not require a new compiler feature for nested union-field offsets. Keep Rust
1.85 support and test generated layout code on that version and cross targets.
If every active variant has zero wire fields, emit a tag-only carrier and omit
the union; do not manufacture an empty C struct/union or an unsupported empty
`prebindgen_reprc::Record`.
Apply that rule per arm as well: a zero-wire arm has no union member or payload
record to measure, even when other arms have storage. The tag alone identifies
that arm; zero-wire Rust fields are reconstructed by their child recipes.

This requires a real decoder change: today's flat bulk reader loads fields
before child conversion. A union reader must load the tag first, check it on
JNI input, and then read **only the selected variant's fields**. Inactive bytes
may be uninitialized; reading them as offsets, pointers or Rust values would be
incorrect even if later conversion ignored them. Writers initialize safe byte
storage and write selected fields, without copying Rust padding. JNI checks
selected descriptors/lengths with the existing bulk validation rules. C adds no
runtime tag, alias or pointer checks.

Tag validity is required for every JNI input form, not only bulk. A native
integer-to-enum match rejects an unknown tag before constructing the source
variant or touching its payload, using the existing binding-failure route
(including the existing non-unit callback-result failure policy). Kotlin
preflight must also select handles only from a valid active branch. This is
scalar/wire validation, not a native alias-check pass. In C the unsafe caller
contract supplies valid tags; an impossible catch-all may use
`unsafe { core::hint::unreachable_unchecked() }`, without adding validation.

Ordinary object delivery, bulk arrays, callback arguments and callback returns
must consume this same branch-aware conversion and cleanup plan. A callback
must not retain an older adapter-only enum converter. Keep existing delivery
defaults and callback scopes; do not add an automatic object fallback when a
requested bulk shape cannot be represented. Diagnose that shape instead.

## Implementation stages and acceptance

Each stage extends the same conversion path; temporary parallel enum converters
are not part of the plan.

1. **Source and compiler:** add payload fields and standard Result data, source
   selection through variants, value/field enum operations and expected-type
   helper handling. Test mixed unit/tuple/named variants and zero-wire unit fields.
2. **Ordinary adapters:** sealed Kotlin values and C tagged carriers in both
   directions, plus field expansions without an ordinary class declaration.
   Cover recursive fields, naming, canonical conversions, inherited boundary
   defaults, local overrides, active-arm drop and partial-consumption reporting.
3. **Bulk and callbacks:** target-measured union layout, tag-first readers,
   Kotlin preflight of selected handle fields, sequences and callbacks using
   the same prepared trees and cleanup helpers.
4. **Boundary verification:** add generated C examples and Kotlin covertest
   coverage before claiming the feature complete. Record generated sources.

Required cases include `Vec<Event>`, records containing enums, enums containing
records/options/collections/handles, borrowed output without implicit copying,
owned non-Clone payloads, `Result<(), E>`, nested Results, callback Result data
in both directions, and outer function typed errors remaining unchanged. Exercise
failures after partial moves, selected-field conversion, Kotlin construction and
callback return conversion. Unused bulk arm bytes must not be inspected.
The Zenoh acceptance matrix adds RecoveryConfig's optional RecoveryMode with
converted Duration, a TimestampStackRecord list with nested enum/record/bytes,
and consuming Reply -> ReplyStruct -> ReplyResult callback expansion. Track
the independent helper-signature, scalar-domain, fixed-array and callback-scope
prerequisites explicitly; this proposal is not a full Zenoh consumer port.

Measure scalar-only and owning-payload enums, single values and arrays, ordinary
and bulk delivery, and callbacks. Compare against equivalent explicit
tag/field conversions on the same base; report latency, allocation counts and
large-arm carrier sizes. The design promises no extra mandatory cloning,
native heap owners or per-row closures. It does not promise that a sealed Kotlin
object is free to allocate or that a compact union always beats expansion.

This PR leaves the payload-enum checkbox open. Implementation is complete only
when the composed boundaries above work, with reviews and CI passing.

## Reproduce and inspect

From the repository root:

```sh
cargo run --manifest-path docs/investigations/payload-enums/probe/Cargo.toml
```

The probe is generation-only; it checks the current rejections and parses the
positive control's generated Rust. It does not compile/run a generated foreign
boundary or benchmark an implementation that does not exist yet.
It also contains 16 selected real Zenoh declaration shapes, tests their current
rejections, and demonstrates why an explicit field expansion must not depend
on declaring an ordinary foreign class. To verify the fixture against the pinned
Zenoh Git object in a local checkout:

```sh
cargo run --manifest-path docs/investigations/payload-enums/probe/Cargo.toml -- \
  --zenoh-flat ../zenoh-flat
```

Current #808 references:

* [Flat parser and source descriptions](../../../prebindgen-flat/src/lib.rs)
* [Shared input/output compilation](../../../prebindgen-declarations/src/lib.rs)
  and [codec operations](../../../prebindgen-declarations/src/operations.rs)
* [Fieldless enum mapping](../../../prebindgen-declarations/src/enums.rs)
* [JNI classes/properties](../../../prebindgen-jni/src/data.rs),
  [callback compilation](../../../prebindgen-jni/src/callback.rs) and
  [bulk reader/writer](../../../prebindgen-jni/src/bulk.rs)
* [Target-measured record layouts](../../../prebindgen-reprc/src/record.rs)

Production comparison sources at the pinned commit:

* [Source variant model](https://github.com/milyin/prebindgen/blob/5e149f869710dfb71162c5af854c20e3ccb44cef/prebindgen-flat/src/flat/element.rs)
* [Sealed-class declaration](https://github.com/milyin/prebindgen/blob/5e149f869710dfb71162c5af854c20e3ccb44cef/prebindgen-jni/src/jni/decl.rs)
  and [expanded sum output](https://github.com/milyin/prebindgen/blob/5e149f869710dfb71162c5af854c20e3ccb44cef/prebindgen-jni/src/jni/emit/sum_out.rs)
* [JNI sequence-of-sums restriction](https://github.com/milyin/prebindgen/blob/5e149f869710dfb71162c5af854c20e3ccb44cef/prebindgen-jni/src/jni/tests/sealed.rs)
* [C tagged-union policy](https://github.com/milyin/prebindgen/blob/5e149f869710dfb71162c5af854c20e3ccb44cef/prebindgen-c/src/builder.rs)
  and [C conversion/drop emission](https://github.com/milyin/prebindgen/blob/5e149f869710dfb71162c5af854c20e3ccb44cef/prebindgen-c/src/trait_impl.rs)
