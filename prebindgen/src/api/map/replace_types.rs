//! Filter items in a sequence of (syn::Item, SourceLocation) by replacing specific types
//! Mostly useful for explicitly appending full paths to types, e.g. 
//! replacing `Option` with `std::option::Option` or `MaybeUninit` with `std::mem::MaybeUninit`

use crate::api::record::SourceLocation;
use std::collections::HashMap;
use syn::{Item, Type, TypePath, visit_mut::VisitMut};

pub struct Builder {
    type_replacements: HashMap<String, String>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            type_replacements: HashMap::new(),
        }
    }

    /// Add a type replacement mapping.
    pub fn replace_type<S: Into<String>>(mut self, from: S, to: S) -> Self {
        self.type_replacements.insert(from.into(), to.into());
        self
    }

    /// Build the ReplaceTypes instance
    pub fn build(self) -> ReplaceTypes {
        ReplaceTypes { builder: self }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// Replace specific types in the items.
pub struct ReplaceTypes {
    builder: Builder,
}

impl ReplaceTypes {
    /// Create a builder for creating a replace types instance
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Call method to use with `map` function
    pub fn call(&self, item: (Item, SourceLocation)) -> (Item, SourceLocation) {
        let (mut item, source_location) = item;
        let mut visitor = TypeReplacer {
            replacements: &self.builder.type_replacements,
        };
        visitor.visit_item_mut(&mut item);
        (item, source_location)
    }

    /// Convert to closure
    pub fn into_closure(self) -> impl FnMut((Item, SourceLocation)) -> (Item, SourceLocation) {
        move |item| self.call(item)
    }
}

struct TypeReplacer<'a> {
    replacements: &'a HashMap<String, String>,
}

impl<'a> VisitMut for TypeReplacer<'a> {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        let Type::Path(TypePath { path, .. }) = ty else {
            return syn::visit_mut::visit_type_mut(self, ty);
        };
        
        // Convert current path to string for matching
        let current_path = path.segments.iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        
        let Some(replacement) = self.replacements.get(&current_path) else {
            return syn::visit_mut::visit_type_mut(self, ty);
        };
        
        let Ok(Type::Path(TypePath { path: new_path, .. })) = syn::parse_str::<Type>(replacement) else {
            return syn::visit_mut::visit_type_mut(self, ty);
        };
        
        // Preserve and process generic arguments from the last segment
        let mut original_args = path.segments.last().unwrap().arguments.clone();
        if let syn::PathArguments::AngleBracketed(ref mut args) = original_args {
            for arg in &mut args.args {
                if let syn::GenericArgument::Type(inner_ty) = arg {
                    self.visit_type_mut(inner_ty);
                }
            }
        }
        
        // Apply replacement with preserved arguments
        path.segments = new_path.segments;
        if let Some(last_segment) = path.segments.last_mut() {
            last_segment.arguments = original_args;
        }
    }
}