//! Drawing the Ski Slope (2026-10-13, group B): **noon, clear, not
//! snowing.** The hour and the weather come first (MAPS "Art lessons"):
//! the sun stands high (about 65°) behind the skiers' backs, the light is
//! a crisp white, the sky a deep saturated blue paling to a hazy blue at
//! the horizon, the air clear (the fog is pale blue and far out), the snow
//! bright white with cool blue shade, and the snow throws light back up
//! under everything. Falling snow is the Weather setting's, not this
//! module's.
//!
//! A palette no other map uses: white snow, blue sky, near-black-green
//! conifers, safety-orange fences, red and yellow lift chairs. Drawn from
//! the course's `SkiLayout` (`ski.rs`) and the shared course data, in the
//! faceted low-poly style: the groomed piste and its corduroy, the banks
//! and the forest beyond the fences, the forks' islands, the chairlifts,
//! the conifers and helicopters the vines hang from, the landmarks, the
//! finish village, and round the eye each frame the sky's dome, a range
//! of snowy mountains and one iconic peak.
//!
//! The scene (`scene.rs`) calls in here for a course with a `SkiLayout`;
//! the Jungle's and the City's drawing do not change.

use crate::course::{Course, OUTRO, STEP};
use crate::fx::{self, cm, deg, int, mul, sin, Dice, ONE};
use crate::geom::{facing, right_of, v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, placed, rgb, rod, shade, Batch, Camera, Rgb};
use crate::scene::{self, Look, Scenery, CHUNK};
use crate::ski::{Conifer, Hanger, HangerKind, Landmark, LandmarkKind, Lift, SkiLayout};

// ---- the look: noon ----

/// The sun: its height over the horizon, and its bearing from the
/// course's downhill heading (behind the skiers and off to the left, so
/// the faces turned up the hill toward the chase camera are lit).
pub const SUN_HIGH: i32 = deg(64);
const SUN_BEARING: i32 = deg(180 + 38);

/// The noon light: a crisp, slightly cool white, bright.
const LIGHT: Rgb = rgb(272, 264, 248);
/// The sky's light from above (the ambient overhead): a clear blue. The
/// dome's zenith is darker and more saturated (`ZENITH`).
const SKY: Rgb = rgb(140, 168, 218);
/// The light thrown back up by the snow.
const BOUNCE: Rgb = rgb(206, 218, 238);
/// The clear air's haze at the horizon: a pale, bright blue; the clear
/// colour behind everything.
pub const FOG: Rgb = rgb(200, 222, 244);
/// The sky straight overhead: a deep, saturated blue.
pub const ZENITH: Rgb = rgb(34, 92, 214);

/// The Ski Slope at noon.
pub fn look() -> Look {
    Look {
        sky: SKY,
        fog: FOG,
        // the clear mountain air: the haze starts late and ends far
        fog_range: (int(110), int(440)),
        light_dir: facing(SUN_BEARING, SUN_HIGH),
        light: LIGHT,
        ground: BOUNCE,
        rim: [0; 3],
        far: crate::render::FAR,
    }
}

/// The Ski Slope's look for its course: the sun stands behind the skiers,
/// from the hill's fall line.
pub fn look_for(c: &Course) -> Look {
    Look {
        light_dir: sun_dir(c),
        ..look()
    }
}

/// Toward the sun.
pub fn sun_dir(c: &Course) -> V3 {
    facing(downhill_yaw(c) + SUN_BEARING, SUN_HIGH)
}

/// The heading of the course as a whole, start to finish (flat).
fn downhill_yaw(c: &Course) -> i32 {
    let last = c.path[c.path.len() - 1];
    yaw_of(last.sub(c.path[0]).flat())
}

// ---- the palette ----

/// The vines: ropes in a hot pink, apart from everything else on the hill
/// (white, blue, near-black green, safety orange, red and yellow), from
/// the in-reach yellow, the catch-up coral and the item flowers' violet.
pub const ROPE_PINK: Rgb = rgb(255, 46, 156);
/// The colourblind mode's ropes: a vivid magenta, brighter than anything
/// in the muted world.
pub const ROPE_VIVID: Rgb = rgb(255, 70, 220);
/// How many times a rope's width its soft glow is across.
pub const ROPE_HALO: i32 = 3;

const SNOW: Rgb = rgb(236, 240, 246);
const GROOMED: Rgb = rgb(232, 235, 242);
const POWDER: Rgb = rgb(244, 247, 252);
const SNOW_DEEP: Rgb = rgb(222, 230, 246);
const NEEDLES: [Rgb; 4] = [
    rgb(20, 46, 36),
    rgb(26, 54, 38),
    rgb(16, 40, 34),
    rgb(32, 60, 44),
];
const BARK: Rgb = rgb(70, 48, 38);
const SNOW_ON: Rgb = rgb(244, 248, 255);
/// The fences: safety orange, the posts nearly black.
pub const FENCE: Rgb = rgb(255, 104, 18);
const FENCE_POST: Rgb = rgb(36, 38, 44);
/// The lifts: galvanised towers, dark haul ropes, red and yellow chairs.
const STEEL: Rgb = rgb(150, 160, 172);
const STEEL_DARK: Rgb = rgb(70, 76, 86);
const ROPE: Rgb = rgb(58, 60, 68);
pub const CHAIRS: [Rgb; 2] = [rgb(214, 34, 40), rgb(252, 198, 24)];
const CHAIR_FRAME: Rgb = rgb(52, 54, 60);
const HOUSING: Rgb = rgb(232, 234, 238);
const HOUSING_TRIM: Rgb = rgb(206, 30, 38);
/// The pads round a lift tower's foot on the piste.
const PAD: Rgb = rgb(30, 70, 170);
const CONCRETE: Rgb = rgb(172, 170, 166);
const GLASS: Rgb = rgb(70, 112, 156);
const TIMBER: Rgb = rgb(132, 82, 46);
const TIMBER_DARK: Rgb = rgb(92, 56, 34);
const STONE: Rgb = rgb(132, 128, 124);
const WARM_WINDOW: Rgb = rgb(255, 214, 130);
const ROCK: Rgb = rgb(96, 98, 110);
const ICE: Rgb = rgb(176, 226, 246);
const ICE_DEEP: Rgb = rgb(110, 190, 230);
const COAL: Rgb = rgb(26, 26, 30);
const CARROT: Rgb = rgb(250, 120, 30);
const SCARF: Rgb = rgb(200, 24, 48);
const RED: Rgb = rgb(214, 30, 40);
const WHITE: Rgb = rgb(250, 250, 250);
const GOLD: Rgb = rgb(250, 200, 60);
const SILVER: Rgb = rgb(206, 212, 224);
const BRONZE: Rgb = rgb(206, 128, 70);
const BANANA: Rgb = rgb(255, 214, 56);
const FLAGS: [Rgb; 5] = [
    rgb(214, 30, 40),
    rgb(30, 80, 190),
    rgb(252, 198, 24),
    rgb(30, 150, 80),
    rgb(250, 250, 250),
];
/// The far mountains: blue-grey rock under white snow.
const MOUNTAIN_ROCK: Rgb = rgb(108, 116, 136);
const PEAK_ROCK: Rgb = rgb(78, 72, 78);

fn up(h: i32) -> V3 {
    v3(0, h, 0)
}

/// A box (or any unit mesh) centred at `at`, `w` along `across`, `h` along
/// `upward` and `d` along `out`.
fn panel(at: V3, across: V3, upward: V3, out: V3, w: i32, h: i32, d: i32) -> Affine {
    Affine {
        c0: across.scale(w),
        c1: upward.scale(h),
        c2: out.scale(d),
        t: at,
    }
}

// ---- where things are on the hill ----

/// The corridor's edges `s` metres along, metres right of the spine: the
/// outer edges `l` and `r`, and (inside a fork, when the branches have
/// parted) the island's edges `il` and `ir`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edges {
    pub l: i32,
    pub r: i32,
    pub island: Option<(i32, i32)>,
}

/// The island is drawn (fenced, banked up) once it is this wide.
const ISLAND_MIN: i32 = int(1);

pub fn edges(c: &Course, s: i32) -> Edges {
    let [(a, ha), (b, hb)] = c.lanes(s);
    let (l, r) = ((a - ha).min(b - hb), (a + ha).max(b + hb));
    let (il, ir) = (a + ha, b - hb);
    Edges {
        l,
        r,
        island: (ir - il >= ISLAND_MIN).then_some((il, ir)),
    }
}

/// The point `a` metres right of the spine, `s` metres along (flat).
fn spot(c: &Course, s: i32, a: i32) -> V3 {
    c.point(s).add(right_of(c.heading_at(s)).scale(a))
}

/// How high the snow is drawn at `a` metres right of the spine, `s`
/// along: the hill (`Course::terrain`, the floor the apes stand on) inside
/// the fences; outside, banks rising into the forest, lumpy with powder;
/// on a fork's island, a low mound.
pub fn snow_height(c: &Course, s: i32, a: i32) -> i32 {
    let p = spot(c, s, a);
    let g = c.terrain(p.x, p.z);
    let e = edges(c, s);
    let out = if a < e.l {
        e.l - a
    } else if a > e.r {
        a - e.r
    } else {
        0
    };
    if out > 0 {
        return g + bank(out) + lumps(p, out);
    }
    if let Some((il, ir)) = e.island {
        if a > il && a < ir {
            let inside = (a - il).min(ir - a);
            let k = fx::div((inside - int(2)).max(0), int(9)).min(ONE);
            let k = mul(mul(k, k), 3 * ONE - 2 * k);
            return g + mul(k, int(3)) + lumps(p, inside) / 2;
        }
    }
    g
}

/// The banks' rise `out` metres beyond a fence: flat for the fence's
/// footing, then curving up into the trees (about 30 m at 110 m out).
fn bank(out: i32) -> i32 {
    let d = (out - int(3)).max(0);
    let t = fx::div(d, int(107));
    mul(mul(t, t), int(32)) + mul(t, int(4))
}

/// Powder lumps, growing in from nothing `out` metres from the edge.
fn lumps(p: V3, out: i32) -> i32 {
    let k = fx::div(out, int(8)).min(ONE);
    let n = sin(mul(p.x, 2300) + mul(p.z, 700)) + fx::cos(mul(p.z, 1900) - mul(p.x, 500));
    mul(mul(n, cm(45)), k)
}

/// Where `p` is: metres along the spine, and metres right of it.
pub fn locate(c: &Course, p: V3) -> (i32, i32) {
    let f = p.flat();
    let mut best = (i64::MAX, 0);
    for (i, q) in c.path.iter().enumerate().step_by(8) {
        let d = f.sub(*q);
        best = best.min((d.dot64(d), i));
    }
    let i = c.nearest(p, best.1);
    let d = f.sub(c.path[i]);
    let s = i as i32 * STEP + d.dot(c.heading[i]);
    (s, d.dot(right_of(c.heading[i])))
}

/// The snow's drawn height under `p`.
pub fn ground_at(c: &Course, p: V3) -> i32 {
    let (s, a) = locate(c, p);
    snow_height(c, s, a)
}

fn on_snow(c: &Course, p: V3) -> V3 {
    v3(p.x, ground_at(c, p), p.z)
}

// ---- the output: shapes sorted into the course's chunks ----

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

/// Builds the Ski Slope's scenery and snow, chunked.
pub fn scenery(c: &Course, ski: &SkiLayout) -> Scenery {
    build(c, ski).0
}

/// How many scenery shapes each part of the Ski Slope takes (for
/// `examples/scenery_cost.rs`).
pub fn parts(c: &Course) -> Vec<(&'static str, usize)> {
    match &c.ski {
        Some(ski) => build(c, ski).1,
        None => Vec::new(),
    }
}

fn build(c: &Course, ski: &SkiLayout) -> (Scenery, Vec<(&'static str, usize)>) {
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
    fences(&mut s, c);
    tally(&s, "fences");
    for l in &ski.lifts {
        lift(&mut s, c, ski, l);
    }
    tally(&s, "lifts");
    for (k, t) in ski.trees.iter().enumerate() {
        conifer(
            s.at(t.at),
            t.at,
            t.height,
            t.radius,
            4,
            t.shade,
            k as i32 * 9000,
        );
    }
    for h in &ski.hangers {
        hanger(&mut s, c, ski, h);
    }
    tally(&s, "conifers and hangers");
    for m in &ski.landmarks {
        landmark(&mut s, c, m);
    }
    start_deck(&mut s, c);
    finish_arch(&mut s.batches[n - 2], c);
    tally(&s, "landmarks");
    forest(&mut s, c, ski);
    tally(&s, "forest");
    let (terrain, ranges) = terrain(c, n);
    (scene::assemble(c, s.batches, terrain, ranges), parts)
}

// ---- the snow ----

/// Columns of the snow across the course: offsets beyond each outer
/// fence (metres), and the cells across the piste (each lane and the
/// island get a third).
const BANK_COLS: [i32; 7] = [3, 9, 18, 32, 52, 78, 112];
const PISTE_CELLS: usize = 12;
/// Rows of the snow along the course, every this many metres.
const ROW: i32 = int(5);

/// The snow under and round the course, its facets: the groomed piste in
/// the groomer's wide stripes, the powder of the banks and islands; and
/// for each chunk its vertex range.
fn terrain(c: &Course, n: usize) -> (Vec<i32>, Vec<(usize, u32, u32)>) {
    let mut cells: Vec<Vec<i32>> = vec![Vec::new(); n];
    let cols = |s: i32| -> Vec<i32> {
        let e = edges(c, s);
        let mut v = Vec::with_capacity(2 * BANK_COLS.len() + PISTE_CELLS + 1);
        for b in BANK_COLS.iter().rev() {
            v.push(e.l - int(*b));
        }
        // the piste: a third each to the left lane, the island, the right
        // lane, eased from even cells as the island opens
        let even = |k: usize| e.l + (e.r - e.l) * k as i32 / PISTE_CELLS as i32;
        let (il, ir) = match e.island {
            Some(i) => i,
            None => (even(PISTE_CELLS / 3), even(2 * PISTE_CELLS / 3)),
        };
        let third = PISTE_CELLS / 3;
        for k in 0..=PISTE_CELLS {
            let x = if k <= third {
                e.l + (il - e.l) * k as i32 / third as i32
            } else if k <= 2 * third {
                il + (ir - il) * (k - third) as i32 / third as i32
            } else {
                ir + (e.r - ir) * (k - 2 * third) as i32 / third as i32
            };
            v.push(x);
        }
        for b in BANK_COLS {
            v.push(e.r + int(b));
        }
        v
    };
    let point = |s: i32, a: i32| {
        let q = spot(c, s, a);
        v3(q.x, snow_height(c, s, a), q.z)
    };
    let mut s = -int(90);
    let mut row = 0usize;
    let end = c.length + OUTRO + int(60);
    let (mut c0, mut p0) = {
        let cs = cols(s);
        let ps: Vec<V3> = cs.iter().map(|a| point(s, *a)).collect();
        (cs, ps)
    };
    while s < end {
        let s1 = s + ROW;
        let c1 = cols(s1);
        let p1: Vec<V3> = c1.iter().map(|a| point(s1, *a)).collect();
        let chunk = chunk_along(c, s + ROW / 2, n);
        let e = edges(c, s + ROW / 2);
        for k in 0..c0.len() - 1 {
            let mid_a = (c0[k] + c0[k + 1] + c1[k] + c1[k + 1]) / 4;
            let piste = k >= BANK_COLS.len() && k < BANK_COLS.len() + PISTE_CELLS;
            let on_island = e.island.is_some_and(|(il, ir)| mid_a > il && mid_a < ir);
            let base = if !piste || on_island {
                let out = (e.l - mid_a).max(mid_a - e.r);
                if out > int(30) {
                    SNOW_DEEP
                } else {
                    POWDER
                }
            } else if k % 2 == 0 {
                // the groomer's passes, alternately a touch darker
                GROOMED
            } else {
                shade(GROOMED, 97)
            };
            let (a, b, cc, d) = (p0[k], p0[k + 1], p1[k + 1], p1[k]);
            for (h, tri) in [[a, d, cc], [a, cc, b]].into_iter().enumerate() {
                let spread: usize = if piste && !on_island { 3 } else { 6 };
                let wobble =
                    100 - (spread / 2) as i32 + ((row * 7 + k * 13 + h * 5) % (spread + 1)) as i32;
                scene::push_ground(&mut cells[chunk], tri, shade(base, wobble));
            }
        }
        (c0, p0) = (c1, p1);
        s = s1;
        row += 1;
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

/// The chunk that a point `s` metres along the course falls in.
fn chunk_along(c: &Course, s: i32, n: usize) -> usize {
    if s > c.length {
        n - 1
    } else {
        ((s.max(0) / STEP) as usize / CHUNK).min(n - 2)
    }
}

// ---- the fences ----

/// Fence posts stand this far apart, the mesh this high.
const POST_EVERY: i32 = int(5);
const FENCE_HIGH: i32 = cm(130);

/// The orange plastic fences: along both outer edges of the corridor from
/// behind the start to past the finish, and round each fork's island.
fn fences(s: &mut Sink, c: &Course) {
    let from = -int(30);
    let to = c.length + int(30);
    for side in [-1, 1] {
        let mut prev: Option<V3> = None;
        let mut at = from;
        while at <= to {
            let e = edges(c, at);
            let a = if side < 0 { e.l - cm(40) } else { e.r + cm(40) };
            let p = spot(c, at, a);
            let p = v3(p.x, c.terrain(p.x, p.z), p.z);
            fence_span(s, prev, p);
            prev = Some(p);
            at += POST_EVERY;
        }
    }
    // round the islands: both inner edges while the island is open, joined
    // at its two noses
    let mut prev: [Option<V3>; 2] = [None, None];
    let mut at = 0;
    while at <= c.length {
        match edges(c, at).island {
            Some((il, ir)) => {
                let ends = [il + cm(40), ir - cm(40)].map(|a| {
                    let p = spot(c, at, a);
                    v3(p.x, c.terrain(p.x, p.z), p.z)
                });
                if prev[0].is_none() {
                    // the nose where the branches part
                    fence_span(s, Some(ends[0]), ends[1]);
                }
                for k in 0..2 {
                    fence_span(s, prev[k], ends[k]);
                    prev[k] = Some(ends[k]);
                }
            }
            None => {
                if let (Some(a), Some(b)) = (prev[0], prev[1]) {
                    fence_span(s, Some(a), b);
                }
                prev = [None, None];
            }
        }
        at += POST_EVERY / 2;
    }
}

/// One post at `p` and the mesh from the last post `from` to it: a band
/// of orange net with a heavier top and bottom edge, sagging a little.
fn fence_span(s: &mut Sink, from: Option<V3>, p: V3) {
    let b = s.at(p);
    b.push(
        Mesh::Cyl,
        &rod(p.sub(up(cm(20))), p.add(up(FENCE_HIGH + cm(15))), cm(7)),
        FENCE_POST,
        0,
    );
    let Some(q) = from else { return };
    let d = p.sub(q);
    if d.len() < cm(30) {
        return;
    }
    let across = right_of(d.flat().norm());
    let mid = q.lerp(p, ONE / 2);
    // the mesh, a thin orange band
    let (a0, a1) = (q.add(up(cm(18))), p.add(up(cm(18))));
    let f = Affine {
        c0: a1.sub(a0),
        c1: up(FENCE_HIGH - cm(30)),
        c2: across.scale(cm(3)),
        t: a0.lerp(a1, ONE / 2).add(up((FENCE_HIGH - cm(30)) / 2)),
    };
    b.push(Mesh::Cube, &f, FENCE, ONE / 10);
    let _ = mid;
    // the top edge: a darker cord
    b.push(
        Mesh::Cube,
        &rod(q.add(up(FENCE_HIGH)), p.add(up(FENCE_HIGH)), cm(6)),
        shade(FENCE, 70),
        0,
    );
}

// ---- the conifers ----

/// A snowy conifer standing at `foot`: a short trunk and `tiers` skirts
/// of near-black-green needles, each with snow lying on its upper part.
pub fn conifer(
    b: &mut Batch,
    foot: V3,
    height: i32,
    radius: i32,
    tiers: i32,
    shade_of: u8,
    spin: i32,
) {
    let tiers = tiers.max(1);
    let colour = NEEDLES[shade_of as usize % NEEDLES.len()];
    if tiers >= 3 {
        b.push(
            Mesh::Cyl,
            &rod(foot.sub(up(cm(30))), foot.add(up(height / 4)), radius / 5),
            BARK,
            0,
        );
    }
    // each tier overlaps the next by nearly half
    let th = mul(
        height * 17 / 20,
        fx::div(ONE, ONE + (tiers - 1) * ONE * 11 / 20),
    );
    for k in 0..tiers {
        let bottom = height * 3 / 20 + k * mul(th, ONE * 11 / 20);
        let r = radius * (100 - k * 60 / tiers) / 100;
        let mid = foot.add(up(bottom + th / 2));
        let yaw = spin + k * deg(23);
        b.push(
            Mesh::Spike,
            &placed(mid, yaw, v3(r * 2, th, r * 2)),
            colour,
            0,
        );
        // the snow on it: the same point, fatter and shorter
        // (the tier above hides its upper half, so the snow shows as a
        // white band round the tier's middle)
        let cover = 11 + (spin.rem_euclid(5) + k) % 4;
        let sh = th * cover / 20;
        let sr = r * 2 * cover / 20 * 107 / 100;
        let top = foot.add(up(bottom + th));
        b.push(
            Mesh::Spike,
            &placed(top.sub(up(sh / 2)), yaw, v3(sr, sh, sr)),
            SNOW_ON,
            0,
        );
    }
}

/// The forest beyond the fences: a dense edge of full trees close in,
/// thinning to simpler ones on the banks, and none on the lifts' lines
/// or round the landmarks.
fn forest(s: &mut Sink, c: &Course, ski: &SkiLayout) {
    let mut dice = Dice(0x0005_C1F0_2E57);
    let clear = |p: V3| -> bool {
        for l in &ski.lifts {
            if seg_distance(p, l.bottom, l.top) < int(9) {
                return false;
            }
            for e in [l.bottom, l.top] {
                if p.flat().sub(e.flat()).len() < int(16) {
                    return false;
                }
            }
        }
        for m in &ski.landmarks {
            if p.flat().sub(m.at.flat()).len() < landmark_clearance(m.kind) {
                return false;
            }
            // a clearing from the run to it, so it is seen on the way
            for back in [int(10), int(40), int(80)] {
                let from = c.point(m.s - back);
                if seg_distance(p, m.at, from) < landmark_clearance(m.kind) * 4 / 5 {
                    return false;
                }
            }
        }
        for t in &ski.trees {
            if p.flat().sub(t.at.flat()).len() < t.radius * 2 {
                return false;
            }
        }
        true
    };
    let mut at = -int(80);
    let end = c.length + OUTRO - int(20);
    while at < end {
        for side in [-1, 1] {
            let e = edges(c, at);
            let edge = if side < 0 { e.l } else { e.r };
            // (how far out, the tiers, the height) for this step's trees
            let mut picks: Vec<(i32, i32, i32)> = Vec::new();
            picks.push((dice.range(5, 10), 3, dice.range(11, 17)));
            if dice.below(3) > 0 {
                picks.push((dice.range(12, 22), 3, dice.range(12, 19)));
            }
            picks.push((dice.range(24, 44), 2, dice.range(13, 20)));
            for _ in 0..2 {
                picks.push((dice.range(46, 118), 1, dice.range(14, 22)));
            }
            for (out, tiers, h) in picks {
                let along = at + int(dice.range(-2, 2));
                let a = edge + side * int(out);
                // past the finish the village's clearing stays open
                if along > c.length + int(20) && out < 40 {
                    continue;
                }
                let p = spot(c, along, a);
                // on a bend, the other side's corridor may be near
                let (s2, a2) = locate(c, p);
                let e2 = edges(c, s2);
                if a2 > e2.l - int(4) && a2 < e2.r + int(4) {
                    continue;
                }
                if !clear(p) {
                    continue;
                }
                let g = snow_height(c, s2, a2) - cm(40);
                let foot = v3(p.x, g, p.z);
                let r = int(h) * (26 + dice.range(0, 8)) / 100;
                let shade_of = dice.below(4) as u8;
                let spin = dice.range(0, fx::TURN);
                conifer(s.at(foot), foot, int(h), r, tiers, shade_of, spin);
            }
        }
        at += int(7);
    }
}

/// The flat distance from `p` to the segment from `a` to `b`.
fn seg_distance(p: V3, a: V3, b: V3) -> i32 {
    let (p, a, b) = (p.flat(), a.flat(), b.flat());
    let d = b.sub(a);
    let l2 = d.dot64(d);
    if l2 == 0 {
        return p.sub(a).len();
    }
    let t = (p.sub(a).dot64(d) as i128 * ONE as i128 / l2 as i128).clamp(0, ONE as i128) as i32;
    p.sub(a.lerp(b, t)).len()
}

// ---- the chairlifts ----

fn lift(s: &mut Sink, c: &Course, ski: &SkiLayout, l: &Lift) {
    let k = ski.lifts.iter().position(|x| x == l).unwrap_or(0);
    let colour = CHAIRS[k % CHAIRS.len()];
    let fwd = l.dir();
    let len = l.length();
    // where the rope is held: the two bullwheels and each tower, along the
    // line; it runs straight between them (`Lift::rope`)
    let mut held = vec![0];
    for t in &l.towers {
        held.push(t.flat().sub(l.bottom.flat()).dot(fwd).clamp(0, len));
    }
    held.push(len);
    for side in [-1, 1] {
        for w in held.windows(2) {
            let (a, b) = (l.rope(w[0], side), l.rope(w[1], side));
            if a.sub(b).len() > cm(20) {
                s.at(a.lerp(b, ONE / 2))
                    .push(Mesh::Cyl, &rod(a, b, cm(9)), ROPE, 0);
            }
        }
    }
    for t in &l.towers {
        tower(s.at(*t), c, *t, fwd, l);
    }
    for (end, toward, d) in [(l.bottom, fwd, 0), (l.top, fwd.neg(), len)] {
        terminal(s.at(end), c, end, toward, l, d, colour);
    }
    // the chairs, up on the right of the line (facing up the hill) and down
    // on the left, apart from those a vine hangs from (drawn with it)
    let hung: Vec<V3> = ski
        .hangers
        .iter()
        .filter(|h| matches!(h.kind, HangerKind::Chair { lift } if lift == k))
        .map(|h| h.from)
        .collect();
    for side in [-1, 1] {
        for n in 1..l.chairs() {
            let (grip, seat) = l.chair(n, side);
            if hung
                .iter()
                .any(|g| g.flat().sub(grip.flat()).len() < cm(50))
            {
                continue;
            }
            let face = if side > 0 { fwd } else { fwd.neg() };
            chair(s.at(grip), grip, seat, face, colour);
        }
    }
}

/// A lift tower at `foot`: a tall tapering steel column on a concrete
/// footing, its crossarm and the sheave trains under each rope, a red cap.
fn tower(b: &mut Batch, c: &Course, foot: V3, fwd: V3, l: &Lift) {
    let g = ground_at(c, foot).min(foot.y);
    let top = foot.y + l.rope_height;
    let base = v3(foot.x, g, foot.z);
    let yaw = yaw_of(fwd);
    b.push(
        Mesh::Cube,
        &placed(base.add(up(cm(40))), yaw, v3(cm(260), cm(140), cm(260))),
        CONCRETE,
        0,
    );
    b.push(
        Mesh::Taper,
        &rod(base, v3(foot.x, top - cm(70), foot.z), cm(150)),
        STEEL,
        0,
    );
    // on the piste, a padded wrap round its foot
    let (s, a) = locate(c, foot);
    let e = edges(c, s);
    if a > e.l - int(2) && a < e.r + int(2) {
        b.push(
            Mesh::Cyl,
            &placed(base.add(up(cm(130))), yaw, v3(cm(260), cm(260), cm(260))),
            PAD,
            0,
        );
        b.push(
            Mesh::Cyl,
            &placed(base.add(up(cm(200))), yaw, v3(cm(270), cm(40), cm(270))),
            WHITE,
            0,
        );
    }
    let right = right_of(fwd);
    let head = v3(foot.x, top - cm(70), foot.z);
    b.push(
        Mesh::Cube,
        &panel(head, right, V3::UP, fwd, l.gauge + int(2), cm(55), cm(60)),
        STEEL,
        0,
    );
    for side in [-1, 1] {
        let at = head.add(right.scale(side * l.gauge / 2)).add(up(cm(45)));
        b.push(
            Mesh::Cube,
            &panel(at, right, V3::UP, fwd, cm(34), cm(36), int(4)),
            STEEL_DARK,
            0,
        );
    }
    b.push(
        Mesh::Cube,
        &placed(head.add(up(cm(70))), yaw, v3(cm(70), cm(60), cm(70))),
        HOUSING_TRIM,
        0,
    );
}

/// A terminal at `end` (`d` along the line), the line running off
/// `toward`: the bullwheel up at the rope's height under a white housing
/// with a red band, held on a braced steel frame, and the station hut at
/// its foot with a banner in the lift's colour.
fn terminal(b: &mut Batch, c: &Course, end: V3, toward: V3, l: &Lift, d: i32, colour: Rgb) {
    let g = ground_at(c, end).min(end.y);
    let foot = v3(end.x, g, end.z);
    let right = right_of(toward);
    let yaw = yaw_of(toward);
    let wheel_y = l.rope(d, 1).y;
    let wheel = v3(end.x, wheel_y, end.z);
    // the housing over the bullwheel, a little back from the line's end
    let housing = wheel.sub(toward.scale(cm(80))).add(up(cm(40)));
    b.push(
        Mesh::Bevel,
        &panel(
            housing,
            right,
            V3::UP,
            toward,
            l.gauge + int(3),
            cm(320),
            l.gauge + int(2),
        ),
        HOUSING,
        0,
    );
    b.push(
        Mesh::Cube,
        &panel(
            housing.sub(up(cm(110))),
            right,
            V3::UP,
            toward,
            l.gauge + cm(320),
            cm(50),
            l.gauge + cm(220),
        ),
        HOUSING_TRIM,
        0,
    );
    b.push(
        Mesh::Cyl,
        &placed(
            wheel.sub(up(cm(30))),
            yaw,
            v3(l.gauge + cm(40), cm(40), l.gauge + cm(40)),
        ),
        STEEL_DARK,
        0,
    );
    // the station tower under it: concrete below, timber-clad above, with
    // bands of windows and a stair tower up its side
    let leg_top = housing.y - cm(160);
    let rise = leg_top - foot.y;
    let mid = foot.sub(toward.scale(cm(80)));
    let w = l.gauge * 4 / 5;
    let lower = rise / 3;
    b.push(
        Mesh::Bevel,
        &panel(
            mid.add(up(lower / 2 - cm(50))),
            right,
            V3::UP,
            toward,
            w + int(1),
            lower + int(1),
            w + int(1),
        ),
        CONCRETE,
        0,
    );
    b.push(
        Mesh::Cube,
        &panel(
            mid.add(up(lower + (rise - lower) / 2)),
            right,
            V3::UP,
            toward,
            w,
            rise - lower,
            w,
        ),
        TIMBER,
        0,
    );
    let bands = (rise / int(9)).clamp(1, 5);
    for k in 1..=bands {
        let y = lower + (rise - lower) * k / (bands + 1);
        b.push(
            Mesh::Cube,
            &panel(
                mid.add(up(y)),
                right,
                V3::UP,
                toward,
                w + cm(20),
                cm(120),
                w + cm(20),
            ),
            GLASS,
            ONE / 5,
        );
    }
    let stair = mid.add(right.scale(w / 2 + cm(80)));
    b.push(
        Mesh::Cube,
        &panel(
            stair.add(up(rise / 2)),
            right,
            V3::UP,
            toward,
            cm(160),
            rise,
            cm(220),
        ),
        STEEL,
        0,
    );
    // the station hut at its foot: timber, a snowy roof, a window, the
    // lift's banner
    let hut = foot
        .sub(toward.scale(l.gauge + int(4)))
        .add(right.scale(l.gauge));
    b.push(
        Mesh::Cube,
        &placed(hut.add(up(cm(140))), yaw, v3(int(4), cm(320), int(5))),
        TIMBER,
        0,
    );
    b.push(
        Mesh::Wedge,
        &placed(hut.add(up(cm(360))), yaw, v3(cm(460), cm(120), cm(560))),
        SNOW,
        0,
    );
    b.push(
        Mesh::Cube,
        &placed(
            hut.add(up(cm(180))).add(toward.scale(cm(250))),
            yaw,
            v3(cm(260), cm(110), cm(10)),
        ),
        GLASS,
        ONE / 5,
    );
    b.push(
        Mesh::Cube,
        &placed(
            hut.add(up(cm(300))).add(toward.scale(cm(255))),
            yaw,
            v3(cm(380), cm(40), cm(8)),
        ),
        colour,
        ONE / 6,
    );
}

/// A chair hanging from its `grip` on the rope down to its seat at
/// `seat`, facing `face` (flat): the hanger bar, the seat, the back and
/// the safety bar with its footrest.
fn chair(b: &mut Batch, grip: V3, seat: V3, face: V3, colour: Rgb) {
    let right = right_of(face);
    b.push(
        Mesh::Cube,
        &rod(grip, seat.add(up(cm(110))), cm(9)),
        CHAIR_FRAME,
        0,
    );
    b.push(
        Mesh::Cube,
        &placed(grip, yaw_of(face), v3(cm(30), cm(24), cm(40))),
        CHAIR_FRAME,
        0,
    );
    // the bar bends back over the seat to the back's top
    let top = seat.add(up(cm(110)));
    let back_top = seat.sub(face.scale(cm(30))).add(up(cm(70)));
    b.push(Mesh::Cube, &rod(top, back_top, cm(8)), CHAIR_FRAME, 0);
    b.push(
        Mesh::Bevel,
        &panel(seat, right, V3::UP, face, cm(190), cm(14), cm(60)),
        colour,
        0,
    );
    b.push(
        Mesh::Bevel,
        &panel(
            seat.sub(face.scale(cm(32))).add(up(cm(36))),
            right,
            V3::UP.add(face.scale(-ONE / 5)).norm(),
            face,
            cm(190),
            cm(64),
            cm(10),
        ),
        colour,
        0,
    );
    b.push(
        Mesh::Cube,
        &rod(
            seat.add(face.scale(cm(40)))
                .add(up(cm(60)))
                .sub(right.scale(cm(90))),
            seat.add(face.scale(cm(40)))
                .add(up(cm(60)))
                .add(right.scale(cm(90))),
            cm(5),
        ),
        CHAIR_FRAME,
        0,
    );
}

// ---- what the vines hang from ----

fn hanger(s: &mut Sink, c: &Course, ski: &SkiLayout, h: &Hanger) {
    let anchor = c.vines.get(h.vine).map(|v| v.anchor).unwrap_or(h.to);
    match h.kind {
        HangerKind::Chair { lift } => {
            let colour = CHAIRS[lift % CHAIRS.len()];
            let face = ski.lifts.get(lift).map_or(v3(0, 0, ONE), |l| {
                let fwd = l.top.flat().sub(l.bottom.flat()).norm();
                // the up rope is on the right of the line
                let side = h.from.flat().sub(l.bottom.flat()).dot(right_of(fwd));
                if side >= 0 {
                    fwd
                } else {
                    fwd.neg()
                }
            });
            chair(s.at(h.from), h.from, h.to, face, colour);
            if anchor.sub(h.to).len() > cm(30) {
                s.at(h.to)
                    .push(Mesh::Cyl, &rod(h.to, anchor, cm(8)), CHAIR_FRAME, 0);
            }
        }
        HangerKind::Tree { tree } => {
            let t: Option<&Conifer> = ski.trees.get(tree);
            branch(s.at(h.to), h.from, h.to, t.map_or(0, |t| t.shade));
            if anchor.sub(h.to).len() > cm(30) {
                s.at(h.to)
                    .push(Mesh::Cyl, &rod(h.to, anchor, cm(6)), ROPE, 0);
            }
        }
        HangerKind::Helicopter => {}
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
                out.push(Mesh::Cyl, &rod(w[0], w[1], cm(7)), ROPE, 0);
            }
            out.push(
                Mesh::Bevel,
                &placed(anchor.add(up(cm(20))), 0, v3(cm(40), cm(40), cm(40))),
                STEEL_DARK,
                0,
            );
        }
    }
}

/// A long branch from the trunk at `from` out to its tip `to`: bark, its
/// needles hanging under it and snow lying along its top.
fn branch(b: &mut Batch, from: V3, to: V3, shade_of: u8) {
    let colour = NEEDLES[shade_of as usize % NEEDLES.len()];
    b.push(Mesh::Taper, &rod(from, to, cm(40)), BARK, 0);
    let d = to.sub(from);
    let len = d.len();
    if len < int(1) {
        return;
    }
    let along = d.norm();
    let side = right_of(along.flat().norm());
    // sprays of needles along it, snow on each
    let n = (len / int(2)).clamp(1, 6);
    for k in 0..n {
        let t = ONE * (2 * k + 1) / (2 * n);
        let at = from.lerp(to, t);
        let w = cm(150) - k * cm(15);
        b.push(
            Mesh::Blob,
            &panel(at.sub(up(cm(15))), side, V3::UP, along, w, cm(55), cm(170)),
            colour,
            0,
        );
        b.push(
            Mesh::Blob,
            &panel(
                at.add(up(cm(18))),
                side,
                V3::UP,
                along,
                w * 4 / 5,
                cm(26),
                cm(150),
            ),
            SNOW_ON,
            0,
        );
    }
}

// ---- the start and the finish ----

/// The start platform: a timber deck on steel legs, a snowy rail round it.
fn start_deck(s: &mut Sink, c: &Course) {
    for pl in &c.platforms {
        let b = s.at(pl.centre);
        let right = right_of(pl.fwd);
        let yaw = yaw_of(pl.fwd);
        let top = pl.top;
        b.push(
            Mesh::Cube,
            &placed(
                v3(pl.centre.x, top - cm(30), pl.centre.z),
                yaw,
                v3(pl.half_width * 2, cm(60), pl.half_len * 2),
            ),
            TIMBER,
            0,
        );
        b.push(
            Mesh::Cube,
            &placed(
                v3(pl.centre.x, top + cm(2), pl.centre.z),
                yaw,
                v3(pl.half_width * 2 - cm(40), cm(6), pl.half_len * 2 - cm(40)),
            ),
            SNOW,
            0,
        );
        for (x, z) in [(-1, -1), (1, -1), (-1, 1), (1, 1), (0, -1), (0, 1)] {
            let at = pl
                .centre
                .add(right.scale(x * (pl.half_width - cm(60))))
                .add(pl.fwd.scale(z * (pl.half_len - cm(60))));
            let g = ground_at(c, at);
            b.push(
                Mesh::Cube,
                &rod(
                    v3(at.x, g - cm(50), at.z),
                    v3(at.x, top - cm(50), at.z),
                    cm(50),
                ),
                STEEL,
                0,
            );
        }
        // rails on the sides and the back; the front is open
        for side in [-1, 1] {
            let a = pl
                .centre
                .add(right.scale(side * pl.half_width))
                .sub(pl.fwd.scale(pl.half_len));
            let z = pl
                .centre
                .add(right.scale(side * pl.half_width))
                .add(pl.fwd.scale(pl.half_len));
            b.push(
                Mesh::Cube,
                &rod(
                    v3(a.x, top + cm(100), a.z),
                    v3(z.x, top + cm(100), z.z),
                    cm(12),
                ),
                HOUSING_TRIM,
                0,
            );
        }
        let a = pl
            .centre
            .add(right.scale(-pl.half_width))
            .sub(pl.fwd.scale(pl.half_len));
        let z = pl
            .centre
            .add(right.scale(pl.half_width))
            .sub(pl.fwd.scale(pl.half_len));
        b.push(
            Mesh::Cube,
            &rod(
                v3(a.x, top + cm(100), a.z),
                v3(z.x, top + cm(100), z.z),
                cm(12),
            ),
            HOUSING_TRIM,
            0,
        );
        // the start gate's banner over the front edge
        let front = pl.centre.add(pl.fwd.scale(pl.half_len));
        for side in [-1, 1] {
            let foot = front.add(right.scale(side * pl.half_width));
            b.push(
                Mesh::Cyl,
                &rod(
                    v3(foot.x, top, foot.z),
                    v3(foot.x, top + int(13), foot.z),
                    cm(30),
                ),
                WHITE,
                0,
            );
        }
        b.push(
            Mesh::Cube,
            &placed(
                v3(front.x, top + int(13), front.z),
                yaw,
                v3(pl.half_width * 2, cm(90), cm(20)),
            ),
            RED,
            ONE / 6,
        );
    }
}

/// The finish: an inflatable red arch over the corridor with a white
/// banner, and a line across the snow.
fn finish_arch(b: &mut Batch, c: &Course) {
    let n = c.path.len() - 1;
    let at = c.path[n];
    let fwd = c.heading[n];
    let right = right_of(fwd);
    let yaw = yaw_of(fwd);
    let e = edges(c, c.length);
    let span = (e.r - e.l) / 2 + int(2);
    let mid = (e.r + e.l) / 2;
    let centre = at.add(right.scale(mid));
    let g = c.terrain(centre.x, centre.z);
    let rise = int(14);
    let pieces = 14;
    let point = |k: i32| {
        let a = fx::HALF * k / pieces;
        centre
            .add(right.scale(-mul(fx::cos(a), span)))
            .add(up(g + mul(sin(a), rise)))
    };
    for k in 0..pieces {
        let (p, q) = (point(k), point(k + 1));
        b.push(Mesh::Cyl, &rod(p, q, int(2)), RED, ONE / 8);
        b.push(
            Mesh::Sphere,
            &placed(q, 0, v3(int(2), int(2), int(2))),
            RED,
            ONE / 8,
        );
    }
    b.push(
        Mesh::Cube,
        &placed(
            centre.add(up(g + rise - int(3))),
            yaw,
            v3(span, int(2), cm(30)),
        ),
        WHITE,
        ONE / 4,
    );
    for k in 0..12 {
        let x = -span / 2 + span * k / 12 + span / 24;
        b.push(
            Mesh::Cube,
            &placed(
                centre
                    .add(right.scale(x))
                    .add(up(g + rise - int(3)))
                    .add(fwd.scale(cm(20))),
                yaw,
                v3(span / 24, cm(80), cm(6)),
            ),
            if k % 2 == 0 { RED } else { COAL },
            ONE / 6,
        );
    }
    b.push(
        Mesh::Cube,
        &placed(centre.add(up(g + cm(4))), yaw, v3(span * 2, cm(8), cm(80))),
        RED,
        ONE / 3,
    );
}

// ---- the landmarks ----

/// How far round a landmark the forest keeps clear.
fn landmark_clearance(k: LandmarkKind) -> i32 {
    match k {
        LandmarkKind::Summit => int(34),
        LandmarkKind::Snowman => int(14),
        LandmarkKind::Lodge => int(30),
        LandmarkKind::SkiJump => int(46),
        LandmarkKind::IceApe => int(18),
        LandmarkKind::FrozenFalls => int(30),
        LandmarkKind::Village => int(60),
    }
}

fn landmark(s: &mut Sink, c: &Course, m: &Landmark) {
    let foot = on_snow(c, m.at);
    let b = s.at(m.at);
    match m.kind {
        LandmarkKind::Summit => summit(b, foot, m.yaw),
        LandmarkKind::Snowman => snowman(b, foot, m.yaw),
        LandmarkKind::Lodge => lodge(b, foot, m.yaw),
        LandmarkKind::SkiJump => ski_jump(b, c, foot, m.yaw, m.s),
        LandmarkKind::IceApe => ice_ape(b, foot, m.yaw),
        LandmarkKind::FrozenFalls => frozen_falls(b, c, foot, m.yaw),
        LandmarkKind::Village => village(s, c, foot, m.yaw),
    }
}

/// A frame at `foot` turned to `yaw`: x across, y up, z the way it faces.
fn frame(foot: V3, yaw: i32) -> Affine {
    Affine::translate(foot).then(&Affine::rot_y(yaw))
}

/// A unit mesh placed in a landmark's frame: centred at `at`, `size`.
fn part(b: &mut Batch, f: &Affine, mesh: Mesh, at: V3, size: V3, colour: Rgb, glow: i32) {
    b.push(
        mesh,
        &f.then(&Affine::translate(at)).then(&Affine::scale(size)),
        colour,
        glow,
    );
}

/// The summit station at the top: a concrete and glass lift station on a
/// deck, and beside it the weather mast: a lattice tower with a great
/// white radar dome, an anemometer and a striped wind sock.
fn summit(b: &mut Batch, foot: V3, yaw: i32) {
    let f = frame(foot, yaw);
    // the station: a long concrete base, a glass band, a flat snowy roof
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, int(3), 0),
        v3(int(22), int(6), int(12)),
        CONCRETE,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, int(7), 0),
        v3(int(21), int(3), int(11)),
        GLASS,
        ONE / 5,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, cm(880), 0),
        v3(int(23), cm(70), int(13)),
        SNOW,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, cm(830), 0),
        v3(cm(2320), cm(40), cm(1320)),
        HOUSING_TRIM,
        0,
    );
    // the mast, beside the station
    let m = v3(-int(19), 0, -int(4));
    let h = int(34);
    for (x, z) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let base = m.add(v3(x * cm(220), 0, z * cm(220)));
        let top = m.add(v3(x * cm(70), h, z * cm(70)));
        b.push(
            Mesh::Cube,
            &rod(f.apply(base), f.apply(top), cm(26)),
            RED,
            0,
        );
    }
    // its cross braces, red and white in turn
    for k in 0..6 {
        let y = h * (k + 1) / 7;
        let w = cm(440) - (cm(300) * (k + 1)) / 7;
        part(
            b,
            &f,
            Mesh::Cube,
            m.add(v3(0, y, 0)),
            v3(w, cm(20), w),
            if k % 2 == 0 { WHITE } else { RED },
            0,
        );
    }
    // the radar dome, a platform under it
    part(
        b,
        &f,
        Mesh::Cyl,
        m.add(v3(0, h, 0)),
        v3(int(5), cm(40), int(5)),
        STEEL_DARK,
        0,
    );
    part(
        b,
        &f,
        Mesh::Geo,
        m.add(v3(0, h + int(3), 0)),
        v3(int(7), int(7), int(7)),
        WHITE,
        ONE / 10,
    );
    // the anemometer and the wind sock on a spar
    let spar = m.add(v3(0, h - int(4), 0));
    b.push(
        Mesh::Cube,
        &rod(f.apply(spar), f.apply(spar.add(v3(int(5), 0, 0))), cm(14)),
        STEEL_DARK,
        0,
    );
    for k in 0..4 {
        let at = spar.add(v3(cm(500) + cm(110) * k, -cm(30), 0));
        part(
            b,
            &f,
            Mesh::Cone,
            at,
            v3(cm(120) - k * cm(18), cm(110), cm(120) - k * cm(18)),
            if k % 2 == 0 { FENCE } else { WHITE },
            0,
        );
    }
    b.push(
        Mesh::Cube,
        &rod(f.apply(spar.add(v3(-int(4), 0, 0))), f.apply(spar), cm(14)),
        STEEL_DARK,
        0,
    );
    for k in 0..3 {
        let a = k * deg(120);
        let tip = spar.add(v3(-int(4), cm(60), 0)).add(v3(
            mul(fx::cos(a), cm(70)),
            0,
            mul(sin(a), cm(70)),
        ));
        part(
            b,
            &f,
            Mesh::Sphere,
            tip,
            v3(cm(30), cm(30), cm(30)),
            WHITE,
            0,
        );
    }
    // a red light on top of the dome
    part(
        b,
        &f,
        Mesh::Octa,
        m.add(v3(0, h + int(7), 0)),
        v3(cm(50), cm(50), cm(50)),
        RED,
        ONE,
    );
}

/// A giant snowman: three great balls of snow, coal eyes and buttons, a
/// carrot nose, a red scarf, a black top hat and stick arms.
fn snowman(b: &mut Batch, foot: V3, yaw: i32) {
    let f = frame(foot, yaw);
    let balls = [(int(9), int(4)), (int(7), int(11)), (cm(520), cm(1650))];
    for (k, (d, y)) in balls.into_iter().enumerate() {
        part(
            b,
            &f,
            Mesh::Geo,
            v3(0, y, 0),
            v3(d, d * 94 / 100, d),
            SNOW,
            0,
        );
        if k == 1 {
            for j in 0..3 {
                part(
                    b,
                    &f,
                    Mesh::Ico,
                    v3(
                        0,
                        y + int(2) - j * int(2),
                        cm(340) + if j == 1 { cm(25) } else { 0 },
                    ),
                    v3(cm(80), cm(80), cm(50)),
                    COAL,
                    0,
                );
            }
        }
    }
    let head = v3(0, cm(1650), 0);
    for x in [-cm(110), cm(110)] {
        part(
            b,
            &f,
            Mesh::Ico,
            head.add(v3(x, cm(80), cm(230))),
            v3(cm(70), cm(70), cm(40)),
            COAL,
            0,
        );
    }
    // the nose, pointing out
    let nose = f.apply(head.add(v3(0, cm(10), cm(250))));
    let tip = f.apply(head.add(v3(0, -cm(20), cm(560))));
    b.push(Mesh::Cone, &rod(nose, tip, cm(90)), CARROT, ONE / 10);
    // the smile
    for k in 0..5 {
        let a = deg(-40) + k * deg(20);
        part(
            b,
            &f,
            Mesh::Ico,
            head.add(v3(
                mul(sin(a), cm(130)),
                -cm(100) + mul(fx::cos(a), -cm(40)) + cm(40),
                cm(235),
            )),
            v3(cm(40), cm(40), cm(30)),
            COAL,
            0,
        );
    }
    // the scarf round the neck, its end hanging down
    part(
        b,
        &f,
        Mesh::Cyl,
        v3(0, cm(1420), 0),
        v3(cm(560), cm(110), cm(560)),
        SCARF,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(cm(170), cm(1240), cm(250)),
        v3(cm(110), cm(380), cm(30)),
        SCARF,
        0,
    );
    for k in 0..3 {
        part(
            b,
            &f,
            Mesh::Cube,
            v3(cm(170), cm(1300) - k * cm(120), cm(266)),
            v3(cm(112), cm(20), cm(6)),
            WHITE,
            0,
        );
    }
    // the top hat
    part(
        b,
        &f,
        Mesh::Cyl,
        v3(0, cm(1880), 0),
        v3(cm(560), cm(30), cm(560)),
        COAL,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cyl,
        v3(0, cm(2080), 0),
        v3(cm(380), cm(400), cm(380)),
        COAL,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cyl,
        v3(0, cm(1930), 0),
        v3(cm(390), cm(60), cm(390)),
        RED,
        0,
    );
    // stick arms with twig fingers
    for side in [-1, 1] {
        let sh = v3(side * cm(300), int(12), 0);
        let hand = v3(side * cm(820), cm(1550), cm(80));
        b.push(
            Mesh::Taper,
            &rod(f.apply(sh), f.apply(hand), cm(40)),
            TIMBER_DARK,
            0,
        );
        for k in [-1, 1] {
            let twig = hand.add(v3(side * cm(120), cm(80) + k * cm(60), k * cm(40)));
            b.push(
                Mesh::Taper,
                &rod(f.apply(hand), f.apply(twig), cm(20)),
                TIMBER_DARK,
                0,
            );
        }
    }
}

/// A timber lodge: log walls on a stone base, a steep snowy roof with
/// deep eaves, warm windows, a stone chimney (its smoke is drawn each
/// frame, `smoke`), a sun deck with red and yellow parasols, and skis
/// stood in the snow.
fn lodge(b: &mut Batch, foot: V3, yaw: i32) {
    let f = frame(foot, yaw).then(&Affine::scale(v3(LODGE_SCALE, LODGE_SCALE, LODGE_SCALE)));
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, cm(80), 0),
        v3(int(20), cm(200), int(13)),
        STONE,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, cm(480), 0),
        v3(int(19), int(6), int(12)),
        TIMBER,
        0,
    );
    // the logs' lines
    for k in 0..6 {
        part(
            b,
            &f,
            Mesh::Cube,
            v3(0, cm(220) + k * int(1), 0),
            v3(cm(1920), cm(14), cm(1220)),
            TIMBER_DARK,
            0,
        );
    }
    // the roof: two steep snowy slopes with deep eaves, the gable facing
    for side in [-1, 1] {
        let slope = f
            .then(&Affine::translate(v3(side * cm(560), cm(1060), 0)))
            .then(&Affine::rot_z(-side * deg(42)))
            .then(&Affine::scale(v3(int(15), cm(80), int(16))));
        b.push(Mesh::Cube, &slope, SNOW, 0);
        let under = f
            .then(&Affine::translate(v3(side * cm(560), cm(1000), 0)))
            .then(&Affine::rot_z(-side * deg(42)))
            .then(&Affine::scale(v3(int(15), cm(40), cm(1580))));
        b.push(Mesh::Cube, &under, TIMBER_DARK, 0);
    }
    // the gable ends
    for z in [-cm(590), cm(590)] {
        b.push(
            Mesh::Pyramid,
            &f.then(&Affine::translate(v3(0, cm(1060), z)))
                .then(&Affine::scale(v3(int(19), int(9), cm(20)))),
            TIMBER,
            0,
        );
    }
    // windows, warm inside, on both long sides and the gable
    for side in [-1, 1] {
        for k in 0..4 {
            part(
                b,
                &f,
                Mesh::Cube,
                v3(-int(6) + k * int(4), cm(480), side * cm(605)),
                v3(cm(200), cm(220), cm(20)),
                WARM_WINDOW,
                ONE / 2,
            );
        }
    }
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, cm(1100), cm(605)),
        v3(cm(300), cm(250), cm(20)),
        WARM_WINDOW,
        ONE / 2,
    );
    // the chimney
    part(
        b,
        &f,
        Mesh::Cube,
        v3(int(6), int(13), -int(2)),
        v3(int(2), int(7), int(2)),
        STONE,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(int(6), cm(1660), -int(2)),
        v3(cm(240), cm(30), cm(240)),
        SNOW,
        0,
    );
    // the sun deck along the front, its parasols
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, cm(170), int(11)),
        v3(int(20), cm(30), int(9)),
        TIMBER,
        0,
    );
    for k in 0..4 {
        let at = v3(-int(7) + k * cm(470), cm(170), int(11));
        part(
            b,
            &f,
            Mesh::Cyl,
            at.add(v3(0, cm(150), 0)),
            v3(cm(12), int(3), cm(12)),
            WHITE,
            0,
        );
        part(
            b,
            &f,
            Mesh::Cone,
            at.add(v3(0, cm(330), 0)),
            v3(int(4), cm(110), int(4)),
            CHAIRS[(k % 2) as usize],
            0,
        );
    }
    // skis stood in the snow by the door
    for k in 0..6 {
        let at = v3(-int(12) + k * cm(45), cm(90), int(4));
        part(
            b,
            &f,
            Mesh::Cube,
            at,
            v3(cm(12), cm(180), cm(4)),
            FLAGS[(k % 4) as usize],
            0,
        );
    }
}

/// Where the lodge's chimney smokes from (world).
fn chimney_top(foot: V3, yaw: i32) -> V3 {
    frame(foot, yaw)
        .then(&Affine::scale(v3(LODGE_SCALE, LODGE_SCALE, LODGE_SCALE)))
        .apply(v3(int(6), cm(1700), -int(2)))
}

/// The lodge and the frozen falls are drawn this much over their models'
/// sizes, to read from the run.
const LODGE_SCALE: i32 = ONE * 8 / 5;
const FALLS_SCALE: i32 = ONE * 8 / 5;

/// A ski jump beside the course, seen side on from the run: a tall
/// concrete tower with a red cabin, the in-run slanting down from it on
/// legs to the take-off, the landing hill below running down toward the
/// course, a judges' tower, and flags along the in-run.
fn ski_jump(b: &mut Batch, c: &Course, foot: V3, yaw: i32, s: i32) {
    // the jump runs down toward the course
    let toward = c.point(s).flat().sub(foot.flat());
    let yaw = if toward.len() > int(1) {
        yaw_of(toward)
    } else {
        yaw
    };
    let f = frame(foot, yaw);
    let top = v3(0, int(44), -int(46));
    let lip = v3(0, int(14), -int(6));
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, int(22), -int(48)),
        v3(int(7), int(44), int(7)),
        CONCRETE,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, int(46), -int(48)),
        v3(int(9), int(5), int(9)),
        HOUSING_TRIM,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, int(46), -int(48) + cm(455)),
        v3(int(7), int(2), cm(20)),
        GLASS,
        ONE / 5,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, cm(4880), -int(48)),
        v3(cm(1000), cm(70), cm(1000)),
        SNOW,
        0,
    );
    // the in-run: a long sloping slab, white with blue edges
    let (a, z) = (f.apply(top), f.apply(lip));
    let d = z.sub(a);
    let across = f.apply_dir(v3(ONE, 0, 0));
    let normal = d.cross(across).norm();
    let normal = if normal.y < 0 { normal.neg() } else { normal };
    let slab = |w: i32, h: i32, lift: i32, off: i32| Affine {
        c0: across.scale(w),
        c1: normal.scale(h),
        c2: d,
        t: a.lerp(z, ONE / 2)
            .add(normal.scale(lift))
            .add(across.scale(off)),
    };
    b.push(Mesh::Cube, &slab(int(6), cm(90), 0, 0), STEEL, 0);
    b.push(Mesh::Cube, &slab(int(5), cm(30), cm(55), 0), SNOW, 0);
    for side in [-1, 1] {
        b.push(
            Mesh::Cube,
            &slab(cm(30), cm(110), cm(60), side * cm(270)),
            rgb(30, 80, 190),
            0,
        );
    }
    // its legs, down to the ground
    for k in 1..5 {
        let p = top.lerp(lip, ONE * k / 5);
        let g = v3(p.x, -int(1), p.z);
        b.push(
            Mesh::Cube,
            &rod(f.apply(g), f.apply(p.sub(up(cm(60)))), cm(90)),
            CONCRETE,
            0,
        );
    }
    // the take-off table, and the landing hill under it running down
    part(
        b,
        &f,
        Mesh::Cube,
        lip.add(v3(0, -cm(60), int(1))),
        v3(int(6), cm(120), int(3)),
        CONCRETE,
        0,
    );
    b.push(
        Mesh::Wedge,
        &f.then(&Affine::translate(v3(0, int(5) - int(1), int(12))))
            .then(&Affine::scale(v3(int(20), int(12), int(36)))),
        SNOW,
        0,
    );
    // the judges' tower beside the landing
    part(
        b,
        &f,
        Mesh::Cube,
        v3(int(15), int(7), int(4)),
        v3(int(4), int(16), int(4)),
        CONCRETE,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(int(15), int(16), int(4)),
        v3(int(6), int(3), int(5)),
        GLASS,
        ONE / 5,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(int(15), cm(1780), int(4)),
        v3(int(7), cm(40), int(6)),
        SNOW,
        0,
    );
    // flags along the in-run
    for k in 0..6 {
        let p = top.lerp(lip, ONE * k / 6).add(v3(-int(4), 0, 0));
        part(
            b,
            &f,
            Mesh::Cyl,
            p.add(v3(0, int(3), 0)),
            v3(cm(14), int(6), cm(14)),
            WHITE,
            0,
        );
        part(
            b,
            &f,
            Mesh::Cube,
            p.add(v3(0, cm(520), cm(90))),
            v3(cm(6), cm(110), cm(180)),
            FLAGS[k as usize % FLAGS.len()],
            ONE / 8,
        );
    }
}

/// A giant ice sculpture of an ape on an ice plinth: an ape's model,
/// twenty times life size, recoloured in pale translucent ice.
fn ice_ape(b: &mut Batch, foot: V3, yaw: i32) {
    part(
        b,
        &frame(foot, yaw),
        Mesh::Hex,
        v3(0, cm(90), 0),
        v3(int(9), cm(180), int(9)),
        ICE_DEEP,
        ONE / 4,
    );
    let marks = b.marks();
    let mut pose = crate::models::Pose::standing(foot.add(up(cm(180))), yaw, 0);
    // (a standing ape is scaled about its feet)
    pose.scale = 2000;
    crate::models::draw(b, crate::rules::Kind::Gorilla, &pose);
    b.recolour_since(&marks, |c, _| {
        let luma = (c[0] * 30 + c[1] * 59 + c[2] * 11) / 100;
        mix(ICE_DEEP, ICE, fx::div(luma, ONE * 3 / 5).min(ONE))
    });
    b.reglow_since(&marks, ONE * 3 / 10);
}

/// A frozen waterfall down a rock face beside the course: a cliff of
/// dark rock, and the falls frozen in pale blue columns and icicles down
/// its middle into a heap of ice.
fn frozen_falls(b: &mut Batch, c: &Course, foot: V3, yaw: i32) {
    let _ = c;
    let f = frame(foot, yaw).then(&Affine::scale(v3(FALLS_SCALE, FALLS_SCALE, FALLS_SCALE)));
    // the cliff: great blocks of dark rock side by side, leaning a little,
    // snow lying on each one's top; lower in the middle, where the falls
    // come over the lip
    for k in -4..=4 {
        let x = k * int(7);
        let h = if k.abs() <= 1 {
            int(22)
        } else {
            int(24) + (k * 7).rem_euclid(5) * int(2) - k.abs() * int(1)
        };
        let back = -int(5) - (k.rem_euclid(3)) * int(1);
        let block = f
            .then(&Affine::translate(v3(x, h / 2 - int(2), back)))
            .then(&Affine::rot_y((k * 5).rem_euclid(11) * deg(2) - deg(10)))
            .then(&Affine::rot_z((k * 3).rem_euclid(7) * deg(1) - deg(3)));
        b.push(
            Mesh::Cube,
            &block.then(&Affine::scale(v3(int(8), h, int(9)))),
            shade(ROCK, 84 + (k * 7).rem_euclid(20)),
            0,
        );
        b.push(
            Mesh::Cube,
            &block
                .then(&Affine::translate(v3(0, h / 2 + cm(30), 0)))
                .then(&Affine::scale(v3(int(8) + cm(40), cm(60), int(9) + cm(40)))),
            SNOW,
            0,
        );
        // a ledge of snow part way up
        b.push(
            Mesh::Cube,
            &block
                .then(&Affine::translate(v3(0, h / 6, int(4) + cm(60))))
                .then(&Affine::scale(v3(int(7), cm(50), cm(140)))),
            SNOW,
            0,
        );
    }
    // the falls: columns of ice down the middle, longest in the middle
    for k in -5..=5 {
        let x = k * cm(130);
        let len = int(20) - (k * k) * cm(40);
        let at = v3(x, int(20) - len / 2, -cm(20) + (k % 2).abs() * cm(50));
        part(
            b,
            &f,
            Mesh::Hex,
            at,
            v3(cm(170), len, cm(140)),
            if k % 2 == 0 { ICE } else { ICE_DEEP },
            ONE / 4,
        );
        // icicles hanging off the lip
        b.push(
            Mesh::Spike,
            &f.then(&Affine::translate(v3(x + cm(60), int(19), cm(100))))
                .then(&Affine::rot_x(deg(180)))
                .then(&Affine::scale(v3(cm(70), int(4), cm(70)))),
            ICE,
            ONE / 4,
        );
    }
    // the heap of ice at the foot, and boulders either side
    part(
        b,
        &f,
        Mesh::Ico,
        v3(0, cm(100), int(3)),
        v3(int(12), int(4), int(7)),
        ICE,
        ONE / 5,
    );
    for x in [-int(14), int(13), -int(22), int(21)] {
        b.push(
            Mesh::Rock,
            &f.then(&Affine::translate(v3(x, int(1), int(4))))
                .then(&Affine::scale(v3(int(5), int(4), int(5)))),
            shade(ROCK, 96),
            0,
        );
    }
}

/// The base village at the finish: chalets with snowy roofs and warm
/// windows round the run-out, a row of flags, and the podium.
fn village(s: &mut Sink, c: &Course, foot: V3, yaw: i32) {
    let _ = (foot, yaw);
    let last = c.path.len() - 1;
    let fwd = c.heading[last];
    let right = right_of(fwd);
    let at = |along: i32, a: i32| on_snow(c, c.point(c.length + along).add(right.scale(a)));
    // the chalets, both sides of the run-out and across its end
    let mut dice = Dice(0x5C1_C4A1);
    let spots: [(i32, i32); 10] = [
        (int(20), -int(40)),
        (int(55), -int(44)),
        (int(90), -int(40)),
        (int(125), -int(46)),
        (int(20), int(42)),
        (int(55), int(46)),
        (int(90), int(40)),
        (int(125), int(44)),
        (int(170), -int(18)),
        (int(170), int(20)),
    ];
    for (k, (along, a)) in spots.into_iter().enumerate() {
        let p = at(along, a);
        let toward = if k >= 8 {
            fwd.neg()
        } else if a < 0 {
            right
        } else {
            right.neg()
        };
        chalet(s.at(p), p, yaw_of(toward), &mut dice);
    }
    // flags along both sides of the run-out
    for side in [-1, 1] {
        for k in 0..6 {
            let p = at(int(10) + k * int(18), side * int(24));
            let b = s.at(p);
            b.push(Mesh::Cyl, &rod(p, p.add(up(int(9))), cm(18)), WHITE, 0);
            b.push(
                Mesh::Cube,
                &placed(
                    p.add(up(cm(800))).add(fwd.scale(cm(110))),
                    yaw_of(right),
                    v3(cm(8), cm(140), cm(220)),
                ),
                FLAGS[(k as usize + if side > 0 { 2 } else { 0 }) % FLAGS.len()],
                ONE / 8,
            );
        }
    }
    // the podium in the middle of the run-out
    let p = at(int(60), 0);
    let b = s.at(p);
    let yaw_f = yaw_of(fwd);
    for (x, h, colour) in [
        (0, int(3), GOLD),
        (-cm(420), int(2), SILVER),
        (cm(420), cm(130), BRONZE),
    ] {
        let q = p.add(right.scale(x));
        b.push(
            Mesh::Cube,
            &placed(q.add(up(h / 2)), yaw_f, v3(int(4), h, int(4))),
            colour,
            ONE / 6,
        );
        b.push(
            Mesh::Cube,
            &placed(q.add(up(h + cm(5))), yaw_f, v3(cm(420), cm(10), cm(420))),
            SNOW,
            ONE / 6,
        );
    }
    // a pile of bananas beside it
    let q = at(int(58), int(12));
    for k in 0..9 {
        let a = k * fx::TURN / 9;
        let mid = q.add(v3(
            mul(sin(a), cm(160)),
            cm(30) + (k % 3) * cm(25),
            mul(fx::cos(a), cm(160)),
        ));
        let along = facing(a + deg(90), 0).scale(cm(60));
        b.push(
            Mesh::Cube,
            &rod(mid.sub(along).add(up(cm(18))), mid, cm(24)),
            BANANA,
            ONE / 6,
        );
        b.push(
            Mesh::Cube,
            &rod(mid, mid.add(along).add(up(cm(18))), cm(24)),
            BANANA,
            ONE / 6,
        );
    }
}

/// A chalet at `foot` facing `yaw`: timber walls on a stone base, a
/// steep snowy roof, a balcony, warm windows.
fn chalet(b: &mut Batch, foot: V3, yaw: i32, dice: &mut Dice) {
    let f = frame(foot, yaw);
    let w = int(dice.range(9, 13));
    let d = int(dice.range(8, 11));
    let h = int(dice.range(5, 8));
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, cm(60), 0),
        v3(w, cm(160), d),
        STONE,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, h / 2 + cm(80), 0),
        v3(w - cm(40), h, d - cm(40)),
        TIMBER,
        0,
    );
    for side in [-1, 1] {
        let slope = Affine::translate(foot)
            .then(&Affine::rot_y(yaw))
            .then(&Affine::translate(v3(side * w / 4, h + cm(80) + w / 5, 0)))
            .then(&Affine::rot_z(-side * deg(38)))
            .then(&Affine::scale(v3(w * 3 / 5 + cm(80), cm(60), d + int(2))));
        b.push(Mesh::Cube, &slope, SNOW, 0);
    }
    b.push(
        Mesh::Pyramid,
        &f.then(&Affine::translate(v3(0, h + cm(80) + w / 5 - cm(20), 0)))
            .then(&Affine::scale(v3(w - cm(60), w * 2 / 5, d - cm(40)))),
        TIMBER_DARK,
        0,
    );
    for x in [-w / 4, w / 4] {
        part(
            b,
            &f,
            Mesh::Cube,
            v3(x, h / 2 + cm(80), d / 2 - cm(15)),
            v3(cm(160), cm(180), cm(10)),
            WARM_WINDOW,
            ONE / 2,
        );
    }
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, h + cm(20), d / 2 + cm(60)),
        v3(w - int(1), cm(20), cm(140)),
        TIMBER_DARK,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, h + cm(70), d / 2 + cm(120)),
        v3(w - int(1), cm(90), cm(10)),
        TIMBER_DARK,
        0,
    );
}

// ---- what moves, and the sky round the eye ----

/// Drawn each frame: the sky's dome, the far mountains and the peak (all
/// round the eye), the helicopters, and the lodge's chimney smoke.
pub fn dynamic(b: &mut Batch, c: &Course, ski: &SkiLayout, eye: V3, tick: u32, race_tick: u32) {
    backdrop(b, c, eye);
    for (k, h) in ski.hangers.iter().enumerate() {
        if h.kind != HangerKind::Helicopter {
            continue;
        }
        let Some(v) = c.vines.get(h.vine) else {
            continue;
        };
        let now = c.anchor(h.vine, race_tick);
        if now.flat().sub(eye.flat()).len() > int(280) {
            continue;
        }
        let hub = h.from.add(now.sub(v.anchor));
        let hub =
            v3(hub.x, hub.y.max(now.y + cm(560)), hub.z).add(up(crate::vehicles::hover(tick, k)));
        let yaw = (k as i32) * deg(97);
        crate::vehicles::helicopter(b, hub, yaw, tick, 1, ONE * 3 / 2);
        let belly = hub.sub(up(cm(420)));
        if belly.y > now.y + cm(30) {
            b.push(Mesh::Cyl, &rod(belly, now, cm(10)), ROPE, 0);
        }
    }
    for m in &ski.landmarks {
        if m.kind == LandmarkKind::Lodge && m.at.flat().sub(eye.flat()).len() < int(300) {
            smoke(b, chimney_top(on_snow(c, m.at), m.yaw), tick);
        }
    }
}

/// Puffs of wood smoke rising from `top`, growing and drifting as they go.
fn smoke(b: &mut Batch, top: V3, tick: u32) {
    let period = 480;
    for k in 0..7u32 {
        let age = ((tick + k * period / 7) % period) as i32;
        let t = fx::div(age, period as i32);
        let r = cm(120) + mul(t, int(4));
        let at = top.add(v3(mul(t, int(7)), mul(t, int(16)), mul(t, int(3))));
        let grey = mix(rgb(200, 204, 212), FOG, t);
        b.push(
            Mesh::Blob,
            &placed(at, k as i32 * 9000 + age * 20, v3(r, r * 4 / 5, r)),
            grey,
            ONE / 4,
        );
    }
}

/// The dome's radius: inside the far plane.
const DOME: i32 = int(560);
/// The far mountains stand this far from the eye, their feet this far
/// below it.
const RANGE_AT: i32 = int(470);
const RANGE_FOOT: i32 = int(170);
/// How far each is hazed toward the fog: the range and the peak.
const RANGE_HAZE: i32 = ONE / 2;
const PEAK_HAZE: i32 = ONE / 6;
/// The glow that marks a sky dome shape, and a far backdrop's (plus its
/// haze), for `scene.frag`.
const DOME_GLOW: i32 = ONE * 4;
const FAR_GLOW: i32 = ONE * 5 / 2;

/// The iconic peak's bearing from the course's downhill heading, and its
/// height over the horizon.
const PEAK_BEARING: i32 = deg(-20);
/// The peak's foot below the eye (higher than the range's, so more of it
/// stands over the trees).
const PEAK_FOOT: i32 = int(120);

/// The sky round the eye: a dome deepening from the haze at the horizon to
/// a saturated blue overhead (`scene.frag` paints it), a ring of snowy
/// mountains, and the one great peak ahead down the valley. All are drawn
/// at a fixed distance from the eye, as if infinitely far.
pub fn backdrop(b: &mut Batch, c: &Course, eye: V3) {
    // the dome's colour is a multiple of the sky's (so the colourblind mode
    // and JOJO MODE, which change the sky, change it too)
    let deep = [0, 1, 2].map(|k| fx::div(ZENITH[k], SKY[k].max(1)).min(ONE));
    b.push(
        Mesh::Sphere,
        &placed(eye, 0, v3(DOME * 2, DOME * 2, DOME * 2)),
        deep,
        DOME_GLOW,
    );
    let base = downhill_yaw(c);
    let foot = eye.y - RANGE_FOOT;
    let mut dice = Dice(0x5C1_3A06E);
    let count = 26;
    for k in 0..count {
        let yaw = base + k * fx::TURN / count + dice.range(-deg(4), deg(4));
        // leave the peak's own place clear
        let off = fx::wrap(yaw - base - PEAK_BEARING);
        let tall = int(dice.range(12, 46)) + RANGE_FOOT;
        let wide = int(dice.range(200, 320));
        if off.abs() < deg(9) {
            continue;
        }
        let at = eye
            .flat()
            .add(facing(yaw, 0).scale(RANGE_AT + int(dice.range(0, 40))));
        let lean = facing(yaw + deg(90), 0).scale(int(dice.range(-30, 30)));
        mountain(
            b,
            v3(at.x, foot, at.z),
            dice.range(0, fx::TURN),
            (wide, tall),
            lean,
            MOUNTAIN_ROCK,
            ONE * 11 / 20,
            RANGE_HAZE,
        );
    }
    peak(b, eye, base + PEAK_BEARING);
    // the valley haze: a ring of the fog's own colour just inside the
    // range, from a little under the horizon down, so the mountains rise
    // out of the haze far off instead of standing at the forest's edge
    let ring = 32;
    for k in 0..ring {
        let a = k * fx::TURN / ring;
        let dir = facing(a, 0);
        let at = eye.flat().add(dir.scale(HAZE_AT));
        let w = mul(HAZE_AT, ONE * 21 / 100);
        b.push(
            Mesh::Cube,
            &panel(
                v3(at.x, eye.y - HAZE_BELOW - int(150), at.z),
                right_of(dir),
                V3::UP,
                dir,
                w,
                int(300),
                int(2),
            ),
            FOG,
            FAR_GLOW + ONE * 99 / 100,
        );
    }
}

/// The valley haze's ring: its distance from the eye, and how far under
/// the eye its top is.
const HAZE_AT: i32 = int(445);
const HAZE_BELOW: i32 = int(14);

/// One mountain: a pyramid with its foot's middle at `foot`, turned `yaw`,
/// `wide` across and `tall`, its top pushed over by `lean`, in `rock`,
/// white from `snow` of the way down from the top.
fn mountain(
    b: &mut Batch,
    foot: V3,
    yaw: i32,
    (wide, tall): (i32, i32),
    lean: V3,
    rock: Rgb,
    snow: i32,
    haze: i32,
) {
    let r = Affine::rot_y(yaw);
    let c0 = r.apply_dir(v3(wide, 0, 0));
    let c2 = r.apply_dir(v3(0, 0, wide));
    let c1 = up(tall).add(lean);
    let body = Affine {
        c0,
        c1,
        c2,
        t: foot.add(c1.scale(ONE / 2)),
    };
    let glow = FAR_GLOW + haze;
    b.push(Mesh::Pyramid, &body, rock, glow);
    if snow <= 0 {
        return;
    }
    let top = foot.add(c1);
    let k = snow;
    let fat = mul(k, ONE * 106 / 100);
    let cap = Affine {
        c0: c0.scale(fat),
        c1: c1.scale(k),
        c2: c2.scale(fat),
        t: top.sub(c1.scale(k / 2)),
    };
    b.push(Mesh::Pyramid, &cap, SNOW, glow);
}

/// The one great peak: a tall, sharp snow horn standing over the valley
/// the course runs down, taller than anything in the range, its summit
/// hooked over and its ridges and faces streaked with dark rock, on two
/// broad shoulders.
fn peak(b: &mut Batch, eye: V3, yaw: i32) {
    let toward = facing(yaw, 0);
    let side = right_of(toward);
    let centre = eye.flat().add(toward.scale(RANGE_AT + int(30)));
    let foot = eye.y - PEAK_FOOT;
    let at = |x: i32, d: i32, h: i32| {
        let p = centre.add(side.scale(x)).add(toward.scale(d));
        v3(p.x, foot + h, p.z)
    };
    let face = yaw_of(toward) + deg(45);
    let glow = FAR_GLOW + PEAK_HAZE;
    // the shoulders, broad, white over blue-grey rock
    mountain(
        b,
        at(-int(160), int(40), 0),
        face + deg(12),
        (int(300), int(150)),
        side.scale(-int(50)),
        MOUNTAIN_ROCK,
        ONE * 3 / 5,
        PEAK_HAZE,
    );
    mountain(
        b,
        at(int(170), int(50), 0),
        face - deg(17),
        (int(300), int(140)),
        side.scale(int(60)),
        MOUNTAIN_ROCK,
        ONE * 3 / 5,
        PEAK_HAZE,
    );
    // the horn: snow, leaning a little
    let (wide, tall) = (int(190), int(250));
    let lean = side.scale(int(20));
    let r = Affine::rot_y(face);
    let horn = Affine {
        c0: r.apply_dir(v3(wide, 0, 0)),
        c1: up(tall).add(lean),
        c2: r.apply_dir(v3(0, 0, wide)),
        t: at(0, 0, 0).add(up(tall).add(lean).scale(ONE / 2)),
    };
    b.push(Mesh::Pyramid, &horn, SNOW, glow);
    let summit = horn.apply(v3(0, ONE / 2, 0));
    // dark rock along its four ridges: two thin fins through the summit,
    // each down a pair of opposite corners, a hair proud of the snow
    for k in 0..2 {
        let corner = horn.apply(v3(
            if k % 2 == 0 { ONE / 2 } else { -ONE / 2 },
            -ONE / 2,
            if k < 2 { ONE / 2 } else { -ONE / 2 },
        ));
        let foot_mid = at(0, 0, 0);
        let out = corner.sub(foot_mid).scale(ONE * 104 / 100);
        let across = right_of(out.flat().norm()).scale(int(22));
        let fin = Affine {
            c0: across,
            c1: summit.sub(foot_mid),
            c2: out.scale(2),
            t: foot_mid.add(summit.sub(foot_mid).scale(ONE / 2)),
        };
        b.push(Mesh::Pyramid, &fin, PEAK_ROCK, glow);
    }
    // rock bands across its upper faces
    for (h, w) in [
        (ONE * 62 / 100, ONE * 40 / 100),
        (ONE * 80 / 100, ONE * 22 / 100),
    ] {
        let band = Affine {
            c0: horn.c0.scale(w * 118 / 100),
            c1: horn.c1.scale(ONE / 12),
            c2: horn.c2.scale(w * 118 / 100),
            t: horn.apply(v3(0, h - ONE / 2, 0)),
        };
        b.push(Mesh::Pyramid, &band, PEAK_ROCK, glow);
    }
}

// ---- the preview spot and the flyover ----

/// A place on the Ski Slope to show an ape on the selection screen: on
/// the piste just below the start, the course and the peak beyond.
pub fn preview(c: &Course) -> (V3, i32, Camera) {
    let s = int(40).min(c.length);
    let fwd = c.heading_at(s);
    let right = right_of(fwd);
    let p = c.point(s).add(right.scale(c.half_width / 3));
    let feet = v3(p.x, c.terrain(p.x, p.z), p.z);
    // the ape faces up the hill toward the camera, which looks down it
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
    use crate::map::Map;

    #[test]
    fn the_snow_is_the_floor_inside_the_fences_and_rises_beyond() {
        let c = Map::Ski.course();
        for m in [50, 400, 900] {
            let s = int(m);
            let e = edges(&c, s);
            // in every lane (on a fork's island the snow is banked up)
            for (mid, _) in c.lanes(s) {
                let p = spot(&c, s, mid);
                assert_eq!(snow_height(&c, s, mid), c.terrain(p.x, p.z), "{m} m");
            }
            let q = spot(&c, s, e.r + int(60));
            assert!(snow_height(&c, s, e.r + int(60)) > c.terrain(q.x, q.z) + int(5));
        }
    }

    /// Every instance in a batch's values: (centre, colour, glow).
    fn shapes(values: &[i32]) -> Vec<(V3, Rgb, i32)> {
        values
            .chunks(crate::render::INSTANCE_VALUES)
            .map(|i| (v3(i[3], i[7], i[11]), [i[12], i[13], i[14]], i[15]))
            .collect()
    }

    #[test]
    fn noon_is_a_high_white_sun_under_a_deep_blue_sky() {
        // the person (2026-10-13): noon, clear; a look no other map has
        let l = scene::look(Map::Ski);
        assert!(l.light_dir.y > fx::sin(deg(55)), "the sun is high");
        let [r, g, b] = l.light;
        assert!(
            r > ONE * 9 / 10 && g > ONE * 9 / 10 && b > ONE * 9 / 10,
            "white light"
        );
        assert!(
            ZENITH[2] > ZENITH[0] * 4 && ZENITH[2] > ZENITH[1] * 2,
            "a deep blue overhead"
        );
        assert!(
            FOG[2] > FOG[0] && FOG.iter().all(|c| *c > ONE * 3 / 4),
            "a pale blue haze"
        );
        for other in [Map::Jungle, Map::City] {
            let o = scene::look(other);
            assert_ne!(o.fog, l.fog);
            assert!(o.light_dir.y < l.light_dir.y, "the highest sun of the maps");
        }
    }

    #[test]
    fn the_ski_slopes_vines_are_pink_ropes_apart_from_the_reach_and_catch_up_looks() {
        let rope = scene::vine_look_on(Map::Ski, false, false, 0);
        assert_eq!(rope.colour, ROPE_PINK);
        assert!(rope.halo > 0, "a soft glow round each rope");
        let reach = scene::vine_look_on(Map::Ski, true, false, 0);
        assert_ne!(reach.colour, rope.colour);
        assert_eq!(
            reach.colour,
            scene::vine_look(true, false, 0).colour,
            "the shared reach yellow"
        );
        assert_eq!(
            scene::vine_look_on(Map::Ski, false, true, 0).colour,
            ROPE_VIVID
        );
        // and apart from the fences' orange, the item flowers' violet and
        // the lift chairs
        for other in [FENCE, scene::PETAL, CHAIRS[0], CHAIRS[1]] {
            let d: i32 = (0..3).map(|k| (other[k] - ROPE_PINK[k]).abs()).sum();
            assert!(d > ONE / 2, "{other:?} too near the ropes");
        }
    }

    #[test]
    fn the_scenery_draws_fences_lifts_and_every_landmark() {
        let c = Map::Ski.course();
        let ski = c.ski.as_ref().unwrap();
        let s = scenery(&c, ski);
        let all = shapes(&s.values);
        let fence = all.iter().filter(|(_, col, _)| *col == FENCE).count();
        assert!(
            fence as i32 > 2 * c.length / POST_EVERY,
            "a fence along both edges: {fence}"
        );
        // a chair on every chair of every lift (those carrying vines too)
        for (k, l) in ski.lifts.iter().enumerate() {
            let chairs = all
                .iter()
                .filter(|(_, col, _)| *col == CHAIRS[k % 2])
                .count() as i32;
            assert!(
                chairs >= 2 * 2 * (l.chairs() - 1),
                "lift {k}: {chairs} chair shapes"
            );
            for t in &l.towers {
                assert!(
                    all.iter()
                        .any(|(p, col, _)| *col == STEEL && p.flat().sub(t.flat()).len() < int(1)),
                    "a tower at each foot"
                );
            }
        }
        for m in &ski.landmarks {
            let near = all
                .iter()
                .filter(|(p, _, _)| p.flat().sub(m.at.flat()).len() < int(60))
                .count();
            assert!(near > 20, "{:?} drawn: {near} shapes", m.kind);
        }
        // the budget (MAPS: about 20k, 6-7k at once)
        assert!(all.len() < 22_000, "{} shapes", all.len());
        // nothing bounces on this map, so no mushrooms are drawn
        assert!(c.mushrooms.is_empty());
    }

    #[test]
    fn the_sky_dome_and_the_range_are_drawn_round_the_eye() {
        let c = Map::Ski.course();
        for eye in [
            c.point(int(100)).add(up(int(10))),
            c.point(int(1500)).add(up(int(40))),
        ] {
            let mut b = Batch::new();
            backdrop(&mut b, &c, eye);
            let (values, _) = b.finish(crate::render::DYNAMIC_BUFFER);
            let all = shapes(&values);
            let dome: Vec<_> = all.iter().filter(|(_, _, g)| *g == DOME_GLOW).collect();
            assert_eq!(dome.len(), 1);
            assert_eq!(dome[0].0, eye, "the dome is centred on the eye");
            let far = all
                .iter()
                .filter(|(_, _, g)| *g > FAR_GLOW && *g < DOME_GLOW)
                .count();
            assert!(far > 40, "the range, the peak and the haze: {far}");
            // all of them inside the far plane
            for (p, _, _) in &all {
                assert!(p.sub(eye).len() < crate::render::FAR);
            }
        }
    }
}
