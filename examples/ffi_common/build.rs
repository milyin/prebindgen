fn main() {
    println!("cargo:warning=OUT_DIR: {}", std::env::var("OUT_DIR").unwrap());
}