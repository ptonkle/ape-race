//! tool_core: every rule of Ape Race.
//!
//! Builder requirement: this crate performs no input or output. It takes
//! values and returns values, so `cargo test` reaches every rule without a
//! browser. `tests/boundary.rs` enforces that, along with the dependency
//! allowlist and the reference ban on floats, hash maps and clocks.

// Vectors use named methods (add, sub, neg) rather than operator traits, so
// every Q16 operation is visible where it happens.
#![allow(
    clippy::too_many_arguments,
    clippy::needless_range_loop,
    clippy::should_implement_trait,
    clippy::len_without_is_empty
)]

pub mod ai;
pub mod ape;
pub mod api;
pub mod app;
pub mod camera;
pub mod city;
pub mod cityscape;
pub mod course;
pub mod focus;
pub mod fx;
pub mod geom;
pub mod hype;
pub mod icon;
pub mod items;
pub mod jaguar;
pub mod map;
pub mod mesh;
pub mod models;
pub mod online;
pub mod race;
pub mod render;
pub mod replay;
pub mod rules;
pub mod scene;
pub mod ski;
pub mod snow;
pub mod sky;
pub mod sound;
pub mod speedo;
pub mod trial;
pub mod vehicles;
pub mod vinesim;
