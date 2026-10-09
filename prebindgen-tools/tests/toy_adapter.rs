//! A toy adapter written against the public API only: its own wire enum,
//! a type that may cross three ways — whole as a handle, as its fields, or
//! built by a constructor — with the way chosen per place, and a
//! foreign-side writer that reads the resolved crossing.

use prebindgen::SourceLocation;
use prebindgen_flat::{
    flat::{Function, ScalarKind, Type, TypeKind, TypeRef},
    Flat,
};
use prebindgen_tools::{
    convert, fun, names, resolve, Access, Choices, Code, Crossing, Decode, Direction, Encode, In,
    Lower, Node, Optional, Out, Place, Presence, Seg, Sequence, Shape, Ways, Wire, WireType,
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

/// The toy's one whole-crossing declaration: an opaque handle.
struct Handle;

/// Everything the toy decided: the ways each type may cross, and which one
/// each occurrence takes, per direction.
struct Toyish<'f> {
    flat: &'f Flat,
    ways: Ways<'f, Handle>,
    inputs: Choices<In>,
    outputs: Choices<Out>,
}

/// The toy's boundary in direction `D`.
struct Boundary<'a, 'f, D: Direction> {
    toy: &'a Toyish<'f>,
    choices: &'a Choices<D>,
}

fn name_of(ty: &TypeRef) -> String {
    match ty.borrow_target().unwrap_or(ty).kind() {
        TypeKind::Named { id, .. } => id.name.clone(),
        _ => panic!("`{ty}` is not named"),
    }
}

impl<'f, D: Direction> Lower<'f, D> for Boundary<'_, 'f, D> {
    type Wire = Toy;
    type Leaf = Handle;
    type Error = String;

    fn ways(&self) -> &Ways<'f, Handle> {
        &self.toy.ways
    }

    fn choices(&self) -> &Choices<D> {
        self.choices
    }

    fn whole(
        &self,
        ty: &TypeRef,
        shape: Shape<'_, &Handle>,
        place: &Place,
    ) -> Result<Option<Wire<Toy>>, String> {
        let wire = |t| Some(Wire::new(place.ident(), t));
        Ok(match shape {
            Shape::Scalar(ScalarKind::I32) => wire(Toy::Int),
            Shape::Scalar(ScalarKind::U64) => wire(Toy::Long { unsigned: true }),
            Shape::Declared { .. } => wire(Toy::Handle(name_of(ty))),
            _ => None,
        })
    }

    fn presence(&self, _: &Crossing<'f, Toy, D>, place: &Place) -> Result<Presence<Toy>, String> {
        Ok(Presence::Flag(Wire::new(
            place.ident_with_suffix("present"),
            Toy::Flag,
        )))
    }

    fn sequence(&self, _: &Crossing<'f, Toy, D>, place: &Place) -> Result<Vec<Wire<Toy>>, String> {
        Err(format!("{place}: the toy has no sequences"))
    }

    fn tag(&self, place: &Place) -> Wire<Toy> {
        Wire::new(place.ident_with_suffix("tag"), Toy::Int)
    }
}

impl<'f> Decode<'f, Toy> for Toyish<'f> {
    type Error = String;

    fn wire(&self, at: &Crossing<'f, Toy, In>, wire: &Wire<Toy>) -> Result<Code, String> {
        let w = &wire.name;
        Ok(match &wire.ty {
            Toy::Long { .. } => Code::new(quote!((#w as u64))),
            Toy::Handle(name) => {
                let t = self.flat.declared_type(name).unwrap().name();
                match Access::of(at.ty()) {
                    Access::Owned => Code::new(quote!(*::std::boxed::Box::from_raw(#w as *mut #t))),
                    _ => Code::new(quote!(&*(#w as *const #t))),
                }
            }
            _ => Code::new(quote!(#w)),
        })
    }

    fn is_present(
        &self,
        _: &Crossing<'f, Toy, In>,
        opt: &Optional<'f, Toy, In>,
    ) -> Result<TokenStream, String> {
        match opt.presence() {
            Presence::Flag(f) => {
                let p = &f.name;
                Ok(quote!(#p != 0))
            }
            Presence::Niche => Err("the toy has no niches".into()),
        }
    }

    fn sequence(
        &self,
        at: &Crossing<'f, Toy, In>,
        _: &Sequence<'f, Toy, In>,
        _: prebindgen_tools::Input,
    ) -> Result<Code, String> {
        Err(format!("{}: the toy has no sequences", at.place()))
    }
}

impl<'f> Encode<'f, Toy> for Toyish<'f> {
    type Error = String;

    fn wire(
        &self,
        _: &Crossing<'f, Toy, Out>,
        wire: &Wire<Toy>,
        v: &TokenStream,
    ) -> Result<Code, String> {
        Ok(Code::new(match &wire.ty {
            Toy::Long { .. } => quote!((#v as i64)),
            Toy::Handle(_) => {
                quote!(::std::boxed::Box::into_raw(::std::boxed::Box::new(#v)) as *mut ::core::ffi::c_void)
            }
            _ => quote!(#v),
        }))
    }

    fn present(&self, _: &Crossing<'f, Toy, Out>, _: &Wire<Toy>) -> TokenStream {
        quote!(1)
    }

    fn sequence(
        &self,
        at: &Crossing<'f, Toy, Out>,
        _: &Sequence<'f, Toy, Out>,
        _: &prebindgen_tools::Output,
        _: &TokenStream,
    ) -> Result<Code, String> {
        Err(format!("{}: the toy has no sequences", at.place()))
    }
}

impl<'f> Toyish<'f> {
    fn input(&self, ty: &TypeRef, place: Place) -> Result<Crossing<'f, Toy, In>, String> {
        resolve(
            &Boundary {
                toy: self,
                choices: &self.inputs,
            },
            ty,
            place,
        )
    }

    fn output(&self, ty: &TypeRef, place: Place) -> Result<Crossing<'f, Toy, Out>, String> {
        resolve(
            &Boundary {
                toy: self,
                choices: &self.outputs,
            },
            ty,
            place,
        )
    }

    /// A generated element: one wrapper around one source function.
    fn wrapper(&self, f: &Function) -> Result<TokenStream, String> {
        let el = Place::new(names::bare(f.name.ident()));
        let mut params = Vec::new();
        let mut args = Vec::new();
        for p in &f.params {
            let c = self.input(&p.ty, el.at(Seg::Param(names::bare(&p.name))))?;
            params.extend(c.wires().iter().map(|w| w.decl()));
            args.push(c.decode(self)?.expr().clone());
        }
        let ret = self.output(&f.ret, el.at(Seg::Return))?;
        let ret_tys = ret.wires().iter().map(|w| w.ty.rust()).collect::<Vec<_>>();
        let (name, callee) = (f.name.ident(), &f.name);
        let result = ret.encode(self)?.apply(quote!(#callee(#(#args),*)));
        Ok(quote! {
            pub unsafe fn #name(#(#params),*) -> ::core::result::Result<(#(#ret_tys,)*), ::std::string::String> {
                ::core::result::Result::Ok(#result)
            }
        })
    }
}

/// The foreign side: a signature read off the crossing alone.
fn foreign<D: Direction>(c: &Crossing<'_, Toy, D>) -> String {
    match c.node() {
        Node::Wire(w) => match &w.ty {
            Toy::Int => "Int".into(),
            Toy::Long { unsigned: true } => "ULong".into(),
            Toy::Long { unsigned: false } => "Long".into(),
            Toy::Flag => "Boolean".into(),
            Toy::Handle(class) => class.clone(),
        },
        Node::Unit => "Unit".into(),
        Node::Converted(v) => foreign(v.repr()),
        Node::Fields(f) => {
            let fs: Vec<String> = f.fields().map(|(_, c)| foreign(c)).collect();
            format!("{}({})", f.item().name, fs.join(", "))
        }
        Node::Optional(o) => format!("{}?", foreign(o.inner())),
        Node::Alternatives(a) => {
            let arms: Vec<String> = a
                .arms()
                .iter()
                .map(|arm| {
                    let fs: Vec<String> = arm.fields().map(|(_, c)| foreign(c)).collect();
                    format!("[{}]", fs.join(", "))
                })
                .collect();
            format!("{} of {}", a.item().name, arms.join(" | "))
        }
        Node::Sequence(s) => format!("List<{}>", foreign(s.elem())),
        Node::Wrapped(w) => foreign(w.inner()),
        Node::Constructed(_) => format!("new {}", name_of(c.ty())),
    }
}

/// What the foreign side passes for a value built by a constructor: its
/// arguments.
fn constructor_args(c: &Crossing<'_, Toy, In>) -> Vec<String> {
    match c.node() {
        Node::Constructed(k) => k
            .args()
            .map(|(p, a)| format!("{}: {}", p.name, foreign(a)))
            .collect(),
        _ => panic!("not built by a constructor"),
    }
}

const SRC: &str = r#"
    pub struct Point { pub x: i32, pub y: u64 }
    pub fn point_new(y: u64) -> Point { Point { x: 0, y } }
    pub struct Storage;
    pub struct Millis(pub u64);
    pub fn millis_from(v: u64) -> Millis { Millis(v) }
    pub fn millis_to(m: &Millis) -> u64 { m.0 }
    pub enum Reading { Empty, At(Point), Span { from: Millis, to: Millis } }
    pub fn send(p: Point, o: Option<Point>, s: &Storage, m: Millis) -> Reading { todo!() }
    pub fn send_raw(p: Point) -> Option<Millis> { todo!() }
    pub fn send_built(p: Point) {}
"#;

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

fn ty(flat: &Flat, src: &str) -> TypeRef {
    flat.classify(&syn::parse_str(src).unwrap()).unwrap()
}

fn toy(flat: &Flat) -> Toyish<'_> {
    let declared = |name: &str| flat.declared_type(name).unwrap();
    let Type::Struct(point) = declared("Point") else {
        panic!()
    };
    let Type::Variant(reading) = declared("Reading") else {
        panic!()
    };
    let mut ways = Ways::new();
    // `Point` may cross three ways.
    let fields = ways.fields(point);
    let handle = ways.whole(point.type_ref(), Handle);
    let built = ways
        .constructed(flat.function("point_new").unwrap())
        .unwrap();
    ways.whole(&ty(flat, "Storage"), Handle);
    ways.alternatives(reading);
    let millis = convert!(Millis)
        .input(fun!(millis_from))
        .output(fun!(millis_to))
        .resolve(flat)
        .unwrap();
    ways.converted(millis);

    // Every `Point` crosses as its fields, but `send_raw` takes its `p` as
    // a handle and `send_built` builds its `p` from `point_new`'s
    // arguments. Out of Rust there is no constructor: the type says so.
    let mut inputs = Choices::new();
    inputs.choose(&ways, fields);
    let param = |f: &str| Place::new(f).at(Seg::Param("p".into()));
    inputs.choose_at(param("send_raw"), handle);
    inputs.choose_at(param("send_built"), built);
    let mut outputs = Choices::new();
    outputs.choose(&ways, fields);
    Toyish {
        flat,
        ways,
        inputs,
        outputs,
    }
}

fn norm(t: impl ToString) -> String {
    t.to_string().replace(' ', "")
}

fn wires<W: WireType>(ws: Vec<&Wire<W>>) -> Vec<(String, W)> {
    ws.into_iter()
        .map(|w| (w.name.to_string(), w.ty.clone()))
        .collect()
}

fn param<'f>(t: &Toyish<'f>, f: &'f Function, i: usize) -> Crossing<'f, Toy, In> {
    let p = &f.params[i];
    let place = Place::new(names::bare(f.name.ident())).at(Seg::Param(names::bare(&p.name)));
    t.input(&p.ty, place).unwrap()
}

#[test]
fn parameters_cross_by_their_choices() {
    let flat = flat();
    let t = toy(&flat);
    let send = flat.function("send").unwrap();

    // Fields: their wires, named by place, rebuilt in source order.
    let p = param(&t, send, 0);
    assert_eq!(
        wires(p.wires()),
        [
            ("p_x".into(), Toy::Int),
            ("p_y".into(), Toy::Long { unsigned: true })
        ]
    );
    assert_eq!(
        norm(p.decode(&t).unwrap().expr()),
        "src::Point{x:p_x,y:(p_yasu64)}"
    );
    assert_eq!(foreign(&p), "Point(Int, ULong)");

    // An option of fields: a presence wire beside the inner wires.
    let o = param(&t, send, 1);
    let names: Vec<String> = wires(o.wires()).into_iter().map(|w| w.0).collect();
    assert_eq!(names, ["o_present", "o_x", "o_y"]);
    assert_eq!(foreign(&o), "Point(Int, ULong)?");

    // A borrowed handle.
    let s = param(&t, send, 2);
    assert_eq!(
        wires(s.wires()),
        [("s".into(), Toy::Handle("Storage".into()))]
    );
    assert_eq!(
        norm(s.decode(&t).unwrap().expr()),
        "&*(sas*constsrc::Storage)"
    );

    // A converted type: its representation's wire.
    let m = param(&t, send, 3);
    assert!(matches!(m.node(), Node::Converted(_)));
    assert_eq!(foreign(&m), "ULong");
    assert_eq!(
        norm(m.decode(&t).unwrap().expr()),
        "{let__repr=(masu64);src::millis_from(__repr)}"
    );
}

#[test]
fn a_place_takes_its_own_way() {
    let flat = flat();
    let t = toy(&flat);

    // The same type, whole as a handle.
    let raw = param(&t, flat.function("send_raw").unwrap(), 0);
    assert_eq!(foreign(&raw), "Point");
    assert_eq!(
        wires(raw.wires()),
        [("p".into(), Toy::Handle("Point".into()))]
    );

    // And built by its constructor, from the constructor's arguments.
    let built = param(&t, flat.function("send_built").unwrap(), 0);
    assert_eq!(constructor_args(&built), ["y: ULong"]);
    assert_eq!(
        wires(built.wires()),
        [("p_y".into(), Toy::Long { unsigned: true })]
    );
    assert_eq!(
        norm(built.decode(&t).unwrap().expr()),
        "src::point_new((p_yasu64))"
    );
}

#[test]
fn a_choice_must_fit_its_place() {
    let flat = flat();
    let mut t = toy(&flat);
    let storage = t.ways.whole(&ty(&flat, "Storage"), Handle);
    // `send`'s `p` is a `Point`, not a `Storage`.
    t.inputs
        .choose_at(Place::new("send").at(Seg::Param("p".into())), storage);
    let send = flat.function("send").unwrap();
    let e = t
        .input(
            &send.params[0].ty,
            Place::new("send").at(Seg::Param("p".into())),
        )
        .unwrap_err();
    assert!(e.contains("the way chosen here is for `Storage`"), "{e}");
}

#[test]
fn a_type_with_several_ways_needs_a_choice() {
    let flat = flat();
    let mut t = toy(&flat);
    t.inputs = Choices::new();
    let send = flat.function("send").unwrap();
    let e = t
        .input(
            &send.params[0].ty,
            Place::new("send").at(Seg::Param("p".into())),
        )
        .unwrap_err();
    assert!(e.contains("`Point` has 3 ways to cross into Rust"), "{e}");
    // Out of Rust the constructor does not count.
    t.outputs = Choices::new();
    let e = t
        .output(&send.params[0].ty, Place::new("send").at(Seg::Return))
        .unwrap_err();
    assert!(e.contains("`Point` has 2 ways to cross out of Rust"), "{e}");
}

#[test]
fn results_leave_by_their_choices() {
    let flat = flat();
    let t = toy(&flat);
    let ret = |f: &str| {
        t.output(
            &flat.function(f).unwrap().ret,
            Place::new(f).at(Seg::Return),
        )
        .unwrap()
    };

    // A sum: the tag, then every alternative's wires.
    let r = ret("send");
    let names: Vec<String> = wires(r.wires()).into_iter().map(|w| w.0).collect();
    assert_eq!(
        names,
        [
            "ret_tag",
            "ret_at_v0_x",
            "ret_at_v0_y",
            "ret_span_from",
            "ret_span_to"
        ]
    );
    assert_eq!(
        foreign(&r),
        "Reading of [] | [Point(Int, ULong)] | [ULong, ULong]"
    );
    let e = r.encode(&t).unwrap().apply(quote!(v));
    syn::parse2::<syn::Expr>(e.clone()).expect("a Rust expression");
    // The taken alternative fills its wires; the others hold placeholders.
    assert!(norm(&e).contains("(0,0,0,0,0)"), "{e}");

    // An optional converted value.
    let o = ret("send_raw");
    assert_eq!(
        wires(o.wires()),
        [
            ("ret_present".into(), Toy::Flag),
            ("ret".into(), Toy::Long { unsigned: true })
        ]
    );
    assert_eq!(foreign(&o), "ULong?");
    syn::parse2::<syn::Expr>(o.encode(&t).unwrap().apply(quote!(v))).expect("a Rust expression");
}

#[test]
fn a_wrapper_assembles_from_its_crossings() {
    let flat = flat();
    let t = toy(&flat);
    for name in ["send", "send_raw", "send_built"] {
        let item = t.wrapper(flat.function(name).unwrap()).unwrap();
        syn::parse2::<syn::ItemFn>(item.clone()).unwrap_or_else(|e| panic!("{e}: {item}"));
    }
}
