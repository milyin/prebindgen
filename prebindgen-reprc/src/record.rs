//! Physical records for grouping expanded boundary fields.
use crate::Result;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeSet;
use syn::ext::IdentExt;
use syn::{Ident, ItemStruct, Type};

/// A named `#[repr(C)]` record of adapter-selected boundary fields.
///
/// Use this after expansion to group an element's scalar fields, handles or
/// buffer descriptors into one physical row. Field order is preserved. Language
/// adapters still select field representations, generate foreign declarations
/// and manage allocation, ownership and delivery.
///
/// This is a wire record, not a layout declaration for the original source
/// type. `repr(C)` does not make arbitrary field types C-compatible, remove
/// padding, initialize padding or permit interpreting arbitrary bytes as Rust
/// values. The adapter remains responsible for field validity and alignment.
/// No ownership conversion, `Copy` implementation or destructor is generated.
/// The adapter also chooses foreign field names; Rust-valid names need not be
/// valid identifiers in its destination language.
///
/// ```
/// use prebindgen_reprc::Record;
/// use quote::quote;
/// use syn::parse_quote;
///
/// let row = Record::new(parse_quote!(Row), [
///     (parse_quote!(sequence), parse_quote!(u64)),
///     (parse_quote!(priority), parse_quote!(u32)),
/// ])?;
/// let definition = row.definition();
/// let layout = row.layout();
/// // Compile these together in the generated target crate. The expressions
/// // use that target's layout, even when the generator runs on another host.
/// let generated = quote! {
///     #definition
///     const ROW_LAYOUT: [usize; 4] = #layout;
/// };
/// # let _ = generated;
/// # Ok::<(), String>(())
/// ```
#[derive(Clone, Debug)]
pub struct Record {
    definition: ItemStruct,
}

impl Record {
    /// Group the supplied carrier types into public named fields, in order.
    ///
    /// Rejects empty records, duplicate fields (including raw identifier aliases)
    /// and invalid Rust syntax. A language that needs a nonempty carrier for a
    /// zero-field expansion must supply its own explicit dummy field.
    /// Records have no generic or lifetime parameters. Field types must be
    /// valid in the generated definition's scope; references needing a record
    /// lifetime parameter must instead use an adapter-selected carrier.
    pub fn new(name: Ident, fields: impl IntoIterator<Item = (Ident, Type)>) -> Result<Self> {
        if name == "_" || syn::parse_str::<Ident>(&name.to_string()).is_err() {
            return Err(format!("invalid C representation record name `{name}`"));
        }
        let mut names = BTreeSet::new();
        let mut declarations = Vec::new();
        for (field, ty) in fields {
            if field == "_" || syn::parse_str::<Ident>(&field.to_string()).is_err() {
                return Err(format!("invalid C representation field `{field}`"));
            }
            let key = field.unraw().to_string();
            if !names.insert(key) {
                return Err(format!("duplicate C representation field `{field}`"));
            }
            declarations.push(quote!(pub #field: #ty));
        }
        if declarations.is_empty() {
            return Err("C representation records require at least one field".into());
        }
        let definition = syn::parse2(quote! {
            #[repr(C)] pub struct #name { #(#declarations),* }
        })
        .map_err(|e| format!("invalid C representation record: {e}"))?;
        Ok(Self { definition })
    }

    /// Generate the public Rust record, without imposing ownership or derives.
    ///
    /// An adapter may append attributes such as `#[derive(Clone, Copy)]` to the
    /// returned syntax tree when its chosen fields support them. Such changes
    /// must preserve the layout described by this record.
    pub fn definition(&self) -> ItemStruct {
        self.definition.clone()
    }

    /// Ordered field identifiers and their adapter-selected Rust carrier types.
    pub fn fields(&self) -> impl ExactSizeIterator<Item = (&Ident, &Type)> {
        self.definition
            .fields
            .iter()
            .map(|field| (field.ident.as_ref().expect("named record field"), &field.ty))
    }

    /// Generate `[size, alignment, field_0_offset, ...]` as a Rust expression.
    ///
    /// Offsets follow [`Self::fields`] order and include any compiler-inserted
    /// padding. Evaluate this beside the unmodified definition in the generated
    /// target crate, for example in a constant or an adapter's layout query.
    /// There are no host measurements or runtime field inspections. Aliases and
    /// nested carrier types are resolved by the target compiler.
    ///
    /// The resulting numbers may differ across targets. A foreign artifact that
    /// serves several platforms must query its loaded native library for these
    /// values, for example once during initialization; it must not embed the
    /// build-script host's offsets.
    pub fn layout(&self) -> TokenStream {
        let name = &self.definition.ident;
        let offsets = self
            .fields()
            .map(|(field, _)| quote!(::core::mem::offset_of!(#name, #field)));
        quote!([
            ::core::mem::size_of::<#name>(),
            ::core::mem::align_of::<#name>(),
            #(#offsets),*
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process::Command};
    use syn::parse_quote;

    #[test]
    fn rejects_empty_duplicate_and_invalid_fields() {
        assert!(Record::new(parse_quote!(Row), []).is_err());
        let duplicate = Record::new(
            parse_quote!(Row),
            [
                (parse_quote!(value), parse_quote!(u8)),
                (parse_quote!(r#value), parse_quote!(u64)),
            ],
        );
        assert!(duplicate.unwrap_err().contains("duplicate"));
        let invalid = Ident::new("_", proc_macro2::Span::call_site());
        assert!(Record::new(parse_quote!(Row), [(invalid, parse_quote!(u8))]).is_err());
    }

    #[test]
    fn layout_is_target_compiled_and_preserves_declared_order() {
        let record = Record::new(
            parse_quote!(Row),
            [
                (parse_quote!(second), parse_quote!(CarrierAlias)),
                (parse_quote!(first), parse_quote!(*const u8)),
            ],
        )
        .unwrap();
        assert_eq!(
            record
                .fields()
                .map(|(n, _)| n.to_string())
                .collect::<Vec<_>>(),
            ["second", "first"]
        );
        assert_eq!(record.definition().attrs.len(), 1); // repr(C), no implicit derives.
        let layout = record.layout().to_string();
        assert!(layout.contains("size_of :: < Row >"));
        assert!(layout.contains("align_of :: < Row >"));
        assert!(layout.find("Row , second").unwrap() < layout.find("Row , first").unwrap());
    }

    #[test]
    fn generated_target_layout_matches_c_with_padding_nested_fields_and_pointers() {
        let nested = Record::new(
            parse_quote!(Inner),
            [
                (parse_quote!(flag), parse_quote!(u8)),
                (parse_quote!(number), parse_quote!(u32)),
            ],
        )
        .unwrap();
        let row = Record::new(
            parse_quote!(Row),
            [
                (parse_quote!(flag), parse_quote!(u8)),
                (parse_quote!(r#type), parse_quote!(u64)),
                (parse_quote!(length), parse_quote!(usize)),
                (parse_quote!(data), parse_quote!(*const u8)),
                (parse_quote!(nested), parse_quote!(Inner)),
                (parse_quote!(tail), parse_quote!(u8)),
            ],
        )
        .unwrap();
        let inner_definition = nested.definition();
        let row_definition = row.definition();
        let inner_layout = nested.layout();
        let row_layout = row.layout();
        let generated = quote! {
            #inner_definition
            #row_definition
            const INNER_LAYOUT: [usize; 4] = #inner_layout;
            const ROW_LAYOUT: [usize; 8] = #row_layout;
            // Have rustc also check that these chosen carriers are FFI-safe.
            pub extern "C" fn check_carriers(_: Row) {}
            fn main() {
                for value in INNER_LAYOUT.into_iter().chain(ROW_LAYOUT) {
                    println!("{}", value);
                }
            }
        };
        let dir = tempfile::tempdir().unwrap();
        let rust = dir.path().join("layout.rs");
        let rust_exe = dir.path().join("rust-layout");
        fs::write(&rust, generated.to_string()).unwrap();
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let compiled = Command::new(rustc)
            .args(["--edition=2021", "-Dwarnings"])
            .arg(&rust)
            .arg("-o")
            .arg(&rust_exe)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );

        let c = dir.path().join("layout.c");
        let c_exe = dir.path().join("c-layout");
        fs::write(
            &c,
            r#"
            #include <stdint.h>
            #include <stddef.h>
            #include <stdio.h>
            typedef struct { uint8_t flag; uint32_t number; } Inner;
            typedef struct { uint8_t flag; uint64_t type; uintptr_t length;
                const uint8_t *data; Inner nested; uint8_t tail; } Row;
            int main(void) {
                size_t layout[] = { sizeof(Inner), _Alignof(Inner),
                    offsetof(Inner, flag), offsetof(Inner, number),
                    sizeof(Row), _Alignof(Row), offsetof(Row, flag),
                    offsetof(Row, type), offsetof(Row, length), offsetof(Row, data),
                    offsetof(Row, nested), offsetof(Row, tail) };
                for (size_t i = 0; i < sizeof(layout)/sizeof(layout[0]); ++i)
                    printf("%zu\n", layout[i]);
                return 0;
            }
        "#,
        )
        .unwrap();
        let cc = std::env::var_os("CC").unwrap_or_else(|| "cc".into());
        let compiled = Command::new(cc)
            .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
            .arg(&c)
            .arg("-o")
            .arg(&c_exe)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let rust_output = Command::new(rust_exe).output().unwrap();
        let c_output = Command::new(c_exe).output().unwrap();
        assert!(rust_output.status.success());
        assert!(c_output.status.success());
        assert_eq!(rust_output.stdout, c_output.stdout);
    }
}
