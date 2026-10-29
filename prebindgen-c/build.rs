fn main() {
    // This is the platform on which this copy of the generator executes, not
    // the eventual binding target. No type-layout probe is compiled here.
    println!(
        "cargo:rustc-env=PREBINDGEN_C_GENERATOR_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
