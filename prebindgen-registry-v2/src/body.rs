//! The instructions the registry composes and the common Rust writer renders.
//!
//! The chapters leave this vocabulary named but undefined (`ConversionBodyId`,
//! `FunctionBodyId`); this is the concrete form the first increment settles on.
//! Three instructions cover the scalar and owned-record paths: apply a target
//! operation, construct a source record, call the source function. Every value
//! is a [`ValueId`] — an identity, not a name. Names are allocated once, by the
//! writer, from definition order, which is why two operations rendered into one
//! wrapper cannot collide.

use crate::target::PrimitiveId;

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
    /// Build a source record from already-converted parts, in field order.
    Construct {
        /// The record's declared name; the writer takes its field shape from
        /// the model, so a tuple record is constructed as one.
        record: String,
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

/// A body under construction, with its own value-identity allocator.
#[derive(Debug, Default)]
pub struct BodyBuilder {
    next: u32,
    instrs: Vec<Instr>,
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
        self.instrs.push(instr);
    }

    pub fn instrs(&self) -> &[Instr] {
        &self.instrs
    }

    pub fn into_instrs(self) -> Vec<Instr> {
        self.instrs
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
    pub instrs: Vec<Instr>,
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
        for instr in &self.instrs {
            let instr = match instr {
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
                    record,
                    parts,
                    result,
                } => {
                    let parts = parts.iter().map(|id| value(&map, *id)).collect();
                    let fresh = out.fresh();
                    map.insert(*result, fresh);
                    Instr::Construct {
                        record: record.clone(),
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
            out.push(instr);
        }
        value(&map, self.result)
    }
}
