fn main() {
  println!("cargo:rustc-link-search={}", "../lz4/lib");
  println!("cargo:rustc-link-search={}", "../zfp/lib");
}
