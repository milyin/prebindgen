//! Kotlin **expression** AST — [`KtExpr`], [`KtStmt`], [`KtPattern`] — and the
//! [`ExprArena`] that owns their binders.
//!
//! Kotlin emission has been half AST and half string concatenation: declarations
//! go through [`super::model`], expressions were `String`s assembled by hand.
//! Every defect class that produced — `replace_ident`'s left-context bug
//! (#172), hand-numbered `e0`/`e1` lambda variables to dodge `it` shadowing,
//! `kt_access_prefix` + base + `kt_access_tail` as a textual template — is
//! invisible until Gradle compiles the output, and a wrong-but-valid expression
//! is not caught even then.
//!
//! This module is **infrastructure only**. No call site is migrated here: Stage
//! 3 (#193) rewrites the emitters that produce plan-carried expressions, and
//! #199 (Stage 5B) migrates the rest and deletes the escape hatches.
//!
//! # Binder identity is structural, not textual
//!
//! Everything that *binds* — lambda parameters, function and constructor
//! parameters, local `val`s — carries a [`BindingId`] and is referenced through
//! [`KtExpr::Local`]. [`KtName`] is a **free-name** set only: classes, members,
//! type references. That split is the whole point: an expression referencing
//! `Name("x")` inserted under a lambda that prints `x` would be captured, and
//! deciding which textual `x` belongs to which binder is exactly the reasoning
//! `replace_ident` gets wrong.
//!
//! Naming is renderer-controlled only for [`Spelling::Fresh`]. A function or
//! constructor parameter is [`Spelling::Fixed`]: those names are Kotlin's
//! named-argument surface, callable from user code, and renaming one would
//! silently break every `foo(bar = …)` call site.
//!
//! # Grafting alpha-remaps
//!
//! [`BindingId`]s are allocated per [`ExprArena`] starting at zero, so two
//! independently built trees **do** collide — and [`ExprArena::graft`] remaps
//! the incoming tree's ids as it copies them in.
//!
//! That allocation scheme is deliberate. Globally-generative ids would make
//! collision impossible by construction, which sounds safer but leaves the
//! remap unexercised and unfalsifiable: the test that proves grafting cannot
//! capture would pass whether or not the remap existed. Making collisions the
//! normal case makes the remap load-bearing.

// This tier lands before its consumers: Stage 3 (#193) rewrites the emitters
// that produce plan-carried expressions, and #199 migrates the rest. Until
// then the AST is exercised by its own tests and by nothing else — the same gap
// `SumSpec` and Tier 0 carry, and it closes with the first emitter that builds
// a tree instead of a string.
#![allow(dead_code)]

use std::collections::{BTreeSet, HashMap};

use super::types::KtType;

/// Identity of a binder within one [`ExprArena`].
///
/// Never compared across arenas — [`ExprArena::graft`] remaps rather than
/// trusting that two arenas agree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingId(u32);

impl BindingId {
    /// The raw index, for diagnostics and test assertions.
    pub fn index(self) -> u32 {
        self.0
    }
}

/// How a binder is spelled in the rendered source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Spelling {
    /// **Public API**: function and constructor parameter names, which are
    /// Kotlin's named-argument surface. Preserved byte-identically — the
    /// renderer may not rename these, and a `Fresh` binder may not shadow one.
    Fixed(KtName),
    /// Renderer-allocated: lambda parameters, locals, temporaries. The hint is
    /// a preference, not a promise.
    Fresh(NameHint),
}

/// A preferred base name for a [`Spelling::Fresh`] binder. The renderer appends
/// a disambiguating suffix when the hint is taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameHint(String);

impl NameHint {
    /// A hint. Non-identifier characters are dropped rather than rejected — a
    /// hint is advisory, and the renderer is what guarantees the emitted name
    /// is legal.
    pub fn new(s: impl AsRef<str>) -> Self {
        let cleaned: String = s
            .as_ref()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        let cleaned = cleaned.trim_start_matches(|c: char| c.is_ascii_digit());
        NameHint(if cleaned.is_empty() {
            "tmp".to_string()
        } else {
            cleaned.to_string()
        })
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A binder: its identity plus how it is spelled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binder {
    pub id: BindingId,
    pub spelling: Spelling,
}

/// A **free** Kotlin name — a class, member, or type reference. Never a binder;
/// binders are [`BindingId`]s.
///
/// Validated at construction, so a malformed identifier is rejected here rather
/// than discovered by Gradle. A dotted name is an FQN and registers an import
/// when rendered, the same way [`KtType`] does.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KtName(String);

/// Why a [`KtName`] was rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KtNameError {
    pub input: String,
    pub reason: &'static str,
}

impl std::fmt::Display for KtNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid Kotlin name `{}`: {}", self.input, self.reason)
    }
}

impl std::error::Error for KtNameError {}

impl KtName {
    /// Validate and build. Each dot-separated segment must be a Kotlin
    /// identifier: a letter or `_` followed by letters, digits or `_`.
    ///
    /// This is what stops a producer smuggling an expression through a name —
    /// without it, `Name("a.b() ?: c")` would render as arbitrary source and
    /// make #199's "no string-built expressions" exit unfalsifiable.
    pub fn new(s: impl Into<String>) -> Result<Self, KtNameError> {
        let s = s.into();
        let reject = |reason| {
            Err(KtNameError {
                input: s.clone(),
                reason,
            })
        };
        if s.is_empty() {
            return reject("empty");
        }
        for seg in s.split('.') {
            if seg.is_empty() {
                return reject("empty path segment");
            }
            let mut chars = seg.chars();
            let first = chars.next().expect("segment is non-empty");
            if !(first.is_ascii_alphabetic() || first == '_') {
                return reject("segment must start with a letter or `_`");
            }
            if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return reject("segment may only contain letters, digits and `_`");
            }
        }
        Ok(KtName(s))
    }

    /// Build, panicking on a malformed name. For generator-internal names that
    /// are structurally guaranteed valid; a malformed one is a generator bug,
    /// not a user error.
    pub fn expect(s: impl Into<String>) -> Self {
        Self::new(s).unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether this is a dotted FQN (and therefore importable).
    pub fn is_qualified(&self) -> bool {
        self.0.contains('.')
    }

    /// The last segment — what a qualified name renders as once imported.
    pub fn simple(&self) -> &str {
        self.0.rsplit('.').next().unwrap_or(&self.0)
    }
}

/// A Kotlin literal. Typed, so a string literal is escaped **by the renderer**
/// rather than pre-escaped by whoever built it.
#[derive(Clone, Debug, PartialEq)]
pub enum KtLiteral {
    Null,
    Bool(bool),
    Int(i32),
    Long(i64),
    Double(f64),
    Str(String),
}

/// A Kotlin expression.
///
/// Every variant is structured. The single exception, [`KtExpr::Raw`], is
/// crate-private and deliberately conspicuous: counting its construction sites
/// is the mechanical check behind #199's global exit.
#[derive(Clone, Debug, PartialEq)]
pub enum KtExpr {
    /// Reference to a binder introduced in this tree.
    Local(BindingId),
    /// A **free** name: class, member, or type reference.
    Name(KtName),
    Literal(KtLiteral),
    /// `recv.name` / `recv?.name`.
    Field {
        recv: Box<KtExpr>,
        name: KtName,
        safe: bool,
    },
    /// `recv.name(args)` / `name(args)` when `recv` is `None`.
    Call {
        recv: Option<Box<KtExpr>>,
        name: KtName,
        args: Vec<KtExpr>,
        /// `?.name(…)`.
        safe: bool,
        /// A trailing lambda argument, rendered outside the parentheses.
        trailing_lambda: Option<Box<KtExpr>>,
    },
    /// `expr as ty` / `expr as? ty`.
    As {
        expr: Box<KtExpr>,
        ty: KtType,
        safe: bool,
    },
    /// `lhs ?: rhs`.
    Elvis(Box<KtExpr>, Box<KtExpr>),
    /// `{ params -> body }`.
    Lambda {
        params: Vec<BindingId>,
        body: Vec<KtStmt>,
    },
    /// `when (subject) { arms }`.
    When {
        subject: Box<KtExpr>,
        arms: Vec<(KtPattern, KtExpr)>,
    },
    /// The placeholder [`fill_hole`] replaces — the tree operation that
    /// `kt_access_prefix` + base + `kt_access_tail` was simulating.
    ///
    /// Never renders. A tree still containing one is a generator bug, so the
    /// renderer panics rather than emitting something plausible.
    Hole,
    /// TEMPORARY escape hatch. Crate-private, every construction site tracked
    /// in #199, whose exit deletes this variant.
    Raw(String),
}

/// A statement inside a lambda or block body.
#[derive(Clone, Debug, PartialEq)]
pub enum KtStmt {
    /// `val <binder> = <expr>` (or `var`).
    Let {
        binder: BindingId,
        mutable: bool,
        value: KtExpr,
    },
    /// A bare expression statement — and, in final position, a lambda's or
    /// expression-body's result.
    Expr(KtExpr),
    /// `return <expr>` / `return`.
    Return(Option<KtExpr>),
}

/// A `when` arm pattern.
#[derive(Clone, Debug, PartialEq)]
pub enum KtPattern {
    /// `is <ty> ->`.
    Is(KtType),
    /// A constant: `<expr> ->`.
    Value(KtExpr),
    /// `else ->`.
    Else,
}

/// Owns the binders of one or more expression trees.
///
/// Ids start at zero per arena, so two independently built arenas collide —
/// see the module docs for why that is the point.
#[derive(Clone, Debug, Default)]
pub struct ExprArena {
    binders: Vec<Binder>,
}

impl ExprArena {
    pub fn new() -> Self {
        Self::default()
    }

    /// A binder whose spelling is **public API** and must survive rendering
    /// byte-identically: a function or constructor parameter.
    pub fn bind_fixed(&mut self, name: KtName) -> BindingId {
        self.push(Spelling::Fixed(name))
    }

    /// A binder the renderer may name: a lambda parameter, a local, a
    /// temporary.
    pub fn bind_fresh(&mut self, hint: impl AsRef<str>) -> BindingId {
        self.push(Spelling::Fresh(NameHint::new(hint)))
    }

    fn push(&mut self, spelling: Spelling) -> BindingId {
        let id = BindingId(self.binders.len() as u32);
        self.binders.push(Binder { id, spelling });
        id
    }

    /// The binder behind an id.
    pub fn binder(&self, id: BindingId) -> &Binder {
        &self.binders[id.0 as usize]
    }

    /// Number of binders — the count a remap test compares against.
    pub fn len(&self) -> usize {
        self.binders.len()
    }

    pub fn is_empty(&self) -> bool {
        self.binders.is_empty()
    }

    /// Copy a tree built in `from` into this arena, **alpha-remapping** every
    /// binder it introduces and every reference to one.
    ///
    /// Without this, grafting two trees that each allocated `BindingId(0)`
    /// would silently merge two unrelated binders — structural capture that
    /// scope-aware *rendering* cannot detect, because at render time the two
    /// are indistinguishable.
    ///
    /// A `Local` referring to a binder the incoming tree does not itself
    /// introduce is a dangling reference and is rejected: it can only mean the
    /// tree was built against a scope that is not being grafted with it.
    pub fn graft(&mut self, from: &ExprArena, expr: &KtExpr) -> KtExpr {
        let mut map = HashMap::new();
        // Introduce first, so a `Local` appearing before its binder in
        // traversal order (a lambda body referring to its own parameter) still
        // resolves.
        self.remap_introductions(from, expr, &mut map);
        self.rewrite_ids(expr, &map)
    }

    /// Allocate a fresh id in `self` for every binder `expr` introduces.
    fn remap_introductions(
        &mut self,
        from: &ExprArena,
        expr: &KtExpr,
        map: &mut HashMap<BindingId, BindingId>,
    ) {
        match expr {
            KtExpr::Lambda { params, body } => {
                for p in params {
                    let fresh = self.push(from.binder(*p).spelling.clone());
                    map.insert(*p, fresh);
                }
                for s in body {
                    self.remap_stmt_introductions(from, s, map);
                }
            }
            KtExpr::Field { recv, .. } => self.remap_introductions(from, recv, map),
            KtExpr::Call {
                recv,
                args,
                trailing_lambda,
                ..
            } => {
                if let Some(r) = recv {
                    self.remap_introductions(from, r, map);
                }
                for a in args {
                    self.remap_introductions(from, a, map);
                }
                if let Some(l) = trailing_lambda {
                    self.remap_introductions(from, l, map);
                }
            }
            KtExpr::As { expr, .. } => self.remap_introductions(from, expr, map),
            KtExpr::Elvis(a, b) => {
                self.remap_introductions(from, a, map);
                self.remap_introductions(from, b, map);
            }
            KtExpr::When { subject, arms } => {
                self.remap_introductions(from, subject, map);
                for (p, e) in arms {
                    if let KtPattern::Value(v) = p {
                        self.remap_introductions(from, v, map);
                    }
                    self.remap_introductions(from, e, map);
                }
            }
            KtExpr::Local(_)
            | KtExpr::Name(_)
            | KtExpr::Literal(_)
            | KtExpr::Hole
            | KtExpr::Raw(_) => {}
        }
    }

    fn remap_stmt_introductions(
        &mut self,
        from: &ExprArena,
        stmt: &KtStmt,
        map: &mut HashMap<BindingId, BindingId>,
    ) {
        match stmt {
            KtStmt::Let { binder, value, .. } => {
                self.remap_introductions(from, value, map);
                let fresh = self.push(from.binder(*binder).spelling.clone());
                map.insert(*binder, fresh);
            }
            KtStmt::Expr(e) => self.remap_introductions(from, e, map),
            KtStmt::Return(Some(e)) => self.remap_introductions(from, e, map),
            KtStmt::Return(None) => {}
        }
    }

    /// Rewrite **both** references and binding positions.
    ///
    /// `map_expr` alone is not enough: it rewrites `Local`s but copies a
    /// `Lambda`'s `params` and a `Let`'s `binder` through unchanged, which
    /// would leave the incoming tree binding the host's ids — precisely the
    /// capture the remap exists to prevent.
    fn rewrite_ids(&self, expr: &KtExpr, map: &HashMap<BindingId, BindingId>) -> KtExpr {
        let lookup = |id: &BindingId| {
            *map.get(id).unwrap_or_else(|| {
                panic!(
                    "ExprArena::graft: `Local({})` refers to a binder the grafted tree does not \
                     introduce — it was built against a scope that is not being grafted with it",
                    id.0
                )
            })
        };
        match expr {
            KtExpr::Local(id) => KtExpr::Local(lookup(id)),
            KtExpr::Name(_) | KtExpr::Literal(_) | KtExpr::Hole | KtExpr::Raw(_) => expr.clone(),
            KtExpr::Field { recv, name, safe } => KtExpr::Field {
                recv: Box::new(self.rewrite_ids(recv, map)),
                name: name.clone(),
                safe: *safe,
            },
            KtExpr::Call {
                recv,
                name,
                args,
                safe,
                trailing_lambda,
            } => KtExpr::Call {
                recv: recv.as_ref().map(|r| Box::new(self.rewrite_ids(r, map))),
                name: name.clone(),
                args: args.iter().map(|a| self.rewrite_ids(a, map)).collect(),
                safe: *safe,
                trailing_lambda: trailing_lambda
                    .as_ref()
                    .map(|l| Box::new(self.rewrite_ids(l, map))),
            },
            KtExpr::As { expr, ty, safe } => KtExpr::As {
                expr: Box::new(self.rewrite_ids(expr, map)),
                ty: ty.clone(),
                safe: *safe,
            },
            KtExpr::Elvis(a, b) => KtExpr::Elvis(
                Box::new(self.rewrite_ids(a, map)),
                Box::new(self.rewrite_ids(b, map)),
            ),
            KtExpr::Lambda { params, body } => KtExpr::Lambda {
                params: params.iter().map(lookup).collect(),
                body: body.iter().map(|s| self.rewrite_stmt_ids(s, map)).collect(),
            },
            KtExpr::When { subject, arms } => KtExpr::When {
                subject: Box::new(self.rewrite_ids(subject, map)),
                arms: arms
                    .iter()
                    .map(|(p, e)| {
                        let p = match p {
                            KtPattern::Value(v) => KtPattern::Value(self.rewrite_ids(v, map)),
                            other => other.clone(),
                        };
                        (p, self.rewrite_ids(e, map))
                    })
                    .collect(),
            },
        }
    }

    fn rewrite_stmt_ids(&self, stmt: &KtStmt, map: &HashMap<BindingId, BindingId>) -> KtStmt {
        match stmt {
            KtStmt::Let {
                binder,
                mutable,
                value,
            } => KtStmt::Let {
                binder: *map
                    .get(binder)
                    .expect("a Let's binder is introduced by its own tree"),
                mutable: *mutable,
                value: self.rewrite_ids(value, map),
            },
            KtStmt::Expr(e) => KtStmt::Expr(self.rewrite_ids(e, map)),
            KtStmt::Return(e) => KtStmt::Return(e.as_ref().map(|e| self.rewrite_ids(e, map))),
        }
    }
}

/// Rewrite a tree bottom-up, replacing any node `f` answers `Some` for.
///
/// The one traversal every tree operation goes through, so a variant added
/// later cannot be silently skipped by half of them.
pub fn map_expr(expr: &KtExpr, f: &mut dyn FnMut(&KtExpr) -> Option<KtExpr>) -> KtExpr {
    if let Some(replacement) = f(expr) {
        return replacement;
    }
    match expr {
        KtExpr::Local(_) | KtExpr::Name(_) | KtExpr::Literal(_) | KtExpr::Hole | KtExpr::Raw(_) => {
            expr.clone()
        }
        KtExpr::Field { recv, name, safe } => KtExpr::Field {
            recv: Box::new(map_expr(recv, f)),
            name: name.clone(),
            safe: *safe,
        },
        KtExpr::Call {
            recv,
            name,
            args,
            safe,
            trailing_lambda,
        } => KtExpr::Call {
            recv: recv.as_ref().map(|r| Box::new(map_expr(r, f))),
            name: name.clone(),
            args: args.iter().map(|a| map_expr(a, f)).collect(),
            safe: *safe,
            trailing_lambda: trailing_lambda.as_ref().map(|l| Box::new(map_expr(l, f))),
        },
        KtExpr::As { expr, ty, safe } => KtExpr::As {
            expr: Box::new(map_expr(expr, f)),
            ty: ty.clone(),
            safe: *safe,
        },
        KtExpr::Elvis(a, b) => KtExpr::Elvis(Box::new(map_expr(a, f)), Box::new(map_expr(b, f))),
        KtExpr::Lambda { params, body } => KtExpr::Lambda {
            params: params.clone(),
            body: body.iter().map(|s| map_stmt(s, f)).collect(),
        },
        KtExpr::When { subject, arms } => KtExpr::When {
            subject: Box::new(map_expr(subject, f)),
            arms: arms
                .iter()
                .map(|(p, e)| {
                    let p = match p {
                        KtPattern::Value(v) => KtPattern::Value(map_expr(v, f)),
                        other => other.clone(),
                    };
                    (p, map_expr(e, f))
                })
                .collect(),
        },
    }
}

/// [`map_expr`] over a statement.
pub fn map_stmt(stmt: &KtStmt, f: &mut dyn FnMut(&KtExpr) -> Option<KtExpr>) -> KtStmt {
    match stmt {
        KtStmt::Let {
            binder,
            mutable,
            value,
        } => KtStmt::Let {
            binder: *binder,
            mutable: *mutable,
            value: map_expr(value, f),
        },
        KtStmt::Expr(e) => KtStmt::Expr(map_expr(e, f)),
        KtStmt::Return(e) => KtStmt::Return(e.as_ref().map(|e| map_expr(e, f))),
    }
}

/// Fill every [`KtExpr::Hole`] in `template` with `value`.
///
/// This is the tree operation the `kt_access_prefix` + base + `kt_access_tail`
/// triple was simulating: a template with the base in the middle
/// (`(<base>.field as? Reading.Exact)?.v0 ?: 0L`) is just a tree with a hole,
/// and filling it is a rewrite rather than three string concatenations that
/// each have to remember the other two.
pub fn fill_hole(template: &KtExpr, value: &KtExpr) -> KtExpr {
    map_expr(template, &mut |e| match e {
        KtExpr::Hole => Some(value.clone()),
        _ => None,
    })
}

/// Whether a tree still contains an unfilled [`KtExpr::Hole`].
pub fn has_hole(expr: &KtExpr) -> bool {
    let mut found = false;
    map_expr(expr, &mut |e| {
        if matches!(e, KtExpr::Hole) {
            found = true;
        }
        None
    });
    found
}

/// Replace every `Local(target)` in `expr` with `value`.
///
/// The replacement for `replace_ident`. Capture is structurally impossible: a
/// reference is a [`BindingId`], not a spelling, so inserting a tree under a
/// binder that happens to *print* the same name cannot rebind it — the renderer
/// allocates the printed names afterwards, and it knows which id each one
/// belongs to.
pub fn substitute(expr: &KtExpr, target: BindingId, value: &KtExpr) -> KtExpr {
    map_expr(expr, &mut |e| match e {
        KtExpr::Local(id) if *id == target => Some(value.clone()),
        _ => None,
    })
}

/// Every free [`KtName`] reachable in a tree.
///
/// Two uses: the import set a rendered unit reports, and the reservation the
/// renderer makes before allocating printable names — a machine-allocated
/// binder may not shadow a name the tree references.
pub fn free_names(expr: &KtExpr) -> BTreeSet<KtName> {
    let mut out = BTreeSet::new();
    map_expr(expr, &mut |e| {
        match e {
            KtExpr::Name(n) => {
                out.insert(n.clone());
            }
            KtExpr::Field { name, .. } | KtExpr::Call { name, .. } => {
                out.insert(name.clone());
            }
            _ => {}
        }
        None
    });
    out
}

// ── convenience constructors ────────────────────────────────────────────
//
// Every one of these builds a structured node; none of them accepts free-form
// expression text. That is what makes #199's exit checkable: the only way to
// get a string into a tree is `KtExpr::Raw`, and its construction sites are
// counted.

impl KtExpr {
    pub fn local(id: BindingId) -> Self {
        KtExpr::Local(id)
    }
    pub fn name(n: impl Into<String>) -> Self {
        KtExpr::Name(KtName::expect(n))
    }
    pub fn null() -> Self {
        KtExpr::Literal(KtLiteral::Null)
    }
    pub fn bool_(v: bool) -> Self {
        KtExpr::Literal(KtLiteral::Bool(v))
    }
    pub fn int(v: i32) -> Self {
        KtExpr::Literal(KtLiteral::Int(v))
    }
    pub fn long(v: i64) -> Self {
        KtExpr::Literal(KtLiteral::Long(v))
    }
    pub fn str_(v: impl Into<String>) -> Self {
        KtExpr::Literal(KtLiteral::Str(v.into()))
    }
    /// `self.name`.
    pub fn field(self, name: impl Into<String>) -> Self {
        KtExpr::Field {
            recv: Box::new(self),
            name: KtName::expect(name),
            safe: false,
        }
    }
    /// `self?.name`.
    pub fn safe_field(self, name: impl Into<String>) -> Self {
        KtExpr::Field {
            recv: Box::new(self),
            name: KtName::expect(name),
            safe: true,
        }
    }
    /// `self.name(args)`.
    pub fn call(self, name: impl Into<String>, args: impl IntoIterator<Item = KtExpr>) -> Self {
        KtExpr::Call {
            recv: Some(Box::new(self)),
            name: KtName::expect(name),
            args: args.into_iter().collect(),
            safe: false,
            trailing_lambda: None,
        }
    }
    /// `self?.name(args)`.
    pub fn safe_call(
        self,
        name: impl Into<String>,
        args: impl IntoIterator<Item = KtExpr>,
    ) -> Self {
        KtExpr::Call {
            recv: Some(Box::new(self)),
            name: KtName::expect(name),
            args: args.into_iter().collect(),
            safe: true,
            trailing_lambda: None,
        }
    }
    /// `name(args)` — an unqualified call.
    pub fn free_call(name: impl Into<String>, args: impl IntoIterator<Item = KtExpr>) -> Self {
        KtExpr::Call {
            recv: None,
            name: KtName::expect(name),
            args: args.into_iter().collect(),
            safe: false,
            trailing_lambda: None,
        }
    }
    /// Attach a trailing lambda: `f(a) { … }`.
    pub fn with_trailing_lambda(mut self, lambda: KtExpr) -> Self {
        match &mut self {
            KtExpr::Call {
                trailing_lambda, ..
            } => *trailing_lambda = Some(Box::new(lambda)),
            other => panic!("a trailing lambda needs a call, got {other:?}"),
        }
        self
    }
    /// `self as ty`.
    pub fn cast(self, ty: KtType) -> Self {
        KtExpr::As {
            expr: Box::new(self),
            ty,
            safe: false,
        }
    }
    /// `self as? ty`.
    pub fn safe_cast(self, ty: KtType) -> Self {
        KtExpr::As {
            expr: Box::new(self),
            ty,
            safe: true,
        }
    }
    /// `self ?: other`.
    pub fn elvis(self, other: KtExpr) -> Self {
        KtExpr::Elvis(Box::new(self), Box::new(other))
    }
    /// `{ params -> body }`.
    pub fn lambda(params: impl IntoIterator<Item = BindingId>, body: Vec<KtStmt>) -> Self {
        KtExpr::Lambda {
            params: params.into_iter().collect(),
            body,
        }
    }
    /// A single-expression lambda body.
    pub fn lambda1(params: impl IntoIterator<Item = BindingId>, body: KtExpr) -> Self {
        Self::lambda(params, vec![KtStmt::Expr(body)])
    }
}

pub mod render;

#[cfg(test)]
mod tests;
