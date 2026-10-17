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
pub mod bay;
pub mod baymodels;
pub mod bayscape;
pub mod camera;
pub mod city;
pub mod cityscape;
pub mod costume;
pub mod course;
pub mod dance;
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
pub mod patch_notes;
pub mod race;
pub mod ravine;
pub mod ravinemodels;
pub mod ravinescape;
pub mod render;
pub mod replay;
pub mod ruins;
pub mod ruinsmodels;
pub mod ruinsscape;
pub mod rules;
pub mod scene;
pub mod sector;
pub mod sectorscape;
pub mod see;
pub mod ski;
pub mod skiscape;
pub mod sky;
pub mod snow;
pub mod sound;
pub mod spacecraft;
pub mod speedo;
pub mod tower;
pub mod towermodels;
pub mod towerscape;
pub mod trial;
pub mod vehicles;
pub mod vinesim;
