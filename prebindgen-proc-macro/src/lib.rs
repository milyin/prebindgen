use prebindgen::{Record, RecordKind};
use proc_macro::TokenStream;
use quote::quote;
use syn::LitStr;
use std::fs::{OpenOptions, metadata};
use std::io::Write;
use std::path::Path;
use syn::{DeriveInput, ItemFn};

/// Get the full path to `<name>.json` generated in OUT_DIR.
fn get_prebindgen_json_path(name: &str) -> std::path::PathBuf {
    let out_dir = std::env::var("OUT_DIR")
        .expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
    std::path::Path::new(&out_dir).join(format!("{}.json", name))
}

/// Attribute macro that copies the annotated item into `<group>.json` in OUT_DIR.
/// Requires a string literal group name: `#[prebindgen("group_name")]`.
#[proc_macro_attribute]
pub fn prebindgen(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    // Parse required JSON group name literal
    let group_lit = syn::parse::<LitStr>(args)
        .expect("`#[prebindgen]` requires a string literal group name");
    let group = group_lit.value();
    // Get the full path to the JSON file
    let file_path = get_prebindgen_json_path(&group);
    let dest_path = Path::new(&file_path);

    // Try to parse as different item types
    let (kind, name, content) = if let Ok(parsed) = syn::parse::<DeriveInput>(input.clone()) {
        // Handle struct, enum, union
        let kind = match &parsed.data {
            syn::Data::Struct(_) => RecordKind::Struct,
            syn::Data::Enum(_) => RecordKind::Enum,
            syn::Data::Union(_) => RecordKind::Union,
        };
        let tokens = quote! { #parsed };
        (kind, parsed.ident.to_string(), tokens.to_string())
    } else if let Ok(parsed) = syn::parse::<ItemFn>(input.clone()) {
        // Handle function
        // For functions, we want to store only the signature without the body
        let mut fn_sig = parsed.clone();
        fn_sig.block = syn::parse_quote! {{ /* placeholder */ }};
        let tokens = quote! { #fn_sig };
        (
            RecordKind::Function,
            parsed.sig.ident.to_string(),
            tokens.to_string(),
        )
    } else {
        // If we can't parse it, return the original input and skip processing
        return input_clone;
    };

    // Create the new record
    let new_record = Record::new(kind, name, content);

    // Convert record to JSON and append to file
    if let Ok(json_content) = serde_json::to_string(&new_record) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(dest_path) {
            // Check if file is empty (just created or was deleted)
            let is_empty = metadata(dest_path).map(|m| m.len() == 0).unwrap_or(true);

            if is_empty {
                // Write opening bracket for JSON array
                let _ = write!(file, "[{},", json_content);
            } else {
                // Just append the record with comma
                let _ = write!(file, "{},", json_content);
            }
            let _ = file.flush();
        }
    }

    // Return the original input unchanged
    input_clone
}
