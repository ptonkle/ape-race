//! Sector Ape's model gallery (2026-10-13): `?gallery=<spec>` on the page
//! shows one of `spacecraft.rs`'s models (or the space jaguar) slowly
//! turning on a starfield, lit from below as the space map will be, with
//! no menus, for `scripts/shot.py` to screenshot before any map uses them.
//!
//! The spec is the model's key, then optional `key=value` pairs, joined by
//! commas: `yaw` and `pitch` (the camera's, degrees from the model's front;
//! a positive pitch looks up from below), `dist` (metres; else framed to the
//! model), `at` (the tick to start at), `hold` (`on`: stay at `at`) and `x`, `y`,
//! `z` (metres: the camera's target, off the model's middle) and, for
//! `cat`, `map` (whose big cat: `map=ruins` is the pith-helmeted jaguar,
//! `map=bay` the navy officer; `flame=on`: Mape's Tower's flame cat).
//! E.g. `?gallery=wreck,yaw=40,pitch=12,at=300,hold=on`.

use crate::baymodels as bay;
use crate::fx::{self, deg, int, mul, ratio, ONE, TURN};
use crate::geom::{facing, v3, Affine, V3};
use crate::models;
use crate::ravinemodels as ravine;
use crate::render::{self, Batch, Camera};
use crate::ruinsmodels as ruins;
use crate::spacecraft::{self as sc, Gait, Mech, MechKind};
use crate::towermodels as tower;

use super::App;

/// The models the gallery shows, by key.
pub const MODELS: [&str; 22] = [
    "mothership",
    "wreck",
    "grunt",
    "brute",
    "drift",
    "apewing",
    "enemy",
    "dogfight",
    "battle",
    "chunk",
    "girder",
    "asteroid",
    "debris",
    "blackhole",
    "sun",
    "planet",
    "cat",
    "armada",
    "hulk",
    "deck",
    "station",
    "tether",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gallery {
    pub model: usize,
    pub yaw: i32,
    pub pitch: i32,
    pub dist: Option<i32>,
    pub at: u32,
    pub hold: bool,
    /// moves the camera's target off the model's middle (metres)
    pub aim: V3,
    /// a Sunset Ruins model instead (`ruinsmodels::GALLERY`), drawn under
    /// the sunset
    pub ruins: Option<usize>,
    /// a Banana Bay Battle model instead (`baymodels::GALLERY`), drawn
    /// under the 3 pm sun on a patch of sea
    pub bay: Option<usize>,
    /// a Mape's Tower model instead (`towermodels::GALLERY`), drawn in
    /// the twilight
    pub tower: Option<usize>,
    /// a Rattlesnake Ravine model instead (`ravinemodels::GALLERY`), drawn
    /// at night
    pub ravine: Option<usize>,
    /// `cat`: the map whose big cat it is (`map=ruins`; else Sector Ape's)
    pub map: crate::map::Map,
    /// `cat`: Mape's Tower's flame cat (`flame=on`), whatever `map` says
    pub flame: bool,
}

impl Gallery {
    pub fn parse(spec: &str) -> Gallery {
        let mut g = Gallery {
            model: 0,
            yaw: deg(35),
            pitch: deg(10),
            dist: None,
            at: 0,
            hold: false,
            aim: V3::ZERO,
            ruins: None,
            bay: None,
            tower: None,
            ravine: None,
            map: crate::map::Map::Sector,
            flame: false,
        };
        for (i, pair) in spec.split(',').enumerate() {
            if i == 0 {
                g.model = MODELS.iter().position(|m| *m == pair).unwrap_or(0);
                g.ruins = ruins::GALLERY.iter().position(|m| *m == pair);
                g.bay = bay::GALLERY.iter().position(|m| *m == pair);
                g.tower = tower::GALLERY.iter().position(|m| *m == pair);
                g.ravine = ravine::GALLERY.iter().position(|m| *m == pair);
                continue;
            }
            let Some((k, v)) = pair.split_once('=') else {
                continue;
            };
            let n = v.parse::<i32>().ok();
            match k {
                "yaw" => g.yaw = n.map(deg).unwrap_or(g.yaw),
                "pitch" => g.pitch = n.map(deg).unwrap_or(g.pitch),
                "dist" => g.dist = n.map(int).or(g.dist),
                "at" => g.at = v.parse().unwrap_or(g.at),
                "hold" => g.hold = v == "on",
                "flame" => g.flame = v == "on",
                "map" => g.map = crate::map::Map::from_key(v).unwrap_or(g.map),
                "x" => g.aim.x = n.map(int).unwrap_or(g.aim.x),
                "y" => g.aim.y = n.map(int).unwrap_or(g.aim.y),
                "z" => g.aim.z = n.map(int).unwrap_or(g.aim.z),
                _ => {}
            }
        }
        g
    }

    fn key(&self) -> &'static str {
        MODELS[self.model]
    }

    /// The model's middle and its rough radius, metres.
    fn framing(&self) -> (V3, i32) {
        if let Some(r) = self.ruins {
            return ruins::framing(ruins::GALLERY[r]);
        }
        if let Some(r) = self.bay {
            return bay::framing(bay::GALLERY[r]);
        }
        if let Some(r) = self.ravine {
            return ravine::framing(ravine::GALLERY[r]);
        }
        if let Some(r) = self.tower {
            return tower::framing(tower::GALLERY[r]);
        }
        match self.key() {
            "mothership" => (V3::ZERO, int(125)),
            "wreck" => (V3::ZERO, int(135)),
            "grunt" | "drift" => (V3::ZERO, int(8)),
            "brute" => (V3::ZERO, int(12)),
            "apewing" | "enemy" => (V3::ZERO, int(7)),
            "dogfight" => (V3::ZERO, int(70)),
            "battle" => (V3::ZERO, int(150)),
            "chunk" | "asteroid" => (V3::ZERO, int(8)),
            "girder" => (V3::ZERO, int(13)),
            "debris" => (V3::ZERO, int(22)),
            "blackhole" => (V3::ZERO, int(250)),
            "sun" => (V3::ZERO, int(90)),
            "planet" => (V3::ZERO, int(120)),
            "cat" if self.flame => (v3(0, fx::cm(100), 0), int(2)),
            "cat" => (v3(0, fx::cm(60), 0), int(1)),
            "hulk" => (V3::ZERO, int(110)),
            "deck" => (V3::ZERO, int(30)),
            "station" => (V3::ZERO, int(150)),
            "tether" => (v3(0, int(6), 0), int(26)),
            _ => (v3(0, 0, int(20)), int(40)),
        }
    }

    /// The camera: still, looking at the model's middle.
    pub fn camera(&self) -> Camera {
        let (mid, r) = self.framing();
        let mid = mid.add(self.aim);
        let dist = self.dist.unwrap_or(mul(r, ratio(17, 10)));
        let fwd = facing(fx::wrap(self.yaw + TURN / 2), self.pitch);
        Camera {
            eye: mid.sub(fwd.scale(dist)),
            fwd,
            fov: deg(45),
        }
    }

    /// The model, turning a full circle every 40 s, posed at `tick`.
    pub fn draw(&self, b: &mut Batch, tick: u32) {
        let turn = (tick as i32).wrapping_mul(TURN / (crate::rules::TICKS_PER_SECOND * 40));
        let f = Affine::rot_y(turn);
        let grunt = Mech {
            kind: MechKind::Grunt,
            gait: Gait::March,
            phase: 0,
        };
        if let Some(r) = self.ruins {
            return ruins::gallery(b, ruins::GALLERY[r], &f, tick);
        }
        if let Some(r) = self.bay {
            return bay::gallery(b, bay::GALLERY[r], &f, tick);
        }
        if let Some(r) = self.ravine {
            return ravine::gallery(b, ravine::GALLERY[r], &f, tick);
        }
        if let Some(r) = self.tower {
            return tower::gallery(b, tower::GALLERY[r], &f, tick);
        }
        match self.key() {
            "mothership" => sc::mothership(b, &f, tick),
            "hulk" | "deck" | "station" | "tether" => {
                crate::sectorscape::gallery(b, self.key(), &f, tick)
            }
            "wreck" => sc::wreck(b, &f, tick, sc::WRECK_GAP),
            "grunt" => sc::mech(b, &f, &grunt, tick),
            "brute" => sc::mech(
                b,
                &f,
                &Mech {
                    kind: MechKind::Brute,
                    ..grunt
                },
                tick,
            ),
            "drift" => sc::mech(
                b,
                &f,
                &Mech {
                    gait: Gait::Drift,
                    ..grunt
                },
                tick,
            ),
            "apewing" => sc::ape_wing(b, &f, tick),
            "enemy" => sc::enemy_fighter(b, &f, tick),
            "dogfight" => sc::dogfight(b, &Affine::IDENTITY, tick),
            "battle" => sc::capital_battle(b, &Affine::IDENTITY, tick),
            "chunk" => sc::hull_chunk(b, &f, tick, 0),
            "girder" => sc::girder(b, &f, 24),
            "asteroid" => sc::asteroid(b, &f, 1),
            "debris" => {
                sc::hull_chunk(b, &f.then(&Affine::translate(v3(-int(14), 0, 0))), tick, 1);
                sc::girder(b, &f.then(&Affine::translate(v3(0, int(2), 0))), 18);
                sc::asteroid(b, &f.then(&Affine::translate(v3(int(14), 0, 0))), 2);
            }
            "blackhole" => sc::black_hole(
                b,
                &Affine::rot_x(-deg(10)).then(&Affine::rot_z(deg(8))),
                int(20),
                tick,
            ),
            "sun" => sc::sun(b, V3::ZERO, int(40), tick),
            "planet" => sc::planet(b, V3::ZERO, int(40), tick),
            "cat" => {
                let j = crate::jaguar::Jaguar {
                    feet: V3::ZERO,
                    yaw: turn,
                    gait: (tick as i32).wrapping_mul(500),
                    stride: ONE,
                    leap: 0,
                    head_down: 0,
                    jaw: ONE / 2,
                };
                let cat = if self.flame {
                    models::FLAME_CAT
                } else {
                    models::BigCat::on(self.map)
                };
                models::jaguar(b, &j, 100, &cat);
            }
            // a line of the armada: grunts marching, a brute drifting
            _ => {
                for k in 0..4 {
                    let at = v3(int(k * 18 - 27), 0, int((k % 2) * 14));
                    let m = Mech {
                        phase: k * 9_000,
                        ..grunt
                    };
                    sc::mech(b, &Affine::translate(at), &m, tick);
                }
                let brute = Mech {
                    kind: MechKind::Brute,
                    gait: Gait::Drift,
                    phase: 0,
                };
                sc::mech(b, &Affine::translate(v3(0, int(16), int(40))), &brute, tick);
            }
        }
    }
}

impl App {
    /// Shows the gallery `spec` describes (no menus, no race).
    pub(super) fn gallery(&mut self, spec: &str) {
        let g = Gallery::parse(spec);
        self.ticks = g.at;
        self.gallery = Some(g);
        self.play = None;
        self.sounds.clear();
    }

    /// The gallery's frame: its view values, instances and draws.
    pub(super) fn gallery_frame(&self, g: &Gallery) -> (Vec<i32>, Vec<i32>, Vec<u32>) {
        let cam = g.camera();
        let look = if g.ruins.is_some() {
            ruins::look()
        } else if g.bay.is_some() {
            bay::gallery_look()
        } else if g.ravine.is_some() {
            ravine::gallery_look()
        } else if let Some(r) = g.tower {
            tower::gallery_look_of(tower::GALLERY[r])
        } else {
            sc::look()
        };
        let view = render::view_values(
            &cam,
            self.aspect(),
            // no fog, to see the colours (the map will have its own)
            (int(900), int(1000)),
            (look.sky, look.fog),
            crate::sky::world(0, 0),
            0,
            (look.light_dir, look.light, look.ground),
            look.rim,
            look.far,
        );
        // nothing is cut in the gallery (SECOND-ORDER BX2)
        let mut view = view;
        view.extend(crate::see::values(None));
        let mut b = Batch::new();
        if g.ruins.is_some() {
            ruins::sky(&mut b, cam.eye, ruins::SUN_YAW);
        } else if g.bay.is_some() || g.tower.is_some() || g.ravine.is_some() {
            // a clear sky: the clear colour alone
        } else {
            sc::stars(&mut b, cam.eye, 500);
        }
        g.draw(&mut b, self.ticks);
        let halos = b.take_halos();
        let mut values = Vec::new();
        let mut draws = b.finish_into(render::DYNAMIC_BUFFER, &mut values);
        draws.extend(halos.finish_as(render::DYNAMIC_BUFFER, Some(render::HALO), &mut values));
        (view, values, draws)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ravine_models_have_their_own_keys_and_parse() {
        for (i, key) in ravine::GALLERY.iter().enumerate() {
            assert!(!MODELS.contains(key), "{key}");
            assert_eq!(Gallery::parse(key).ravine, Some(i), "{key}");
        }
    }

    #[test]
    fn a_gallery_spec_picks_its_model_and_camera() {
        let g = Gallery::parse("wreck,yaw=90,pitch=-5,dist=300,at=60,hold=on,bogus=1");
        assert_eq!(g.key(), "wreck");
        assert_eq!((g.yaw, g.pitch, g.dist), (deg(90), -deg(5), Some(int(300))));
        assert_eq!((g.at, g.hold), (60, true));
        assert_eq!(Gallery::parse("nonsense").key(), "mothership");
        // every model draws something
        for k in 0..MODELS.len() {
            let g = Gallery {
                model: k,
                ..Gallery::parse("")
            };
            let mut b = Batch::new();
            g.draw(&mut b, 100);
            assert!(b.count() > 0, "{}", MODELS[k]);
        }
    }

    #[test]
    fn the_gallery_draws_no_menus_and_holds_when_asked() {
        let mut a = App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap();
        a.act("screen.gallery", "grunt,at=50,hold=on");
        let v: serde_json::Value = serde_json::from_str(&a.ui()).unwrap();
        assert_eq!(v["panels"], serde_json::Value::Array(Vec::new()));
        let mut t = 0;
        for _ in 0..10 {
            t += 16_667;
            a.advance(t);
        }
        assert_eq!(a.ticks, 50);
        let (_, values, draws) = a.frame();
        assert!(!values.is_empty() && !draws.is_empty());
    }
}
