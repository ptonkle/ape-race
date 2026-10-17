//! Writes the ape calls as WAV files for the design journal (TRIAGE T58):
//! `cargo run -p tool_core --example ape_calls -- <dir>` makes
//! `<dir>/<kind>.wav` for every kind and `<dir>/gorilla-pass.wav` (the
//! gorilla's clip heard as it swings past), all at full volume.
use std::fs;
use tool_core::rules::Kind;
use tool_core::sound;

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    fs::create_dir_all(&dir).expect("make the output folder");
    let mut out: Vec<(String, Vec<u8>)> = Kind::ALL
        .iter()
        .map(|&k| (format!("{}.wav", k.key()), sound::call(k, 100)))
        .collect();
    out.push(("gorilla-pass.wav".to_string(), sound::gorilla_pass(100)));
    for (name, bytes) in out {
        let path = format!("{dir}/{name}");
        fs::write(&path, &bytes).expect("write the WAV");
        println!("{path}: {} bytes", bytes.len());
    }
}
