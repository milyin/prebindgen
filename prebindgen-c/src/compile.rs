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
    recipe::{At, Carrier, Compile, Cx, Frag, Mode, Parts, Validity, Yield},
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
    /// What the fragment produces, which is all the registry reads of it.
    pub(crate) yields: Yield,
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
                .or_else(|| self.gen.in_tagged_union(ty, self.registry, cx.emit()))
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
        let c_struct = self.gen.c_type_ident(&key);
        let src = self.gen.src_ty_of(&key);
        // Each part converts itself. The three statements of a field's wire —
        // this conversion, its twin in the other direction, and the mirror's
        // own field list — collapse to one: the part's fragment says it, and
        // `data_field_wire` is checked against it rather than re-deriving it.
        for (part, frag) in parts {
            let declared = self.gen.data_field_wire(&part.ty);
            if declared
                .as_ref()
                .is_some_and(|w| !same_wire(w, &frag.destination))
            {
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
            .map(|((_, frag), fname)| {
                let conv = &frag.function.sig.ident;
                let call = quote!(#conv(v.#fname));
                if fallible(&frag.function) {
                    quote!(#call?)
                } else {
                    call
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
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _arms: &[(&Alternative, &CFrag)],
    ) -> Frag<Self> {
        Err(refuse(at, "Cbindgen states no choice rows yet"))
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

    fn plan(&mut self, _cx: &mut Cx<'_>, _bound: &Bound, _root: &CFrag) -> Result<(), String> {
        Ok(())
    }
}
