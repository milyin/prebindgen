<!-- spec: {"kind": "stage", "stage": "04-select"} -->

[Project contents](../README.md) · Previous: [Record binding requests](03-requests.md) · Next: [Represent and compose values](05-represent.md)

# Select conversion relations

Status: implemented for what the three element paths need, which is a value
carried whole — a scalar, or an opaque handle — or a struct read through its
fields. The constructor and projector
relations, and the `Via` values that would pin one, are
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
[wrapper](06-boundary.md#assemble-the-wrapper-boundary), a Rust function whose
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
call is [the boundary's](06-boundary.md#assemble-the-wrapper-boundary); here each
value is planned on its own.

## Three questions, two chapters

Three separate questions have to be answered for every conversion, and keeping
them apart is what lets one algorithm serve both languages. Everything here
happens while the binding is being generated, from what the build script
configured; no part of it is decided while the binding runs. The binding
answered the first two when it was built, in the rules its frontend recorded,
and the registry looks the answers up. The third is the registry's alone. The
**target** — the [language adapter](../README.md#the-components) for the
binding being generated, `prebindgen-c` or `prebindgen-jni`, in the role it
plays facing the engine — answers none of them: it writes the code once they
are answered.

- **On the Rust side, what is this value made of?** A `Stamp` is made of its two
  fields, so obtaining one means obtaining two `i64`s and constructing
  `Stamp { secs, nanos }` from them. An `i64` is made of nothing smaller: it is
  converted whole. Each such answer is a
  [**relation**](#what-a-relation-is) — a link from a Rust type to the Rust
  values it is built from or read into, together with the source-level means of
  getting between them. Relations are pure source-side facts, and the registry
  is the only component that walks one.

  Which relation applies follows from the configuration. Declared as a C
  struct or a Kotlin data class, `Stamp` is built from its fields. Declared as
  an opaque handle, it is carried whole and its fields are never read. The
  frontend knows which when it records the declaration, so it records the
  relation with it: the
  [conversion rule](03-requests.md#conversion-rules) for `Stamp` holds a
  `Product` [representation](05-represent.md#represent-and-compose-values)
  through `Via::Fields`, or a `Terminal` one, which is the atomic relation.
  The registry finds the rule that applies to this value, resolves the
  relation against the source model, and walks the result. When a rule asks
  for something V2 has no lowering for, such as one of the C declarators it
  does not implement yet, its representation is `Unsupported` with the
  reason, and nothing that needs the value can be generated. Such a request
  fails the build; while V2's coverage is still being completed it is
  [reported as a skipped declaration](07-retain.md#unsupported-requests-and-public-api-dependencies)
  instead, which is a property of the transition rather than of the design.
- **On the foreign side, what carries the value, and how is it accessed?** A
  by-value C struct whose members are read with ordinary field reads, or a JVM
  object whose properties are read by calling `getSecs()` and `getNanos()`
  through JNI. This answer is the rest of the same representation: the
  [wire type](05-represent.md#describing-target-values-and-operations), and the
  operations that read it or convert it. The frontend states it, because only the
  language's own crate knows what a C struct or a JVM object is; the registry
  holds it, types it and orders it without needing to know.
- **How are the pieces put together?** Read each part, convert it, construct the
  Rust value, in that order, stopping if a step fails. This is the registry's
  job, and it is identical in both languages.

Put together, planning one conversion is this recursion — the registry's own
loop, which calls no target code:

```text
plan(type, direction, position):
    repr = rules.at(position)                         # a rule for this one value,
        or rules.for_type(type)                       # else one for every value of the type,
        or refuse: unsupported.conversion.no_rule     # cheap: a lookup, no recursion yet
    relation = the relation of type that repr names   # Terminal: atomic; Product: its Via
    mark (type, direction, repr) as being resolved
                                                      # meeting this mark again is a cycle

    parts    = the source model's parts of that relation
                   # where that choice leads: the fields of a struct, the
                   # arguments of a constructor; nowhere at all for a scalar
    children = [ plan(part.type, direction, that part's position)
                 for part in parts ]
    if any child is unsupported:
        this conversion is unsupported, and so is everything that needed it

    ---- everything above is this chapter; everything below is the next ----

    if a node exists for (type, direction, repr, children):
        return it                                     # the identity is complete
                                                      # only once the children are

    body = compose(relation, children, repr)
               # obtain each part, convert it, construct the Rust value — or the
               # reverse, when the direction is out of Rust — typing each of
               # repr's operations from the wire types the children resolved to

    record the node and return it
```

The algorithm is a depth-first walk that starts at a requested type, takes one
outgoing relation per type it reaches, and builds its answer on the way back
up. The line across the middle is the seam between this chapter and the next,
and it is a real seam in the walk: everything above it happens on the way
*down* — a type's rule is found before any of its parts is looked at — and
everything below happens on the way *up*, after every part has been answered.
This chapter is the descent. Its result is a **selection tree**: for every
value a request needs, the type, the direction, the representation that
applies and the relation it names, and under it the same for each part, down
to the scalars where the tree stops. [The next chapter](05-represent.md) fills
that tree in from the leaves upward, with the instructions that move each
value, and its result is the
[node](05-represent.md#represent-and-compose-values), the unit the later
stages read.

The algorithm receives the value's *position*, not just its type: the output
it is reached from and the [value path](03-requests.md#conversion-rules) from
that output's root to the value, such as `param stamp` or
`param stamp.field secs`. The position is what a rule at one
[site](03-requests.md#a-values-position-in-an-exported-function) or
[part](03-requests.md#fields-constructor-arguments-and-enum-variants) is
looked up by, so it is how an override reaches a nested child. No target
method sees a position. The lookup is the *only* step that reads one, which is
deliberate: every later question is asked about a conversion's identity rather
than about one of the places it is used, so a rule recorded at a position has
to take effect here, as a different conversion. The plan that results is
shared by identity, so two positions whose representation and children
match get the same node.

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

The tree does not yet say which instructions read `secs` or build `Stamp`;
putting them together is the next chapter's work. What it does settle is
that `Stamp` will be built from two fields and not from a handle or a
constructor argument, that each field's wire type is the `i64` rule's, and that
an `i64` is a leaf. The `Stamp` wire type's members are therefore known before
anything is written: two, each carried as its field's rule says.

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
a [wire type](05-represent.md#describing-target-values-and-operations), a wire type, a Kotlin class or a C struct: it is the answer to "how is
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

As built, the engine has the three relations its element paths need:

```rust
/// How the registry constructs or reads a Rust value.
pub enum Relation {
    /// The whole value converted by one target operation: no parts, no
    /// recursion. An `i64`.
    Atomic,
    /// The struct's fields. `Stamp` is `secs` and `nanos`.
    Struct(StructRelation),
    /// A callback's arguments, in order. `impl Fn(i64)` is one `i64`.
    Callback(Vec<Part>),
}

pub struct StructRelation {
    /// The struct's declared name, which is how the writer finds its shape
    /// again when it renders a construction.
    pub name: String,
    pub parts: Vec<Part>,
}

/// One field of a struct relation, or one argument of a callback or a
/// constructor relation.
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
type's outgoing edges on demand and keeps them for the run: the first time a
type is planned it registers the atomic relation, and the struct relation when
the model resolves the name to a struct. Registering once per type rather than
once per visit is what makes a `RelationId` stable: a fresh id on every visit
would make every parallel edge unique, and nothing would ever share a node.
`RelationId` is private to the registry. No target sees one.

A callback's relation is implicit too: an `impl Fn(A, B)` is related to its
arguments, positional parts with no names. It is the one relation whose parts
cross the other way from the value: Rust receives the callable and hands the
arguments to it, so a callback entering Rust plans each argument leaving Rust.
The direction follows from the relation, not from anything the binding says per
argument.

The user does not register a relation for each scalar or field-based struct.
A **struct relation** is implicit: for any struct the source model describes,
the registry registers the relation built from its fields, so `Stamp.fields`
exists without anyone asking for it. A **scalar** has no parts at all — an
`i64` is not built from anything — so its relation is the atomic one, the
whole value converted by a single operation the target supplies.

Which of a type's relations a value takes is not chosen by anyone at planning
time. The representation that applies to the value states it:

```rust
/// Which relation of its type a `Product` representation reads a value through.
pub enum Via {
    /// The struct relation: one part per field.
    Fields,
}
```

A `Terminal` representation is the atomic relation of any type. A `Callback`
representation is the callback relation of an `impl Fn(..)`, and is refused
as `unsupported.type.not_a_callback` on any other type. A `Product`
through `Via::Fields` is the struct relation of a type the model describes as
a struct. `Fields` on anything else — an extern, a type the binding declared
although the source never exported it — has no relation to resolve to, and
the value is refused as `unsupported.type.not_a_struct`. That refusal is a
fact about the model, so every target reports it in the same words.

The representation comes from the most specific
[conversion rule](03-requests.md#conversion-rules) that covers the value, and
that rule is used whole:

1. the rule recorded at this value's position;
2. the rule recorded for this value's type.

This order is the registry's, and it is the only precedence there is: both
frontends need exactly this one, and a frontend with an override that should
lose to a type rule has recorded it at the wrong scope. A value neither
covers is refused as `unsupported.conversion.no_rule`. The lookup runs no
target code, so selection is a pure function of the model and the binding.

Selection precedes child traversal, which is what lets an atomic opaque
representation leave a struct's private fields uninspected. Child types retain
wrappers, references and lifetimes; cloning needs an explicit operation.

## What the target writes

The target knows how to write its language, and nothing else. The registry
decides what to write: which wire types a wrapper takes, which operations run in
which order, which declarations a public API needs. It then calls the target
with everything a piece of text needs already worked out — a **feed**. A
writer cannot refuse and cannot change the plan: whatever could make a value
unsupported was decided when the binding was built, and the plan is complete
before the first writer runs.

```rust
pub trait Target: Sized {
    const NAME: &'static str;

    /// The adapter's wire types: one variant per kind, holding what only that
    /// kind needs. C: a declared type's name. JNI: a Kotlin class, from which
    /// the JVM descriptor follows.
    type WireType: WireType;
    /// The kinds a wrapper can take and return. Every kind by default.
    const PARAMS: &'static [WireKindOf<Self>] = <WireKindOf<Self> as WireKind>::ALL;
    const RETURNS: &'static [WireKindOf<Self>] = <WireKindOf<Self> as WireKind>::ALL;
    /// The target's own operations. C: calling through a callback's closure
    /// struct. JNI: a getter call, the two ways a failure is reported, and a
    /// callback's capture, call and report.
    type Op: Clone + Eq + Hash + Debug;
    /// What only the foreign writer reads about an output. C: nothing. JNI: a
    /// Kotlin package and name.
    type OutputMeta: Clone + Eq + Hash + Debug;

    /// One of the target's operations, as one Rust expression.
    fn write_operation(&self, op: &Self::Op, feed: &OperationFeed<'_, Self>) -> Written;

    /// The Rust items a wire type needs declared: a `repr(C)` struct or enum
    /// mirror, an incomplete type behind a pointer. None for a type Rust
    /// already has, such as `i64` or `JObject`.
    fn write_wire_type(&self, feed: &WireTypeFeed<'_, Self>) -> Vec<TokenStream>;
}

/// What a writer produced, and the helpers it needs emitted once beside it.
pub struct Written {
    pub text: TokenStream,
    pub helpers: Vec<Artifact>,
}

pub struct OperationFeed<'a, T: Target> {
    /// The value the operation is applied to, already named, and what it holds.
    pub value: Option<(syn::Ident, Fed<'a, T>)>,
    /// The runtime contexts it asked for, each bound to the wrapper parameter
    /// supplying it.
    pub contexts: Vec<(String, syn::Ident)>,
    /// The error a reporting operation reports.
    pub error: Option<syn::Ident>,
    /// What the expression must produce.
    pub result: Option<Fed<'a, T>>,
    /// For an operation applied per part, such as a `Product`'s `read`: which part.
    pub part: Option<&'a Part>,
    /// For a callback's `capture` and `invoke`: the arguments' wire types,
    /// each named for `invoke`, which is handed them.
    pub args: Vec<(Option<syn::Ident>, &'a T::WireType)>,
}

pub enum Fed<'a, T: Target> {
    Source(&'a TypeRef),
    WireType(&'a T::WireType),
    Captured,               // What a callback's `capture` produced.
}

pub struct WireTypeFeed<'a, T: Target> {
    pub wire_type: &'a T::WireType,
    /// For the wire type of a `Product` or a `Callback`: each part, with the
    /// wire type it resolved to.
    pub parts: Vec<(&'a Part, &'a T::WireType)>,
    /// For a wire type of a fieldless enum's value: that enum, from the model.
    pub unit: Option<&'a Enum>,
}
```

The JNI getter shows the split. The binding stated
`read: Operation::Target(JniOp::Getter)` for `Stamp`; `JniOp::Getter` carries
no name and no type. To read `secs`, the registry calls `write_operation`
with the object operand and its `example.Stamp` wire type, the `env` context,
the part `secs`, and the result wire type the `i64` rule resolved to —
`JniWireType::Long`, whose descriptor is `J`. The JNI writer turns the part's name into `getSecs`
by the Kotlin convention and the result's descriptor into `()J`, and
writes `env.call_method(&stamp, "getSecs", "()J", &[]).and_then(|value| value.j())`.
Nothing in the feed was decided by the writer, and nothing the writer produced
changes what the registry planned.

C writes even less. Every C conversion is a standard operation the registry
writes itself — a member read, a pointer cast, a match over an enum's values —
and C's one operation of its own is calling through a callback's closure
struct. What C writes is mostly wire types: fed the `Stamp` wire type and its
two parts, each with its resolved `i64` wire type, it writes
`#[repr(C)] pub struct Stamp { pub secs: i64, pub nanos: i64 }`, which
cbindgen then turns into the header's declaration.

What each stage feeds:

| Stage | The registry decides | The target writes |
| --- | --- | --- |
| Selection (this chapter) | Which representation, relation and wire type each value has | Nothing |
| [Composition](05-represent.md) | Which operations run in what order, and each operand's and result's wire type | Each target operation, as an expression |
| [The wrapper](06-boundary.md) | The wrapper's parameters and return from the function form and the resolved wire types, and a route per failure category | Each reporting operation a route names |
| [Retention](07-retain.md) | Which outputs survive: those whose values' types are exposed by surviving outputs | Nothing |
| [Emission](08-emit.md) | The wire types of every retained type output, each with its resolved members | Each wire type's declaration |

The foreign declarations are not a call the registry makes. The JNI
frontend's Kotlin writer reads the finished generation — the retained
outputs in order, the values planned for each, and the binding's metadata —
and C has none to write, since cbindgen derives the header from the Rust.

Of a wire type, the registry reads only its kind and its Rust type; the rest
of what a variant holds — a C name, a Kotlin class — is the writers'. The
registry compares whole wire types for equality, because two of one Rust type
— two `JObject`s of different classes — are different wire types.

A conversion speaks for one value. It cannot state a choice for a child,
because a child is looked up again at its own position, and two ways to say
the same thing would need a second precedence rule; a choice for a child is a
conversion rule recorded at the child's position instead.

These feeds are read-only views over the completed plan. A writer cannot
invoke the registry's recursive compiler or modify the plan tables. Common
representations — a struct read through its members, a scalar carried
unchanged — are built from standard operations the registry writes itself,
so a new target's first version is a table of wire types and rules rather than
a library. A target that needs a conversion the selected relation's children
do not give it has
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
| Stable snapshot association and normalized type keys | Conversion-cache identity including direction and the representation that applied. |
| Source locations and unsupported-item descriptions | Binding-specific dependency paths and skipped-output reports. |

The registry and language frontends use Flat independently. Flat makes no
conversion choice, has no target representation, no recursive binding planner,
and no dependency on the registry. A `FunctionView` has no `as_constructor()`
method; `ConstructorRelation::new(function)` belongs to the registry library.

## What is not settled here

The atomic, struct and callback relations are implemented, and
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
- [Function taking a callback][fn_callback_select] · [C][fn_callback_select_c] · [Kotlin/JNI][fn_callback_select_jni]

[fn_select]: ../examples/fn/04-select.md
[fn_select_c]: ../examples/fn/04-select.c.md
[fn_select_jni]: ../examples/fn/04-select.jni.md
[struct_select]: ../examples/struct/04-select.md
[struct_select_c]: ../examples/struct/04-select.c.md
[struct_select_jni]: ../examples/struct/04-select.jni.md
[typedef_select]: ../examples/typedef/04-select.md
[typedef_select_c]: ../examples/typedef/04-select.c.md
[typedef_select_jni]: ../examples/typedef/04-select.jni.md
[fn_callback_select]: ../examples/fn_callback/04-select.md
[fn_callback_select_c]: ../examples/fn_callback/04-select.c.md
[fn_callback_select_jni]: ../examples/fn_callback/04-select.jni.md
