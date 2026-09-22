//! The two conditions this crate's items are written under.
//!
//! Neither is ever set. `v2check` compiles the generated code that mirrors
//! them, and declaring them here is what keeps `rustc`'s unexpected-`cfg` lint
//! from firing on the source they are written in.

fn main() {
    println!("cargo::rustc-check-cfg=cfg(v2check_conditional_field)");
    println!("cargo::rustc-check-cfg=cfg(v2check_conditional_fn)");
}
