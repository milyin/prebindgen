//! A toy adapter written against the public API only: its own wire enum,
//! type-level declarations with a per-parameter override, recursion over
//! `shape`, and a foreign-side writer that reads the `Form` tree.

use prebindgen::SourceLocation;
use prebindgen_flat::{
    flat::{Function, ScalarKind, Type, TypeKind, TypeRef},
    Flat,
};
use prebindgen_tools::{
    convert, fun, names, shape, Access, Form, FormKind, Input, Output, Overrides, Place, Qualifier,
    ResolvedConversion, Seg, Shape, Wire, WireType,
};
use proc_macro2::TokenStream;
use quote::quote;

/// The toy language's closed set of boundary types. `Long` is `i64` either
/// way; `unsigned` is what the foreign side makes of it.
#[derive(Clone, Debug, PartialEq)]
enum Toy {
    Int,
    Long { unsigned: bool },
    Flag,
    Handle(String),
}

impl WireType for Toy {
    fn rust(&self) -> TokenStream {
        match self {
            Toy::Int => quote!(i32),
            Toy::Long { .. } => quote!(i64),
            Toy::Flag => quote!(u8),
            Toy::Handle(_) => quote!(*mut ::core::ffi::c_void),
        }
    }

    fn placeholder(&self) -> TokenStream {
        match self {
            Toy::Handle(_) => quote!(::core::ptr::null_mut()),
            _ => quote!(0),
        }
    }
}

/// The toy's declarations.
#[derive(Debug)]
enum Decl {
    Handle,
    Record,
    Sum,
    Convert(Box<ResolvedConversion>),
}

struct Adapter<'f> {
    flat: &'f Flat,
    q: Qualifier<'f>,
    decls: Overrides<Decl>,
}

fn name_of(ty: &TypeRef) -> String {
    match ty.borrow_target().unwrap_or(ty).kind() {
        TypeKind::Named { id, .. } => id.name.clone(),
        _ => panic!("`{ty}` is not named"),
    }
}

impl Adapter<'_> {
    fn ty(&self, src: &str) -> TypeRef {
        self.flat.classify(&syn::parse_str(src).unwrap()).unwrap()
    }

    fn input(&self, ty: &TypeRef, place: &Place) -> Result<Input<Toy>, String> {
        let w = place.ident("");
        let wire = |t: Toy| Wire::new(w.clone(), t);
        Ok(match shape(ty, |t| self.decls.get(place, t))? {
            Shape::Scalar(ScalarKind::I32) => Input::wire(ty, wire(Toy::Int), quote!(#w)),
            Shape::Scalar(ScalarKind::U64) => {
                Input::wire(ty, wire(Toy::Long { unsigned: true }), quote!((#w as u64)))
            }
            Shape::Option(inner) => {
                let p = place.ident("present");
                let inner = self.input(inner, &place.at(Seg::Some))?;
                Input::optional(
                    ty,
                    Some(Wire::new(p.clone(), Toy::Flag)),
                    quote!(#p != 0),
                    inner,
                )
            }
            Shape::Declared {
                ty: dty,
                declaration,
            } => {
                let name = name_of(dty);
                let input = match declaration {
                    Decl::Handle => {
                        let t = self.q.path(&names::ident(&name));
                        let wire = wire(Toy::Handle(name.clone()));
                        return Ok(match Access::of(dty) {
                            Access::Owned => Input::wire(
                                ty,
                                wire,
                                quote!(*::std::boxed::Box::from_raw(#w as *mut #t)),
                            ),
                            _ => Input::wire(ty, wire, quote!(&*(#w as *const #t))),
                        });
                    }
                    Decl::Record => {
                        let Some(Type::Struct(s)) = self.flat.declared_type(name.as_str()) else {
                            return Err(format!("`{name}` is not a struct"));
                        };
                        let fields = s
                            .fields
                            .iter()
                            .map(|f| self.input(&f.ty, &place.at(Seg::field(f))))
                            .collect::<Result<_, _>>()?;
                        Input::record(&self.q, s, fields)
                    }
                    Decl::Sum => {
                        let Some(Type::Variant(v)) = self.flat.declared_type(name.as_str()) else {
                            return Err(format!("`{name}` is not a sum"));
                        };
                        let mut alts = Vec::new();
                        for a in &v.alternatives {
                            let at = place.at(Seg::Alt(names::bare(&a.name)));
                            let fields = a
                                .fields
                                .iter()
                                .map(|f| self.input(&f.ty, &at.at(Seg::field(f))))
                                .collect::<Result<_, _>>()?;
                            alts.push(fields);
                        }
                        Input::sum(&self.q, v, Wire::new(place.ident("tag"), Toy::Int), alts)
                    }
                    Decl::Convert(c) => {
                        c.decode(&self.q, |repr| self.input(repr, &place.at(Seg::Repr)))?
                    }
                };
                match Access::of(dty) {
                    Access::Owned => input,
                    _ => input.map(|e| quote!(&#e)),
                }
            }
            s => return Err(format!("`{ty}`: no toy form for {s:?}")),
        })
    }

    fn output(&self, ty: &TypeRef, place: &Place, v: &TokenStream) -> Result<Output<Toy>, String> {
        let w = place.ident("");
        let wire = |t: Toy| Wire::new(w.clone(), t);
        Ok(match shape(ty, |t| self.decls.get(place, t))? {
            Shape::Unit => Output::unit(ty, v),
            Shape::Scalar(ScalarKind::I32) => Output::wire(ty, wire(Toy::Int), v),
            Shape::Scalar(ScalarKind::U64) => {
                Output::wire(ty, wire(Toy::Long { unsigned: true }), quote!((#v as i64)))
            }
            Shape::Option(inner) => {
                let p = Wire::new(place.ident("present"), Toy::Flag);
                Output::optional(ty, Some((p, quote!(1))), v, |x| {
                    self.output(inner, &place.at(Seg::Some), &x)
                })?
            }
            Shape::Declared {
                ty: dty,
                declaration,
            } if Access::of(dty) == Access::Owned => {
                let name = name_of(dty);
                match declaration {
                    Decl::Handle => Output::wire(
                        ty,
                        wire(Toy::Handle(name)),
                        quote!(::std::boxed::Box::into_raw(::std::boxed::Box::new(#v)) as *mut ::core::ffi::c_void),
                    ),
                    Decl::Record => {
                        let Some(Type::Struct(s)) = self.flat.declared_type(name.as_str()) else {
                            return Err(format!("`{name}` is not a struct"));
                        };
                        Output::record(&self.q, s, v, |f, b| {
                            self.output(&f.ty, &place.at(Seg::field(f)), &b)
                        })?
                    }
                    Decl::Sum => {
                        let Some(Type::Variant(sum)) = self.flat.declared_type(name.as_str())
                        else {
                            return Err(format!("`{name}` is not a sum"));
                        };
                        let tag = Wire::new(place.ident("tag"), Toy::Int);
                        Output::sum(&self.q, sum, tag, v, |a, f, b| {
                            let at = place.at(Seg::Alt(names::bare(&a.name)));
                            self.output(&f.ty, &at.at(Seg::field(f)), &b)
                        })?
                    }
                    Decl::Convert(c) => c.encode(&self.q, v, |repr, r| {
                        self.output(repr, &place.at(Seg::Repr), &r)
                    })?,
                }
            }
            s => return Err(format!("`{ty}`: no toy form for {s:?}")),
        })
    }

    /// A generated element: one wrapper around one source function.
    fn wrapper(&self, f: &Function) -> Result<TokenStream, String> {
        let el = Place::new(names::bare(&f.name));
        let mut params = Vec::new();
        let mut args = Vec::new();
        for p in &f.params {
            let input = self.input(&p.ty, &el.at(Seg::Param(names::bare(&p.name))))?;
            params.extend(input.wires().iter().map(|w| w.decl()));
            args.push(input.expr);
        }
        let out = self.output(&f.ret, &el.at(Seg::Return), &quote!(__ret))?;
        let ret_tys = out.wires().iter().map(|w| w.ty.rust()).collect::<Vec<_>>();
        let (name, callee, e) = (&f.name, self.q.path(&f.name), &out.expr);
        Ok(quote! {
            pub unsafe fn #name(#(#params),*) -> ::core::result::Result<(#(#ret_tys,)*), ::std::string::String> {
                let __ret = #callee(#(#args),*);
                ::core::result::Result::Ok(#e)
            }
        })
    }
}

/// The foreign side: a signature read off the form alone.
fn foreign(f: &Form<Toy>) -> String {
    match &f.kind {
        FormKind::Wire(w) => match &w.ty {
            Toy::Int => "Int".into(),
            Toy::Long { unsigned: true } => "ULong".into(),
            Toy::Long { unsigned: false } => "Long".into(),
            Toy::Flag => "Boolean".into(),
            Toy::Handle(class) => class.clone(),
        },
        FormKind::Via(repr) => foreign(repr),
        FormKind::Parts(parts) => {
            let parts: Vec<String> = parts
                .iter()
                .map(|(s, f)| format!("{s:?}={}", foreign(f)))
                .collect();
            format!("{}({})", f.ty, parts.join(", "))
        }
        FormKind::Optional { inner, .. } => format!("{}?", foreign(inner)),
        FormKind::Sum { alts, .. } => {
            let alts: Vec<String> = alts
                .iter()
                .map(|a| {
                    let fs: Vec<String> = a.iter().map(|(_, f)| foreign(f)).collect();
                    format!("[{}]", fs.join(", "))
                })
                .collect();
            format!("{} of {}", f.ty, alts.join(" | "))
        }
        FormKind::Seq { elem, .. } => format!("List<{}>", foreign(elem)),
    }
}

const SRC: &str = r#"
    pub struct Point { pub x: i32, pub y: u64 }
    pub struct Storage;
    pub struct Millis(pub u64);
    pub fn millis_from(v: u64) -> Millis { Millis(v) }
    pub fn millis_to(m: &Millis) -> u64 { m.0 }
    pub enum Reading { Empty, At(Point), Span { from: Millis, to: Millis } }
    pub fn send(p: Point, o: Option<Point>, s: &Storage, m: Millis) -> Reading { todo!() }
    pub fn send_raw(p: Point) -> Option<Millis> { todo!() }
"#;

fn adapter(flat: &Flat) -> Adapter<'_> {
    let mut a = Adapter {
        flat,
        q: Qualifier::new(flat),
        decls: Overrides::new(),
    };
    let mut decls = Overrides::new();
    decls.ty(&a.ty("Point"), Decl::Record);
    decls.ty(&a.ty("Storage"), Decl::Handle);
    decls.ty(&a.ty("Reading"), Decl::Sum);
    let millis = convert!(Millis)
        .input(fun!(millis_from))
        .output(fun!(millis_to))
        .resolve(flat)
        .unwrap();
    decls.ty(&a.ty("Millis"), Decl::Convert(Box::new(millis)));
    // `send_raw` takes its `p` as a handle; every other `Point` is a record.
    decls.at(
        Place::new("send_raw").at(Seg::Param("p".into())),
        Decl::Handle,
    );
    a.decls = decls;
    a
}

fn flat() -> Flat {
    let loc = SourceLocation {
        crate_name: Some("src".into()),
        ..Default::default()
    };
    let items = syn::parse_file(SRC).unwrap().items.into_iter();
    Flat::builder()
        .items(items.map(|i| (i, loc.clone())))
        .build()
        .unwrap()
}

fn norm(t: impl ToString) -> String {
    t.to_string().replace(' ', "")
}

fn wires<W: WireType>(ws: Vec<&Wire<W>>) -> Vec<(String, W)> {
    ws.into_iter()
        .map(|w| (w.name.to_string(), w.ty.clone()))
        .collect()
}

#[test]
fn parameters_cross_by_their_decisions() {
    let flat = flat();
    let a = adapter(&flat);
    let send = flat.function("send").unwrap();
    let el = Place::new("send");
    let param = |i: usize| {
        let p = &send.params[i];
        a.input(&p.ty, &el.at(Seg::Param(names::bare(&p.name))))
            .unwrap()
    };

    // A record: its fields' wires, named by place, rebuilt in source order.
    let p = param(0);
    assert_eq!(
        wires(p.wires()),
        [
            ("p_x".into(), Toy::Int),
            ("p_y".into(), Toy::Long { unsigned: true })
        ]
    );
    assert_eq!(norm(&p.expr), "src::Point{x:p_x,y:(p_yasu64)}");
    assert_eq!(
        foreign(&p.form),
        r#"Point(Field("x")=Int, Field("y")=ULong)"#
    );

    // An optional record: a presence wire beside the inner record's.
    let o = param(1);
    assert_eq!(
        wires(o.wires())
            .iter()
            .map(|w| w.0.as_str())
            .collect::<Vec<_>>(),
        ["o_present", "o_x", "o_y"]
    );
    assert_eq!(
        foreign(&o.form),
        r#"Point(Field("x")=Int, Field("y")=ULong)?"#
    );

    // A borrowed handle.
    let s = param(2);
    assert_eq!(
        wires(s.wires()),
        [("s".into(), Toy::Handle("Storage".into()))]
    );
    assert_eq!(norm(&s.expr), "&*(sas*constsrc::Storage)");

    // A converted type: its representation's wire, under a Via node.
    let m = param(3);
    assert!(matches!(m.form.kind, FormKind::Via(_)));
    assert_eq!(foreign(&m.form), "ULong");
    assert_eq!(
        norm(&m.expr),
        "{let__repr=(masu64);src::millis_from(__repr)}"
    );
}

#[test]
fn a_place_override_replaces_the_type_default() {
    let flat = flat();
    let a = adapter(&flat);
    let f = flat.function("send_raw").unwrap();
    let p = a
        .input(
            &f.params[0].ty,
            &Place::new("send_raw").at(Seg::Param("p".into())),
        )
        .unwrap();
    // The foreign side learns of the override from the form alone.
    assert_eq!(foreign(&p.form), "Point");
    assert_eq!(
        wires(p.wires()),
        [("p".into(), Toy::Handle("Point".into()))]
    );
}

#[test]
fn results_leave_by_their_decisions() {
    let flat = flat();
    let a = adapter(&flat);
    let ret = |f: &str| {
        a.output(
            &flat.function(f).unwrap().ret,
            &Place::new(f).at(Seg::Return),
            &quote!(v),
        )
        .unwrap()
    };

    // A sum: the tag, then every alternative's wires.
    let r = ret("send");
    assert_eq!(
        wires(r.wires())
            .iter()
            .map(|w| w.0.as_str())
            .collect::<Vec<_>>(),
        [
            "ret_tag",
            "ret_at_v0_x",
            "ret_at_v0_y",
            "ret_span_from",
            "ret_span_to"
        ]
    );
    assert_eq!(
        foreign(&r.form),
        r#"Reading of [] | [Point(Field("x")=Int, Field("y")=ULong)] | [ULong, ULong]"#
    );
    syn::parse2::<syn::Expr>(r.expr.clone()).expect("a Rust expression");
    // The taken alternative fills its wires; the others hold placeholders.
    assert!(norm(&r.expr).contains("(0,0,0,0,0)"), "{}", r.expr);

    // An optional converted value.
    let o = ret("send_raw");
    assert_eq!(
        wires(o.wires()),
        [
            ("ret_present".into(), Toy::Flag),
            ("ret".into(), Toy::Long { unsigned: true })
        ]
    );
    assert_eq!(foreign(&o.form), "ULong?");
    syn::parse2::<syn::Expr>(o.expr).expect("a Rust expression");
}

#[test]
fn a_wrapper_assembles_from_its_parts() {
    let flat = flat();
    let a = adapter(&flat);
    for name in ["send", "send_raw"] {
        let item = a.wrapper(flat.function(name).unwrap()).unwrap();
        syn::parse2::<syn::ItemFn>(item.clone()).unwrap_or_else(|e| panic!("{e}: {item}"));
    }
}
