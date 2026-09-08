<!-- spec: {"contract": "source-relations", "kind": "contract", "stage": "04-values"} -->

[Pipeline chapter](../04-values.md) · [Project contents](../../README.md)

# Source construction and decomposition

Status: proposed design. API sketches describe intended contracts, not implemented functionality.

## 4. Describing source construction and decomposition

For `normalize(stamp: Stamp) -> Stamp`, the wrapper converts foreign input into a Rust `Stamp`, calls `normalize`, and converts its result for foreign code. Here, **value** means a runtime argument, result or field. Input **construction** can assemble `Stamp { secs, nanos }` from converted fields or call `stamp_from_millis` with one converted argument. Output **decomposition** can read fields or call an accessor. A constructor does not automatically provide an inverse accessor.

A **relation** is the registry's description of how to construct or read a Rust value for a conversion. The same function can be exported directly, selected as a constructor, or used to extract another value. Those are binding roles, so relation construction belongs in the common registry library. Flat supplies the checked source facts used to validate those roles.

### Flat provides neutral source views

The [Flat V2 project](../02-flat/flat-overview.md) defines a source-inspection API useful independently of binding generation. Function lookup returns a checked `FunctionView` directly; parameter, result and field access retain the same immutable source model. The registry and frontend use this API independently.

```rust
impl Flat {
    pub fn function(&self, name: &str) -> Option<FunctionView>;
}
impl FunctionView {
    pub fn parameters(&self) -> impl Iterator<Item = ParameterView>;
    pub fn return_type(&self) -> TypeView;
}
impl TypeView {
    pub fn as_record(&self) -> Option<RecordView>;
    pub fn referent(&self) -> Option<TypeView>;
}
impl RecordView {
    pub fn fields(&self) -> impl Iterator<Item = FieldView>;
}
```

`ParameterView` and `FieldView` provide their exact `TypeView`s. Flat creates the views and keeps their constructors and storage indices private. A record view exposes structural fields only when Flat models them. Reaching a record through `&Stamp` requires an explicit `referent()` step; that inspection does not itself implement a borrow conversion. Details of [storage, model checks](../02-flat/flat-model.md#private-storage-and-model-consistency) and [incremental adoption](../02-flat/flat-implementation.md#implementation-sequence-and-acceptance) belong to the Flat project.

For `parse_stamp(&str) -> Result<Stamp, Error>`, Flat reports one parameter and a `Result` return type. Whether `Ok` means successful construction is decided by the registry when that function is selected as a constructor.

### The registry validates conversion roles

Registry-library constructors accept checked Flat views. The following prototypes and private internals show the first three roles:

```rust
impl RecordRelation {
    pub fn new(record: RecordView) -> Self;
}
impl ConstructorRelation {
    pub fn new(function: FunctionView) -> Result<Self, RelationError>;
}
impl ProjectionRelation {
    pub fn new(function: FunctionView) -> Result<Self, RelationError>;
}

pub struct RecordRelation {
    record: RecordView, // Derive subject and fields from the checked record.
}
pub struct ConstructorRelation {
    function: FunctionView,      // Derive arguments from its signature.
    output: ConstructionOutput, // Checked interpretation of its return type.
}
enum ConstructionOutput {
    Value(TypeView), // Constructed type.
    Fallible { constructed: TypeView, error: TypeView },
}
pub struct ProjectionRelation {
    function: FunctionView, // Initially requires exactly one source argument.
    input: TypeView,        // Derived exact argument type: T, &T, or &mut T.
    output: TypeView,       // Derived complete return type, including wrappers.
}
```

All relation fields are private. Read-only accessors expose source types and arguments. A constructor's subject is derived from its result; a projector's subject/access requirements come from its input. Callers cannot pair an arbitrary subject with an unrelated operation. For `stamp_from_millis(i64) -> Stamp`, the constructor has one `i64` argument and produces `Stamp`, regardless of `Stamp`'s fields. For a helper `stamp_parts(&Stamp) -> StampParts`, where `StampParts` is a named record with `secs: i64` and `nanos: i64` fields, projection requires a shared borrow and produces that record.

`RelationError` reports a source function incompatible with the requested role. Validation establishes the role's internal consistency, not that every target supports it. The registry separately checks a selected role against the conversion's expected type, direction and ownership: producing `Stamp` alone does not satisfy `&Stamp` without a supported temporary-and-borrow step. Default fallible construction interprets `Result::Ok` as the value and `Err` as failure; a different treatment requires an explicit conversion role.

### Registering and selecting a relation

```rust
pub enum Relation {
    Record(RecordRelation),
    Construct(ConstructorRelation),
    Project(ProjectionRelation),
    // Later: checked atomic, optional, sequence, variant, Rust-to-Rust
    // conversion, representation-reuse and callable descriptions.
}

struct RelationDefinition {
    operation: Relation, // Checked role; no independently assignable subject field.
}
struct RelationId { /* private table identity and entry ID */ }
struct Selection {
    relation: RelationId, // Registered RelationDefinition containing the checked role.
    policy: PolicyId,     // Target representation settings.
}
struct ConversionRules {
    sites: Map<SiteId, Selection>, // Parameter/result overrides.
    parts: Map<PartId, Selection>, // Field/helper-argument selections.
    defaults: DefaultRules,       // Frontend-defined override precedence.
}
```

Registry-library registration accepts checked relations and issues opaque `RelationId`s; request import validates their model and table context. Frontends may construct these descriptions without running recursive conversion planning. The earlier label `Stamp.fields` denotes a registered `Relation::Record` backed by the `Stamp` record view.

The registry follows `Selection.relation` to the checked operation, obtains its fields or helper arguments, and recursively plans their conversions. Projection calls its helper once and processes the saved result through the conversion rules. The adapter describes foreign representations; the registry assembles the source-side instructions executed by the wrapper.

Selection precedes child traversal: an atomic opaque representation does not inspect unused private fields. Helper arguments need not resemble fields. Child types retain wrappers, references and lifetimes; cloning needs an explicit operation. Callback arguments reverse direction. Future roles follow the same checked-construction pattern and produce unsupported outcomes until implemented.

## Responsibility boundary with Flat

The [registry's relation API](#4-describing-source-construction-and-decomposition)
consumes `FunctionView` and `RecordView`. The registry assigns conversion roles
and validates them against a requested direction and exact `TypeView`.

| Flat provides | Registry adds |
| --- | --- |
| Function parameters and complete return type | Whether the function constructs a value or projects one from an input. |
| Record shape and typed fields | Recursive field conversions and value construction/decomposition. |
| `Result` child types | Whether a selected constructor treats `Ok` as construction success and routes `Err` as failure. |
| Exact reference/wrapper structure and source access facts | Temporary lifetimes, borrow use and ownership in the generated conversion. |
| Stable snapshot association and normalized type keys | Conversion-cache identity including direction, selected relation and target policy. |
| Source locations and unsupported-item descriptions | Binding-specific dependency paths and skipped-output reports. |

The registry and language frontends use Flat independently. Flat has no
conversion-selection policy, target representation, recursive binding planner,
or dependency on the registry. A `FunctionView` has no `as_constructor()`
method; `ConstructorRelation::new(function)` belongs to the registry library.
