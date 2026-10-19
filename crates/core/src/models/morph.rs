//! The Spindle's outfit (`Gear::Morph`, 2026-10-18; Sam, in the brief:
//! "the monkeys should be dressed in morph suits that make their entire
//! bodies a single reflective color, kinda like tron"). Each ape is drawn
//! as it always is, then everything it is made of (the fur, the face, the
//! eyes, the hands and feet, the nose, the tail) takes the suit's one
//! colour (`Batch::recolour_since`), so the suit follows the kind's own
//! shape: the gibbon's long arms, the proboscis monkey's nose, the ears,
//! the tails. A real morph suit covers the face, so this one does too.
//!
//! "Reflective": the renderer has no specular term, so the shine is made
//! from what there is: a vivid suit colour with a small glow of its own
//! (`SUIT_GLOW`) and a few lighter strips where a glossy suit catches the
//! light (`SHINE`, half lit). "Like Tron": thin seams in a light tint of
//! the suit (`seam_of`), drawn at full glow so they read in the tube's
//! even light, down the outside of every limb, round the wrists, the
//! ankles and the waist, in a V down the chest, down the spine, round the
//! tail and as a visor across the eyes. The racer's palette picks the
//! colour (`suit`): seven colourways, none near the vines' green or the
//! catch-up magenta. The map's Floor is Lava cat is `BLOCK_OCELOT`.

use super::*;
use crate::render::mix;

/// The colourways, in the kinds' order for the classic palette.
pub(super) const HOT_ORANGE: Rgb = rgb(255, 140, 0);
pub(super) const ELECTRIC_BLUE: Rgb = rgb(28, 108, 255);
pub(super) const RED: Rgb = rgb(255, 20, 20);
pub(super) const YELLOW: Rgb = rgb(255, 236, 40);
pub(super) const VIOLET: Rgb = rgb(150, 60, 255);
pub(super) const CYAN: Rgb = rgb(0, 222, 255);
pub(super) const WHITE: Rgb = rgb(238, 242, 255);
pub(super) const SUITS: [Rgb; 7] = [HOT_ORANGE, ELECTRIC_BLUE, RED, YELLOW, VIOLET, CYAN, WHITE];

/// The white suit's seams (a light tint of white would not show on it).
pub(super) const WHITE_SEAM: Rgb = rgb(70, 236, 255);

/// The suit's own glow: lit, but brighter than cloth (`Batch::push`).
pub(super) const SUIT_GLOW: i32 = ONE * 2 / 5;
/// The lighter strips where the suit catches the light.
pub(super) const SHINE_GLOW: i32 = ONE / 2;
/// The seams: drawn at full colour whatever the light.
pub(super) const SEAM_GLOW: i32 = ONE;

/// `kind`'s suit in `palette` (0 to `PALETTES` - 1; past that it wraps):
/// the six kinds wear six colours in the classic palette, and each kind's
/// four palettes are four of the seven.
pub(super) fn suit(kind: Kind, palette: u8) -> Rgb {
    SUITS[(kind.index() + (palette % PALETTES) as usize) % SUITS.len()]
}

/// The seams' colour on a suit of colour `c`: a light tint of it.
pub(super) fn seam_of(c: Rgb) -> Rgb {
    if c == WHITE {
        WHITE_SEAM
    } else {
        mix(c, rgb(255, 255, 255), ONE * 11 / 20)
    }
}

/// The shine on a suit of colour `c`: lighter, still its hue.
pub(super) fn shine_of(c: Rgb) -> Rgb {
    mix(c, rgb(255, 255, 255), ONE * 3 / 10)
}

/// How far out the torso's surface is, percent of its depth: the front at
/// the chest and at the belly, and the back.
fn surface(kind: Kind) -> (i32, i32, i32) {
    match kind {
        Kind::Gorilla => (56, 56, -54),
        Kind::Proboscis => (52, 72, -50),
        Kind::Orangutan => (54, 52, -58),
        _ => (54, 52, -50),
    }
}

/// The waist seam's girth, percent of the torso's width and depth, and
/// how far forward its middle is (the proboscis monkey's belly).
fn girth(kind: Kind) -> (i32, i32, i32) {
    match kind {
        Kind::Gorilla => (100, 102, -2),
        Kind::Proboscis => (104, 124, 12),
        _ => (96, 100, -2),
    }
}

/// The suit over the ape `kind` drawn since `from` (its body, limbs, head
/// and tail), posed as `s` at `tick`: every one of those shapes recoloured
/// to the suit with its own small glow, then the shine and the seams.
#[allow(clippy::too_many_arguments)]
pub(super) fn morph(
    rig: &mut Rig,
    kind: Kind,
    palette: u8,
    l: &Look,
    m: &Dims,
    s: &Skel,
    tick: u32,
    from: &[usize],
) {
    let c = suit(kind, palette);
    rig.b.recolour_since(from, |_, _| c);
    rig.b.reglow_since(from, SUIT_GLOW);
    shine(rig, kind, m, s, l, shine_of(c));
    let seam = seam_of(c);
    torso_seams(rig, kind, m, s, seam);
    limb_seams(rig, m, s, seam);
    visor(rig, kind, l, &s.head, seam);
    tail_seams(rig, kind, m, s, tick, seam);
}

/// A seam from `a` to `b`, `w` across, at full glow.
fn line(rig: &mut Rig, a: V3, b: V3, w: i32, c: Rgb) {
    if a.dist(b) > ONE / 200 {
        rig.glow(Mesh::Hex, rod(a, b, w), c, SEAM_GLOW);
    }
}

/// A seam round a limb at `at` along `along`, `across` wide.
fn ring(rig: &mut Rig, at: V3, along: V3, across: i32, w: i32, c: Rgb) {
    let half = dir_or(along, V3::UP).scale(w / 2);
    rig.glow(
        Mesh::Hex,
        rod(at.sub(half), at.add(half), across),
        c,
        SEAM_GLOW,
    );
}

/// The seams' width on this ape: thin, a little thicker on the big kinds.
fn seam_w(m: &Dims) -> i32 {
    pct(m.arm_w, 26).max(cm(2))
}

/// The waist, a V down the chest from the shoulders to the sternum and on
/// down the belly, and the spine.
fn torso_seams(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, c: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| body.at(pct(w, x), pct(t, y), pct(d, z));
    let sw = seam_w(m);
    let (chest, belly, back) = surface(kind);
    let (gw, gd, gz) = girth(kind);
    // round the waist
    rig.glow(
        Mesh::Barrel,
        body.shape(
            v3(0, pct(t, 22), pct(d, gz)),
            v3(pct(w, gw), sw, pct(d, gd)),
        ),
        c,
        SEAM_GLOW,
    );
    // the V and the line down the belly
    let sternum = at(0, 56, chest);
    for side in SIDES {
        line(rig, at(side * 30, 92, chest - 18), sternum, sw, c);
    }
    line(rig, sternum, at(0, 26, belly), sw, c);
    // down the spine, from the yoke to the waist
    line(rig, at(0, 96, back + 10), at(0, 74, back), sw, c);
    line(rig, at(0, 74, back), at(0, 26, back + 4), sw, c);
}

/// Down the outside of each arm and leg and the front of each forearm and
/// shin, round each wrist and ankle.
fn limb_seams(rig: &mut Rig, m: &Dims, s: &Skel, c: Rgb) {
    let sw = seam_w(m);
    for (i, side) in SIDES.into_iter().enumerate() {
        let out = s.body.x.scale(side);
        let fwd = s.head.z;
        // the arm, as `limbs` bends it
        let sh = shoulder_of(&s.body, m, side);
        let (elbow, wrist) = bend(sh, s.hands[i], m.arm, s.arm_hint[i]);
        let cuff = elbow.lerp(wrist, ONE * 82 / 100);
        let (upper, fore) = (pct(m.arm_w, 130), pct(m.arm_w, 112));
        along(rig, sh.lerp(elbow, ONE / 6), elbow, upper, out, sw, c);
        along(rig, elbow, cuff, fore, out, sw, c);
        along(rig, elbow, cuff, fore, fwd, sw, c);
        ring(rig, cuff, wrist.sub(elbow), pct(fore, 82), sw * 3 / 2, c);
        // the leg
        let hip = hip_of(&s.body, m, side);
        let (knee, ankle) = bend(hip, s.feet[i], m.leg, s.leg_hint[i]);
        let top = hip.lerp(knee, ONE / 6);
        let cuff = knee.lerp(ankle, ONE * 82 / 100);
        let (thigh, shin) = (pct(m.leg_w, 130), pct(m.leg_w, 110));
        along(rig, top, knee, thigh, out, sw, c);
        along(rig, knee, cuff, shin, out, sw, c);
        along(rig, knee, cuff, shin, fwd, sw, c);
        ring(rig, cuff, ankle.sub(knee), pct(shin, 82), sw * 3 / 2, c);
    }
}

/// A seam down a tapering limb segment from `a` (`w` across, as
/// `Rig::bone` draws it) to `b`, on its `out` side.
fn along(rig: &mut Rig, a: V3, b: V3, w: i32, out: V3, sw: i32, c: Rgb) {
    let way = b.sub(a);
    if way.len() < ONE / 200 {
        return;
    }
    let n = dir_or(out.reject(way.norm()), out);
    // (the six-sided bone is 0.43 of its width out at its flats and tapers
    // to six tenths of it at `b`; a little proud of that)
    let ra = pct(w, 48);
    let rb = pct(w, 31);
    line(rig, a.add(n.scale(ra)), b.add(n.scale(rb)), sw, c);
}

/// A visor across the eyes (a bar at their height in front of them and an
/// arm back each side to the ear), as Sector Ape's sunglasses sit.
fn visor(rig: &mut Rig, kind: Kind, l: &Look, h: &Frame, c: Rgb) {
    let d = cm(l.head);
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    let (ey, ex) = eye_spot(kind);
    let lw = (l.eye + 16).min(2 * ex - 3);
    let z = (42 + l.eye / 2 + 3).max(38 + (l.brow + 14) / 2 + 2);
    rig.glow(
        Mesh::Bevel,
        h.shape(p(0, ey, z), p(2 * ex + lw, 6, 4)),
        c,
        SEAM_GLOW,
    );
    for side in SIDES {
        let a = h.at(side * pct(d, ex + lw / 2), pct(d, ey), pct(d, z - 2));
        let b = h.at(side * pct(d, 49), pct(d, ey + 2), -pct(d, 8));
        line(rig, a, b, pct(d, 5), c);
    }
}

/// The lighter strips where a glossy suit catches the light: down each
/// side of the back (the top, on all fours), over the crown of the head
/// and down the front of each thigh.
fn shine(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, l: &Look, c: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let (_, _, back) = surface(kind);
    for side in SIDES {
        rig.glow(
            Mesh::Ico,
            body.shape(
                v3(side * pct(w, 22), pct(t, 62), pct(d, back + 8)),
                v3(pct(w, 10), pct(t, 50), pct(d, 10)),
            ),
            c,
            SHINE_GLOW,
        );
    }
    let h = &s.head;
    let hd = cm(l.head);
    rig.glow(
        Mesh::Ico,
        h.shape(
            v3(-pct(hd, 12), pct(hd, 40), pct(hd, 6)),
            v3(pct(hd, 14), pct(hd, 16), pct(hd, 56)),
        ),
        c,
        SHINE_GLOW,
    );
    let fwd = body.z;
    for (i, side) in SIDES.into_iter().enumerate() {
        let hip = hip_of(body, m, side);
        let (knee, _) = bend(hip, s.feet[i], m.leg, s.leg_hint[i]);
        let way = knee.sub(hip);
        if way.len() < ONE / 200 {
            continue;
        }
        let n = dir_or(fwd.reject(way.norm()), fwd);
        let r = pct(m.leg_w, 50);
        let a = hip.lerp(knee, ONE * 3 / 10).add(n.scale(r));
        let b = hip.lerp(knee, ONE * 8 / 10).add(n.scale(r * 4 / 5));
        rig.glow(Mesh::Hex, rod(a, b, pct(m.leg_w, 30)), c, SHINE_GLOW);
    }
}

/// Rings round the tail (the lemur's and the proboscis monkey's), on the
/// joints `tail` draws: on the lemur, every other one (its rings, in the
/// suit's light).
fn tail_seams(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, tick: u32, c: Rgb) {
    let (count, seg, w0, w1, bend_by) = match kind {
        Kind::Lemur => (12, cm(8), cm(9), cm(7), s.tail_bend),
        Kind::Proboscis => (8, cm(9), cm(6), cm(4), s.tail_bend / 3),
        _ => return,
    };
    // (the tail's own path, as `models::tail` walks it)
    let (lift, curl) = match kind {
        Kind::Lemur => (cm(40), s.tail_curl),
        _ => (cm(30), V3::UP.neg()),
    };
    let back = dir_or(s.tail_back.add(V3::UP.scale(lift)), s.body.z.neg());
    let curl = dir_or(curl.reject(back), s.body.z.neg());
    let side = back.cross(curl).norm();
    let mut at = s.body.at(0, pct(m.t, 10), -pct(m.d, 42));
    let every = if kind == Kind::Lemur { 2 } else { 3 };
    for i in 0..count {
        let a = bend_by * (i + 1);
        let sway = mul(fx::sin(spin(tick, deg(3)) - i * deg(20)), cm(25));
        let way = dir_or(
            back.scale(fx::cos(a))
                .add(curl.scale(fx::sin(a)))
                .add(side.scale(sway)),
            back,
        );
        let next = at.add(way.scale(seg));
        let w = w0 + (w1 - w0) * i / count;
        if i % every == 1 {
            let wide = if kind == Kind::Lemur { w } else { pct(w, 120) };
            ring(
                rig,
                at.lerp(next, ONE / 2),
                way,
                pct(wide, 108),
                seam_w(m),
                c,
            );
        }
        at = next;
    }
}

/// The Spindle's Floor is Lava cat: a blocky ocelot as Minecraft draws
/// one, as near as `BigCat` can: yellow-tan with dark brown spots, a pale
/// belly, green eyes and a thinner tail (the jaguar's build; its rounded
/// shapes stay).
pub(super) const OCELOT_COAT: Rgb = rgb(236, 192, 84);
pub(super) const OCELOT_DEEP: Rgb = rgb(196, 146, 52);
pub(super) const OCELOT_CREAM: Rgb = rgb(250, 228, 160);
pub(super) const OCELOT_SPOT: Rgb = rgb(70, 44, 22);
pub(super) const OCELOT_EYE: Rgb = rgb(96, 214, 70);

#[cfg(test)]
mod tests {
    use super::super::food::tests::read;
    use super::super::tests::{at_rest, poses, triangles, Shape};
    use super::*;
    use crate::costume::Costume;
    use crate::map::Map;

    fn drawn_in(kind: Kind, palette: u8, pose: &Pose) -> Vec<Shape> {
        let mut b = Batch::new();
        draw_in(&mut b, kind, palette, pose);
        read(b)
    }

    fn morph_pose(p: Pose) -> Pose {
        Pose {
            gear: Gear::Morph,
            ..p
        }
    }

    /// How far apart two colours are (the largest channel's difference,
    /// 0 to 255).
    fn apart(a: Rgb, b: Rgb) -> i32 {
        (0..3).map(|i| (a[i] - b[i]).abs()).max().unwrap() * 255 / ONE
    }

    #[test]
    fn on_the_spindle_every_kind_wears_a_morph_suit_in_one_colour_in_every_pose() {
        // Sam: "the monkeys should be dressed in morph suits that make their
        // entire bodies a single reflective color, kinda like tron"
        assert_eq!(Gear::from_key("morph"), Some(Gear::Morph));
        for map in Map::ALL {
            assert_eq!(
                Gear::on(map) == Gear::Morph,
                map.key() == "spindle",
                "{map:?}"
            );
        }
        for kind in Kind::ALL {
            for palette in 0..PALETTES {
                let c = suit(kind, palette);
                let (seam, shine) = (seam_of(c), shine_of(c));
                let lp = look_in(kind, palette);
                for p in poses() {
                    let s = drawn_in(kind, palette, &morph_pose(p));
                    let bare = drawn_in(kind, palette, &p);
                    // every shape the bare ape is made of, in the suit (but
                    // the hair a tight suit flattens: `shaved`)
                    let suited = s.iter().filter(|x| x.colour == c).count();
                    let stars = bare.iter().filter(|x| x.colour == STAR).count();
                    assert_eq!(
                        suited,
                        bare.len() - stars - shaved(kind, &bare, &s),
                        "{kind:?} {palette}"
                    );
                    for x in &s {
                        let ok = (x.colour == c && x.glow == SUIT_GLOW)
                            || (x.colour == seam && x.glow == SEAM_GLOW)
                            || (x.colour == shine && x.glow == SHINE_GLOW)
                            || (x.colour == STAR && p.stunned);
                        assert!(
                            ok,
                            "{kind:?} {palette}: mesh {} colour {:?} glow {}",
                            x.mesh, x.colour, x.glow
                        );
                    }
                    // nothing of the ape's own colours left
                    for own in [lp.fur, lp.limb, lp.chest, lp.face, lp.skin] {
                        assert!(
                            !s.iter().any(|x| x.colour == own) || own == c,
                            "{kind:?} {palette}: {own:?} shows"
                        );
                    }
                    let seams = s.iter().filter(|x| x.colour == seam).count();
                    assert!(seams >= 18, "{kind:?}: {seams} seams");
                    assert!(s.iter().filter(|x| x.colour == shine).count() >= 3);
                    assert!(s.len() <= 300, "{kind:?} {} shapes", s.len());
                    for x in &s {
                        for a in x.axes {
                            assert!(
                                a.len() >= ONE / 1000 && a.len() <= int(2),
                                "{kind:?} mesh {} axis {}",
                                x.mesh,
                                a.len()
                            );
                        }
                        assert!(x.at.dist(p.hands) <= int(3));
                    }
                }
            }
        }
    }

    fn same(x: &Shape, y: &Shape) -> bool {
        y.mesh == x.mesh && y.at == x.at && y.axes == x.axes
    }

    /// The bare ape's shapes the suit leaves out: the orangutan's arm hair
    /// and crest and the gibbon's crown (as under a hat), hair blades all.
    fn shaved(kind: Kind, bare: &[Shape], s: &[Shape]) -> usize {
        let lost = bare.iter().filter(|x| !s.iter().any(|y| same(x, y)));
        let lost: Vec<&Shape> = lost.collect();
        for x in &lost {
            assert!(
                matches!(kind, Kind::Orangutan | Kind::Gibbon) && x.mesh == Mesh::Shard as u32,
                "{kind:?}: mesh {} at {:?} lost",
                x.mesh,
                x.at
            );
        }
        lost.len()
    }

    #[test]
    fn the_suit_keeps_every_kinds_shape() {
        // the suit follows the body: the bare ape's every shape, in the
        // same place and size, only its colour changed; only the hair a
        // hat would flatten goes (the lemur's ears, the chimpanzee's
        // sideburns, the shoulder ruffs stay)
        for kind in Kind::ALL {
            for p in poses() {
                let bare = drawn_in(kind, 0, &p);
                let s = drawn_in(kind, 0, &morph_pose(p));
                let gone = shaved(kind, &bare, &s);
                assert!(gone <= 18, "{kind:?}: {gone} shapes gone");
            }
        }
    }

    #[test]
    fn the_colourways_are_vivid_and_apart_from_the_vines_and_the_catch_up_glow() {
        // the Spindle's vines (bright green) and the catch-up magenta
        let (vine, magenta) = (rgb(70, 255, 120), rgb(255, 30, 160));
        assert!(SUITS.len() >= 4);
        for (i, a) in SUITS.iter().enumerate() {
            // vivid: one channel near full
            assert!(a.iter().max().unwrap() * 255 / ONE >= 230, "{a:?}");
            assert!(apart(*a, vine) >= 100, "{a:?} near the vines");
            assert!(apart(*a, magenta) >= 100, "{a:?} near magenta");
            for b in &SUITS[i + 1..] {
                assert!(apart(*a, *b) >= 90, "{a:?} near {b:?}");
            }
            // the seams lighter than the suit, still told from it
            let s = seam_of(*a);
            assert!(apart(*a, s) >= 60, "{a:?}: seams lost on it");
        }
        // the six kinds in six colours, and each kind's palettes its own
        let classic: Vec<Rgb> = Kind::ALL.iter().map(|k| suit(*k, 0)).collect();
        for (i, a) in classic.iter().enumerate() {
            assert!(!classic[i + 1..].contains(a), "{a:?} twice");
        }
        for kind in Kind::ALL {
            let mine: Vec<Rgb> = (0..PALETTES).map(|p| suit(kind, p)).collect();
            for (i, a) in mine.iter().enumerate() {
                assert!(!mine[i + 1..].contains(a), "{kind:?}: {a:?} twice");
            }
            assert_eq!(suit(kind, PALETTES), suit(kind, 0));
        }
    }

    #[test]
    fn the_seams_glow_down_every_limb_round_the_waist_and_across_the_eyes() {
        for kind in Kind::ALL {
            let pose = morph_pose(at_rest());
            let s = drawn_in(kind, 0, &pose);
            let seam = seam_of(suit(kind, 0));
            let seams: Vec<&Shape> = s.iter().filter(|x| x.colour == seam).collect();
            assert!(seams.iter().all(|x| x.glow == ONE), "{kind:?}");
            let fwd = facing(pose.yaw, 0);
            let side = V3::UP.cross(fwd).norm();
            let middle = s
                .iter()
                .fold(V3::ZERO, |a, x| a.add(x.at.scale(ONE / s.len() as i32)));
            // seams out to both sides of the body, front and back, top and
            // bottom (the limbs', the spine's, the visor, the ankles')
            for dir in [side, side.neg(), fwd, fwd.neg(), V3::UP, V3::UP.neg()] {
                assert!(
                    seams.iter().any(|x| x.at.sub(middle).dot(dir) > cm(8)),
                    "{kind:?}: no seam toward {dir:?}"
                );
            }
            // the waist
            let waist = seams
                .iter()
                .filter(|x| x.mesh == Mesh::Barrel as u32)
                .count();
            assert_eq!(waist, 1, "{kind:?}: no waist seam");
            // the visor: across the face, ahead of the body's middle and
            // near the top of the ape
            let top = s.iter().map(|x| x.at.y).max().unwrap();
            let visor = seams
                .iter()
                .find(|x| x.mesh == Mesh::Bevel as u32)
                .unwrap_or_else(|| panic!("{kind:?}: no visor"));
            assert!(visor.at.sub(middle).dot(fwd) > 0, "{kind:?}: visor behind");
            assert!(top - visor.at.y < cm(60), "{kind:?}: visor low");
        }
    }

    #[test]
    fn the_morph_suit_covers_the_face() {
        // a real morph suit covers the face: no eye whites, pupils, nose,
        // mouth or bare face
        for kind in Kind::ALL {
            for p in poses() {
                let s = drawn_in(kind, 0, &morph_pose(p));
                let l = look(kind);
                for c in [
                    EYE_WHITE, LEMUR_EYE, BLACK, NOSE, FACE_RING, TAIL_GREY, l.face,
                ] {
                    assert!(!s.iter().any(|x| x.colour == c), "{kind:?}: {c:?}");
                }
                // and the eyes' glow is the suit's, not their own
                assert!(!s.iter().any(|x| x.glow == ONE / 3), "{kind:?}");
            }
        }
    }

    #[test]
    fn the_morph_suits_cost_less_than_the_cowboy_outfits() {
        let mut lines = Vec::new();
        for kind in Kind::ALL {
            let p = at_rest();
            let tris = |g: Gear| triangles(&drawn_in(kind, 0, &Pose { gear: g, ..p }));
            let bare = tris(Gear::None);
            let morph = tris(Gear::Morph) - bare;
            let cowboy = tris(Gear::Cowboy) - bare;
            lines.push(format!("{kind:?}: morph +{morph}, cowboy +{cowboy}"));
            assert!(morph <= cowboy, "{}", lines.join("; "));
        }
    }

    #[test]
    fn the_morph_suit_is_a_costume_unlocked_on_its_map() {
        assert!(Costume::ALL.contains(&Costume::Morph));
        assert_eq!(Costume::from_key("morph"), Some(Costume::Morph));
        assert_eq!(Costume::Morph.key(), "morph");
        assert_eq!(Gear::from_key(Costume::Morph.key()), Some(Gear::Morph));
        assert_eq!(Costume::of_map(Map::Spindle), Some(Costume::Morph));
        assert_eq!(Costume::Morph.map(), Some(Map::Spindle));
        assert!(!Costume::Morph.free());
        for map in Map::ALL {
            assert_eq!(Costume::Morph.gear(map), Gear::Morph);
        }
    }

    #[test]
    fn only_the_spindles_cat_is_the_block_ocelot() {
        for map in Map::ALL {
            assert_eq!(
                BigCat::on(map) == BLOCK_OCELOT,
                map.key() == "spindle",
                "{map:?}"
            );
        }
        let j = crate::jaguar::Jaguar {
            feet: V3::ZERO,
            yaw: deg(30),
            gait: deg(50),
            stride: ONE,
            leap: 0,
            head_down: 0,
            jaw: ONE / 2,
        };
        let mut b = Batch::new();
        jaguar(&mut b, &j, 100, &BLOCK_OCELOT);
        let s = read(b);
        // yellow-tan with dark spots and green eyes, none of the jaguar's
        // own coat
        for c in [OCELOT_COAT, OCELOT_SPOT, OCELOT_EYE] {
            assert!(s.iter().any(|x| x.colour == c), "no {c:?}");
        }
        assert!(!s
            .iter()
            .any(|x| x.colour == JAG_COAT || x.colour == JAG_SPOT));
        // green eyes
        assert!(OCELOT_EYE[1] > OCELOT_EYE[0] && OCELOT_EYE[1] > OCELOT_EYE[2]);
        // a cat with no costume pieces on it
        const { assert!(!BLOCK_OCELOT.helmet && !BLOCK_OCELOT.gummy && !BLOCK_OCELOT.flame) };
    }
}
