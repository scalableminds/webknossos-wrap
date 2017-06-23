use std::env;

fn main() {
    let lib_root = env::var("MEXLIBROOT").expect("MEXLIBROOT not set");
    println!("cargo:rustc-link-search={}", lib_root)
}
