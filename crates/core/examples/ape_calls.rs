//! Writes the ape calls as WAV files for the design journal (TRIAGE T58):
//! `cargo run -p tool_core --example ape_calls -- <dir>` makes
//! `<dir>/gorilla.wav` (the call) and `<dir>/gorilla-pass.wav` (the same
//! clip heard as the gorilla swings past), both at full volume.
use std::fs;
use tool_core::sound;

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    fs::create_dir_all(&dir).expect("make the output folder");
    let out = [
        ("gorilla.wav", sound::gorilla(100)),
        ("gorilla-pass.wav", sound::gorilla_pass(100)),
    ];
    for (name, bytes) in out {
        let path = format!("{dir}/{name}");
        fs::write(&path, &bytes).expect("write the WAV");
        println!("{path}: {} bytes", bytes.len());
    }
}
