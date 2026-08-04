//! Questions about the model, answered through the registry that projects it.

use std::collections::HashMap;

use super::{
    view::{default_module_of, origin_module_of},
    *,
};

impl<M> Registry<M> {
    /// The parameter-side fold for each `(function, parameter)` position.
    ///
    /// Inherent rather than on [`Conversions`]: a fold is read when a wrapper's
    /// parameters are emitted, never while a conversion is being built, so no
    /// generic caller needs it.
    pub fn expansion_plans(
        &self,
    ) -> &HashMap<(syn::Ident, syn::Ident), crate::api::core::expand::FoldPlan> {
        &self.expansion_plans
    }

    /// What the binding declared — read by the emitter's gate.
    pub(crate) fn declared(&self) -> &Declared {
        &self.declared
    }

    /// The parsed model this registry projects.
    pub fn flat(&self) -> &crate::api::core::flat::Flat {
        &self.flat
    }

    /// Every **named** item the model holds — functions, structs, either enum
    /// shape, consts — regardless of whether the stream carried an origin stamp.
    ///
    /// Lives here so an adapter that needs "anything the source crate defines"
    /// does not enumerate element kinds itself: a new kind is taught here once
    /// instead of drifting in each adapter. An **alias is deliberately absent**
    /// — see the arm below — and callers are expected to pair this with
    /// `origin_module(..).unwrap_or_else(default_module)`.
    pub fn named_item_idents(&self) -> impl Iterator<Item = &syn::Ident> {
        use crate::api::core::flat::{Element, Type};
        self.flat.elements().filter_map(|e| match e {
            // An `Extern` names a type without declaring a body, and is
            // deliberately absent: its caller decides which names to qualify in
            // generated Rust, and qualifying an alias would move that output.
            Element::Type(Type::Extern(_)) => None,
            Element::Function(_) | Element::Type(_) | Element::Constant(_) => e.name(),
            Element::Guard(_) | Element::Unsupported(_) => None,
        })
    }

    /// Whether the source declares a type under this name — **including an
    /// alias**.
    ///
    /// An alias counts because `#[prebindgen] pub type Handle = ..` *is* a
    /// declaration of that name: it can be declared bare by an adapter (landing
    /// in the no-indexed-body branch below, which is what
    /// `ptr_class(ZKeyExpr<'static>)` relies on), so a diagnostic that says
    /// "no such captured item" would be false.
    pub(super) fn declares_type(&self, ident: &syn::Ident) -> bool {
        self.flat.declared_type(ident).is_some()
    }

    /// The origin crate's **module path** for an item, read off the element's
    /// own [`SourceLocation`] stamp, or `None` when unknown — callers then fall
    /// back to [`Self::default_module`].
    pub fn origin_module(&self, ident: &syn::Ident) -> Option<syn::Path> {
        // Off the element's own location, which covers both populations: a
        // captured item stamped at capture time, and a binding-local fn stamped
        // by `add_local_function`.
        origin_module_of(&self.flat, ident)
    }

    /// The default module for references with no recorded origin: the
    /// first-seen item origin. `None` for an origin-less item-level
    /// registry (adapters then fall back to `crate`). To change a module
    /// name, override it at the source — a stream's origin stamps
    /// (`Source::builder(dir).crate_name("myflat")`) — never here: a
    /// registry-level override could only fix ONE module, which is
    /// incomplete with chained multi-source streams.
    pub fn default_module(&self) -> Option<syn::Path> {
        default_module_of(&self.flat)
    }

    /// Module paths of every ingested source, ingestion order — e.g. for a
    /// glob import that must see all sources' items.
    pub fn all_source_modules(&self) -> Vec<syn::Path> {
        self.flat
            .source_modules()
            .iter()
            .filter_map(|m| syn::parse_str(m).ok())
            .collect()
    }
}
