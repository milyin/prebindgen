<!-- spec: {"kind": "stage", "stage": "05-represent"} -->

[Project contents](../README.md) · Previous: [Select conversion relations](04-select.md) · Next: [Assemble the wrapper boundary](06-boundary.md)

# Represent and compose values

Status: implemented for a value carried whole — a scalar, an opaque handle, a
fieldless enum — and for a struct read through its fields on the way into
Rust. A struct leaving Rust, multi-value layouts, optional values, and the
validity and resource contracts are [described but not built](../extensions.md).

[The previous chapter](04-select.md) walked down from each requested value and
found, for every value it met, how its
[conversion](04-select.md#select-conversion-relations) is to be made: the
[conversion rule](03-requests.md#conversion-rules) that applies and the
[relation](04-select.md#what-a-relation-is) it names. What it left behind is a
selection tree, and this chapter walks back up it. On the way up the registry
composes, at every position, the operations the binding stated with the
already-finished children into the instructions that move the value. The unit
this produces is the one every later stage reads.

What the binding states for a value is its **representation**: which carriers
hold it on the target's side, and the operations that read or convert them —
a C struct read member by member, a JVM object read through its getters, a
handle taken back from its address. The frontend states it when it builds the
binding, because only the language's own crate knows what a C struct or a JVM
object is; the registry holds it, types each of its operations from the plan,
and orders them.

Each of those operations is a **primitive**: one operation, such as "read the
part's member of the aggregate" or "call the part's getter on this object with
this environment". A primitive describes an operation, not a use of it — it
names no variable and belongs to no exported function — so the same
description is applied wherever that operation is needed. It states whether it
can fail, and with what, and which runtime context it needs; what it reads and
produces follows from where the representation uses it. Those facts let the
registry compose operations safely instead of pasting text together.

The result is a **node**, meaning one reusable conversion plan in the engine's
dependency graph. It records the type, direction, the representation that
applied and the relation it names, the child conversions and the
instructions. Two functions taking an owned `Stamp` in the same way share a
node. A borrow such as `&Stamp` needs a different node, as does a use with
different field conversions or a different representation.

## Node identity and the plan graph

The algorithm saves completed plans in a **cache** so it can reuse them. Reuse
requires more than matching the Rust type. Building `Stamp` from two fields is
different from carrying it whole as a handle. Even two field-based conversions
differ if they convert `secs` differently. The cache key therefore includes the
representation's id — equal representations share one, whoever declared them —
and the completed child plans. This is why the cache is consulted on the way up
and not on the way down: the recursion resolves the children before it has
that full key. Two positions that resolve to the same key get the same node.

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
representation, so the same plan. The two `i64` nodes do not merge, because
direction is part of a node's identity — reading an integer out of a foreign
argument and handing one back are different operations.

The registry walks the struct and combines its child plans. The binding states
only what is local: how the aggregate holds its members and how one member is
read. This division lets supported nested combinations reuse the traversal
algorithm. It does not make a new kind of value automatically supported: an
optional field still needs [the optional-value design](../extensions.md#optional-values),
which is not built.

## Describing target values and operations

The registry knows how source values are composed. It also needs a description
of the values used on the target side and the operations that access them.

The **public surface** is the API foreign users see: C types and functions or
Kotlin classes and methods. The **wire representation** is the set of values
passed through the foreign calling interface, or **ABI** (application binary
interface). A **carrier** is a Rust type holding conversion data, either on
that boundary or temporarily inside generated Rust code. The frontend declares
each one as a `WireType`: the Rust type — `i64`, `*mut Ledger`, a `repr(C)`
`Stamp`, a `JObject` — which of the target's few wire types it is, for an
aggregate which wire types its members may be, and metadata only the target's
writers read, such as a C name or a JVM descriptor. Two carriers may share a
Rust type and differ in metadata: a `JObject` holding an `example.Stamp` is a
different carrier from one holding an `other.Stamp`.

### Individual target operations

A primitive is one operation, such as converting a scalar, reading a JVM
property, handing out a handle, or signalling an error. (The word means an
indivisible *operation* here, not a primitive type; a scalar conversion is one
of the things a primitive can do.) The binding states it as an `Operation`:

```rust
struct Operation<Op> {
    implementation: Implementation<Op>, // Standard(StandardOp), or the target's own Op.
    context: Vec<String>,               // Runtime contexts it needs, by name: "jni.env".
    failure: Option<Failure>,           // Its failure category and error type, if it can fail.
}
```

A **standard** operation is one the registry writes itself, because writing it
means naming a source type, which only the registry can do: the identity, a
member read, a handle handed out, taken back or released, a fieldless enum
matched value by value. A target's own operation is one only its language has —
a JVM getter call, a throw — and the target writes it when the registry feeds
it the operands. C has none: every C operation is standard, so C's `Op` type
has no values at all.

An operation's text may need a generated helper, such as the function a JNI
failure route calls to throw. Each such unit is an **artifact**: named, and
emitted once however many operations need it. A target's writer returns the
artifacts its text needs beside the text.

This is the JNI getter that reads a `Stamp` object's properties, as the JNI
frontend states it:

```rust
Operation::target(JniOp::Getter)
    .context("jni.env") // The environment: an operand, not an ambient variable.
    // A JVM call can fail, and the error is the jni crate's.
    .fails(FailureCategory::Runtime, parse_quote!(jni::errors::Error))
```

It names no property and no type. Applied to the part `secs`, to an
environment the [wrapper](06-boundary.md#assemble-the-wrapper-boundary) has
named `env` and to an object it has named `stamp`,
with the part resolved to a `jlong` whose metadata says `J`, the JNI writer is
fed all of that and writes exactly this much Rust:

```rust
env.call_method(&stamp, "getSecs", "()J", &[])
    .and_then(|value| value.j())
```

This expression returns either a JNI integer or a JNI error. `call_method`
invokes the getter, and `.j()` extracts the long result. The target writes that
single operation. The common writer supplies the surrounding `let`, branches on
success or failure, and follows the function's error route. For C, reading the
same field is simply `stamp.secs`, a standard member read, which cannot report
an error and needs no environment. The registry combines either kind of read
with the same source-struct construction algorithm.

### Failure of an operation

An operation's failure describes what can go wrong while performing it. The
registry needs this information to compose fallible operations and stop the
success path before any unavailable result is used.

```rust
struct Failure {
    category: FailureCategory, // Selects the containing wrapper's error route.
    error: syn::Type,          // Typed value supplied on the failure path.
}

enum FailureCategory {
    Domain,  // Error reported by an explicitly selected source operation.
    Binding, // Invalid foreign value or failed representation conversion.
    Runtime, // Failure from target runtime operations, such as JNI access.
}
```

One operation has one error type. The common writer renders a fallible
application with separate success and error paths; the target's writer
supplies only the local expression that yields the result. The registry owns
the branch, later conversion calls, cleanup and final return or error-handler
invocation. A standard operation that can fail — taking a handle back, which
refuses a null address, or reading an enum from a number no value has — fails
in the binding category with a `String` message.

Failure is the only contract a described operation carries today. How long a
result stays usable, and what an operation acquires or releases, are the
[validity](../extensions.md#validity-of-results) and
[resource](../extensions.md#runtime-resources) contracts, which nothing the
engine plans yet needs: every result is a value copied out, and every operation
acquires nothing.

### Registry registration and application

A **primitive application** is an instruction in the registry's conversion or
function body. It names one use of an operation and the already available
values that supply its operands. The registry registers the use with
everything the writer will be fed — the value the operation is applied to and
the carrier or source type it holds, what it produces, the part it serves,
the contexts it asked for — allocates result identities, and records failure
paths. These identities denote runtime values in a plan; they are neither
runtime values nor generated variable names. Only the common writer chooses
the final Rust names.

The registry checks what it can: that a member's carrier is one its aggregate
holds, that every context an operation asks for is supplied, that a route
reports the error type the operation raises. A target's writer is still
responsible for writing the operation correctly. Generated Rust compilation
and focused runtime tests verify that, including error paths.

The [C member reads][struct_represent_c] and [JNI property getters][struct_represent_jni]
of the struct path are worked examples of these operations, down to the
fragment each one writes.

## Target representations

A representation is one of three shapes, each stated once and referred to by
id from every rule and output that uses it:

```rust
enum Representation<Op> {
    // The whole value, one operation each way: a scalar, a handle, a fieldless
    // enum. Each direction has a carrier of its own.
    Terminal {
        into_rust: Option<Codec<Op>>,   // None: never crosses into Rust.
        out_of_rust: Option<Codec<Op>>,
        release: Option<Operation<Op>>, // How the foreign side gives a held value back.
    },
    // The parts of a relation, carried together in one carrier, into Rust.
    Product { via: Via, carrier: CarrierId, read: Operation<Op> },
    // A representation the target does not lower, refused by name.
    Unsupported(Unsupported),
}

struct Codec<Op> {
    carrier: CarrierId,
    operation: Operation<Op>,
}
```

A `Terminal` converts the whole value in one operation. Its two directions may
use different carriers: a C enum leaves Rust as the C enum and arrives as
`MaybeUninit` of it, since C lets an enum variable hold any `int` and a Rust
enum holding a number none of its values has is undefined behaviour before any
match could look at it.

A `Product` reads one part per part of the selected relation, reading a struct
on the way into Rust. It has no construction operation, so a struct leaving
Rust is a reported skip, `unsupported.struct.out_of_rust`. Its carrier must
have members, and states which wire types they may be: C's aggregate holds the
scalar and nothing else yet, a JVM object's getters read a `long`. A part that
resolves to anything else refuses the struct where the member is.

`release` is what makes a representation a handle. A value the foreign side
holds by address is one it owes back, and the release is the operation that
takes it back without converting it: the typed destructor a C caller or a
Kotlin `free()` calls. A representation the foreign side holds by value names
none. A type output exposing a representation with a release exports it as a
wrapper of its own at [the boundary](06-boundary.md#assembling-an-exported-function),
under the form the output names for it, and the registry plans the out-of-Rust
direction too. The three operations a handle is made of — `IntoRaw`, `FromRaw`,
`Release` — are standard ones, because each spells a source type; the carrier
the address is cast to is the representation's, and
[the handle path][typedef_represent] shows both targets doing exactly that.

Empty and multi-value layouts, and the optional, sequence, choice and callable
shapes, are [described but not built](../extensions.md).

## The conversion plans the registry builds

A finished node stores its `id`, the
[crossing](03-requests.md#finding-an-existing-conversion-plan), the relation
its representation named, the representation and the carrier it crosses in,
its `children`, a body, and the failure categories its operations can raise.
The [full contract](../extensions.md#the-full-value-contract) adds what the
value's permitted use and validity are; the failure categories are the part of
it the engine has.

```rust
struct ValuePlan {
    id: NodeId,                  // Reference used by callers of this conversion.
    crossing: Crossing,          // Exact source type and direction being converted.
    relation: Relation,          // The edge the walk took out of this type.
    representation: ReprId,      // The representation that applied.
    carrier: CarrierId,          // The carrier it crosses in, in this direction.
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
    Context(String),   // A runtime scope the wrapper supplies under this name.
}
```

A conversion's body is a **template**: one value identity is its carrier — the
value handed in when the conversion is used — and one is its result. Using a
conversion inlines its instructions under the caller's identities, so a
conversion belongs to no function and is still written down once. An identity
conversion is the degenerate template: no instructions, and the result *is* the
carrier, which is why a scalar child renders nothing between a member read and
the construction that uses it.

An operation that needs a runtime scope names it, and the function form says
which wrapper parameter supplies it; nothing in a written fragment can reach for
a variable its caller happens to have.

For the struct, composition is: apply the `read` to the carrier once per part,
in part order; use each child's template on what the read produced; construct
the source struct from the results. The registry generates all child calls and
source traversal. For the owned values this increment produces, ordinary Rust
temporaries rely on Rust destruction at scope exit. A borrowed input such as
`&Stamp`, and any value that holds a handle, need the
[validity and resource contracts](../extensions.md#validity-of-results) before
the registry can schedule the temporary, the cleanup and their order.

## What is not settled here

The scalar, owned-struct, handle and fieldless-enum cases are implemented, and
[the first increment](../implementation.md#the-first-increment-as-built) records
what building them settled — the instruction set, the standard operations — and
what it left open. The largest open items belong to this chapter, and
[the extension contracts](../extensions.md) hold them: a struct's construction
out of Rust, [multi-value layouts](../extensions.md#multi-value-layouts), the
[containers](../extensions.md#containers) that would carry generic types,
[optional values](../extensions.md#optional-values), and the
[validity](../extensions.md#validity-of-results) and
[resource](../extensions.md#runtime-resources) contracts that an operation does
not yet carry because nothing in the increment produces a borrowed or
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
