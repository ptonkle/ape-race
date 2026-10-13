//! Drawing the City (2026-10-12, group Y): New York at dusk, drawn from
//! the course's `CityLayout` (`city.rs`) and the shared course data, in the
//! faceted low-poly style (TRIAGE T3). Streets and sidewalks, buildings by
//! style with lit windows (brownstones, glass towers, art deco, brick
//! lofts, and a Times-Square stretch of billboards), rooftop water towers
//! and fire escapes, a skyline beyond the streets with an Empire-State-like
//! tower, the river and its embankments, a Brooklyn-Bridge-style
//! suspension bridge, parked cars and cabs, cranes, street lights and the
//! wires the vines hang from, the finish gantry and a plaza past it.
//!
//! The scene (`scene.rs`) calls in here for a course with a `CityLayout`;
//! the Jungle's drawing does not change.

use crate::city::{Block, Bridge, Car, CityLayout, Hanger, HangerKind, River, CABLES};
use crate::course::{ground, Course, OUTRO, STEP};
use crate::fx::{self, cm, deg, int, mul, sin, ONE};
use crate::geom::{right_of, v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{placed, rgb, rod, shade, Batch, Camera, Rgb};
use crate::scene::{self, Look, Scenery, CHUNK};
use crate::vehicles;

/// The City at dusk: a rose and violet haze low down, a deep blue from
/// above, a low orange sun, and the fog far enough out for the skyline.
pub fn look() -> Look {
    Look {
        sky: rgb(84, 92, 150),
        fog: FOG,
        fog_range: (int(80), int(290)),
        light_dir: v3(-cm(62), cm(30), cm(72)).norm(),
        light: rgb(255, 156, 96),
        ground: rgb(70, 58, 72),
    }
}

/// The City's look for its course: the light comes from its low sun
/// (`sun_dir`).
pub fn look_for(c: &Course) -> Look {
    let l = look();
    Look {
        light_dir: sun_dir(c)
            .flat()
            .norm()
            .scale(ONE * 95 / 100)
            .add(v3(0, cm(30), 0))
            .norm(),
        ..l
    }
}

/// Toward the setting sun: just off the course's heading over the river (so
/// it is in view crossing the bridge, its glow on the open water beside the
/// deck), 3° over the horizon.
pub fn sun_dir(c: &Course) -> V3 {
    let at = c
        .city
        .as_ref()
        .and_then(|l| l.river)
        .map_or(c.length / 2, |r| (r.from + r.to) / 2);
    let yaw = yaw_of(c.heading_at(at)) + deg(22);
    crate::geom::facing(yaw, deg(3))
}

/// How far out the sun and its halo are drawn (inside the far plane).
const SUN_AT: i32 = int(520);

/// The low sun: a glowing disc on the horizon with rings of warm haze
/// round it, kept at the same place in the sky wherever the eye is (drawn
/// round the eye each frame, unfogged: `glow` over ONE), and its glitter
/// across the river's water toward the eye.
pub fn sun(b: &mut Batch, c: &Course, eye: V3) {
    let dir = sun_dir(c);
    let face = |at: V3, r: i32| Affine {
        c0: right_of(dir).scale(r * 2),
        c1: dir.scale(cm(20)),
        c2: right_of(dir).cross(dir).norm().scale(r * 2),
        t: at,
    };
    let rings = [
        (int(30), rgb(255, 236, 170), 0),
        (int(46), rgb(255, 170, 90), cm(200)),
        (int(74), rgb(236, 120, 96), cm(400)),
        (int(118), rgb(184, 104, 120), cm(600)),
    ];
    for (r, colour, back) in rings {
        let at = eye.add(dir.scale(SUN_AT + back));
        b.push(Mesh::Cyl, &face(at, r), colour, ONE * 2);
    }
    // the glitter: short bright strips on the water along the line to it
    let Some(r) = c.city.as_ref().and_then(|l| l.river) else {
        return;
    };
    let f = RiverFrame::of(c, Some(r));
    let flat = dir.flat().norm();
    let mut t = int(12);
    let mut k = 0;
    while t < int(280) {
        let p = eye.flat().add(flat.scale(t));
        if f.river_here(c, p) {
            let w = cm(120) + t / 25;
            let len = int(4) + t / 14;
            let side = right_of(flat).scale(mul(sin(k * 9000), w));
            let at = v3(p.x, r.level + cm(3), p.z).add(side);
            let colour = if k % 2 == 0 {
                rgb(255, 200, 120)
            } else {
                rgb(255, 150, 90)
            };
            b.push(
                Mesh::Cube,
                &panel(at, right_of(flat), V3::UP, flat, w, cm(2), len),
                colour,
                ONE,
            );
        }
        t += int(4) + t / 16;
        k += 1;
    }
}

/// The dusk haze: the far colour, and the clear colour behind everything.
pub const FOG: Rgb = rgb(132, 96, 132);

const ASPHALT: Rgb = rgb(56, 56, 64);
const LANE: Rgb = rgb(214, 180, 70);
const ZEBRA: Rgb = rgb(196, 196, 190);
const SIDEWALK: Rgb = rgb(118, 114, 110);
const PAVER: [Rgb; 2] = [rgb(150, 138, 124), rgb(128, 120, 112)];
const RIVERBED: Rgb = rgb(36, 40, 40);
const WATER: Rgb = rgb(32, 50, 84);
const STONE: Rgb = rgb(150, 134, 110);
const GRANITE: Rgb = rgb(178, 158, 126);
const STEEL: Rgb = rgb(56, 62, 70);
const CABLE: Rgb = rgb(70, 74, 82);
/// The bridge's suspenders and stays: thin, and near the dusk haze's own
/// colour, so they sit back behind the vines.
const SUSPENDER: Rgb = rgb(92, 76, 100);
const POLE: Rgb = rgb(44, 58, 52);
const LAMP: Rgb = rgb(255, 214, 140);
const CRANE: Rgb = rgb(236, 176, 30);
const CONCRETE: Rgb = rgb(140, 138, 134);
const RED_LIGHT: Rgb = rgb(255, 40, 30);
const GREEN_LIGHT: Rgb = rgb(60, 255, 120);
const AMBER: Rgb = rgb(255, 170, 40);
const WOOD_TANK: Rgb = rgb(112, 82, 58);
const BANNER: Rgb = rgb(240, 236, 220);
const BANNER_DARK: Rgb = rgb(40, 40, 46);
const GOLD: Rgb = rgb(250, 200, 60);
const SILVER: Rgb = rgb(206, 212, 224);
const BRONZE: Rgb = rgb(206, 128, 70);
const BANANA: Rgb = rgb(255, 214, 56);

/// Windows lit at dusk: warm, pale, cool, amber.
const LIT: [Rgb; 4] = [
    rgb(255, 210, 130),
    rgb(255, 236, 196),
    rgb(206, 224, 255),
    rgb(255, 184, 100),
];
const DARK_GLASS: Rgb = rgb(34, 44, 64);
const BILLBOARDS: [Rgb; 7] = [
    rgb(255, 40, 120),
    rgb(40, 200, 255),
    rgb(255, 220, 40),
    rgb(120, 60, 255),
    rgb(255, 110, 30),
    rgb(60, 255, 140),
    rgb(255, 255, 255),
];
const AWNINGS: [Rgb; 4] = [
    rgb(150, 30, 40),
    rgb(30, 90, 60),
    rgb(30, 50, 110),
    rgb(160, 110, 40),
];

/// The looks the scene gives buildings, from their `Block.style`
/// (`city::style`): brownstones, brick mid-rises and lofts, art deco
/// stone offices, glass towers, riverside warehouses, a building site's
/// hoarding, and the grand terminal closing the plaza.
const BROWNSTONE: u8 = 0;
const GLASS: u8 = 1;
const DECO: u8 = 2;
const LOFT: u8 = 3;
const WAREHOUSE: u8 = 4;
const HOARDING: u8 = 5;
const TERMINAL: u8 = 6;

fn look_of(style: u8) -> u8 {
    use crate::city::style as s;
    match style {
        s::BROWNSTONE => BROWNSTONE,
        s::OFFICE => DECO,
        s::TOWER => GLASS,
        s::WAREHOUSE => WAREHOUSE,
        s::HOARDING => HOARDING,
        s::TERMINAL => TERMINAL,
        _ => LOFT,
    }
}

/// The Times Square stretch: buildings this far along the course (Midtown,
/// the second street) wear billboards up their lower floors.
const TIMES_FROM: i32 = int(296);
const TIMES_TO: i32 = int(512);

// ---- the output: shapes sorted into the course's chunks ----

/// Shapes going into the scenery's chunks, by where they stand.
struct Sink<'a> {
    c: &'a Course,
    n: usize,
    batches: Vec<Batch>,
}

impl Sink<'_> {
    fn at(&mut self, p: V3) -> &mut Batch {
        let k = scene::chunk_of(self.c, p, self.n);
        &mut self.batches[k]
    }
}

/// A box (or any unit mesh) centred at `at`, `w` along `across`, `h` along
/// `up` and `d` along `out`.
fn panel(at: V3, across: V3, up: V3, out: V3, w: i32, h: i32, d: i32) -> Affine {
    Affine {
        c0: across.scale(w),
        c1: up.scale(h),
        c2: out.scale(d),
        t: at,
    }
}

fn up(h: i32) -> V3 {
    v3(0, h, 0)
}

fn on_ground(p: V3) -> V3 {
    v3(p.x, ground(p.x, p.z), p.z)
}

/// A fixed choice from a seed: a whole number in `[0, n)`.
fn pick(seed: u64, n: u32) -> u32 {
    fx::Dice(seed).below(n)
}

fn seed_of(a: usize, b: usize) -> u64 {
    (a as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (b as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
}

// ---- the scenery ----

/// The City's scenery and ground, chunked like the Jungle's.
pub fn scenery(c: &Course, city: &CityLayout) -> Scenery {
    build(c, city).0
}

/// How many scenery shapes each part of the City takes (for
/// `examples/scenery_cost.rs`).
pub fn parts(c: &Course) -> Vec<(&'static str, usize)> {
    match &c.city {
        Some(city) => build(c, city).1,
        None => Vec::new(),
    }
}

fn build(c: &Course, city: &CityLayout) -> (Scenery, Vec<(&'static str, usize)>) {
    let n = scene::chunk_count(c);
    let mut s = Sink {
        c,
        n,
        batches: (0..n).map(|_| Batch::new()).collect(),
    };
    let mut parts = Vec::new();
    let mut done = 0;
    let mut tally = |s: &Sink, name: &'static str| {
        let now: usize = s.batches.iter().map(|b| b.count()).sum();
        parts.push((name, now - done));
        done = now;
    };
    for (k, bl) in city.blocks.iter().enumerate() {
        building(&mut s, bl, k);
    }
    tally(&s, "buildings");
    // the parked cars (the moving ones are drawn each frame, `cars_dynamic`)
    for (k, car) in city.cars.iter().enumerate() {
        if car.drive.is_some() {
            continue;
        }
        let at = on_ground(car.at);
        vehicles::car(s.at(at), car.kind, &car_frame(c, car, at, car.yaw), k);
    }
    for boat in &city.boats {
        let top = c.mushrooms.get(boat.bouncer).map(|m| m.top.y - boat.at.y);
        let scale = match top {
            Some(h) if h > 0 => fx::div(h, vehicles::boat_top(boat.kind)).clamp(ONE / 3, ONE * 2),
            _ => ONE,
        };
        vehicles::boat(s.at(boat.at), boat.kind, boat.at, boat.yaw, scale);
    }
    tally(&s, "cars and boats");
    let frame = RiverFrame::of(c, city.river);
    if let Some(r) = city.river {
        river(&mut s, c, &frame, r, city.bridge.as_ref());
    }
    if let Some(br) = &city.bridge {
        bridge(&mut s, c, br, city.river);
    }
    tally(&s, "river and bridge");
    for (k, h) in city.hangers.iter().enumerate() {
        hanger(&mut s, c, h, k);
    }
    tally(&s, "hangers");
    for pl in &c.platforms {
        let on_bridge = city
            .bridge
            .as_ref()
            .is_some_and(|br| on_deck(br, pl.centre));
        if !on_bridge {
            scaffold(
                s.at(pl.centre),
                pl.centre,
                pl.fwd,
                pl.half_len,
                pl.half_width,
            );
        }
    }
    streets(&mut s, c, city);
    tally(&s, "streets");
    skyline(&mut s, c, city, &frame);
    tally(&s, "skyline");
    finish_gantry(&mut s.batches[n - 2], c);
    plaza(&mut s.batches[n - 1], c);
    tally(&s, "finish and plaza");
    let (terrain, ranges) = terrain(c, &frame, n);
    (scene::assemble(c, s.batches, terrain, ranges), parts)
}

/// A car's frame standing at `at` facing `yaw`, sized so its roof is the
/// top of its bouncer.
fn car_frame(c: &Course, car: &Car, at: V3, yaw: i32) -> Affine {
    let height = c.mushrooms.get(car.bouncer).map(|m| m.top.y - car.at.y);
    let scale = match height {
        Some(h) if h > 0 => fx::div(h, vehicles::roof(car.kind)).clamp(ONE * 4 / 5, ONE * 3),
        _ => ONE,
    };
    vehicles::frame(at, yaw, scale)
}

/// The cabs and buses driving along the streets, where `Course::car_at`
/// puts them this tick (the same function their bouncers follow), near
/// the eye.
pub fn cars_dynamic(b: &mut Batch, c: &Course, city: &CityLayout, eye: V3, tick: u32) {
    for (k, car) in city.cars.iter().enumerate() {
        if car.drive.is_none() {
            continue;
        }
        let (at, yaw) = c.car_at(k, tick);
        if at.flat().sub(eye.flat()).len() > int(200) {
            continue;
        }
        vehicles::car(b, car.kind, &car_frame(c, car, at, yaw), k);
    }
}

// ---- the ground ----

/// The ground's grid is laid out square to the river (so its banks are
/// straight grid lines), or to the world when there is none.
struct RiverFrame {
    o: V3,
    u: V3,
    v: V3,
    /// the river's far bank, metres along `u` (its near bank is at 0)
    far: Option<i32>,
    /// the stretch of the course (metres along it) the river is drawn
    /// near: the band across the city is water only where the course's
    /// nearest point is the crossing, not where another street passes
    /// through the same band (the first left turn)
    span: Option<(i32, i32)>,
}

impl RiverFrame {
    fn of(c: &Course, river: Option<River>) -> RiverFrame {
        match river {
            Some(r) => {
                let o = c.point(r.from).flat();
                let u = c.heading_at((r.from + r.to) / 2).flat().norm();
                let far = c.point(r.to).flat().sub(o).dot(u);
                RiverFrame {
                    o,
                    u,
                    v: right_of(u),
                    far: Some(far.max(int(4))),
                    span: Some((r.from - RIVER_MARGIN, r.to + RIVER_MARGIN)),
                }
            }
            None => RiverFrame {
                o: V3::ZERO,
                u: v3(0, 0, ONE),
                v: v3(-ONE, 0, 0),
                far: None,
                span: None,
            },
        }
    }

    fn world(&self, a: i32, b: i32) -> V3 {
        self.o.add(self.u.scale(a)).add(self.v.scale(b))
    }

    fn local(&self, p: V3) -> (i32, i32) {
        let d = p.flat().sub(self.o);
        (d.dot(self.u), d.dot(self.v))
    }

    fn in_river(&self, a: i32) -> bool {
        self.far.is_some_and(|f| a > 0 && a < f)
    }

    /// Whether the river is drawn at `p`: inside its band, and near the
    /// crossing rather than another street.
    fn river_here(&self, c: &Course, p: V3) -> bool {
        let Some((from, to)) = self.span else {
            return false;
        };
        if !self.in_river(self.local(p).0) {
            return false;
        }
        let s = nearest(c, p) as i32 * crate::course::STEP;
        s > from && s < to
    }
}

/// Grid lines from `lo` to `hi` about `g` apart, with each of `must` among
/// them exactly.
fn breaks(lo: i32, hi: i32, g: i32, must: &[i32]) -> Vec<i32> {
    let mut out: Vec<i32> = Vec::new();
    let mut x = lo;
    while x <= hi {
        if must.iter().all(|m| (x - m).abs() > g / 3) {
            out.push(x);
        }
        x += g;
    }
    for m in must {
        if *m > lo && *m < hi {
            out.push(*m);
        }
    }
    out.sort();
    out
}

/// The centreline points the ground and the skyline measure from: every
/// fourth sample, and the plaza past the finish and the street behind the
/// start.
fn samples(c: &Course) -> Vec<V3> {
    let mut pts: Vec<V3> = c.path.iter().step_by(4).copied().collect();
    let mut s = int(10);
    while s <= OUTRO {
        pts.push(c.point(c.length + s));
        s += int(10);
    }
    pts.push(c.point(-int(40)));
    pts
}

/// The flat distance from `p` to the nearest of `pts`.
fn distance(pts: &[V3], p: V3) -> i32 {
    let f = p.flat();
    let mut best = i64::MAX;
    for q in pts {
        let d = f.sub(q.flat());
        best = best.min(d.dot64(d));
    }
    fx::isqrt(best as u128) as i32
}

/// The nearest centreline sample to `p`: a coarse search of the whole
/// course, then a fine one near the best.
fn nearest(c: &Course, p: V3) -> usize {
    let f = p.flat();
    let mut best = (i64::MAX, 0);
    for (i, q) in c.path.iter().enumerate().step_by(8) {
        let d = f.sub(*q);
        best = best.min((d.dot64(d), i));
    }
    c.nearest(p, best.1)
}

/// The river is drawn only where the course's nearest point is within
/// this far of the crossing.
const RIVER_MARGIN: i32 = int(60);

/// How far out from the centreline the ground is drawn.
const GROUND_OUT: i32 = int(240);
const CELL: i32 = int(12);

/// The ground: a grid of faceted asphalt cells square to the river, the
/// riverbed under the water, and the plaza's paving past the finish; and
/// for each chunk its vertex range.
fn terrain(c: &Course, f: &RiverFrame, n: usize) -> (Vec<i32>, Vec<(usize, u32, u32)>) {
    let pts = samples(c);
    let (mut a0, mut a1, mut b0, mut b1) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
    for p in &pts {
        let (a, b) = f.local(*p);
        a0 = a0.min(a);
        a1 = a1.max(a);
        b0 = b0.min(b);
        b1 = b1.max(b);
    }
    let m = GROUND_OUT + CELL;
    let must: Vec<i32> = f.far.map(|x| vec![0, x]).unwrap_or_default();
    let us = breaks(a0 - m, a1 + m, CELL, &must);
    let vs = breaks(b0 - m, b1 + m, CELL, &[]);
    let last = c.path.len() - 1;
    let end = c.path[last];
    let end_fwd = c.heading[last];
    let mut cells: Vec<Vec<i32>> = vec![Vec::new(); n];
    for (i, w) in us.windows(2).enumerate() {
        for (j, h) in vs.windows(2).enumerate() {
            let (ua, ub, va, vb) = (w[0], w[1], h[0], h[1]);
            let mid = f.world((ua + ub) / 2, (va + vb) / 2);
            let wet = f.river_here(c, mid);
            let d = distance(&pts, mid);
            if d > GROUND_OUT + if wet { int(80) } else { 0 } {
                continue;
            }
            let k = nearest(c, mid);
            let past = mid.flat().sub(end).dot(end_fwd);
            let chunk = if k == last && past > 0 {
                n - 1
            } else {
                (k / CHUNK).min(n - 2)
            };
            let height = |p: V3| {
                let g = ground(p.x, p.z);
                if wet {
                    g.min(river_level(c) - int(4))
                } else {
                    g
                }
            };
            let corner = |a: i32, b: i32| {
                let p = f.world(a, b);
                v3(p.x, height(p), p.z)
            };
            let q = [
                corner(ua, va),
                corner(ub, va),
                corner(ub, vb),
                corner(ua, vb),
            ];
            let plaza = !wet && past > -int(4) && past < OUTRO && {
                let side = mid.flat().sub(end).dot(right_of(end_fwd));
                side.abs() < int(46)
            };
            let base = if wet {
                RIVERBED
            } else if plaza {
                PAVER[(i + j) % 2]
            } else {
                ASPHALT
            };
            for (t, tri) in [[q[0], q[1], q[2]], [q[0], q[2], q[3]]]
                .into_iter()
                .enumerate()
            {
                let wobble = 94 + ((i * 7 + j * 13 + t * 5) % 9) as i32;
                scene::push_ground(&mut cells[chunk], tri, shade(base, wobble));
            }
        }
    }
    let mut out = Vec::new();
    let mut ranges = Vec::new();
    for (k, v) in cells.into_iter().enumerate() {
        if v.is_empty() {
            continue;
        }
        let first = (out.len() / crate::mesh::VERTEX_VALUES) as u32;
        let count = (v.len() / crate::mesh::VERTEX_VALUES) as u32;
        out.extend(v);
        ranges.push((k, first, count));
    }
    (out, ranges)
}

/// The river's surface height (0 when the course has none).
fn river_level(c: &Course) -> i32 {
    c.city
        .as_ref()
        .and_then(|l| l.river)
        .map(|r| r.level)
        .unwrap_or(0)
}

// ---- the river ----

/// The water, in pieces 25 m across (only near the crossing); its stone embankments with a railing
/// along their tops; and the city's lights reflected in it.
fn river(s: &mut Sink, c: &Course, f: &RiverFrame, r: River, bridge: Option<&Bridge>) {
    let Some(far) = f.far else { return };
    let pts = samples(c);
    let piece = int(25);
    let reach = GROUND_OUT + int(80);
    let crossing = f.local(c.point((r.from + r.to) / 2)).1;
    let mut b = -int(1400);
    let mut k = 0;
    while b < int(1400) {
        let mid_b = b + piece / 2;
        let mid = f.world(far / 2, mid_b);
        if distance(&pts, mid) > reach + piece / 2 || !f.river_here(c, mid) {
            b += piece;
            k += 1;
            continue;
        }
        let at = v3(mid.x, r.level - cm(30), mid.z);
        let water = panel(at, f.v, V3::UP, f.u, piece, cm(60), far);
        s.at(mid).push(Mesh::Cube, &water, WATER, ONE / 10);
        // the lights' reflections: short bright streaks across the water
        for j in 0..2 {
            let seed = seed_of(k, j);
            let a = (pick(seed, 1000) as i32) * (far / 1000);
            let bb = b + (pick(seed ^ 7, 1000) as i32) * (piece / 1000);
            let p = f.world(a, bb);
            let colour = LIT[pick(seed ^ 3, 4) as usize];
            s.at(mid).push(
                Mesh::Cube,
                &panel(
                    v3(p.x, r.level + cm(2), p.z),
                    f.v,
                    V3::UP,
                    f.u,
                    cm(40),
                    cm(2),
                    int(6),
                ),
                shade(colour, 70),
                ONE * 3 / 5,
            );
        }
        // the embankments, a stone wall down each bank, 25 m at a time
        for bank in [0, far] {
            let out = if bank == 0 { f.u.neg() } else { f.u };
            let mut w = b;
            while w < b + piece {
                let p = f.world(bank, w + int(25) / 2).add(out.scale(cm(60)));
                let top = ground(p.x, p.z) + cm(6);
                let bed = r.level - int(4);
                let wall = panel(
                    v3(p.x, (top + bed) / 2, p.z),
                    f.v,
                    V3::UP,
                    f.u,
                    int(25),
                    top - bed,
                    cm(120),
                );
                s.at(p)
                    .push(Mesh::Cube, &wall, shade(STONE, 92 + (w / ONE % 3) * 6), 0);
                // a railing along the top, not across the bridge's way
                let clear = bridge.map_or(int(26), |br| br.half_width + int(6));
                if (w + int(25) / 2 - crossing).abs() > clear {
                    let a = v3(p.x, top + cm(100), p.z).sub(f.v.scale(int(25) / 2));
                    let e = a.add(f.v.scale(int(25)));
                    s.at(p).push(Mesh::Cyl, &rod(a, e, cm(10)), STEEL, 0);
                    s.at(p)
                        .push(Mesh::Cube, &rod(v3(a.x, top, a.z), a, cm(10)), STEEL, 0);
                }
                w += int(25);
            }
        }
        b += piece;
        k += 1;
    }
}

// ---- the bridge ----

/// Whether `p` stands on the bridge's deck (seen from above).
fn on_deck(br: &Bridge, p: V3) -> bool {
    let d = br.end.flat().sub(br.start.flat());
    let len = d.len();
    if len == 0 {
        return false;
    }
    let fwd = d.norm();
    let q = p.flat().sub(br.start.flat());
    let along = q.dot(fwd);
    along > -int(2) && along < len + int(2) && q.dot(right_of(fwd)).abs() < br.half_width + int(2)
}

/// A Brooklyn-Bridge-style suspension bridge (`city::Bridge` and its
/// cables): a steel deck on deep girders with outriggers out to the outer
/// cables, rails and lamps, ladders down to the water; granite towers
/// with a tall pointed Gothic arch, their legs outside the corridor; the
/// four main cables slung over them (`Bridge::cable`), suspenders down to
/// the deck (sparser over the way, and clear of the vines), and the
/// diagonal stays fanning from the tower tops along the outer cables.
fn bridge(s: &mut Sink, c: &Course, br: &Bridge, river: Option<River>) {
    let run = br.end.sub(br.start).flat();
    let len = run.len();
    if len < int(4) {
        return;
    }
    let fwd = run.norm();
    let right = right_of(fwd);
    let w = br.half_width;
    let water = river.map_or(br.start.y - int(4), |r| r.level);
    let outer = CABLES[CABLES.len() - 1].abs();
    let deck = |t: i32, x: i32| br.start.add(fwd.scale(t)).add(right.scale(x));
    // the deck in 20 m pieces, each in its own chunk
    let pieces = (len / int(20)).max(1);
    for k in 0..pieces {
        let t0 = len * k / pieces;
        let t1 = len * (k + 1) / pieces;
        let mid = deck((t0 + t1) / 2, 0);
        let l = t1 - t0;
        let out = s.at(mid);
        out.push(
            Mesh::Cube,
            &panel(
                mid.sub(up(cm(40))),
                right,
                V3::UP,
                fwd,
                w * 2,
                cm(80),
                l + cm(4),
            ),
            rgb(66, 62, 62),
            0,
        );
        for side in [-1, 1] {
            let edge = mid.add(right.scale(side * w));
            out.push(
                Mesh::Cube,
                &panel(
                    edge.sub(up(cm(150))),
                    right,
                    V3::UP,
                    fwd,
                    cm(80),
                    int(3),
                    l + cm(4),
                ),
                STEEL,
                0,
            );
            let rail = edge.add(up(cm(110))).sub(right.scale(side * cm(30)));
            out.push(
                Mesh::Cube,
                &panel(rail, right, V3::UP, fwd, cm(12), cm(12), l),
                STEEL,
                0,
            );
            let post = deck(t0, side * (w - cm(30)));
            out.push(
                Mesh::Cube,
                &rod(post, post.add(up(cm(110))), cm(14)),
                STEEL,
                0,
            );
            // the lamp posts, lit
            let lamp = post.add(up(int(5)));
            out.push(Mesh::Hex, &rod(post, lamp, cm(20)), POLE, 0);
            out.push(
                Mesh::Bevel,
                &placed(lamp, 0, v3(cm(50), cm(70), cm(50))),
                LAMP,
                ONE,
            );
            // the outer girder the outer cables' suspenders hold
            let rim = mid.add(right.scale(side * outer)).sub(up(cm(90)));
            out.push(
                Mesh::Cube,
                &panel(rim, right, V3::UP, fwd, cm(50), cm(70), l + cm(4)),
                STEEL,
                0,
            );
        }
        // a cross beam under the deck out to the outer girders
        out.push(
            Mesh::Cube,
            &panel(
                mid.sub(up(cm(120))),
                right,
                V3::UP,
                fwd,
                outer * 2,
                cm(50),
                cm(60),
            ),
            STEEL,
            0,
        );
        out.push(
            Mesh::Cube,
            &panel(
                mid.add(up(cm(2))),
                right,
                V3::UP,
                fwd,
                cm(16),
                cm(2),
                int(3),
            ),
            LANE,
            ONE / 6,
        );
    }
    // the ladders down to the water
    for l in br.ladders() {
        let inward = br
            .start
            .flat()
            .add(fwd.scale(l.flat().sub(br.start.flat()).dot(fwd)))
            .sub(l.flat())
            .norm();
        let foot = l.sub(inward.scale(cm(30)));
        let low = water - int(1);
        let out = s.at(l);
        let across = right_of(inward);
        for x in [-cm(30), cm(30)] {
            let a = foot.add(across.scale(x));
            out.push(
                Mesh::Cube,
                &rod(v3(a.x, low, a.z), a.add(up(cm(100))), cm(8)),
                STEEL,
                0,
            );
        }
        let mut y = low + cm(30);
        while y < l.y {
            let a = v3(foot.x, y, foot.z);
            out.push(
                Mesh::Cube,
                &rod(
                    a.sub(across.scale(cm(30))),
                    a.add(across.scale(cm(30))),
                    cm(6),
                ),
                STEEL,
                0,
            );
            y += cm(60);
        }
    }
    // the towers
    for t in &br.towers {
        tower(s, *t, fwd, right, c.half_width, br.tower_height, water);
    }
    // the main cables, their suspenders and the stays
    let vines: Vec<V3> = c.vines.iter().map(|v| v.anchor).collect();
    let clear = |p: V3| vines.iter().all(|a| a.flat().sub(p.flat()).len() > int(4));
    let towers: Vec<i32> = br
        .towers
        .iter()
        .map(|t| t.flat().sub(br.start.flat()).dot(fwd))
        .collect();
    for (k, &x) in CABLES.iter().enumerate() {
        let is_outer = x.abs() == outer;
        let step = int(5);
        let mut t = 0;
        while t < len {
            let t1 = (t + step).min(len);
            let (a, b) = (br.cable(k, t), br.cable(k, t1));
            let out = s.at(a);
            out.push(
                Mesh::Cyl,
                &rod(a, b, if is_outer { cm(50) } else { cm(40) }),
                CABLE,
                0,
            );
            // suspenders only from the outer cables, beside the deck, thin
            // and dim against the dusk so the swingable wires stand out
            if is_outer && t % int(10) == 0 && t > 0 {
                let foot = deck(t, x).sub(up(cm(60)));
                if a.y > foot.y + int(2) && clear(a) {
                    out.push(Mesh::Cube, &rod(foot, a, cm(5)), SUSPENDER, ONE / 3);
                }
            }
            t = t1;
        }
        // where each cable comes down at the deck's ends: an anchor block
        for end in [0, len] {
            let p = br.cable(k, end);
            let foot = deck(end, x);
            s.at(foot).push(
                Mesh::Bevel,
                &placed(
                    foot.add(up(cm(50))),
                    yaw_of(fwd),
                    v3(cm(150), cm(100), cm(200)),
                ),
                STONE,
                0,
            );
            s.at(foot)
                .push(Mesh::Cyl, &rod(foot.add(up(cm(90))), p, cm(50)), CABLE, 0);
        }
        // the stays fan from the tower tops along the outer cables
        if is_outer {
            for &at in &towers {
                let top = br.cable(k, at);
                for dir in [-1, 1] {
                    for j in 1..=3 {
                        let to = at + dir * int(9) * j;
                        if to <= 0 || to >= len {
                            continue;
                        }
                        let foot = deck(to, x).sub(up(cm(60)));
                        s.at(foot)
                            .push(Mesh::Cube, &rod(top, foot, cm(5)), SUSPENDER, ONE / 3);
                    }
                }
            }
        }
    }
}

/// One of the bridge's granite towers, standing `h` over the deck at
/// `foot` (its middle on the deck), its feet in the water: a leg each side
/// outside the corridor (`half` metres out), a tall pointed arch between
/// them, a solid top over it with a cornice and pinnacles, washed by
/// floodlights at dusk.
fn tower(s: &mut Sink, foot: V3, fwd: V3, right: V3, half: i32, h: i32, water: i32) {
    let out = s.at(foot);
    let leg = int(7);
    let depth = int(10);
    let inner = half + int(1);
    let lit = ONE / 6;
    let bottom = water - int(2);
    let arch_side = foot.y + mul(h, ONE * 72 / 100);
    let arch_top = foot.y + mul(h, ONE * 90 / 100);
    let top = foot.y + h;
    for side in [-1, 1] {
        let p = foot.add(right.scale(side * (inner + leg / 2)));
        out.push(
            Mesh::Cube,
            &panel(
                v3(p.x, (bottom + top) / 2, p.z),
                right,
                V3::UP,
                fwd,
                leg,
                top - bottom,
                depth,
            ),
            GRANITE,
            lit,
        );
        // a tall dark lancet window up each leg's face, both ways
        for face in [-1, 1] {
            let q = p.add(fwd.scale(face * (depth / 2 + cm(4))));
            out.push(
                Mesh::Cube,
                &panel(
                    v3(q.x, foot.y + h / 2, q.z),
                    right,
                    V3::UP,
                    fwd,
                    cm(140),
                    h * 2 / 5,
                    cm(10),
                ),
                shade(GRANITE, 55),
                0,
            );
        }
        // the arch's shoulder: a wedge from the leg in to the point
        let corner = foot.add(right.scale(side * inner / 2));
        let wedge = panel(
            v3(corner.x, (arch_side + arch_top) / 2, corner.z),
            fwd,
            V3::UP.neg(),
            right.scale(-side * ONE),
            depth - cm(40),
            arch_top - arch_side,
            inner,
        );
        out.push(Mesh::Wedge, &wedge, shade(GRANITE, 94), lit);
        out.push(
            Mesh::Pyramid,
            &placed(p.add(up(h + int(2))), 0, v3(int(3), int(4), int(3))),
            shade(GRANITE, 90),
            lit,
        );
    }
    let span = (inner + leg) * 2;
    out.push(
        Mesh::Cube,
        &panel(
            foot.add(up((arch_top + top) / 2 - foot.y)),
            right,
            V3::UP,
            fwd,
            span,
            top - arch_top,
            depth,
        ),
        GRANITE,
        lit,
    );
    out.push(
        Mesh::Cube,
        &panel(
            foot.add(up(h + cm(40))),
            right,
            V3::UP,
            fwd,
            span + int(2),
            cm(80),
            depth + int(2),
        ),
        shade(GRANITE, 86),
        lit,
    );
    // the pier the legs stand on, in the water
    out.push(
        Mesh::Cube,
        &panel(
            v3(foot.x, bottom + int(2), foot.z),
            right,
            V3::UP,
            fwd,
            span + int(4),
            int(5),
            depth + int(4),
        ),
        shade(STONE, 80),
        0,
    );
}

// ---- buildings ----

/// A side of a building: its middle at the foot of the wall, the way it
/// faces, the way across it, and its half width.
#[derive(Clone, Copy)]
struct Face {
    mid: V3,
    out: V3,
    across: V3,
    half: i32,
}

/// How much of a side is drawn: windows with mullions (the sides that
/// face the street), bands of light only (the sides at an angle), none
/// (the back).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Detail {
    Full,
    Bands,
    Plain,
}

/// The way a style's windows go.
struct Windows {
    floor: i32,
    bay: i32,
    /// the window's share of the floor's height, percent
    tall: i32,
    mullion: i32,
    /// percent of windows lit
    lit: u32,
    /// how many windows in a run lit (or dark) together, fewest and most
    runs: (i32, i32),
    /// the frame between the windows
    frame: Rgb,
    glass: Rgb,
}

fn windows(style: u8, body: Rgb) -> Windows {
    match style {
        BROWNSTONE => Windows {
            floor: cm(340),
            bay: cm(260),
            tall: 56,
            mullion: cm(110),
            lit: 60,
            frame: shade(body, 92),
            runs: (2, 5),
            glass: DARK_GLASS,
        },
        GLASS => Windows {
            floor: cm(380),
            bay: cm(300),
            tall: 74,
            mullion: cm(22),
            lit: 50,
            frame: rgb(96, 104, 116),
            runs: (3, 7),
            glass: rgb(56, 78, 112),
        },
        DECO => Windows {
            floor: cm(360),
            bay: cm(300),
            tall: 62,
            mullion: cm(120),
            lit: 55,
            frame: shade(body, 104),
            runs: (2, 5),
            glass: DARK_GLASS,
        },
        WAREHOUSE => Windows {
            floor: cm(460),
            bay: cm(420),
            tall: 50,
            mullion: cm(160),
            lit: 30,
            frame: shade(body, 90),
            runs: (2, 5),
            glass: DARK_GLASS,
        },
        _ => Windows {
            floor: cm(400),
            bay: cm(300),
            tall: 60,
            mullion: cm(70),
            lit: 55,
            frame: shade(body, 88),
            runs: (2, 5),
            glass: DARK_GLASS,
        },
    }
}

fn body_colour(style: u8, seed: u64) -> Rgb {
    let k = pick(seed, 3) as usize;
    match style {
        BROWNSTONE => [rgb(120, 72, 56), rgb(104, 64, 52), rgb(134, 88, 66)][k],
        GLASS => [rgb(44, 60, 86), rgb(56, 72, 96), rgb(38, 52, 74)][k],
        DECO => [rgb(178, 162, 136), rgb(160, 146, 124), rgb(190, 174, 148)][k],
        WAREHOUSE => [rgb(110, 58, 46), rgb(96, 70, 56), rgb(124, 66, 50)][k],
        TERMINAL => [rgb(176, 160, 132), rgb(170, 154, 128), rgb(182, 166, 138)][k],
        _ => [rgb(140, 64, 50), rgb(120, 58, 46), rgb(152, 82, 60)][k],
    }
}

/// A building from its block: its body (in tiers when tall), windows lit
/// at dusk on the sides that face the course, and its roof's dressing.
fn building(s: &mut Sink, bl: &Block, idx: usize) {
    let c = s.c;
    let style = look_of(bl.style);
    let seed = seed_of(idx, 0xB10C);
    let fwd = bl.fwd.flat().norm();
    let right = right_of(fwd);
    let centre = bl.centre;
    let corners = [(-1, -1), (-1, 1), (1, -1), (1, 1)].map(|(a, b)| {
        let p = centre
            .add(fwd.scale(a * bl.half_len))
            .add(right.scale(b * bl.half_width));
        ground(p.x, p.z)
    });
    let lowest = corners.iter().copied().min().unwrap_or(0);
    let base = lowest - cm(150);
    let gy = ground(centre.x, centre.z);
    let i = nearest(c, centre);
    let near = c.path[i];
    let toward = near.flat().sub(centre.flat()).norm();
    let at = i as i32 * STEP;
    let times = at > TIMES_FROM && at < TIMES_TO;
    let body = body_colour(style, seed);
    let win = windows(style, body);
    let out = s.at(centre);
    if style == HOARDING {
        hoarding(
            out,
            centre,
            fwd,
            right,
            bl.half_len,
            bl.half_width,
            bl.height.max(int(3)),
            seed,
        );
        return;
    }
    if style == TERMINAL {
        terminal(out, bl, toward, body);
        return;
    }
    // the sidewalk round it: a slab tilted to the ground under it
    sidewalk(
        out,
        centre,
        fwd,
        right,
        bl.half_len + int(3),
        bl.half_width + int(3),
    );
    // tiers: tall art deco and glass buildings step back as they rise
    let h = bl.height;
    let tiers: Vec<(i32, i32, i32)> = if style == DECO && h > int(60) {
        vec![
            (0, mul(h, ONE * 62 / 100), 100),
            (mul(h, ONE * 62 / 100), mul(h, ONE * 84 / 100), 80),
            (mul(h, ONE * 84 / 100), h, 60),
        ]
    } else if style == GLASS && h > int(90) {
        vec![
            (0, mul(h, ONE * 80 / 100), 100),
            (mul(h, ONE * 80 / 100), h, 84),
        ]
    } else {
        vec![(0, h, 100)]
    };
    for (t, &(y0, y1, pct)) in tiers.iter().enumerate() {
        let hl = bl.half_len * pct / 100;
        let hw = bl.half_width * pct / 100;
        let from = if t == 0 { base } else { gy + y0 };
        let to = gy + y1;
        out.push(
            Mesh::Cube,
            &panel(
                v3(centre.x, (from + to) / 2, centre.z),
                right,
                V3::UP,
                fwd,
                hw * 2,
                to - from,
                hl * 2,
            ),
            body,
            0,
        );
        let faces = [
            Face {
                mid: centre.add(fwd.scale(hl)),
                out: fwd,
                across: right,
                half: hw,
            },
            Face {
                mid: centre.sub(fwd.scale(hl)),
                out: fwd.neg(),
                across: right.neg(),
                half: hw,
            },
            Face {
                mid: centre.add(right.scale(hw)),
                out: right,
                across: fwd.neg(),
                half: hl,
            },
            Face {
                mid: centre.sub(right.scale(hw)),
                out: right.neg(),
                across: fwd,
                half: hl,
            },
        ];
        for (k, face) in faces.into_iter().enumerate() {
            let facing = face.out.dot(toward);
            let detail = if facing > ONE / 2 {
                Detail::Full
            } else if facing > -ONE / 4 {
                Detail::Bands
            } else {
                Detail::Plain
            };
            if detail == Detail::Plain {
                continue;
            }
            let face = Face {
                mid: v3(face.mid.x, gy, face.mid.z),
                ..face
            };
            let fseed = seed_of(idx, k * 8 + t);
            let ground_floor = t == 0;
            if times && ground_floor && to - gy > int(16) {
                let top = (gy + mul(h, ONE * 7 / 10)).min(gy + int(44));
                billboards(out, &face, gy + int(5), top, fseed, detail);
                facade(out, &face, top + int(1), to, &win, detail, fseed, false);
            } else {
                let shop = ground_floor && style != GLASS && detail == Detail::Full;
                let from_y = if ground_floor { gy } else { gy + y0 };
                facade(out, &face, from_y, to, &win, detail, fseed, shop);
            }
            if style == BROWNSTONE && detail == Detail::Full && ground_floor {
                fire_escape(out, &face, gy, to, &win, fseed);
            }
        }
    }
    // the roof
    let (y0, y1, pct) = tiers[tiers.len() - 1];
    let _ = y0;
    let top = gy + y1;
    let hl = bl.half_len * pct / 100;
    let hw = bl.half_width * pct / 100;
    roof(out, style, centre, fwd, right, top, hl, hw, h, seed);
}

/// A building site's hoarding round its block: painted plywood walls
/// pasted with lit posters, the dug-out lot inside with stacked steel.
fn hoarding(b: &mut Batch, centre: V3, fwd: V3, right: V3, hl: i32, hw: i32, h: i32, seed: u64) {
    let gy = ground(centre.x, centre.z);
    let ply = [rgb(40, 70, 120), rgb(36, 90, 70), rgb(120, 110, 96)][pick(seed, 3) as usize];
    b.push(
        Mesh::Cube,
        &panel(
            v3(centre.x, gy - cm(20), centre.z),
            right,
            V3::UP,
            fwd,
            hw * 2,
            cm(40),
            hl * 2,
        ),
        rgb(86, 70, 54),
        0,
    );
    let mut d = fx::Dice(seed ^ 0x40A2);
    for (out, across, half, depth) in [
        (fwd, right, hw, hl),
        (fwd.neg(), right.neg(), hw, hl),
        (right, fwd.neg(), hl, hw),
        (right.neg(), fwd, hl, hw),
    ] {
        let mid = centre.add(out.scale(depth));
        b.push(
            Mesh::Cube,
            &panel(
                v3(mid.x, gy + h / 2 - cm(20), mid.z),
                across,
                V3::UP,
                out,
                half * 2,
                h + cm(40),
                cm(20),
            ),
            ply,
            0,
        );
        // posters along it, lit
        let mut x = -half + int(2);
        while x + int(3) < half {
            let p = mid.add(across.scale(x + cm(140))).add(out.scale(cm(12)));
            b.push(
                Mesh::Cube,
                &panel(
                    v3(p.x, gy + h / 2, p.z),
                    across,
                    V3::UP,
                    out,
                    cm(220),
                    h * 3 / 5,
                    cm(4),
                ),
                BILLBOARDS[d.below(BILLBOARDS.len() as u32) as usize],
                ONE * 2 / 5,
            );
            x += int(d.range(4, 8));
        }
    }
    for k in 0..3 {
        let p = centre
            .add(fwd.scale(-hl / 2 + k * hl / 3))
            .add(right.scale(hw / 3));
        b.push(
            Mesh::Cube,
            &panel(
                v3(p.x, gy + cm(40) + k * cm(10), p.z),
                right,
                V3::UP,
                fwd,
                int(8),
                cm(40),
                cm(60),
            ),
            rgb(150, 60, 40),
            0,
        );
    }
}

/// The grand terminal closing the plaza (a Grand-Central-like front): a
/// stone body, columns across its front, three tall lit arched windows, a
/// cornice and a lit clock; lit windows in rows along its wings.
fn terminal(b: &mut Batch, bl: &Block, toward: V3, body: Rgb) {
    let fwd = bl.fwd.flat().norm();
    let right = right_of(fwd);
    let centre = bl.centre;
    let gy = ground(centre.x, centre.z);
    let h = bl.height.max(int(20));
    b.push(
        Mesh::Cube,
        &panel(
            v3(centre.x, gy + h / 2 - int(1), centre.z),
            right,
            V3::UP,
            fwd,
            bl.half_width * 2,
            h + int(2),
            bl.half_len * 2,
        ),
        body,
        0,
    );
    // the side facing the plaza
    let faces = [
        (fwd, right, bl.half_width, bl.half_len),
        (fwd.neg(), right.neg(), bl.half_width, bl.half_len),
        (right, fwd.neg(), bl.half_len, bl.half_width),
        (right.neg(), fwd, bl.half_len, bl.half_width),
    ];
    let (out, across, half, depth) = faces
        .into_iter()
        .max_by_key(|f| f.0.dot(toward))
        .unwrap_or(faces[0]);
    let front = centre.add(out.scale(depth));
    let at = |x: i32, y: i32, o: i32| {
        v3(front.x, gy + y, front.z)
            .add(across.scale(x))
            .add(out.scale(o))
    };
    let span = half.min(int(36));
    for k in -1..=1 {
        let x = k * span * 2 / 3;
        let w = span / 3;
        let hh = h * 11 / 20;
        b.push(
            Mesh::Cube,
            &panel(
                at(x, int(2) + hh / 2, cm(10)),
                across,
                V3::UP,
                out,
                w,
                hh,
                cm(20),
            ),
            LIT[0],
            ONE * 3 / 4,
        );
        b.push(
            Mesh::Pyramid,
            &panel(
                at(x, int(2) + hh + w / 4, cm(10)),
                across,
                V3::UP,
                out,
                w,
                w / 2,
                cm(20),
            ),
            LIT[0],
            ONE * 3 / 4,
        );
    }
    for k in 0..8 {
        let x = -span + k * span * 2 / 7;
        b.push(
            Mesh::Hex,
            &rod(at(x, 0, cm(140)), at(x, h * 3 / 4, cm(140)), cm(170)),
            shade(body, 108),
            0,
        );
    }
    b.push(
        Mesh::Cube,
        &panel(
            at(0, h * 3 / 4 + cm(80), cm(120)),
            across,
            V3::UP,
            out,
            span * 2 + int(3),
            cm(160),
            int(3),
        ),
        shade(body, 86),
        0,
    );
    b.push(
        Mesh::Hex,
        &Affine {
            c0: across.scale(int(6)),
            c1: out.scale(cm(30)),
            c2: V3::UP.scale(int(6)),
            t: at(0, h * 3 / 4 + int(5), cm(40)),
        },
        rgb(255, 240, 200),
        ONE * 3 / 4,
    );
    // its wings, windows lit in rows
    for side in [-1, 1] {
        for row in 0..3 {
            b.push(
                Mesh::Cube,
                &panel(
                    at(
                        side * (span + (half - span) / 2),
                        int(5) + row * int(6),
                        cm(5),
                    ),
                    across,
                    V3::UP,
                    out,
                    (half - span).max(int(4)) - int(4),
                    cm(220),
                    cm(10),
                ),
                LIT[(row as usize + 1) % 4],
                ONE * 3 / 4,
            );
        }
    }
}

/// A sidewalk slab round a building, tilted to follow the ground's hills.
fn sidewalk(b: &mut Batch, centre: V3, fwd: V3, right: V3, hl: i32, hw: i32) {
    let g = |p: V3| ground(p.x, p.z);
    let f = g(centre.add(fwd.scale(hl))) - g(centre.sub(fwd.scale(hl)));
    let r = g(centre.add(right.scale(hw))) - g(centre.sub(right.scale(hw)));
    let mid = g(centre);
    let a = Affine {
        c0: right.scale(hw * 2).add(up(r)),
        c1: up(cm(60)),
        c2: fwd.scale(hl * 2).add(up(f)),
        t: v3(centre.x, mid - cm(16), centre.z),
    };
    b.push(Mesh::Cube, &a, SIDEWALK, 0);
}

/// Windows up one side of a building between `from` and `to`: a band of
/// glass per floor in runs of lit and dark windows, cut into windows by
/// mullions (full detail), or the runs alone (bands). A shop's lit front
/// and awning on the ground floor when `shop`.
fn facade(
    b: &mut Batch,
    face: &Face,
    from: i32,
    to: i32,
    win: &Windows,
    detail: Detail,
    seed: u64,
    shop: bool,
) {
    let margin = cm(80);
    let span = face.half - margin;
    if span <= win.bay || to - from < win.floor {
        return;
    }
    let bays = (span * 2 / win.bay).max(1);
    let bay = span * 2 / bays;
    let floors = (to - from - cm(60)) / win.floor;
    let first = if shop { 1 } else { 0 };
    let tall = win.floor * win.tall / 100;
    let mut dice = fx::Dice(seed);
    let at = |x: i32, y: i32, out: i32| {
        face.mid
            .add(face.across.scale(x))
            .add(face.out.scale(out))
            .add(up(y - face.mid.y))
    };
    let step = if detail == Detail::Full { 1 } else { 3 };
    let mut f = first;
    while f < floors {
        let y = from + f * win.floor + win.floor / 2 + cm(10);
        let mut k = 0;
        while k < bays {
            let run = (dice.range(win.runs.0, win.runs.1)).min(bays - k);
            let lit = dice.below(100) < win.lit;
            let colour = if lit {
                LIT[dice.below(4) as usize]
            } else {
                win.glass
            };
            let x0 = -span + k * bay;
            let x1 = x0 + run * bay;
            let gap = if detail == Detail::Full { 0 } else { cm(60) };
            b.push(
                Mesh::Cube,
                &panel(
                    at((x0 + x1) / 2, y, cm(5)),
                    face.across,
                    V3::UP,
                    face.out,
                    x1 - x0 - gap,
                    tall,
                    cm(10),
                ),
                colour,
                if lit { ONE * 4 / 5 } else { ONE / 10 },
            );
            k += run;
        }
        f += step;
    }
    if detail == Detail::Full && floors > first {
        let lo = from + first * win.floor;
        let hi = from + floors * win.floor;
        for j in 0..=bays {
            let x = -span + j * bay;
            b.push(
                Mesh::Cube,
                &panel(
                    at(x, (lo + hi) / 2, cm(9)),
                    face.across,
                    V3::UP,
                    face.out,
                    win.mullion,
                    hi - lo,
                    cm(14),
                ),
                win.frame,
                0,
            );
        }
    }
    if shop {
        let colour = LIT[(seed % 4) as usize];
        b.push(
            Mesh::Cube,
            &panel(
                at(0, from + cm(150), cm(5)),
                face.across,
                V3::UP,
                face.out,
                span * 2,
                cm(240),
                cm(10),
            ),
            colour,
            ONE * 4 / 5,
        );
        // the awning, sloping out over the sidewalk
        let tilt = face.out.scale(cm(140)).add(up(-cm(60)));
        let a = Affine {
            c0: face.across.scale(span * 2 - int(1)),
            c1: up(cm(8)),
            c2: tilt,
            t: at(0, from + cm(320), cm(70)),
        };
        b.push(Mesh::Cube, &a, AWNINGS[(seed / 4 % 4) as usize], 0);
    }
}

/// A brownstone's fire escape: a landing at each floor and a ladder up
/// between them, zigzagging up the street side.
fn fire_escape(b: &mut Batch, face: &Face, from: i32, to: i32, win: &Windows, seed: u64) {
    let floors = (to - from) / win.floor;
    if floors < 3 || face.half < int(6) {
        return;
    }
    let x = if seed.is_multiple_of(2) {
        face.half / 3
    } else {
        -face.half / 3
    };
    for f in 1..floors {
        let y = from + f * win.floor - cm(20);
        let at = face
            .mid
            .add(face.across.scale(x))
            .add(face.out.scale(cm(70)))
            .add(up(y - face.mid.y));
        b.push(
            Mesh::Cube,
            &panel(at, face.across, V3::UP, face.out, int(5), cm(10), cm(120)),
            rgb(30, 30, 34),
            0,
        );
        b.push(
            Mesh::Cube,
            &panel(
                at.add(up(cm(50))).add(face.out.scale(cm(56))),
                face.across,
                V3::UP,
                face.out,
                int(5),
                cm(6),
                cm(6),
            ),
            rgb(30, 30, 34),
            0,
        );
        if f + 1 < floors {
            let s = if f % 2 == 0 { 1 } else { -1 };
            let a = at.add(face.across.scale(s * int(2)));
            let e = at.add(face.across.scale(-s * int(2))).add(up(win.floor));
            b.push(Mesh::Cube, &rod(a, e, cm(10)), rgb(30, 30, 34), 0);
        }
    }
}

/// Billboards up a Times Square front: rows of boards, each a dark frame,
/// a lit panel and a bright design on it, over a news ticker.
fn billboards(b: &mut Batch, face: &Face, from: i32, to: i32, seed: u64, detail: Detail) {
    let mut d = fx::Dice(seed ^ 0xB111);
    let span = face.half - cm(60);
    if span < int(3) {
        return;
    }
    let at = |x: i32, y: i32, out: i32| {
        face.mid
            .add(face.across.scale(x))
            .add(face.out.scale(out))
            .add(up(y - face.mid.y))
    };
    // the ticker
    b.push(
        Mesh::Cube,
        &panel(
            at(0, from - cm(150), cm(20)),
            face.across,
            V3::UP,
            face.out,
            span * 2,
            cm(120),
            cm(30),
        ),
        [BILLBOARDS[2], BILLBOARDS[0]][d.below(2) as usize],
        ONE,
    );
    let mut y = from;
    while y < to - int(4) {
        let h = int(d.range(5, 11)).min(to - y);
        let boards = if detail == Detail::Full {
            d.range(1, 3)
        } else {
            1
        };
        let mut x = -span;
        for k in 0..boards {
            let w = if k == boards - 1 {
                span - x
            } else {
                (span * 2 / boards) + int(d.range(-3, 3))
            };
            let mid = at(x + w / 2, y + h / 2, cm(40));
            let colour = BILLBOARDS[d.below(BILLBOARDS.len() as u32) as usize];
            let accent = BILLBOARDS[d.below(BILLBOARDS.len() as u32) as usize];
            b.push(
                Mesh::Cube,
                &panel(
                    mid,
                    face.across,
                    V3::UP,
                    face.out,
                    w - cm(40),
                    h - cm(40),
                    cm(60),
                ),
                rgb(24, 24, 28),
                0,
            );
            b.push(
                Mesh::Cube,
                &panel(
                    mid.add(face.out.scale(cm(32))),
                    face.across,
                    V3::UP,
                    face.out,
                    w - cm(100),
                    h - cm(100),
                    cm(6),
                ),
                colour,
                ONE,
            );
            let (dw, dh) = if d.below(2) == 0 {
                (w * 2 / 3, h / 4)
            } else {
                (w / 4, h * 2 / 3)
            };
            let off = face.across.scale(int(d.range(-2, 2)));
            b.push(
                Mesh::Cube,
                &panel(
                    mid.add(face.out.scale(cm(36))).add(off),
                    face.across,
                    V3::UP,
                    face.out,
                    dw,
                    dh,
                    cm(4),
                ),
                accent,
                ONE,
            );
            x += w;
        }
        y += h + cm(60);
    }
}

/// The roof's dressing: a cornice on the older buildings, a wooden water
/// tower on many of them, plant boxes on the towers, a spire with a red
/// light on the tallest, the art deco crown lit.
fn roof(
    b: &mut Batch,
    style: u8,
    centre: V3,
    fwd: V3,
    right: V3,
    top: i32,
    hl: i32,
    hw: i32,
    h: i32,
    seed: u64,
) {
    let c = v3(centre.x, top, centre.z);
    if style == BROWNSTONE || style == LOFT || style == WAREHOUSE {
        b.push(
            Mesh::Cube,
            &panel(
                c.add(up(cm(30))),
                right,
                V3::UP,
                fwd,
                hw * 2 + cm(80),
                cm(60),
                hl * 2 + cm(80),
            ),
            shade(body_colour(style, seed), 70),
            0,
        );
    }
    let tank = match style {
        BROWNSTONE | LOFT | WAREHOUSE => pick(seed ^ 1, 100) < 60,
        DECO => h < int(60) && pick(seed ^ 1, 100) < 35,
        _ => false,
    };
    if tank && hl > int(4) && hw > int(4) {
        let off = fwd
            .scale(hl / 3)
            .add(right.scale(if seed.is_multiple_of(2) {
                hw / 3
            } else {
                -hw / 3
            }));
        water_tower(b, c.add(off), seed);
    }
    if style == GLASS {
        let n = 1 + pick(seed ^ 5, 2) as i32;
        for k in 0..n {
            let p = c
                .add(fwd.scale(hl / 3 - k * hl / 2))
                .add(right.scale(hw / 4));
            b.push(
                Mesh::Bevel,
                &panel(
                    p.add(up(cm(120))),
                    right,
                    V3::UP,
                    fwd,
                    int(5),
                    cm(240),
                    int(4),
                ),
                CONCRETE,
                0,
            );
        }
    }
    if style == DECO && h > int(80) {
        // a stepped crown, lit, and a spire
        for k in 0..3 {
            let s = 80 - k * 22;
            let y = top + int(2) * k + int(1);
            b.push(
                Mesh::Cube,
                &panel(
                    v3(c.x, y, c.z),
                    right,
                    V3::UP,
                    fwd,
                    hw * 2 * s / 100,
                    int(2),
                    hl * 2 * s / 100,
                ),
                shade(GRANITE, 110),
                ONE / 3,
            );
        }
        let spire = c.add(up(int(6)));
        b.push(
            Mesh::Spike,
            &rod(spire, spire.add(up(int(14))), int(2)),
            SILVER,
            ONE / 4,
        );
        red_light(b, spire.add(up(int(14))));
    } else if h > int(90) {
        let mast = c.add(up(int(5)));
        b.push(Mesh::Cyl, &rod(c, mast.add(up(int(5))), cm(40)), STEEL, 0);
        red_light(b, mast.add(up(int(5))));
    }
}

fn red_light(b: &mut Batch, at: V3) {
    b.push(
        Mesh::Octa,
        &placed(at, 0, v3(cm(80), cm(80), cm(80))),
        RED_LIGHT,
        ONE,
    );
}

/// A rooftop water tower: a wooden tank with iron hoops on four legs and a
/// pointed roof.
fn water_tower(b: &mut Batch, at: V3, seed: u64) {
    let r = cm(170) + (pick(seed ^ 9, 4) as i32) * cm(15);
    let legs = int(3);
    for k in 0..4 {
        let a = deg(45) + k * deg(90);
        let foot = scene::around(at, a, r * 3 / 4, 0);
        let knee = scene::around(at, a, r * 2 / 3, legs);
        b.push(Mesh::Cube, &rod(foot, knee, cm(16)), rgb(40, 36, 34), 0);
    }
    let mid = at.add(up(legs + r));
    b.push(
        Mesh::Hex,
        &placed(mid, seed as i32 % 9000, v3(r * 2, r * 2, r * 2)),
        WOOD_TANK,
        0,
    );
    for y in [-r / 2, r / 2] {
        b.push(
            Mesh::Hex,
            &placed(
                mid.add(up(y)),
                seed as i32 % 9000,
                v3(r * 2 + cm(8), cm(10), r * 2 + cm(8)),
            ),
            rgb(40, 36, 34),
            0,
        );
    }
    b.push(
        Mesh::Cone,
        &placed(
            mid.add(up(r + cm(60))),
            0,
            v3(r * 2 + cm(20), cm(120), r * 2 + cm(20)),
        ),
        rgb(60, 52, 48),
        0,
    );
}

// ---- what the vines hang from ----

/// The still part of what a vine hangs from: a tower crane's mast, jib and
/// counterweight; a street light or a traffic-light gantry; a wire strung
/// between buildings; a clamp on the bridge's cable. (Helicopters, and a
/// crane's trolley and hook, move: `hanger_dynamic`.)
fn hanger(s: &mut Sink, c: &Course, h: &Hanger, k: usize) {
    let anchor = c.vines.get(h.vine).map(|v| v.anchor).unwrap_or(h.to);
    match h.kind {
        HangerKind::Helicopter => {}
        HangerKind::Crane => crane(s.at(h.from), h.from, h.to),
        HangerKind::StreetLight => gantry(s.at(h.from), h.from, h.to, k),
        HangerKind::Wire => {
            let out = s.at(anchor);
            let sag = h.from.sub(h.to).len() / 30;
            let pts: Vec<V3> = (0..=6)
                .map(|j| {
                    let t = ONE * j / 6;
                    h.from.lerp(h.to, t).sub(up(mul(mul(4 * t, ONE - t), sag)))
                })
                .collect();
            for w in pts.windows(2) {
                out.push(Mesh::Cyl, &rod(w[0], w[1], cm(10)), CABLE, 0);
            }
            for e in [h.from, h.to] {
                out.push(
                    Mesh::Bevel,
                    &placed(e, 0, v3(cm(40), cm(40), cm(40))),
                    STEEL,
                    0,
                );
            }
            // the junction box the vine drops from
            out.push(
                Mesh::Bevel,
                &placed(anchor.add(up(cm(25))), 0, v3(cm(60), cm(50), cm(60))),
                STEEL,
                0,
            );
        }
        HangerKind::BridgeCable => {
            let out = s.at(h.from);
            out.push(
                Mesh::Bevel,
                &placed(h.from, 0, v3(cm(90), cm(70), cm(90))),
                STEEL,
                0,
            );
            if h.from.sub(anchor).len() > cm(40) {
                out.push(Mesh::Cyl, &rod(h.from, anchor, cm(12)), CABLE, 0);
            }
        }
    }
}

/// A tower crane standing at `foot` with its jib reaching to `tip`: a
/// lattice mast, the operator's cab and the slewing ring, the jib and the
/// counter-jib with its concrete blocks, the peak with its tie bars, and
/// red lights on top and at the tip.
fn crane(b: &mut Batch, foot: V3, tip: V3) {
    let top = v3(foot.x, tip.y, foot.z);
    let reach = tip.flat().sub(top.flat());
    if reach.len() < int(2) || top.y - foot.y < int(4) {
        return;
    }
    let along = reach.norm();
    let side = right_of(along);
    let m = cm(100);
    // the mast: four corner chords and a zigzag up each face
    let corners = [(-1, -1), (1, -1), (1, 1), (-1, 1)]
        .map(|(a, c)| along.scale(a * m).add(side.scale(c * m)));
    for q in corners {
        b.push(Mesh::Cube, &rod(foot.add(q), top.add(q), cm(18)), CRANE, 0);
    }
    let mut y = foot.y;
    let mut k = 0;
    while y + int(6) <= top.y {
        // a zigzag up two faces a section, turn about
        for f in (k % 2..4).step_by(2) {
            let a = corners[f];
            let e = corners[(f + 1) % 4];
            let (p, q) = if k % 2 == 0 { (a, e) } else { (e, a) };
            b.push(
                Mesh::Cube,
                &rod(
                    v3(foot.x, y, foot.z).add(p),
                    v3(foot.x, y + int(6), foot.z).add(q),
                    cm(10),
                ),
                CRANE,
                0,
            );
        }
        y += int(6);
        k += 1;
    }
    // the cab, the jib (two lower chords and a top chord, braced) and the
    // counter-jib with its blocks
    b.push(
        Mesh::Bevel,
        &panel(
            top.add(side.scale(cm(180))).add(up(-cm(80))),
            side,
            V3::UP,
            along,
            cm(200),
            cm(220),
            cm(220),
        ),
        rgb(220, 222, 214),
        0,
    );
    b.push(
        Mesh::Cube,
        &panel(
            top.add(side.scale(cm(240)))
                .add(up(-cm(60)))
                .add(along.scale(cm(112))),
            side,
            V3::UP,
            along,
            cm(150),
            cm(110),
            cm(6),
        ),
        shade(LIT[0], 90),
        ONE * 3 / 5,
    );
    let len = reach.len();
    let back = int(14).min(len / 2);
    let tail = top.sub(along.scale(back));
    for s in [-1, 1] {
        let off = side.scale(s * cm(80));
        b.push(
            Mesh::Cube,
            &rod(top.add(off), tip.add(off), cm(16)),
            CRANE,
            0,
        );
        b.push(
            Mesh::Cube,
            &rod(tail.add(off), top.add(off), cm(16)),
            CRANE,
            0,
        );
    }
    let lift = up(cm(150));
    b.push(
        Mesh::Cube,
        &rod(top.add(lift), tip.add(up(cm(60))), cm(14)),
        CRANE,
        0,
    );
    let mut a = 0;
    let mut j = 0;
    while a + int(5) <= len {
        let lo = top.add(along.scale(a + if j % 2 == 0 { 0 } else { int(5) }));
        let hi = top
            .add(along.scale(a + if j % 2 == 0 { int(5) } else { 0 }))
            .add(lift);
        b.push(Mesh::Cube, &rod(lo, hi, cm(8)), CRANE, 0);
        a += int(5);
        j += 1;
    }
    for k in 0..3 {
        let p = tail
            .add(along.scale(int(1) + k * cm(170)))
            .add(up(-cm(120)));
        b.push(
            Mesh::Bevel,
            &panel(p, side, V3::UP, along, cm(220), cm(200), cm(150)),
            CONCRETE,
            0,
        );
    }
    let peak = top.add(up(int(8)));
    for q in [corners[0], corners[2]] {
        b.push(
            Mesh::Cube,
            &rod(top.add(q.scale(ONE / 2)), peak, cm(16)),
            CRANE,
            0,
        );
    }
    b.push(
        Mesh::Cube,
        &rod(peak, top.add(along.scale(len * 2 / 3)).add(lift), cm(6)),
        CABLE,
        0,
    );
    b.push(Mesh::Cube, &rod(peak, tail.add(lift), cm(6)), CABLE, 0);
    red_light(b, peak.add(up(cm(40))));
    red_light(b, tip.add(up(cm(80))));
}

/// A street light (even `k`) or a traffic-light gantry (odd) from `foot`
/// with its arm out to `end`, where the vine hangs.
fn gantry(b: &mut Batch, foot: V3, end: V3, k: usize) {
    let top = v3(foot.x, end.y, foot.z);
    if top.y - foot.y < int(2) {
        return;
    }
    let w = if top.y - foot.y > int(15) {
        cm(70)
    } else {
        cm(36)
    };
    b.push(Mesh::Hex, &rod(foot, top.add(up(cm(40))), w), POLE, 0);
    b.push(Mesh::Hex, &rod(foot, foot.add(up(cm(60))), w * 2), POLE, 0);
    b.push(Mesh::Cube, &rod(top, end, w * 2 / 3), POLE, 0);
    let arm = end.flat().sub(top.flat());
    let along = if arm.len() > 0 {
        arm.norm()
    } else {
        v3(0, 0, ONE)
    };
    let side = right_of(along);
    if k.is_multiple_of(2) {
        // the lamp head at the arm's end
        let head = end.sub(along.scale(cm(100))).sub(up(cm(30)));
        b.push(
            Mesh::Bevel,
            &panel(head, side, V3::UP, along, cm(90), cm(30), cm(170)),
            LAMP,
            ONE,
        );
    } else {
        // two signals along the arm, one lit red, one green
        for (j, lit) in [(1, RED_LIGHT), (2, GREEN_LIGHT)] {
            let p = top.add(along.scale(arm.len() * j / 3)).sub(up(cm(140)));
            b.push(
                Mesh::Bevel,
                &panel(p, side, V3::UP, along, cm(60), cm(180), cm(60)),
                rgb(220, 180, 30),
                0,
            );
            let y = if j == 1 { cm(50) } else { -cm(50) };
            b.push(
                Mesh::Hex,
                &placed(
                    p.add(up(y)).add(side.scale(cm(32))),
                    0,
                    v3(cm(36), cm(36), cm(36)),
                ),
                lit,
                ONE,
            );
            b.push(
                Mesh::Hex,
                &placed(p.add(side.scale(cm(32))), 0, v3(cm(36), cm(36), cm(36))),
                shade(AMBER, 40),
                0,
            );
        }
        // the street sign
        b.push(
            Mesh::Cube,
            &panel(
                top.sub(up(cm(220))).add(along.scale(cm(60))),
                side,
                V3::UP,
                along,
                cm(6),
                cm(40),
                cm(160),
            ),
            rgb(20, 110, 60),
            ONE / 4,
        );
    }
}

/// What moves on what a vine hangs from: a helicopter hovering (its rotor
/// turning, following its vine's anchor when that slides), a crane's
/// trolley and hook block at the jib where the vine hangs. Drawn each
/// frame, near the eye.
pub fn hangers_dynamic(
    b: &mut Batch,
    c: &Course,
    city: &CityLayout,
    eye: V3,
    tick: u32,
    race_tick: u32,
) {
    for (k, h) in city.hangers.iter().enumerate() {
        let Some(v) = c.vines.get(h.vine) else {
            continue;
        };
        let now = c.anchor(h.vine, race_tick);
        if now.flat().sub(eye.flat()).len() > int(260) {
            continue;
        }
        match h.kind {
            HangerKind::Helicopter => {
                let hub = h.from.add(now.sub(v.anchor));
                // high enough that its skids clear the vine's top
                let hub =
                    v3(hub.x, hub.y.max(now.y + cm(560)), hub.z).add(up(vehicles::hover(tick, k)));
                let yaw = (k as i32) * deg(97);
                vehicles::helicopter(b, hub, yaw, tick, k, ONE * 3 / 2);
                // the cable from its belly down to the vine's top
                let belly = hub.sub(up(cm(420)));
                if belly.y > now.y + cm(30) {
                    b.push(Mesh::Cyl, &rod(belly, now, cm(12)), CABLE, 0);
                }
            }
            HangerKind::Crane => {
                let trolley = v3(now.x, h.to.y, now.z);
                b.push(
                    Mesh::Bevel,
                    &placed(trolley.sub(up(cm(40))), 0, v3(cm(140), cm(60), cm(180))),
                    STEEL,
                    0,
                );
                if trolley.y - now.y > cm(80) {
                    b.push(
                        Mesh::Cube,
                        &rod(trolley, now.add(up(cm(50))), cm(6)),
                        CABLE,
                        0,
                    );
                }
                b.push(
                    Mesh::Bevel,
                    &placed(now.add(up(cm(30))), 0, v3(cm(80), cm(70), cm(50))),
                    CRANE,
                    0,
                );
            }
            _ => {}
        }
    }
}

// ---- the streets ----

/// How sharply the course turns at sample `i` (the sine of the angle
/// between the headings 8 m before and after).
fn turning(c: &Course, i: usize) -> i32 {
    let n = c.path.len() - 1;
    let a = c.heading[i.saturating_sub(4)];
    let b = c.heading[(i + 4).min(n)];
    a.cross(b).y.abs()
}

/// The street's markings and furniture: a dashed line down the middle,
/// zebra crossings at the corners, street lamps with pools of light.
fn streets(s: &mut Sink, c: &Course, city: &CityLayout) {
    let hw = c.half_width;
    let dry = |at: i32| {
        city.river
            .is_none_or(|r| at < r.from - int(8) || at > r.to + int(8))
    };
    let corner = |i: usize| turning(c, i) > ONE / 4;
    // the dashes
    let mut at = int(6);
    while at < c.length {
        let i = (at / STEP) as usize;
        if dry(at) && !corner(i) {
            let p = on_ground(c.point(at));
            let fwd = c.heading_at(at);
            s.at(p).push(
                Mesh::Cube,
                &panel(
                    p.add(up(cm(3))),
                    right_of(fwd),
                    V3::UP,
                    fwd,
                    cm(18),
                    cm(4),
                    int(3),
                ),
                LANE,
                ONE / 6,
            );
        }
        at += int(9);
    }
    // the crossings, either side of each corner
    let mut was = false;
    for i in 0..c.path.len() {
        let now = corner(i);
        if now != was {
            let at = i as i32 * STEP + if now { -int(7) } else { int(7) };
            if dry(at) && at > 0 && at < c.length {
                zebra(s, c, at, hw);
            }
        }
        was = now;
    }
    // the lamps, every 34 m on alternate sides
    let mut at = int(20);
    let mut k = 0;
    while at < c.length {
        let i = (at / STEP) as usize;
        if dry(at) && !corner(i) {
            let side = if k % 2 == 0 { 1 } else { -1 };
            let fwd = c.heading_at(at);
            let right = right_of(fwd);
            let foot = on_ground(c.point(at).add(right.scale(side * (hw - cm(100)))));
            let top = foot.add(up(int(8)));
            let head = top.sub(right.scale(side * int(3)));
            let out = s.at(foot);
            out.push(Mesh::Hex, &rod(foot, top, cm(28)), POLE, 0);
            out.push(Mesh::Cube, &rod(top, head, cm(16)), POLE, 0);
            out.push(
                Mesh::Bevel,
                &panel(
                    head.sub(up(cm(20))),
                    fwd,
                    V3::UP,
                    right,
                    cm(70),
                    cm(26),
                    cm(120),
                ),
                LAMP,
                ONE,
            );
        }
        at += int(34);
        k += 1;
    }
}

fn zebra(s: &mut Sink, c: &Course, at: i32, hw: i32) {
    let fwd = c.heading_at(at);
    let right = right_of(fwd);
    let mut a = -hw + int(2);
    while a < hw - int(2) {
        let p = on_ground(c.point(at).add(right.scale(a)));
        s.at(p).push(
            Mesh::Cube,
            &panel(p.add(up(cm(3))), right, V3::UP, fwd, cm(80), cm(4), int(4)),
            ZEBRA,
            ONE / 8,
        );
        a += cm(170);
    }
}

/// The start platform on the City: a steel scaffold deck on posts, with a
/// rail round it.
fn scaffold(b: &mut Batch, centre: V3, fwd: V3, hl: i32, hw: i32) {
    let right = right_of(fwd);
    b.push(
        Mesh::Cube,
        &panel(
            centre.sub(up(cm(30))),
            right,
            V3::UP,
            fwd,
            hw * 2,
            cm(60),
            hl * 2,
        ),
        rgb(90, 86, 82),
        0,
    );
    let g = ground(centre.x, centre.z);
    for sx in [-1, 1] {
        for sz in [-1, 0, 1] {
            let p = centre
                .add(right.scale(sx * (hw - cm(40))))
                .add(fwd.scale(sz * (hl - cm(40))));
            b.push(
                Mesh::Cube,
                &rod(v3(p.x, g, p.z), v3(p.x, centre.y, p.z), cm(30)),
                STEEL,
                0,
            );
        }
        let a = centre
            .add(right.scale(sx * (hw - cm(40))))
            .sub(fwd.scale(hl - cm(40)));
        let e = centre
            .add(right.scale(sx * (hw - cm(40))))
            .add(fwd.scale(hl - cm(40)));
        // the cross braces under the deck, and the rail
        b.push(
            Mesh::Cube,
            &rod(v3(a.x, g, a.z), e.sub(up(cm(60))), cm(16)),
            STEEL,
            0,
        );
        b.push(
            Mesh::Cube,
            &rod(a.add(up(cm(110))), e.add(up(cm(110))), cm(10)),
            CRANE,
            0,
        );
    }
}

// ---- the skyline ----

/// Where the Empire-State-like tower stands: the far-side skyline spot
/// nearest this, two thirds of the way along the course, off to the left.
fn landmark_target(c: &Course) -> V3 {
    let s = c.length * 2 / 3;
    c.point(s).sub(right_of(c.heading_at(s)).scale(int(150)))
}

/// Towers beyond the streets, in the haze: plain shafts with bands of lit
/// windows toward the course, a few with crowns; the Empire-State-like
/// tower among them.
fn skyline(s: &mut Sink, c: &Course, city: &CityLayout, f: &RiverFrame) {
    let pts = samples(c);
    let (mut x0, mut x1, mut z0, mut z1) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
    for p in &pts {
        x0 = x0.min(p.x);
        x1 = x1.max(p.x);
        z0 = z0.min(p.z);
        z1 = z1.max(p.z);
    }
    let gap = int(78);
    let m = int(240);
    let target = landmark_target(c);
    let mut spots: Vec<(V3, i32, u64)> = Vec::new();
    let mut x = x0 - m;
    let mut i = 0usize;
    while x < x1 + m {
        let mut z = z0 - m;
        let mut j = 0usize;
        while z < z1 + m {
            let seed = seed_of(i + 7, j + 11);
            let p = v3(
                x + int(pick(seed, 24) as i32 - 12),
                0,
                z + int(pick(seed ^ 1, 24) as i32 - 12),
            );
            let d = distance(&pts, p);
            let wet = f.far.is_some() && {
                let (a, _) = f.local(p);
                a > -int(25) && a < f.far.unwrap_or(0) + int(25)
            };
            let clear = city.blocks.iter().all(|b| {
                let r = b.half_len.max(b.half_width) + int(16);
                b.centre.flat().sub(p).len() > r
            });
            if d > int(80) && d < int(250) && !wet && clear {
                spots.push((p, d, seed));
            }
            z += gap;
            j += 1;
        }
        x += gap;
        i += 1;
    }
    let landmark = spots
        .iter()
        .enumerate()
        .filter(|(_, (_, d, _))| *d > int(110))
        .min_by_key(|(_, (p, _, _))| p.sub(target.flat()).dot64(p.sub(target.flat())))
        .map(|(k, _)| k);
    for (k, &(p, d, seed)) in spots.iter().enumerate() {
        let at = on_ground(p);
        let near = c.path[nearest(c, at)];
        let toward = near.flat().sub(at.flat()).norm();
        if Some(k) == landmark {
            empire_state(s.at(at), at, toward);
            continue;
        }
        let h = int(40) + int(pick(seed ^ 2, 70) as i32) + (d - int(80)) * 2 / 3;
        far_tower(s.at(at), at, toward, h, seed);
    }
}

/// A tower in the distance: a shaft (glass or stone), bands of lit windows
/// every other floor on the side toward the course, sometimes a crown.
fn far_tower(b: &mut Batch, at: V3, toward: V3, h: i32, seed: u64) {
    let glass = pick(seed ^ 3, 2) == 0;
    let body = if glass {
        body_colour(GLASS, seed)
    } else {
        shade(body_colour(DECO, seed), 80)
    };
    let fwd = toward;
    let right = right_of(fwd);
    let hl = int(8 + pick(seed ^ 4, 8) as i32);
    let hw = int(8 + pick(seed ^ 5, 8) as i32);
    let gy = at.y;
    b.push(
        Mesh::Cube,
        &panel(
            at.add(up(h / 2 - int(1))),
            right,
            V3::UP,
            fwd,
            hw * 2,
            h + int(2),
            hl * 2,
        ),
        body,
        0,
    );
    let mut d = fx::Dice(seed ^ 0xFA4);
    for (face_out, across, half) in [
        (fwd, right, hw),
        (right, fwd.neg(), hl),
        (right.neg(), fwd, hl),
    ] {
        let mid = at.add(face_out.scale(if face_out == fwd { hl } else { hw }));
        let mut y = gy + int(8);
        let step = if face_out == fwd { cm(760) } else { cm(2280) };
        while y < gy + h - int(4) {
            let lit = d.below(100) < 55;
            let colour = if lit {
                LIT[d.below(4) as usize]
            } else {
                DARK_GLASS
            };
            let w = half * 2 - int(d.range(2, 6));
            b.push(
                Mesh::Cube,
                &panel(
                    v3(mid.x, y, mid.z).add(face_out.scale(cm(5))),
                    across,
                    V3::UP,
                    face_out,
                    w,
                    cm(220),
                    cm(10),
                ),
                colour,
                if lit { ONE * 3 / 4 } else { 0 },
            );
            y += step;
        }
    }
    if pick(seed ^ 6, 4) == 0 {
        let top = at.add(up(h));
        b.push(
            Mesh::Pyramid,
            &panel(
                top.add(up(int(5))),
                right,
                V3::UP,
                fwd,
                hw * 3 / 2,
                int(10),
                hl * 3 / 2,
            ),
            shade(body, 120),
            ONE / 4,
        );
        red_light(b, top.add(up(int(11))));
    }
}

/// The Empire-State-like tower: a broad base, setbacks, a long shaft of
/// lit window strips, a stepped crown lit gold, and the mast with its red
/// light.
fn empire_state(b: &mut Batch, at: V3, toward: V3) {
    let fwd = toward;
    let right = right_of(fwd);
    let stone = rgb(150, 140, 124);
    let tiers = [
        (0, 24, 52, 40),
        (24, 32, 44, 32),
        (32, 150, 34, 24),
        (150, 160, 26, 18),
        (160, 168, 18, 14),
        (168, 174, 12, 10),
    ];
    for (k, &(y0, y1, w, d)) in tiers.iter().enumerate() {
        let (y0, y1) = (at.y + int(y0), at.y + int(y1));
        let glow = if k >= 3 { ONE * 3 / 5 } else { 0 };
        let colour = if k >= 3 { rgb(255, 214, 140) } else { stone };
        b.push(
            Mesh::Cube,
            &panel(
                v3(at.x, (y0 + y1) / 2, at.z),
                right,
                V3::UP,
                fwd,
                int(w),
                y1 - y0,
                int(d),
            ),
            colour,
            glow,
        );
        if k == 2 {
            // the shaft's window strips, up the faces toward the course
            for (out, across, half, depth) in [
                (fwd, right, w / 2, d / 2),
                (right, fwd.neg(), d / 2, w / 2),
                (right.neg(), fwd, d / 2, w / 2),
            ] {
                let mut x = -int(half) + int(2);
                let mut j = 0;
                while x < int(half) - int(1) {
                    let lit = (j * 7 + half) % 3 != 0;
                    let p = at.add(out.scale(int(depth) + cm(5))).add(across.scale(x));
                    b.push(
                        Mesh::Cube,
                        &panel(
                            v3(p.x, (y0 + y1) / 2, p.z),
                            across,
                            V3::UP,
                            out,
                            cm(140),
                            y1 - y0 - int(4),
                            cm(10),
                        ),
                        if lit {
                            LIT[(j % 4) as usize]
                        } else {
                            DARK_GLASS
                        },
                        if lit { ONE * 3 / 5 } else { 0 },
                    );
                    x += int(3);
                    j += 1;
                }
            }
        }
    }
    let top = at.add(up(int(174)));
    b.push(
        Mesh::Hex,
        &rod(top, top.add(up(int(14))), int(3)),
        SILVER,
        ONE / 3,
    );
    b.push(
        Mesh::Spike,
        &rod(top.add(up(int(14))), top.add(up(int(34))), int(1)),
        SILVER,
        ONE / 4,
    );
    red_light(b, top.add(up(int(34))));
}

// ---- the finish and the plaza ----

/// The finish on the City: two steel towers studded with bulbs, the
/// chequered banner between them, and a glowing line on the street.
fn finish_gantry(b: &mut Batch, c: &Course) {
    let n = c.path.len() - 1;
    let at = c.path[n];
    let fwd = c.heading[n];
    let right = right_of(fwd);
    let yaw = yaw_of(fwd);
    let gy = ground(at.x, at.z);
    let span = c.half_width;
    for s in [-1, 1] {
        let foot = at.add(right.scale(s * span));
        b.push(
            Mesh::Bevel,
            &placed(
                v3(foot.x, gy + int(19), foot.z),
                yaw,
                v3(int(2), int(38), int(2)),
            ),
            STEEL,
            0,
        );
        for k in 0..10 {
            let p = foot
                .sub(right.scale(s * cm(110)))
                .add(v3(0, gy + int(2 + k * 4), 0));
            b.push(
                Mesh::Octa,
                &placed(p, 0, v3(cm(50), cm(50), cm(50))),
                if k % 2 == 0 { LAMP } else { BANNER },
                ONE,
            );
        }
    }
    let cells = 16;
    for k in 0..cells {
        for row in 0..2 {
            let x = -span + (2 * span) * k / cells + span / cells;
            let p = at
                .add(right.scale(x))
                .add(v3(0, gy + int(34) + row * int(2), 0));
            let colour = if (k + row) % 2 == 0 {
                BANNER
            } else {
                BANNER_DARK
            };
            b.push(
                Mesh::Cube,
                &placed(p, yaw, v3(2 * span / cells, int(2), cm(40))),
                colour,
                ONE / 3,
            );
        }
    }
    b.push(
        Mesh::Cube,
        &placed(
            v3(at.x, gy + int(37), at.z).add(fwd.scale(cm(30))),
            yaw,
            v3(span * 2, cm(30), cm(20)),
        ),
        LAMP,
        ONE,
    );
    b.push(
        Mesh::Cube,
        &placed(
            v3(at.x, gy + cm(5), at.z),
            yaw,
            v3(span * 2, cm(10), cm(80)),
        ),
        BANNER,
        ONE / 2,
    );
}

/// The plaza past the finish (where a finished ape coasts to a stop):
/// strings of lights across it, the podium, a fountain, a fruit cart piled
/// with bananas (the terminal closing it off is a block, `terminal`).
fn plaza(b: &mut Batch, c: &Course) {
    let last = c.path.len() - 1;
    let fwd = c.heading[last];
    let right = right_of(fwd);
    let yaw = yaw_of(fwd);
    let at = |s: i32, a: i32| on_ground(c.point(c.length + s).add(right.scale(a)));
    // strings of lights
    for s in [int(24), int(64), int(104)] {
        let ends = [at(s, -c.half_width), at(s, c.half_width)];
        for e in ends {
            b.push(Mesh::Hex, &rod(e, e.add(up(int(9))), cm(30)), POLE, 0);
        }
        let (l, r) = (ends[0].add(up(int(9))), ends[1].add(up(int(9))));
        let sag = |t: i32| up(-mul(mul(4 * t, ONE - t), cm(200)));
        let mut prev = l;
        let bulbs = 20;
        for k in 0..bulbs {
            let t = ONE * (2 * k + 1) / (2 * bulbs);
            let p = l.lerp(r, t).add(sag(t));
            b.push(Mesh::Cyl, &rod(prev, p, cm(6)), BANNER_DARK, 0);
            b.push(
                Mesh::Ico,
                &placed(p.sub(up(cm(25))), 0, v3(cm(40), cm(40), cm(40))),
                [LAMP, BILLBOARDS[0], BILLBOARDS[1], BILLBOARDS[2]][k as usize % 4],
                ONE,
            );
            prev = p;
        }
        b.push(Mesh::Cyl, &rod(prev, r, cm(6)), BANNER_DARK, 0);
    }
    // the podium
    let p = at(int(46), 0);
    for (x, h, colour) in [
        (0, int(3), GOLD),
        (-cm(420), int(2), SILVER),
        (cm(420), cm(130), BRONZE),
    ] {
        let q = p.add(right.scale(x));
        b.push(
            Mesh::Cube,
            &placed(q.add(up(h / 2)), yaw, v3(int(4), h, int(4))),
            colour,
            ONE / 6,
        );
        b.push(
            Mesh::Cube,
            &placed(q.add(up(h + cm(5))), yaw, v3(cm(420), cm(10), cm(420))),
            BANNER,
            ONE / 6,
        );
    }
    // the fountain: a stone basin, lit water and its jets
    let f = at(int(84), 0);
    b.push(
        Mesh::Hex,
        &placed(f.add(up(cm(40))), 0, v3(int(14), cm(80), int(14))),
        STONE,
        0,
    );
    b.push(
        Mesh::Hex,
        &placed(f.add(up(cm(70))), 0, v3(int(13), cm(30), int(13))),
        rgb(90, 170, 230),
        ONE / 2,
    );
    b.push(
        Mesh::Hex,
        &placed(f.add(up(cm(150))), 0, v3(int(3), int(2), int(3))),
        STONE,
        0,
    );
    for k in 0..6 {
        let a = k * deg(60);
        let foot = scene::around(f, a, int(4), cm(80));
        let tip = scene::around(f, a, int(2), int(5));
        b.push(
            Mesh::Cyl,
            &rod(foot, tip, cm(20)),
            rgb(200, 236, 255),
            ONE * 3 / 5,
        );
    }
    b.push(
        Mesh::Cyl,
        &rod(f.add(up(int(2))), f.add(up(int(8))), cm(40)),
        rgb(200, 236, 255),
        ONE * 3 / 5,
    );
    // the fruit cart, with a striped umbrella and its bananas
    let q = at(int(46), int(13));
    b.push(
        Mesh::Bevel,
        &placed(q.add(up(cm(100))), yaw, v3(int(3), cm(110), int(2))),
        rgb(200, 200, 196),
        0,
    );
    for x in [-cm(110), cm(110)] {
        b.push(
            Mesh::Hex,
            &Affine::translate(q.add(right.scale(x)).add(up(cm(40))))
                .then(&Affine::rot_y(yaw))
                .then(&Affine::rot_x(deg(90)))
                .then(&Affine::scale(v3(cm(80), cm(14), cm(80)))),
            rgb(30, 30, 30),
            0,
        );
    }
    b.push(
        Mesh::Cyl,
        &rod(q.add(up(cm(150))), q.add(up(int(4))), cm(10)),
        STEEL,
        0,
    );
    b.push(
        Mesh::Cone,
        &placed(q.add(up(cm(430))), 0, v3(int(4), cm(80), int(4))),
        rgb(230, 70, 50),
        ONE / 6,
    );
    for k in 0..12 {
        let a = k * fx::TURN / 12;
        let mid = scene::around(q, a, cm(60) + (k % 2) * cm(30), cm(170));
        let along = crate::geom::facing(a + deg(90), 0).scale(cm(40));
        b.push(
            Mesh::Cube,
            &rod(mid.sub(along), mid.add(along), cm(18)),
            BANANA,
            ONE / 6,
        );
    }
}

// ---- the vines as wires ----

/// The City's vines are electric cables: dark, with a faint glow at dusk.
pub const WIRE: Rgb = rgb(60, 156, 196);
/// The colourblind mode's wires (TRIAGE T33): a vivid cyan, brighter than
/// anything in the muted world.
pub const WIRE_VIVID: Rgb = rgb(90, 240, 255);
/// An item wire's sparks (TRIAGE T46 on the City): violet, round a white
/// heart, apart from the in-reach yellow, the catch-up coral and the
/// wires' cyan; three up the wire.
pub const SPARK: Rgb = rgb(170, 100, 255);
pub const SPARK_HEART: Rgb = rgb(255, 252, 236);
/// While an item is taken: a dull insulator where each spark was.
pub const INSULATOR: Rgb = rgb(110, 112, 120);

/// A spark at `at`: three crossed bright rods round a white heart, turning.
pub fn spark(b: &mut Batch, at: V3, spin: i32) {
    for k in 0..3 {
        let a = spin + k * deg(60);
        let dir = v3(
            mul(fx::cos(a), ONE),
            mul(sin(a), ONE * 3 / 4),
            mul(sin(a + deg(45)), ONE / 2),
        )
        .norm();
        b.push(
            Mesh::Cube,
            &rod(at.sub(dir.scale(cm(60))), at.add(dir.scale(cm(60))), cm(12)),
            SPARK,
            ONE,
        );
    }
    b.push(
        Mesh::Octa,
        &placed(at, spin, v3(cm(44), cm(44), cm(44))),
        SPARK_HEART,
        ONE,
    );
}

// ---- the preview spot ----

/// A place on the City to show an ape on the selection screen: a street
/// corner a little way in, at dusk with the buildings behind. The feet's
/// point on the ground and the way the ape faces, and a camera on it.
pub fn preview(c: &Course) -> (V3, i32, Camera) {
    let s = int(60).min(c.length);
    let fwd = c.heading_at(s);
    let right = right_of(fwd);
    let feet = on_ground(c.point(s).add(right.scale(c.half_width / 3)));
    // the ape faces back toward the camera, which looks a little up the street
    let eye = feet
        .add(fwd.scale(-cm(290)))
        .add(right.scale(-cm(60)))
        .add(up(cm(125)));
    let target = feet.add(up(cm(62)));
    let yaw = yaw_of(eye.sub(feet).flat());
    (
        feet,
        yaw,
        Camera {
            eye,
            fwd: target.sub(eye).norm(),
            fov: deg(34),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::CarKind;
    use crate::map::Map;
    use crate::render::{DRAW_VALUES, DYNAMIC_BUFFER, INSTANCE_VALUES, STATIC_BUFFER};

    #[test]
    fn the_river_is_drawn_on_the_course_only_where_there_is_water() {
        // the river's band crosses the whole city; the first street passes
        // through it near the first left turn, where the ground was drawn
        // as riverbed under water the ape could not swim in
        let c = crate::map::Map::City.course();
        let r = c.city.as_ref().unwrap().river.unwrap();
        let f = RiverFrame::of(&c, Some(r));
        let mut wet = 0;
        for (i, p) in c.path.iter().enumerate() {
            let s = i as i32 * crate::course::STEP;
            if f.river_here(&c, *p) {
                wet += 1;
                assert!(
                    c.water(*p).is_some(),
                    "river drawn at {} m with no water",
                    s / ONE
                );
            }
        }
        assert!(wet > 0, "the crossing has its river");
    }

    #[test]
    fn grid_lines_meet_the_banks_exactly() {
        let b = breaks(-int(50), int(80), int(12), &[0, int(31)]);
        assert!(b.contains(&0) && b.contains(&int(31)));
        assert!(b.windows(2).all(|w| w[1] > w[0] && w[1] - w[0] <= int(16)));
    }

    /// Every instance in `values` (centre, colour, glow).
    fn shapes(values: &[i32]) -> Vec<(V3, Rgb, i32)> {
        values
            .chunks(INSTANCE_VALUES)
            .map(|i| (v3(i[3], i[7], i[11]), [i[12], i[13], i[14]], i[15]))
            .collect()
    }

    fn race_on(c: Course) -> crate::race::Race {
        let e = crate::race::Entry {
            name: "P".into(),
            kind: crate::rules::Kind::Chimpanzee,
            driver: crate::race::Driver::Local,
        };
        crate::race::Race::new(c, &[e], false, 1)
    }

    #[test]
    fn the_city_draws_its_buildings_and_cars_and_no_mushrooms() {
        let c = Map::City.course();
        let city = c.city.as_ref().unwrap();
        let s = scenery(&c, city);
        let all = shapes(&s.values);
        // a body for every building, standing on its block
        for b in &city.blocks {
            assert!(
                all.iter()
                    .any(|(p, _, _)| p.flat().sub(b.centre.flat()).len() < int(1)
                        && p.y > b.centre.y),
                "no body for the block at {:?}",
                b.centre
            );
        }
        // each parked car's yellow, or its paint, at its place; no
        // mushroom cap anywhere (cars and boats bounce instead)
        let cabs = city
            .cars
            .iter()
            .filter(|car| car.drive.is_none() && car.kind == CarKind::Cab)
            .count();
        let yellow = all
            .iter()
            .filter(|(_, colour, _)| *colour == rgb(255, 196, 30))
            .count();
        assert!(
            cabs > 0 && yellow >= cabs * 2,
            "{cabs} cabs, {yellow} yellow shapes"
        );
        assert!(!all.iter().any(|(_, colour, _)| *colour == rgb(238, 58, 96)));
        // the bridge's towers and cables are there
        let br = city.bridge.as_ref().unwrap();
        let granite = all
            .iter()
            .filter(|(_, colour, _)| *colour == GRANITE)
            .count();
        assert!(granite >= br.towers.len() * 3, "{granite}");
    }

    #[test]
    fn the_city_is_a_few_thousand_shapes_near_any_point() {
        let c = Map::City.course();
        let s = crate::scene::scenery(&c);
        let n = s.values.len() / INSTANCE_VALUES;
        assert!(n > 5_000 && n < 26_000, "{n}");
        let mut most = 0;
        for k in 0..c.path.len() / 25 {
            let d = s.draws_near(c.path[k * 25]);
            let m: u32 = d
                .chunks(DRAW_VALUES)
                .filter(|d| d[1] != Mesh::Terrain as u32)
                .map(|d| d[3])
                .sum();
            most = most.max(m);
        }
        assert!(most < 8_000, "{most} drawn at once");
        // the ground: whole triangles facing up, every vertex in one chunk
        assert_eq!(s.terrain.len() % (crate::mesh::VERTEX_VALUES * 3), 0);
        let mut drawn = 0;
        for ch in &s.chunks {
            for d in ch.draws.chunks(DRAW_VALUES) {
                assert_eq!(d[0], STATIC_BUFFER);
                if d[1] == Mesh::Terrain as u32 {
                    drawn += d[5];
                }
            }
        }
        assert_eq!(drawn as usize, s.terrain.len() / crate::mesh::VERTEX_VALUES);
    }

    #[test]
    fn moving_cars_are_drawn_where_their_bouncers_are() {
        let c = Map::City.course();
        let city = c.city.as_ref().unwrap();
        let (k, car) = city
            .cars
            .iter()
            .enumerate()
            .find(|(_, car)| car.drive.is_some())
            .expect("a moving car");
        for tick in [0, 600, 1800] {
            let (at, _) = c.car_at(k, tick);
            let mut b = Batch::new();
            cars_dynamic(&mut b, &c, city, at, tick);
            let (v, _) = b.finish(DYNAMIC_BUFFER);
            let near = shapes(&v)
                .iter()
                .filter(|(p, _, _)| p.flat().sub(at.flat()).len() < int(7))
                .count();
            assert!(near >= 8, "tick {tick}: {near} shapes at the car");
            // and its bouncer is over it
            let m = c.bouncer(car.bouncer, tick);
            assert!(m.top.flat().sub(at.flat()).len() < cm(10));
        }
    }

    #[test]
    fn city_item_wires_spark_and_go_dull_while_taken() {
        // TRIAGE T46 on the City: sparks, not flowers; none with no items
        let mut race = race_on(Map::City.course());
        let v = race.crate_vines[0];
        let a = race.course.anchor(v, race.tick);
        let len = race.course.vines[v].length;
        let mid = a.sub(v3(0, len / 2, 0));
        let seen = |race: &crate::race::Race| {
            let eye = mid.add(v3(0, 0, -int(20)));
            let cam = Camera {
                eye,
                fwd: mid.sub(eye).norm(),
                fov: deg(70),
            };
            let apes: Vec<V3> = race.racers.iter().map(|r| r.ape.pos).collect();
            let (vines, _) = crate::scene::dynamic(&crate::scene::Frame {
                race,
                cam: &cam,
                apes: &apes,
                local: Some(0),
                tick: 10,
                chains: &[],
                vivid: false,
            });
            let (v, _) = vines.finish(DYNAMIC_BUFFER);
            let near = |colour: Rgb| {
                shapes(&v)
                    .iter()
                    .filter(|(p, c, _)| *c == colour && p.flat().sub(a.flat()).len() < int(3))
                    .count()
            };
            (near(SPARK), near(INSULATOR), near(crate::scene::PETAL))
        };
        let (sparks, dull, petals) = seen(&race);
        assert_eq!(
            (sparks, dull, petals),
            (9, 0, 0),
            "three sparks of three rods"
        );
        race.crates[0] = 5;
        assert_eq!(seen(&race), (0, 3, 0), "taken: three insulators");
        race.crates[0] = 0;
        race.items = crate::items::ItemSet::NONE;
        assert_eq!(seen(&race), (0, 0, 0), "no items: no marks");
    }

    #[test]
    fn helicopters_follow_their_vines() {
        let c = Map::City.course();
        let city = c.city.as_ref().unwrap();
        let h = city
            .hangers
            .iter()
            .find(|h| h.kind == HangerKind::Helicopter)
            .expect("a helicopter");
        let a = c.anchor(h.vine, 0);
        let mut b = Batch::new();
        hangers_dynamic(&mut b, &c, city, a, 0, 0);
        let (v, _) = b.finish(DYNAMIC_BUFFER);
        let over = shapes(&v)
            .iter()
            .filter(|(p, _, _)| p.flat().sub(h.from.flat()).len() < int(8) && p.y > a.y)
            .count();
        assert!(over >= 12, "{over} shapes over the vine");
    }

    #[test]
    fn the_preview_stands_in_the_street_facing_the_camera() {
        let c = Map::City.course();
        let city = c.city.as_ref().unwrap();
        let (feet, yaw, cam) = preview(&c);
        assert_eq!(feet.y, c.floor(feet.x, feet.z));
        for b in &city.blocks {
            let d = cam.eye.flat().sub(b.centre.flat());
            let inside =
                d.dot(b.fwd).abs() < b.half_len && d.dot(right_of(b.fwd)).abs() < b.half_width;
            assert!(!inside, "the camera is inside a building");
        }
        let (x, y) = cam
            .screen(ONE * 16 / 9, feet.add(v3(0, cm(60), 0)))
            .unwrap();
        assert!((x - 50 * ONE).abs() < 10 * ONE && (y - 50 * ONE).abs() < 15 * ONE);
        let facing = crate::geom::facing(yaw, 0);
        assert!(facing.dot(cam.eye.sub(feet).flat().norm()) > ONE * 9 / 10);
    }
}
