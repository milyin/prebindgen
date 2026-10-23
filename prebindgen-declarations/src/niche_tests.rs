//! Executable conformance tests for codecs with domains unrelated to C or JNI.
use super::*;
type InputPlan<W> = prebindgen_conversions::InputPlan<W, (), String>;
type OutputPlan<W> = prebindgen_conversions::OutputPlan<W, (), String>;
use prebindgen_flat::WireKind;

// This is intentionally not an enum of primitive sentinel kinds. An adapter may
// retain arbitrary structured metadata and choose its own expression language.
type Predicate = fn(TokenStream, TokenStream) -> TokenStream;
type Case = (Type, Vec<TokenStream>, Predicate, TokenStream);

#[derive(Clone, Debug)]
struct Sentinel {
    value: TokenStream,
    absent: Predicate,
}
#[derive(Clone, Debug)]
struct Domain {
    ty: Type,
    unused: Vec<Sentinel>,
    remaining: Option<usize>,
    prefix: bool,
    valid: TokenStream,
}
impl WireKind for Domain {
    fn rust_type(&self) -> Type {
        self.ty.clone()
    }
}
struct DomainCodec;
impl Codec for DomainCodec {
    type Niche = Sentinel;
    type Group = String;
    fn optional_metadata(
        &self,
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<Sentinel>,
    ) -> String {
        format!("optional:{presence}:{discriminator}:{niche:?}")
    }
    fn field_group(&self, name: syn::Ident) -> String {
        format!("member:{name}")
    }
    fn fields_group(&self) -> String {
        "record".into()
    }
    type Extra = ();
    type Context = ();
    type Wire = Domain;
    type Input = InputPlan<Domain>;
    type Output = OutputPlan<Domain>;
    fn selector(&self, _: &()) -> Domain {
        Domain {
            ty: syn::parse_quote!(i32),
            unused: vec![],
            remaining: None,
            prefix: false,
            valid: quote!(true),
        }
    }
    fn consumes(&self, _: &Type, _: &()) -> bool {
        false
    }
    fn input_leaf(
        &self,
        _: &Type,
        _: &(),
        _: syn::Ident,
        _: syn::Ident,
        _: &Condition,
    ) -> Result<InputPlan<Domain>> {
        unreachable!()
    }
    fn output_leaf(
        &self,
        _: &Type,
        _: &(),
        _: syn::Ident,
        _: TokenStream,
        _: bool,
    ) -> Result<OutputPlan<Domain>> {
        unreachable!()
    }
    fn niche(&self, wire: &Domain) -> Option<Sentinel> {
        wire.remaining.and_then(|i| wire.unused.get(i)).cloned()
    }
    fn set_niche(&self, wire: &mut Domain, niche: Option<Sentinel>) {
        wire.remaining = niche.and_then(|n| {
            wire.unused
                .iter()
                .position(|s| s.value.to_string() == n.value.to_string())
        });
    }
    fn is_present(&self, _: &Domain, n: &Sentinel, v: TokenStream) -> Result<TokenStream> {
        let absent = (n.absent)(v, n.value.clone());
        Ok(quote!(!(#absent)))
    }
    fn absent_value(&self, _: &Domain, n: Option<&Sentinel>, _: bool) -> Result<TokenStream> {
        Ok(n.map(|n| n.value.clone())
            .unwrap_or_else(|| quote!(Default::default())))
    }
    fn remaining_niche(&self, wire: &Domain, n: &Sentinel) -> Option<Sentinel> {
        let index = wire
            .unused
            .iter()
            .position(|s| s.value.to_string() == n.value.to_string())?;
        wire.unused.get(index + 1).cloned()
    }
}
fn input(domain: &Domain, depth: usize, active: Condition) -> InputPlan<Domain> {
    if depth == 0 {
        let check = active.rust();
        let valid = &domain.valid;
        let invalid = domain
            .unused
            .iter()
            .map(|n| (n.absent)(quote!(raw), n.value.clone()));
        let mut wires = vec![Wire {
            name: format_ident!("raw"),
            kind: domain.clone(),
        }];
        let prefix_valid = if domain.prefix {
            wires.insert(
                0,
                Wire {
                    name: format_ident!("prefix"),
                    kind: DomainCodec.selector(&()),
                },
            );
            quote!(prefix == 123)
        } else {
            quote!(true)
        };
        return InputPlan { logical:Default::default(),
            fallible: false,
            context: Default::default(),
            transport: wires.into(),
            validation: quote!(if #check && (!(#prefix_valid) || !(#valid) #(|| #invalid)*) {return Err("invalid payload");}),
            acquire: quote!(),
            resources: vec![],
            acknowledgement: None,
            statements: quote!(),
            argument: quote!(raw),
        }.with_logical();
    }
    let tag = format_ident!("tag{depth}");
    let child = input(domain, depth - 1, active.arm(tag.clone(), 1));
    optional_input(
        &DomainCodec,
        &(),
        tag,
        format_ident!("arg{depth}"),
        &active,
        child,
    )
    .unwrap()
}
fn output(domain: &Domain, depth: usize, source: TokenStream) -> OutputPlan<Domain> {
    if depth == 0 {
        let valid = &domain.valid;
        let invalid = domain
            .unused
            .iter()
            .map(|n| (n.absent)(quote!(raw), n.value.clone()));
        let mut wires = vec![Wire {
            name: format_ident!("raw"),
            kind: domain.clone(),
        }];
        let mut values = vec![quote!(raw)];
        if domain.prefix {
            wires.insert(
                0,
                Wire {
                    name: format_ident!("prefix"),
                    kind: DomainCodec.selector(&()),
                },
            );
            values.insert(0, quote!(123i32));
        }
        return OutputPlan { logical:Default::default(),
            fallible: false,
            context: Default::default(),
            delivery: Default::default(),
            transport: wires.into(),
            statements: quote!(let raw = #source; if !(#valid) #(|| #invalid)* { return Err("invalid payload"); }),
            commit: quote!(),
            values,
        }.with_logical();
    }
    let payload = format_ident!("payload{depth}");
    let child = output(domain, depth - 1, quote!(#payload));
    optional_output(
        &DomainCodec,
        &(),
        format_ident!("tag{depth}"),
        payload,
        source,
        false,
        child,
    )
    .unwrap()
}
fn equal(v: TokenStream, n: TokenStream) -> TokenStream {
    quote!(#v == #n)
}
fn bits(v: TokenStream, n: TokenStream) -> TokenStream {
    quote!((#v).to_bits() == (#n).to_bits())
}
fn nan(v: TokenStream, _: TokenStream) -> TokenStream {
    quote!((#v).is_nan())
}
fn negative(v: TokenStream, _: TokenStream) -> TokenStream {
    quote!(#v < 0)
}

#[test]
fn arbitrary_domains_round_trip_all_nested_states_and_exhaust_to_flags() {
    let cases: Vec<Case> = vec![
        // An enum-like wire with two spare discriminants.
        (
            syn::parse_quote!(u8),
            vec![quote!(3u8), quote!(4u8)],
            equal,
            quote!(2u8),
        ),
        // Values outside i64, including a full-width unsigned sentinel.
        (
            syn::parse_quote!(u128),
            vec![quote!(u128::MAX), quote!(u128::MAX - 1)],
            equal,
            quote!(u128::MAX - 2),
        ),
        // Float domains: one excluded NaN payload, or the entire NaN class.
        (
            syn::parse_quote!(f64),
            vec![quote!(f64::from_bits(0x7ff8000000000042))],
            bits,
            quote!(-0.0f64),
        ),
        (
            syn::parse_quote!(f32),
            vec![quote!(f32::NAN)],
            nan,
            quote!(f32::INFINITY),
        ),
        // An arbitrary predicate covering an entire invalid range.
        (
            syn::parse_quote!(i32),
            vec![quote!(-7i32)],
            negative,
            quote!(0i32),
        ),
        // A structured wire sentinel, not a primitive tag.
        (
            syn::parse_quote!((u64, u64)),
            vec![quote!((u64::MAX, 0u64)), quote!((u64::MAX, 1u64))],
            equal,
            quote!((0u64, u64::MAX)),
        ),
        // Unrestricted floats have no niche, even though NaNs exist.
        (
            syn::parse_quote!(f64),
            vec![],
            equal,
            quote!(f64::NEG_INFINITY),
        ),
    ];
    let mut functions = TokenStream::new();
    let mut checks = TokenStream::new();
    for (case, (ty, values, predicate, payload)) in cases.into_iter().enumerate() {
        for prefix in [false, true] {
            let suffix = usize::from(prefix);
            let domain = Domain {
                ty: ty.clone(),
                prefix,
                valid: if case == 0 {
                    quote!(raw <= 2)
                } else {
                    quote!(true)
                },
                remaining: Some(0),
                unused: values
                    .iter()
                    .cloned()
                    .map(|value| Sentinel {
                        value,
                        absent: predicate,
                    })
                    .collect(),
            };
            for depth in 1..=4 {
                let inp = input(&domain, depth, Condition::default());
                let out = output(&domain, depth, quote!(source));
                let expected = 1 + usize::from(prefix) + depth.saturating_sub(domain.unused.len());
                assert_eq!(inp.transport.len(), expected);
                assert_eq!(out.transport.len(), expected);
                let decode = format_ident!("decode{case}_{depth}_{suffix}");
                let encode = format_ident!("encode{case}_{depth}_{suffix}");
                let mut source_ty = ty.clone();
                for _ in 0..depth {
                    source_ty = syn::parse_quote!(Option<#source_ty>);
                }
                let params = inp.transport.iter().map(|w| {
                    let n = &w.name;
                    let t = w.kind.rust_type();
                    quote!(#n:#t)
                });
                let names = inp.transport.iter().map(|w| &w.name).collect::<Vec<_>>();
                let types = out.transport.iter().map(|w| w.kind.rust_type());
                let validation = inp.validation;
                let construct = inp.statements;
                let argument = inp.argument;
                let prepare = out.statements;
                let commit = out.commit;
                let results = out.values;
                functions.extend(quote!{
                fn #decode(#(#params),*) -> Result<#source_ty,&'static str> {#validation #construct Ok(#argument)}
                fn #encode(source:#source_ty)->Result<(#(#types,)*), &'static str> {#prepare let result=(#(#results,)*);#commit Ok(result)}
            });
                if depth == 1 {
                    let extra = if prefix { quote!(123,) } else { quote!() };
                    for niche in &domain.unused {
                        let value = &niche.value;
                        checks.extend(quote!(assert!(#encode(Some(#value)).is_err());));
                    }
                    if case == 0 {
                        checks.extend(quote!(assert!(#decode(#extra 5).is_err());));
                    }
                    // The next unused representation is not yet a valid optional state.
                    if let Some(next) = domain.unused.get(1) {
                        let value = &next.value;
                        checks.extend(quote!(assert!(#decode(#extra #value).is_err());));
                    }
                    if type_key(&ty) == "f64" {
                        // Preserve another NaN payload even when one NaN pattern is reserved.
                        let value = quote!(f64::from_bits(0x7ff8000000000099));
                        checks.extend(quote!{{let value=#value;let (#(#names,)*)=#encode(Some(value)).unwrap();assert_eq!(#decode(#(#names),*).unwrap().unwrap().to_bits(),value.to_bits());}});
                    }
                }
                let mut states = vec![quote!(None)];
                let mut present = payload.clone();
                for _ in 0..depth {
                    present = quote!(Some(#present));
                }
                states.push(present);
                for level in 1..depth {
                    let mut state = quote!(None);
                    for _ in 0..level {
                        state = quote!(Some(#state));
                    }
                    states.push(state);
                }
                for state in states {
                    checks.extend(quote!{{let source:#source_ty=#state;let (#(#names,)*)=#encode(source).unwrap();assert_eq!(#decode(#(#names),*).unwrap(),source);}});
                }
            }
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("domains.rs");
    let exe = dir.path().join("domains");
    std::fs::write(&file, quote!(#functions fn main(){#checks}).to_string()).unwrap();
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let result = std::process::Command::new(rustc)
        .arg("--edition=2021")
        .arg(&file)
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(std::process::Command::new(exe).status().unwrap().success());
}

#[test]
fn optional_niches_live_on_leaves_and_only_the_discriminator_keeps_one() {
    let domain = Domain {
        ty: syn::parse_quote!(u8),
        remaining: Some(0),
        unused: vec![
            Sentinel {
                value: quote!(3u8),
                absent: equal,
            },
            Sentinel {
                value: quote!(4u8),
                absent: equal,
            },
        ],
        prefix: false,
        valid: quote!(true),
    };
    let mut child = output(&domain, 0, quote!(payload));
    // A second view has the same base domain, but its absent default cannot be
    // treated as an independent discriminator by the enclosing optional layer.
    child.transport.push(Wire {
        name: format_ident!("second"),
        kind: domain.clone(),
    });
    child.values.push(quote!(raw));
    let mut plan = optional_output(
        &DomainCodec,
        &(),
        format_ident!("tag1"),
        format_ident!("payload"),
        quote!(source),
        false,
        child,
    )
    .unwrap();
    assert_eq!(
        DomainCodec
            .niche(&plan.transport[0].kind)
            .unwrap()
            .value
            .to_string(),
        "4u8"
    );
    assert!(DomainCodec.niche(&plan.transport[1].kind).is_none());
    assert_eq!(plan.transport[0].kind.unused.len(), 2); // Wire's base domain is unchanged.
    assert_eq!(plan.transport[1].kind.unused.len(), 2);
    plan = optional_output(
        &DomainCodec,
        &(),
        format_ident!("tag2"),
        format_ident!("outer_payload"),
        quote!(outer_source),
        false,
        plan,
    )
    .unwrap();
    assert_eq!(plan.transport.len(), 2);
    assert!(plan
        .transport
        .iter()
        .all(|w| DomainCodec.niche(&w.kind).is_none()));
    plan = optional_output(
        &DomainCodec,
        &(),
        format_ident!("tag3"),
        format_ident!("outermost_payload"),
        quote!(outermost_source),
        false,
        plan,
    )
    .unwrap();
    assert_eq!(plan.transport.len(), 3); // Exhaustion adds a flag, not a wire wrapper.
    assert_eq!(plan.transport[0].name, "tag3");
    plan.transport.validate().unwrap();
    let prebindgen_conversions::TransportNode::Optional(container) = &plan.transport.nodes()[0]
    else {
        panic!()
    };
    assert!(container
        .metadata
        .as_ref()
        .unwrap()
        .starts_with("optional:tag3:"));
    assert!(
        matches!(&container.representation,prebindgen_conversions::TransportRepresentation::Inline {controls} if controls.len()==1)
    );
    let prebindgen_conversions::TransportNode::Optional(inner) = &container.element.nodes()[0]
    else {
        panic!()
    };
    assert!(inner
        .metadata
        .as_ref()
        .unwrap()
        .starts_with("optional:tag2:"));
    let mut independent = output(&domain, 0, quote!(payload));
    for w in independent.transport.iter_mut() {
        DomainCodec.set_niche(&mut w.kind, None);
    }
    assert!(DomainCodec.niche(&independent.transport[0].kind).is_none());
}
