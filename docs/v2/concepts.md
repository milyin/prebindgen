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
- The [target adapter](README.md#the-components) supplies language-specific
  answers to the registry. For C, it describes member reads from a struct. For
  JNI, it describes getter calls on a JVM object. The registry combines those
  operations; the adapter does not independently traverse the entire type tree.
  Where a chapter is describing what the engine asks rather than who implements
  the answer, it calls this component simply **the target**.

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
the generator supports it. [The report](report.md) later accounts for each
declaration; a source item that nobody requested currently has no entry.

The current engine's `Declaration` is one value: the kind the target gets and
the Rust name, printed as `type:Stamp` or `fn:stamp_sum`, and it is its own
identity. Renaming the foreign function does not change it. Exposing the same
source function at several placements makes several declarations, told apart
by a projection label the frontend supplies; one declaration stated twice is
rejected.
See [Record binding requests](stages/03-requests.md#record-binding-requests).

### Choice

One language-specific decision the binding recorded about a declaration or a
value, held by the frontend that recorded it and by the target that frontend
builds. For example, C records that `Stamp` crosses as a by-value struct, while
JNI records the Kotlin class that will carry it. The registry stores none of
them and knows no precedence among them: it asks the target whenever it needs a
language-specific decision, and the target resolves what applies from its own
storage. The shared planner never interprets a C or Kotlin option.
See [What a choice records](stages/03-requests.md#what-a-choice-records).

### Conversion key

The adapter's own name for one way of converting a value, returned from
`select` beside the relation. The registry never looks inside it; it compares
keys, and two values whose crossing, relation, children and key all match share
one plan. So a key means one thing: *equal keys are interchangeable
conversions*. Settings that generate differently must produce different keys,
and a fresh key per visit would share nothing and defeat cycle detection. See
[Finding an existing conversion plan](stages/03-requests.md#finding-an-existing-conversion-plan).

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
is used. Current V2 represents this with a declaration id and a path; the
separate `SiteId` type shown in the design is not implemented yet.
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
the selected relation, conversion key and child plans are also needed to
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

### Representation

The target adapter's description of the foreign-side values and the operations
that access them. C uses a `repr(C)` struct whose members can be read directly.
JNI uses a JVM object whose properties are read by getter calls. Both can serve
the same source-side struct relation, but need different access operations.
The adapter describes those operations; the registry combines them with the
children's conversions. See [Represent and compose values](stages/05-represent.md#represent-and-compose-values).

### Carrier

A value carrying data during conversion: a wrapper argument, wrapper return or
intermediate value in generated Rust. In the JNI example, the object reference
is one carrier and an integer returned by a getter is another. Its `WireType`
describes the Rust type used to hold it and whether it is safe at the wrapper
boundary. An internal temporary need not itself be legal as an
exported parameter. See [Describing target values and operations](stages/05-represent.md#describing-target-values-and-operations).

### Primitive

One typed operation supplied by the target adapter: read a member, call a
getter or report an error. The word means an operation here, not a primitive
type such as `i64`. Its description states its inputs, result, possible failure
and generated-code dependencies. It does not choose the caller's local variable
names or decide what the enclosing function does on failure. The registry
plans those connections and control flow before the writer renders code.
See [Represent and compose values](stages/05-represent.md#represent-and-compose-values).

### Node

A completed conversion plan stored for reuse. Its identity includes the
crossing, selected relation, conversion key and child-plan identities. Those
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

The final status of one declaration on a completed run: emitted, skipped or
explicitly ignored. [The report](report.md) lists them so a missing function
does not have to be diagnosed by inspecting generated code alone. An emitted
declaration passed generation; it still needs compilation and runtime testing.
A generation error, such as contradictory configuration, returns no completed
generation result or report. The skipped outcome belongs to the V2 transition:
once V2 covers what V1 covers, a request it cannot generate fails the build
instead. See [Retain supported output](stages/07-retain.md#retain-supported-output).

### Capability

The category of missing support that prevents a declaration from being emitted.
The stable code `unsupported.jni.carrier`, for example, identifies a value that
the JNI target cannot carry yet. The accompanying explanation and path identify
the particular type and position. Several skipped declarations can share a
code, helping a developer see which missing feature would unblock the most
requests — and, while V2 is being completed, which gap to close next. See [Retain supported output](stages/07-retain.md#retain-supported-output).
