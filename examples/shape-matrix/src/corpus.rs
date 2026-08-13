//! What gets enumerated: the shapes, the positions they are tried in, and the
//! declarations a shape needs in order to mean anything.

/// A supporting type a shape refers to.
///
/// These are *declarations*, which is a separate axis from the type axis: the
/// model records a field's type as "a named type called `Rec`" and stops, so a
/// cell about a struct field has to emit a struct, not just write a type.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Need {
    /// A plain product of scalars, declared to each target as a value type
    /// (Kotlin data class / C struct).
    Record,
    /// A product declared as an opaque handle (Kotlin class over a pointer /
    /// C opaque pointer).
    Handle,
    /// An enum whose alternatives carry payloads.
    Sum,
    /// An enum whose alternatives are all fieldless.
    UnitEnum,
    /// A type used in the error position of a `Result`.
    Error,
}

impl Need {
    /// The Rust the fixture declares for it.
    ///
    /// Everything derives `Clone`, as the types in a real source crate do. That
    /// is not incidental: several generator paths clone — a borrowed handle
    /// crossing out becomes an owned one — so a fixture whose types were not
    /// `Clone` would spend its cells measuring that constraint instead of
    /// whether the shape crosses.
    pub fn source(self) -> &'static str {
        match self {
            Need::Record => "#[derive(Clone)] pub struct Rec { pub id: u64, pub tag: u32 }",
            Need::Handle => "#[derive(Clone)] pub struct Handle { pub id: u64 }",
            // `Display` too: `result_sum_err` puts this in an error position,
            // where both targets render the error as text.
            Need::Sum => {
                "#[derive(Clone)] pub enum Sum { Num(u64), Nothing }\n\
                          impl std::fmt::Display for Sum {\n\
                          fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n\
                          write!(f, \"sum\") } }"
            }
            Need::UnitEnum => "#[derive(Clone, Copy)] pub enum Mode { On = 0, Off = 1 }",
            // Both targets need a way to render an error, so the accessor is
            // part of the declaration rather than something a cell goes
            // without.
            // An error type is `Clone` and `Display` because that is what an
            // error type is; a fixture without them spends its fallible cells
            // measuring that requirement instead of whether the shape crosses.
            Need::Error => {
                "#[derive(Clone)] pub struct ZError { pub code: u64 }\n\
                 impl std::fmt::Display for ZError {\n\
                 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n\
                 write!(f, \"error {}\", self.code) } }\n\
                 pub fn zerror_message(e: &ZError) -> String { unimplemented!() }"
            }
        }
    }

    /// The type name it declares.
    pub fn type_name(self) -> &'static str {
        match self {
            Need::Record => "Rec",
            Need::Handle => "Handle",
            Need::Sum => "Sum",
            Need::UnitEnum => "Mode",
            Need::Error => "ZError",
        }
    }
}

/// One shape under test: a Rust type, plus whatever it needs declared.
pub struct Shape {
    /// Stable id, used in the report and in coverage receipts.
    pub id: &'static str,
    /// The type as the fixture writes it.
    pub spelling: &'static str,
    pub needs: &'static [Need],
}

/// A **call**: several values crossing together.
///
/// A separate axis because aliasing is a property of a call and not of a value.
/// Two parameters can name the same underlying resource, and what the binding
/// must do about it — reject the call before any conversion runs, or leave it
/// alone — cannot be stated about either parameter on its own. Both generators
/// emit an alias preflight for exactly this, under a rule about the *set* of
/// parameters: at least one consumed handle, and any other handle.
///
/// What is measured here is whether such a call can be expressed at all. Whether
/// the emitted guard *fires* — rejecting the aliased call, sparing the
/// unaliased one, and running before ownership moves — is a claim about running
/// code, and waits for the runtime stage rather than being asserted against
/// emitted text.
pub struct Call {
    /// Stable id, used in the report and as the cell's receipt key.
    pub id: &'static str,
    /// The parameter types, in order.
    pub params: &'static [&'static str],
    pub needs: &'static [Need],
}

/// The call shapes.
///
/// Chosen around the preflight rule rather than at random: pairs that must be
/// guarded, pairs that must **not** be, and pairs in different resource domains
/// where the question does not arise.
pub const CALLS: &[Call] = &[
    // Two consumed handles — `z_combine(x, x)` hands one allocation to two
    // consuming converters. The case both generators name in their own docs.
    Call {
        id: "consume_consume",
        params: &["Handle", "Handle"],
        needs: &[Need::Handle],
    },
    // A consume beside a borrow: the borrow dangles the moment the consume
    // takes ownership. Narrower rules ("two consumed parameters") miss it.
    Call {
        id: "consume_borrow",
        params: &["Handle", "&Handle"],
        needs: &[Need::Handle],
    },
    // Two shared borrows of one resource are legal Rust and legal C, so this
    // pair must stay expressible — a guard here would remove working surface.
    Call {
        id: "borrow_borrow",
        params: &["&Handle", "&Handle"],
        needs: &[Need::Handle],
    },
    // A consume beside an optional handle: the same domain through a different
    // spelling, which is why the comparison is on the resource and not on the
    // declared type.
    Call {
        id: "consume_optional",
        params: &["Handle", "Option<Handle>"],
        needs: &[Need::Handle],
    },
    // A handle beside a value struct — different domains, nothing to alias.
    Call {
        id: "handle_and_record",
        params: &["Handle", "Rec"],
        needs: &[Need::Handle, Need::Record],
    },
    // A handle beside a sum, whose payload is not a handle: the pair a
    // domain-blind rule would flag.
    Call {
        id: "handle_and_sum",
        params: &["Handle", "Sum"],
        needs: &[Need::Handle, Need::Sum],
    },
    // No handles at all — the control: whatever the preflight does, it has
    // nothing to say here.
    Call {
        id: "two_records",
        params: &["Rec", "Rec"],
        needs: &[Need::Record],
    },
    // Three, with the consume last: position within the call must not matter.
    Call {
        id: "borrow_borrow_consume",
        params: &["&Handle", "&Handle", "Handle"],
        needs: &[Need::Handle],
    },
];

/// Where the shape is placed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Position {
    /// A function parameter.
    Param,
    /// A function's return value.
    Return,
    /// A field of a struct that crosses whole.
    Field,
    /// The payload of one alternative of an enum that crosses whole.
    Payload,
}

impl Position {
    pub const ALL: &'static [Position] = &[
        Position::Param,
        Position::Return,
        Position::Field,
        Position::Payload,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Position::Param => "parameter",
            Position::Return => "return",
            Position::Field => "struct field",
            Position::Payload => "enum payload",
        }
    }

    /// The part of a cell id this position contributes — file-safe, since a
    /// cell id names the file rustc reports diagnostics against.
    pub fn slug(self) -> &'static str {
        match self {
            Position::Param => "param",
            Position::Return => "ret",
            Position::Field => "field",
            Position::Payload => "payload",
        }
    }
}

/// The shapes.
///
/// Bounded on purpose — one canonical representative per distinct decision,
/// composite depth at most 3 counting named nodes, wrapper nesting at most 2.
/// The list is checked against the model in `every_type_form_is_covered`: every
/// form `TypeKind` accepts must appear somewhere in here.
pub const SHAPES: &[Shape] = &[
    // ── leaves ────────────────────────────────────────────────────────────
    Shape {
        id: "scalar",
        spelling: "u64",
        needs: &[],
    },
    Shape {
        id: "bool",
        spelling: "bool",
        needs: &[],
    },
    Shape {
        id: "unit",
        spelling: "()",
        needs: &[],
    },
    Shape {
        id: "string",
        spelling: "String",
        needs: &[],
    },
    Shape {
        id: "str_ref",
        spelling: "&str",
        needs: &[],
    },
    // ── declared types, by how each target is told to treat them ──────────
    Shape {
        id: "record",
        spelling: "Rec",
        needs: &[Need::Record],
    },
    Shape {
        id: "handle",
        spelling: "Handle",
        needs: &[Need::Handle],
    },
    Shape {
        id: "sum",
        spelling: "Sum",
        needs: &[Need::Sum],
    },
    Shape {
        id: "unit_enum",
        spelling: "Mode",
        needs: &[Need::UnitEnum],
    },
    // ── borrows ───────────────────────────────────────────────────────────
    Shape {
        id: "shared_ref",
        spelling: "&Rec",
        needs: &[Need::Record],
    },
    Shape {
        id: "exclusive_ref",
        spelling: "&mut Rec",
        needs: &[Need::Record],
    },
    Shape {
        id: "handle_ref",
        spelling: "&Handle",
        needs: &[Need::Handle],
    },
    Shape {
        id: "out_param",
        spelling: "&mut MaybeUninit<u64>",
        needs: &[],
    },
    // ── wrappers over a scalar ────────────────────────────────────────────
    Shape {
        id: "opt_scalar",
        spelling: "Option<u64>",
        needs: &[],
    },
    Shape {
        id: "vec_scalar",
        spelling: "Vec<u64>",
        needs: &[],
    },
    Shape {
        id: "slice_scalar",
        spelling: "&[u64]",
        needs: &[],
    },
    Shape {
        id: "slice_mut_scalar",
        spelling: "&mut [u64]",
        needs: &[],
    },
    Shape {
        id: "array_scalar",
        spelling: "[u8; 4]",
        needs: &[],
    },
    Shape {
        id: "boxed_scalar",
        spelling: "Box<u64>",
        needs: &[],
    },
    Shape {
        id: "cow_str",
        spelling: "Cow<'static, str>",
        needs: &[],
    },
    // ── wrappers over a declared type ─────────────────────────────────────
    Shape {
        id: "opt_record",
        spelling: "Option<Rec>",
        needs: &[Need::Record],
    },
    Shape {
        id: "opt_handle",
        spelling: "Option<Handle>",
        needs: &[Need::Handle],
    },
    Shape {
        id: "opt_ref",
        spelling: "Option<&Handle>",
        needs: &[Need::Handle],
    },
    Shape {
        id: "vec_record",
        spelling: "Vec<Rec>",
        needs: &[Need::Record],
    },
    Shape {
        id: "vec_handle",
        spelling: "Vec<Handle>",
        needs: &[Need::Handle],
    },
    Shape {
        id: "vec_ref",
        spelling: "Vec<&Handle>",
        needs: &[Need::Handle],
    },
    Shape {
        id: "vec_sum",
        spelling: "Vec<Sum>",
        needs: &[Need::Sum],
    },
    Shape {
        id: "opt_sum",
        spelling: "Option<Sum>",
        needs: &[Need::Sum],
    },
    Shape {
        id: "array_record",
        spelling: "[Rec; 2]",
        needs: &[Need::Record],
    },
    // ── nested wrappers ───────────────────────────────────────────────────
    Shape {
        id: "opt_vec",
        spelling: "Option<Vec<u64>>",
        needs: &[],
    },
    Shape {
        id: "vec_opt",
        spelling: "Vec<Option<u64>>",
        needs: &[],
    },
    // ── fallible and callback ─────────────────────────────────────────────
    Shape {
        id: "result_scalar",
        spelling: "Result<u64, ZError>",
        needs: &[Need::Error],
    },
    Shape {
        id: "result_handle",
        spelling: "Result<Handle, ZError>",
        needs: &[Need::Handle, Need::Error],
    },
    Shape {
        id: "result_sum_err",
        spelling: "Result<u64, Sum>",
        needs: &[Need::Sum],
    },
    Shape {
        id: "callback",
        spelling: "impl Fn(u64) + Send + Sync + 'static",
        needs: &[],
    },
    Shape {
        id: "callback_handle",
        spelling: "impl Fn(Handle) + Send + Sync + 'static",
        needs: &[Need::Handle],
    },
];
