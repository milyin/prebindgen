use prebindgen::{get_prebindgen_out_dir, trace, Record, RecordKind};
use proc_macro::TokenStream;
use quote::quote;
use syn::LitStr;
use std::fs::{OpenOptions, metadata};
use std::io::Write;
use std::path::Path;
use syn::{DeriveInput, ItemFn};

/// Get the full path to `{group}_{pid}_{thread_id}.jsonl` generated in OUT_DIR.
fn get_prebindgen_jsonl_path(name: &str) -> std::path::PathBuf {
    let thread_id = std::thread::current().id();
    let process_id = std::process::id();
    // Extract numeric thread ID from ThreadId debug representation
    let thread_id_str = format!("{:?}", thread_id);
    let thread_id_num = thread_id_str
        .strip_prefix("ThreadId(")
        .and_then(|s| s.strip_suffix(")"))
        .unwrap_or("0");
    get_prebindgen_out_dir().join(format!("{}_{}_{}.jsonl", name, process_id, thread_id_num))
}

#[proc_macro]
pub fn prebindgen_out_dir(_input: TokenStream) -> TokenStream {
    let file_path = get_prebindgen_out_dir();
    let path_str = file_path.to_string_lossy();

    // Return just the string literal
    let expanded = quote! {
        #path_str
    };

    TokenStream::from(expanded)
}

/// Attribute macro that copies the annotated item into `<group>.jsonl` in OUT_DIR.
/// Requires a string literal group name: `#[prebindgen("group_name")]`.
#[proc_macro_attribute]
pub fn prebindgen(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    // Parse required JSON group name literal
    let group_lit = syn::parse::<LitStr>(args)
        .expect("`#[prebindgen]` requires a string literal group name");
    let group = group_lit.value();
    // Get the full path to the JSONL file
    let file_path = get_prebindgen_jsonl_path(&group);
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

    // Convert record to JSON and append to file in JSON-lines format
    if let Ok(json_content) = serde_json::to_string(&new_record) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(dest_path) {
            // Check if file is empty (just created or was deleted)
            let is_empty = metadata(dest_path).map(|m| m.len() == 0).unwrap_or(true);

            if is_empty {
                // Create new JSONL file
                trace!("Creating jsonl file: {}", dest_path.display());
            }
            
            // Write the record as a single line (JSON-lines format)
            let _ = writeln!(file, "{}", json_content);
            let _ = file.flush();
        }
    }

    // Return the original input unchanged
    input_clone
}
