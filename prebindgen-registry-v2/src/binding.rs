//! What a frontend states before planning: the whole of what planning reads.
//!
//! A frontend turns its user's configuration into one [`Binding`] — the
//! carriers generated Rust may hold a value in, the representations values
//! cross as, the rules saying which values take which representation, and the
//! outputs to expose — and hands it to [`generate`](crate::generate) with its
//! [`Target`]. The registry plans from the binding and the source model alone:
//! no target code runs until the plan is complete. So the configuration can be
//! checked before planning, printed, and diffed.

use prebindgen_flat::TypeKey;

use crate::{
    decl::Declaration,
    target::{FailureCategory, Target, Terminal, Unsupported},
};

/// Which binding issued an id. Every [`Binding`] draws a fresh one, and each id
/// it issues carries it, so an id handed to another binding is recognised as
/// foreign instead of naming whatever sits at the same index there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Issuer(u64);

impl Issuer {
    fn fresh() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Issuer(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// A carrier the binding declared, valid only in the binding that issued it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CarrierId {
    issuer: Issuer,
    pub(crate) index: usize,
}

/// A representation the binding declared, valid only in the binding that
/// issued it. Equal representations get one id, which is what makes it a
/// conversion's identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReprId {
    issuer: Issuer,
    pub(crate) index: usize,
}

/// One of the binding's outputs, valid only in the binding that issued it.
///
/// An output is a declaration *and* the form recorded with it, and the pair is
/// what tells two of them apart: one entity declared twice — `Stamp` as a data
/// class and as a handle — is two outputs, each planned, accounted for and
/// required separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputId {
    issuer: Issuer,
    pub(crate) index: usize,
}

impl std::fmt::Display for CarrierId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c{}", self.index)
    }
}

impl std::fmt::Display for ReprId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.index)
    }
}

/// A Rust type generated code may hold a value in at the boundary.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WireType<C, M> {
    /// The Rust type: `i64`, `*mut ledger_t`, `Stamp`, `JObject<'_>`.
    pub rust: syn::Type,
    /// Which of the adapter's few wire types it is — what a holder's
    /// [`Accepts`] is stated in.
    pub class: C,
    /// For an aggregate, which wire types its members may be; `None` for a
    /// carrier with no members. A `Product` needs a carrier with members.
    pub members: Option<Accepts<C>>,
    /// What the target's writers need to know of it.
    pub meta: M,
}

/// Which wire types a holder may hold: an aggregate's members, a wrapper's
/// parameters or its return.
///
/// The registry checks each placement once the plan has worked out what is
/// placed there, and refuses the value it would put anywhere else.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Accepts<C> {
    pub classes: Vec<C>,
}

impl<C: PartialEq> Accepts<C> {
    pub fn of(classes: impl IntoIterator<Item = C>) -> Self {
        Accepts {
            classes: classes.into_iter().collect(),
        }
    }

    pub fn holds(&self, class: &C) -> bool {
        self.classes.contains(class)
    }
}

/// How the values a conversion rule covers cross.
///
/// It names its carriers and operations and states no type for either: a
/// codec reads its carrier and produces the source type, a `read` reads the
/// product's carrier and produces the part's, whatever carrier that part
/// resolves to. The registry works them out when it plans a value, and feeds
/// them to the writer.
// A binding holds a few dozen of these, each once, so the unboxed codecs cost
// nothing worth an indirection.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Representation<Op> {
    /// The whole value, one operation each way: a scalar, a handle, a
    /// fieldless enum. Each direction has a carrier of its own, because some
    /// values are received in a form they are not returned in — a C enum
    /// arrives as `MaybeUninit` of itself.
    Terminal {
        /// `None`: the value never crosses into Rust.
        into_rust: Option<Codec<Op>>,
        /// `None`: the value never crosses out of Rust.
        out_of_rust: Option<Codec<Op>>,
        /// How the foreign side gives a held value back without converting
        /// it — a handle's typed drop. A type output with a release exports it
        /// as a wrapper of its own.
        release: Option<Operation<Op>>,
    },
    /// The parts of a relation, carried together in one carrier, into Rust.
    Product {
        via: Via,
        /// The aggregate or object holding the parts. Must have members.
        carrier: CarrierId,
        /// One part out of the carrier, applied once per part.
        read: Operation<Op>,
    },
    /// A representation the target does not lower, refused by name.
    Unsupported(Unsupported),
}

/// One direction of a [`Representation::Terminal`]: the carrier, and the
/// operation between it and the source type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Codec<Op> {
    pub carrier: CarrierId,
    pub operation: Operation<Op>,
}

/// Which relation of its type a `Product` reads a value through.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Via {
    /// The struct relation: one part per field.
    Fields,
}

/// One operation, as the binding states it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Operation<Op> {
    pub implementation: Implementation<Op>,
    /// Runtime contexts it needs, by name: `jni.env`. A function form's
    /// context parameters supply them.
    pub context: Vec<String>,
    /// Its failure category and error type, if it can fail.
    pub failure: Option<Failure>,
}

impl<Op> Operation<Op> {
    /// A standard operation, with the failure it has by definition.
    pub fn standard(op: StandardOp) -> Self {
        let failure = match &op {
            StandardOp::FromRaw
            | StandardOp::EnumIn {
                invalid: Some(_), ..
            } => Some(Failure::binding_message()),
            _ => None,
        };
        Operation {
            implementation: Implementation::Standard(op),
            context: Vec::new(),
            failure,
        }
    }

    /// One of the target's own operations, infallible until stated
    /// otherwise.
    pub fn target(op: Op) -> Self {
        Operation {
            implementation: Implementation::Target(op),
            context: Vec::new(),
            failure: None,
        }
    }

    /// The same, needing the runtime context `name`.
    pub fn context(mut self, name: impl Into<String>) -> Self {
        self.context.push(name.into());
        self
    }

    /// The same, failing in `category` with an error of type `error`.
    pub fn fails(mut self, category: FailureCategory, error: syn::Type) -> Self {
        self.failure = Some(Failure {
            category,
            error: Box::new(error),
        });
        self
    }
}

/// What actually performs an operation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Implementation<Op> {
    Standard(StandardOp),
    Target(Op),
}

/// How an operation fails.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Failure {
    pub category: FailureCategory,
    pub error: Box<syn::Type>,
}

impl Failure {
    /// A binding failure carrying a `String` message, which is what every
    /// standard operation that can fail raises.
    pub fn binding_message() -> Self {
        Failure {
            category: FailureCategory::Binding,
            error: Box::new(syn::parse_quote!(String)),
        }
    }
}

/// An operation the registry writes itself, because writing it means naming a
/// source type, which only the registry can do.
///
/// None states a type: where it is used says which. What differs per target —
/// the carrier an address is cast to, what a value is carried as — is the
/// carrier's and the arms'.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StandardOp {
    /// The result *is* the operand: the carrier and the source value are one
    /// Rust value. Renders nothing at all.
    Identity,
    /// Read the part's member of the aggregate the carrier is: `arg0.secs`.
    ReadMember,
    /// Hand an owned source value to the foreign side as an address:
    /// `Box::into_raw(Box::new(v)) as <carrier>`. The foreign side owns the
    /// allocation from here on.
    IntoRaw,
    /// Take back what [`StandardOp::IntoRaw`] handed out:
    /// `*Box::from_raw(v as *mut <source>)`. A null address is a binding
    /// failure carrying a `String`; the handle is consumed either way.
    FromRaw,
    /// Drop what [`StandardOp::IntoRaw`] handed out without converting it. A
    /// null address releases nothing, as `free(NULL)` does. Produces no value.
    Release,
    /// Take a value of a fieldless source enum to what carries it: `match v {
    /// source::Op::Add => <carried>, … }`, one arm per value.
    EnumOut { values: Vec<crate::target::EnumArm> },
    /// The reverse: a carried value back to the source enum.
    ///
    /// With `invalid` set the match ends in a default arm that fails with
    /// that message, formatted with the value — the carrier can hold
    /// something no value of the enum names. With `bits` set the match is on
    /// the carrier's bits read as that integer type: C's `MaybeUninit<op_t>`
    /// holds whatever `int` the caller passed. The carrier must be exactly as
    /// large as `bits`, and initialized, which only the caller can promise, so
    /// the registry makes a wrapper that reads one `unsafe`.
    EnumIn {
        values: Vec<crate::target::EnumArm>,
        invalid: Option<String>,
        bits: Option<Box<syn::Type>>,
    },
}

/// How one source function is exported: everything about its wrapper that is
/// not a planned value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionForm<Op, C> {
    /// The `extern` string: `"C"`, `"system"`.
    pub abi: String,
    /// The symbol the wrapper is exported under.
    pub symbol: String,
    /// Wrapper parameters no source parameter feeds, before those that do:
    /// the JNI environment, the receiver.
    pub context: Vec<ContextParam>,
    /// The names of the wrapper parameters that carry the inputs, in source
    /// order — the source's own names for a function, one name for a
    /// release.
    pub inputs: Vec<syn::Ident>,
    /// One route per failure category the conversions can raise. A category
    /// with no route makes the export unsupported.
    pub routes: Vec<FailureRoute<Op>>,
    /// Attributes beyond `#[no_mangle]`, which the writer owns.
    pub attrs: Vec<syn::Attribute>,
    /// Whether the wrapper is an `unsafe fn`.
    pub unsafety: bool,
    /// The wire types a wrapper parameter may be.
    pub params: Accepts<C>,
    /// The wire types the wrapper may return.
    pub ret: Accepts<C>,
}

/// A wrapper parameter the calling convention adds.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContextParam {
    pub name: syn::Ident,
    pub ty: syn::Type,
    /// The runtime context it supplies, by the name operations ask for it
    /// under; `None` for one the convention requires and nothing uses.
    pub supplies: Option<String>,
    /// Whether the binding needs `mut`.
    pub mutable: bool,
}

/// What a wrapper does when a conversion fails in one category.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FailureRoute<Op> {
    pub category: FailureCategory,
    /// An operation reporting the error before the call ends, handed the
    /// error of the type stated here.
    pub report: Option<Report<Op>>,
    /// What to do when reporting itself fails.
    pub on_report_failure: Terminal,
    /// How this route ends.
    pub terminate: Terminal,
}

/// A failure route's reporting operation, and the error type it takes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Report<Op> {
    pub error: syn::Type,
    pub operation: Operation<Op>,
}

/// What one output is.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OutputForm<Op, C, O> {
    /// One representation of a type, exposed: its carriers' Rust
    /// declarations, and its foreign declaration. The representation is also
    /// the rule at the output's root, so two outputs of one type each plan as
    /// declared.
    Type {
        representation: ReprId,
        /// The wrapper releasing a handed-out value, for a representation
        /// that has a release.
        release: Option<FunctionForm<Op, C>>,
        meta: O,
    },
    /// A source function, exported through a wrapper of this form.
    Function { form: FunctionForm<Op, C>, meta: O },
    /// A declarator the target does not lower, refused by name.
    Unsupported(Unsupported),
}

/// Which values a conversion rule covers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Every value of this type, wherever it turns up.
    Type(TypeKey),
    /// The one value at this path inside this output.
    At(OutputId, ValuePath),
}

/// From an output's root to one value inside it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ValuePath(pub Vec<Step>);

impl ValuePath {
    pub fn root() -> Self {
        ValuePath(Vec::new())
    }

    /// This path, one step further.
    pub fn then(&self, step: Step) -> Self {
        let mut steps = self.0.clone();
        steps.push(step);
        ValuePath(steps)
    }
}

impl std::fmt::Display for ValuePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let steps: Vec<String> = self.0.iter().map(Step::to_string).collect();
        write!(f, "{}", steps.join("."))
    }
}

/// One step of a [`ValuePath`], named as the source names it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Step {
    /// A source function's parameter, by the name the source gives it.
    Param(String),
    /// A source function's result.
    Return,
    /// A field of the struct the current value is read through: its name, or
    /// its position for a tuple field.
    Field(String),
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::Param(name) => write!(f, "param {name}"),
            Step::Return => write!(f, "return"),
            Step::Field(name) => write!(f, "field {name}"),
        }
    }
}

/// A carrier, as a binding for target `T` states one.
pub type CarrierOf<T> = WireType<<T as Target>::WireClass, <T as Target>::CarrierMeta>;
/// A representation, as a binding for target `T` states one.
pub type RepresentationOf<T> = Representation<<T as Target>::Op>;
/// A function form, as a binding for target `T` states one.
pub type FunctionFormOf<T> = FunctionForm<<T as Target>::Op, <T as Target>::WireClass>;
/// An output's form, as a binding for target `T` states one.
pub type OutputFormOf<T> =
    OutputForm<<T as Target>::Op, <T as Target>::WireClass, <T as Target>::OutputMeta>;

/// Everything planning reads from a binding.
///
/// The ids it issues are valid only in it. Every method taking an id panics if
/// the id was issued by another binding: that is a frontend bug, found where
/// the id is handed over rather than as a wrong plan.
pub struct Binding<T: Target> {
    issuer: Issuer,
    carriers: Vec<CarrierOf<T>>,
    representations: Vec<RepresentationOf<T>>,
    rules: Vec<(Scope, ReprId)>,
    outputs: Vec<(Declaration, OutputFormOf<T>)>,
}

impl<T: Target> Default for Binding<T> {
    fn default() -> Self {
        Binding {
            issuer: Issuer::fresh(),
            carriers: Vec::new(),
            representations: Vec::new(),
            rules: Vec::new(),
            outputs: Vec::new(),
        }
    }
}

impl<T: Target> Binding<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// A carrier generated Rust may use. Declaring an equal carrier again
    /// returns the same id.
    pub fn carrier(&mut self, carrier: CarrierOf<T>) -> CarrierId {
        let index = match self.carriers.iter().position(|known| *known == carrier) {
            Some(index) => index,
            None => {
                self.carriers.push(carrier);
                self.carriers.len() - 1
            }
        };
        CarrierId {
            issuer: self.issuer,
            index,
        }
    }

    /// One way a type crosses. Declaring an equal representation again
    /// returns the same id.
    ///
    /// Panics if a carrier it names was issued by another binding.
    pub fn representation(&mut self, representation: RepresentationOf<T>) -> ReprId {
        let carriers: Vec<CarrierId> = match &representation {
            Representation::Terminal {
                into_rust,
                out_of_rust,
                ..
            } => into_rust
                .iter()
                .chain(out_of_rust)
                .map(|codec| codec.carrier)
                .collect(),
            Representation::Product { carrier, .. } => vec![*carrier],
            Representation::Unsupported(_) => Vec::new(),
        };
        for carrier in carriers {
            self.check(carrier.issuer, "carrier");
        }
        let index = match self
            .representations
            .iter()
            .position(|known| *known == representation)
        {
            Some(index) => index,
            None => {
                self.representations.push(representation);
                self.representations.len() - 1
            }
        };
        ReprId {
            issuer: self.issuer,
            index,
        }
    }

    /// The values `scope` covers cross as `representation`.
    ///
    /// Panics if `representation`, or the output `scope` names, was issued by
    /// another binding.
    pub fn rule(&mut self, scope: Scope, representation: ReprId) {
        self.check(representation.issuer, "representation");
        if let Scope::At(output, _) = &scope {
            self.check(output.issuer, "output");
        }
        self.rules.push((scope, representation));
    }

    /// Expose `declaration` in `form`. The id is how a rule addresses a value
    /// inside the output.
    ///
    /// Panics if the representation a type form names was issued by another
    /// binding.
    pub fn output(&mut self, declaration: Declaration, form: OutputFormOf<T>) -> OutputId {
        if let OutputForm::Type { representation, .. } = &form {
            self.check(representation.issuer, "representation");
        }
        self.outputs.push((declaration, form));
        self.output_id(self.outputs.len() - 1)
    }

    /// The carrier `id` names. Panics if another binding issued `id`.
    pub fn carrier_of(&self, id: CarrierId) -> &CarrierOf<T> {
        self.check(id.issuer, "carrier");
        &self.carriers[id.index]
    }

    /// The representation `id` names. Panics if another binding issued `id`.
    pub fn representation_of(&self, id: ReprId) -> &RepresentationOf<T> {
        self.check(id.issuer, "representation");
        &self.representations[id.index]
    }

    /// Every rule, in the order the frontend stated them.
    pub fn rules(&self) -> &[(Scope, ReprId)] {
        &self.rules
    }

    /// Every output, in the order the frontend stated them; an output's
    /// position here is its id's.
    pub fn outputs(&self) -> &[(Declaration, OutputFormOf<T>)] {
        &self.outputs
    }

    /// The declaration and form recorded as one output. Panics if another
    /// binding issued `id`.
    pub fn output_of(&self, id: OutputId) -> &(Declaration, OutputFormOf<T>) {
        self.check(id.issuer, "output");
        &self.outputs[id.index]
    }

    /// The form recorded with one output. Panics if another binding issued
    /// `id`.
    pub fn form_of(&self, id: OutputId) -> &OutputFormOf<T> {
        &self.output_of(id).1
    }

    /// The id of the output at `index` in [`outputs`](Self::outputs).
    pub(crate) fn output_id(&self, index: usize) -> OutputId {
        OutputId {
            issuer: self.issuer,
            index,
        }
    }

    /// An id this binding issued exists in it, since nothing is ever removed:
    /// the issuer is all there is to check.
    fn check(&self, issuer: Issuer, kind: &str) {
        assert!(
            issuer == self.issuer,
            "a {kind} id issued by another binding was handed to this one"
        );
    }
}

impl<T: Target> std::fmt::Display for Binding<T> {
    /// One line per carrier, representation, rule and output, a function
    /// form's details on the lines under its output, with the target's own
    /// vocabulary in its `Debug` form. It prints every field planning reads,
    /// which makes it the diagnostic to diff when two builds of one binding
    /// generate differently. It is not a proof that two bindings are equal:
    /// the target's classes, metadata and operations print only as much as
    /// their `Debug` tells apart.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, carrier) in self.carriers.iter().enumerate() {
            let members = match &carrier.members {
                Some(accepts) => format!("members {:?}", accepts.classes),
                None => "no members".to_string(),
            };
            writeln!(
                f,
                "carrier  c{index}  {}  {:?}  {members}  {:?}",
                tokens(&carrier.rust),
                carrier.class,
                carrier.meta
            )?;
        }
        for (index, representation) in self.representations.iter().enumerate() {
            let described = match representation {
                Representation::Terminal {
                    into_rust,
                    out_of_rust,
                    release,
                } => {
                    let codec = |codec: &Option<Codec<T::Op>>| match codec {
                        Some(codec) => format!("{} {}", codec.carrier, operation(&codec.operation)),
                        None => "-".to_string(),
                    };
                    format!(
                        "terminal  in: {}  out: {}{}",
                        codec(into_rust),
                        codec(out_of_rust),
                        match release {
                            Some(release) => format!("  release: {}", operation(release)),
                            None => String::new(),
                        }
                    )
                }
                Representation::Product { via, carrier, read } => {
                    format!("product  {carrier}  {via:?}  read: {}", operation(read))
                }
                Representation::Unsupported(reason) => {
                    format!("unsupported  {}: {}", reason.capability, reason.explanation)
                }
            };
            writeln!(f, "repr     r{index}  {described}")?;
        }
        for (scope, representation) in &self.rules {
            let scope = match scope {
                Scope::Type(key) => format!("type {}", key.as_str()),
                Scope::At(output, path) => {
                    format!("at {} {path}", self.outputs[output.index].0)
                }
            };
            writeln!(f, "rule     {scope}  {representation}")?;
        }
        for (declaration, form) in &self.outputs {
            match form {
                OutputForm::Type {
                    representation,
                    release,
                    meta,
                } => {
                    writeln!(f, "output   {declaration}  type {representation}  {meta:?}")?;
                    if let Some(release) = release {
                        write_form(f, "release", release)?;
                    }
                }
                OutputForm::Function { form, meta } => {
                    writeln!(f, "output   {declaration}  function  {meta:?}")?;
                    write_form(f, "form", form)?;
                }
                OutputForm::Unsupported(reason) => writeln!(
                    f,
                    "output   {declaration}  unsupported {}: {}",
                    reason.capability, reason.explanation
                )?,
            }
        }
        Ok(())
    }
}

fn tokens(item: &impl quote::ToTokens) -> String {
    item.to_token_stream().to_string()
}

/// An operation on one line: what it is, the contexts it needs and the failure
/// it can raise.
fn operation<Op: std::fmt::Debug>(operation: &Operation<Op>) -> String {
    let mut text = format!("{:?}", operation.implementation);
    if !operation.context.is_empty() {
        text += &format!(" needs {}", operation.context.join(", "));
    }
    if let Some(failure) = &operation.failure {
        text += &format!(
            " fails {} {}",
            failure.category.as_str(),
            tokens(&*failure.error)
        );
    }
    text
}

fn terminal(terminal: &Terminal) -> String {
    match terminal {
        Terminal::Return(value) => format!("return {}", tokens(value)),
        Terminal::Abort => "abort".to_string(),
    }
}

/// A function form, indented under the output it belongs to: its signature on
/// one line, its acceptance on the next, then one line per route.
fn write_form<Op: std::fmt::Debug, C: std::fmt::Debug>(
    f: &mut std::fmt::Formatter<'_>,
    label: &str,
    form: &FunctionForm<Op, C>,
) -> std::fmt::Result {
    let context: Vec<String> = form
        .context
        .iter()
        .map(|param| {
            format!(
                "{}{}: {}{}",
                if param.mutable { "mut " } else { "" },
                param.name,
                tokens(&param.ty),
                match &param.supplies {
                    Some(name) => format!(" supplies {name}"),
                    None => String::new(),
                }
            )
        })
        .collect();
    let inputs: Vec<String> = form.inputs.iter().map(ToString::to_string).collect();
    let attrs: Vec<String> = form.attrs.iter().map(tokens).collect();
    writeln!(
        f,
        "         {label}  {}extern {:?} {}  context [{}]  inputs [{}]{}",
        if form.unsafety { "unsafe " } else { "" },
        form.abi,
        form.symbol,
        context.join(", "),
        inputs.join(", "),
        match attrs.is_empty() {
            true => String::new(),
            false => format!("  attrs [{}]", attrs.join(" ")),
        }
    )?;
    writeln!(
        f,
        "         {label}  params {:?}  ret {:?}",
        form.params.classes, form.ret.classes
    )?;
    for route in &form.routes {
        let report = match &route.report {
            Some(report) => format!(
                "report {} by {}, if that fails {}, then ",
                tokens(&report.error),
                operation(&report.operation),
                terminal(&route.on_report_failure)
            ),
            None => String::new(),
        };
        writeln!(
            f,
            "         {label}  route {}: {report}{}",
            route.category.as_str(),
            terminal(&route.terminate)
        )?;
    }
    Ok(())
}
