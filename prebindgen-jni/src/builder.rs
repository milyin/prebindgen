//! The builder a build script drives.

use prebindgen::SourceLocation;
use prebindgen_flat::Flat;

use crate::{
    decl::{ConvertDecl, ExpandDecl, IgnoreDecl, PackageDecl},
    plan::Plan,
    write, Error, Generation,
};

pub(crate) type NameHook = Box<dyn Fn(&str, &str) -> String>;
pub(crate) type MethodHook = Box<dyn Fn(&str, &str, &str) -> String>;
pub(crate) type HarnessHook = Box<dyn Fn(&str) -> String>;

/// Generator-wide settings (`set_*`), plus the collected declarations.
pub struct JniGenBuilder {
    pub(crate) items: Vec<(syn::Item, SourceLocation)>,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) package_prefix: String,
    pub(crate) native_init: Option<String>,
    pub(crate) harness_hook: Option<HarnessHook>,
    pub(crate) fun_hook: Option<NameHook>,
    pub(crate) ptr_hook: Option<NameHook>,
    pub(crate) data_hook: Option<NameHook>,
    pub(crate) enum_hook: Option<NameHook>,
    pub(crate) method_hook: Option<MethodHook>,
    pub(crate) iface_hook: Option<NameHook>,
    pub(crate) handle_locks: bool,
    pub(crate) packages: Vec<PackageDecl>,
    pub(crate) converts: Vec<ConvertDecl>,
    pub(crate) expands: Vec<ExpandDecl>,
    pub(crate) ignores: Vec<IgnoreDecl>,
}

/// The JNI / Kotlin adapter.
pub struct JniGen;

impl JniGen {
    pub fn builder() -> JniGenBuilder {
        JniGenBuilder::new()
    }
}

impl Default for JniGenBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl JniGenBuilder {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            source_module: None,
            package_prefix: String::new(),
            native_init: None,
            harness_hook: None,
            fun_hook: None,
            ptr_hook: None,
            data_hook: None,
            enum_hook: None,
            method_hook: None,
            iface_hook: None,
            handle_locks: true,
            packages: Vec::new(),
            converts: Vec::new(),
            expands: Vec::new(),
            ignores: Vec::new(),
        }
    }

    /// Every item captured in a source crate's `PREBINDGEN_OUT_DIR`.
    pub fn source<P: AsRef<std::path::Path>>(mut self, dir: P) -> Self {
        self.items.extend(prebindgen::Source::new(dir).items_all());
        self
    }

    /// The items of a dependency renamed in `Cargo.toml` to `crate_name`.
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

    /// The module an item is named by when its capture carries no crate.
    pub fn set_source_module(mut self, module: syn::Path) -> Self {
        self.source_module = Some(module);
        self
    }

    /// The base Kotlin package: `io.example.binding`.
    pub fn set_package_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.package_prefix = prefix.into();
        self
    }

    /// A Kotlin statement the native-method holder runs before its first
    /// call — typically loading the library.
    pub fn set_jni_native_init(mut self, stmt: impl Into<String>) -> Self {
        self.native_init = Some(stmt.into());
        self
    }

    /// The name of the object holding the `external` declarations, from its
    /// default `JNINative`.
    pub fn set_harness_name_mangle(mut self, f: impl Fn(&str) -> String + 'static) -> Self {
        self.harness_hook = Some(Box::new(f));
        self
    }

    /// A package function's Kotlin name: `(package, camelCaseName)`.
    pub fn set_fun_name_mangle(mut self, f: impl Fn(&str, &str) -> String + 'static) -> Self {
        self.fun_hook = Some(Box::new(f));
        self
    }

    /// A handle class's Kotlin name: `(package, RustName)`.
    pub fn set_ptr_class_name_mangle(mut self, f: impl Fn(&str, &str) -> String + 'static) -> Self {
        self.ptr_hook = Some(Box::new(f));
        self
    }

    /// A data or sealed class's Kotlin name: `(package, RustName)`.
    pub fn set_data_class_name_mangle(
        mut self,
        f: impl Fn(&str, &str) -> String + 'static,
    ) -> Self {
        self.data_hook = Some(Box::new(f));
        self
    }

    /// An enum class's Kotlin name: `(package, RustName)`.
    pub fn set_enum_name_mangle(mut self, f: impl Fn(&str, &str) -> String + 'static) -> Self {
        self.enum_hook = Some(Box::new(f));
        self
    }

    /// The name of a class's `.interface()` interface: `(package,
    /// ClassName)`, which must differ from the class name. Default: the
    /// class name plus `Api`. An `.interface_name(..)` overrides it.
    pub fn set_interface_name_mangle(mut self, f: impl Fn(&str, &str) -> String + 'static) -> Self {
        self.iface_hook = Some(Box::new(f));
        self
    }

    /// A member's Kotlin name: `(package, ClassName, camelCaseName)`. Also
    /// names the native-method holder's `external` declarations, with the
    /// holder as the class.
    pub fn set_method_name_mangle(
        mut self,
        f: impl Fn(&str, &str, &str) -> String + 'static,
    ) -> Self {
        self.method_hook = Some(Box::new(f));
        self
    }

    /// Whether wrappers lock the handles they pass (default on): a handle
    /// cannot be closed by another thread while a call uses it.
    pub fn set_emit_handle_locks(mut self, on: bool) -> Self {
        self.handle_locks = on;
        self
    }

    /// Bind classes, functions and constants under one Kotlin package.
    ///
    /// Build the [`PackageDecl`] with [`package!`](crate::package) and add
    /// classes ([`ptr_class!`](crate::ptr_class), [`data_class!`](crate::data_class),
    /// [`enum_class!`](crate::enum_class), [`sealed_class!`](crate::sealed_class)),
    /// functions ([`fun!`](crate::fun)) and constants
    /// ([`constant!`](crate::constant)). A source-defined type a bound function
    /// uses must be declared as a class or a conversion; primitives, strings,
    /// `Option`, `Vec`, arrays, borrows and callbacks need no declaration (see
    /// [How values cross](crate#how-values-cross)).
    /// `examples/covertest-kotlin/build.rs` declares every kind.
    pub fn package(mut self, p: PackageDecl) -> Self {
        self.packages.push(p);
        self
    }

    /// Let a type cross as another type, through declared functions or
    /// `From`/`TryFrom` impls: `convert!(Millis).input(fun!(from_raw)).output(fun!(to_raw))`.
    /// Kotlin sees the representation type; see [`ConvertDecl`].
    pub fn convert(mut self, c: ConvertDecl) -> Self {
        self.converts.push(c);
        self
    }

    /// Set a type's default boundary shape:
    ///
    /// * [`ExpandReturnDecl`](crate::ExpandReturnDecl) (`expand_return!`) — a result of the type is
    ///   delivered as its fields to a caller-supplied builder (a folder for a
    ///   `Vec`, the handler for an `Err`, the arguments of a callback);
    /// * [`ExpandParamDecl`](crate::ExpandParamDecl) (`expand_param!`) — a parameter of the type is
    ///   built by a constructor or passed as a handle, chosen by a selector.
    ///
    /// A function overrides either for itself with
    /// [`FunctionDecl::expand_return`](crate::FunctionDecl::expand_return) / [`FunctionDecl::expand_param`](crate::FunctionDecl::expand_param).
    pub fn expand(mut self, e: impl Into<ExpandDecl>) -> Self {
        self.expands.push(e.into());
        self
    }

    /// Acknowledge items that are deliberately not bound.
    pub fn ignore(mut self, i: impl Into<IgnoreDecl>) -> Self {
        self.ignores.push(i.into());
        self
    }

    /// Parse the sources and generate the binding.
    pub fn build(self) -> Result<Generation, Error> {
        let flat = Flat::builder()
            .items(self.items.clone())
            .build()
            .map_err(|e| Error(e.to_string()))?;
        let plan = Plan::new(&self, &flat)?;
        write::write(&plan)
    }
}
