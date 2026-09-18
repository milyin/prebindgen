<!-- spec: {"kind": "stage", "stage": "04-select"} -->

[Project contents](../README.md) · Previous: [Record binding requests](03-requests.md) · Next: [Represent and compose values](05-represent.md)

# Select conversion relations

Status: implemented for what the three element paths need, which is a value
carried whole — a scalar, or an opaque handle — or a struct read through its
fields. The constructor and projector
relations, and the rules that would pin one, are
[described but not built](../extensions.md#constructor-and-projector-relations).

The examples in this chapter use one small source crate — a struct and a function
over it, marked for binding generation:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64;
```

At this point the engine knows the Rust source and the requested foreign API.
It now needs a recipe for moving each value between them. A **plan** is that
recipe, stored as structured data during the build. The plan does not call your
Rust function during generation; it describes operations that generated code
will execute later when a foreign caller uses the binding.

A foreign caller does not call `stamp_sum`. It calls a generated entry point in
its own language: a C function declared in a generated header, taking a
`repr(C)` struct of two integers, or a Kotlin method taking a JVM object.
Behind that entry point is the generated
[wrapper](06-boundary.md#assemble-the-native-boundary), a Rust function whose
ABI the foreign side can reach. It receives what the caller sent, and has to
call `stamp_sum`, whose signature demands an owned `source::Stamp`.

So the wrapper builds that `Stamp` out of what arrived, calls `stamp_sum`, and
turns the returned `i64` into something it can send back. Under the
configuration these chapters use, building it means reading two fields —
members of the C struct, getters on the JVM object — and constructing
`Stamp { secs, nanos }`. That is the selected route, not the only one: a binding
configured to carry `Stamp` whole, as an opaque handle, reads no fields at all.
Each of those value-shaped problems is a **conversion**, and planning one is
the work of this chapter and the next. Ordering the plans around the source
call is [the boundary's](06-boundary.md#assemble-the-native-boundary); here each
value is planned on its own.

## Three questions, two chapters

Three separate questions have to be answered for every conversion, and keeping
them apart is what lets one algorithm serve both languages. Everything here
happens while the binding is being generated, from what the build script
configured; no part of it is decided while the binding runs. Two of the
questions are answered by the **target**: the
[language adapter](../README.md#the-components) for the binding being generated
— `prebindgen-c` or `prebindgen-jni` — in the role it plays facing the engine,
answering for its own language item by item. The third is the registry's alone.

- **On the Rust side, what is this value made of?** A `Stamp` is made of its two
  fields, so obtaining one means obtaining two `i64`s and constructing
  `Stamp { secs, nanos }` from them. An `i64` is made of nothing smaller: it is
  converted whole. Each such answer is a
  [**relation**](#what-a-relation-is) — a link from a Rust type to the Rust
  values it is built from or read into, together with the source-level means of
  getting between them. Relations are pure source-side facts, and the registry
  is the only component that walks one.

  Which relation applies is the target's answer, though, because it follows from
  the configuration. Declared as a C struct or a Kotlin data class, `Stamp` is
  built from its fields. Declared as an opaque handle, it is carried whole and
  its fields are never read. So the registry lists what the source model offers
  for the type — the fields, if the type is a struct, and the whole value in any
  case — and the target names the one its configuration calls for. When the
  configuration calls for something V2 has no lowering for, such as one of the C
  declarators it does not implement yet, the target answers that instead of
  naming a relation, and nothing that needs the value can be generated. Such a
  request fails the build; while V2's coverage is still being completed it is
  [reported as a skipped declaration](07-retain.md#unsupported-requests-and-public-api-dependencies)
  instead, which is a property of the transition rather than of the design.
- **On the foreign side, what carries the value, and how is it accessed?** A
  by-value C struct whose members are read with ordinary field reads, or a JVM
  object whose properties are read by calling `getSecs()` and `getNanos()`
  through JNI. This answer is a
  [representation](05-represent.md#represent-and-compose-values), and only the
  target can give it: nothing in the registry knows what a C struct or a JVM
  object is.
- **How are the pieces put together?** Read each part, convert it, construct the
  Rust value, in that order, stopping if a step fails. This is the registry's
  job, and it is identical in both languages.

Put together, planning one conversion is this recursion — the registry's own
loop, with the two target questions marked:

```text
plan(type, direction, position):
    policy   = effective policy for this position     # site path, else type policy,
                                                      # else run default
    relation = target.select(type, direction, applicable rules, policy)
                                                      # which way out of this type;
                                                      # cheap: no recursion yet
    mark (type, direction, relation, policy) as being resolved
                                                      # meeting this mark again is a cycle

    parts    = the source model's parts of that relation
                   # where that choice leads: the fields of a struct, the
                   # arguments of a constructor; nowhere at all for a scalar
    children = [ plan(part.type, direction, that part's position)
                 for part in parts ]
    if any child is unsupported:
        this conversion is unsupported, and so is everything that needed it

    ---- everything above is this chapter; everything below is the next ----

    if a node exists for (type, direction, relation, policy, children):
        return it                                     # the identity is complete
                                                      # only once the children are

    repr = target.represent(relation, children, policy)
               # which carriers hold the value, and the operations that access them
    body = compose(relation, children, repr)
               # obtain each part, convert it, construct the Rust value — or the
               # reverse, when the direction is out of Rust

    record the node and return it
```

The algorithm is a depth-first walk that starts at a requested type, takes one
outgoing relation per type it reaches, and builds its answer on the way back
up. The line across the middle is the seam between this chapter and the next,
and it is a real seam in the walk: everything above it happens on the way
*down* — the first question is answered for a type before any of its parts is
looked at — and everything below happens on the way *up*, after every part has
been answered. This chapter is the descent. Its result is a **selection tree**:
for every value a request needs, the type, the direction, the relation the
target chose, and under it the same for each part, down to the scalars where
the tree stops. [The next chapter](05-represent.md) fills that tree in from the
leaves upward, with what carries each value and the instructions that move it,
and its result is the [node](05-represent.md#represent-and-compose-values), the
unit the later stages read.

The algorithm receives the value's *position*, not just its type. In the
design vocabulary, a `SiteId` (parameter 0 of this exported function) or a
`PartId` (the `secs` field of this relation) is what an override is recorded
against, so the position is what turns the recorded rules into this
conversion's effective [policy](03-requests.md#what-policy-means). Positions
are how overrides reach a nested child. Current code uses
`Position { declaration, path }` instead of separate `SiteId`/`PartId`
structs. The plan that results is still shared by identity, so two positions
that resolve to the same key get the same node.

For `Stamp` the recursion is one level deep: two `i64` children that need no
work of their own. A struct with a struct field simply makes `plan` call itself
again, and neither adapter learns anything about the nesting — which is the
point. Planning `stamp_sum` selects this:

```text
   Param(0) --> Stamp, into Rust,  relation Stamp.fields
                  +-- part secs  --> i64, into Rust, relation atomic
                  +-- part nanos --> i64, into Rust, relation atomic

   Return   --> i64, out of Rust, relation atomic
```

Nothing here says how a `Stamp` arrives or how `secs` is read; that is the next
chapter's question. What it does say is settled before that question is asked:
`Stamp` will be built from two fields and not from a handle or a constructor
argument, so the representation the target is asked for next is a
representation of *two members*, and an `i64` is a leaf.

### Refusal and cycles

A conversion is planned all the way or not at all. If any child conversion is
unsupported — a field of a type nothing can carry yet — the conversion is
unsupported, and so is every request that needed it: refused with that reason,
never generated in a reduced form. That propagation runs backwards along the
edges of the tree just drawn: one refused leaf refuses every conversion that
reaches it, and then every request that needed one of those. What a refusal
does to the build is
[retention's decision](07-retain.md#unsupported-requests-and-public-api-dependencies).

The descent also has to notice when it is going in circles. A Rust type may
perfectly well be described in terms of itself:

```text
   struct Branch { pub child: Box<Branch>, pub id: i64 }

   plan(Branch, into Rust, Branch.fields)
     +-- part child --> plan(Branch, into Rust, Branch.fields)   <-- still open
```

While a conversion is being planned the registry marks it as being resolved.
Meeting that mark again is a cycle in the source graph, and the registry
answers it with `unsupported.conversion.recursive` rather than descending
forever. The mark is bookkeeping, not a plan: it is replaced by a node or by
an unsupported result, and no later stage sees a conversion that half exists.
Acyclicity of the finished plan graph is therefore something the descent
enforces at this one point, not something the source model guarantees; a
recursive conversion becomes supported by giving that case a real answer, not
by relaxing the check.

## What a relation is

For `normalize(stamp: Stamp) -> Stamp`, the wrapper converts foreign input into
a Rust `Stamp`, calls `normalize`, and converts its result for foreign code.
Here, **value** means a runtime argument, result or field. Input
**construction** can assemble `Stamp { secs, nanos }` from converted fields or
call `stamp_from_millis` with one converted argument. Output **decomposition**
can read fields or call an accessor. A constructor does not automatically
provide an inverse accessor.

A **relation** is a link inside the source domain: from one Rust type to the
Rust values it is built from or read into, together with the source-level means
of getting between them. `Stamp` is related to `(secs: i64, nanos: i64)` by its
fields; it would be related to `(millis: i64)` by `stamp_from_millis`, and to
`StampParts` by an accessor `stamp_parts(&Stamp)`. Each of those is a relation
of `Stamp`, and a type can stand in several at once. Nothing in a relation names
a [carrier](05-represent.md#describing-target-values-and-operations), a wire type, a Kotlin class or a C struct: it is the answer to "how is
the Rust value built or read?", and that answer is the same whichever language
is on the other side.

Relations are links between source types, so they form a graph over those types,
and planning is a walk across it. Drawing that graph exactly is worth the effort,
because three of its properties account for most of the algorithm above.

**A relation links one type to several others at once.** `Stamp` is not related
to `secs` and separately to `nanos`; it is related to the pair, and a conversion
needs both or neither. So the relation is not a single edge but a bundle of them
— a hyperedge — and the edge out to one type is a part of it. Drawing the
relation as a box of its own puts both on the page:

```text
              relations of Stamp        parts of each

    Stamp --+--> [ Stamp.fields ] --+--> i64   (secs,  part 0)
            |                       |
            |                       +--> i64   (nanos, part 1)
            |
            +--> [ atomic ]            (none)

      i64 ------> [ atomic ]            (none)
```

`Relation::Atomic` is the arity-zero case: a bundle of no edges, which is exactly
what makes a scalar a leaf. `Part` is the single edge, and that is why `Part`,
not `Relation`, is what the recursion iterates.

**A type has several outgoing relations, and they are distinct.** `Stamp` above
offers two; a build that declared `stamp_from_millis` as a constructor would
offer a third, landing on one `i64` instead of two. These are parallel
hyperedges out of the same type, so the source graph is a multigraph and its
edges need labels. `RelationId` is that label, which is why it belongs in a
conversion's identity: `Stamp` through its fields and `Stamp` through
`stamp_from_millis` produce different code from the same type and the same
direction, and must never share a node.

**The graph may contain cycles.** Nothing stops a Rust type from being described
in terms of itself, directly or through a ring of types. The acyclicity the plan
graph enjoys is [enforced during the walk](#refusal-and-cycles), by refusing a
conversion that is already open, and is not a property of the source.

As built, the engine has the two relations its element paths need:

```rust
/// How the registry constructs or reads a Rust value.
pub enum Relation {
    /// The whole value converted by one target operation: no parts, no
    /// recursion. An `i64`.
    Atomic,
    /// The struct's fields. `Stamp` is `secs` and `nanos`.
    Struct(StructRelation),
}

pub struct StructRelation {
    /// The struct's declared name, which is how the writer finds its shape
    /// again when it renders a construction.
    pub name: String,
    pub parts: Vec<Part>,
}

/// One field of a struct relation, or one argument of a constructor relation.
pub struct Part {
    pub name: Option<String>,  // `secs`; `None` for a positional field
    pub index: usize,
    pub ty: TypeRef,           // a source type — the part is another Rust value
}
```

`Part::ty` is a Flat `TypeRef`, a source type: every edge of a relation lands on
another Rust value, which the registry plans recursively with the same algorithm.
That is what keeps the walk the registry's and nobody else's — an edge that
landed on a C struct or a Kotlin class would need a different traversal. The
[constructor and projector relations](../extensions.md#constructor-and-projector-relations)
are the same shape with different edges and a source function as the means,
which is why adding one changes nothing under this type.

The important distinction is between the structure Rust declares and the
operation selected for a conversion. `Stamp` always has the same two fields,
but a future constructor-based binding could accept only milliseconds and let
a helper calculate those fields. A relation describes that selected way to
build or read the value, rather than treating the field list as the only option.

## Registering and selecting a relation

Flat stores no links between types — its references are names, resolved on
lookup — so the graph is not a structure the model holds. The registry builds a
type's outgoing edges on demand and keeps them for the run. `Run::candidates`
registers them the first time a type is planned (the atomic relation for every
type; the struct one when the model resolves the name to a struct) and hands the
target the list as `(RelationId, Relation)` pairs. Registering once per type
rather than once per visit is what makes the label stable: a fresh id on every
visit would make every parallel edge unique, and nothing would ever share a
node.

The user does not register a relation for each scalar or field-based struct.
A **struct relation** is implicit: for any struct the source model describes,
the registry registers the relation built from its fields, so `Stamp.fields`
exists without anyone asking for it. A **scalar** has no parts at all — an
`i64` is not built from anything — so its relation is the atomic one, the
whole value converted by a single operation the target supplies. A constructor
or projector relation would be explicit — it names a function, so someone has
to say which — and that declaration, like the rule that would pin a relation
at a position, is
[not built](../extensions.md#constructor-and-projector-relations): a request
carries policies recorded for a type and for a position, and the target
derives its relation from those.

The target answers `select` with a `RelationId`, an index into that run's
table under the same convention as every other `…Id` here — that is, it names
which outgoing edge the walk takes from this type. The id rather than the value
is what travels, because the id is what the cache key compares.

So `select` chooses from the edges leaving that type: the implicit relation,
plus any explicit ones registered for it. Selection precedes child traversal,
which is what lets an atomic opaque representation leave a struct's private
fields uninspected. Child types retain wrappers, references and lifetimes;
cloning needs an explicit operation.

## How the registry asks a target for decisions

The target interface has four planning operations. Each answers a local question
with a description; the registry decides when to ask it. The first, `select`,
is this chapter's; `represent` is [the next chapter's](05-represent.md), and
the other two belong to the stages after that. The sketch below uses the
design's descriptor names. Current `represent` receives `ChildValue` entries
containing a part and layout, not full validity and resource contracts. The
same trait also has `render_operation`, used later during Rust emission.

```rust
trait Target {
    type Policy;  // Configuration choices recorded by the frontend.
    type Payload; // Owned operation/rendering descriptions.

    fn select(
        &self,
        query: SelectionQuery<'_, Self::Policy>, // Source value and applicable choices.
    ) -> TargetSupport<RelationId>; // One of the relations the query offered.

    fn represent(
        &self,
        shape: ResolvedShape<'_>, // Selected source operation and its direct children.
        children: &[ValueDescriptor<Self::Payload>], // Completed child conversion descriptions.
        policy: &Self::Policy,    // Effective choices for this value.
    ) -> TargetSupport<ReprSpec<Self::Payload>>;

    fn boundary(
        &self,
        site: &SiteDescriptor, // Exported call's signature and boundary roles.
        values: &ResolvedValues<Self::Payload>, // Its resolved input/output values.
        policy: &Self::Policy, // Calling and error-delivery choices.
    ) -> TargetSupport<BoundarySpec<Self::Payload>>;

    fn surface(
        &self,
        request: &SurfaceRequest<Self::Policy>, // Public name/placement, policy and promises.
        values: &ResolvedValues<Self::Payload>, // Values needed to describe that public API.
    ) -> TargetSupport<SurfaceSpec<Self::Payload>>;
}
```

Every method answers with `TargetSupport<Answer>`, which is one of three things:
a ready description, a specific unsupported reason, or a fatal planning error
(defined with the other support
[outcomes](07-retain.md#retain-supported-output)). Two of the answers are
specified in later chapters, because they are about later stages:
[`BoundarySpec`](06-boundary.md#assembling-an-exported-function) describes
where native arguments and results go, and
[`SurfaceSpec`](07-retain.md#dependencies-of-public-declarations) describes a
public foreign declaration and what it requires.

The method inputs and results serve different stages:

| Method | Information available | Target's answer | Registry's next job |
| --- | --- | --- | --- |
| `select` | Exact source type/direction, local source facts and applicable conversion rules and policy | The relation chosen for *this* value, and nothing else | Inspect that relation's parts and recursively resolve their conversions. |
| `represent` | `ResolvedShape`: source operation with model-derived child types; `ValueDescriptor`s: completed child layouts and contracts | Representation layout and target operations | Compose the complete value conversion. |
| `boundary` | `SiteDescriptor`: call signature/roles; `ResolvedValues`: its completed value descriptions | Argument placement, result delivery and error actions | Assemble and validate the complete native wrapper. |
| `surface` | `SurfaceRequest`: requested name, placement, policy and promises; required value descriptions | Public declaration description and requirements | Check dependencies before deciding whether to emit it. |

A selection speaks for one value. It cannot declare a choice for a child,
because a child is planned by a recursion that asks the target again, and two
ways to say the same thing would need a precedence rule; a choice for a child is
a conversion rule recorded at the child's position instead.

`select` is on this interface, even though a relation is a source-side
description, because the choice among the available relations depends on how the
target intends to carry the value: a representation that hands out an opaque
handle wants the atomic relation, not the fields. The target chooses; it does not
invent. It picks from the relations registered for that type, and where the
configuration pinned one, it either honours that choice or reports why it cannot.
Walking whatever it picked remains the registry's work.

These views are read-only. The adapter can inspect direct child descriptors but
cannot invoke the registry's recursive compiler or modify the registry's plan
tables. Relations and representations that recur — a struct read through its
members, a scalar carried unchanged — should be available to an adapter as
ready-made descriptions it names rather than builds, so that a new target's
first version is a handful of choices rather than a library. A target that
needs a conversion the selected relation's children do not give it has
[no way to ask for one yet](../extensions.md#requesting-further-conversions).

## Responsibility boundary with Flat

The relation vocabulary consumes what Flat knows about a function and a struct.
The registry assigns conversion roles and validates them against a requested
direction and exact type.

| Flat provides | Registry adds |
| --- | --- |
| Function parameters and complete return type | Whether the function constructs a value or projects one from an input. |
| Struct shape and typed fields | Recursive field conversions and value construction/decomposition. |
| `Result` child types | Whether a selected constructor treats `Ok` as construction success and routes `Err` as failure. |
| Exact reference/wrapper structure and source access facts | Temporary lifetimes, borrow use and ownership in the generated conversion. |
| Stable snapshot association and normalized type keys | Conversion-cache identity including direction, selected relation and target policy. |
| Source locations and unsupported-item descriptions | Binding-specific dependency paths and skipped-output reports. |

The registry and language frontends use Flat independently. Flat has no
conversion-selection policy, target representation, recursive binding planner,
or dependency on the registry. A `FunctionView` has no `as_constructor()`
method; `ConstructorRelation::new(function)` belongs to the registry library.

## What is not settled here

The atomic and struct relations are implemented, and
[the first increment](../implementation.md#the-first-increment-as-built)
records what building them settled. Everything else this chapter names — the
[source views](../extensions.md#source-views) the registry would read through,
the [constructor and projector relations](../extensions.md#constructor-and-projector-relations)
and the conversion rules that would pin one, and a target's
[request for a further conversion](../extensions.md#requesting-further-conversions)
— is described and not built.

## Elements at this stage

- [Function taking an owned struct][fn_select] · [C][fn_select_c] · [Kotlin/JNI][fn_select_jni]
- [Struct with scalar fields][struct_select] · [C][struct_select_c] · [Kotlin/JNI][struct_select_jni]
- [Type alias declaring an opaque handle][typedef_select] · [C][typedef_select_c] · [Kotlin/JNI][typedef_select_jni]

[fn_select]: ../examples/fn/04-select.md
[fn_select_c]: ../examples/fn/04-select.c.md
[fn_select_jni]: ../examples/fn/04-select.jni.md
[struct_select]: ../examples/struct/04-select.md
[struct_select_c]: ../examples/struct/04-select.c.md
[struct_select_jni]: ../examples/struct/04-select.jni.md
[typedef_select]: ../examples/typedef/04-select.md
[typedef_select_c]: ../examples/typedef/04-select.c.md
[typedef_select_jni]: ../examples/typedef/04-select.jni.md
