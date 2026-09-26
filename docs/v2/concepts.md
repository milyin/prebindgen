<!-- spec: {"kind": "concepts"} -->

[Project contents](README.md)

# The vocabulary

Start with a small Rust library. It exposes a struct with two integer fields
and a function that adds them. The annotation tells prebindgen which items it
may inspect when generating bindings:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64 { stamp.secs.wrapping_add(stamp.nanos) }
```

A C binding should let a caller pass a C struct to `stamp_sum`. A Kotlin binding
should provide `stampSum(stamp: Stamp): Long`. Both calls must eventually reach
the same Rust function, even though neither caller supplies a Rust `Stamp`
directly. Three components cooperate to generate the connecting code:

- The [frontend](README.md#the-components) is the Rust configuration API used
  in the binding crate's build script: `prebindgen-c` or `prebindgen-jni`. You
  use it to say which items to expose and what the foreign API should look like.
- The [registry](README.md#the-components), `prebindgen-registry-v2`, is the
  shared planner. It determines how values must be converted, assembles the
  wrappers and renders their Rust code. It also records which requests
  it could not satisfy.
- The [target adapter](README.md#the-components) knows how to write its
  language. Its frontend states, before planning, which Rust types may hold a
  value at the boundary and how each value crosses — for C, a struct read
  member by member; for JNI, a JVM object read through getters. Its writers
  then turn what the registry feeds them into text: one getter call, one
  `repr(C)` struct declaration, the Kotlin sources. The adapter never walks
  the type tree and never decides anything while the registry plans.
  Where a chapter is describing the writing role rather than the crate, it
  calls this component simply **the target**.

Frontend and adapter are two roles in the same language-specific crate, not
two packages you configure separately. The registry is shared by both targets.

The stage chapters explain the process in order, but a stage often needs to
refer to something produced later. For example, configuration chooses how a
value should look before planning builds its conversion. This page introduces
the vocabulary needed to follow those connections. Read it as a short tour,
then follow each entry's link when you need the detailed contract. The examples
describe the implemented `Stamp` path; extensions are identified as planned.

Every word below is **defined** — set in bold — on exactly one page, under the
heading its entry links to, and every other page links its first mention of the
word there — except for the four that are also ordinary English words the
chapters need in that sense (declaration, root, part, capability), which keep
only the single definition. [`validate.py`](FORMAT.md#vocabulary) checks both.

## What the source provides

### Source item

A function, type or constant made available to the generator, normally by
marking it with `#[prebindgen]`. Here, `Stamp` and `stamp_sum` are source items.
Capture saves their syntax; the source model then describes their types and
signatures in a form later stages can inspect.

Being a source item does not automatically make something part of a C or
Kotlin API. The binding still has to request it. Conversely, a binding can
declare things that are not captured source items, such as a callback signature
or a type like `String`.
See [Capture source items](stages/01-source.md#capture-source-items).

## What the binding asks for

### Declaration

One requested public item: for example, expose `Stamp` as a C struct or
`stamp_sum` as a Kotlin function. This is a statement of intent, not proof that
the generator supports it. The run later says which declarations it could not
generate; a source item nobody requested is not one of them.

The current engine's `Declaration` is one value: the kind the target gets and
the Rust name, printed as `type:Stamp` or `fn:stamp_sum`. Renaming the foreign
function does not change it. What the engine plans and accounts for is that
declaration together with the [choice](#choice) recorded with it, so exposing
the same source function at several placements makes several outputs; stating
one declaration twice under one choice is rejected.
See [Record binding requests](stages/03-requests.md#record-binding-requests).

### Choice

One language-specific decision the binding recorded about a declaration or a
value. For example, C records that `Stamp` crosses as a by-value struct, while
JNI records the Kotlin class that will carry it. The frontend hands every
choice to the registry as data before planning: as a Rust type allowed at the
boundary, as a rule for how some values cross, as the form an exported
function takes, or as metadata only the foreign writer reads. The registry
decides everything from those; it compares metadata but never interprets a C
or Kotlin option.
See [What a choice records](stages/03-requests.md#what-a-choice-records).

### Conversion rule

How the values in one scope cross, recorded by the frontend before planning:
every value of one type, or the one value at one position inside one requested
function or type. It says what holds those values at the boundary and how
they are read and built, so C's rule for `Stamp` says a `repr(C)` struct read
member by member. Rules are data the registry holds and looks values up in,
with one precedence: a rule at a value's position outranks a rule for its
type. See [Conversion rules](stages/03-requests.md#conversion-rules).

### Root

A declaration considered as a starting point for planning. Asking for
`stamp_sum` starts work on its input, result and exported entry point; those
pieces are dependencies of that root. Explicitly asking for `Stamp` makes the
type a root too. It can therefore remain in the output even if a function that
uses it is skipped for an unrelated reason.
See [Where planning starts](stages/03-requests.md#where-planning-starts).

### Site

A position inside a requested function: parameter 0 of `stamp_sum`, for
example, or its return value. A type alone cannot identify that position:
two parameters may have the same type but require different settings.
Position-specific overrides and diagnostic paths therefore name where a value
is used, as the first step of a path under the output that exports the
function: `param stamp`, or `return`.
See [A value's position in an exported function](stages/03-requests.md#a-values-position-in-an-exported-function).

### Part

A position inside a source value, such as the `secs` field of `Stamp`. Planning
the struct means planning each of its parts and then combining the results.
When constructor relations are implemented, a constructor's arguments can
serve as parts instead of fields. A part is therefore not necessarily a field
in the wider design: it is one step from a value to a value it is made of,
whatever the source-level means of taking that step. Sites and parts together describe the positions at which
planning happens. See [Record binding requests](stages/03-requests.md#record-binding-requests).

## What the registry plans

### Conversion

The work needed to turn one side's value into the value required on the other
side. For a Kotlin argument, this means reading the object's properties and
constructing a Rust `Stamp`. For the function's result, it means delivering a
Rust `i64` as the foreign integer type.

A conversion plan does not call `stamp_sum` or belong exclusively to that
function. Another function needing the same value transformation can reuse it.
See [Select conversion relations](stages/04-select.md#select-conversion-relations).

### Crossing

The exact source type paired with its direction of travel. The argument makes
the crossing `Stamp` into Rust; the result makes the crossing `i64` out of
Rust. Direction matters: constructing a Rust struct is not the same operation
as taking a returned struct apart. The crossing identifies the problem, but
the representation that applies and the child plans are also needed to
identify a reusable solution. See [Finding an existing conversion plan](stages/03-requests.md#finding-an-existing-conversion-plan).

### Relation

A link from one Rust type to the other Rust values it is constructed from or
read into. `Stamp` has a struct relation to its two fields: obtain two `i64`
values and construct `Stamp { secs, nanos }`. It links the type to both of them
at once, so a relation is a bundle of edges rather than a single edge, and the
edge to one of those values is a part. A scalar uses an atomic relation, the
empty bundle: handled whole, with nowhere to descend.

Relations make the source types a graph, and planning a conversion is a walk
across it. One type can have several — its fields, and, when implemented, a
constructor such as building `Stamp` from one `millis` argument — so the edges
carry labels, and the label taken is part of the resulting plan's identity.
That graph may contain cycles; the plan graph built from it may not.

A relation says nothing about a C struct or Kotlin object: it describes the
source-side work.
See [What a relation is](stages/04-select.md#what-a-relation-is).

### Carrier

A Rust type the binding allows to hold a value during conversion: a wrapper
argument, wrapper return or intermediate value in generated Rust. In the JNI
example, the `Stamp` object reference is one carrier and the `jlong` a getter
returns is another. The frontend declares each one as a `WireType`: the Rust
type, which of the adapter's few wire types it is, for an aggregate which
wire types its members may be, and the metadata the target's writers need — a
C name, a JVM descriptor. Two carriers may share a
Rust type and differ in metadata, as two `JObject`s of different classes do.
Where a carrier may appear is stated by what holds it: a function form lists
the wire types its wrapper parameters may be, so an internal temporary need
not be legal as an exported parameter. See [Describing target values and operations](stages/05-represent.md#describing-target-values-and-operations).

### Representation

How the values a conversion rule covers cross: which carrier holds them,
which relation they are read through, and the operations that read and build
the carrier. C uses a `repr(C)` struct whose members can be read directly.
JNI uses a JVM object whose properties are read by getter calls. Both can serve
the same source-side struct relation, but need different access operations.
A type may have several — `Stamp` as a C struct and as a handle — so the
frontend declares each once, and a conversion rule or an exposed type names
it. The registry combines its operations with the children's conversions. See [Represent and compose values](stages/05-represent.md#represent-and-compose-values).

### Primitive

One operation a representation or a function form names: read a member, call a
getter or report an error. The word means an operation here, not a primitive
type such as `i64`. A standard one the registry writes itself; a target's own
one the target writes when the registry feeds it typed operands. Its
description states its possible failure and the runtime contexts it needs;
its operand and result types follow from where it is used. It does not choose the caller's local variable
names or decide what the enclosing function does on failure. The registry
plans those connections and control flow before the writer renders code.
See [Represent and compose values](stages/05-represent.md#represent-and-compose-values).

### Node

A completed conversion plan stored for reuse. Its identity includes the
crossing, the representation that applied and child-plan identities. Those
details explain why “same Rust type” is not enough for sharing: two structs
whose fields need different conversions need different plans too. In the
example, both `i64` fields can reuse one input node, applied once per field.

Nodes and the children they name form the graph the later stages read. It is
acyclic: a node is recorded only once every child it names exists, so no edge
can point at a plan still being built. A conversion that would need itself is
refused rather than followed, which is how a cyclic source type stays out of it.
See [Represent and compose values](stages/05-represent.md#represent-and-compose-values).

### Artifact

A generated Rust unit needed by an operation or public declaration. The JNI
error-reporting helper and the C-compatible `Stamp` type are examples. Giving
each unit a name lets the engine collect dependencies and avoid emitting the
same helper repeatedly. This is a build-time dependency, not a runtime resource
that needs cleanup. Some design sketches use the word more broadly for
generated outputs; current engine `Artifact` values contain Rust.
See [Individual target operations](stages/05-represent.md#individual-target-operations).

## What comes out

### Wrapper

The generated Rust function around one source call. It receives the foreign
arguments, runs the input conversions, calls `stamp_sum`, then converts and
delivers the result. It also handles conversion failures according to the
target's convention. C calls an `extern "C"` function; the JVM reaches an
`extern "system"` JNI entry point through an `external` method. This wrapper is
distinct from the Kotlin convenience function that calls that `external` method.
See [Assemble the wrapper boundary](stages/06-boundary.md#assemble-the-wrapper-boundary).

### Outcome

The final status of one declaration on a completed run: emitted or skipped.
`Generation::skipped` lists the skipped ones with the capability that stopped
each, so a missing function does not have to be diagnosed by inspecting
generated code alone. An emitted declaration passed generation; it still needs
compilation and runtime testing. A generation error, such as contradictory
configuration, returns no generation at all. The skipped outcome belongs to the V2 transition:
once V2 covers what V1 covers, a request it cannot generate fails the build
instead. See [Retain supported output](stages/07-retain.md#retain-supported-output).

### Capability

The category of missing support that prevents a declaration from being emitted.
The stable code `unsupported.jni.carrier`, for example, identifies a value that
the JNI target cannot carry yet. The accompanying explanation and path identify
the particular type and position. Several skipped declarations can share a
code, helping a developer see which missing feature would unblock the most
requests — and, while V2 is being completed, which gap to close next. See [Retain supported output](stages/07-retain.md#retain-supported-output).
