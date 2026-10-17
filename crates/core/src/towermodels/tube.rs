//! The second design's portal world (`notes/tower2-plan.md` steps 7–10):
//! the vertical tube the apes drop down (about 300 m long and 35 m
//! across, two thirds of a v1 spiral's 55 m width), in the zipper space's
//! colours, with portholes the creatures look in through; the bounce
//! pads across it, drawn as the void's own bouncers (glowing sea-green
//! films with a bright rim, readable from above at speed); and the
//! horizontal tunnel at its foot with the mouth's anchor bar the sling
//! vines hang from. The zip (`zip`), the creatures and the banana sea
//! are reused as they are.

use super::{bar, halo_ball, halo_bar, pick, pulse, put, put_turned, round, wave, DEEP_SKY};
use crate::fx::{cm, cos, int, mul, sin, sqrt, HALF, ONE, QUARTER, TURN};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, rgb, Batch, Rgb};

/// The tube's length, radius and the tunnel's length (a bridge's).
pub const TUBE_LENGTH: i32 = int(300);
pub const TUBE_RADIUS: i32 = int(17) + cm(50);
pub const TUNNEL_LENGTH: i32 = int(250);

/// Panels round the tube's wall.
const PANELS: i32 = 12;
/// A band of panels' height.
const BAND: i32 = int(12);
/// A porthole's radius, and the hole it cuts.
pub const PORTHOLE_R: i32 = int(4);

const WALL: Rgb = rgb(40, 30, 72);
const WALL_PALE: Rgb = rgb(56, 44, 94);
const WALL_DEEP: Rgb = rgb(26, 20, 50);
const RIB: Rgb = rgb(70, 50, 118);
const RING: Rgb = rgb(110, 255, 200);
const RING_DIM: Rgb = rgb(60, 150, 130);
const FILM: Rgb = rgb(140, 236, 216);
const FILM_DEEP: Rgb = rgb(70, 170, 160);
const BUBBLE: Rgb = rgb(190, 255, 240);
const GLASS: Rgb = rgb(120, 200, 190);
const BAR_IRON: Rgb = rgb(58, 32, 88);
const STUD: Rgb = rgb(170, 120, 240);

/// A porthole in a tube's wall: its depth down the tube (0 at the top,
/// `len` at the foot) and its bearing (0 is +z, a quarter turn +x).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Porthole {
    pub depth: i32,
    pub bearing: i32,
}

/// The vertical tube, radius `r`, its top at y 0 and its foot at y
/// `-len`: twelve-sided bands of dark violet panels with a glowing
/// sea-green rib between bands, and at each of `portholes` a round hole
/// through the wall, its ring glowing (`tube_porthole`). Returns a frame
/// outside each porthole for a creature to stand in, facing in (its feet
/// 2 m under the hole's middle, 3 m outside the wall). About 25 shapes a
/// 12 m band, 20 a porthole: about 800 for 300 m with ten portholes.
pub fn tube(
    b: &mut Batch,
    f: &Affine,
    r: i32,
    len: i32,
    portholes: &[Porthole],
    k: u32,
) -> Vec<Affine> {
    tube_with_door(b, f, r, len, portholes, None, k)
}

/// A doorway through the tube's wall at its foot, where the tunnel's
/// mouth opens: its bearing (0 is +z), its half width and its height
/// over the foot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TubeDoor {
    pub bearing: i32,
    pub half: i32,
    pub height: i32,
}

/// `tube`, and with a `door` the foot's bands open round it (no panel
/// and no rib across it) and a floor disc under the foot: the tube's
/// last segment over the tunnel's mouth.
pub fn tube_with_door(
    b: &mut Batch,
    f: &Affine,
    r: i32,
    len: i32,
    portholes: &[Porthole],
    door: Option<TubeDoor>,
    k: u32,
) -> Vec<Affine> {
    let bands = (len / BAND).max(1);
    let bh = len / bands;
    let panel_w = mul(2 * r, crate::fx::ratio(268, 1000)) + cm(40); // 2 r tan 15°, overlapping
    for j in 0..bands {
        let y0 = -bh * j;
        let y1 = -bh * (j + 1);
        // the door's bearings in this band, if it reaches up into it
        let open = door
            .filter(|d| y1 < -len + d.height)
            .map(|d| (d.bearing, crate::fx::atan2(d.half, r) + TURN / 24));
        let shut = |a: i32| open.is_none_or(|(at, half)| !angle_near(a, at, half));
        for i in 0..PANELS {
            let a = TURN * i / PANELS;
            if !shut(a) {
                continue;
            }
            let g = f.then(&Affine::rot_y(a));
            // a porthole on this panel in this band cuts it in two
            let hole = portholes.iter().find(|p| {
                let d = -p.depth;
                d <= y0 && d > y1 && angle_near(p.bearing, a, TURN / PANELS / 2)
            });
            let c = [WALL, WALL_PALE, WALL_DEEP]
                [pick(k.wrapping_mul(7) + (j * PANELS + i) as u32, 2, 0, 2) as usize];
            match hole {
                None => put(
                    b,
                    &g,
                    Mesh::Cube,
                    v3(0, (y0 + y1) / 2, r + int(1)),
                    v3(panel_w, bh - cm(20), int(2)),
                    c,
                    0,
                ),
                Some(p) => {
                    let d = -p.depth;
                    let (lo, hi) = (d - PORTHOLE_R, d + PORTHOLE_R);
                    if hi < y0 {
                        put(
                            b,
                            &g,
                            Mesh::Cube,
                            v3(0, (hi + y0) / 2, r + int(1)),
                            v3(panel_w, y0 - hi, int(2)),
                            c,
                            0,
                        );
                    }
                    if lo > y1 {
                        put(
                            b,
                            &g,
                            Mesh::Cube,
                            v3(0, (lo + y1) / 2, r + int(1)),
                            v3(panel_w, lo - y1, int(2)),
                            c,
                            0,
                        );
                    }
                    for s in [-1, 1] {
                        let x0 = s * PORTHOLE_R;
                        let x1 = s * panel_w / 2;
                        put(
                            b,
                            &g,
                            Mesh::Cube,
                            v3((x0 + x1) / 2, d, r + int(1)),
                            v3((x1 - x0).abs(), 2 * PORTHOLE_R, int(2)),
                            c,
                            0,
                        );
                    }
                }
            }
        }
        // the rib between bands: a glowing ring, dim and bright in turn
        // (round a door, a bar a panel, none across it)
        let c = if j % 3 == 0 { RING } else { RING_DIM };
        let glow = if j % 3 == 0 { ONE * 3 / 4 } else { ONE / 3 };
        if open.is_none() {
            ring(b, f, r + cm(40), y1 + cm(10), cm(30), c, glow);
        } else {
            for i in 0..PANELS {
                let a = TURN * i / PANELS;
                if shut(a) {
                    let g = f.then(&Affine::rot_y(a));
                    let (x, y, z) = (panel_w / 2, y1 + cm(10), r + cm(40));
                    bar(
                        b,
                        &g,
                        Mesh::Cube,
                        v3(-x, y, z),
                        v3(x, y, z),
                        cm(30),
                        c,
                        glow,
                    );
                }
            }
        }
    }
    if door.is_some() {
        // the floor the tunnel's floor runs on from
        let d = 2 * r + int(2);
        put(
            b,
            f,
            Mesh::Cyl,
            v3(0, -len - int(1), 0),
            v3(d, int(2), d),
            WALL_DEEP,
            0,
        );
    }
    let mut out = Vec::new();
    for (i, p) in portholes.iter().enumerate() {
        out.push(tube_porthole(b, f, r, p.depth, p.bearing, k + i as u32));
    }
    out
}

/// Whether bearing `a` is within `tol` of `b`.
fn angle_near(a: i32, b: i32, tol: i32) -> bool {
    crate::fx::wrap(a.wrapping_sub(b)).abs() <= tol
}

/// A ring of 12 bars at radius `r`, height `y`, `w` thick, in `c`.
fn ring(b: &mut Batch, f: &Affine, r: i32, y: i32, w: i32, c: Rgb, glow: i32) {
    let n = PANELS;
    for i in 0..n {
        bar(
            b,
            f,
            Mesh::Cube,
            round(r, TURN * i / n, y),
            round(r, TURN * (i + 1) / n, y),
            w,
            c,
            glow,
        );
    }
}

/// A porthole's ring round the hole at `depth` down a tube of radius `r`
/// at bearing `a`: a glowing octagonal ring of sea-green with brass studs
/// and a glassy film in the hole. Returns the creature's frame outside it
/// (see `tube`). About 20 shapes.
pub fn tube_porthole(b: &mut Batch, f: &Affine, r: i32, depth: i32, a: i32, k: u32) -> Affine {
    let g = f
        .then(&Affine::rot_y(a))
        .then(&Affine::translate(v3(0, -depth, r + int(1))));
    let n = 8;
    let rr = PORTHOLE_R + cm(40);
    for i in 0..n {
        let p = |j: i32| {
            let t = TURN * j / n;
            v3(mul(rr, sin(t)), mul(rr, cos(t)), 0)
        };
        bar(b, &g, Mesh::Cube, p(i), p(i + 1), cm(70), RING, ONE * 3 / 4);
        put(
            b,
            &g,
            Mesh::Octa,
            p(i),
            v3(cm(50), cm(50), cm(60)),
            STUD,
            ONE / 2,
        );
    }
    // the glass: a thin film in the hole, bubbles on it
    put_turned(
        b,
        &g,
        Mesh::Cyl,
        V3::ZERO,
        &Affine::rot_x(QUARTER),
        v3(2 * PORTHOLE_R, cm(16), 2 * PORTHOLE_R),
        GLASS,
        ONE / 4,
    );
    for j in 0..3u32 {
        let t = TURN * pick(k * 3 + j, 2, 0, 999) / 1000;
        let d = PORTHOLE_R * pick(k * 3 + j, 3, 20, 80) / 100;
        put(
            b,
            &g,
            Mesh::Sphere,
            v3(mul(d, sin(t)), mul(d, cos(t)), cm(20)),
            v3(cm(40), cm(40), cm(40)),
            BUBBLE,
            ONE / 2,
        );
    }
    halo_ball(b, &g, V3::ZERO, 3 * PORTHOLE_R, RING, ONE / 3);
    // the creature stands outside, 2 m under the hole's middle, facing in
    g.then(&Affine::translate(v3(0, -int(2), int(3))))
        .then(&Affine::rot_y(HALF))
}

/// The shapes a bounce pad takes across the tube: each covers part of the
/// width so the fall weaves round them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadShape {
    /// the +x half of the disc (turn it with `yaw`)
    Half,
    /// a ring with a hole half the radius in its middle
    Ring,
    /// two opposite quarters: +x+z and -x-z
    Quarters,
    /// a ring whose hole, of radius `hole`, is at `at` (in the pad's
    /// plane, from the axis): the physics' ring (`tower::PadShape::Ring`,
    /// its hole off the middle at the weave's offset)
    Holed { at: V3, hole: i32 },
}

impl PadShape {
    pub const ALL: [PadShape; 3] = [PadShape::Half, PadShape::Ring, PadShape::Quarters];

    /// Shape `k` of the three, round and round.
    pub fn nth(k: u32) -> PadShape {
        PadShape::ALL[(k % 3) as usize]
    }
}

/// The ring pad's hole, in radii.
pub const RING_HOLE: i32 = ONE / 2;

/// Whether a pad of `shape` in a tube of radius `r` (before its `yaw`)
/// covers the point `(x, z)` across the tube.
pub fn pad_covers(shape: PadShape, r: i32, x: i32, z: i32) -> bool {
    let d2 = (x as i64 * x as i64 + z as i64 * z as i64) >> 16;
    let r2 = (r as i64 * r as i64) >> 16;
    if d2 > r2 {
        return false;
    }
    match shape {
        PadShape::Half => x >= 0,
        PadShape::Ring => {
            let hole = mul(r, RING_HOLE);
            d2 >= (hole as i64 * hole as i64) >> 16
        }
        PadShape::Quarters => (x >= 0 && z >= 0) || (x <= 0 && z <= 0),
        PadShape::Holed { at, hole } => v3(x - at.x, 0, z - at.z).longer_than(hole),
    }
}

/// A bounce pad across a tube of radius `r`, flat in the xz plane at y 0
/// (its bouncing face up), turned `yaw` about the axis: a glassy
/// sea-green film (strips of it) with a bright glowing rim all round its
/// outline, bubbles swelling along the rim, and the whole breathing with
/// the tick; `squash` 0..ONE flares it (just bounced on). The glow reads
/// from above at falling speed. About 30–45 shapes and a halo.
#[allow(clippy::too_many_arguments)]
pub fn void_pad(
    b: &mut Batch,
    f: &Affine,
    r: i32,
    shape: PadShape,
    yaw: i32,
    squash: i32,
    tick: u32,
    k: u32,
) {
    let g = f.then(&Affine::rot_y(yaw));
    let sq = squash.clamp(0, ONE);
    let breathe = pulse(tick, 3, k, ONE * 9 / 10);
    let film = mix(FILM, [ONE; 3], sq / 2);
    let rim = mix(RING, [ONE; 3], sq / 3);
    let thick = cm(50);
    // the film: strips across the covered part
    let rows = 6;
    let strip = |b: &mut Batch, x0: i32, x1: i32, z0: i32, z1: i32| {
        put(
            b,
            &g,
            Mesh::Cube,
            v3((x0 + x1) / 2, 0, (z0 + z1) / 2),
            v3((x1 - x0).abs() + cm(10), thick, (z1 - z0).abs()),
            film,
            ONE / 2,
        );
    };
    let chord = |x: i32| -> i32 {
        let s = ((r as i64 * r as i64 - x as i64 * x as i64) >> 16).max(0) as i32;
        sqrt(s)
    };
    match shape {
        PadShape::Half => {
            for i in 0..rows {
                let x0 = r * i / rows;
                let x1 = r * (i + 1) / rows;
                let c = chord((x0 + x1) / 2);
                strip(b, x0, x1, -c, c);
            }
            // the rim: the straight edge and the arc
            bar(
                b,
                &g,
                Mesh::Cube,
                v3(0, 0, -r),
                v3(0, 0, r),
                cm(60),
                rim,
                ONE,
            );
            arc_rim(b, &g, r, -QUARTER, QUARTER, 8, rim, tick, k);
        }
        PadShape::Ring => {
            let hole = mul(r, RING_HOLE);
            let mid = (r + hole) / 2;
            let n = 16;
            for i in 0..n {
                bar(
                    b,
                    &g,
                    Mesh::Cube,
                    round(mid, TURN * i / n, 0),
                    round(mid, TURN * (i + 1) / n, 0),
                    r - hole,
                    film,
                    ONE / 2,
                );
            }
            arc_rim(b, &g, r, 0, TURN, 12, rim, tick, k);
            arc_rim(b, &g, hole, 0, TURN, 8, rim, tick, k + 7);
        }
        PadShape::Quarters => {
            for s in [1, -1] {
                for i in 0..rows {
                    let x0 = s * r * i / rows;
                    let x1 = s * r * (i + 1) / rows;
                    let c = chord((x0 + x1) / 2);
                    strip(b, x0, x1, 0, s * c);
                }
                bar(
                    b,
                    &g,
                    Mesh::Cube,
                    V3::ZERO,
                    v3(s * r, 0, 0),
                    cm(60),
                    rim,
                    ONE,
                );
                bar(
                    b,
                    &g,
                    Mesh::Cube,
                    V3::ZERO,
                    v3(0, 0, s * r),
                    cm(60),
                    rim,
                    ONE,
                );
            }
            arc_rim(b, &g, r, 0, QUARTER, 5, rim, tick, k);
            arc_rim(b, &g, r, HALF, HALF + QUARTER, 5, rim, tick, k + 3);
        }
        PadShape::Holed { at, hole } => {
            // a fan of film strips from the hole's rim out to the wall
            // (each wide enough to close the fan at the wall), a rim
            // round both
            let n = 24;
            let deep = mix(film, FILM_DEEP, ONE / 3);
            for i in 0..n {
                let a = TURN * i / n;
                let dir = v3(sin(a), 0, cos(a));
                let c = if (i + k as i32) % 2 == 0 { film } else { deep };
                bar(
                    b,
                    &g,
                    Mesh::Cube,
                    at.add(dir.scale(hole)),
                    dir.scale(r),
                    int(5),
                    c,
                    ONE / 2,
                );
            }
            arc_rim(b, &g, r, 0, TURN, 12, rim, tick, k);
            arc_rim(
                b,
                &g.then(&Affine::translate(at)),
                hole,
                0,
                TURN,
                8,
                rim,
                tick,
                k + 7,
            );
        }
    }
    let d = mul(2 * r, breathe);
    halo_ball(b, &g, v3(0, cm(20), 0), d, FILM_DEEP, ONE / 4 + sq / 2);
}

/// A glowing rim along the arc of radius `r` from bearing `a0` to `a1`
/// in `n` bars, a bubble swelling at each joint.
#[allow(clippy::too_many_arguments)]
fn arc_rim(b: &mut Batch, g: &Affine, r: i32, a0: i32, a1: i32, n: i32, c: Rgb, tick: u32, k: u32) {
    for i in 0..n {
        let p = round(r, a0 + (a1 - a0) * i / n, 0);
        let q = round(r, a0 + (a1 - a0) * (i + 1) / n, 0);
        bar(b, g, Mesh::Cube, p, q, cm(60), c, ONE);
        let s = mul(cm(90), pulse(tick, 2, k * 16 + i as u32, ONE / 2));
        put(b, g, Mesh::Sphere, p, v3(s, s, s), BUBBLE, ONE / 2);
    }
}

/// Where the pad at `depth` down the tube goes, for a tube `len` long
/// with `n` pads from `first` to `last` (depths): pad `i`'s depth.
pub fn pad_depth(i: u32, n: u32, first: i32, last: i32) -> i32 {
    if n <= 1 {
        return first;
    }
    first + (last - first) / (n as i32 - 1) * i as i32
}

/// The horizontal tunnel at the tube's foot, radius `r`, running from its
/// mouth at the origin along +z for `len`: the tube's own panels and ribs
/// laid on their side. The tube's foot opens into its mouth; the finish
/// is part way along it (the course draws that) and the banana sea past
/// its far end (`banana_sea`). About 25 shapes a 12 m.
pub fn tunnel(b: &mut Batch, f: &Affine, r: i32, len: i32, k: u32) {
    // the tube runs down -y; turn it so -y lies along +z
    let turn = Affine::rot_x(QUARTER);
    let turn = if turn.apply_dir(v3(0, -ONE, 0)).z > 0 {
        turn
    } else {
        Affine::rot_x(-QUARTER)
    };
    tube(b, &f.then(&turn), r, len, &[], k);
}

/// The tunnel's mouth, in the xy plane at z 0 facing -z (into the tube's
/// foot), radius `r`: a thick ring of void stone round the opening and,
/// across its top, the anchor bar the sling vines hang from, a violet
/// iron beam with glowing studs. Returns the bar's two ends (in `f`'s
/// frame): hang the vines along the line between them. About 33 shapes.
pub fn tunnel_mouth(b: &mut Batch, f: &Affine, r: i32, tick: u32, k: u32) -> (V3, V3) {
    let n = 12;
    let rr = r + int(1) + cm(50);
    for i in 0..n {
        let p = |j: i32| {
            let t = TURN * j / n;
            v3(mul(rr, sin(t)), mul(rr, cos(t)), 0)
        };
        let c = if i % 2 == 0 { WALL_PALE } else { RIB };
        bar(b, f, Mesh::Bevel, p(i), p(i + 1), int(3), c, 0);
        put(
            b,
            f,
            Mesh::Octa,
            p(i).add(v3(0, 0, -int(1))),
            v3(cm(80), cm(80), cm(80)),
            STUD,
            ONE / 2,
        );
    }
    // the anchor bar across the top of the mouth
    let y = mul(r, ONE * 4 / 5);
    let hw = mul(r, ONE * 3 / 5);
    let (a, z) = (v3(-hw, y, 0), v3(hw, y, 0));
    bar(
        b,
        f,
        Mesh::Hex,
        a.add(v3(-int(2), 0, 0)),
        z.add(v3(int(2), 0, 0)),
        int(1),
        BAR_IRON,
        0,
    );
    let studs = (2 * hw / int(4)).max(1);
    for i in 0..=studs {
        let p = a.lerp(z, ONE * i / studs);
        let s = mul(cm(60), pulse(tick, 2, k + i as u32, ONE * 3 / 5));
        put(
            b,
            f,
            Mesh::Octa,
            p.add(v3(0, -cm(60), 0)),
            v3(s, s, s),
            RING,
            ONE,
        );
    }
    halo_bar(b, f, a, z, int(3), RING, ONE / 3);
    // brackets up from the bar to the ring
    for s in [-1, 1] {
        bar(
            b,
            f,
            Mesh::Cube,
            v3(s * hw, y, 0),
            v3(s * mul(rr, sin(QUARTER / 3)), mul(rr, cos(QUARTER / 3)), 0),
            cm(40),
            BAR_IRON,
            0,
        );
    }
    let _ = (DEEP_SKY, wave as fn(u32, i32, i32) -> i32);
    (a, z)
}

/// The tube's foot joined to the tunnel's mouth: the tube's floor is
/// the tunnel's floor (its last segment drawn `tube_with_door`). The
/// frame for `tunnel` beside a tube whose foot (its floor, on its axis)
/// is at `foot`, the tunnel running off along bearing `a`: its axis `r`
/// over the floor, its mouth at the tube's wall (`r` out along `a`).
pub fn tunnel_frame(foot: &Affine, r: i32, a: i32) -> Affine {
    foot.then(&Affine::rot_y(a))
        .then(&Affine::translate(v3(0, r, r)))
}
