//! Rendering [`KtExpr`] to Kotlin source: precedence-correct parenthesization,
//! scope-tracked name allocation, and import collection from the tree.
//!
//! The three things the string-built expressions could not do:
//!
//! - **Precedence** was previously the producer's job, so a missing paren was
//!   a Gradle error at best and a silently-wrong expression at worst. Here it
//!   is a property of the tree: a child is parenthesized exactly when its
//!   precedence is lower than its position allows.
//! - **Names** were hand-numbered (`e0`, `e1`, …) to dodge `it` shadowing —
//!   scope management by naming convention. Here the renderer allocates, with
//!   a real scope stack, and a machine-allocated name can collide with neither
//!   a [`Spelling::Fixed`] name nor any free [`KtName`] the tree references.
//! - **Imports** were registered by hand alongside the raw text, so the two
//!   could drift. Here a rendered unit reports what it needs, because the
//!   names are in the tree.

use std::collections::HashSet;

use super::{
    super::types::ImportSet, free_names, BindingId, ExprArena, KtExpr, KtLiteral, KtName,
    KtPattern, KtStmt, Spelling,
};

/// Binding strength of an expression, high binds tighter. Only the operators
/// this AST models appear; Kotlin's full table is larger and irrelevant until
/// a node needs it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    /// `a ?: b` — the loosest thing we build.
    Elvis = 1,
    /// `a as T`, `a as? T`.
    As = 2,
    /// `a.b`, `a?.b`, `f(x)` — postfix, binds tightest.
    Postfix = 3,
    /// A leaf: never needs parentheses.
    Atom = 4,
}

fn prec(e: &KtExpr) -> Prec {
    match e {
        KtExpr::Elvis(..) => Prec::Elvis,
        KtExpr::As { .. } => Prec::As,
        KtExpr::Field { .. } | KtExpr::Call { .. } => Prec::Postfix,
        // A `when` and a lambda are brace-delimited, so they are self-bracketing
        // in every position we emit them.
        KtExpr::Lambda { .. } | KtExpr::When { .. } => Prec::Atom,
        KtExpr::Local(_) | KtExpr::Name(_) | KtExpr::Literal(_) => Prec::Atom,
        KtExpr::Hole | KtExpr::Raw(_) => Prec::Atom,
    }
}

/// The renderer's scope stack: which printed name each live binder has, and
/// which names are taken.
struct Scope<'a> {
    arena: &'a ExprArena,
    /// Printed name per binder, innermost last.
    frames: Vec<Vec<(BindingId, String)>>,
    /// Every name currently unavailable: the free names reserved up front plus
    /// every live binder's printed name.
    taken: HashSet<String>,
}

impl<'a> Scope<'a> {
    /// Reserve every free [`KtName`] the tree references **before** allocating
    /// any binder name.
    ///
    /// Without this a machine-allocated binder could shadow a name the tree
    /// refers to — the same capture the `BindingId` scheme prevents for
    /// binders, arriving through the one position `BindingId`s do not cover.
    fn new(arena: &'a ExprArena, root: &KtExpr) -> Self {
        let mut taken = HashSet::new();
        for n in free_names(root) {
            // A qualified name renders as its last segment once imported, so
            // that is the spelling a binder could collide with.
            taken.insert(n.simple().to_string());
        }
        Self {
            arena,
            frames: Vec::new(),
            taken,
        }
    }

    fn push(&mut self, binders: &[BindingId]) {
        let mut frame = Vec::new();
        for id in binders {
            let name = match &self.arena.binder(*id).spelling {
                // Public API: preserved byte-identically. A `Fixed` name is
                // *claimed*, never renamed, so a later `Fresh` binder avoids it
                // rather than the other way round.
                Spelling::Fixed(n) => n.as_str().to_string(),
                Spelling::Fresh(hint) => self.allocate(hint.as_str()),
            };
            self.taken.insert(name.clone());
            frame.push((*id, name));
        }
        self.frames.push(frame);
    }

    fn pop(&mut self) {
        if let Some(frame) = self.frames.pop() {
            for (_, name) in frame {
                self.taken.remove(&name);
            }
        }
    }

    /// `hint`, else `hint2`, `hint3`, … — the first spelling not already taken.
    fn allocate(&self, hint: &str) -> String {
        if !self.taken.contains(hint) {
            return hint.to_string();
        }
        (2..)
            .map(|i| format!("{hint}{i}"))
            .find(|c| !self.taken.contains(c))
            .expect("an unbounded sequence always yields a free name")
    }

    fn lookup(&self, id: BindingId) -> &str {
        for frame in self.frames.iter().rev() {
            for (bid, name) in frame {
                if *bid == id {
                    return name;
                }
            }
        }
        panic!(
            "KtExpr::Local({}) is not in scope — the tree references a binder nothing introduces",
            id.index()
        )
    }
}

/// Render `expr` to Kotlin source, registering the FQNs it references in
/// `imports`.
pub fn render_expr(arena: &ExprArena, expr: &KtExpr, imports: &mut ImportSet) -> String {
    let mut scope = Scope::new(arena, expr);
    write_expr(expr, &mut scope, imports, Prec::Elvis)
}

/// Render a statement list as a block body's lines.
pub fn render_stmts(arena: &ExprArena, stmts: &[KtStmt], imports: &mut ImportSet) -> Vec<String> {
    // Reserve against every statement, not just the first: a name free in one
    // and taken in another must be avoided in both.
    let mut scope = Scope::new(
        arena,
        &KtExpr::Lambda {
            params: Vec::new(),
            body: stmts.to_vec(),
        },
    );
    scope.push(&[]);
    let out = write_stmts(stmts, &mut scope, imports);
    scope.pop();
    out
}

/// Write `e`, parenthesizing when its precedence is looser than `needed`.
fn write_expr(e: &KtExpr, scope: &mut Scope, imports: &mut ImportSet, needed: Prec) -> String {
    let rendered = write_bare(e, scope, imports);
    if prec(e) < needed {
        format!("({rendered})")
    } else {
        rendered
    }
}

fn write_bare(e: &KtExpr, scope: &mut Scope, imports: &mut ImportSet) -> String {
    match e {
        KtExpr::Local(id) => scope.lookup(*id).to_string(),
        KtExpr::Name(n) => render_name(n, imports),
        KtExpr::Literal(l) => render_literal(l),
        KtExpr::Field { recv, name, safe } => {
            let r = write_expr(recv, scope, imports, Prec::Postfix);
            format!("{r}{}{}", if *safe { "?." } else { "." }, name.simple())
        }
        KtExpr::Call {
            recv,
            name,
            args,
            safe,
            trailing_lambda,
        } => {
            let rendered_args: Vec<String> = args
                .iter()
                .map(|a| write_expr(a, scope, imports, Prec::Elvis))
                .collect();
            let head = match recv {
                Some(r) => {
                    let r = write_expr(r, scope, imports, Prec::Postfix);
                    format!("{r}{}{}", if *safe { "?." } else { "." }, name.simple())
                }
                // An unqualified call names a free function, which may be
                // imported like a class.
                None => render_name(name, imports),
            };
            let mut out = format!("{head}({})", rendered_args.join(", "));
            if let Some(l) = trailing_lambda {
                out.push(' ');
                out.push_str(&write_bare(l, scope, imports));
            }
            out
        }
        KtExpr::As { expr, ty, safe } => {
            // `as` is left-associative and binds tighter than elvis, so the
            // operand needs `As` strength.
            let inner = write_expr(expr, scope, imports, Prec::As);
            format!(
                "{inner} {} {}",
                if *safe { "as?" } else { "as" },
                ty.render(imports)
            )
        }
        KtExpr::Elvis(a, b) => {
            // Right-associative: the left operand must bind tighter, the right
            // may be another elvis.
            let lhs = write_expr(a, scope, imports, Prec::As);
            let rhs = write_expr(b, scope, imports, Prec::Elvis);
            format!("{lhs} ?: {rhs}")
        }
        KtExpr::Lambda { params, body } => {
            scope.push(params);
            let names: Vec<String> = params
                .iter()
                .map(|p| scope.lookup(*p).to_string())
                .collect();
            let lines = write_stmts(body, scope, imports);
            scope.pop();
            let head = if names.is_empty() {
                String::new()
            } else {
                format!("{} -> ", names.join(", "))
            };
            format!("{{ {head}{} }}", lines.join("; "))
        }
        KtExpr::When { subject, arms } => {
            let subj = write_expr(subject, scope, imports, Prec::Elvis);
            let rendered: Vec<String> = arms
                .iter()
                .map(|(p, body)| {
                    let pat = match p {
                        KtPattern::Is(ty) => format!("is {}", ty.render(imports)),
                        KtPattern::Value(v) => write_expr(v, scope, imports, Prec::Elvis),
                        KtPattern::Else => "else".to_string(),
                    };
                    format!("{pat} -> {}", write_expr(body, scope, imports, Prec::Elvis))
                })
                .collect();
            format!("when ({subj}) {{ {} }}", rendered.join("; "))
        }
        KtExpr::Hole => panic!(
            "KtExpr::Hole reached the renderer — a template was emitted without being filled \
             (see `fill_hole`)"
        ),
        KtExpr::Raw(s) => s.clone(),
    }
}

fn write_stmts(stmts: &[KtStmt], scope: &mut Scope, imports: &mut ImportSet) -> Vec<String> {
    let mut out = Vec::new();
    // Locals introduced here are visible to every later statement, so they go
    // into one frame that grows as the block is walked.
    let mut introduced: Vec<BindingId> = Vec::new();
    for s in stmts {
        match s {
            KtStmt::Let {
                binder,
                mutable,
                value,
            } => {
                // The value is rendered BEFORE the binder enters scope: `val x
                // = x` must refer to the outer `x`.
                let v = write_expr(value, scope, imports, Prec::Elvis);
                scope.push(&[*binder]);
                introduced.push(*binder);
                let kw = if *mutable { "var" } else { "val" };
                out.push(format!("{kw} {} = {v}", scope.lookup(*binder)));
            }
            KtStmt::Expr(e) => out.push(write_expr(e, scope, imports, Prec::Elvis)),
            KtStmt::Return(Some(e)) => out.push(format!(
                "return {}",
                write_expr(e, scope, imports, Prec::Elvis)
            )),
            KtStmt::Return(None) => out.push("return".to_string()),
        }
    }
    for _ in introduced {
        scope.pop();
    }
    out
}

/// A qualified name registers an import and renders short; a bare one renders
/// verbatim. Same rule [`super::super::types::KtType`] follows, so a class
/// referenced from a type and from an expression agree on their import.
fn render_name(n: &KtName, imports: &mut ImportSet) -> String {
    if n.is_qualified() {
        imports.short(n.as_str())
    } else {
        n.as_str().to_string()
    }
}

fn render_literal(l: &KtLiteral) -> String {
    match l {
        KtLiteral::Null => "null".to_string(),
        KtLiteral::Bool(b) => b.to_string(),
        KtLiteral::Int(v) => v.to_string(),
        KtLiteral::Long(v) => format!("{v}L"),
        KtLiteral::Double(v) => {
            // Kotlin needs a decimal point to infer `Double`.
            if v.fract() == 0.0 && v.is_finite() {
                format!("{v:.1}")
            } else {
                v.to_string()
            }
        }
        // Escaped HERE, by the renderer — never pre-escaped by a producer, which
        // is the whole reason `KtLiteral::Str` holds the raw value.
        KtLiteral::Str(s) => {
            let mut out = String::with_capacity(s.len() + 2);
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    '$' => out.push_str("\\$"),
                    other => out.push(other),
                }
            }
            out.push('"');
            out
        }
    }
}
