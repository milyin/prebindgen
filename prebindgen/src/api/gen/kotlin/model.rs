//! Declaration model: [`KtFile`] → [`KtDecl`] (classes, functions,
//! properties, type aliases, raw blocks). Chained builders in the same
//! style as the JniGen config builder. Rendering lives in
//! [`super::render`]; this module is pure data.

use super::{code::Code, types::KtType};

/// Visibility modifier. `Public` renders explicitly (matching the existing
/// generated style); `Default` renders nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Vis {
    #[default]
    Default,
    Public,
    Internal,
    Private,
}

impl Vis {
    pub(crate) fn prefix(self) -> &'static str {
        match self {
            Vis::Default => "",
            Vis::Public => "public ",
            Vis::Internal => "internal ",
            Vis::Private => "private ",
        }
    }
}

/// One Kotlin source file fragment: a package plus top-level declarations.
/// Fragments of the same package are merged by [`super::file::merge_files`].
#[derive(Clone, Debug)]
pub struct KtFile {
    pub package: String,
    pub decls: Vec<KtDecl>,
    /// FQNs referenced only inside raw text the model can't see (e.g. body
    /// strings built with pre-shortened type names). Registered into the
    /// file's import set before any declaration renders.
    pub extra_imports: Vec<String>,
}

impl KtFile {
    pub fn new(package: impl Into<String>) -> Self {
        Self {
            package: package.into(),
            decls: Vec::new(),
            extra_imports: Vec::new(),
        }
    }

    pub fn decl(mut self, d: impl Into<KtDecl>) -> Self {
        self.decls.push(d.into());
        self
    }

    /// Register an FQN referenced only inside raw text.
    pub fn import(mut self, fqn: impl Into<String>) -> Self {
        self.extra_imports.push(fqn.into());
        self
    }

    pub fn imports(mut self, fqns: impl IntoIterator<Item = String>) -> Self {
        self.extra_imports.extend(fqns);
        self
    }
}

/// A top-level (or member-level, for `Raw`) declaration.
#[derive(Clone, Debug)]
pub enum KtDecl {
    Class(KtClass),
    Fun(KtFun),
    FunInterface(KtFunInterface),
    Property(KtProperty),
    TypeAlias {
        vis: Vis,
        name: String,
        target: KtType,
    },
    /// Pre-rendered code at declaration position. `name` is the identity
    /// used for duplicate detection during merge.
    Raw {
        name: String,
        code: Code,
    },
}

impl KtDecl {
    /// Identity for duplicate detection within a merged package.
    pub fn name(&self) -> &str {
        match self {
            KtDecl::Class(c) => &c.name,
            KtDecl::Fun(f) => &f.name,
            KtDecl::FunInterface(i) => &i.name,
            KtDecl::Property(p) => &p.name,
            KtDecl::TypeAlias { name, .. } => name,
            KtDecl::Raw { name, .. } => name,
        }
    }
}

impl From<KtClass> for KtDecl {
    fn from(c: KtClass) -> Self {
        KtDecl::Class(c)
    }
}
impl From<KtFunInterface> for KtDecl {
    fn from(i: KtFunInterface) -> Self {
        KtDecl::FunInterface(i)
    }
}
impl From<KtFun> for KtDecl {
    fn from(f: KtFun) -> Self {
        KtDecl::Fun(f)
    }
}
impl From<KtProperty> for KtDecl {
    fn from(p: KtProperty) -> Self {
        KtDecl::Property(p)
    }
}

/// A `fun interface` (SAM) declaration: exactly one abstract method.
///
/// The method is a [`KtFun`] rendered without a body ([`KtBody::None`]); its
/// JNI-callable JVM name is the method's `name` verbatim — keep the interface
/// and the method `public` and its params free of `@JvmInline` value classes,
/// or Kotlin mangles the JVM method name and native `GetMethodID` fails at
/// runtime.
#[derive(Clone, Debug)]
pub struct KtFunInterface {
    pub vis: Vis,
    pub name: String,
    /// Type parameters with variance as written, e.g. `["out R"]`.
    pub type_params: Vec<String>,
    pub kdoc: Option<String>,
    /// The single abstract method.
    pub method: KtFun,
}

impl KtFunInterface {
    pub fn new(name: impl Into<String>, method: KtFun) -> Self {
        Self {
            vis: Vis::Default,
            name: name.into(),
            type_params: Vec::new(),
            kdoc: None,
            method,
        }
    }
    pub fn vis(mut self, v: Vis) -> Self {
        self.vis = v;
        self
    }
    pub fn type_param(mut self, p: impl Into<String>) -> Self {
        self.type_params.push(p.into());
        self
    }
    pub fn kdoc(mut self, d: impl Into<String>) -> Self {
        self.kdoc = Some(d.into());
        self
    }
}

/// The kind of class-like declaration.
#[derive(Clone, Debug)]
pub enum ClassKind {
    Plain,
    Abstract,
    Data,
    /// Enum class with its entries: `(NAME, ctor-args-text)`.
    Enum(Vec<KtEnumEntry>),
    /// `@JvmInline value class` (the annotation is added by the renderer).
    ValueInline,
    Object,
    /// `companion object` (only valid as [`KtClass::companion`]).
    Companion,
}

#[derive(Clone, Debug)]
pub struct KtEnumEntry {
    pub name: String,
    /// Constructor argument text, e.g. `"0"` → `NAME(0)`.
    pub args: Option<String>,
}

/// Primary-constructor parameter, optionally a `val`/`var` property.
#[derive(Clone, Debug)]
pub struct KtCtorParam {
    pub name: String,
    pub ty: KtType,
    /// `None` = plain ctor param; `Some(false)` = `val`, `Some(true)` = `var`.
    pub prop: Option<bool>,
    pub vis: Vis,
    pub default: Option<String>,
    pub annotations: Vec<String>,
}

impl KtCtorParam {
    pub fn new(name: impl Into<String>, ty: KtType) -> Self {
        Self {
            name: name.into(),
            ty,
            prop: None,
            vis: Vis::Default,
            default: None,
            annotations: Vec::new(),
        }
    }
    pub fn val(mut self) -> Self {
        self.prop = Some(false);
        self
    }
    pub fn var(mut self) -> Self {
        self.prop = Some(true);
        self
    }
    pub fn vis(mut self, v: Vis) -> Self {
        self.vis = v;
        self
    }
    pub fn default(mut self, d: impl Into<String>) -> Self {
        self.default = Some(d.into());
        self
    }
    pub fn annotation(mut self, a: impl Into<String>) -> Self {
        self.annotations.push(a.into());
        self
    }
}

/// A class / object / enum / data / value-class declaration.
#[derive(Clone, Debug)]
pub struct KtClass {
    pub kind: ClassKind,
    pub name: String,
    pub vis: Vis,
    pub annotations: Vec<String>,
    pub kdoc: Option<String>,
    pub ctor_params: Vec<KtCtorParam>,
    /// Supertypes with optional constructor-argument text:
    /// `(NativeHandle, Some("initialPtr"))` → `: NativeHandle(initialPtr)`;
    /// `(AutoCloseable, None)` → `: AutoCloseable`.
    pub supertypes: Vec<(KtType, Option<String>)>,
    pub members: Vec<KtDecl>,
    pub companion: Option<Box<KtClass>>,
}

impl KtClass {
    pub fn new(kind: ClassKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
            vis: Vis::Default,
            annotations: Vec::new(),
            kdoc: None,
            ctor_params: Vec::new(),
            supertypes: Vec::new(),
            members: Vec::new(),
            companion: None,
        }
    }
    pub fn object_(name: impl Into<String>) -> Self {
        Self::new(ClassKind::Object, name)
    }
    pub fn companion_object() -> Self {
        Self::new(ClassKind::Companion, "")
    }

    pub fn vis(mut self, v: Vis) -> Self {
        self.vis = v;
        self
    }
    pub fn annotation(mut self, a: impl Into<String>) -> Self {
        self.annotations.push(a.into());
        self
    }
    pub fn kdoc(mut self, d: impl Into<String>) -> Self {
        self.kdoc = Some(d.into());
        self
    }
    pub fn ctor_param(mut self, p: KtCtorParam) -> Self {
        self.ctor_params.push(p);
        self
    }
    pub fn supertype(mut self, ty: KtType, ctor_args: Option<&str>) -> Self {
        self.supertypes.push((ty, ctor_args.map(str::to_string)));
        self
    }
    pub fn member(mut self, d: impl Into<KtDecl>) -> Self {
        self.members.push(d.into());
        self
    }
    pub fn companion(mut self, c: KtClass) -> Self {
        self.companion = Some(Box::new(c));
        self
    }
}

/// A function body.
#[derive(Clone, Debug, Default)]
pub enum KtBody {
    /// No body (`external` / `abstract` declarations).
    #[default]
    None,
    /// Single-expression body: `= <code>` (one line) or multi-line after `=`.
    Expr(Code),
    /// Block body: `{ … }`.
    Block(Code),
}

/// A function declaration (top-level or member).
#[derive(Clone, Debug)]
pub struct KtFun {
    pub name: String,
    pub vis: Vis,
    /// Modifier keywords in render order, e.g. `external`, `inline`,
    /// `override`, `abstract`, `operator`.
    pub modifiers: Vec<String>,
    pub annotations: Vec<String>,
    pub kdoc: Option<String>,
    /// Generic type-variable names: `["R"]` → `fun <R> …`.
    pub generics: Vec<String>,
    pub params: Vec<KtParam>,
    pub ret: Option<KtType>,
    pub body: KtBody,
}

impl KtFun {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vis: Vis::Default,
            modifiers: Vec::new(),
            annotations: Vec::new(),
            kdoc: None,
            generics: Vec::new(),
            params: Vec::new(),
            ret: None,
            body: KtBody::None,
        }
    }

    pub fn vis(mut self, v: Vis) -> Self {
        self.vis = v;
        self
    }
    pub fn modifier(mut self, m: impl Into<String>) -> Self {
        self.modifiers.push(m.into());
        self
    }
    pub fn annotation(mut self, a: impl Into<String>) -> Self {
        self.annotations.push(a.into());
        self
    }
    pub fn kdoc(mut self, d: impl Into<String>) -> Self {
        self.kdoc = Some(d.into());
        self
    }
    pub fn generic(mut self, g: impl Into<String>) -> Self {
        self.generics.push(g.into());
        self
    }
    pub fn param(mut self, p: KtParam) -> Self {
        self.params.push(p);
        self
    }
    pub fn returns(mut self, ty: KtType) -> Self {
        self.ret = Some(ty);
        self
    }
    pub fn body(mut self, c: Code) -> Self {
        self.body = KtBody::Block(c);
        self
    }
    pub fn expr_body(mut self, c: Code) -> Self {
        self.body = KtBody::Expr(c);
        self
    }
}

/// A function parameter with an optional default-value expression (raw
/// Kotlin text, e.g. a lambda literal).
#[derive(Clone, Debug)]
pub struct KtParam {
    pub name: String,
    pub ty: KtType,
    pub default: Option<String>,
}

impl KtParam {
    pub fn new(name: impl Into<String>, ty: KtType) -> Self {
        Self {
            name: name.into(),
            ty,
            default: None,
        }
    }
    pub fn default(mut self, d: impl Into<String>) -> Self {
        self.default = Some(d.into());
        self
    }
}

/// A property declaration (top-level or member).
#[derive(Clone, Debug)]
pub struct KtProperty {
    pub name: String,
    pub ty: Option<KtType>,
    /// Raw initializer expression text.
    pub initializer: Option<String>,
    pub mutable: bool,
    pub vis: Vis,
    /// Inline annotations rendered before the keyword: `@Volatile internal var …`.
    pub annotations: Vec<String>,
    pub kdoc: Option<String>,
    /// Raw accessor text rendered indented under the property (e.g. a
    /// custom getter `get() = …`).
    pub accessors: Option<Code>,
}

impl KtProperty {
    pub fn val(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ty: None,
            initializer: None,
            mutable: false,
            vis: Vis::Default,
            annotations: Vec::new(),
            kdoc: None,
            accessors: None,
        }
    }
    pub fn var(name: impl Into<String>) -> Self {
        Self {
            mutable: true,
            ..Self::val(name)
        }
    }
    pub fn ty(mut self, t: KtType) -> Self {
        self.ty = Some(t);
        self
    }
    pub fn initializer(mut self, i: impl Into<String>) -> Self {
        self.initializer = Some(i.into());
        self
    }
    pub fn vis(mut self, v: Vis) -> Self {
        self.vis = v;
        self
    }
    pub fn annotation(mut self, a: impl Into<String>) -> Self {
        self.annotations.push(a.into());
        self
    }
    pub fn kdoc(mut self, d: impl Into<String>) -> Self {
        self.kdoc = Some(d.into());
        self
    }
    pub fn accessors(mut self, c: Code) -> Self {
        self.accessors = Some(c);
        self
    }
}
