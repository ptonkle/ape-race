//! Where the menus sit beside the previewed ape (TRIAGE T47), and the
//! preview camera framed to match. On the screens that show one ape (Race
//! Setup and the online Lobby) the menus dock to the left on a wide screen
//! and the camera slides so the ape stands in the middle of the open part;
//! on a narrow or tall screen the menus go below and the camera backs off
//! and drops so the ape stands in the top part. The page only applies the
//! dock it is given (`body[data-dock]`, `--dock-w`, `--dock-top` in
//! `web/style.css`), so the framing and the layout cannot disagree.

use serde_json::{json, Value};

use super::{App, Stage};
use crate::fx::{self, cm, ratio, ONE};
use crate::geom::{v3, V3};
use crate::render::Camera;

/// A menu panel's width in CSS pixels (25 rem: `.panel.setup`).
const PANEL_W: i32 = 400;
/// The space around and between docked panels, in CSS pixels.
const GAP: i32 = 24;
/// The narrowest open part beside the docked menus that still shows the
/// ape well, in CSS pixels; narrower screens put the menus below.
const OPEN_MIN: i32 = 320;
/// With the menus below, the share of the height left for the ape (Q16).
const OPEN_TOP: i32 = ratio(46, 100);
/// The ape's middle, where the preview camera looks (`models::showcase_camera`).
const FOCUS: V3 = v3(0, cm(62), 0);

/// Where the menus go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Dock {
    /// In the middle, as on every screen without an ape preview.
    Centre,
    /// A column on the left this many CSS pixels wide (one panel, or two
    /// side by side when the screen is wide enough for both and the ape).
    Side(i32),
    /// Below the ape, from this many CSS pixels down.
    Below(i32),
}

impl App {
    pub(super) fn dock(&self) -> Dock {
        if self.stage != Stage::Showcase {
            return Dock::Centre;
        }
        let (w, h) = (self.view_w, self.view_h);
        let one = PANEL_W + 2 * GAP;
        let two = 2 * PANEL_W + 3 * GAP;
        let panels = self.panel_count();
        if w > h && w >= two + PANEL_W && panels >= 2 {
            Dock::Side(two)
        } else if w > h && w >= one + OPEN_MIN {
            Dock::Side(one)
        } else {
            Dock::Below(fx::mul(h, OPEN_TOP))
        }
    }

    fn panel_count(&self) -> usize {
        match self.screen {
            super::Screen::Lobby => self.lobby_panels().len(),
            _ => 1,
        }
    }

    /// The dock for the page: `dock`, `dock_w` and `dock_top` as CSS values.
    pub(super) fn dock_ui(&self, v: &mut Value) {
        let (name, w, top) = match self.dock() {
            Dock::Centre => ("centre", 0, 0),
            Dock::Side(w) => ("side", w, 0),
            Dock::Below(top) => ("below", 0, top),
        };
        v["dock"] = json!(name);
        v["dock_w"] = json!(format!("{w}px"));
        v["dock_top"] = json!(format!("{top}px"));
    }

    /// The preview camera moved so the ape stands in the open part of the
    /// screen the dock leaves.
    pub(super) fn framed(&self, cam: Camera) -> Camera {
        let h = self.view_h.max(1);
        let fwd = cam.fwd.norm();
        let right = fwd.cross(V3::UP).norm();
        let up = right.cross(fwd);
        let half = cam.fov / 2;
        let tan = fx::div(fx::sin(half), fx::cos(half));
        let d = FOCUS.sub(cam.eye).dot(fwd);
        match self.dock() {
            Dock::Centre => cam,
            Dock::Side(w) => {
                // the open part's middle is w / width right of the screen's
                // middle in clip space, which is w / height of tan * d
                let slide = fx::mul(fx::mul(fx::div(w, h), tan), d);
                Camera {
                    eye: cam.eye.sub(right.scale(slide)),
                    ..cam
                }
            }
            Dock::Below(top) => {
                // back off so the ape fits the top part, then drop the
                // camera so it stands in that part's middle
                let open = fx::div(top, h).clamp(ONE / 4, ONE);
                let back = fx::mul(fx::div(d, open), ratio(2, 3)).max(d);
                let eye = FOCUS.sub(fwd.scale(back));
                let drop = fx::mul(fx::mul(ONE - open, tan), back);
                Camera {
                    eye: eye.sub(up.scale(drop)),
                    ..cam
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Screen;
    use super::*;
    use crate::models;

    fn app_at(screen: Screen, w: i32, h: i32) -> App {
        let mut a = App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap();
        a.resize(w, h);
        a.go(screen);
        a
    }

    /// Where the ape's middle lands, in hundredths of the width and height.
    fn ape_on_screen(a: &App) -> (i32, i32) {
        let cam = a.framed(models::showcase_camera());
        let (x, y) = cam.screen(fx::div(a.view_w, a.view_h), FOCUS).unwrap();
        (x / ONE, y / ONE)
    }

    #[test]
    fn on_a_wide_screen_the_menus_dock_left_and_the_ape_stands_in_the_open_part() {
        for (w, h) in [(1280, 720), (1920, 1080), (900, 700)] {
            let a = app_at(Screen::Setup, w, h);
            let Dock::Side(dock) = a.dock() else {
                panic!("{w}x{h}: {:?}", a.dock())
            };
            let open_middle = (dock + (w - dock) / 2) * 100 / w;
            let (x, y) = ape_on_screen(&a);
            assert!(
                (x - open_middle).abs() <= 2,
                "{w}x{h}: ape at {x}%, open part's middle {open_middle}%"
            );
            assert!((40..=60).contains(&y), "{w}x{h}: ape at {y}% down");
        }
    }

    #[test]
    fn the_lobby_docks_its_two_panels_side_by_side_when_there_is_room() {
        let mut a = app_at(Screen::Online, 1920, 1080);
        a.act("online.host", "");
        assert_eq!(a.screen, Screen::Lobby);
        assert_eq!(a.dock(), Dock::Side(2 * PANEL_W + 3 * GAP));
        a.resize(1280, 720);
        assert_eq!(a.dock(), Dock::Side(2 * PANEL_W + 3 * GAP));
        a.resize(1100, 720);
        assert_eq!(a.dock(), Dock::Side(PANEL_W + 2 * GAP));
        let a = app_at(Screen::Setup, 1920, 1080);
        assert_eq!(a.dock(), Dock::Side(PANEL_W + 2 * GAP));
    }

    #[test]
    fn on_a_phone_the_menus_go_below_and_the_ape_stands_above_them() {
        let a = app_at(Screen::Lobby, 390, 844);
        let Dock::Below(top) = a.dock() else {
            panic!("{:?}", a.dock())
        };
        let (x, y) = ape_on_screen(&a);
        let open_middle = top * 50 / 844;
        assert!((x - 50).abs() <= 1, "ape at {x}% across");
        assert!(
            (y - open_middle).abs() <= 2,
            "ape at {y}% down, open part's middle {open_middle}%"
        );
    }

    #[test]
    fn screens_without_the_ape_keep_their_menus_in_the_middle() {
        let mut a = app_at(Screen::Title, 1280, 720);
        assert_eq!(a.dock(), Dock::Centre);
        a.go(Screen::Online);
        assert_eq!(a.dock(), Dock::Centre);
        let ui: Value = serde_json::from_str(&a.ui()).unwrap();
        assert_eq!(ui["dock"], "centre");
        a.go(Screen::Setup);
        let ui: Value = serde_json::from_str(&a.ui()).unwrap();
        assert_eq!(ui["dock"], "side");
        assert_eq!(ui["dock_w"], "448px");
    }
}
