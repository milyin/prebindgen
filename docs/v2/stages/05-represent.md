<!-- spec: {"kind": "stage", "stage": "05-represent"} -->

[Project contents](../README.md) · Previous: [Select conversion relations](04-select.md) · Next: [Assemble the wrapper boundary](06-boundary.md)

# Represent and compose values

Status: implemented for a value carried whole and for a struct read through its
fields on the way into Rust. A struct leaving Rust, multi-value layouts,
optional values, and the validity and resource contracts are
[described but not built](../extensions.md).

[The previous chapter](04-select.md) walked down from each requested value and
chose, for every type it met, which
[relation](04-select.md#what-a-relation-is) to take. What it left behind is a
selection tree, and this chapter walks back up it. On the way up two things
happen at every position, and they are the second and third of
[the three questions](04-select.md#three-questions-two-chapters): the target says
what carries the value on its side and how that carrier is accessed, and the
registry composes those operations with the already-finished children into the
instructions that move the value. The unit this produces is the one every later
stage reads.

The unit the target supplies is a **primitive**: one typed operation, such as
"read the `secs` member of a `Stamp`" or "call the `getSecs()` getter on this
object with this environment". A primitive describes an operation, not a use of
it — it names no variable and belongs to no exported function — so the same
description can be applied wherever that operation is needed. Along with the
operation, the target states what the operation needs, what it produces,
whether it can fail, and any generated helper it depends on. Those facts let the
registry compose operations safely instead of pasting text together. The
target's whole answer for one value — which carriers hold it, and the
primitives that read or build them — is its **representation**.

The result is a **node**, meaning one reusable [conversion](04-select.md#select-conversion-relations)
plan in the engine's dependency graph. It records the type, direction, selected
relation, child conversions, target representation and instructions. Two
functions taking an owned `Stamp` in the same way can share a node. A borrow
such as `&Stamp` needs a different node, as does a use with different field
conversions or a different
[conversion key](03-requests.md#finding-an-existing-conversion-plan).

## Node identity and the plan graph

The algorithm saves completed plans in a **cache** so it can reuse them. Reuse
requires more than matching the Rust type. Building `Stamp` from two fields is
different from calling a constructor with one integer. Even two field-based
conversions differ if they convert `secs` differently. The cache key therefore
includes the selected relation, the
conversion key the target returned for this value, and the completed
child plans. This is why the cache is consulted on the way up and not on the way
down: the recursion resolves the children before it has that full key. Two
positions that resolve to the same key get the same node.

Nodes and the children they name form the graph that every later stage reads,
and two of its properties are worth stating outright, because the algorithm is
what produces them.

It is **acyclic**, by construction rather than by luck: a node is recorded only
once every child it needs already exists, so an edge out of a node always points
at something complete. Nothing in the graph can point back at a plan still being
built, because such a plan is not in the graph at all — while a conversion is
open the registry holds only [the mark that detects a cycle](04-select.md#refusal-and-cycles),
and that mark is replaced by a node or by an unsupported result. No half-built
node is ever published.

It is a graph and not a tree, because the cache makes equal subplans the same
node. Planning `stamp_sum` produces four edges over three nodes:

```text
   wrapper for stamp_sum
     |
     +-- input  --> node: Stamp, into Rust, Stamp.fields
     |                       |
     |                       +-- part secs  --+
     |                       |                +--> node: i64, into Rust, atomic
     |                       +-- part nanos --+
     |
     +-- output --> node: i64, out of Rust, atomic
```

Both fields converge on one node, applied twice: same type, same direction, same
relation, same conversion key, so the same plan. The two `i64` nodes do not merge,
because direction is part of a node's identity — reading an integer out of a
foreign argument and handing one back are different operations.

The registry walks the struct and combines its child plans. The target answers
local representation questions, such as how to read one member. This division
lets supported nested combinations reuse the traversal algorithm. It does not
make a new kind of value automatically supported: an optional field still needs
[the optional-value protocol](../extensions.md#optional-values), which is not
built.

## Describing target values and operations

The registry knows how source values are composed. It also needs a description
of the values used on the target side and the operations that access them.

The **public surface** is the API foreign users see: C types and functions or
Kotlin classes and methods. The **wire representation** is the set of values
passed through the foreign calling interface, or **ABI** (application binary
interface). A Kotlin `Stamp` object can have two integer arguments as its wire
representation. A **carrier** is a value holding conversion data, either on
that boundary or temporarily inside generated Rust code.

### Individual target operations

A primitive is one typed operation supplied by the target, such as converting a
scalar, reading a JVM property, allocating a handle, or signaling an error. (The
word means an indivisible *operation* here, not a primitive type; a scalar
conversion is one of the things a primitive can do.) The registry schedules
these operations within the complete conversion.

An operation may need a generated helper or type declaration. Each such unit is
an **artifact**. Operation descriptions carry whole `Artifact` values,
identified by name; the registry deduplicates them. Several artifacts can share
one output file.

A described operation has operands, one optional result, a failure, its
dependencies and an implementation. Each operand also has a role: an ordinary
value, an error, or a named runtime context. Carrier descriptions are inline
`WireType { ty, abi }` values, and source types are Flat `TypeRef`s. The
[extended contract](../extensions.md#validity-of-results) adds validity and
resources to this; nothing the engine plans today needs either.

```rust
struct PrimitiveSpec<Payload> {
    operands: Vec<OperandSpec>,    // Ordered input positions, including runtime context.
    result: Option<OperationType>, // The value produced on success, if any.
    failure: PrimitiveFailure,     // Possible failure and the type of error it produces.
    dependencies: Vec<Artifact>,   // Generated helpers/types needed by this operation.
    implementation: Operation<Payload>, // What to render: standard, or the target's own.
}
```

This is the JNI getter that reads `secs` from a `Stamp` object:

```rust
PrimitiveSpec {
    operands: vec![
        // The JNI environment: an operand, not an ambient variable.
        OperandSpec::context("jni.env", OperationType::Carrier(jni_environment), Access::Exclusive),
        // The object whose property is read.
        OperandSpec::value(OperationType::Carrier(stamp_object), Access::Shared),
    ],
    result: Some(OperationType::Carrier(jni_long)),
    // A JVM call can fail, and the error is the jni crate's.
    failure: PrimitiveFailure::Fallible {
        error: OperationType::Carrier(jni_error),
        category: FailureCategory::Runtime,
    },
    dependencies: vec![],
    implementation: Operation::Target(JniPayload::Getter {
        name: "getSecs".into(),
        descriptor: "()J".into(),   // JVM descriptor: no arguments, returns a long
    }),
}
```

Applied to an environment the boundary has named `env` and an object it has
named `stamp`, that one description renders exactly this much Rust:

```rust
env.call_method(&stamp, "getSecs", "()J", &[])
    .and_then(|value| value.j())
```

This expression returns either a JNI integer or a JNI error. `call_method`
invokes the getter, and `.j()` extracts the long result. The adapter renders
that single operation. The common writer supplies the surrounding `let`, branches
on success or failure, and follows the function's error route. For C, reading
the same field is simply `stamp.secs`, which cannot report an error and needs
no JNI environment. The registry combines either kind of read with the same
source-struct construction algorithm.

`implementation` is `Operation<Payload>`: either a **standard** operation the
registry itself renders — the identity conversion, an aggregate member read — or
the adapter's own `Payload`. Reading a C aggregate member is the first, which is
why the C adapter ships no renderer at all and its payload type has no values.
`Payload` is rendering data whose type and interpretation are defined by the
language implementation, such as a JVM property descriptor or a target
operation description. The adapter supplies payload values; the registry stores
those values in the plans and retains the required values in the completed
`Generation`. Language-provided rendering code reads the retained payloads.
The conversion key names the settings that asked for a representation; payload
records the chosen implementation details after planning. Language implementations can use separate payload types
for primitives, representations and foreign declarations.

### The operation specification and its uses

`PrimitiveSpec` describes an operation during generation. The specification
contains no particular caller's object, temporary name or exported function.
The registry can apply that operation in several conversion plans, with
different operands at each use.

The signature describes one application. An **operand** is a value supplied to
the operation; a **result** is a value available after successful execution.
Environment values are operands too, so a property read cannot silently depend
on a variable named `env` in its caller.

```rust
struct OperandSpec {
    ty: OperationType, // Exact type expected at this input position.
    access: Access,    // Whether this application consumes or borrows the input.
    role: OperandRole, // An ordinary value, the error, or a named runtime context.
}

enum OperationType {
    Source(TypeRef),   // An exact Rust type in the source model.
    Carrier(WireType), // A target/runtime type, described below.
}

enum Access {
    Owned,     // The operation may consume/move the value.
    Shared,    // It may read through a shared borrow.
    Exclusive, // It may modify through an exclusive borrow.
}
```

`Access` says what an operation is allowed to do with a value. Read on an operand
it is a demand — this operation will move the value, or only borrow it. Read on a
finished conversion's result, later in this chapter, it is a permission — this is
what a caller may do with what the conversion produced. One vocabulary, two ends
of the same value.

`WireType` describes one carrier type on the target-facing side of a
conversion: a JNI integer, an object reference, the JNI environment, or a
Rust-only intermediate that never leaves the
[wrapper](06-boundary.md#assemble-the-wrapper-boundary). Not every carrier is an
ABI type, so the descriptor states whether this one may appear in an extern
signature. An environment wrapper used inside generated Rust need not itself be
an ABI argument type. `Access` describes the operation's use of its operand and
must agree with the operand's exact Rust type. It does not authorize cloning or
replacing a move with a borrow. The same access vocabulary is used by
[completed conversion plans](#the-conversion-plans-the-registry-builds).

### Failure of an operation

A primitive's failure describes what can go wrong while performing that
operation. The registry needs this information to compose fallible operations
and stop the success path before any unavailable result is used.

```rust
enum PrimitiveFailure {
    Infallible,
    Fallible {
        error: OperationType, // Typed value supplied on the failure path.
        category: FailureCategory, // Selects the containing boundary's error route.
    },
}

enum FailureCategory {
    Domain,  // Error reported by an explicitly selected source operation.
    Binding, // Invalid foreign value or failed representation conversion.
    Runtime, // Failure from target runtime operations, such as JNI access.
}
```

One specification has one error type; an operation with several error variants
uses a typed error enum. The common writer renders a fallible application with
separate success and error paths. The target's operation renderer supplies the
local operation that yields that result. The registry owns the branch, later
conversion calls, cleanup and final return or error-handler invocation. For JNI,
the error contract must also state how pending JVM exceptions are represented
and which recovery operations are legal. Unsupported exception handling skips
the affected binding; the planner cannot assume it is safe to continue calling
JNI methods after a failed read.

Failure is the only contract a described operation carries today. How long a
result stays usable, and what an operation acquires or releases, are the
[validity](../extensions.md#validity-of-results) and
[resource](../extensions.md#runtime-resources) contracts, which nothing the
engine plans yet needs: every result is a value copied out, and every operation
acquires nothing.

### Registry registration and application

The adapter returns primitive specifications with its representation or boundary
description. The registry validates their types, local references and failure
contracts, registers the descriptions, and issues opaque `PrimitiveId`
references. The specification fields above show the stored information; the
implementation keeps validated records private and exposes checked
construction and read-only inspection.

A **primitive application** is an instruction in the registry's conversion or
function body. It names a registered primitive and the already available values
that supply its operands. The registry checks operand types and access,
allocates result identities, and records failure paths. These identities denote
runtime values in a plan; they are neither runtime values nor generated
variable names. Only the common writer chooses the final Rust names.

Contract validation can check the structure and composition of the description.
Target rendering code is still responsible for implementing the declared
operation correctly. Generated Rust compilation and focused runtime tests must
verify that responsibility, including error paths; metadata cannot prove that
arbitrary renderer code obeys its description.

The [C member reads][struct_represent_c] and [JNI property getters][struct_represent_jni]
of the struct path are worked examples of these specifications, down to the
fragment each one renders.

## Target representations

A target representation tells the registry which values carry converted data and
how to access them using the operations above. A **layout** describes the
values contained in a representation. A **protocol** describes the operations
used to read or construct them. The registry supplies the recursion that
converts the corresponding Rust fields.

```rust
enum Layout {
    Scalar(WireType),     // One scalar/reference/carrier value.
    Aggregate {
        ty: WireType,             // Type of the one containing struct carrier.
        members: Vec<syn::Member>, // One named/indexed member per part.
    },
}

struct ReprSpec<Payload> {
    layout: Layout,      // Values that carry this representation.
    protocol: Protocol,  // Operations used to access/construct those values.
    release: Option<PrimitiveSpec>, // How the foreign side gives a value back unconverted, if it owes one.
}

enum Protocol {
    Terminal { codec: PrimitiveSpec },        // Converts the whole value in one operation.
    Product { projections: Vec<PrimitiveSpec> }, // One read per part, in part order.
}
```

Two layouts and two protocols, one direction: convert the whole value, or
project one part per part of the selected relation, reading a struct on the way
into Rust. `Product` has no target-construction operation yet, so a struct
leaving Rust is a reported skip — C reports `unsupported.struct.out_of_rust`
from the registry; JNI refuses earlier with `unsupported.jni.object_output`. A
future C output could use a struct literal, while a future separate-arguments
JNI form would map children to argument slots. The implemented JVM-object input
uses getter operations. These are different target operations around the same
registry algorithm for converting the selected parts.

`release` is what makes a representation a handle. A value the foreign side
holds by address is one it owes back, and the release is the operation that
takes it back without converting it: the typed destructor a C caller or a
Kotlin `free()` calls. A representation the foreign side holds by value names
none. Stated on the into-Rust representation, whose carrier it takes, it is
planned as a wrapper of its own at [the boundary](06-boundary.md#assembling-an-exported-function),
and its presence is what makes the registry require the out-of-Rust direction
too. The three operations a handle is made of — `IntoRaw`, `FromRaw`, `Release`
— are standard ones, because each spells a source type; the target states the
carrier the address is cast to, and [the handle path][typedef_represent] shows
both targets doing exactly that.

A carrier being expressible as a Rust type does not make it safe in an extern
signature; the `abi` flag states whether the adapter permits that use. The C
`repr(C)` declaration is a Rust artifact attached to the public `SurfaceSpec`,
not to the layout. A whole function's calling convention belongs in `AbiSpec`
at [the boundary](06-boundary.md#assembling-an-exported-function).

The protocols carry their operation specifications inline; the registry assigns
operation ids when it composes instructions. Empty and multi-value layouts,
nested member layouts, and the optional, sequence, choice and callable
protocols are [described but not built](../extensions.md#multi-value-layouts).

## The conversion plans the registry builds

A finished node stores its `id`, the
[crossing](03-requests.md#finding-an-existing-conversion-plan), the selected `relation`, the
`repr` the target returned, its `children`, a body, and the failure categories
its operations can raise. The
[full contract](../extensions.md#the-full-value-contract) adds what the
value's permitted use and validity are; the failure categories are the part of
it the engine has.

```rust
struct ValuePlan<Payload> {
    id: NodeId,                  // Reference used by callers of this conversion.
    crossing: Crossing,          // Exact source type and direction being converted.
    relation: Relation,          // The edge the walk took out of this type, as the target chose it.
    repr: ReprSpec<Payload>,     // Target layout and access operations.
    children: Vec<NodeId>,       // One per part of the relation, in part order.
    body: NodeBody,              // Registry-owned structured instructions.
    failures: Vec<FailureCategory>, // What the body's operations can raise.
}
```

`children` is the node's outgoing edge list: it is derived from the resolved
relation, never authored, and it is what a later stage follows to collect
everything a retained conversion needs.

The body is structured instructions, and the common writer renders them as
Rust, allocating temporary names centrally from identities. The instructions
are three, and every value in them is an identity rather than a name:

```rust
enum Instr {
    // Apply a registered operation to values this body already has, binding
    // its result when it produces one.
    Apply { primitive: PrimitiveId, operands: Vec<Operand>, result: Option<ValueId> },
    // Build a source struct from converted parts, in field order.
    Construct { name: String, parts: Vec<ValueId>, result: ValueId },
    // Call the source function, once.
    Call { function: String, args: Vec<ValueId>, result: Option<ValueId> },
}

enum Operand {
    Value(ValueId),    // A value this body has.
    Context(String),   // A runtime scope the boundary supplies under this name.
}
```

A conversion's body is a **template**: one value identity is its carrier — the
value handed in when the conversion is used — and one is its result. Using a
conversion inlines its instructions under the caller's identities, so a
conversion belongs to no function and is still written down once. An identity
conversion is the degenerate template: no instructions, and the result *is* the
carrier, which is why a scalar child renders nothing between a member read and
the construction that uses it.

An operation that needs a runtime scope names it, and the boundary says which
wrapper parameter supplies it; nothing in a rendered fragment can reach for a
variable its caller happens to have.

For the struct, composition is: apply each projection to the carrier, in part
order; use each child's template on what the projection produced; construct the
source struct from the results. The registry generates all child calls and
source traversal. Access follows the exact source type and operation, and for
the owned values this increment produces, ordinary Rust temporaries rely on Rust
destruction at scope exit. A borrowed input such as `&Stamp`, and any value that
holds a handle, need the
[validity and resource contracts](../extensions.md#validity-of-results) before
the registry can schedule the temporary, the cleanup and their order.

## What is not settled here

The scalar and owned-struct cases are implemented, and
[the first increment](../implementation.md#the-first-increment-as-built) records
what building them settled — the instruction set, the standard-operation
variant — and what it left open. The largest open items belong to this chapter,
and [the extension contracts](../extensions.md) hold them: the construction half
of a product, [multi-value layouts](../extensions.md#multi-value-layouts) and
the sequence, choice and callable protocols,
[optional values](../extensions.md#optional-values), and the
[validity](../extensions.md#validity-of-results) and
[resource](../extensions.md#runtime-resources) contracts that `PrimitiveSpec`
does not yet carry because nothing in the increment produces a borrowed or
resource-bearing value.

## Elements at this stage

- [Function taking an owned struct][fn_represent] · [C][fn_represent_c] · [Kotlin/JNI][fn_represent_jni]
- [Struct with scalar fields][struct_represent] · [C][struct_represent_c] · [Kotlin/JNI][struct_represent_jni]
- [Type alias declaring an opaque handle][typedef_represent] · [C][typedef_represent_c] · [Kotlin/JNI][typedef_represent_jni]

[fn_represent]: ../examples/fn/05-represent.md
[fn_represent_c]: ../examples/fn/05-represent.c.md
[fn_represent_jni]: ../examples/fn/05-represent.jni.md
[struct_represent]: ../examples/struct/05-represent.md
[struct_represent_c]: ../examples/struct/05-represent.c.md
[struct_represent_jni]: ../examples/struct/05-represent.jni.md
[typedef_represent]: ../examples/typedef/05-represent.md
[typedef_represent_c]: ../examples/typedef/05-represent.c.md
[typedef_represent_jni]: ../examples/typedef/05-represent.jni.md
