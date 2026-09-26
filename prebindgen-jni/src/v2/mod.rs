//! The JNI binding under the v2 engine.
//!
//! [`Declarations`] keeps accumulating exactly as it does for v1 — same
//! `package!`/`ptr_class!`/`fun!` surface, same `set_*` settings, same
//! name-mangle closures — and this module is the only thing that reads them
//! for the other engine: it states the whole binding as data — the wire types the
//! JVM holds values in, how each type's values cross, the native method and
//! `Java_…` symbol each wrapper gets — and hands it to the engine with a
//! [`JniTarget`], which only writes. An ignore is a v1 decision about v1's
//! undeclared-item warnings, which v2 does not emit, so none reaches the
//! engine. [`generate`] hands back a [`Generation`] the frontend writes out —
//! the Rust through the engine's writer, the Kotlin through its own writer in
//! `kotlin.rs`.
//!
//! Nothing of v1 runs on this route. Kotlin names, packages and `Java_…`
//! symbols come from this adapter's settings applied to the declarations,
//! which is the same answer v1 would give; the engine never guesses one.

mod kotlin;
mod target;

use prebindgen_registry::{
    flat::{Flat, TypeKind, TypeRef},
    TypeKey,
};
use prebindgen_registry_v2::{
    field_is_conditional, generate, mirrored_i32_enum, Binding, ContextParam, Declaration,
    EngineError, EnumArm, FailureCategory, FailureRoute, FunctionForm, FunctionFormOf, Generation,
    InRepresentation, Operation, OutRepresentation, OutputForm, OutputFormOf, PlanningError,
    Report, Scope, StandardOp, Target, Terminal, Unsupported, Via,
};
use quote::format_ident;
pub use target::{JniOp, JniOutput, JniTarget, JniWireKind, JniWireType, KotlinType};

use crate::jni::{ClassMember, Declarations, FunctionEntry};

impl Declarations {
    /// Run the v2 engine over the declarations accumulated so far.
    pub(crate) fn generate_v2(
        &self,
        sources: prebindgen_registry::flat::FlatBuilder,
    ) -> Result<Generation<JniTarget>, EngineError> {
        let mut sources = sources;
        // What the binding defines itself enters the model as entities: a
        // helper with the signature `fun!(crate::x).sig(..)` stated, reached
        // where its path says; an opaque class the source never exported,
        // reached at the binding's root. A captured item of the same name is
        // what a class meant, and an error for a function, as under v1.
        for (ident, path, sig) in &self.local_fns {
            let mut sig = sig.clone();
            sig.ident = ident.clone();
            let prefix = prebindgen_registry::decl::local_path_prefix(path);
            let module: syn::Path = syn::parse_str(&prefix).unwrap_or_else(|_| {
                panic!("binding-local fn `{ident}` is declared at an unparseable path `{prefix}`")
            });
            sources = sources.local_function(sig, module);
        }
        // Every declared class, not only the handle ones: a data class over a
        // type the model cannot see into is then refused for what it is,
        // rather than failing the build as a name nothing captured. The item's
        // name is the key's, without the arguments a key may carry —
        // `ptr_class!(Publisher<'static>)` names `Publisher`.
        for key in sorted(self.types.keys()) {
            if let Some(name) = key.short_name().and_then(|name| syn::parse_str(&name).ok()) {
                sources = sources.local_type(name);
            }
        }
        let flat = sources.build()?;
        // The module generated code reaches the source through: the first
        // source's own crate, as v1 resolves it.
        let source_module = flat
            .source_modules()
            .first()
            .and_then(|module| syn::parse_str(module).ok())
            .unwrap_or_else(|| syn::parse_quote!(crate));
        let binding = self.binding(&flat)?;
        generate(flat, &JniTarget, binding, source_module)
    }

    /// Write the Kotlin side of a v2 generation under `kotlin_root`.
    pub(crate) fn write_kotlin_v2(
        &self,
        generation: &Generation<JniTarget>,
        kotlin_root: &std::path::Path,
    ) -> Result<Vec<std::path::PathBuf>, kotlin_codegen::WriteKotlinError> {
        kotlin::write(self, generation, kotlin_root)
    }

    /// Everything this binding declared, as the data the engine plans from.
    ///
    /// Read in one sorted pass over the declarations, so that a run over
    /// unchanged input emits the same file. Each declared class is stated
    /// twice over one representation: as the rule for every value of the
    /// type, and as an output exposing it.
    ///
    /// Fails when the binding contradicts itself — two declarations claiming
    /// one native method on the harness — which is the frontend's own
    /// validation and not a capability the engine lacks.
    fn binding(&self, flat: &Flat) -> Result<Binding<JniTarget>, EngineError> {
        let mut binding = Binding::new();
        // Every wrapper hangs off one harness object, so its native methods
        // share a namespace: two declarations naming the same one would be two
        // definitions of one `Java_…` symbol, which the generated code cannot
        // compile. The binding is what decides the names, so it is told here.
        let mut natives: std::collections::HashMap<String, Declaration> =
            std::collections::HashMap::new();
        let mut collision = None;
        let mut claim = |native: &str, declaration: &Declaration| {
            if let Some(taken) = natives.insert(native.to_string(), declaration.clone()) {
                collision.get_or_insert_with(|| {
                    format!(
                        "`{taken}` and `{declaration}` would both be the native method \
                         `{native}` on the JNI harness; give one of them another Kotlin name"
                    )
                });
            }
        };

        // The scalar this target carries so far: an `i64` is a `jlong`, and
        // the two are one Rust value.
        let jlong = binding.wire_type(JniWireType::Long);
        let i64_key = TypeKey::parse("i64").expect("a scalar's name is a type key");
        let i64_in = binding.in_representation(InRepresentation::Whole {
            wire_type: jlong,
            operation: Operation::Standard(StandardOp::Identity),
        });
        let i64_out = binding.out_representation(OutRepresentation::Whole {
            wire_type: jlong,
            operation: Operation::Standard(StandardOp::Identity),
            release: None,
        });
        binding.rule(Scope::Type(i64_key.clone()), i64_in);
        binding.rule(Scope::Type(i64_key), i64_out);

        // A function is declared wherever it is placed — as a class member, as
        // a package function, as the `val` a `constant!(X).fun(..)` reads
        // through — and one function may be placed more than once. Each
        // placement is declared and accounted for on its own, and each needs
        // its own native method, because the harness has one namespace: the
        // name a single placement takes is the Rust identifier's, as v1 names
        // it, and a further placement is named after where it is placed.
        let placements: std::collections::HashMap<&syn::Ident, usize> = self
            .class_members
            .values()
            .flatten()
            .map(|member| &member.rust_ident)
            .chain(self.packages.values().flat_map(|config| {
                config
                    .functions
                    .iter()
                    .chain(&config.constant_functions)
                    .map(|entry| &entry.rust_ident)
            }))
            .fold(std::collections::HashMap::new(), |mut count, ident| {
                *count.entry(ident).or_default() += 1;
                count
            });
        let placed_more_than_once =
            |ident: &syn::Ident| placements.get(ident).is_some_and(|n| *n > 1);

        // Declared classes. A declared class need not name a captured item: a
        // target may represent `String` or `Vec<u8>` without the source
        // exporting one.
        for key in sorted(self.types.keys()) {
            let config = &self.types[key];
            let placement = self.kotlin_fqn(key).unwrap_or_default();
            let (package, class) = match placement.rsplit_once('.') {
                Some((package, class)) => (package.to_string(), class.to_string()),
                None => (String::new(), placement.clone()),
            };
            let declaration = Declaration::Type(key.clone());
            let declarator = declarator(&config.kind);
            // A class implementing an interface, or generating one, is
            // refused rather than emitted without it: the binding asked for
            // that supertype, and v2 writes none.
            let interface = config.interface_enabled || !config.interfaces.is_empty();
            let (into_rust, out_of_rust, release, meta) = match config.kind {
                _ if interface => (
                    InRepresentation::Unsupported(unimplemented(
                        declarator,
                        "interface",
                        &declaration,
                        &placement,
                    )),
                    Some(OutRepresentation::Unsupported(unimplemented(
                        declarator,
                        "interface",
                        &declaration,
                        &placement,
                    ))),
                    None,
                    JniOutput::DataClass {
                        package: package.clone(),
                        class: class.clone(),
                    },
                ),
                crate::jni::DeclaredKind::Data => {
                    let into_rust = self.data_class(&mut binding, flat, key, &placement);
                    // A class refused as a whole is refused both ways.
                    let out_of_rust = match &into_rust {
                        InRepresentation::Unsupported(reason) => {
                            OutRepresentation::Unsupported(reason.clone())
                        }
                        _ => OutRepresentation::struct_unsupported(key),
                    };
                    (
                        into_rust,
                        Some(out_of_rust),
                        None,
                        JniOutput::DataClass {
                            package: package.clone(),
                            class: class.clone(),
                        },
                    )
                }
                // A Kotlin `enum class` of the same values: what crosses is the
                // number each value carries.
                crate::jni::DeclaredKind::Enum(_) => {
                    let (into_rust, out_of_rust, values) =
                        self.enum_class(&mut binding, flat, key, &placement);
                    (
                        into_rust,
                        Some(out_of_rust),
                        None,
                        JniOutput::EnumClass {
                            package: package.clone(),
                            class: class.clone(),
                            values,
                        },
                    )
                }
                // The release is a native method on the harness like any other,
                // named after the class: `freeLedger`.
                crate::jni::DeclaredKind::Ptr(_) => {
                    let native = self.mangle_jni_method(&format!("free{class}"));
                    claim(&native, &declaration);
                    let address = binding.wire_type(JniWireType::Handle {
                        kotlin_class: placement.clone(),
                    });
                    let into_rust = InRepresentation::Whole {
                        wire_type: address,
                        operation: Operation::Standard(StandardOp::FromRaw),
                    };
                    let out_of_rust = OutRepresentation::Whole {
                        wire_type: address,
                        operation: Operation::Standard(StandardOp::IntoRaw),
                        release: Some(Operation::Standard(StandardOp::Release)),
                    };
                    let release = release_form(self.native_method_symbol(&native));
                    (
                        into_rust,
                        Some(out_of_rust),
                        Some(release),
                        JniOutput::PtrClass {
                            package: package.clone(),
                            class: class.clone(),
                            native,
                        },
                    )
                }
                _ => (
                    InRepresentation::Unsupported(unimplemented(
                        declarator,
                        declarator,
                        &declaration,
                        &placement,
                    )),
                    Some(OutRepresentation::Unsupported(unimplemented(
                        declarator,
                        declarator,
                        &declaration,
                        &placement,
                    ))),
                    None,
                    JniOutput::DataClass {
                        package: package.clone(),
                        class: class.clone(),
                    },
                ),
            };
            let into_rust = binding.in_representation(into_rust);
            binding.rule(Scope::Type(key.clone()), into_rust);
            let out_of_rust = out_of_rust.map(|out| binding.out_representation(out));
            if let Some(out_of_rust) = out_of_rust {
                binding.rule(Scope::Type(key.clone()), out_of_rust);
            }
            binding.output(
                declaration,
                OutputForm::Type {
                    into_rust,
                    out_of_rust,
                    release,
                    meta,
                },
            );

            // Members are separately selected: a class can be emitted with one
            // of its methods skipped, so each is a declaration of its own. None is
            // lowered yet: a method's receiver is a handle.
            for member in self.class_members.get(key).into_iter().flatten() {
                let placed = format!("{placement}.{}", self.effective_method_name(key, member));
                let declaration = Declaration::Function(member.rust_ident.clone());
                let represented = member_representation(member);
                let refusal = unimplemented(represented, represented, &declaration, &placed);
                binding.output(declaration, OutputForm::Unsupported(refusal));
            }
        }

        self.callbacks(&mut binding, flat);

        // Free-standing package functions and constants.
        for (subpackage, config) in &self.packages {
            let package = self.package_name(subpackage);
            let placed = |entry: &FunctionEntry| {
                format!(
                    "{package}.{}",
                    self.effective_function_name(subpackage, entry)
                )
            };
            for entry in &config.functions {
                let method = self.effective_function_name(subpackage, entry);
                // The native method is named from the Rust identifier, through
                // the method-name hook, as v1 names it — never from the public
                // function's `.name()`: two packages may each export a `value`,
                // and the harness has one namespace. A function placed more
                // than once is the exception: each placement is a wrapper of
                // its own, so each is named after where it is placed, which is
                // what tells the placements apart.
                let native_name = match placed_more_than_once(&entry.rust_ident) {
                    true if subpackage.is_empty() => method.clone(),
                    true => format!("{}_{method}", subpackage.replace('.', "_")),
                    false => entry.rust_ident.to_string(),
                };
                let native = self.mangle_jni_method(&crate::util::snake_to_camel(&native_name));
                let declaration = Declaration::Function(entry.rust_ident.clone());
                // A function under a setting v2 does not honour is refused
                // rather than emitted with the setting dropped: the default
                // interface is not the one the binding asked for.
                let form: OutputFormOf<JniTarget> = match self
                    .unimplemented_setting(flat, &entry.rust_ident)
                {
                    Some(setting) => OutputForm::Unsupported(unimplemented(
                        "fun",
                        setting,
                        &declaration,
                        &placed(entry),
                    )),
                    None => {
                        claim(&native, &declaration);
                        match flat.function(&entry.rust_ident.to_string()) {
                            Some(function) => OutputForm::Function {
                                form: function_form(self.native_method_symbol(&native), function),
                                meta: JniOutput::Function {
                                    package: package.clone(),
                                    method,
                                    native,
                                },
                            },
                            // Not in the model: the engine fails the run
                            // over the declaration naming nothing.
                            None => OutputForm::Unsupported(unimplemented(
                                "fun",
                                "fun",
                                &declaration,
                                &placed(entry),
                            )),
                        }
                    }
                };
                binding.output(declaration, form);
            }
            // A `constant!(X)` names the `#[prebindgen]` const it reads. The
            // Kotlin `val` keeps the const's own name — it is not a function
            // and takes neither the camel-casing nor the function-name hook.
            for entry in &config.constants {
                let placed = format!(
                    "{package}.{}",
                    entry
                        .kotlin_name_override
                        .clone()
                        .unwrap_or_else(|| entry.rust_ident.to_string())
                );
                let declaration = Declaration::Const(entry.rust_ident.clone());
                let refusal = unimplemented("constant", "constant", &declaration, &placed);
                binding.output(declaration, OutputForm::Unsupported(refusal));
            }
            // A `constant!(X).fun(..)` is a Kotlin `val` read through a nullary
            // function: the declaration is the function's, and the `val` is
            // what this target chooses to show the call as.
            for entry in &config.constant_functions {
                let declaration = Declaration::Function(entry.rust_ident.clone());
                let refusal =
                    unimplemented("constant_fun", "constant_fun", &declaration, &placed(entry));
                binding.output(declaration, OutputForm::Unsupported(refusal));
            }
            // A `constant!(X).expr(..)` has no Rust item behind it at all.
            for decl in &config.constant_exprs {
                let declaration = Declaration::ComputedConst(decl.kotlin_name.clone());
                let refusal = unimplemented(
                    "constant_expr",
                    "constant_expr",
                    &declaration,
                    &format!("{package}.{}", decl.kotlin_name),
                );
                binding.output(declaration, OutputForm::Unsupported(refusal));
            }
        }

        // Declared conversions: the wire mapping for one Rust type, defined by
        // the binding rather than selected out of the source.
        //
        // A binding-local fn is NOT a declaration of its own. It is a helper the
        // binding defines, and what the target exports is the member or the
        // package function it was bound to — already stated above.
        for decl in &self.convert_decls {
            let declaration = Declaration::Conversion(decl.key().clone());
            let refusal = unimplemented(
                "convert",
                "convert",
                &declaration,
                &self.kotlin_fqn(decl.key()).unwrap_or_default(),
            );
            binding.output(declaration, OutputForm::Unsupported(refusal));
        }

        if let Some(collision) = collision {
            return Err(EngineError::Planning(PlanningError::InvalidInput(
                collision,
            )));
        }
        Ok(binding)
    }

    /// Every callback signature a package function takes, as a Kotlin `fun
    /// interface` in the base package.
    ///
    /// No declarator names one: a Kotlin callback is whatever the functions
    /// exported take, as it is in v1, and is named from its arguments as v1
    /// names it — `LongCallback`, `LedgerOperationCallback`. The callable
    /// arrives as a JVM object, is held through a global reference, and is
    /// called from whatever thread Rust calls it from; a call that fails has
    /// no caller to report to, so the failure is written out and the call
    /// returns.
    fn callbacks(&self, binding: &mut Binding<JniTarget>, flat: &Flat) {
        let mut signatures: std::collections::BTreeMap<String, TypeRef> = Default::default();
        for config in self.packages.values() {
            for entry in &config.functions {
                let Some(function) = flat.function(&entry.rust_ident.to_string()) else {
                    continue;
                };
                for param in &function.params {
                    if let TypeKind::Callback { .. } = param.ty.kind() {
                        signatures
                            .entry(param.ty.key().as_str().to_string())
                            .or_insert_with(|| param.ty.clone());
                    }
                }
            }
        }
        let package = self.package_name("");
        let qualified = |class: &str| match package.is_empty() {
            true => class.to_string(),
            false => format!("{package}.{class}"),
        };
        let jni_error: syn::Type = syn::parse_quote!(jni::errors::Error);
        let mut named_callbacks = Vec::new();
        for ty in signatures.into_values() {
            let TypeKind::Callback { args } = ty.kind() else {
                continue;
            };
            // Each argument by the Kotlin name its type goes by, and whether
            // the callable takes it in another form than the public interface
            // shows: a handle as its address, an enum as its number.
            let named: Vec<(String, bool)> = args
                .iter()
                .map(|arg| {
                    let key = arg.key();
                    let adapted = matches!(
                        self.types.get(&key).map(|config| &config.kind),
                        Some(crate::jni::DeclaredKind::Ptr(_) | crate::jni::DeclaredKind::Enum(_))
                    );
                    let short = match self.kotlin_fqn(&key) {
                        Some(fqn) => fqn.rsplit('.').next().unwrap_or_default().to_string(),
                        None if key.as_str() == "i64" => "Long".to_string(),
                        None => key.short_name().unwrap_or_else(|| key.as_str().to_string()),
                    };
                    (short, adapted)
                })
                .collect();
            let class = match named.is_empty() {
                true => "VoidCallback".to_string(),
                false => format!(
                    "{}Callback",
                    named
                        .iter()
                        .map(|(short, _)| short.as_str())
                        .collect::<String>()
                ),
            };
            let raw = named
                .iter()
                .any(|(_, adapted)| *adapted)
                .then(|| format!("{class}Raw"));
            named_callbacks.push((ty, class, raw));
        }
        // A name built from short names is not unique — `Fn(FooBar, Baz)` and
        // `Fn(Foo, BarBaz)`, or two `Foo`s from two packages, are both
        // `…Callback` — and every callback lands in one package, beside the
        // declared classes. Two declarations under one name would not compile,
        // so every callback whose name is taken twice is refused, naming what
        // it collides with, and a binding can see it rather than a Kotlin
        // compiler error.
        let mut claimed: std::collections::HashMap<String, Vec<String>> = self
            .types
            .keys()
            .filter_map(|key| {
                Some((
                    self.kotlin_fqn(key)?,
                    vec![format!("type:{}", key.as_str())],
                ))
            })
            .collect();
        for (ty, class, raw) in &named_callbacks {
            for name in std::iter::once(class).chain(raw) {
                claimed
                    .entry(qualified(name))
                    .or_default()
                    .push(Declaration::Callback(ty.key()).to_string());
            }
        }
        for (ty, class, raw) in named_callbacks {
            let collision = std::iter::once(&class)
                .chain(&raw)
                .map(|name| qualified(name))
                .find(|name| claimed[name].len() > 1);
            if let Some(name) = collision {
                let declaration = Declaration::Callback(ty.key());
                let representation =
                    binding.in_representation(InRepresentation::Unsupported(Unsupported::new(
                        "unsupported.jni.callback_name",
                        format!(
                            "`{declaration}` would be the Kotlin `{name}`, as would {}",
                            claimed[&name]
                                .iter()
                                .filter(|other| **other != declaration.to_string())
                                .map(|other| format!("`{other}`"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    )));
                binding.rule(Scope::Type(ty.key()), representation);
                binding.output(
                    declaration,
                    OutputForm::Type {
                        into_rust: representation,
                        out_of_rust: None,
                        release: None,
                        meta: JniOutput::Callback {
                            package: package.clone(),
                            class,
                            raw,
                        },
                    },
                );
                continue;
            }
            let wire_type = binding.wire_type(JniWireType::Callable {
                interface: qualified(&class),
                raw: raw.as_deref().map(qualified),
            });
            let representation = binding.in_representation(InRepresentation::Callable {
                wire_type,
                capture: Operation::Target(JniOp::CaptureCallback),
                invoke: Operation::Target(JniOp::CallCallback),
                routes: vec![FailureRoute {
                    category: FailureCategory::Runtime,
                    report: Some(Report {
                        error: jni_error.clone(),
                        operation: Operation::Target(JniOp::ReportCallbackError),
                    }),
                    on_report_failure: Terminal::Abort,
                    terminate: Terminal::Return(syn::parse_quote!(())),
                }],
            });
            binding.rule(Scope::Type(ty.key()), representation);
            binding.output(
                Declaration::Callback(ty.key()),
                OutputForm::Type {
                    into_rust: representation,
                    out_of_rust: None,
                    release: None,
                    meta: JniOutput::Callback {
                        package: package.clone(),
                        class,
                        raw,
                    },
                },
            );
        }
    }

    /// A data class: a JVM object whose properties are read, one per field —
    /// or why the class cannot be one.
    ///
    /// What the model already says is checked here: a Kotlin data class needs
    /// a property, a property needs a name, and Kotlin cannot state a
    /// condition, so a field written under one would be a property filled in
    /// by every caller and read by the library only sometimes.
    fn data_class(
        &self,
        binding: &mut Binding<JniTarget>,
        flat: &Flat,
        key: &TypeKey,
        placement: &str,
    ) -> InRepresentation<JniOp> {
        if let Some(strukt) = key.short_name().and_then(|name| flat.struct_type(&name)) {
            if strukt.fields.is_empty() {
                return InRepresentation::Unsupported(Unsupported::new(
                    "unsupported.jni.empty_class",
                    format!(
                        "`{}` has no fields, and a Kotlin data class needs at least one property",
                        strukt.name
                    ),
                ));
            }
            for field in &strukt.fields {
                let Some(name) = field.name.as_ref() else {
                    return InRepresentation::Unsupported(Unsupported::new(
                        "unsupported.jni.positional_field",
                        "a positional field has no Kotlin property to read".to_string(),
                    ));
                };
                if field_is_conditional(field) {
                    return InRepresentation::Unsupported(Unsupported::new(
                        "unsupported.jni.conditional_field",
                        format!(
                            "field `{name}` is written under a condition this build cannot \
                             evaluate, and Kotlin cannot state one: a property for it would \
                             be filled in by every caller and read by the wrapper only \
                             sometimes"
                        ),
                    ));
                }
            }
        }
        let object = binding.wire_type(JniWireType::Object {
            kotlin_class: placement.to_string(),
        });
        InRepresentation::Parts {
            via: Via::Fields,
            wire_type: object,
            // A property read is a JVM call, which can fail: see `JniOp`.
            read: Operation::Target(JniOp::Getter),
        }
    }

    /// An `enum class`: a value crosses as the number Rust assigns it, which
    /// is what the Kotlin class carries too — or why the enum cannot be one.
    fn enum_class(
        &self,
        binding: &mut Binding<JniTarget>,
        flat: &Flat,
        key: &TypeKey,
        placement: &str,
    ) -> (
        InRepresentation<JniOp>,
        OutRepresentation<JniOp>,
        Vec<(String, i32)>,
    ) {
        let unit = key.short_name().and_then(|name| flat.unit_enum(&name));
        let values = match mirrored_i32_enum(unit, placement, JniTarget::NAME) {
            Ok(values) => values,
            Err(refusal) => {
                return (
                    InRepresentation::Unsupported(refusal.clone()),
                    OutRepresentation::Unsupported(refusal),
                    Vec::new(),
                )
            }
        };
        let number = binding.wire_type(JniWireType::Int {
            kotlin_enum: placement.to_string(),
        });
        let arms: Vec<EnumArm> = values
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
        let named = values
            .iter()
            .map(|(value, number)| {
                let screaming =
                    crate::util::camel_to_screaming_snake(target::plain(&value.name.to_string()));
                (kotlin_codegen::escape_kotlin_ident(&screaming), *number)
            })
            .collect();
        // Out of Rust every value names one number; into Rust the wire type is
        // an `Int` and can hold something no value names, which is what a
        // caller passing one gets told, rather than a value it did not ask for.
        let into_rust = InRepresentation::Whole {
            wire_type: number,
            operation: Operation::Standard(StandardOp::EnumIn {
                values: arms.clone(),
                invalid: Some(format!("`{placement}` has no value numbered {{}}")),
                bits: None,
            }),
        };
        let out_of_rust = OutRepresentation::Whole {
            wire_type: number,
            operation: Operation::Standard(StandardOp::EnumOut { values: arms }),
            release: None,
        };
        (into_rust, out_of_rust, named)
    }
}

/// The form of a native method exporting `function`.
///
/// The two parameters the JVM adds are named around the source's: a source
/// parameter called `env` keeps its name, and the environment steps aside. The
/// native method is an instance method of the harness `object`, so what the
/// JVM passes second is the singleton, not a class.
fn function_form(
    symbol: String,
    function: &prebindgen_registry::flat::Function,
) -> FunctionFormOf<JniTarget> {
    let returns = !matches!(
        function.ret.kind(),
        prebindgen_registry::flat::TypeKind::Unit
    );
    jni_form(
        symbol,
        vec![
            ContextParam {
                name: target::free_name("env", function),
                ty: syn::parse_quote!(jni::JNIEnv<'_>),
                supplies: Some("jni.env".to_string()),
                mutable: true,
            },
            ContextParam {
                name: target::free_name("_this", function),
                ty: syn::parse_quote!(jni::objects::JObject<'_>),
                supplies: None,
                mutable: false,
            },
        ],
        // A wrapper parameter keeps the source parameter's name, as v1's do.
        function
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect(),
        returns,
    )
}

/// The form of the native method releasing a handle: it calls nothing on the
/// JVM, so the environment it is handed goes unused — and is named so — and it
/// has no source parameter to take a name from, and takes v1's.
fn release_form(symbol: String) -> FunctionFormOf<JniTarget> {
    jni_form(
        symbol,
        vec![
            ContextParam {
                name: format_ident!("_env"),
                ty: syn::parse_quote!(jni::JNIEnv<'_>),
                supplies: Some("jni.env".to_string()),
                mutable: false,
            },
            ContextParam {
                name: format_ident!("_this"),
                ty: syn::parse_quote!(jni::objects::JObject<'_>),
                supplies: None,
                mutable: false,
            },
        ],
        vec![format_ident!("ptr")],
        false,
    )
}

/// What every native method shares: `extern "system"`, and a route for each
/// failure a conversion can raise, each throwing into the JVM.
///
/// Zero is not a result: it is what a native method must return while an
/// exception is pending, and Kotlin observes the exception. A wrapper that
/// returns nothing terminates with nothing.
fn jni_form(
    symbol: String,
    context: Vec<ContextParam>,
    inputs: Vec<syn::Ident>,
    returns: bool,
) -> FunctionFormOf<JniTarget> {
    let jni_error: syn::Type = syn::parse_quote!(jni::errors::Error);
    let terminate = || match returns {
        true => Terminal::Return(syn::parse_quote!(0)),
        false => Terminal::Return(syn::parse_quote!(())),
    };
    FunctionForm {
        abi: "system".to_string(),
        symbol,
        context,
        inputs,
        routes: vec![
            // A runtime failure: a JVM call that failed.
            FailureRoute {
                category: FailureCategory::Runtime,
                report: Some(Report {
                    error: jni_error.clone(),
                    operation: Operation::Target(JniOp::ReportError),
                }),
                on_report_failure: Terminal::Abort,
                terminate: terminate(),
            },
            // A binding failure: the caller broke the contract — a null
            // handle — and the message says how.
            FailureRoute {
                category: FailureCategory::Binding,
                report: Some(Report {
                    error: syn::parse_quote!(String),
                    operation: Operation::Target(JniOp::ThrowMessage),
                }),
                on_report_failure: Terminal::Abort,
                terminate: terminate(),
            },
        ],
        attrs: Vec::new(),
        unsafety: false,
    }
}

/// A declarator v2 has no lowering for — or a setting on one it does not
/// honour — refused by name, with the Kotlin placement it would have had, so a
/// skip says which capability it waits for and where it was going.
fn unimplemented(
    declarator: &str,
    capability: &str,
    declaration: &Declaration,
    placement: &str,
) -> Unsupported {
    Unsupported::new(
        format!("unsupported.jni.{capability}"),
        format!(
            "`{declaration}` is declared as a `{declarator}` at `{placement}`, which the v2 JNI \
             target does not lower yet"
        ),
    )
}

impl Declarations {
    /// A setting v2 does not lower yet that applies to this function, if any.
    ///
    /// Either the function's own — a per-function `expand_param`,
    /// `expand_return` or `split_on_param` — or a type-level boundary
    /// declaration for the type of one of its parameters or of its result,
    /// which is where such a declaration takes effect (a field of that type is
    /// not a boundary). A function under such a setting is refused rather than
    /// emitted with the setting silently dropped: the default interface is not
    /// the one the binding asked for.
    fn unimplemented_setting(
        &self,
        flat: &prebindgen_registry::flat::Flat,
        ident: &syn::Ident,
    ) -> Option<&'static str> {
        if self.fn_param_expands.iter().any(|(fun, ..)| fun == ident) {
            return Some("expand_param");
        }
        if self.fn_return_expands.iter().any(|(fun, ..)| fun == ident) {
            return Some("expand_return");
        }
        if self.fn_split_params.iter().any(|(fun, ..)| fun == ident) {
            return Some("split_on_param");
        }
        // A binding-local function is not in the model; the engine refuses it
        // on its own account.
        let function = flat.function(&ident.to_string())?;
        if function.params.iter().any(|param| {
            self.param_expand_decls
                .iter()
                .any(|decl| *decl.key() == param.ty.key())
        }) {
            return Some("expand_param");
        }
        if self
            .return_expand_decls
            .iter()
            .any(|decl| *decl.key() == function.ret.key())
        {
            return Some("expand_return");
        }
        None
    }
}

/// The declarations' hash-keyed storage, in a stable order: the engine emits in
/// request order, and a run over unchanged input has to write the same file.
fn sorted<T: Ord>(keys: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut keys: Vec<T> = keys.into_iter().collect();
    keys.sort();
    keys
}

/// A declared class's declarator, as the report names it.
fn declarator(kind: &crate::jni::DeclaredKind) -> &'static str {
    match kind {
        crate::jni::DeclaredKind::Ptr(_) => "ptr_class",
        crate::jni::DeclaredKind::Enum(_) => "enum_class",
        crate::jni::DeclaredKind::Sealed(_) => "sealed_class",
        crate::jni::DeclaredKind::Data => "data_class",
    }
}

/// A class member's declarator, as the report names it.
fn member_representation(member: &ClassMember) -> &'static str {
    match member.kind {
        crate::jni::MemberKind::Method => "method",
        crate::jni::MemberKind::Constructor => "constructor",
    }
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
