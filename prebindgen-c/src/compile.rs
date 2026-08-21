//! What one crossing costs on the C wire.
//!
//! [`crate::rows`] states which parts a value gets across in; this says what
//! each of those parts looks like in C. The registry drives the walk over the
//! table and hands every hook the fragments its parts already produced, so
//! nothing here looks a converter up and nothing here recurses.
//!
//! A [`CFrag`] is what a `ConverterImpl` was, minus the bookkeeping the table
//! now does: no `subs` walk to keep the registry's reachability right by hand,
//! no `pre_stages` chain, and no guessing which of nine categories a type falls
//! into — the row already said.

use prebindgen_registry::{
    flat::{Alternative, Function},
    recipe::{At, Carrier, Compile, Cx, Frag, Mode, Parts, Role, Validity, Yield},
};

use super::*;

/// The C adapter's answer for one crossing.
#[derive(Clone)]
pub(crate) struct CFrag {
    /// The C wire type this crossing carries.
    pub(crate) destination: syn::Type,
    /// The generated converter, complete.
    pub(crate) function: syn::ItemFn,
    /// Bit patterns the wire can represent and this conversion never produces.
    pub(crate) niches: Niches,
    /// Inner types this fragment composed from, which is what marks them
    /// reachable in the registry that still emits them.
    pub(crate) subs: Vec<TypeKey>,
    /// One arm's payload, converted, for a fragment on its way from
    /// [`Compile::fields`] to [`Compile::choice`].
    ///
    /// A union's arm is a product whose parts do not assemble into a value of
    /// their own — they are bound in a `match` and rebuilt on the other side —
    /// so the arm's fragment carries the converted expressions rather than a
    /// function. `None` for every other fragment, which is what a struct's is.
    pub(crate) arm: Option<Arm>,
    /// What the fragment produces, which is all the registry reads of it.
    pub(crate) yields: Yield,
}

/// One alternative's payload, converted in both directions.
///
/// What [`Compile::fields`] hands [`Compile::choice`] for a tagged union: the
/// per-field expressions, keyed by nothing because their order is the
/// alternative's own.
#[derive(Clone)]
pub(crate) struct Arm {
    /// Converted payload expressions, in field order, over the bindings
    /// `__f0..__fN` a `match` arm introduces.
    pub(crate) exprs: Vec<TokenStream>,
    /// Whether any of them can fail, so the union's own converter knows
    /// whether it needs a `Result`.
    pub(crate) fallible: bool,
    /// Inner types the payloads composed from.
    pub(crate) subs: Vec<TypeKey>,
}

impl Carrier for CFrag {
    fn yields(&self) -> Yield {
        self.yields.clone()
    }
}

impl CFrag {
    /// One of the adapter's existing converter builders, as a fragment.
    fn from_converter(at: At<'_>, conv: ConverterImpl<()>) -> Self {
        let validity = validity_of(&conv, at.crossing.assembly());
        Self {
            destination: conv.destination,
            function: conv.function,
            niches: conv.niches,
            subs: conv.subs,
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity,
            },
        }
    }

    /// What the registry that still emits converters expects back.
    pub(crate) fn into_converter(self) -> ConverterImpl<()> {
        ConverterImpl {
            destination: self.destination,
            function: self.function,
            niches: self.niches,
            subs: self.subs,
            pre_stages: vec![],
            metadata: (),
        }
    }
}

/// How long what this conversion produces stays usable.
///
/// A property of the **conversion**, not of how the crossing was spelled.
/// Reading the spelling is wrong in both directions: a conversion may clone or
/// allocate out of a borrow, and — the case C actually has — a conversion may
/// hand C a pointer *into* a Rust value from a crossing that looks owned.
fn validity_of(conv: &ConverterImpl<()>, assembly: Assembly) -> Validity {
    match assembly {
        // Rust to C. A `*const T` is the zero-copy borrow: `out_borrow_or_result`
        // casts the Rust value's own address, and `repr_c_struct`'s reinterpret
        // does the same, so the pointer dies with the value it points into. A
        // `*mut` is a handle C now owns (`Box::into_raw`) or a block C must free
        // (`__cbg_alloc_cstr`), and every by-value wire is a copy.
        Assembly::Deconstruct => match &conv.destination {
            syn::Type::Ptr(p) if p.mutability.is_none() => Validity::Borrowed,
            _ => Validity::SelfSufficient,
        },
        // C to Rust: what the converter's own function hands back. A decode
        // yielding `&'a T` borrows the caller's memory, which is right at a
        // parameter and refused at a return.
        Assembly::Construct => match &conv.function.sig.output {
            syn::ReturnType::Type(_, ty) if produces_borrow(ty) => Validity::Borrowed,
            _ => Validity::SelfSufficient,
        },
    }
}

/// Whether a converter's return type hands back a borrow — `&T`, or a
/// `Result<&T, E>` whose success arm is one.
fn produces_borrow(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(_) => true,
        _ => result_parts(ty).is_some_and(|(ok, _)| matches!(ok, syn::Type::Reference(_))),
    }
}

/// The adapter, for the length of one crossing's compilation.
///
/// Holds the binding's declarations and the registry view the emission helpers
/// still read the model through.
pub(crate) struct CCompile<'a, R> {
    pub(crate) gen: &'a CbindgenBuilder,
    pub(crate) registry: &'a R,
}

/// Whether the mirror's declared field wire and the conversion's are the same
/// C type.
///
/// Compared **modulo pointer constness**, because the mirror declares one
/// field and the two directions read it differently: a `String` field is
/// `*mut c_char` in the struct C owns and frees, and the decode takes the same
/// memory as `*const`. That is one wire and two readings of it, not two wires,
/// and C says so with a cast rather than a second field.
fn same_wire(declared: &syn::Type, produced: &syn::Type) -> bool {
    fn strip(t: &syn::Type) -> syn::Type {
        match t {
            syn::Type::Ptr(p) => {
                let inner = strip(&p.elem);
                syn::parse_quote!(*const #inner)
            }
            other => other.clone(),
        }
    }
    TypeKey::from_type(&strip(declared)) == TypeKey::from_type(&strip(produced))
}

/// Whether the mirror holds this field as `MaybeUninit<T>` where the
/// conversion produces a bare `T`.
///
/// One field, two readings again: the decode needs somewhere a C caller's
/// arbitrary bytes can legally sit until they are checked, and the encode
/// writes a value that is already valid.
fn held_uninit(declared: &syn::Type, produced: &syn::Type) -> bool {
    let syn::Type::Path(p) = declared else {
        return false;
    };
    let Some(last) = p.path.segments.last() else {
        return false;
    };
    if last.ident != "MaybeUninit" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return false;
    };
    matches!(args.args.first(), Some(syn::GenericArgument::Type(inner))
        if TypeKey::from_type(inner) == TypeKey::from_type(produced))
}

/// Whether a generated converter can fail, read off its own return type.
fn fallible(function: &syn::ItemFn) -> bool {
    matches!(&function.sig.output, syn::ReturnType::Type(_, t) if is_result(t))
}

/// A refusal naming the crossing that could not be answered.
fn refuse(at: At<'_>, why: &str) -> String {
    format!("Cbindgen: {} ({why})", at.crossing.key())
}

impl<R: Conversions<()>> CCompile<'_, R> {
    fn wrap(&self, at: At<'_>, why: &str, conv: Option<ConverterImpl<()>>) -> Frag<Self> {
        conv.map(|c| CFrag::from_converter(at, c))
            .ok_or_else(|| refuse(at, why))
    }
}

impl<R: Conversions<()>> Compile for CCompile<'_, R> {
    type Fragment = CFrag;
    /// C keeps its own per-site emission for now: the exported signature, the
    /// call and the cleanup are built in `emit.rs` from the resolved registry.
    type Plan = ();
    type Error = String;

    fn atomic(&mut self, cx: &mut Cx<'_>, at: At<'_>) -> Frag<Self> {
        let ty = at.crossing.spelled();
        // The field row, where a value crosses differently **inside a
        // `data_struct`'s mirror** than it does on its own. Two types have one:
        // `bool`, whose field shares a mirror with the decode that normalises
        // it, and `String`, whose field decodes a null pointer leniently so one
        // field cannot make a whole struct's decode fallible.
        // A `Box`-over-handle rides in a union arm as a bare pointer the C side
        // owns, and that is the only place C crosses one — a handle parameter
        // is spelled `Blob` and reclaimed from its own pointer.
        //
        // Keyed by the **spelling** rather than by a row of its own: a row is
        // filed under `Crossing::key`, which strips `Box`, so `Box<Blob>` and
        // `Blob` share one row and could not be told apart there. A fragment is
        // keyed by the spelling, which is exactly the distinction needed.
        if *at.recipe == crate::rows::payload() {
            let conv = match at.crossing.assembly() {
                Assembly::Construct => self.gen.in_boxed_payload(ty),
                Assembly::Deconstruct => self.gen.out_boxed_payload(ty),
            };
            return self.wrap(at, "no payload reading for this handle", conv);
        }
        if *at.recipe == crate::rows::in_field() {
            let conv = match at.crossing.assembly() {
                Assembly::Construct => self
                    .gen
                    .in_bool(ty)
                    .or_else(|| self.gen.in_string_field(ty)),
                // Only `bool` reads differently on the way out; a `String`
                // field is allocated exactly as a `String` return is.
                Assembly::Deconstruct => self.gen.out_bool_field(ty),
            };
            return self.wrap(at, "no field reading for this type", conv);
        }
        let conv = match at.crossing.assembly() {
            Assembly::Construct => self
                .gen
                .in_custom(ty, self.registry, cx.emit())
                .or_else(|| self.gen.in_opaque_handle(ty))
                .or_else(|| self.gen.in_value_opaque(ty, self.registry))
                .or_else(|| self.gen.in_enum(ty, self.registry))
                .or_else(|| self.gen.in_string(ty))
                .or_else(|| self.gen.in_str(ty))
                .or_else(|| self.gen.in_bool(ty))
                .or_else(|| self.gen.in_scalar(ty))
                .or_else(|| self.gen.in_borrow(ty))
                .or_else(|| {
                    // A callback whose signature was declared: its own
                    // `#[repr(C)]` closure struct.
                    let args = ty.callback_args()?;
                    self.gen.dispatch_fn_input(args, self.registry)
                }),
            Assembly::Deconstruct => self
                .gen
                .out_custom(ty, self.registry, cx.emit())
                .or_else(|| self.gen.out_terminal(ty, self.registry, cx.emit()))
                .or_else(|| self.gen.out_borrow_or_result(ty)),
        };
        self.wrap(at, "no C representation for this type", conv)
    }

    fn optional(&mut self, _cx: &mut Cx<'_>, at: At<'_>, inner: &CFrag) -> Frag<Self> {
        let Some(elem) = at.crossing.value().optional_inner() else {
            return Err(refuse(
                at,
                "an optional row over a type that is not optional",
            ));
        };
        let conv = match at.crossing.assembly() {
            Assembly::Construct => self.gen.in_option(elem, inner),
            Assembly::Deconstruct => Some(self.gen.out_arity_marker("option", elem)),
        };
        self.wrap(at, "no C representation for this optional", conv)
    }

    fn sequence(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _elements: Mode,
        _inner: &CFrag,
    ) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let conv = match at.crossing.assembly() {
            // A `&[E]` is the only run C builds a Rust value out of, and it does
            // it zero-copy from the caller's own block.
            Assembly::Construct => self.gen.in_slice(ty),
            // A `&[E]` callback argument is delivered by reference and has its
            // own marker; every other run is lowered structurally from the
            // element's.
            Assembly::Deconstruct => self
                .gen
                .out_slice_marker(ty)
                .or_else(|| Some(self.gen.out_arity_marker("vec", ty.sequence_elem()?))),
        };
        self.wrap(at, "no C representation for this run", conv)
    }

    fn identity(&mut self, _cx: &mut Cx<'_>, at: At<'_>, _inner: &CFrag) -> Frag<Self> {
        Err(refuse(at, "Cbindgen declares no identity rows"))
    }

    fn construct(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _func: &Function,
        _args: Parts<'_, Self>,
    ) -> Frag<Self> {
        Err(refuse(at, "Cbindgen declares no constructor rows"))
    }

    fn value_form(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _func: &Function,
        _parts: Parts<'_, Self>,
    ) -> Frag<Self> {
        Err(refuse(at, "Cbindgen declares no value-form rows"))
    }

    fn fields(&mut self, _cx: &mut Cx<'_>, at: At<'_>, parts: Parts<'_, Self>) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let key = ty.key();
        // A tagged union's arm is a product whose parts do not assemble into a
        // value of their own: they are bound by a `match` and rebuilt on the
        // other side. So this hands `choice` the converted payloads and builds
        // no function — which is what "compose a product's fragment from its
        // parts' fragments" means when the product is one arm of a sum.
        if self.gen.tagged_unions.contains_key(&key) {
            // Which alternative these parts belong to, for a refusal that names
            // the arm the way the declaration writes it.
            let arm_name = self
                .gen
                .union_arm_name(&key, self.registry, parts)
                .unwrap_or_default();
            // Acceptance is decided here, before a part is asked to convert:
            // `payload_field_wire` is where a union says what one of its fields
            // can carry, and its refusals name the shape rather than the
            // missing converter — a `Vec` needs two C wires and a union field
            // has one, which is a fact about the union and not about the
            // sequence. Asking the registry for a `Vec<u8>` conversion first
            // would report an unresolved crossing instead.
            //
            // A refusal here is a **declaration** error, not a missing
            // conversion, so it aborts with the reason rather than leaving the
            // crossing unresolved — the same contract the walk this replaces
            // had, and the same one `prereq_data_structs` uses for a field it
            // cannot mirror.
            //
            // Only the shapes that can **never** cross. "This payload has no
            // converter yet" was the other half of the old check and is not a
            // question any more: the part in hand *is* the conversion, so a
            // payload that reached here has one by construction.
            for (part, _) in parts {
                if let Err(why) = self.gen.payload_shape_refusal(&part.ty) {
                    panic!(
                        "Cbindgen::tagged_union: payload `{}::{}{}` of type `{}` cannot cross: {}",
                        type_short(&key),
                        arm_name,
                        match &part.name.parse::<usize>() {
                            Ok(_) => String::new(),
                            Err(_) => format!(".{}", part.name),
                        },
                        part.ty,
                        why
                    );
                }
            }
            let exprs = parts
                .iter()
                .enumerate()
                .map(|(i, (part, frag))| {
                    let bind = format_ident!("__f{}", i);
                    let conv = &frag.function.sig.ident;
                    let call = if fallible(&frag.function) {
                        quote!(#conv(#bind)?)
                    } else {
                        quote!(#conv(#bind))
                    };
                    // The union holds a payload whose bytes C may write —
                    // a nested enum or a `bool` — as `MaybeUninit`, so the
                    // decode can check them before assuming them valid. Same
                    // holding form a struct's mirror field takes, and the same
                    // reason: the wrap belongs to whatever holds the value, not
                    // to the value's own conversion.
                    match (
                        at.crossing.assembly(),
                        self.gen.payload_field_wire(&part.ty),
                    ) {
                        (Assembly::Deconstruct, Ok(w)) if held_uninit(&w, &frag.destination) => {
                            quote!(::core::mem::MaybeUninit::new(#call))
                        }
                        _ => call,
                    }
                })
                .collect();
            // A payload built inline from the handle's own C name is not a
            // crossing the registry has to resolve — the old walk did not make
            // one either. Marking it reachable would demand a whole-value
            // conversion for `Box<Blob>`, which C has none of: a handle
            // parameter is spelled `Blob`.
            let subs: Vec<TypeKey> = parts
                .iter()
                .filter(|(p, _)| {
                    self.gen.declared_opaque_payload_inner(&p.ty).is_none()
                        && r_boxed_inner(&p.ty).is_none()
                })
                .map(|(p, _)| p.ty.key())
                .collect();
            return Ok(CFrag {
                destination: syn::parse_quote!(()),
                function: syn::parse_quote!(
                    #[allow(dead_code)]
                    fn __cbg_arm() {}
                ),
                niches: Niches::empty(),
                subs: subs.clone(),
                arm: Some(Arm {
                    fallible: parts.iter().any(|(_, f)| fallible(&f.function)),
                    subs,
                    exprs,
                }),
                yields: Yield {
                    ty: at.crossing.value().stripped_key(),
                    mode: at.crossing.mode(),
                    validity: Validity::SelfSufficient,
                },
            });
        }
        let c_struct = self.gen.c_type_ident(&key);
        let src = self.gen.src_ty_of(&key);
        // Each part converts itself. The three statements of a field's wire —
        // this conversion, its twin in the other direction, and the mirror's
        // own field list — collapse to one: the part's fragment says it, and
        // `data_field_wire` is checked against it rather than re-deriving it.
        for (part, frag) in parts {
            let declared = self.gen.data_field_wire(&part.ty);
            if declared.as_ref().is_some_and(|w| {
                !same_wire(w, &frag.destination) && !held_uninit(w, &frag.destination)
            }) {
                return Err(refuse(
                    at,
                    &format!(
                        "field `{}` crosses as `{}` and its mirror declares `{}`",
                        part.name,
                        frag.destination.to_token_stream(),
                        declared.to_token_stream(),
                    ),
                ));
            }
        }
        let names: Vec<syn::Ident> = parts
            .iter()
            .map(|(p, _)| format_ident!("{}", p.name))
            .collect();
        let calls: Vec<TokenStream> = parts
            .iter()
            .zip(&names)
            .map(|((part, frag), fname)| {
                let conv = &frag.function.sig.ident;
                let call = quote!(#conv(v.#fname));
                let call = if fallible(&frag.function) {
                    quote!(#call?)
                } else {
                    call
                };
                // The mirror holds a tagged-union field as `MaybeUninit`, so a
                // C caller can hand over any discriminant and the decode can
                // check it before assuming it initialised. That is the
                // struct's holding form rather than the union's wire, so the
                // wrap belongs here and the union's own conversion stays the
                // one both directions use.
                match (at.crossing.assembly(), self.gen.data_field_wire(&part.ty)) {
                    (Assembly::Deconstruct, Some(w)) if held_uninit(&w, &frag.destination) => {
                        quote!(::core::mem::MaybeUninit::new(#call))
                    }
                    _ => call,
                }
            })
            .collect();
        let any_fallible = parts.iter().any(|(_, f)| fallible(&f.function));
        let subs: Vec<TypeKey> = parts.iter().map(|(p, _)| p.ty.key()).collect();

        match at.crossing.assembly() {
            Assembly::Construct => {
                let name = CbindgenBuilder::in_name_of(&key);
                let function: syn::ItemFn = if any_fallible {
                    syn::parse_quote!(
                        #[allow(non_snake_case, unused_variables, dead_code)]
                        pub(crate) unsafe fn #name(
                            v: #c_struct,
                        ) -> ::core::result::Result<#src, ::std::string::String> {
                            ::core::result::Result::Ok(#src { #(#names: #calls),* })
                        }
                    )
                } else {
                    syn::parse_quote!(
                        #[allow(non_snake_case, unused_variables, dead_code)]
                        pub(crate) unsafe fn #name(v: #c_struct) -> #src {
                            #src { #(#names: #calls),* }
                        }
                    )
                };
                Ok(CFrag::from_converter(
                    at,
                    ConverterImpl {
                        subs,
                        destination: syn::parse_quote!(#c_struct),
                        function,
                        pre_stages: vec![],
                        niches: Niches::empty(),
                        metadata: (),
                    },
                ))
            }
            Assembly::Deconstruct => {
                let name = CbindgenBuilder::out_name_of(&key);
                let function: syn::ItemFn = if any_fallible {
                    syn::parse_quote!(
                        #[allow(non_snake_case, unused_variables, dead_code)]
                        pub(crate) fn #name(
                            v: #src,
                        ) -> ::core::result::Result<#c_struct, ::std::string::String> {
                            ::core::result::Result::Ok(#c_struct { #(#names: #calls),* })
                        }
                    )
                } else {
                    syn::parse_quote!(
                        #[allow(non_snake_case, unused_variables, dead_code)]
                        pub(crate) fn #name(v: #src) -> #c_struct {
                            #c_struct { #(#names: #calls),* }
                        }
                    )
                };
                Ok(CFrag::from_converter(
                    at,
                    ConverterImpl {
                        subs,
                        destination: syn::parse_quote!(#c_struct),
                        function,
                        pre_stages: vec![],
                        niches: Niches::empty(),
                        metadata: (),
                    },
                ))
            }
        }
    }

    fn choice(
        &mut self,
        cx: &mut Cx<'_>,
        at: At<'_>,
        arms: &[(&Alternative, &CFrag)],
    ) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let key = ty.key();
        let cname = self.gen.c_type_ident(&key);
        let src = self.gen.src_ty_of(&key);
        let emit = cx.emit();
        let construct = at.crossing.assembly() == Assembly::Construct;

        let mut subs: Vec<TypeKey> = Vec::new();
        let mut fallible_any = false;
        let mut match_arms: Vec<TokenStream> = Vec::new();
        for (alternative, frag) in arms {
            let Some(arm) = frag.arm.as_ref() else {
                return Err(refuse(at, "an arm that composed no payload"));
            };
            subs.extend(arm.subs.iter().cloned());
            fallible_any |= arm.fallible;
            let vident = &alternative.name;
            let binds: Vec<syn::Ident> = (0..alternative.fields.len())
                .map(|i| format_ident!("__f{}", i))
                .collect();
            let bound: Vec<TokenStream> = alternative
                .fields
                .iter()
                .zip(&binds)
                .map(|(f, b)| f.bind(b))
                .collect();
            let built: Vec<TokenStream> = alternative
                .fields
                .iter()
                .zip(&arm.exprs)
                .map(|(f, e)| f.bind(e))
                .collect();
            // The alternative's own delimiters on both sides, so a tuple arm
            // and a braced one each spell themselves.
            let (from_head, to_head) = if construct {
                (quote!(#cname::#vident), quote!(#src::#vident))
            } else {
                (quote!(#src::#vident), quote!(#cname::#vident))
            };
            let from = emit.shape_alternative(alternative, from_head, &bound);
            let to = emit.shape_alternative(alternative, to_head, &built);
            match_arms.push(quote!(#from => #to,));
        }

        let conv = if construct {
            let name = CbindgenBuilder::in_name_of(&key);
            let bad = format!(
                "invalid tag {{}} for `{cname}` (expected 0..{})",
                arms.len()
            );
            // A C-supplied mirror may hold any discriminant, so the tag is
            // checked before the value is assumed initialised. Neither the tag
            // nor the check is a crossing — the adapter invents both, which is
            // why no row mentions them.
            let guard = self.gen.tag_guard(
                &cname,
                arms.len(),
                quote!(v),
                quote!(return ::core::result::Result::Err(::std::format!(#bad, __tag));),
            );
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: ::core::mem::MaybeUninit<#cname>,
                ) -> ::core::result::Result<#src, ::std::string::String> {
                    #guard
                    let v = v.assume_init();
                    ::core::result::Result::Ok(match v { #(#match_arms)* })
                }
            );
            ConverterImpl {
                subs,
                destination: syn::parse_quote!(::core::mem::MaybeUninit<#cname>),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            }
        } else {
            if fallible_any {
                return Err(refuse(
                    at,
                    "a payload whose encode can fail, which a union has no way to report",
                ));
            }
            let name = CbindgenBuilder::out_name_of(&key);
            // `MaybeUninit`, matching the wire the decode takes: one C type for
            // both directions, so a union returned by one function can be
            // handed straight to another that takes one. The value written is
            // always initialised — only the *type* has room for a discriminant
            // C might not have written.
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src) -> ::core::mem::MaybeUninit<#cname> {
                    ::core::mem::MaybeUninit::new(match v { #(#match_arms)* })
                }
            );
            ConverterImpl {
                subs,
                destination: syn::parse_quote!(::core::mem::MaybeUninit<#cname>),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            }
        };
        let _ = fallible_any;
        Ok(CFrag::from_converter(at, conv))
    }

    fn callback(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _args: &[&CFrag],
        _result: Option<&CFrag>,
    ) -> Frag<Self> {
        let Some(args) = at.crossing.value().callback_args() else {
            return Err(refuse(at, "a callback row over a type that is not one"));
        };
        let conv = self.gen.dispatch_fn_input(args, self.registry);
        self.wrap(at, "undeclared callback signature", conv)
    }

    /// C hands out borrows deliberately, so a returned one is not an error.
    ///
    /// A zero-copy accessor — `fn(&Sample) -> &ZBytes` — crosses as
    /// `*const zbytes_t`, and C's own contract is that a `const` pointer is
    /// non-owning: the caller neither frees it nor outlives the value it points
    /// into. That is the target's ownership model rather than a weaker check,
    /// and it is why the default strict reading belongs to the JVM and not
    /// here.
    fn tolerates(&self, _role: &Role) -> Validity {
        Validity::Borrowed
    }

    fn plan(&mut self, _cx: &mut Cx<'_>, _bound: &Bound, _root: &CFrag) -> Result<(), String> {
        Ok(())
    }
}
