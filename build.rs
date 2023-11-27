use std::env;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    println!("MANIFEST_DIR: {:?}", manifest_dir);
    println!("OUT_DIR: {:?}", out_dir);

    panic!();
}
