//! The C binding under the v2 engine.
//!
//! [`CbindgenBuilder`] keeps accumulating declarations exactly as it does for
//! v1 — same builder, same modifiers, same manglers — and this module is the
//! only thing that reads them for the other engine: it states the whole
//! binding as data — the carriers C holds values in, how each type's values
//! cross, the form each exported function takes — and hands it to the engine
//! with a [`CTarget`], which only writes. An ignore is a v1 decision about v1's
//! undeclared-item warnings, which v2 does not emit, so none reaches the
//! engine. [`generate`] hands back a [`Generation`] the frontend writes out.
//!
//! Nothing of v1 runs on this route. The engine reads the same sources and the
//! same declarations and owns everything after that; the frontend's part is
//! naming — which C name a type or a symbol gets is its manglers applied, the
//! same answer v1 would give — and saying, for each declarator it does not
//! lower, which one it was, so a skip can name the capability it waits for.

mod target;

use prebindgen_registry::{flat::Flat, TypeKey};
use prebindgen_registry_v2::{
    generate, mirrored_i32_enum, Accepts, Binding, Codec, Declaration, EngineError, EnumArm,
    FailureCategory, FailureRoute, FunctionFormOf, Generation, Operation, OutputForm,
    Representation, Scope, StandardOp, Target, Terminal, Unsupported, Via, WireType,
};
use quote::format_ident;
pub use target::{CCarrier, CClass, COp, CTarget};

use crate::CbindgenBuilder;

impl CbindgenBuilder {
    /// Run the v2 engine over the declarations accumulated so far.
    pub(crate) fn generate_v2(&self) -> Result<Generation<CTarget>, EngineError> {
        // A declared type the source never exported — `String` as a handle —
        // is an item the binding defines: the model gets an extern for it, and
        // steps aside where the source captured a type of that name. Every
        // declared type is registered, not only the opaque ones: a data
        // declaration over a type the model cannot see into is then refused
        // for what it is, rather than failing the build as a name nothing
        // captured. The item's name is the key's, without the arguments a key
        // may carry — `ptr_type!(Publisher<'static>)` names the item
        // `Publisher`.
        let mut sources = self.sources.clone();
        let declared_types = self
            .data
            .keys()
            .chain(self.opaque.keys())
            .chain(self.value_opaque.keys())
            .chain(self.enums.keys())
            .chain(self.tagged_unions.keys());
        for key in sorted(declared_types) {
            if let Some(name) = key.short_name().and_then(|name| syn::parse_str(&name).ok()) {
                sources = sources.local_type(name);
            }
        }
        let flat = sources.build()?;
        // The module generated code reaches the source through: the one the
        // build script set, else the first source's own crate.
        let source_module = self.source_module.clone().unwrap_or_else(|| {
            flat.source_modules()
                .first()
                .and_then(|module| syn::parse_str(module).ok())
                .unwrap_or_else(|| syn::parse_quote!(crate))
        });
        let binding = self.binding(&flat);
        generate(flat, &CTarget, binding, source_module)
    }

    /// Everything this binding declared, as the data the engine plans from.
    ///
    /// Read in one sorted pass over the builder's storage, so that a run over
    /// unchanged input emits the same file. Each declared type is stated
    /// twice over one representation: as the rule for every value of the
    /// type, and as an output exposing it, which is what makes the type's own
    /// declaration emitted whether or not a function uses it.
    fn binding(&self, flat: &Flat) -> Binding<CTarget> {
        let mut binding = Binding::new();

        // The scalar this target carries so far: an `i64` is an `int64_t`, and
        // crosses unchanged.
        let i64_c = binding.carrier(WireType {
            rust: syn::parse_quote!(i64),
            class: CClass::I64,
            members: None,
            meta: CCarrier::Builtin,
        });
        let unchanged = Codec {
            carrier: i64_c,
            operation: Operation::standard(StandardOp::Identity),
        };
        let i64_whole = binding.representation(Representation::Terminal {
            into_rust: Some(unchanged.clone()),
            out_of_rust: Some(unchanged),
            release: None,
        });
        binding.rule(Scope::Type(type_key("i64")), i64_whole);

        let declare_type = |binding: &mut Binding<CTarget>,
                            key: &TypeKey,
                            representation: Representation<COp>,
                            release: Option<FunctionFormOf<CTarget>>| {
            let representation = binding.representation(representation);
            binding.rule(Scope::Type(key.clone()), representation);
            binding.output(
                Declaration::Type(key.clone()),
                OutputForm::Type {
                    representation,
                    release,
                    meta: (),
                },
            );
        };

        // By-value data structs: a `repr(C)` aggregate, one member per field,
        // each member read where the wrapper needs the field.
        for key in sorted(self.data.keys()) {
            let c_name = self.c_type_name(key);
            let representation = match self.aggregate_refusal(flat, key, &c_name) {
                Some(refusal) => Representation::Unsupported(refusal),
                None => {
                    let ident = format_ident!("{c_name}");
                    let carrier = binding.carrier(WireType {
                        rust: syn::parse_quote!(#ident),
                        class: CClass::Aggregate,
                        // What a member may be: the scalar, not yet another
                        // aggregate, a handle or an enum.
                        members: Some(Accepts::of([CClass::I64])),
                        meta: CCarrier::Aggregate { c_name },
                    });
                    Representation::Product {
                        via: Via::Fields,
                        carrier,
                        read: Operation::standard(StandardOp::ReadMember),
                    }
                }
            };
            declare_type(&mut binding, key, representation, None);
        }

        // Opaque handles: `<c_name> *` to a Rust-owned value, freed through
        // the typed destructor the manglers name.
        for key in sorted(self.opaque.keys()) {
            let c_name = self.c_type_name(key);
            let ident = format_ident!("{c_name}");
            let pointer = binding.carrier(WireType {
                rust: syn::parse_quote!(*mut #ident),
                class: CClass::Pointer,
                members: None,
                meta: CCarrier::Opaque { c_name },
            });
            let representation = Representation::Terminal {
                into_rust: Some(Codec {
                    carrier: pointer,
                    operation: Operation::standard(StandardOp::FromRaw),
                }),
                out_of_rust: Some(Codec {
                    carrier: pointer,
                    operation: Operation::standard(StandardOp::IntoRaw),
                }),
                release: Some(Operation::standard(StandardOp::Release)),
            };
            // A release has no source parameter to take a name from, and
            // takes v1's.
            let release = form(
                self.destructor_symbol(key).to_string(),
                vec![format_ident!("this_")],
            );
            declare_type(&mut binding, key, representation, Some(release));
        }

        // A declared enum: C sees a `repr(C)` enum of the same values, and a
        // value of the type crosses as one of them. Into Rust it arrives as
        // `MaybeUninit` of that enum and is read as the C `int` it holds: C
        // lets an enum variable hold any `int`, and a Rust enum holding a
        // number none of its values has is undefined behaviour before any
        // match can look at it. So that direction matches the number, and
        // fails on one no value has.
        for key in sorted(self.enums.keys()) {
            let c_name = self.c_type_name(key);
            let unit = key.short_name().and_then(|name| flat.unit_enum(&name));
            let representation = match mirrored_i32_enum(unit, &c_name, CTarget::NAME) {
                Err(refusal) => Representation::Unsupported(refusal),
                Ok(values) => {
                    let ident = format_ident!("{c_name}");
                    let enumeration = binding.carrier(WireType {
                        rust: syn::parse_quote!(#ident),
                        class: CClass::Enum,
                        members: None,
                        meta: CCarrier::Enum {
                            c_name: c_name.clone(),
                        },
                    });
                    let storage = binding.carrier(WireType {
                        rust: syn::parse_quote!(::core::mem::MaybeUninit<#ident>),
                        class: CClass::Enum,
                        members: None,
                        meta: CCarrier::EnumBits,
                    });
                    let named = values
                        .iter()
                        .map(|(value, _)| {
                            let name = &value.name;
                            EnumArm {
                                name: name.clone(),
                                shape: value.shape,
                                carried: syn::parse_quote!(#ident::#name),
                            }
                        })
                        .collect();
                    let numbered = values
                        .iter()
                        .map(|(value, number)| {
                            let number = proc_macro2::Literal::i32_unsuffixed(*number);
                            EnumArm {
                                name: value.name.clone(),
                                shape: value.shape,
                                carried: syn::parse_quote!(#number),
                            }
                        })
                        .collect();
                    Representation::Terminal {
                        into_rust: Some(Codec {
                            carrier: storage,
                            operation: Operation::standard(StandardOp::EnumIn {
                                values: numbered,
                                invalid: Some(format!("`{c_name}` has no value numbered {{}}")),
                                bits: Some(Box::new(syn::parse_quote!(::core::ffi::c_int))),
                            }),
                        }),
                        out_of_rust: Some(Codec {
                            carrier: enumeration,
                            operation: Operation::standard(StandardOp::EnumOut { values: named }),
                        }),
                        release: None,
                    }
                }
            };
            declare_type(&mut binding, key, representation, None);
        }

        // Every other declarator is accounted for and refused by name. A
        // declared type need not be a captured item.
        for (keys, declarator) in [
            (sorted(self.value_opaque.keys()), "value_opaque"),
            (sorted(self.tagged_unions.keys()), "tagged_union"),
        ] {
            for key in keys {
                let refusal = unimplemented(declarator, &Declaration::Type(key.clone()));
                declare_type(
                    &mut binding,
                    key,
                    Representation::Unsupported(refusal),
                    None,
                );
            }
        }

        // Callback signatures: a C closure struct the caller fills in, moved
        // into the closure Rust builds and called through on every call. Its
        // arguments leave Rust as they do anywhere, and none of those
        // conversions can fail here, so a call needs no route.
        for key in sorted(self.callbacks.keys()) {
            let args: Vec<syn::Type> = key
                .iter()
                .map(|arg| syn::parse_str(arg.as_str()).expect("a type key is a type"))
                .collect();
            let callback =
                TypeKey::from_type(&syn::parse_quote!(impl Fn(#(#args),*) + Send + Sync + 'static));
            let c_name = self.callback_c_name(key);
            let ident = format_ident!("{c_name}");
            let closure = binding.carrier(WireType {
                rust: syn::parse_quote!(#ident),
                class: CClass::Closure,
                // What an argument may be: what leaves Rust in a register —
                // the scalar, an address, an enum. A by-value aggregate leaving
                // Rust has no construction yet.
                members: Some(Accepts::of([CClass::I64, CClass::Pointer, CClass::Enum])),
                meta: CCarrier::Closure { c_name },
            });
            let representation = binding.representation(Representation::Callback {
                carrier: closure,
                capture: Operation::standard(StandardOp::Identity),
                invoke: Operation::target(COp::Call),
                routes: Vec::new(),
            });
            binding.rule(Scope::Type(callback.clone()), representation);
            binding.output(
                Declaration::Callback(callback),
                OutputForm::Type {
                    representation,
                    release: None,
                    meta: (),
                },
            );
        }

        // Declared conversions: the wire mapping for one Rust type, defined by
        // the binding rather than selected out of the source.
        for decl in &self.convert_decls {
            let declaration = Declaration::Conversion(decl.key().clone());
            let refusal = unimplemented("convert", &declaration);
            binding.output(declaration, OutputForm::Unsupported(refusal));
        }

        // Exported functions — the one kind that must name a captured item. A
        // C parameter keeps the source parameter's name: that is what the
        // header shows, and what v1 shows.
        for ident in sorted(self.functions.keys()) {
            let inputs = flat
                .function(&ident.to_string())
                .map(|function| function.params.iter().map(|p| p.name.clone()).collect())
                .unwrap_or_default();
            binding.output(
                Declaration::Function(ident.clone()),
                OutputForm::Function {
                    form: form(self.fn_symbol(ident).to_string(), inputs),
                    meta: (),
                },
            );
        }

        binding
    }

    /// Why a `data_type!` cannot be a C aggregate, when the model already
    /// says: no fields at all, or a field with no name for a member.
    fn aggregate_refusal(&self, flat: &Flat, key: &TypeKey, c_name: &str) -> Option<Unsupported> {
        let name = key.short_name()?;
        let strukt = flat.struct_type(&name)?;
        if strukt.fields.is_empty() {
            // A `repr(C)` struct with no members has no portable C
            // representation, and rustc's FFI lint says so about passing one
            // across an `extern "C"` boundary.
            return Some(Unsupported::new(
                "unsupported.c.empty_aggregate",
                format!("`{c_name}` has no fields, and an empty aggregate has no portable C form"),
            ));
        }
        strukt
            .fields
            .iter()
            .find(|field| field.name.is_none())
            .map(|field| {
                Unsupported::new(
                    "unsupported.c.positional_field",
                    format!(
                        "field {} of `{c_name}` has no name, and a C member needs one",
                        field.index
                    ),
                )
            })
    }
}

/// The form every C wrapper takes: `extern "C"`, no parameter the convention
/// adds, and any C wire type on either side.
///
/// A member read is infallible and a scalar crosses unchanged; the one thing
/// that can fail is a handle arriving null, or an enum arriving as a number
/// none of its values has, and C has no exception to raise. The process stops,
/// as it does under v1's `.panic()`. A runtime failure has no route, and a
/// function that could raise one is skipped until it does.
fn form(symbol: String, inputs: Vec<syn::Ident>) -> FunctionFormOf<CTarget> {
    prebindgen_registry_v2::FunctionForm {
        abi: "C".to_string(),
        symbol,
        context: Vec::new(),
        inputs,
        routes: vec![FailureRoute {
            category: FailureCategory::Binding,
            report: None,
            on_report_failure: Terminal::Abort,
            terminate: Terminal::Abort,
        }],
        attrs: Vec::new(),
        unsafety: false,
        params: Accepts::of(CClass::all()),
        ret: Accepts::of(CClass::all()),
    }
}

/// A declarator v2 has no lowering for, refused by its name so a skip says
/// which capability it waits for.
fn unimplemented(declarator: &str, declaration: &Declaration) -> Unsupported {
    Unsupported::new(
        format!("unsupported.c.{declarator}"),
        format!("`{declaration}` is declared with `{declarator}`, which the v2 C target does not lower yet"),
    )
}

/// A type key the frontend names by spelling, such as a scalar's.
fn type_key(spelled: &str) -> TypeKey {
    TypeKey::parse(spelled).expect("a scalar's name is a type key")
}

/// The builder's hash-keyed storage, in a stable order: the engine emits in
/// request order, and a run over unchanged input has to write the same file.
fn sorted<T: Ord>(keys: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut keys: Vec<T> = keys.into_iter().collect();
    keys.sort();
    keys
}

/// Print one cargo warning per capability a skip named, with the declarations
/// it took down.
///
/// A build log is not a list of everything: at most five declarations per
/// capability, and a count of the rest. What a build script needs from it is
/// which capability to ask for next, not which of forty declarations waits on
/// it.
pub(crate) fn warn_skipped(skipped: &[(Declaration, prebindgen_registry_v2::Skip)]) {
    let mut by_capability: std::collections::BTreeMap<&str, Vec<String>> =
        std::collections::BTreeMap::new();
    for (declaration, skip) in skipped {
        by_capability
            .entry(skip.capability.as_str())
            .or_default()
            .push(declaration.to_string());
    }
    for (capability, mut roots) in by_capability {
        roots.sort();
        roots.dedup();
        let shown = roots.len().min(5);
        let more = match roots.len() - shown {
            0 => String::new(),
            rest => format!(" (+{rest} more)"),
        };
        println!(
            "cargo:warning=SKIP {capability}: {}{more}",
            roots[..shown].join(", ")
        );
    }
}
