//! Where the menus sit beside the previewed ape (TRIAGE T47), and the
//! preview camera framed to match. On the screens that show one ape (Race
//! Setup and the online Lobby) the menus dock to the left on a wide screen
//! and the camera slides so the ape stands in the middle of the open part;
//! on a narrow or tall screen the menus go below and the camera backs off
//! and drops so the ape stands in the top part. The page only applies the
//! dock it is given (`body[data-dock]`, `--dock-w`, `--dock-top` in
//! `web/style.css`), so the framing and the layout cannot disagree.
//!
//! The previewed ape stands on the chosen map (the City, 2026-10-12): on
//! the Jungle in its small jungle stage as before, elsewhere at a spot on
//! the map's course with the map behind it ([`preview_spot`]).

use serde_json::{json, Value};

use super::{App, Stage};
use crate::course::Course;
use crate::fx::{self, cm, deg, ratio, ONE, TURN};
use crate::geom::{facing, v3, yaw_of, V3};
use crate::map::Map;
use crate::models;
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
const FOCUS_UP: i32 = cm(62);
/// The preview camera's height above the feet and its distance from the
/// ape (`models::showcase_camera`: 1.25 m up, 2.9 m off).
const EYE_UP: i32 = cm(125);
const EYE_BACK: i32 = cm(290);
/// Half the ape's width on screen in Sunset Ruins' and Banana Bay
/// Battle's lobbies, as a share of the height (their lobby cameras stand
/// 3.6 m back).
const APE_HALF_W: i32 = ratio(14, 100);
/// The most a map's own lobby camera turns from its own view.
const MAX_TURN: i32 = deg(30);
/// Banana Bay Battle's lobby camera stands at least this far back for
/// each screen height the open part beside the menus is wide (so the ape,
/// at its widest as it turns, fits the open part).
const BAY_FIT: i32 = cm(410);

/// Where the previewed ape stands on a map and the way the camera looks
/// at it (flat, unit). The Jungle: its stage at the origin, seen from +z
/// as before the City. Elsewhere: the course's first start, looking down
/// the course, so the map stretches away behind the ape. Built from the
/// shared course data so it works on any course; the art agent's City
/// framing slots in here.
pub(super) fn preview_spot(map: Map, course: Option<&Course>) -> (V3, V3) {
    match (map, course) {
        (Map::Jungle, _) | (_, None) => (V3::ZERO, v3(0, 0, -ONE)),
        (_, Some(c)) => {
            // the map's own spot (the City: a Midtown street), seen from
            // the side its preview camera looks from
            let spot = crate::scene::preview(c);
            let ahead = spot.feet.sub(spot.camera.eye).flat();
            let look = if ahead.len() > 0 {
                ahead.norm()
            } else {
                c.heading_at(0).flat().norm()
            };
            (spot.feet, look)
        }
    }
}

/// The docked panels stacked in one column, their heights (the page's
/// measures, CSS pixels separated by spaces) added up with the gaps
/// between them; 0 for anything unreadable.
pub(super) fn stacked_height(measures: &str) -> i32 {
    let heights: Option<Vec<i32>> = measures
        .split_whitespace()
        .map(|h| h.parse::<i32>().ok().filter(|h| (0..10_000).contains(h)))
        .collect();
    match heights {
        Some(hs) if !hs.is_empty() => hs.iter().sum::<i32>() + GAP * (hs.len() as i32 - 1),
        _ => 0,
    }
}

/// Moves the map cards out of the docked panels into a strip of their own
/// above the ape (TRIAGE T82): every map's picture and name in one row.
fn strip_out(v: &mut Value) {
    let Some(panels) = v["panels"].as_array_mut() else {
        return;
    };
    let mut cards = None;
    for p in panels.iter_mut() {
        if let Some(items) = p["items"].as_array_mut() {
            if let Some(i) = items.iter().position(|it| it["type"] == "tiles") {
                cards = Some(items.remove(i));
            }
        }
    }
    if let Some(mut cards) = cards {
        cards["strip"] = json!(true);
        panels.push(json!({ "kind": "strip", "items": [cards] }));
    }
}

/// The preview camera for an ape standing at `feet`, looking `look`.
fn preview_view(feet: V3, look: V3) -> Camera {
    let target = feet.add(v3(0, FOCUS_UP, 0));
    let eye = feet.add(v3(0, EYE_UP, 0)).sub(look.scale(EYE_BACK));
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

/// Where the menus go on a screen that shows the ape: `panels` panels on
/// a `w` x `h` screen (CSS pixels), the page's measure of them stacked
/// `menus_h` high (0 unmeasured). Two panels stack in one column when they
/// fit the height (the map cards left the panels for the strip, TRIAGE
/// T82), else they stand side by side.
fn dock_for(w: i32, h: i32, panels: usize, menus_h: i32) -> Dock {
    let one = PANEL_W + 2 * GAP;
    let two = 2 * PANEL_W + 3 * GAP;
    let stacked = menus_h > 0 && menus_h + 2 * GAP <= h;
    if w > h && w >= two + PANEL_W && panels >= 2 && !stacked {
        Dock::Side(two)
    } else if w > h && w >= one + OPEN_MIN {
        Dock::Side(one)
    } else {
        Dock::Below(fx::mul(h, OPEN_TOP))
    }
}

impl App {
    pub(super) fn dock(&self) -> Dock {
        if self.stage != Stage::Showcase {
            return Dock::Centre;
        }
        dock_for(self.view_w, self.view_h, self.panel_count(), self.menus_h)
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
        if name == "side" {
            strip_out(v);
        }
        v["dock"] = json!(name);
        v["dock_w"] = json!(format!("{w}px"));
        v["dock_top"] = json!(format!("{top}px"));
    }

    fn preview_at(&self) -> (V3, V3) {
        let map = self.map();
        preview_spot(map, self.course_of(map))
    }

    /// The preview camera on the chosen map, framed beside the menus. On
    /// Sunset Ruins the map's own lobby camera (`ruinsscape::preview`: low
    /// and level, the temple's mouth behind the ape, not the ground).
    pub(super) fn preview_camera(&self) -> Camera {
        let map = self.map();
        let (feet, look) = self.preview_at();
        let focus = feet.add(v3(0, FOCUS_UP, 0));
        match (map, self.course_of(map)) {
            (Map::Ruins, Some(c)) => self.ruins_camera(c, focus),
            (Map::Bay, Some(c)) => self.bay_camera(c, focus),
            (Map::Tower, Some(c)) => self.tower_camera(c, focus),
            (Map::Ravine, Some(c)) => self.ravine_camera(c, focus),
            _ => self.framed(preview_view(feet, look), focus),
        }
    }

    /// Banana Bay Battle's lobby camera (`bayscape::lobby`), framed beside
    /// the menus and turned so the fleet's landmark ship sits between the
    /// docked panels and the ape, as on Sunset Ruins.
    fn bay_camera(&self, c: &Course, focus: V3) -> Camera {
        let Some(bay) = c.bay.as_deref() else {
            return self.framed(crate::scene::preview(c).camera, focus);
        };
        let (feet, look) = crate::bayscape::lobby::lobby_spot(bay);
        let mark = crate::bayscape::lobby::landmark(bay);
        // backed off where the open part is narrow (the lobby's two panels
        // at 1366 or 1280 wide), so the ape fits it as it turns
        let h = self.view_h.max(1);
        let open = match self.dock() {
            Dock::Side(dock) => self.view_w - dock,
            _ => self.view_w,
        }
        .max(1);
        let back = crate::bayscape::lobby::LOBBY_BACK.max(fx::mul(BAY_FIT, fx::div(h, open)));
        let cam = move |feet, look| crate::bayscape::lobby::lobby_camera_at(feet, look, back);
        self.turned(feet, look, mark, focus, &cam)
    }

    /// Mape's Tower's lobby camera (`towerscape::lobby`), framed beside the
    /// menus and turned so the keep's edge sits between the docked panels
    /// and the ape: the wall beside the menus, the ape against the sky.
    fn tower_camera(&self, c: &Course, focus: V3) -> Camera {
        let Some(t) = c.tower.as_deref() else {
            return self.framed(crate::scene::preview(c).camera, focus);
        };
        let (feet, look) = crate::towerscape::lobby::lobby_spot(c, t);
        let mark = crate::towerscape::lobby::landmark(c, t);
        let h = self.view_h.max(1);
        let open = match self.dock() {
            Dock::Side(dock) => self.view_w - dock,
            _ => self.view_w,
        }
        .max(1);
        let back = crate::towerscape::lobby::LOBBY_BACK.max(fx::mul(BAY_FIT, fx::div(h, open)));
        let cam = move |feet, look| crate::towerscape::lobby::lobby_camera_at(feet, look, back);
        self.turned(feet, look, mark, focus, &cam)
    }

    /// Rattlesnake Ravine's lobby camera (`ravinescape::lobby`), framed
    /// beside the menus and turned so the first arch sits between the
    /// docked panels and the ape, backed off where the open part is narrow
    /// (the Bay's kind).
    fn ravine_camera(&self, c: &Course, focus: V3) -> Camera {
        let Some(r) = c.ravine.as_deref() else {
            return self.framed(crate::scene::preview(c).camera, focus);
        };
        let (feet, look) = crate::ravinescape::lobby::lobby_spot(r);
        let mark = crate::ravinescape::lobby::landmark(r);
        let h = self.view_h.max(1);
        let open = match self.dock() {
            Dock::Side(dock) => self.view_w - dock,
            _ => self.view_w,
        }
        .max(1);
        let back = crate::ravinescape::lobby::LOBBY_BACK.max(fx::mul(BAY_FIT, fx::div(h, open)));
        let cam = move |feet, look| crate::ravinescape::lobby::lobby_camera_at(feet, look, back);
        self.turned(feet, look, mark, focus, &cam)
    }

    /// Sunset Ruins' lobby camera, framed beside the menus and turned so
    /// the temple's mouth sits between the docked panels and the ape.
    fn ruins_camera(&self, c: &Course, focus: V3) -> Camera {
        let Some(r) = c.ruins.as_deref() else {
            return self.framed(crate::scene::preview(c).camera, focus);
        };
        let (feet, look) = crate::ruinsscape::lobby_spot(c, r);
        let mouth = r.mouth.opening.centre;
        self.turned(feet, look, mouth, focus, &crate::ruinsscape::lobby_camera)
    }

    /// A map's own lobby camera (`lobby`, for an ape at `feet` looking
    /// `look`), framed beside the menus and turned so `mark` sits in the
    /// open part between the docked panels and the ape (RS23, RS28): its
    /// middle halfway between the dock's edge and the ape's left side,
    /// whether one panel or two are docked. The turn is found by a few
    /// secant steps on where the mark lands.
    fn turned(
        &self,
        feet: V3,
        look: V3,
        mark: V3,
        focus: V3,
        lobby: &dyn Fn(V3, V3) -> Camera,
    ) -> Camera {
        let base = yaw_of(look);
        let cam_at = |turn: i32| self.framed(lobby(feet, facing(base + turn, 0)), focus);
        let Dock::Side(dock) = self.dock() else {
            return cam_at(0);
        };
        let (w, h) = (self.view_w.max(1), self.view_h.max(1));
        let ape = dock + (w - dock) / 2;
        let target = fx::div(dock + ape - fx::mul(h, APE_HALF_W), 2 * w) * 100;
        let aspect = fx::div(w, h);
        let miss = |turn: i32| {
            cam_at(turn)
                .screen(aspect, mark)
                .map_or(0, |(x, _)| x - target)
        };
        let (mut t0, mut t1) = (0, deg(3));
        let (mut f0, mut f1) = (miss(t0), miss(t1));
        for _ in 0..4 {
            if f1 == f0 {
                break;
            }
            let t2 = t1 - (f1 as i64 * (t1 - t0) as i64 / (f1 - f0) as i64) as i32;
            let t2 = t2.clamp(-MAX_TURN, MAX_TURN);
            (t0, f0) = (t1, f1);
            (t1, f1) = (t2, miss(t2));
        }
        cam_at(t1)
    }

    /// The preview's moving parts: the vines (on a map's own ground) and
    /// the ape, turning a full circle every 6 s as `models::showcase` does.
    pub(super) fn preview_batches(&self, vivid: bool) -> (Batch, Batch) {
        let map = self.map();
        let mut b = Batch::new();
        if map == Map::Jungle {
            models::showcase_in(
                &mut b,
                self.settings.kind,
                self.settings.palette(),
                self.settings.costume.gear(map),
                self.ticks,
            );
            return (Batch::new(), b);
        }
        let (feet, _) = self.preview_at();
        let yaw = ((self.ticks as i64 % 720) * TURN as i64 / 720) as i32;
        let pose = models::Pose {
            gear: self.settings.costume.gear(map),
            ..models::Pose::standing(feet, yaw, self.ticks)
        };
        models::draw_in(&mut b, self.settings.kind, self.settings.palette(), &pose);
        if let Some(c) = self.course_of(map) {
            crate::scene::backdrop(&mut b, c, self.preview_camera().eye, self.ticks);
        }
        let mut vines = Batch::new();
        if let Some(c) = self.course_of(map) {
            // on Banana Bay Battle the start row is not yet rigged: its
            // ropes would hang a few metres in front of the ape (BL2)
            let keep = |i: usize| match (c.bay.as_deref(), c.tower.as_deref(), c.ravine.as_deref())
            {
                (Some(bay), _, _) => !crate::bayscape::lobby::start_row(c, bay, i),
                // Mape's Tower: the start's beams too (TL1)
                (_, Some(t), _) => !crate::towerscape::lobby::hidden_in_lobby(c, t, i),
                // Rattlesnake Ravine: the lobby bridge's own vines (RL1)
                (_, _, Some(r)) => !crate::ravinescape::lobby::hidden_in_lobby(c, r, i),
                _ => true,
            };
            crate::scene::vines_where(&mut vines, c, self.ticks, vivid, &keep);
        }
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

    fn app_at(screen: Screen, w: i32, h: i32) -> App {
        let mut a = App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap();
        a.resize(w, h);
        a.go(screen);
        a
    }

    /// Where the ape's middle lands, in hundredths of the width and height.
    fn ape_on_screen(a: &App) -> (i32, i32) {
        let cam = a.preview_camera();
        let (feet, _) = a.preview_at();
        let focus = feet.add(v3(0, FOCUS_UP, 0));
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
    fn on_sunset_ruins_the_lobby_camera_looks_past_the_ape_at_the_temple_mouth() {
        // the person (1.10.1): "just looking at the ground"; RS23: in the
        // online lobby the mouth sat behind the Race Setup panel
        for (w, h) in [(1920, 1080), (1366, 768), (1280, 720)] {
            for (name, a) in ruins_screens(w, h) {
                assert_eq!(a.map(), Map::Ruins, "{name}");
                let c = a.course_of(Map::Ruins).unwrap();
                let cam = a.preview_camera();
                // nearly level, not pitched down at the feet
                assert!(
                    cam.fwd.y.abs() < fx::sin(deg(4)),
                    "{name} {w}x{h}: pitch {}",
                    cam.fwd.y
                );
                // the mouth's middle in the open part, between the menus'
                // edge and the ape's left side
                let mouth = c.ruins.as_deref().unwrap().mouth.opening.centre;
                let (mx, my) = cam.screen(fx::div(w, h), mouth).unwrap();
                let (mx, my) = (mx / ONE, my / ONE);
                let (x, y) = ape_on_screen(&a);
                let Dock::Side(dock) = a.dock() else { panic!() };
                let ape_left = x - fx::mul(h, APE_HALF_W) * 100 / w;
                assert!(
                    mx > dock * 100 / w + 2 && mx < ape_left - 2,
                    "{name} {w}x{h}: mouth at {mx}%, menus to {}%, ape from {ape_left}%",
                    dock * 100 / w
                );
                assert!((15..=70).contains(&my), "{name} {w}x{h}: mouth {my}% down");
                let open_middle = (dock + (w - dock) / 2) * 100 / w;
                assert!(
                    (x - open_middle).abs() <= 2,
                    "{name} {w}x{h}: ape at {x}%, open part's middle {open_middle}%"
                );
                assert!((40..=70).contains(&y), "{name} {w}x{h}: ape at {y}% down");
            }
        }
    }

    #[test]
    fn on_banana_bay_battle_the_fleet_shows_between_the_menus_and_the_ape_on_the_pier() {
        for (w, h) in [(1920, 1080), (1366, 768), (1280, 720)] {
            for (name, a) in screens_on("bay", w, h) {
                assert_eq!(a.map(), Map::Bay, "{name}");
                let c = a.course_of(Map::Bay).unwrap();
                let bay = c.bay.as_deref().unwrap();
                let cam = a.preview_camera();
                let aspect = fx::div(w, h);
                assert!(
                    cam.fwd.y.abs() < fx::sin(deg(4)),
                    "{name} {w}x{h}: pitch {}",
                    cam.fwd.y
                );
                // the landmark ship between the menus' edge and the ape
                let mark = crate::bayscape::lobby::landmark(bay);
                let (mx, my) = cam.screen(aspect, mark).unwrap();
                let (mx, my) = (mx / ONE, my / ONE);
                let (x, y) = ape_on_screen(&a);
                let Dock::Side(dock) = a.dock() else { panic!() };
                let ape_left = x - fx::mul(h, APE_HALF_W) * 100 / w;
                assert!(
                    mx > dock * 100 / w + 2 && mx < ape_left - 2,
                    "{name} {w}x{h}: fleet at {mx}%, menus to {}%, ape from {ape_left}%",
                    dock * 100 / w
                );
                assert!((5..=70).contains(&my), "{name} {w}x{h}: fleet {my}% down");
                let open_middle = (dock + (w - dock) / 2) * 100 / w;
                assert!(
                    (x - open_middle).abs() <= 2,
                    "{name} {w}x{h}: ape at {x}%, open part's middle {open_middle}%"
                );
                assert!((40..=70).contains(&y), "{name} {w}x{h}: ape at {y}% down");
                // every kind stands on the planks, wholly in the frame and
                // clear of the menus, as it turns
                let (feet, _) = a.preview_at();
                assert_eq!(feet.y, bay.dock.from.y, "{name}: on the planks");
                let turns = (0..8).map(|k| k * TURN / 8);
                for (kind, yaw) in crate::rules::Kind::ALL
                    .into_iter()
                    .flat_map(|k| turns.clone().map(move |y| (k, y)))
                {
                    let mut b = Batch::new();
                    let pose = models::Pose {
                        gear: models::Gear::on(Map::Bay),
                        ..models::Pose::standing(feet, yaw, 0)
                    };
                    models::draw_in(&mut b, kind, 0, &pose);
                    let (v, _) = b.finish(crate::render::DYNAMIC_BUFFER);
                    for i in v.chunks(crate::render::INSTANCE_VALUES) {
                        let p = v3(i[3], i[7], i[11]);
                        let (px, py) = cam.screen(aspect, p).unwrap();
                        let (px, py) = (px / ONE, py / ONE);
                        assert!(
                            px > dock * 100 / w && px < 100 && (3..97).contains(&py),
                            "{name} {w}x{h} {kind:?}: a part at {px}%, {py}%"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn on_mapes_tower_the_keeps_edge_shows_between_the_menus_and_the_ape_on_the_start_deck() {
        for (w, h) in [(1920, 1080), (1366, 768), (1280, 720)] {
            for (name, a) in screens_on("tower", w, h) {
                assert_eq!(a.map(), Map::Tower, "{name}");
                let c = a.course_of(Map::Tower).unwrap();
                let t = c.tower.as_deref().unwrap();
                let cam = a.preview_camera();
                let aspect = fx::div(w, h);
                assert!(
                    cam.fwd.y.abs() < fx::sin(deg(4)),
                    "{name} {w}x{h}: pitch {}",
                    cam.fwd.y
                );
                // the keep's edge between the menus' edge and the ape
                let mark = crate::towerscape::lobby::landmark(c, t);
                let (mx, my) = cam.screen(aspect, mark).unwrap();
                let (mx, my) = (mx / ONE, my / ONE);
                let (x, y) = ape_on_screen(&a);
                let Dock::Side(dock) = a.dock() else { panic!() };
                let ape_left = x - fx::mul(h, APE_HALF_W) * 100 / w;
                assert!(
                    mx > dock * 100 / w + 2 && mx < ape_left - 2,
                    "{name} {w}x{h}: keep's edge at {mx}%, menus to {}%, ape from {ape_left}%",
                    dock * 100 / w
                );
                assert!((5..=70).contains(&my), "{name} {w}x{h}: edge {my}% down");
                let open_middle = (dock + (w - dock) / 2) * 100 / w;
                assert!(
                    (x - open_middle).abs() <= 2,
                    "{name} {w}x{h}: ape at {x}%, open part's middle {open_middle}%"
                );
                assert!((40..=70).contains(&y), "{name} {w}x{h}: ape at {y}% down");
                // every kind wholly in the frame and clear of the menus as
                // it turns
                let (feet, _) = a.preview_at();
                let turns = (0..8).map(|k| k * TURN / 8);
                for (kind, yaw) in crate::rules::Kind::ALL
                    .into_iter()
                    .flat_map(|k| turns.clone().map(move |y| (k, y)))
                {
                    let mut b = Batch::new();
                    let pose = models::Pose {
                        gear: models::Gear::on(Map::Tower),
                        ..models::Pose::standing(feet, yaw, 0)
                    };
                    models::draw_in(&mut b, kind, 0, &pose);
                    let (v, _) = b.finish(crate::render::DYNAMIC_BUFFER);
                    for i in v.chunks(crate::render::INSTANCE_VALUES) {
                        let p = v3(i[3], i[7], i[11]);
                        let (px, py) = cam.screen(aspect, p).unwrap();
                        let (px, py) = (px / ONE, py / ONE);
                        assert!(
                            px > dock * 100 / w && px < 100 && (3..97).contains(&py),
                            "{name} {w}x{h} {kind:?}: a part at {px}%, {py}%"
                        );
                    }
                }
            }
        }
    }

    /// Race Setup, Time Trial and the online lobby (two panels docked)
    /// on Sunset Ruins.
    fn ruins_screens(w: i32, h: i32) -> Vec<(&'static str, App)> {
        screens_on("ruins", w, h)
    }

    /// Race Setup, Time Trial and the online lobby (two panels docked)
    /// on the map `key`.
    fn screens_on(key: &str, w: i32, h: i32) -> Vec<(&'static str, App)> {
        let mut setup = app_at(Screen::Setup, w, h);
        setup.act("setup.map", key);
        let mut trial = app_at(Screen::Title, w, h);
        trial.act("menu.trial", "");
        trial.act("setup.map", key);
        assert!(trial.trial && trial.screen == Screen::Setup);
        let mut lobby = app_at(Screen::Online, w, h);
        lobby.act("online.host", "");
        lobby.act("lobby.map", key);
        assert_eq!(lobby.screen, Screen::Lobby);
        assert_eq!(lobby.dock(), Dock::Side(2 * PANEL_W + 3 * GAP));
        vec![("setup", setup), ("trial", trial), ("lobby", lobby)]
    }

    #[test]
    fn the_lobby_docks_its_two_panels_side_by_side_when_there_is_room() {
        let two = 2 * PANEL_W + 3 * GAP;
        let one = PANEL_W + 2 * GAP;
        assert_eq!(dock_for(1920, 1080, 2, 0), Dock::Side(two));
        assert_eq!(dock_for(1280, 720, 2, 0), Dock::Side(two));
        assert_eq!(dock_for(1100, 720, 2, 0), Dock::Side(one));
        assert_eq!(dock_for(1920, 1080, 1, 0), Dock::Side(one), "one panel");
    }

    #[test]
    fn the_lobbys_two_panels_stack_in_one_column_when_they_fit_the_height() {
        let two = 2 * PANEL_W + 3 * GAP;
        let one = PANEL_W + 2 * GAP;
        let at = |measures: &str| dock_for(1906, 916, 2, stacked_height(measures));
        assert_eq!(at(""), Dock::Side(two), "unmeasured: side by side");
        // 380 + 24 + 440 = 844, and 24 above and below: fits 916
        assert_eq!(at("380 440"), Dock::Side(one), "fits 916: one column");
        assert_eq!(at("400 460"), Dock::Side(two), "884 + gaps is too tall");
        assert_eq!(at("junk"), Dock::Side(two), "unreadable: side by side");
    }

    /// The app's own wiring of the dock: the lobby counts its two panels,
    /// Race Setup one, and the page's measure of the stacked panels reaches
    /// the dock (the rules themselves: `dock_for` above).
    #[test]
    fn the_app_docks_the_lobbys_two_panels_and_the_setups_one() {
        let two = 2 * PANEL_W + 3 * GAP;
        let one = PANEL_W + 2 * GAP;
        let mut a = app_at(Screen::Online, 1906, 916);
        a.act("online.host", "");
        assert_eq!(a.screen, Screen::Lobby);
        assert_eq!(a.dock(), Dock::Side(two));
        a.act("dock.menus_h", "380 440");
        assert_eq!(a.dock(), Dock::Side(one), "measured to fit: one column");
        a.resize(1100, 720);
        a.act("dock.menus_h", "");
        assert_eq!(a.dock(), Dock::Side(one), "narrower: one column");
        let a = app_at(Screen::Setup, 1920, 1080);
        assert_eq!(a.dock(), Dock::Side(one));
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
