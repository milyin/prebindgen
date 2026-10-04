
The relationship is **flat type ⇄ representation type(s) ⇄ wire type(s)**.
`convert!` describes a directional step through another Rust type; record
helpers construct or decompose several parts. A generator chooses the wire
encoding and destination surface. Input and output are independent choices.

A [`ConversionPlan`] retains three kinds of information:

| Information | Owner | Example |
|---|---|---|
| Wire representation and per-slot meaning | Generator's [`WireType`] enum | A `Long` containing an unsigned integer or a class handle |
| Conversion structure | Shared [`Form`] tree | A record of two fields, each converted through another type |
| Meaning of the whole destination value | Generator's metadata type | A constructor, class, builder, or child metadata tree |

## A complete policy and an occurrence override

This example exposes `u64` as either a number or a duration. Both cross as
`i64`; destination generation learns the selected meaning from the plan,
including the override. The generated element is named by the generator,
without needing a corresponding flat item.

```
use prebindgen_flat::flat::{ScalarKind, TypeRef};
use prebindgen_tools::{
    ConversionPlan, ConversionPolicy, Direction, Input, Output, Overrides,
    Place, Resolver, Scope, Seg, Wire, WireType,
};
use quote::quote;

#[derive(Clone, Debug, PartialEq)]
enum Boundary {
    Long { meaning: Meaning },
}

#[derive(Clone, Debug, PartialEq)]
enum Meaning { Number, Duration }

impl WireType for Boundary {
    fn rust_type(&self) -> syn::Type { syn::parse_quote!(i64) }
}

struct Policy;
impl ConversionPolicy for Policy {
    type Wire = Boundary;
    type Rule = Meaning;
    type Metadata = Meaning;

    fn resolve(
        &self,
        _scope: &Scope<'_, Self>,
        ty: &TypeRef,
        direction: Direction,
        place: &Place,
        rule: Option<&Meaning>,
    ) -> Result<ConversionPlan<Boundary, Meaning>, String> {
        if ty.key() != TypeRef::scalar(ScalarKind::U64).key() {
            return Err(format!("unsupported type `{ty}`"));
        }
        let meaning = rule.cloned().unwrap_or(Meaning::Number);
        let name = place.ident("");
        let wire = Wire::new(name.clone(), Boundary::Long { meaning: meaning.clone() });
        match direction {
            Direction::IntoRust => ConversionPlan::input(
                Input::wire(ty, wire, quote!(#name as u64)), meaning,
            ),
            Direction::OutOfRust => {
                let value = place.ident("source");
                ConversionPlan::output(value.clone(),
                    Output::wire(ty, wire, quote!(#value as i64)), meaning)
            }
        }
    }
}

let ty = TypeRef::scalar(ScalarKind::U64);
let mut resolver = Resolver::new(Policy);
resolver.default(&ty, Direction::IntoRust, Meaning::Number)?;

let count = Place::new("combined").at(Seg::Param("count".into()));
let timeout = Place::new("combined").at(Seg::Param("timeout".into()));
let mut overrides = Overrides::new();
overrides.at(timeout.clone(), &ty, Direction::IntoRust, Meaning::Duration)?;
let scope = resolver.scope("combined", overrides)?;

let count_plan = scope.resolve(&ty, Direction::IntoRust, &count)?;
let timeout_plan = scope.resolve(&ty, Direction::IntoRust, &timeout)?;
assert_eq!(count_plan.metadata(), &Meaning::Number);
assert_eq!(timeout_plan.metadata(), &Meaning::Duration);
assert_eq!(count_plan.wires()[0].ty.rust_type(), timeout_plan.wires()[0].ty.rust_type());

// A destination writer can now choose "ULong" or "Duration" without
// inspecting declarations again. Rust generation uses the same plan.
let call = timeout_plan.with_input(&syn::parse_quote!(__timeout), |value| {
    quote!(source::wait(#value))
})?;
syn::parse2::<syn::Expr>(call)?;

// The occurrence override applies only IntoRust; output uses policy fallback.
let output = scope.resolve(&ty, Direction::OutOfRust, &timeout)?;
assert_eq!(output.metadata(), &Meaning::Number);
scope.finish()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Composing children

Within policy, call `scope.resolve(child_type, direction, child_place)` before
building a record, an intermediate representation, or any other composition.
Use [`Seg::field`] for fields and [`Seg::Repr`] for intermediate types. Rules
apply to exact occurrences; a parent override does not replace child defaults.

Use [`ConversionPlan::into_input`] or [`ConversionPlan::into_output`] to pass
a selected child to an expression combinator. Both return the child's
destination metadata along with its conversion, so policy can retain it in
the parent's metadata. A child with input setup must instead be emitted at
the element level with `with_input`; flattening such setup is deferred.

[`Input::record`] and [`Output::record`] preserve source field order.
[`Stage::decode`] and [`Stage::encode`] compose a resolved `convert!` step
with its representation's conversion. Their endpoints must match. Low-level
combinators describe the Rust side and shared form; the policy builds its
destination metadata from those forms and any child metadata it needs.

## Prototype boundaries

The resolver checks duplicate declarations, expected override types, unused
overrides, plan root type/direction, and conversion cycles. Plan constructors
reject duplicate wire names. Low-level record/stage endpoint violations are
programmer errors and panic. Rust token semantics still require compilation.

The generator owns slot placement (return, out-parameter, callback, field),
resource transfer, error routing, and names across multiple plans. Exact defaults
do not automatically apply to borrowed types. Borrowed values can use root
setup storage via [`ConversionPlan::input_with_setup`]; setup lives through the
body emitted by [`ConversionPlan::with_input`]. Automatic conditional storage,
choice-constructor expansion, recursive forward declarations, and C/JNI
migration are deferred. Sequence helpers currently record an adapter-written
loop rather than choose an encoding for it.
