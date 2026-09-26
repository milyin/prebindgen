//! The instructions the registry composes and the common Rust writer renders.
//!
//! The chapters leave this vocabulary named but undefined (`ConversionBodyId`,
//! `FunctionBodyId`); this is the concrete form the first increment settles on.
//! Four instructions cover what it carries: apply an operation, construct a
//! source struct, call the source function, and build the closure a callback
//! enters Rust as. Every value
//! is a [`ValueId`] — an identity, not a name. Names are allocated once, by the
//! writer, from definition order, which is why two operations rendered into one
//! wrapper cannot collide.

use std::collections::HashMap;

use prebindgen_flat::flat::TypeRef;

use crate::{binding::ReprId, plan::PrimitiveId};

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
    /// Build the closure a callback enters Rust as: `move |params| { instrs }`,
    /// taking `captured` with it. A failure inside it takes a route of the
    /// callback representation's own, since the closure has no caller to hand
    /// one to.
    Closure {
        representation: ReprId,
        captured: ValueId,
        /// The arguments Rust calls it with, and their source types.
        params: Vec<(ValueId, TypeRef)>,
        instrs: Vec<Stmt>,
        result: ValueId,
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

    /// Take the instructions added since `from` out of this body, keeping
    /// their value identities allocated: a closure's own instructions are
    /// built here, then moved into the closure.
    pub fn split_off(&mut self, from: usize) -> Vec<Stmt> {
        self.instrs.split_off(from)
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
        let mut map = HashMap::new();
        map.insert(self.carrier, carrier);
        for step in remap(&self.instrs, &mut map, out) {
            out.instrs.push(step);
        }
        value(&map, self.result)
    }
}

fn value(map: &HashMap<ValueId, ValueId>, id: ValueId) -> ValueId {
    *map.get(&id)
        .expect("a body uses no value before defining it")
}

/// `stmts` with every value identity renamed into `out`'s: operands through
/// `map`, results allocated fresh and added to it. A closure's instructions
/// are renamed the same way, into the same body, since its names and the
/// names around it share one scope.
fn remap(stmts: &[Stmt], map: &mut HashMap<ValueId, ValueId>, out: &mut BodyBuilder) -> Vec<Stmt> {
    let fresh = |map: &mut HashMap<ValueId, ValueId>, out: &mut BodyBuilder, id: ValueId| {
        let fresh = out.fresh();
        map.insert(id, fresh);
        fresh
    };
    stmts
        .iter()
        .map(|step| {
            let instr = match &step.instr {
                Instr::Apply {
                    primitive,
                    operands,
                    result,
                } => {
                    let operands = operands
                        .iter()
                        .map(|operand| match operand {
                            Operand::Value(id) => Operand::Value(value(map, *id)),
                            Operand::Context(name) => Operand::Context(name.clone()),
                        })
                        .collect();
                    Instr::Apply {
                        primitive: *primitive,
                        operands,
                        result: result.map(|id| fresh(map, out, id)),
                    }
                }
                Instr::Construct {
                    name,
                    parts,
                    result,
                } => Instr::Construct {
                    name: name.clone(),
                    parts: parts.iter().map(|id| value(map, *id)).collect(),
                    result: fresh(map, out, *result),
                },
                Instr::Call {
                    function,
                    args,
                    result,
                } => Instr::Call {
                    function: function.clone(),
                    args: args.iter().map(|id| value(map, *id)).collect(),
                    result: result.map(|id| fresh(map, out, id)),
                },
                Instr::Closure {
                    representation,
                    captured,
                    params,
                    instrs,
                    result,
                } => {
                    let captured = value(map, *captured);
                    let params = params
                        .iter()
                        .map(|(id, ty)| (fresh(map, out, *id), ty.clone()))
                        .collect();
                    let instrs = remap(instrs, map, out);
                    Instr::Closure {
                        representation: *representation,
                        captured,
                        params,
                        instrs,
                        result: fresh(map, out, *result),
                    }
                }
            };
            Stmt {
                instr,
                conditions: step.conditions.clone(),
            }
        })
        .collect()
}
