//! Target-compiled tagged-union carriers, independent of source enum layouts.
use crate::{Record, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type};

/// A C-representation tag followed by a union of converted variant records.
/// Empty variants have no union member. Ownership stays with the adapter.
#[derive(Clone, Debug)]
pub struct TaggedUnion {
    name: Ident,
    arms: Vec<Option<Record>>,
}
impl TaggedUnion {
    /// Build a signed-i32-tagged carrier. Arm indices are zero-based wire tags.
    /// Records must have distinct valid names and C-compatible fields.
    pub fn new(name: Ident, arms: Vec<Option<Record>>) -> Result<Self> {
        if arms.is_empty() || i32::try_from(arms.len()).is_err() {
            return Err("tagged union needs 1..=i32::MAX variants".into());
        }
        let mut names = std::collections::BTreeSet::new();
        for arm in arms.iter().flatten() {
            if arm.definition().ident == name
                || arm.definition().ident == format_ident!("{name}_payload")
                || !names.insert(arm.definition().ident.to_string())
            {
                return Err("duplicate tagged-union arm record".into());
            }
        }
        // Reuse the record validator for the public carrier identifier.
        Record::new(
            name.clone(),
            [(syn::parse_quote!(tag), syn::parse_quote!(i32))],
        )?;
        Ok(Self { name, arms })
    }
    /// The carrier's Rust identifier.
    pub fn name(&self) -> &Ident {
        &self.name
    }
    /// Generated union identifier, when at least one variant needs storage.
    pub fn payload_name(&self) -> Option<Ident> {
        self.arms
            .iter()
            .any(Option::is_some)
            .then(|| format_ident!("{}_payload", self.name))
    }
    /// The source-order arm records; None denotes a zero-storage arm.
    pub fn arms(&self) -> &[Option<Record>] {
        &self.arms
    }
    /// Emit all arm records, the payload union and the carrier structure.
    /// ManuallyDrop permits owning/non-Copy carrier fields without implicit drop.
    pub fn definition(&self) -> TokenStream {
        let name = &self.name;
        if let Some(payload) = self.payload_name() {
            let definitions = self.arms.iter().flatten().map(Record::definition);
            let members = self.arms.iter().enumerate().filter_map(|(index, record)| {
                record.as_ref().map(|r| {
                    let field = format_ident!("a{index}");
                    let ty = r.definition().ident;
                    quote!(pub #field: ::core::mem::ManuallyDrop<#ty>)
                })
            });
            quote!(#(#definitions)* #[repr(C)] pub union #payload {pub __empty: [u8;0],#(#members,)*} #[repr(C)] pub struct #name {pub tag:i32,pub payload:#payload})
        } else {
            quote!(#[repr(C)] pub struct #name {pub tag:i32})
        }
    }
    /// Target-side byte offset of the discriminator.
    pub fn tag_offset(&self) -> TokenStream {
        let n = &self.name;
        quote!(::core::mem::offset_of!(#n,tag))
    }
    /// Target-side byte offset of an active variant member. Union arms start at zero.
    /// Compute component offsets separately to preserve Rust 1.85 compatibility.
    pub fn field_offset(&self, arm: usize, field: &Ident) -> Result<TokenStream> {
        let record = self
            .arms
            .get(arm)
            .and_then(Option::as_ref)
            .ok_or("variant has no payload storage")?;
        if !record.fields().any(|(name, _)| name == field) {
            return Err("unknown tagged-union field".into());
        }
        let root = &self.name;
        let arm = record.definition().ident;
        Ok(quote!(
            ::core::mem::offset_of!(#root,payload) + ::core::mem::offset_of!(#arm,#field)
        ))
    }
    /// Carrier type syntax, for embedding in another generated record.
    pub fn ty(&self) -> Type {
        let name = &self.name;
        syn::parse_quote!(#name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process::Command};
    use syn::parse_quote;
    #[test]
    fn empty_arms_have_no_payload_record() {
        let tag = TaggedUnion::new(parse_quote!(Empty), vec![None, None]).unwrap();
        assert!(tag.payload_name().is_none());
        assert!(!tag.definition().to_string().contains("union"));
        assert!(tag.field_offset(0, &parse_quote!(missing)).is_err());
    }
    #[test]
    fn target_union_layout_matches_c() {
        let arm = Record::new(
            parse_quote!(Numeric),
            [
                (parse_quote!(small), parse_quote!(u8)),
                (parse_quote!(large), parse_quote!(u64)),
            ],
        )
        .unwrap();
        let pointer = Record::new(
            parse_quote!(Pointer),
            [
                (parse_quote!(data), parse_quote!(*const u8)),
                (parse_quote!(length), parse_quote!(usize)),
            ],
        )
        .unwrap();
        let union =
            TaggedUnion::new(parse_quote!(Message), vec![None, Some(arm), Some(pointer)]).unwrap();
        let definition = union.definition();
        let tag = union.tag_offset();
        let small = union.field_offset(1, &parse_quote!(small)).unwrap();
        let large = union.field_offset(1, &parse_quote!(large)).unwrap();
        let data = union.field_offset(2, &parse_quote!(data)).unwrap();
        let source = quote!(#definition fn main(){for n in [::core::mem::size_of::<Message>(),::core::mem::align_of::<Message>(),#tag,#small,#large,#data]{println!("{}",n);}});
        let dir = tempfile::tempdir().unwrap();
        let rust = dir.path().join("layout.rs");
        let rust_exe = dir.path().join("rust-layout");
        fs::write(&rust, source.to_string()).unwrap();
        let output = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args(["--edition=2021", "-Dwarnings"])
            .arg(&rust)
            .arg("-o")
            .arg(&rust_exe)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let c = dir.path().join("layout.c");
        let c_exe = dir.path().join("c-layout");
        fs::write(&c,r#"#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
typedef struct { uint8_t small; uint64_t large; } Numeric;
typedef struct { const uint8_t *data; size_t length; } Pointer;
typedef struct { int32_t tag; union { Numeric numeric; Pointer pointer; } payload; } Message;
int main(void) { size_t layout[]={sizeof(Message),_Alignof(Message),offsetof(Message,tag),offsetof(Message,payload)+offsetof(Numeric,small),offsetof(Message,payload)+offsetof(Numeric,large),offsetof(Message,payload)+offsetof(Pointer,data)}; for(size_t i=0;i<sizeof(layout)/sizeof(layout[0]);++i) printf("%zu\n",layout[i]); return 0; }
"#).unwrap();
        let output = Command::new(std::env::var_os("CC").unwrap_or_else(|| "cc".into()))
            .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
            .arg(&c)
            .arg("-o")
            .arg(&c_exe)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            Command::new(rust_exe).output().unwrap().stdout,
            Command::new(c_exe).output().unwrap().stdout
        );
    }
}
