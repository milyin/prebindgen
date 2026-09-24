//! The instructions the registry composes and the common Rust writer renders.
//!
//! The chapters leave this vocabulary named but undefined (`ConversionBodyId`,
//! `FunctionBodyId`); this is the concrete form the first increment settles on.
//! Three instructions cover the scalar and owned-struct paths: apply a target
//! operation, construct a source struct, call the source function. Every value
//! is a [`ValueId`] — an identity, not a name. Names are allocated once, by the
//! writer, from definition order, which is why two operations rendered into one
//! wrapper cannot collide.

use crate::plan::PrimitiveId;

/// One runtime value inside one body. Not a variable name: the writer chooses
/// those.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValueId(pub(crate) u32);

/// What supplies an operand.
#[derive(Clone, Debug)]
pub enum Operand {
    /// A value this body already has.
    Value(ValueId),
    /// A runtime context the boundary supplies under this name.
    Context(String),
}

/// One step of a conversion or a wrapper.
#[derive(Clone, Debug)]
pub enum Instr {
    /// Apply a registered operation, binding its result when it has one.
    Apply {
        primitive: PrimitiveId,
        operands: Vec<Operand>,
        result: Option<ValueId>,
    },
    /// Build a source struct from already-converted parts, in field order.
    Construct {
        /// The struct's declared name; the writer takes its field shape from
        /// the model, so a tuple struct is constructed as one.
        name: String,
        parts: Vec<ValueId>,
        result: ValueId,
    },
    /// Call the source function once, binding its result unless it returns
    /// nothing — a `let` over a unit call names a value no one can use.
    Call {
        function: String,
        args: Vec<ValueId>,
        result: Option<ValueId>,
    },
}

/// One instruction, and the conditions under which it exists at all.
///
/// `conditions` is empty for all but a field whose `#[cfg]` the capture reader
/// could not answer: every instruction serving such a field carries it, so the
/// statements the writer renders appear exactly where the field does. Nothing
/// reads them — they are the tokens the source wrote, conjoined by Rust when
/// there is more than one.
#[derive(Clone, Debug)]
pub struct Stmt {
    pub instr: Instr,
    pub conditions: Vec<proc_macro2::TokenStream>,
}

impl Stmt {
    fn new(instr: Instr) -> Self {
        Stmt {
            instr,
            conditions: Vec::new(),
        }
    }
}

/// A body under construction, with its own value-identity allocator.
#[derive(Debug, Default)]
pub struct BodyBuilder {
    next: u32,
    instrs: Vec<Stmt>,
}

impl BodyBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// A value identity nothing else in this body uses.
    pub fn fresh(&mut self) -> ValueId {
        let id = ValueId(self.next);
        self.next += 1;
        id
    }

    pub fn push(&mut self, instr: Instr) {
        self.instrs.push(Stmt::new(instr));
    }

    pub fn instrs(&self) -> &[Stmt] {
        &self.instrs
    }

    pub fn into_instrs(self) -> Vec<Stmt> {
        self.instrs
    }

    /// How many instructions this body already has, so a caller can name the
    /// range it is about to add.
    pub fn len(&self) -> usize {
        self.instrs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instrs.is_empty()
    }

    /// Put `conditions` on every instruction added since `from`.
    ///
    /// The caller is whoever knows what a stretch of instructions is *for* — a
    /// part's conversion, say. Conditions accumulate rather than replace, so a
    /// conditional field of a conditional field carries both.
    pub fn condition(&mut self, from: usize, conditions: &[proc_macro2::TokenStream]) {
        if conditions.is_empty() {
            return;
        }
        for step in &mut self.instrs[from..] {
            step.conditions.extend(conditions.iter().cloned());
        }
    }
}

/// A planned conversion's instructions, as a template.
///
/// [`NodeBody::carrier`] is the value handed in when the conversion is used;
/// [`NodeBody::result`] is what it produces. An identity conversion is the
/// degenerate case — no instructions, and the result *is* the carrier — which
/// is what makes a scalar child render nothing.
#[derive(Clone, Debug)]
pub struct NodeBody {
    pub carrier: ValueId,
    pub instrs: Vec<Stmt>,
    pub result: ValueId,
}

impl NodeBody {
    /// Inline this conversion into `out`, with `carrier` supplying its input.
    ///
    /// Value identities are local to a template, so they are remapped as they
    /// are met: an operand must already be mapped, and a result is allocated
    /// fresh in the destination body.
    pub fn inline(&self, carrier: ValueId, out: &mut BodyBuilder) -> ValueId {
        let mut map = std::collections::HashMap::new();
        map.insert(self.carrier, carrier);
        let value = |map: &std::collections::HashMap<ValueId, ValueId>, id: ValueId| {
            *map.get(&id)
                .expect("a body uses no value before defining it")
        };
        for step in &self.instrs {
            let instr = match &step.instr {
                Instr::Apply {
                    primitive,
                    operands,
                    result,
                } => {
                    let operands = operands
                        .iter()
                        .map(|operand| match operand {
                            Operand::Value(id) => Operand::Value(value(&map, *id)),
                            Operand::Context(name) => Operand::Context(name.clone()),
                        })
                        .collect();
                    let result = result.map(|id| {
                        let fresh = out.fresh();
                        map.insert(id, fresh);
                        fresh
                    });
                    Instr::Apply {
                        primitive: *primitive,
                        operands,
                        result,
                    }
                }
                Instr::Construct {
                    name,
                    parts,
                    result,
                } => {
                    let parts = parts.iter().map(|id| value(&map, *id)).collect();
                    let fresh = out.fresh();
                    map.insert(*result, fresh);
                    Instr::Construct {
                        name: name.clone(),
                        parts,
                        result: fresh,
                    }
                }
                Instr::Call {
                    function,
                    args,
                    result,
                } => {
                    let args = args.iter().map(|id| value(&map, *id)).collect();
                    let result = result.map(|id| {
                        let fresh = out.fresh();
                        map.insert(id, fresh);
                        fresh
                    });
                    Instr::Call {
                        function: function.clone(),
                        args,
                        result,
                    }
                }
            };
            let from = out.len();
            out.push(instr);
            out.condition(from, &step.conditions);
        }
        value(&map, self.result)
    }
}
