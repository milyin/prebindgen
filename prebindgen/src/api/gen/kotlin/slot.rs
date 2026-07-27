//! Exclusive bridges between the legacy textual expression fields and the typed
//! [`KtExpr`](super::expr::KtExpr) tier.
//!
//! # Why a sum and not two fields
//!
//! Introducing typed replacements *alongside* the legacy fields would let both
//! be populated at once and leave the renderer to decide which wins — two
//! authorities for one fact, which is the defect #187 exists to remove,
//! recreated inside its own migration.
//!
//! [`ExprSlot`] is a sum, so an expression position holds a textual answer or a
//! structured one and never both. #199 deletes the `Legacy` variants; until
//! then the type is what enforces the exclusion, not review.
//!
//! # The positions
//!
//! Every API able to embed expression text is covered, not only the ones whose
//! grammatical category is "expression":
//!
//! | Position | Was | Now |
//! |---|---|---|
//! | function body | `KtBody::Expr(Code)` / `Block(Code)` | `ExprSlot<KtExpr>` / `ExprSlot<Vec<KtStmt>>` |
//! | enum entry args | `Option<String>` | `ExprSlot<Vec<KtExpr>>` |
//! | ctor param default | `Option<String>` | `ExprSlot<KtExpr>` |
//! | fn param default | `Option<String>` | `ExprSlot<KtExpr>` |
//! | property init / delegate | two `Option<String>` fields | [`PropertyValue`] |
//! | supertype ctor args | `Option<String>` | `ExprSlot<Vec<KtExpr>>` |
//! | property accessors | `Option<Code>` | [`KtAccessor`] |
//! | annotations | `Vec<String>` | `Vec<AnnotationSlot>` |
//!
//! Two of those are not expression slots and need more than a slot:
//! **accessors are declarations containing bodies**, so they get a structured
//! [`KtAccessor`]; and **annotation arguments are expressions**, so
//! [`KtAnnotation`] carries `Vec<KtExpr>`.

// Same reason as `expr`: the typed arms exist so #193 and #199 have somewhere
// to land, and nothing constructs one yet.
#![allow(dead_code)]

use super::{
    code::Code,
    expr::{BindingId, ExprArena, KtExpr, KtName, KtStmt},
};

/// A typed tree together with the arena owning its binders.
///
/// The arena travels **with** the tree rather than sitting on the enclosing
/// file or function, so a slot renders with no ambient state and two slots can
/// be built independently. Merging them is exactly what
/// [`ExprArena::graft`](super::expr::ExprArena::graft) is for, and it
/// alpha-remaps — which is why independent arenas are safe to hand around.
#[derive(Clone, Debug)]
pub struct Ast<T> {
    pub arena: ExprArena,
    /// Binders already in scope at the tree's **root** — the enclosing
    /// declaration's parameters.
    ///
    /// Without this a typed function body could not reference its own
    /// parameter: every slot renders in a fresh scope, so `Local(param)` would
    /// be unbound and the renderer would panic. Reaching for
    /// `Name("initialPtr")` instead would put a binder back in the free-name
    /// set and restore exactly the textual capture risk `BindingId` exists to
    /// remove — so the parameter has to arrive as a binder, not as a name.
    ///
    /// Kept on the `Ast` rather than on the declaration so a slot still renders
    /// with no ambient state, and so the binders and the arena that owns them
    /// cannot be supplied from two different places.
    pub scope: Vec<BindingId>,
    pub tree: T,
}

impl<T> Ast<T> {
    /// A tree with no enclosing binders.
    pub fn new(arena: ExprArena, tree: T) -> Self {
        Self {
            arena,
            scope: Vec::new(),
            tree,
        }
    }

    /// A tree rendered with `scope` already bound — see [`Self::scope`].
    pub fn in_scope(arena: ExprArena, scope: Vec<BindingId>, tree: T) -> Self {
        Self { arena, scope, tree }
    }
}

/// One expression position: a legacy textual answer **or** a structured one.
///
/// Mutually exclusive by construction — there is no state in which both are
/// present, so the renderer never chooses.
#[derive(Clone, Debug)]
pub enum ExprSlot<T> {
    /// Pre-rendered text. Deleted by #199.
    Legacy(Code),
    /// The typed tree plus its arena.
    Ast(Ast<T>),
}

impl<T> ExprSlot<T> {
    /// Wrap pre-rendered text — the mechanical 5A bridge.
    pub fn legacy(code: Code) -> Self {
        ExprSlot::Legacy(code)
    }

    /// Wrap a typed tree.
    pub fn ast(arena: ExprArena, tree: T) -> Self {
        ExprSlot::Ast(Ast::new(arena, tree))
    }

    /// Wrap a typed tree that may reference the enclosing declaration's
    /// parameters — see [`Ast::scope`].
    pub fn ast_in_scope(arena: ExprArena, scope: Vec<BindingId>, tree: T) -> Self {
        ExprSlot::Ast(Ast::in_scope(arena, scope, tree))
    }

    /// Whether this slot still holds text — the predicate #199's exit counts.
    pub fn is_legacy(&self) -> bool {
        matches!(self, ExprSlot::Legacy(_))
    }
}

/// A property's value, replacing the prose-enforced initializer/delegate
/// exclusion.
///
/// `KtProperty` used to hold `initializer: Option<String>` and `delegate:
/// Option<String>` with a doc comment noting they are "mutually exclusive" —
/// a product where a sum belongs, enforced by prose and by a `debug_assert` in
/// the renderer. That is the #180 pattern sitting in the Kotlin model, and it
/// is fixed here for the same reason #180 fixed it in `jnigen`: a state that
/// cannot be represented cannot be reached.
#[derive(Clone, Debug, Default)]
pub enum PropertyValue {
    /// No value — an abstract property, or one with only accessors.
    #[default]
    None,
    /// `val x = <expr>`.
    Initializer(ExprSlot<KtExpr>),
    /// `val x by <expr>` (e.g. `lazy { … }`).
    Delegate(ExprSlot<KtExpr>),
}

/// Which accessor of a property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessorKind {
    Get,
    Set,
}

/// A property accessor.
///
/// **A declaration containing a body**, not an expression — which is why it is
/// not an `ExprSlot<KtExpr>`. A getter may be `get() = <expr>` or `get() { … }`,
/// and a setter binds its parameter.
#[derive(Clone, Debug)]
pub struct KtAccessor {
    pub kind: AccessorKind,
    /// `= <expr>` when `Some(expr)`, else a block of `body`.
    pub body: AccessorBody,
}

/// An accessor's body — expression form or block form.
#[derive(Clone, Debug)]
pub enum AccessorBody {
    /// Legacy pre-rendered accessor text, verbatim. Deleted by #199.
    Legacy(Code),
    /// `get() = <expr>`.
    Expr(Ast<KtExpr>),
    /// `get() { <stmts> }`.
    Block(Ast<Vec<KtStmt>>),
}

impl KtAccessor {
    /// Wrap pre-rendered accessor text. The mechanical bridge; #199 replaces
    /// each caller with a structured accessor.
    pub fn legacy(code: Code) -> Self {
        Self {
            kind: AccessorKind::Get,
            body: AccessorBody::Legacy(code),
        }
    }
    pub fn get_expr(arena: ExprArena, e: KtExpr) -> Self {
        Self {
            kind: AccessorKind::Get,
            body: AccessorBody::Expr(Ast::new(arena, e)),
        }
    }
    pub fn is_legacy(&self) -> bool {
        matches!(self.body, AccessorBody::Legacy(_))
    }
}

/// A Kotlin annotation: a name plus **expression** arguments.
///
/// Annotations were the one expression-bearing position left as bare `String`.
/// The codebase already writes `.annotation("Suppress(\"UNCHECKED_CAST\")")` —
/// a call with a string-literal argument, i.e. an expression spelled as text.
#[derive(Clone, Debug, PartialEq)]
pub struct KtAnnotation {
    pub name: KtName,
    pub args: Vec<KtExpr>,
}

impl KtAnnotation {
    pub fn new(name: KtName) -> Self {
        Self {
            name,
            args: Vec::new(),
        }
    }
    pub fn arg(mut self, e: KtExpr) -> Self {
        self.args.push(e);
        self
    }
}

/// Annotation text that is **intended** to be of literal origin.
///
/// Two constructors reach it, and neither makes literal origin a property of
/// the *type*:
///
/// - [`kt_annotation_text!`] takes a `literal` fragment, so **its own callers**
///   cannot pass `String::leak(…)` — the macro is where the guarantee lives.
/// - [`Self::__from_literal`] is what the macro expands to, and on its own it
///   accepts any `&'static str`, leaked included. Narrowing the payload to
///   `&'static str` was never enough for exactly that reason: the type proves
///   lifetime, not origin.
/// - [`Self::from_legacy_string`] is the 5A bridge that let the field type
///   change without touching call sites, and takes an owned `String`.
///
/// So the guarantee here is **audited, not typed**. Both direct constructors
/// are crate-internal (`api::gen` is `pub(crate)`), which bounds the audit to
/// this crate, and a test pins the call sites of each so neither can grow
/// unnoticed. #199 drives both to zero and deletes them — the same
/// enumerate-then-delete contract `KtExpr::Raw` is under.
///
/// Sealing `__from_literal` properly would need a witness type the macro can
/// mint and nothing else can; that is worth doing when this type outlives 5B,
/// and pointless if #199 deletes it first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticAnnotationText(std::borrow::Cow<'static, str>);

impl StaticAnnotationText {
    /// Do not call directly — use [`kt_annotation_text!`], which accepts only a
    /// string literal.
    #[doc(hidden)]
    pub const fn __from_literal(s: &'static str) -> Self {
        StaticAnnotationText(std::borrow::Cow::Borrowed(s))
    }

    /// The mechanical 5A bridge: wrap an existing textual annotation so the
    /// field type can change without migrating call sites.
    ///
    /// **Every caller is a #199 work item.** It is deliberately named so that
    /// `grep from_legacy_string` is the progress metric, and a test pins the
    /// count so it cannot grow unnoticed.
    pub fn from_legacy_string(s: impl Into<String>) -> Self {
        StaticAnnotationText(std::borrow::Cow::Owned(s.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Build a [`StaticAnnotationText`] from a **string literal** — the form that
/// makes literal origin a compile-time property, since `macro_rules!`'s
/// `literal` fragment cannot match `String::leak(…)` or any other expression.
#[allow(unused_macros)]
macro_rules! kt_annotation_text {
    ($s:literal) => {
        $crate::api::gen::kotlin::slot::StaticAnnotationText::__from_literal($s)
    };
}

#[allow(unused_imports)]
pub(crate) use kt_annotation_text;

/// One annotation: legacy text **or** a structured annotation. Same exclusion
/// [`ExprSlot`] provides, for the one position that is not an expression slot.
#[derive(Clone, Debug, PartialEq)]
pub enum AnnotationSlot {
    /// Literal-origin text. Deleted by #199.
    Legacy(StaticAnnotationText),
    Ast(KtAnnotation),
}

impl AnnotationSlot {
    pub fn is_legacy(&self) -> bool {
        matches!(self, AnnotationSlot::Legacy(_))
    }
}

// ── rendering and import collection ─────────────────────────────────────
//
// A slot answers for itself in both directions, so the declaration renderer
// never has to know which arm it holds — which is what keeps the `Legacy`
// deletion in #199 a local change.

use super::{
    expr::render::{render_expr, render_expr_in_scope},
    types::ImportSet,
};

impl ExprSlot<KtExpr> {
    /// Render to a single Kotlin expression.
    pub fn render_inline(&self, imports: &mut ImportSet) -> String {
        match self {
            ExprSlot::Legacy(c) => {
                let mut s = String::new();
                c.render(0, &mut s);
                s.trim_end().to_string()
            }
            ExprSlot::Ast(a) => render_expr_in_scope(&a.arena, &a.scope, &a.tree, imports),
        }
    }

    /// The FQNs this slot references.
    ///
    /// For a typed tree they come **from the tree**, which is the point: a
    /// rendered unit reports the imports it needs instead of having them
    /// registered by hand alongside raw text, where the two could drift.
    pub fn collect_imports(&self, sink: &mut Vec<String>) {
        match self {
            ExprSlot::Legacy(c) => c.collect_imports(sink),
            ExprSlot::Ast(a) => sink.extend(
                super::expr::free_names(&a.tree)
                    .into_iter()
                    .filter(|n| n.is_qualified())
                    .map(|n| n.as_str().to_string()),
            ),
        }
    }
}

impl ExprSlot<Vec<KtExpr>> {
    /// Render as a comma-separated argument list (no surrounding parentheses).
    pub fn render_args(&self, imports: &mut ImportSet) -> String {
        match self {
            ExprSlot::Legacy(c) => {
                let mut s = String::new();
                c.render(0, &mut s);
                s.trim_end().to_string()
            }
            ExprSlot::Ast(a) => a
                .tree
                .iter()
                .map(|e| render_expr_in_scope(&a.arena, &a.scope, e, imports))
                .collect::<Vec<_>>()
                .join(", "),
        }
    }

    pub fn collect_imports(&self, sink: &mut Vec<String>) {
        match self {
            ExprSlot::Legacy(c) => c.collect_imports(sink),
            ExprSlot::Ast(a) => {
                for e in &a.tree {
                    sink.extend(
                        super::expr::free_names(e)
                            .into_iter()
                            .filter(|n| n.is_qualified())
                            .map(|n| n.as_str().to_string()),
                    );
                }
            }
        }
    }
}

impl ExprSlot<Vec<KtStmt>> {
    /// Render as block-body lines.
    pub fn render_lines(&self, imports: &mut ImportSet) -> Code {
        match self {
            ExprSlot::Legacy(c) => c.clone(),
            ExprSlot::Ast(a) => {
                let mut code = Code::new();
                for line in super::expr::render::render_stmts(&a.arena, &a.scope, &a.tree, imports)
                {
                    code = code.line(line);
                }
                code
            }
        }
    }

    pub fn collect_imports(&self, sink: &mut Vec<String>) {
        match self {
            ExprSlot::Legacy(c) => c.collect_imports(sink),
            ExprSlot::Ast(a) => {
                let wrapper = KtExpr::Lambda {
                    params: Vec::new(),
                    body: a.tree.clone(),
                };
                sink.extend(
                    super::expr::free_names(&wrapper)
                        .into_iter()
                        .filter(|n| n.is_qualified())
                        .map(|n| n.as_str().to_string()),
                );
            }
        }
    }
}

impl PropertyValue {
    pub fn collect_imports(&self, sink: &mut Vec<String>) {
        match self {
            PropertyValue::None => {}
            PropertyValue::Initializer(s) | PropertyValue::Delegate(s) => s.collect_imports(sink),
        }
    }
}

impl KtAccessor {
    /// Render the accessor's lines, indented under the property by the caller.
    pub fn render_lines(&self, imports: &mut ImportSet) -> Code {
        let head = match self.kind {
            AccessorKind::Get => "get()",
            AccessorKind::Set => "set(value)",
        };
        match &self.body {
            AccessorBody::Legacy(c) => c.clone(),
            // `render_expr_in_scope`, like every other typed slot: a getter or
            // setter expression referencing a constructor parameter or the
            // setter's own binder would otherwise be unbound.
            AccessorBody::Expr(a) => Code::new().line(format!(
                "{head} = {}",
                render_expr_in_scope(&a.arena, &a.scope, &a.tree, imports)
            )),
            AccessorBody::Block(a) => {
                let mut code = Code::new();
                let lines = super::expr::render::render_stmts(&a.arena, &a.scope, &a.tree, imports);
                code = code.blk(format!("{head} {{"), |c| {
                    let mut c = c;
                    for l in lines {
                        c = c.line(l);
                    }
                    c
                });
                code
            }
        }
    }

    pub fn collect_imports(&self, sink: &mut Vec<String>) {
        match &self.body {
            AccessorBody::Legacy(c) => c.collect_imports(sink),
            AccessorBody::Expr(a) => sink.extend(
                super::expr::free_names(&a.tree)
                    .into_iter()
                    .filter(|n| n.is_qualified())
                    .map(|n| n.as_str().to_string()),
            ),
            AccessorBody::Block(a) => {
                let wrapper = KtExpr::Lambda {
                    params: Vec::new(),
                    body: a.tree.clone(),
                };
                sink.extend(
                    super::expr::free_names(&wrapper)
                        .into_iter()
                        .filter(|n| n.is_qualified())
                        .map(|n| n.as_str().to_string()),
                );
            }
        }
    }
}

impl AnnotationSlot {
    /// Render without the leading `@`.
    pub fn render(&self, imports: &mut ImportSet) -> String {
        match self {
            AnnotationSlot::Legacy(t) => t.as_str().to_string(),
            AnnotationSlot::Ast(a) => {
                let arena = ExprArena::new();
                let name = if a.name.is_qualified() {
                    imports.short(a.name.as_str())
                } else {
                    a.name.as_str().to_string()
                };
                if a.args.is_empty() {
                    name
                } else {
                    let args: Vec<String> = a
                        .args
                        .iter()
                        .map(|e| render_expr(&arena, e, imports))
                        .collect();
                    format!("{name}({})", args.join(", "))
                }
            }
        }
    }

    /// Whether this renders exactly `text` — the predicate the renderer's
    /// "already annotated" checks need without reaching into the arm.
    pub fn renders_as(&self, text: &str) -> bool {
        match self {
            AnnotationSlot::Legacy(t) => t.as_str() == text,
            AnnotationSlot::Ast(a) => a.args.is_empty() && a.name.as_str() == text,
        }
    }
}
