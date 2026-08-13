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
    pub fn source(self) -> &'static str {
        match self {
            Need::Record => "pub struct Rec { pub id: u64, pub tag: u32 }",
            Need::Handle => "pub struct Handle { pub id: u64 }",
            Need::Sum => "pub enum Sum { Num(u64), Nothing }",
            Need::UnitEnum => "pub enum Mode { On = 0, Off = 1 }",
            // Both targets need a way to render an error, so the accessor is
            // part of the declaration rather than something a cell goes
            // without.
            Need::Error => {
                "pub struct ZError { pub code: u64 }\n\
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
