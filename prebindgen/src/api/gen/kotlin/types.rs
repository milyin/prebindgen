//! Kotlin type references ([`KtType`]) and the per-file import collector
//! ([`ImportSet`]).
//!
//! A type renders against an `ImportSet`: a dotted FQN registers an import
//! and renders as its short name; a dot-free name renders bare (builtins,
//! type variables). If two distinct FQNs share a simple name within one
//! file, the first registered owns the import and any later one renders
//! fully qualified — the file always compiles.

use std::collections::BTreeMap;

/// A Kotlin type reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KtType {
    /// A named type: builtin (`Int`), type variable (`R`), or class FQN
    /// (`io.zenoh.jni.ZKeyExpr`), optionally generic (`List<T>`).
    Named {
        fqn: String,
        args: Vec<KtType>,
        nullable: bool,
    },
    /// A function type with **named** parameters:
    /// `(je: String?, message: String) -> Unit`.
    Function {
        params: Vec<(String, KtType)>,
        ret: Box<KtType>,
        nullable: bool,
    },
}

impl KtType {
    pub const UNIT: &'static str = "Unit";

    /// A named type (builtin, type variable, or class FQN).
    pub fn cls(fqn: impl Into<String>) -> Self {
        KtType::Named {
            fqn: fqn.into(),
            args: vec![],
            nullable: false,
        }
    }

    /// A generic named type, e.g. `generic("List", [cls("io.x.Y")])`.
    pub fn generic(fqn: impl Into<String>, args: impl IntoIterator<Item = KtType>) -> Self {
        KtType::Named {
            fqn: fqn.into(),
            args: args.into_iter().collect(),
            nullable: false,
        }
    }

    /// A function type with named parameters.
    pub fn lambda(params: impl IntoIterator<Item = (String, KtType)>, ret: KtType) -> Self {
        KtType::Function {
            params: params.into_iter().collect(),
            ret: Box::new(ret),
            nullable: false,
        }
    }

    pub fn unit() -> Self {
        Self::cls("Unit")
    }
    pub fn int() -> Self {
        Self::cls("Int")
    }
    pub fn long() -> Self {
        Self::cls("Long")
    }
    pub fn boolean() -> Self {
        Self::cls("Boolean")
    }
    pub fn string() -> Self {
        Self::cls("String")
    }
    pub fn byte_array() -> Self {
        Self::cls("ByteArray")
    }
    pub fn any() -> Self {
        Self::cls("Any")
    }
    /// A bare type variable (`R`, `A`) — renders verbatim, never imported.
    pub fn var_(name: impl Into<String>) -> Self {
        Self::cls(name)
    }
    /// Shorthand for the ubiquitous `R` type variable.
    pub fn var_r() -> Self {
        Self::cls("R")
    }

    /// This type made nullable (`T?`).
    pub fn nullable(mut self) -> Self {
        match &mut self {
            KtType::Named { nullable, .. } | KtType::Function { nullable, .. } => *nullable = true,
        }
        self
    }

    /// Whether this type is nullable (`T?` / `((…) -> …)?`).
    pub fn is_nullable(&self) -> bool {
        match self {
            KtType::Named { nullable, .. } | KtType::Function { nullable, .. } => *nullable,
        }
    }

    /// The (possibly dotted) name of a non-generic named type — `None` for
    /// function types and generics. This is the FQN-or-short-name string a
    /// leaf was constructed from.
    pub fn leaf_name(&self) -> Option<&str> {
        match self {
            KtType::Named { fqn, args, .. } if args.is_empty() => Some(fqn),
            _ => None,
        }
    }

    /// The simple (dot-free) name of a named type: last FQN segment, generic
    /// arguments ignored. `None` for function types.
    pub fn simple_name(&self) -> Option<&str> {
        match self {
            KtType::Named { fqn, .. } => Some(fqn.rsplit('.').next().unwrap_or(fqn)),
            KtType::Function { .. } => None,
        }
    }

    /// Render to Kotlin source, registering imports in `imports`.
    pub fn render(&self, imports: &mut ImportSet) -> String {
        match self {
            KtType::Named {
                fqn,
                args,
                nullable,
            } => {
                let mut s = imports.short(fqn);
                if !args.is_empty() {
                    s.push('<');
                    let rendered: Vec<String> = args.iter().map(|a| a.render(imports)).collect();
                    s.push_str(&rendered.join(", "));
                    s.push('>');
                }
                if *nullable {
                    s.push('?');
                }
                s
            }
            KtType::Function {
                params,
                ret,
                nullable,
            } => {
                let ps: Vec<String> = params
                    .iter()
                    .map(|(n, t)| {
                        if n.is_empty() {
                            t.render(imports)
                        } else {
                            format!("{n}: {}", t.render(imports))
                        }
                    })
                    .collect();
                let core = format!("({}) -> {}", ps.join(", "), ret.render(imports));
                if *nullable {
                    format!("({core})?")
                } else {
                    core
                }
            }
        }
    }
}

/// Renders the type with names exactly as constructed (FQNs stay fully
/// qualified — no import shortening). For diagnostics and any context
/// without an [`ImportSet`].
impl std::fmt::Display for KtType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KtType::Named {
                fqn,
                args,
                nullable,
            } => {
                f.write_str(fqn)?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{a}")?;
                    }
                    write!(f, ">")?;
                }
                if *nullable {
                    write!(f, "?")?;
                }
                Ok(())
            }
            KtType::Function {
                params,
                ret,
                nullable,
            } => {
                if *nullable {
                    write!(f, "(")?;
                }
                write!(f, "(")?;
                for (i, (n, t)) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if !n.is_empty() {
                        write!(f, "{n}: ")?;
                    }
                    write!(f, "{t}")?;
                }
                write!(f, ") -> {ret}")?;
                if *nullable {
                    write!(f, ")?")?;
                }
                Ok(())
            }
        }
    }
}

/// Per-file import collector. Maps simple name → owning FQN; first
/// registration wins, later distinct FQNs with the same simple name render
/// fully qualified.
#[derive(Default, Debug)]
pub struct ImportSet {
    /// The package of the file being rendered — same-package FQNs need no
    /// import and render short.
    package: String,
    /// simple name → FQN that owns it in this file.
    by_simple: BTreeMap<String, String>,
    /// Top-level FUNCTION imports (lowercase simple names): Kotlin allows
    /// several with the same simple name (overload resolution), so they
    /// bypass the simple-name ownership map — every registered FQN gets its
    /// import line.
    fn_imports: std::collections::BTreeSet<String>,
}

impl ImportSet {
    pub fn new(package: impl Into<String>) -> Self {
        Self {
            package: package.into(),
            by_simple: BTreeMap::new(),
            fn_imports: Default::default(),
        }
    }

    /// Resolve a (possibly dotted) name to the text the use site should
    /// emit, registering an import when needed. Only a dotted **identifier
    /// path** (`io.zenoh.jni.ZKeyExpr`) is treated as an FQN; any other
    /// shape (dot-free names, verbatim function-type strings) renders
    /// unchanged.
    pub fn short(&mut self, name: &str) -> String {
        let is_fqn_path = name.contains('.')
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
        if !is_fqn_path {
            return name.to_string();
        }
        let Some((_pkg, simple)) = name.rsplit_once('.') else {
            return name.to_string();
        };
        // Lowercase simple name = a top-level function import (extension
        // adapters, helpers): same-named overloads from different packages
        // may coexist — register them all, always render short.
        if simple.chars().next().is_some_and(|c| c.is_lowercase()) {
            self.fn_imports.insert(name.to_string());
            return simple.to_string();
        }
        match self.by_simple.get(simple) {
            Some(owner) if owner == name => simple.to_string(),
            Some(_) => name.to_string(), // collision: render fully qualified
            None => {
                self.by_simple.insert(simple.to_string(), name.to_string());
                simple.to_string()
            }
        }
    }

    /// Register an FQN referenced only inside raw code text (so the import
    /// line is emitted even though no `KtType` renders it).
    pub fn register(&mut self, fqn: &str) {
        let _ = self.short(fqn);
    }

    /// The sorted import lines for the file: every registered FQN except
    /// same-package ones.
    pub fn import_lines(&self) -> Vec<String> {
        self.by_simple
            .values()
            .chain(self.fn_imports.iter())
            .filter(|fqn| {
                fqn.rsplit_once('.')
                    .map(|(pkg, _)| pkg != self.package)
                    .unwrap_or(false)
            })
            .map(|fqn| format!("import {fqn}"))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
