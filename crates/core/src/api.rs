//! The one door into the core. The shim holds a [`Session`] and forwards
//! each call; the page never reaches past it.

use crate::app::App;
use crate::render;

pub struct Session {
    app: Result<App, String>,
}

impl Session {
    pub fn new(strings: &str, settings: &str, seed: u64) -> Session {
        Session {
            app: App::new(strings, settings, seed),
        }
    }

    /// The view model as JSON; `{"error": ...}` when the data could not load.
    pub fn ui(&mut self) -> String {
        match &mut self.app {
            Ok(a) => a.ui(),
            Err(e) => serde_json::json!({ "error": e }).to_string(),
        }
    }

    pub fn act(&mut self, id: &str, value: &str) {
        if let Ok(a) = &mut self.app {
            a.act(id, value);
        }
    }

    /// The replays kept in browser storage (TRIAGE T15), once at start.
    pub fn load_replays(&mut self, text: &str) {
        if let Ok(a) = &mut self.app {
            a.load_replays(text);
        }
    }

    pub fn key(&mut self, code: &str, down: bool) {
        if let Ok(a) = &mut self.app {
            a.key(code, down);
        }
    }

    pub fn mouse_move(&mut self, dx: i32, dy: i32) {
        if let Ok(a) = &mut self.app {
            a.mouse_move(dx, dy);
        }
    }

    pub fn mouse_button(&mut self, button: i32, down: bool) {
        if let Ok(a) = &mut self.app {
            a.mouse_button(button, down);
        }
    }

    /// A drag over the 3D view while the pointer is free (TRIAGE T23).
    pub fn drag(&mut self, dx: i32, dy: i32) {
        if let Ok(a) = &mut self.app {
            a.drag(dx, dy);
        }
    }

    /// The mouse wheel over the 3D view.
    pub fn wheel(&mut self, dy: i32) {
        if let Ok(a) = &mut self.app {
            a.wheel(dy);
        }
    }

    pub fn pointer_lock(&mut self, locked: bool) {
        if let Ok(a) = &mut self.app {
            a.pointer_lock(locked);
        }
    }

    pub fn blur(&mut self) {
        if let Ok(a) = &mut self.app {
            a.blur();
        }
    }

    pub fn advance(&mut self, now_us: i64) {
        if let Ok(a) = &mut self.app {
            a.advance(now_us);
        }
    }

    /// The CSS size and the device pixel ratio (in hundredths) in; the
    /// drawing buffer's size out, capped so a big screen stays smooth.
    pub fn resize(&mut self, css_w: i32, css_h: i32, dpr: i32) -> Vec<i32> {
        let dpr = dpr.clamp(100, 200);
        let mut w = css_w.max(1) * dpr / 100;
        let mut h = css_h.max(1) * dpr / 100;
        let most = 2560;
        if w > most {
            h = h * most / w;
            w = most;
        }
        if let Ok(a) = &mut self.app {
            a.resize(w, h);
        }
        vec![w, h.max(1)]
    }

    /// A transport event from web/online.js: connected, message,
    /// disconnected or error.
    pub fn net_event(&mut self, kind: &str, peer: &str, text: &str, host: bool) {
        if let Ok(a) = &mut self.app {
            a.net_event(kind, peer, text, host);
        }
    }

    /// An invite or reply code from the pasted-code path.
    pub fn net_code(&mut self, which: &str, code: &str) {
        if let Ok(a) = &mut self.app {
            a.net_code(which, code);
        }
    }

    /// Messages to send, as JSON: [{"to": peer or null, "text": ...}].
    pub fn outbox(&mut self) -> String {
        match &mut self.app {
            Ok(a) => a.take_outbox(),
            Err(_) => "[]".to_string(),
        }
    }

    pub fn scene_version(&self) -> u32 {
        self.app.as_ref().map(|a| a.scene_version()).unwrap_or(0)
    }

    pub fn static_scene(&self) -> (Vec<i32>, Vec<u32>, Vec<i32>) {
        self.app
            .as_ref()
            .map(|a| a.static_scene())
            .unwrap_or_default()
    }

    pub fn frame(&self) -> (Vec<i32>, Vec<i32>, Vec<u32>) {
        self.app.as_ref().map(|a| a.frame()).unwrap_or_default()
    }

    pub fn sound(&self, key: &str) -> Vec<u8> {
        self.app.as_ref().map(|a| a.sound(key)).unwrap_or_default()
    }
}

pub fn numbers() -> String {
    render::numbers_json()
}

pub fn mesh(id: u32) -> Vec<i32> {
    crate::mesh::Mesh::ALL
        .iter()
        .find(|m| **m as u32 == id)
        .map(|m| crate::mesh::vertices(*m))
        .unwrap_or_default()
}

pub fn uniforms() -> Vec<i32> {
    crate::scene::uniforms()
}

pub fn clear_colour() -> Vec<i32> {
    crate::scene::clear_colour()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_strings_give_an_error_view() {
        let mut s = Session::new("not json", "", 1);
        assert!(s.ui().contains("error"));
        assert_eq!(s.frame(), Default::default());
    }

    #[test]
    fn the_buffer_size_follows_the_pixel_ratio_and_is_capped() {
        let mut s = Session::new("{}", "", 1);
        assert_eq!(s.resize(800, 600, 150), vec![1200, 900]);
        assert_eq!(s.resize(3000, 1000, 200)[0], 2560);
    }
}
