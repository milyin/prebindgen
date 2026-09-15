<!-- spec: {"kind": "concepts"} -->

[Project contents](README.md)

# The vocabulary

The pipeline is described one stage at a time, in the order the stages run. Its
vocabulary does not respect that order: most of the nouns name what flows
*between* stages, so a chapter reaches for a word two stages before the chapter
that owns it. This page introduces each of those words once, in a sentence or
two, and says which chapter specifies it. Read it first; come back to it when a
chapter uses a word you have not met.

Every word below is **defined** — set in bold — on exactly one page, the one
linked from its entry, and every other page links its first mention of the word
there. [`validate.py`](FORMAT.md) checks both.

### Source item

A function, type or constant the source crate marked with `#[prebindgen]`, as
captured and then as the source model describes it. Everything the pipeline
generates is generated *from* source items and *for* the declarations that name
them. — [Capture source items](stages/01-source.md#capture-source-items)

### Declaration

One thing a binding asked to expose: this function, that type, at this foreign
placement. Identified by a `DeclarationId` built from the source name, so a
foreign rename does not change what a test or a report is talking about. A
declaration is what the report accounts for; a source item nothing declared is
not in the report at all. — [Record binding requests](stages/03-requests.md#record-binding-requests)

### Policy

The target-specific choices recorded with a declaration or a value: `Stamp` as
a by-value C aggregate, this exported symbol, that Kotlin class. Data the
target itself interprets later; the registry only carries it and hands it back
to the target that wrote it. — [What policy means](stages/03-requests.md#what-policy-means)

### Site

A position inside a declared function — parameter 0 of the exported
`stamp_sum`, or its return — which is what an override is recorded against and
what a skip's diagnostic path names. — [A value's position in an exported function](stages/03-requests.md#a-values-position-in-an-exported-function)

### Conversion

One value-shaped problem: turning what a foreign caller passed into the Rust
value a source function takes, or the reverse for its result. Planning
conversions is the pipeline's central work; a conversion knows nothing about
which function it serves. — [Plan value conversions](stages/04-values.md#plan-value-conversions)

### Relation

A link inside the source domain: from a Rust type to the values it is built from
or read into, with the source-level means of getting between them — `Stamp` to
its two fields, or to the one argument of a constructor. A type can stand in
several; the target selects one, by `RelationId`. — [What a relation is](stages/04-values.md#what-a-relation-is)

### Representation

The target's answer to what carries a value on the foreign side and how it is
accessed: a `repr(C)` struct read member by member, a JVM object read through
its getters. Described by the target, walked by the registry. — [Plan value conversions](stages/04-values.md#plan-value-conversions)

### Carrier

A value holding conversion data on its way across: a native argument, a return,
or a temporary inside the generated Rust. A carrier has a `WireType`, and only
one flagged as ABI-safe may appear in an extern signature. — [Individual target operations](stages/04-values.md#individual-target-operations)

### Primitive

One typed operation the target supplies — read this member, call this getter,
throw this error — with what it needs, what it produces and how it can fail.
An operation, not a primitive type. The registry composes primitives; it never
pastes their text together. — [Plan value conversions](stages/04-values.md#plan-value-conversions)

### Node

A planned conversion, kept so that every value crossing the same way shares
it. Its identity is the exact type, the direction, the selected relation and
the effective policy, plus the children's identities. — [Plan value conversions](stages/04-values.md#plan-value-conversions)

### Artifact

A generated unit of Rust an operation or a declaration depends on — a helper
function, a `repr(C)` type — named so the registry keeps one copy and publishes
only what a retained output needs. — [The operation specification and its uses](stages/04-values.md#the-operation-specification-and-its-uses)

### Wrapper

The native function a foreign caller actually calls: the registry assembles it
around the conversions for one declared function, in the calling convention the
target dictates. — [Assemble the native boundary](stages/05-boundary.md#assemble-the-native-boundary)

### Outcome

What became of a declaration: emitted, skipped with the capability it waits on
and the path to where planning stopped, or ignored because the user said so.
Every declaration has exactly one, and the report lists them all. — [Retain supported output](stages/06-retain.md#retain-supported-output)
