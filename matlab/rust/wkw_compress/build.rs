use std::env;

fn main() {
    let link_paths = env::var("EXTRALINKPATHS")
	                    .expect("EXTRALINKPATHS not set");
	
	for link_path in link_paths.split(";") {
		println!("cargo:rustc-link-search={}", link_path);
	}
}
