// the separation to main.rs and lib.rs is very artificial here
// as I just want tu run `cargo run --lib` on host target
// so all of the stuff that is WASM specific is in main
pub mod gui;
