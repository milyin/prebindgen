use super::*;
use crate::{api::test_util::unique_test_dir, SourceLocation};

mod builder;
mod callbacks;
mod errors;
mod inputs;
mod lowering;
mod returns;
mod structs;

fn write(cbindgen: Cbindgen, registry: Registry<()>, tag: &str) -> String {
    let dir = unique_test_dir(&format!("cbindgen_{tag}"));
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join(format!("{tag}.rs"));
    let gen = registry.resolve(cbindgen).expect("resolve");
    let path = gen.write_rust(&out).expect("write_rust");
    std::fs::read_to_string(&path).unwrap()
}

fn error_struct() -> syn::ItemStruct {
    syn::parse_quote!(
        pub struct Error {
            pub message: String,
        }
    )
}

fn catch<F: FnOnce()>(f: F) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err()
}
