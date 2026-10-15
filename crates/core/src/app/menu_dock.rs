//! Where the menus sit beside the previewed ape (TRIAGE T47), and the
//! preview camera framed to match. On the screens that show the apes (Race
//! Setup and the online Lobby, `lineup.rs`) the menus dock to the left on a
//! wide screen and the camera slides so the apes stand in the open part;
//! on a narrow or tall screen the menus go below and the camera backs off
//! and drops so the ape stands in the top part. The page only applies the
//! dock it is given (`body[data-dock]`, `--dock-w`, `--dock-top` in
//! `web/style.css`), so the framing and the layout cannot disagree.
//!
//! The previewed apes stand on the chosen map, at its own lobby spot with
//! the map behind them ([`preview_spot`]; the Jungle's too since T77).

use serde_json::{json, Value};

use super::{App, Stage};
use crate::course::Course;
use crate::fx::{self, cm, deg, ratio, ONE};
use crate::geom::{v3, V3};
use crate::render::{Batch, Camera};

/// A menu panel's width in CSS pixels (25 rem: `.panel.setup`).
const PANEL_W: i32 = 400;
/// The space around and between docked panels, in CSS pixels.
const GAP: i32 = 24;
/// The narrowest open part beside the docked menus that still shows the
/// ape well, in CSS pixels; narrower screens put the menus below.
const OPEN_MIN: i32 = 320;
/// With the menus below, the share of the height left for the ape (Q16).
const OPEN_TOP: i32 = ratio(46, 100);
/// The ape's middle above its feet, where the preview camera looks
/// (`models::showcase_camera`).
pub(super) const FOCUS_UP: i32 = cm(62);
/// The preview camera's height above the feet and its distance from the
/// ape (`models::showcase_camera`: 1.25 m up, 2.9 m off).
const EYE_UP: i32 = cm(125);
const EYE_BACK: i32 = cm(290);
/// The furthest the preview camera backs off to fit a lineup (Q16 times
/// one ape's framing).
const MOST_BACK: i32 = 12 * ONE;

/// Where the previewed ape stands on a map and the way the camera looks
/// at it (flat, unit): every map's own lobby spot (`scene::preview`: the
/// Jungle's start platform, the City's bridge, the Ski Slope's piste,
/// Sector Ape's launch deck; TRIAGE T77, the Jungle stood at the origin on
/// the forest floor before), seen from the side its preview camera looks
/// from, and that camera's eye from the ape's middle (each map sets its
/// own height and angle: the City's looks up at the bridge). Before a
/// course is built, the origin seen from +z.
pub(super) fn preview_spot(course: Option<&Course>) -> (V3, V3, V3) {
    let standard = |look: V3| v3(0, EYE_UP - FOCUS_UP, 0).sub(look.scale(EYE_BACK));
    match course {
        None => (V3::ZERO, v3(0, 0, -ONE), standard(v3(0, 0, -ONE))),
        Some(c) => {
            let spot = crate::scene::preview(c);
            let ahead = spot.feet.sub(spot.camera.eye).flat();
            let look = if ahead.len() > 0 {
                ahead.norm()
            } else {
                c.heading_at(0).flat().norm()
            };
            let off = spot.camera.eye.sub(spot.feet.add(v3(0, FOCUS_UP, 0)));
            let off = if ahead.len() > 0 { off } else { standard(look) };
            (spot.feet, look, off)
        }
    }
}

/// The preview camera for one ape standing at `feet`, looking `look`.
#[cfg(test)]
fn preview_view(feet: V3, look: V3) -> Camera {
    let off = v3(0, EYE_UP - FOCUS_UP, 0).sub(look.scale(EYE_BACK));
    preview_view_back(feet.add(v3(0, FOCUS_UP, 0)), off, ONE)
}

/// A camera on `target` (an ape's middle) from `off` away, `back` times as
/// far off (Q16; ONE frames one ape). Backing off, it rises only a
/// quarter as fast as it goes back, so a lineup is seen against the map
/// rather than from above against the ground.
fn preview_view_back(target: V3, off: V3, back: i32) -> Camera {
    let rise = ONE + (back - ONE) / 4;
    let eye = target
        .add(v3(off.x, 0, off.z).scale(back))
        .add(v3(0, fx::mul(off.y, rise), 0));
    Camera {
        eye,
        fwd: target.sub(eye).norm(),
        fov: deg(34),
    }
}

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

    pub(super) fn preview_at(&self) -> (V3, V3) {
        let (feet, look, _) = preview_spot(self.course_of(self.map()));
        (feet, look)
    }

    /// The preview camera on the chosen map, framed beside the menus: on
    /// the lineup's middle (TRIAGE T76), backed off along its own line
    /// until every ape in the lineup fits the open part of the screen.
    pub(super) fn preview_camera(&self) -> Camera {
        let stands = self.lineup();
        self.lineup_camera(&stands)
    }

    fn lineup_camera(&self, stands: &[super::lineup::Stand]) -> Camera {
        let (_, look, off) = preview_spot(self.course_of(self.map()));
        let focus = Self::lineup_focus(stands);
        let right = crate::geom::right_of(look);
        let mut back = ONE;
        loop {
            let cam = self.framed(preview_view_back(focus, off, back), focus);
            if stands.len() < 2 || back >= MOST_BACK || self.fits(&cam, stands, right) {
                return cam;
            }
            back += ONE / 16;
        }
    }

    /// The open part of the screen the dock leaves, in hundredths of the
    /// width and height (Q16, as `Camera::screen` gives them), less a margin.
    pub(super) fn open_box(&self) -> (i32, i32, i32, i32) {
        let (w, h) = (self.view_w.max(1), self.view_h.max(1));
        let m = 3 * ONE;
        let (x0, y1) = match self.dock() {
            Dock::Centre => (0, 100 * ONE),
            Dock::Side(d) => (fx::div(d * 100, w), 100 * ONE),
            Dock::Below(top) => (0, fx::div(top * 100, h)),
        };
        (x0 + m, 100 * ONE - m, m, y1 - m)
    }

    /// The preview's moving parts: the vines and the map's moving backdrop
    /// round the spot, and the lineup with your pedestal (TRIAGE T76).
    pub(super) fn preview_batches(&self, vivid: bool) -> (Batch, Batch) {
        let mut b = Batch::new();
        let stands = self.lineup();
        let eye = self.lineup_camera(&stands).eye;
        self.draw_lineup(&mut b, &stands, eye);
        let Some(c) = self.course_of(self.map()) else {
            return (Batch::new(), b);
        };
        crate::scene::backdrop(&mut b, c, eye, self.ticks);
        let focus = Self::lineup_focus(&stands);
        let vines = crate::scene::flyover_dynamic(c, self.ticks, vivid, focus).0;
        (vines, b)
    }

    /// The preview camera moved so the ape (its middle at `focus`) stands
    /// in the open part of the screen the dock leaves.
    pub(super) fn framed(&self, cam: Camera, focus: V3) -> Camera {
        let h = self.view_h.max(1);
        let fwd = cam.fwd.norm();
        let right = fwd.cross(V3::UP).norm();
        let up = right.cross(fwd);
        let half = cam.fov / 2;
        let tan = fx::div(fx::sin(half), fx::cos(half));
        let d = focus.sub(cam.eye).dot(fwd);
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
                let eye = focus.sub(fwd.scale(back));
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
    use crate::map::Map;
    use crate::models;

    fn app_at(screen: Screen, w: i32, h: i32) -> App {
        let mut a = App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap();
        a.resize(w, h);
        a.go(screen);
        a
    }

    /// Where the ape's middle lands, in hundredths of the width and height.
    fn ape_on_screen(a: &App) -> (i32, i32) {
        let cam = a.preview_camera();
        // on its pedestal: the lineup of one
        let focus = App::lineup_focus(&a.lineup());
        let (x, y) = cam.screen(fx::div(a.view_w, a.view_h), focus).unwrap();
        (x / ONE, y / ONE)
    }

    #[test]
    fn the_jungle_preview_camera_is_the_showcase_camera() {
        let cam = preview_view(V3::ZERO, v3(0, 0, -ONE));
        let old = models::showcase_camera();
        assert_eq!(cam.eye, old.eye);
        assert_eq!(cam.fwd, old.fwd);
        assert_eq!(cam.fov, old.fov);
    }

    #[test]
    fn on_the_city_the_ape_stands_at_its_preview_spot_framed_beside_the_menus() {
        for (w, h) in [(1280, 720), (390, 844)] {
            let mut a = app_at(Screen::Setup, w, h);
            a.act("setup.map", "city");
            let (feet, _) = a.preview_at();
            let spot = crate::scene::preview(a.course_of(Map::City).unwrap());
            assert_eq!(feet, spot.feet, "the City's own preview spot");
            let (x, y) = ape_on_screen(&a);
            match a.dock() {
                Dock::Side(dock) => {
                    let open_middle = (dock + (w - dock) / 2) * 100 / w;
                    assert!((x - open_middle).abs() <= 2, "{w}x{h}: ape at {x}%");
                    assert!((40..=60).contains(&y), "{w}x{h}: ape at {y}% down");
                }
                Dock::Below(top) => {
                    assert!((x - 50).abs() <= 1, "{w}x{h}: ape at {x}% across");
                    assert!((y - top * 50 / h).abs() <= 2, "{w}x{h}: ape at {y}% down");
                }
                Dock::Centre => panic!("the setup docks its menus"),
            }
        }
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
