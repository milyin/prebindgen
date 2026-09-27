//! The declaration surface a build script drives.

use prebindgen::SourceLocation;
use prebindgen_tools::{flat::Flat, Conversion};

use crate::{plan::Plan, write, Cbindgen, Error};

/// A name mangler over one component.
pub(crate) type Mangle1 = Box<dyn Fn(&str) -> String>;
/// A name mangler over a callback's argument bases.
pub(crate) type MangleN = Box<dyn Fn(&[String]) -> String>;

/// How a declared type crosses.
#[derive(Clone, Debug)]
pub(crate) enum TypeKind {
    /// A `Box`-owned handle behind an opaque pointer.
    Opaque,
    /// The `E` of fallible functions, handed to C as a message.
    OpaqueError { message: syn::Ident },
    /// A fieldless enum, mirrored as a C enum.
    Enum,
    /// A data-carrying enum, mirrored as a tag + union.
    Union,
    /// A struct converted field by field into a mirror.
    Data,
    /// A `#[repr(C)]` struct reinterpreted in place.
    ReprC { assume_valid: bool },
}

#[derive(Clone, Debug)]
pub(crate) struct TypeDecl {
    pub ty: syn::Type,
    pub kind: TypeKind,
    pub base: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct FunctionDecl {
    pub name: syn::Ident,
    pub panic: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct CallbackDecl {
    pub ty: syn::Type,
    pub base: Option<String>,
}

/// What the last declaration was, so the modifiers (`.panic()`,
/// `.base_name()`, `.assume_c_field_validity()`) know what they modify.
#[derive(Clone, Copy)]
enum Last {
    None,
    Type(usize),
    Function(usize),
    Callback(usize),
}

/// Configures the C adapter. Items are **opt-in**: nothing is generated for
/// an item that is not declared.
pub struct CbindgenBuilder {
    pub(crate) items: Vec<(syn::Item, SourceLocation)>,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) free_fn: Option<String>,
    pub(crate) mangle_type: Option<Mangle1>,
    pub(crate) mangle_destructor: Option<Mangle1>,
    pub(crate) mangle_callback: Option<MangleN>,
    pub(crate) mangle_function: Option<Mangle1>,
    pub(crate) types: Vec<TypeDecl>,
    pub(crate) functions: Vec<FunctionDecl>,
    pub(crate) callbacks: Vec<CallbackDecl>,
    pub(crate) conversions: Vec<Conversion>,
    pub(crate) ignored: Vec<syn::Ident>,
    last: Last,
}

impl Default for CbindgenBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CbindgenBuilder {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            source_module: None,
            free_fn: None,
            mangle_type: None,
            mangle_destructor: None,
            mangle_callback: None,
            mangle_function: None,
            types: Vec::new(),
            functions: Vec::new(),
            callbacks: Vec::new(),
            conversions: Vec::new(),
            ignored: Vec::new(),
            last: Last::None,
        }
    }

    /// Every item captured in `dir` (a source crate's `PREBINDGEN_OUT_DIR`).
    pub fn source<P: AsRef<std::path::Path>>(mut self, dir: P) -> Self {
        self.items.extend(prebindgen::Source::new(dir).items_all());
        self
    }

    /// The items of a dependency this crate renames in `Cargo.toml`.
    pub fn source_named<P: AsRef<std::path::Path>>(mut self, dir: P, crate_name: &str) -> Self {
        let source = prebindgen::Source::builder(dir)
            .crate_name(crate_name)
            .build();
        self.items.extend(source.items_all());
        self
    }

    /// Items from any stream.
    pub fn items<I: IntoIterator<Item = (syn::Item, SourceLocation)>>(mut self, items: I) -> Self {
        self.items.extend(items);
        self
    }

    /// The module generated code names an item by when its capture carries
    /// no crate name. Captured items name their own crate.
    pub fn source_module(mut self, p: syn::Path) -> Self {
        self.source_module = Some(p);
        self
    }

    /// The exported C function that frees memory this layer hands out — a
    /// returned `char *`, a returned array.
    pub fn free_memory_function(mut self, name: impl Into<String>) -> Self {
        self.free_fn = Some(name.into());
        self
    }

    /// C type name from a type's base (the snake_case Rust name).
    pub fn mangle_type_name(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.mangle_type = Some(Box::new(f));
        self
    }

    /// Destructor name from a type's base.
    pub fn mangle_destructor(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.mangle_destructor = Some(Box::new(f));
        self
    }

    /// Closure struct name from the bases of a callback's arguments (or its
    /// declared `.base_name`).
    pub fn mangle_callback(mut self, f: impl Fn(&[String]) -> String + 'static) -> Self {
        self.mangle_callback = Some(Box::new(f));
        self
    }

    /// Exported symbol from a Rust function name.
    pub fn mangle_function(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.mangle_function = Some(Box::new(f));
        self
    }

    fn push_type(mut self, ty: syn::Type, kind: TypeKind) -> Self {
        self.types.push(TypeDecl {
            ty,
            kind,
            base: None,
        });
        self.last = Last::Type(self.types.len() - 1);
        self
    }

    /// A handle: C holds `T *` from `Box::into_raw` and releases it with the
    /// generated `<base>_drop`.
    pub fn opaque_ptr(self, ty: syn::Type) -> Self {
        self.push_type(ty, TypeKind::Opaque)
    }

    /// The error type of fallible functions: C receives it as a `char *`
    /// message produced by `message` (a `#[prebindgen] fn(&E) -> String`).
    pub fn opaque_error(self, ty: syn::Type, message: syn::Ident) -> Self {
        self.push_type(ty, TypeKind::OpaqueError { message })
    }

    /// A fieldless enum, as a C enum. Values arriving from C are validated.
    pub fn enum_type(self, ty: syn::Type) -> Self {
        self.push_type(ty, TypeKind::Enum)
    }

    /// A data-carrying enum, as a tagged union with a `<base>_drop` for the
    /// active arm's owned memory.
    pub fn tagged_union(self, ty: syn::Type) -> Self {
        self.push_type(ty, TypeKind::Union)
    }

    /// A struct crossing by value, field by field.
    pub fn data_struct(self, ty: syn::Type) -> Self {
        self.push_type(ty, TypeKind::Data)
    }

    /// A `#[repr(C)]` struct crossing by reinterpretation of its memory.
    pub fn repr_c_struct(self, ty: syn::Type) -> Self {
        self.push_type(
            ty,
            TypeKind::ReprC {
                assume_valid: false,
            },
        )
    }

    /// On the last `repr_c_struct`: accept restricted-validity fields
    /// (`bool`) the reinterpretation cannot check.
    pub fn assume_c_field_validity(mut self) -> Self {
        if let Last::Type(i) = self.last {
            if let TypeKind::ReprC { assume_valid } = &mut self.types[i].kind {
                *assume_valid = true;
            }
        }
        self
    }

    /// A source type crossing as another type through declared conversions.
    pub fn convert(mut self, conversion: Conversion) -> Self {
        self.conversions.push(conversion);
        self.last = Last::None;
        self
    }

    /// A callback signature (`impl Fn(..) + Send + Sync + 'static`), as a
    /// closure struct `{ context, call, drop }`.
    pub fn callback(mut self, ty: syn::Type) -> Self {
        self.callbacks.push(CallbackDecl { ty, base: None });
        self.last = Last::Callback(self.callbacks.len() - 1);
        self
    }

    /// On the last type or callback: the base its C names are mangled from.
    pub fn base_name(mut self, base: impl Into<String>) -> Self {
        let base = Some(base.into());
        match self.last {
            Last::Type(i) => self.types[i].base = base,
            Last::Callback(i) => self.callbacks[i].base = base,
            _ => {}
        }
        self
    }

    /// Export a function.
    pub fn function(mut self, name: syn::Ident) -> Self {
        self.functions.push(FunctionDecl { name, panic: false });
        self.last = Last::Function(self.functions.len() - 1);
        self
    }

    /// On the last function: abort instead of reporting when an input fails
    /// to convert (a null handle, a bad discriminant) — for a function with
    /// no `Result` to report through.
    pub fn panic(mut self) -> Self {
        if let Last::Function(i) = self.last {
            self.functions[i].panic = true;
        }
        self
    }

    /// Acknowledge a function that is deliberately not exported.
    pub fn ignore_function(mut self, name: syn::Ident) -> Self {
        self.ignored.push(name);
        self.last = Last::None;
        self
    }

    /// Parse the sources and generate the binding.
    pub fn build(self) -> Result<Cbindgen, Error> {
        let flat = Flat::builder()
            .items(self.items.clone())
            .build()
            .map_err(|e| Error(e.to_string()))?;
        let plan = Plan::new(&self, &flat)?;
        Ok(Cbindgen {
            file: write::write(&plan)?,
        })
    }
}
