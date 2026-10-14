//! tool_shim: moves values between the page and `tool_core::api`.
//!
//! Builder requirement: the shim decides nothing. Each method is one call
//! into the core and a conversion of its arguments or result. The core
//! speaks Q16.16 integers; WebGL wants `f32`, so values on the way out are
//! divided by the core's own `ONE`; the page's float clock (milliseconds)
//! becomes whole microseconds, and mouse movement whole pixels.
//! `crates/core/tests/boundary.rs` fails if a branch, a loop or the `?`
//! operator appears here.

use tool_core::{api, fx};
use wasm_bindgen::prelude::*;

const ONE: f32 = fx::ONE as f32;

fn floats(v: Vec<i32>) -> Vec<f32> {
    v.into_iter().map(|x| x as f32 / ONE).collect()
}

#[wasm_bindgen]
pub struct Game {
    s: api::Session,
    frame: (Vec<i32>, Vec<i32>, Vec<u32>),
    scene: (Vec<i32>, Vec<u32>, Vec<i32>),
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(strings: &str, settings: &str, entropy: f64) -> Game {
        Game {
            s: api::Session::new(strings, settings, entropy as u64),
            frame: Default::default(),
            scene: Default::default(),
        }
    }

    pub fn ui(&mut self) -> String {
        self.s.ui()
    }

    pub fn act(&mut self, id: &str, value: &str) {
        self.s.act(id, value)
    }

    pub fn load_replays(&mut self, text: &str) {
        self.s.load_replays(text)
    }

    pub fn replays_not_saved(&mut self) {
        self.s.replays_not_saved()
    }

    pub fn key(&mut self, code: &str, down: bool) {
        self.s.key(code, down)
    }

    pub fn mouse_move(&mut self, dx: f64, dy: f64) {
        self.s.mouse_move(dx as i32, dy as i32)
    }

    pub fn mouse_button(&mut self, button: i32, down: bool) {
        self.s.mouse_button(button, down)
    }

    pub fn drag(&mut self, dx: f64, dy: f64) {
        self.s.drag(dx as i32, dy as i32)
    }

    pub fn wheel(&mut self, dy: f64) {
        self.s.wheel(dy as i32)
    }

    pub fn pointer_lock(&mut self, locked: bool) {
        self.s.pointer_lock(locked)
    }

    pub fn blur(&mut self) {
        self.s.blur()
    }

    pub fn advance(&mut self, now_ms: f64) {
        self.s.advance((now_ms * 1000.0) as i64)
    }

    /// CSS size and device pixel ratio in; drawing-buffer size out.
    pub fn resize(&mut self, css_w: f64, css_h: f64, dpr: f64) -> Vec<i32> {
        self.s
            .resize(css_w as i32, css_h as i32, (dpr * 100.0) as i32)
    }

    pub fn net_event(&mut self, kind: &str, peer: &str, text: &str, host: bool) {
        self.s.net_event(kind, peer, text, host)
    }

    pub fn net_code(&mut self, which: &str, code: &str) {
        self.s.net_code(which, code)
    }

    pub fn outbox(&mut self) -> String {
        self.s.outbox()
    }

    pub fn scene_version(&self) -> u32 {
        self.s.scene_version()
    }

    /// Builds the static scene; read it with the three methods below.
    pub fn build_scene(&mut self) {
        self.scene = self.s.static_scene()
    }

    pub fn static_instances(&self) -> Vec<f32> {
        floats(self.scene.0.clone())
    }

    pub fn static_draws(&self) -> Vec<u32> {
        self.scene.1.clone()
    }

    pub fn terrain(&self) -> Vec<f32> {
        floats(self.scene.2.clone())
    }

    /// Builds this frame; read it with the three methods below.
    pub fn build_frame(&mut self) {
        self.frame = self.s.frame()
    }

    pub fn view(&self) -> Vec<f32> {
        floats(self.frame.0.clone())
    }

    pub fn instances(&self) -> Vec<f32> {
        floats(self.frame.1.clone())
    }

    pub fn draws(&self) -> Vec<u32> {
        self.frame.2.clone()
    }

    pub fn sound(&self, key: &str) -> Vec<u8> {
        self.s.sound(key)
    }
}

#[wasm_bindgen]
pub fn numbers() -> String {
    api::numbers()
}

#[wasm_bindgen]
pub fn mesh(id: u32) -> Vec<f32> {
    floats(api::mesh(id))
}

#[wasm_bindgen]
pub fn uniforms() -> Vec<f32> {
    floats(api::uniforms())
}

#[wasm_bindgen]
pub fn clear_colour() -> Vec<f32> {
    floats(api::clear_colour())
}
