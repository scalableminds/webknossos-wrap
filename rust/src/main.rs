extern crate wkwrap as wkw;

use std::env;
use std::path::Path;

fn recover_header(pwd: &Path) -> bool {
    let recovery = wkw::dataset::recover_header(pwd);

    if recovery.is_ok() {
        verify_headers(pwd)
    } else {
        println!("Could not recover .wkw header");
        false
    }
}

fn verify_headers(pwd: &Path) -> bool {
    let dataset = wkw::Dataset::new(pwd).unwrap();
    let okay = dataset.verify_headers().unwrap();

    if !okay {
        println!("Found .wkw file(s) with conflicting header");
    }

    okay
}

fn main() {
    // parse input arguments
    let pwd = env::current_dir().unwrap();
    let args: Vec<String> = env::args().collect();
    let arg_count = args.len();

    if arg_count < 2 {
        println!("Not enough input arguments");
        return;
    }

    // parse sub-command
    let exit_code = match args[1].as_ref() {
        "recover-header" => if recover_header(pwd.as_path()) { 0 } else { -2 },
        "verify-headers" => if verify_headers(pwd.as_path()) { 0 } else { -3 },
        _                => -1
    };

    std::process::exit(exit_code as i32);
}
