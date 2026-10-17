//! The second design's portal world (`notes/tower2-plan.md` steps 7–10):
//! the vertical tube the apes drop down (about 300 m long and 35 m
//! across, T100), in the zipper space's colours, with portholes the
//! creatures look in through; the trampolines floating in it (dark mats,
//! glowing bullseyes and bright padded rims, readable from above at
//! speed); and the
//! horizontal tunnel at its foot with the mouth's anchor bar the sling
//! vines hang from. The zip (`zip`), the creatures and the banana sea
//! are reused as they are.

use super::{bar, halo_ball, halo_bar, pick, pulse, put, put_turned, round, wave, DEEP_SKY};
use crate::fx::{cm, cos, int, mul, sin, HALF, ONE, QUARTER, TURN};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, rgb, Batch, Rgb};

/// The tube's length and radius (70 m across, T100), the tunnel's length
/// (a bridge's) and radius (the first tube's, about the tunnel's width).
pub const TUBE_LENGTH: i32 = int(300);
pub const TUBE_RADIUS: i32 = int(35);
pub const TUNNEL_LENGTH: i32 = int(250);
pub const TUNNEL_RADIUS: i32 = int(17) + cm(50);

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
const MAT: Rgb = rgb(34, 22, 58);
const PAD_A: Rgb = rgb(236, 90, 200);
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

/// A trampoline floating in the void (T100), its bouncing face the
/// frame's xz plane at y 0 (face up; lean the frame to lean it), `r` in
/// radius: a dark violet mat with a glowing sea-green bullseye, a padded
/// rim of eight segments in alternating magenta and sea-green, and a
/// shallow skirt under it; `squash` 0..ONE flares the mat (just bounced
/// on). 11 shapes.
pub fn void_trampoline(b: &mut Batch, f: &Affine, r: i32, squash: i32, tick: u32, k: u32) {
    let sq = squash.clamp(0, ONE);
    let breathe = pulse(tick, 3, k, ONE * 3 / 5);
    let mat = mix(MAT, FILM, sq / 2);
    let mat_r = r - cm(70);
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, -cm(10), 0),
        v3(2 * mat_r, cm(20), 2 * mat_r),
        mat,
        ONE / 8,
    );
    // the bullseye, a little proud of the mat
    let eye = r * 2 / 5;
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, cm(2), 0),
        v3(2 * eye, cm(6), 2 * eye),
        mix(FILM_DEEP, FILM, breathe),
        ONE / 2 + sq / 2,
    );
    // the padded rim
    let n = 8;
    for i in 0..n {
        let p = round(r - cm(35), TURN * i / n, 0);
        let q = round(r - cm(35), TURN * (i + 1) / n, 0);
        let c = if (i + k as i32) % 2 == 0 { PAD_A } else { RING };
        bar(b, f, Mesh::Cube, p, q, cm(70), c, ONE / 3);
    }
    // the skirt under it
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, -cm(60), 0),
        v3(2 * r - cm(40), cm(60), 2 * r - cm(40)),
        WALL_DEEP,
        0,
    );
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
/// frame for a `tunnel` of radius `tr` beside a tube of radius `r` whose
/// foot (its floor, on its axis) is at `foot`, the tunnel running off
/// along bearing `a`: its axis `tr` over the floor, its mouth at the
/// tube's wall (`r` out along `a`).
pub fn tunnel_frame(foot: &Affine, r: i32, tr: i32, a: i32) -> Affine {
    foot.then(&Affine::rot_y(a))
        .then(&Affine::translate(v3(0, tr, r)))
}
