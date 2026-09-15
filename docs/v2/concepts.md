<!-- spec: {"kind": "concepts"} -->

[Project contents](README.md)

# The vocabulary

One source crate marks a record and a function over it:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64 { stamp.secs.wrapping_add(stamp.nanos) }
```

A C binding crate wants `stamp_sum(struct Stamp)` in a header; a Kotlin binding
crate wants `stampSum(stamp: Stamp): Long`. Three parties make that happen. The
[frontend](README.md#the-components) is the Rust API a binding crate's build script configures —
`prebindgen-c` or `prebindgen-jni` — saying what to expose and how it should
look. The [registry](README.md#the-components) (`prebindgen-registry-v2`) is the shared planner: it
works out every conversion a binding needs, assembles the native functions,
renders the Rust and reports what it could not do. The [target adapter](README.md#the-components) is
the same crate as the frontend, facing the other way: the registry asks it the
language-specific questions — what carries a `Stamp` in C, how a Kotlin object's
properties are read — and it answers, item by item, without a type walk of its
own. Frontend and adapter are one crate per target language; the registry is one
crate for all of them.

The chapters describe the pipeline one stage at a time, in the order the stages
run. The vocabulary does not respect that order: most of the nouns name what
flows *between* stages, so a chapter reaches for a word two stages before the
chapter that owns it. This page introduces each of those words once, in plain
terms first and the identifiers second, and says which chapter specifies it.
Read it first; come back when a chapter uses a word you have not met.

Every word below is **defined** — set in bold — on exactly one page, under the
heading its entry links to, and every other page links its first mention of the
word there — except for the four that are also ordinary English words the
chapters need in that sense (declaration, root, part, capability), which keep
only the single definition. [`validate.py`](FORMAT.md#vocabulary) checks both.

## What the source provides

### Source item

A function, type or constant the source crate marked with `#[prebindgen]` —
`Stamp`, `stamp_sum` — as captured and then as the source model describes it.
Most of what a binding declares names one; a binding may also declare things
the source never exported, such as a callback signature or a type like
`String`. — [Capture source items](stages/01-source.md#capture-source-items)

## What the binding asks for

### Declaration

One thing a binding asked to expose: this function, that type, at this foreign
placement — `Stamp` as a C struct, `stamp_sum` as a Kotlin function. [The
report](report.md) accounts for declarations; a source item nothing declared
currently has no entry in it. Identified by a `DeclarationId` built from the source name, so a
foreign rename does not change what a test or a report is talking about. —
[Record binding requests](stages/03-requests.md#record-binding-requests)

### Policy

The target-specific choices recorded with a declaration or a value: `Stamp` as
a by-value C aggregate, this exported symbol, that Kotlin class. Data the
target adapter itself interprets later; the registry only carries it and hands
it back to the adapter that wrote it. Two policies that look alike are still two
— sharing happens by identity, never by comparing them. — [What policy means](stages/03-requests.md#what-policy-means)

### Root

A declaration as the starting point of planning: exposing `stamp_sum` is a
root, and the conversions needed to build its wrapper are its dependencies.
A declared type is a root too, even if no function uses it. —
[Where planning starts](stages/03-requests.md#where-planning-starts)

### Site

A position inside a declared function — parameter 0 of the exported
`stamp_sum`, or its return — which is what an override is recorded against and
what a skip's diagnostic path names. — [A value's position in an exported function](stages/03-requests.md#a-values-position-in-an-exported-function)

### Part

A position inside a source value — the `secs` field of `Stamp`, or (once
constructor relations are built) the one argument of a constructor — as a
relation exposes it. Sites and parts together
are the **positions** planning happens at. — [Record binding requests](stages/03-requests.md#record-binding-requests)

## What the registry plans

### Conversion

One value-shaped problem: turning what a foreign caller passed into the Rust
value a source function takes, or the reverse for its result. Planning
conversions is the pipeline's central work; a conversion knows nothing about
which function it serves. — [Plan value conversions](stages/04-values.md#plan-value-conversions)

### Crossing

What a conversion is *of*: an exact source type together with the direction it
travels — `Stamp` into Rust as a parameter, `i64` out of Rust as a result. Two
crossings of the same type in opposite directions are two different
conversions. — [Finding an existing conversion plan](stages/03-requests.md#finding-an-existing-conversion-plan)

### Relation

A link inside the source domain: from a Rust type to the values it is built from
or read into, with the source-level means of getting between them — `Stamp` to
its two fields, or (planned, not built yet) to the one argument of a
constructor. A type can stand in several; the target adapter selects one, by
`RelationId`. — [What a relation is](stages/04-values.md#what-a-relation-is)

### Representation

The target adapter's answer to what carries a value on the foreign side and how
it is accessed: a `repr(C)` struct read member by member, a JVM object read
through its getters. Described by the adapter, walked by the registry. —
[Plan value conversions](stages/04-values.md#plan-value-conversions)

### Carrier

A value holding conversion data on its way across: a native argument, a return,
or a temporary inside the generated Rust. A carrier has a `WireType`, and only
one flagged as ABI-safe may be a native parameter or return. —
[Describing target values and operations](stages/04-values.md#describing-target-values-and-operations)

### Primitive

One typed operation the target adapter supplies — read this member, call this
getter, throw this error — with what it needs, what it produces and how it can
fail. An operation, not a primitive type. The registry composes primitives; it
never pastes their text together. — [Plan value conversions](stages/04-values.md#plan-value-conversions)

### Node

A planned conversion, kept so that every value crossing the same way shares
it. Its identity is the crossing, the selected relation, the policy's identity
and the children's identities. — [Plan value conversions](stages/04-values.md#plan-value-conversions)

### Artifact

A generated unit of Rust that an operation or a declaration depends on — a
helper function, a `repr(C)` type — named so the registry keeps one copy and
publishes only what a retained output needs. The retention chapter uses the
word more broadly for anything a run writes out; the registry's own artifacts
are Rust. — [Individual target operations](stages/04-values.md#individual-target-operations)

## What comes out

### Wrapper

The native function a foreign caller actually calls: the registry assembles it
around the conversions for one declared function, in the calling convention the
target dictates. — [Assemble the native boundary](stages/05-boundary.md#assemble-the-native-boundary)

### Outcome

What became of a declaration on a run that completed: emitted, skipped, or
ignored because the user said so. Every declaration has exactly one, and [the
report](report.md) lists them all. A run that fails — contradictory configuration, a
violated invariant — produces no outcomes at all. — [Retain supported output](stages/06-retain.md#retain-supported-output)

### Capability

What a skipped declaration waits on: a stable code such as
`unsupported.jni.carrier`, the same across runs, so a report can be diffed and
the next piece of work chosen from it. — [Retain supported output](stages/06-retain.md#retain-supported-output)
