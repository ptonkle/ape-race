//! The apes behind the setup menus (TRIAGE T76): in the online Lobby every
//! seat's ape (its kind and colours, in the map's gear) side by side, with
//! yours on the far left, forward of the rest and on a little pedestal; in
//! the single-player Race Setup only your own ape, on its pedestal. After
//! a race, back in the lobby or the setup, the winner's ape dances (its
//! kind's finish dance, `Pose.dance`) and the rest stand, until the winner
//! changes ape or the next race starts.
//!
//! The row stands at the map's preview spot (`menu_dock::preview_spot`),
//! across the camera's view: each ape on whatever is under it there (the
//! deck, the snow, the street). The camera backs off along its own line
//! until every ape fits in the open part of the screen beside or above the
//! docked menus (`App::framed`).

use super::{App, Screen};
use crate::course::Course;
use crate::dance::Dance;
use crate::fx::{cm, deg, ratio, ONE, TURN};
use crate::geom::{right_of, v3, yaw_of, V3};
use crate::mesh::Mesh;
use crate::models::{self, Gear, Pose};
use crate::race::Race;
use crate::render::{placed, rgb, Batch, Camera, Rgb};
use crate::rules::{Kind, BODY_DROP};

/// How high your pedestal stands, and how wide it is across its corners.
pub(super) const PEDESTAL: i32 = cm(18);
const PEDESTAL_W: i32 = cm(84);
/// How far the pedestal reaches down into the ground (so on a slope it
/// never floats).
const PEDESTAL_SINK: i32 = cm(40);
const PEDESTAL_STONE: Rgb = rgb(226, 220, 206);
const PEDESTAL_GOLD: Rgb = rgb(236, 184, 64);
/// The others stand this far apart across the view, and this far behind
/// yours (yours is brought forward).
const GAP: i32 = cm(95);
const BEHIND: i32 = cm(60);
/// The most an ape stands above or below the spot: further down, it has
/// stepped off the deck or the platform.
pub(super) const DROP_MOST: i32 = cm(60);
/// The others turn this far from facing the camera, toward your ape.
const TURN_IN: i32 = deg(18);

/// The ape that won the last race, as the lobby or the setup knows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Winner {
    /// won online (the Lobby's seats), else offline (the Race Setup)
    pub online: bool,
    pub name: String,
    pub kind: Kind,
    pub palette: u8,
    /// this browser's player
    pub me: bool,
}

/// The first finisher of a race of two apes or more (of two at the same
/// tick, the first entered); None when no ape has finished.
pub(super) fn winner_of(race: &Race, local: usize, online: bool) -> Option<Winner> {
    if race.racers.len() < 2 {
        return None;
    }
    let (k, _) = race
        .racers
        .iter()
        .enumerate()
        .filter_map(|(k, r)| r.finish.map(|f| (k, f)))
        .min_by_key(|(k, f)| (*f, *k))?;
    Some(Winner {
        online,
        name: race.racers[k].name.clone(),
        kind: race.racers[k].ape.kind,
        palette: race.racers[k].palette,
        me: k == local,
    })
}

/// One ape of the lineup: where its feet stand, which ape, and whether it
/// is yours and whether it dances.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Stand {
    pub feet: V3,
    pub kind: Kind,
    pub palette: u8,
    pub me: bool,
    pub dances: bool,
}

/// Where `n` apes stand at a preview spot (`anchor`, seen looking `look`):
/// yours at the spot on top of its pedestal, the rest in a row going back
/// to its right, each a step further back than the last, so the row is
/// narrow on screen. Each stands on what is under it; where the ground
/// ends behind the spot (Sector Ape's deck edge) the steps back shorten
/// and the row spreads wider instead.
pub(super) fn row(course: Option<&Course>, anchor: V3, look: V3, n: usize) -> Vec<V3> {
    let right = right_of(look);
    let at = |i: usize, back: i32| {
        if i == 0 {
            return anchor.add(v3(0, PEDESTAL, 0));
        }
        let across = GAP + (BEHIND - back) / 2;
        let p = anchor
            .add(right.scale(across * i as i32))
            .add(look.scale(back * i as i32));
        let y = course.map_or(anchor.y, |c| {
            c.support(v3(p.x, anchor.y + BODY_DROP + cm(30), p.z))
        });
        v3(p.x, y, p.z)
    };
    let mut back = BEHIND;
    loop {
        let feet: Vec<V3> = (0..n).map(|i| at(i, back)).collect();
        let on = feet.iter().all(|f| (f.y - anchor.y).abs() <= DROP_MOST);
        if on || back <= 0 {
            return feet;
        }
        back -= cm(10);
    }
}

impl App {
    /// The apes to show, yours first: online every seat in the lobby, in
    /// seat order after yours; offline only yours.
    fn seats(&self) -> Vec<(String, Kind, u8, bool)> {
        if self.screen == Screen::Lobby {
            if let Some(o) = &self.online.session {
                let seats = o.lobby().seats;
                let mut out: Vec<_> = seats
                    .iter()
                    .filter(|s| s.me)
                    .chain(seats.iter().filter(|s| !s.me))
                    .map(|s| (s.name.clone(), s.kind, s.palette, s.me))
                    .collect();
                out.truncate(crate::online::MAX_APES);
                if !out.is_empty() {
                    return out;
                }
            }
        }
        vec![(
            String::new(),
            self.settings.kind,
            self.settings.palette(),
            true,
        )]
    }

    /// Whether the seat's ape is the last race's winner, unchanged.
    fn won(&self, name: &str, kind: Kind, palette: u8, me: bool) -> bool {
        let online = self.screen == Screen::Lobby;
        self.winner.as_ref().is_some_and(|w| {
            w.online == online
                && w.kind == kind
                && w.palette == palette
                && if online { w.name == name } else { w.me && me }
        })
    }

    /// The lineup at the chosen map's preview spot.
    pub(super) fn lineup(&self) -> Vec<Stand> {
        let (anchor, look) = self.preview_at();
        let seats = self.seats();
        let course = self.course_of(self.map());
        row(course, anchor, look, seats.len())
            .into_iter()
            .zip(seats)
            .map(|(feet, (name, kind, palette, me))| Stand {
                feet,
                kind,
                palette,
                me,
                dances: self.won(&name, kind, palette, me),
            })
            .collect()
    }

    /// The point the preview camera centres on: the middle of the lineup,
    /// at an ape's middle height.
    pub(super) fn lineup_focus(stands: &[Stand]) -> V3 {
        let n = stands.len().max(1) as i32;
        let sum = stands.iter().fold(V3::ZERO, |a, s| a.add(s.feet));
        let mid = v3(sum.x / n, sum.y / n, sum.z / n);
        mid.add(v3(0, super::menu_dock::FOCUS_UP, 0))
    }

    /// Whether every ape's outline lands inside the open part of the screen.
    pub(super) fn fits(&self, cam: &Camera, stands: &[Stand], right: V3) -> bool {
        let (x0, x1, y0, y1) = self.open_box();
        let aspect = crate::fx::div(self.view_w, self.view_h.max(1));
        stands.iter().all(|s| {
            [-cm(45), cm(45)].iter().all(|side| {
                [cm(0), cm(185)].iter().all(|h| {
                    let p = s.feet.add(right.scale(*side)).add(v3(0, *h, 0));
                    cam.screen(aspect, p)
                        .is_some_and(|(x, y)| (x0..=x1).contains(&x) && (y0..=y1).contains(&y))
                })
            })
        })
    }

    /// The apes and your pedestal.
    pub(super) fn draw_lineup(&self, b: &mut Batch, stands: &[Stand], eye: V3) {
        let gear = Gear::on(self.map());
        for s in stands {
            let face = yaw_of(eye.sub(s.feet).flat());
            let yaw = if s.dances {
                face
            } else if s.me {
                // yours turns a full circle every 6 s, as the showcase did
                ((self.ticks as i64 % 720) * TURN as i64 / 720) as i32
            } else {
                face + TURN_IN
            };
            let pose = Pose {
                gear,
                dance: s.dances.then_some(Dance {
                    beat: self.ticks,
                    weight: ONE,
                }),
                ..Pose::standing(s.feet, yaw, self.ticks)
            };
            models::draw_in(b, s.kind, s.palette, &pose);
            if s.me {
                pedestal(b, s.feet);
            }
        }
    }
}

/// Your pedestal under feet at `top`: a stone hexagon with a gold band
/// round its top.
fn pedestal(b: &mut Batch, top: V3) {
    let h = PEDESTAL + PEDESTAL_SINK;
    let base = top.sub(v3(0, h / 2, 0));
    b.push(
        Mesh::Hex,
        &placed(base, 0, v3(PEDESTAL_W, h, PEDESTAL_W)),
        PEDESTAL_STONE,
        0,
    );
    let band = cm(5);
    b.push(
        Mesh::Hex,
        &placed(
            top.sub(v3(0, band / 2 + cm(1), 0)),
            0,
            v3(PEDESTAL_W + cm(6), band, PEDESTAL_W + cm(6)),
        ),
        PEDESTAL_GOLD,
        ratio(1, 5),
    );
}

#[cfg(test)]
mod tests {
    use super::super::Screen;
    use super::*;
    use crate::map::Map;

    fn app(w: i32, h: i32) -> App {
        let mut a = App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap();
        a.resize(w, h);
        a
    }

    /// The host of a room with three AI apes, on `map`.
    fn room_of_four(map: Map, w: i32, h: i32) -> App {
        let mut a = app(w, h);
        a.act("menu.online", "");
        a.act("online.host", "");
        a.act("lobby.ai", "3");
        a.act("lobby.map", map.key());
        assert_eq!(a.screen, Screen::Lobby);
        assert_eq!(a.map(), map);
        a
    }

    /// Where a point lands across the screen, in hundredths of the width.
    fn across(a: &App, p: V3) -> i32 {
        let cam = a.preview_camera();
        cam.screen(crate::fx::div(a.view_w, a.view_h), p).unwrap().0 / ONE
    }

    #[test]
    fn every_map_has_room_for_four_apes_standing_round_its_lobby_spot() {
        for map in Map::ALL {
            let a = room_of_four(map, 1280, 720);
            let c = a.course_of(map).unwrap();
            let spot = crate::scene::preview(c);
            let stands = a.lineup();
            assert_eq!(stands.len(), 4, "{map:?}");
            assert!(stands[0].me, "{map:?}: yours first");
            assert_eq!(
                stands[0].feet,
                spot.feet.add(v3(0, PEDESTAL, 0)),
                "{map:?}: on its pedestal"
            );
            for (i, s) in stands.iter().enumerate().skip(1) {
                assert!(!s.me);
                // on the ground or the deck there, not fallen off it
                let under = c.support(s.feet.add(v3(0, BODY_DROP, 0)));
                assert_eq!(under, s.feet.y, "{map:?}: ape {i} stands on something");
                assert!(
                    (s.feet.y - spot.feet.y).abs() <= DROP_MOST,
                    "{map:?}: ape {i} {} cm from the spot's height",
                    (s.feet.y - spot.feet.y) * 100 / ONE
                );
            }
        }
    }

    #[test]
    fn near_a_deck_edge_the_row_steps_back_less_and_stays_on_the_deck() {
        let c = Map::Sector.course();
        let (spot, look, _) = super::super::menu_dock::preview_spot(Some(&c));
        let on = |p: V3| (c.support(p.add(v3(0, BODY_DROP, 0))) - spot.y).abs() <= DROP_MOST;
        // out toward the edge behind the spot, to 1.2 m short of it
        let mut edge = spot;
        while on(edge.add(look.scale(cm(10)))) {
            edge = edge.add(look.scale(cm(10)));
        }
        let anchor = edge.sub(look.scale(cm(120)));
        let feet = row(Some(&c), anchor, look, 4);
        for (i, f) in feet.iter().enumerate().skip(1) {
            assert!(on(*f), "ape {i} stepped off the deck");
        }
    }

    #[test]
    fn the_jungle_lobby_stands_on_the_start_platform_not_the_forest_floor() {
        let mut a = app(1280, 720);
        a.act("menu.race", "");
        let c = a.course_of(Map::Jungle).unwrap();
        let deck = c.platforms[0];
        let (feet, _) = a.preview_at();
        assert_eq!(feet.y, deck.top);
        assert!(deck.top > c.floor(feet.x, feet.z) + crate::fx::int(10));
        assert_eq!(a.shown, super::super::Shown::World(Map::Jungle));
    }

    #[test]
    fn four_apes_fit_beside_the_docked_menus_yours_far_left_and_nearest() {
        for map in Map::ALL {
            for (w, h) in [(1280, 720), (1100, 720), (1920, 1080), (1366, 768)] {
                let a = room_of_four(map, w, h);
                let stands = a.lineup();
                let cam = a.preview_camera();
                let (_, look) = a.preview_at();
                assert!(a.fits(&cam, &stands, right_of(look)), "{map:?} {w}x{h}");
                let mine = across(&a, stands[0].feet);
                let near = stands[0].feet.sub(cam.eye).len();
                for s in &stands[1..] {
                    assert!(across(&a, s.feet) > mine, "{map:?} {w}x{h}: yours far left");
                    assert!(
                        s.feet.sub(cam.eye).len() > near,
                        "{map:?} {w}x{h}: yours in front"
                    );
                }
            }
        }
    }

    #[test]
    fn the_race_setup_shows_only_your_ape_on_its_pedestal() {
        let mut a = app(1280, 720);
        a.act("menu.race", "");
        a.act("setup.ape", "gibbon");
        let stands = a.lineup();
        assert_eq!(stands.len(), 1, "the AI opponents are not shown");
        assert!(stands[0].me);
        assert_eq!(stands[0].kind, Kind::Gibbon);
        let (feet, _) = a.preview_at();
        assert_eq!(stands[0].feet, feet.add(v3(0, PEDESTAL, 0)));
        // the pedestal is drawn under it: one ape's shapes and two more
        let mut b = Batch::new();
        a.draw_lineup(&mut b, &stands, a.preview_camera().eye);
        let mut one = Batch::new();
        let pose = Pose::standing(stands[0].feet, 0, a.ticks);
        models::draw_in(&mut one, Kind::Gibbon, stands[0].palette, &pose);
        assert_eq!(b.count(), one.count() + 2);
    }

    /// A single-player race on `a`, finished with ape `first` the winner.
    fn race_won_by(a: &mut App, first: usize) {
        a.act("menu.race", "");
        a.act("menu.start", "");
        let p = a.play.as_mut().unwrap();
        assert!(p.race.racers.len() >= 2);
        p.race.racers[first].finish = Some(900);
        p.race.racers[1 - first].finish = Some(950);
    }

    #[test]
    fn after_a_race_you_won_your_ape_dances_in_the_setup_until_you_change_ape() {
        let mut a = app(1280, 720);
        race_won_by(&mut a, 0);
        a.act("menu.main", "");
        a.act("menu.race", "");
        assert!(a.lineup()[0].dances);
        let kind = a.settings.kind;
        let other = Kind::ALL.into_iter().find(|k| *k != kind).unwrap();
        a.act("setup.ape", other.key());
        assert!(!a.lineup()[0].dances, "a new ape does not dance");
        a.act("setup.ape", kind.key());
        assert!(a.lineup()[0].dances, "the winning ape again");
        // the next race ends it
        a.act("menu.start", "");
        a.act("menu.main", "");
        a.act("menu.race", "");
        assert!(!a.lineup()[0].dances);
        // an AI won: yours does not dance
        race_won_by(&mut a, 1);
        a.act("menu.main", "");
        a.act("menu.race", "");
        assert!(!a.lineup()[0].dances);
    }

    #[test]
    fn after_an_online_race_the_winners_seat_dances_and_the_rest_stand() {
        let mut a = room_of_four(Map::Jungle, 1280, 720);
        a.act("lobby.start", "");
        let mut now = 0;
        while a.play.is_none() && now < 2_000_000 {
            now += 10_000;
            a.advance(now);
        }
        let p = a.play.as_mut().expect("the race started");
        assert_eq!(p.race.racers.len(), 4);
        p.race.racers[2].finish = Some(800);
        p.race.racers[0].finish = Some(820);
        let name = p.race.racers[2].name.clone();
        a.act("menu.rematch", "");
        assert_eq!(a.screen, Screen::Lobby);
        let stands = a.lineup();
        let seats = a.online.session.as_ref().unwrap().lobby().seats;
        let dancing: Vec<usize> = (0..4).filter(|i| stands[*i].dances).collect();
        assert_eq!(dancing, vec![2], "the winner alone");
        assert_eq!(seats[2].name, name, "the winner's own seat");
        assert_eq!(
            (stands[2].kind, stands[2].palette),
            (seats[2].kind, seats[2].palette)
        );
        // the next race starts: the dance is over
        a.act("lobby.start", "");
        for _ in 0..50 {
            now += 10_000;
            a.advance(now);
        }
        assert!(a.winner.is_none());
    }
}
