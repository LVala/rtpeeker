use std::env;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // TODO: in theory we shouldn't modify files outside of OUT_DIR
    // but oh well
    Command::new("trunk")
        .args(["build", "--release", "--dist"])
        .arg(&format!("{}/client/dist", manifest_dir))
        .arg(&format!("{}/client/index.html", manifest_dir))
        .status()
        .unwrap();

    println!("cargo:rerun-if-changed=client");
}
