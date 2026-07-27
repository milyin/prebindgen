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

use std::collections::HashMap;

use super::{
    super::types::ImportSet, free_names, is_hard_keyword, BindingId, ExprArena, KtExpr, KtLambda,
    KtLiteral, KtName, KtPattern, KtStmt, Spelling,
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
        KtExpr::Local(_) | KtExpr::Name(_) | KtExpr::This | KtExpr::Literal(_) => Prec::Atom,
        KtExpr::Hole | KtExpr::Raw(_) => Prec::Atom,
    }
}

/// The renderer's scope stack: which printed name each live binder has, and
/// which names are taken.
struct Scope<'a> {
    arena: &'a ExprArena,
    /// Printed name per binder, innermost last.
    frames: Vec<Vec<(BindingId, String)>>,
    /// How many times each spelling is currently claimed — **counted**, not a
    /// set.
    ///
    /// A set conflates two different claims on one name. The free-name
    /// reservations made in [`Self::new`] are permanent for the whole render,
    /// while a binder's claim lasts only until its frame pops. With a set, a
    /// `Fixed` binder that happens to share a free name's spelling inserts a
    /// no-op and then *removes the reservation* when it pops — after which a
    /// later `Fresh` binder is free to allocate that spelling and capture the
    /// free reference. Counting keeps the two claims independent.
    taken: HashMap<String, usize>,
    /// Every `BindingId` this render has already introduced, **never cleared**.
    ///
    /// One `BindingId` is one binding site. Reusing it in two places —
    /// `lambda1([b], lambda1([b], local(b)))`, two lambda parameters, or two
    /// sequential `Let`s — gives one structural identity two binding sites, and
    /// `Local(b)` then changes referent purely because `lookup` walks frames
    /// innermost-first. It also silently breaks `substitute`, which would
    /// rewrite both.
    ///
    /// Checked across the whole render rather than only against live frames:
    /// two *sibling* scopes reusing an id never overlap, so `lookup` stays
    /// unambiguous, but the identity is still duplicated and substitution is
    /// still wrong.
    introduced: std::collections::HashSet<BindingId>,
}

impl<'a> Scope<'a> {
    /// Reserve every free [`KtName`] the tree references **before** allocating
    /// any binder name.
    ///
    /// Without this a machine-allocated binder could shadow a name the tree
    /// refers to — the same capture the `BindingId` scheme prevents for
    /// binders, arriving through the one position `BindingId`s do not cover.
    fn new(arena: &'a ExprArena, root: &KtExpr) -> Self {
        let mut taken: HashMap<String, usize> = HashMap::new();
        for n in free_names(root) {
            // A qualified name renders as its last segment once imported, so
            // that is the spelling a binder could collide with. These claims
            // are never released.
            *taken.entry(n.simple().to_string()).or_default() += 1;
        }
        Self {
            arena,
            frames: Vec::new(),
            taken,
            introduced: std::collections::HashSet::new(),
        }
    }

    fn push(&mut self, binders: &[BindingId]) {
        let mut frame = Vec::new();
        for id in binders {
            assert!(
                self.introduced.insert(*id),
                "BindingId({}) is introduced twice in one tree — one binder identity cannot have \
                 two binding sites: `Local` would change referent by nesting depth, and \
                 `substitute` would rewrite both. Allocate a second binder, or graft the \
                 duplicated subtree into fresh ids.",
                id.index()
            );
            let name = match &self.arena.binder(*id).spelling {
                // Public API: preserved byte-identically. A `Fixed` name is
                // *claimed*, never renamed, so a later `Fresh` binder avoids it
                // rather than the other way round.
                Spelling::Fixed(n) => n.as_str().to_string(),
                Spelling::Fresh(hint) => self.allocate(hint.as_str()),
            };
            *self.taken.entry(name.clone()).or_default() += 1;
            frame.push((*id, name));
        }
        self.frames.push(frame);
    }

    fn pop(&mut self) {
        if let Some(frame) = self.frames.pop() {
            for (_, name) in frame {
                if let Some(count) = self.taken.get_mut(&name) {
                    *count -= 1;
                    if *count == 0 {
                        self.taken.remove(&name);
                    }
                }
            }
        }
    }

    fn is_taken(&self, name: &str) -> bool {
        self.taken.contains_key(name)
    }

    /// Whether a **live binder** currently prints as `simple`.
    ///
    /// Distinct from [`Self::is_taken`], which also counts the permanent
    /// free-name reservations: a reserved-but-unbound name shadows nothing.
    fn live_binder_named(&self, simple: &str) -> bool {
        self.frames.iter().flatten().any(|(_, name)| name == simple)
    }

    /// `hint`, else `hint2`, `hint3`, … — the first spelling that is neither
    /// taken nor a Kotlin hard keyword.
    ///
    /// A hint is advisory and arrives from a leaf name or a field name, so it
    /// can easily be `when` or `object`; emitting that bare would be
    /// uncompilable. Suffixing sidesteps it with no extra machinery — `when`
    /// becomes `when2`.
    fn allocate(&self, hint: &str) -> String {
        if !self.is_taken(hint) && !is_hard_keyword(hint) {
            return hint.to_string();
        }
        (2..)
            .map(|i| format!("{hint}{i}"))
            .find(|c| !self.is_taken(c) && !is_hard_keyword(c))
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

/// Render `expr` with no enclosing binders.
pub fn render_expr(arena: &ExprArena, expr: &KtExpr, imports: &mut ImportSet) -> String {
    render_expr_in_scope(arena, &[], expr, imports)
}

/// Render `expr` with `outer` already bound — the enclosing declaration's
/// parameters.
///
/// This is what lets a typed function body, default, supertype argument or
/// accessor reference its own parameter through `Local`. Without it every slot
/// would render in a fresh scope and such a reference would be unbound;
/// reaching for `Name("initialPtr")` instead would put a binder back into the
/// free-name set and restore the textual capture risk `BindingId` removes.
pub fn render_expr_in_scope(
    arena: &ExprArena,
    outer: &[BindingId],
    expr: &KtExpr,
    imports: &mut ImportSet,
) -> String {
    let mut scope = Scope::new(arena, expr);
    scope.push(outer);
    let out = write_expr(expr, &mut scope, imports, Prec::Elvis);
    scope.pop();
    out
}

/// Render `expr` with `outer` bound, **also returning the printed name of each
/// binder in `outer`**.
///
/// The renderer allocates those names, so anything that has to *spell* an outer
/// binder elsewhere — a setter's `set(<param>)` header, say — has to ask the
/// same allocation rather than guess. Deriving the header independently is how
/// a signature ends up saying `value` while the body says `value2`.
pub fn render_expr_in_scope_named(
    arena: &ExprArena,
    outer: &[BindingId],
    expr: &KtExpr,
    imports: &mut ImportSet,
) -> (Vec<String>, String) {
    let mut scope = Scope::new(arena, expr);
    scope.push(outer);
    let names = outer.iter().map(|b| scope.lookup(*b).to_string()).collect();
    let out = write_expr(expr, &mut scope, imports, Prec::Elvis);
    scope.pop();
    (names, out)
}

/// [`render_stmts`], also returning the printed names of `outer`.
pub fn render_stmts_named(
    arena: &ExprArena,
    outer: &[BindingId],
    stmts: &[KtStmt],
    imports: &mut ImportSet,
) -> (Vec<String>, Vec<String>) {
    let mut scope = Scope::new(arena, &KtExpr::Lambda(KtLambda::new([], stmts.to_vec())));
    scope.push(outer);
    let names = outer.iter().map(|b| scope.lookup(*b).to_string()).collect();
    let out = write_stmts(stmts, &mut scope, imports);
    scope.pop();
    (names, out)
}

/// Render a statement list as a block body's lines, with `outer` already bound.
pub fn render_stmts(
    arena: &ExprArena,
    outer: &[BindingId],
    stmts: &[KtStmt],
    imports: &mut ImportSet,
) -> Vec<String> {
    // Reserve against every statement, not just the first: a name free in one
    // and taken in another must be avoided in both.
    let mut scope = Scope::new(arena, &KtExpr::Lambda(KtLambda::new([], stmts.to_vec())));
    scope.push(outer);
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
        KtExpr::Name(n) => render_name(n, scope, imports),
        KtExpr::This => "this".to_string(),
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
                None => render_name(name, scope, imports),
            };
            let mut out = format!("{head}({})", rendered_args.join(", "));
            if let Some(l) = trailing_lambda {
                out.push(' ');
                out.push_str(&write_lambda(l, scope, imports));
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
        KtExpr::Lambda(l) => write_lambda(l, scope, imports),
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

/// `{ params -> body }`, with the parameters bound for the body only.
fn write_lambda(l: &KtLambda, scope: &mut Scope, imports: &mut ImportSet) -> String {
    scope.push(&l.params);
    let names: Vec<String> = l
        .params
        .iter()
        .map(|p| scope.lookup(*p).to_string())
        .collect();
    let lines = write_stmts(&l.body, scope, imports);
    scope.pop();
    let head = if names.is_empty() {
        String::new()
    } else {
        format!("{} -> ", names.join(", "))
    };
    format!("{{ {head}{} }}", lines.join("; "))
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
///
/// **Unless a live binder already prints as that short name.** Shortening
/// `io.example.config` to `config` inside `{ config -> … }` silently turns a
/// qualified free reference into the parameter — capture through the one
/// position `BindingId`s do not cover, arriving via the import set rather than
/// via the scope.
///
/// A `Fresh` binder can never cause this: [`Scope::allocate`] avoids every
/// reserved free name. Only a `Fixed` binder can, because its spelling is
/// verbatim public API and is not the renderer's to move — so:
///
/// - a **qualified** name stays fully qualified, which reads correctly whatever
///   is in scope;
/// - a **bare** name is rejected, because there is nothing left to disambiguate
///   with: the free reference and the binder are the same token, and the only
///   fix would be renaming a `Fixed` binder that callers name in arguments.
fn render_name(n: &KtName, scope: &Scope, imports: &mut ImportSet) -> String {
    if scope.live_binder_named(n.simple()) {
        if n.is_qualified() {
            // Deliberately not `imports.short`: the whole point is to *not*
            // collapse onto the shadowed spelling.
            return n.as_str().to_string();
        }
        panic!(
            "free name `{}` is shadowed by a binder printed the same way — a bare free name and \
             a `Fixed` binder cannot be told apart in the output, and renaming the binder would \
             change a named-argument surface. Qualify the reference, or rename the binder at its \
             declaration.",
            n.as_str()
        );
    }
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
        // `-2147483648` / `-9223372036854775808` do not round-trip: Kotlin
        // parses them as unary minus applied to a *positive* literal that is
        // one past the type's maximum, and rejects them as out of range. The
        // named constants are the only spellings that carry the value.
        KtLiteral::Int(v) if *v == i32::MIN => "Int.MIN_VALUE".to_string(),
        KtLiteral::Int(v) => v.to_string(),
        KtLiteral::Long(v) if *v == i64::MIN => "Long.MIN_VALUE".to_string(),
        KtLiteral::Long(v) => format!("{v}L"),
        // Rust prints these as `NaN` / `inf` / `-inf`, none of which Kotlin
        // accepts as a literal.
        KtLiteral::Double(v) if v.is_nan() => "Double.NaN".to_string(),
        KtLiteral::Double(v) if v.is_infinite() && *v > 0.0 => {
            "Double.POSITIVE_INFINITY".to_string()
        }
        KtLiteral::Double(v) if v.is_infinite() => "Double.NEGATIVE_INFINITY".to_string(),
        KtLiteral::Double(v) => {
            // Kotlin needs a decimal point to infer `Double`.
            if v.fract() == 0.0 {
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
