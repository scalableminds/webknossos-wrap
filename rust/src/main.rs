extern crate wkwrap as wkw;

use std::env;

fn recover_header() {
    let pwd = env::current_dir().unwrap();
    wkw::dataset::recover_header(pwd.as_path()).unwrap();
}

fn main() {
    // parse input arguments
    let args: Vec<String> = env::args().collect();
    let arg_count = args.len();

    if arg_count < 2 {
        println!("Not enough input arguments");
        return;
    }

    // parse sub-command
    match args[1].as_ref() {
        "recover-header" => recover_header(),
        "verify-headers" => println!("Not implemented yet"),
        _                => println!("Invalid sub-command")
    }
}
