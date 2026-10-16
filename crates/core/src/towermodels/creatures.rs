//! The zipper space's vine emitters, in place of the mages: wobbly
//! translucent blob creatures and Davy-Jones-like sea monsters, each with
//! a fixed point its vine starts from; and the descent's beams (the
//! spell beam in the deep spells' colours, with bubbles).

use super::{bar, halo_ball, pick, put, put_turned, round, spell_beam, wave, Spell};
use crate::fx::{cm, cos, deg, int, mul, sin, ONE, TURN};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, rgb, Batch, Rgb};

const EYE_WHITE: Rgb = rgb(240, 244, 220);
const EYE_BLACK: Rgb = rgb(10, 10, 16);
const MOUTH: Rgb = rgb(30, 16, 40);
const COAT: Rgb = rgb(30, 62, 60);
const COAT_DARK: Rgb = rgb(20, 40, 42);
const SKIN: Rgb = rgb(78, 128, 116);
const SKIN_DARK: Rgb = rgb(54, 92, 86);
const SUCKER: Rgb = rgb(150, 190, 170);
const CLAW: Rgb = rgb(168, 62, 48);
const CLAW_DARK: Rgb = rgb(110, 36, 30);
const BARNACLE: Rgb = rgb(200, 196, 174);
const CORAL: Rgb = rgb(222, 92, 150);
const CORAL_VIOLET: Rgb = rgb(150, 80, 200);
const HAT: Rgb = rgb(34, 28, 34);
const BRASS: Rgb = rgb(150, 136, 70);
const STAFF: Rgb = rgb(90, 70, 80);
const SEA_EYE: Rgb = rgb(120, 255, 200);
const BUBBLE: Rgb = rgb(190, 255, 240);

/// The blobs' bodies: sea-green, violet, sickly teal.
const BLOB_TINTS: [Rgb; 3] = [rgb(110, 230, 180), rgb(170, 120, 240), rgb(120, 200, 210)];

/// Where a blob `size` across starts its vine, in its frame: the glowing
/// tip of the arm it holds up (still; the body wobbles round it).
pub fn blob_beam_start(size: i32) -> V3 {
    v3(size * 3 / 5, size * 3 / 2, size / 5)
}

/// A blob creature, `size` across, sitting on y 0 facing +z: a wobbly
/// glowing body (its glow and a soft halo for translucence), lumps
/// rolling over it, three to five eyes (some on stalks) looking about, a
/// gaping mouth, and one arm held up with a glowing tip in `spell`'s
/// colour, where its vine starts. Returns that point in world space.
/// About 25 shapes.
pub fn blob(b: &mut Batch, f: &Affine, size: i32, spell: Spell, tick: u32, k: u32) -> V3 {
    let body = BLOB_TINTS[(k % 3) as usize];
    let wob = |p: i32| wave(tick, 2, p + k as i32 * 7_000);
    let sx = size + mul(size / 10, wob(0));
    let sy = size * 4 / 5 - mul(size / 10, wob(0));
    let centre = v3(0, sy / 2, 0);
    put(b, f, Mesh::Geo, centre, v3(sx, sy, sx), body, ONE / 3);
    halo_ball(b, f, centre, size * 3 / 2, body, ONE / 3);
    // lumps rolling over it, a shade deeper
    for j in 0..5 {
        let a = TURN * j / 5 + mul(wob(j * 4_000), deg(20));
        let p = round(sx * 2 / 5, a, sy * pick(k * 5 + j as u32, 1, 25, 70) / 100);
        let s = size * pick(k * 5 + j as u32, 2, 25, 40) / 100;
        put(
            b,
            f,
            Mesh::Blob,
            p,
            v3(s, s, s),
            mix(body, MOUTH, ONE / 6),
            ONE / 4,
        );
    }
    // eyes in an arc over the front, every other on a stalk
    let eyes = 3 + (k % 3) as i32;
    for e in 0..eyes {
        let a = deg(-55) + deg(110) * e / (eyes - 1);
        let base = round(sx * 2 / 5, a, sy * 3 / 5);
        let tip = if e % 2 == 1 {
            let t = base.add(v3(0, size / 3 + mul(size / 12, wob(e * 3_000)), size / 8));
            bar(b, f, Mesh::Taper, base, t, size / 10, body, ONE / 4);
            t
        } else {
            round(sx * 47 / 100, a, sy * 3 / 5)
        };
        let es = size * pick(k * 7 + e as u32, 3, 18, 30) / 100;
        put(b, f, Mesh::Ico, tip, v3(es, es, es), EYE_WHITE, ONE / 4);
        let look = mul(wave(tick, 3, e * 9_000 + k as i32 * 3_000), es / 4);
        let fwd = v3(mul(es * 2 / 5, sin(a)) + look, 0, mul(es * 2 / 5, cos(a)));
        put(
            b,
            f,
            Mesh::Ico,
            tip.add(fwd),
            v3(es / 2, es / 2, es / 2),
            EYE_BLACK,
            0,
        );
    }
    // a gaping mouth
    put(
        b,
        f,
        Mesh::Geo,
        v3(0, sy / 3, sx * 9 / 20),
        v3(sx / 3, sy / 6 + mul(sy / 14, wob(5_000)), size / 8),
        MOUTH,
        0,
    );
    // the arm held up, its tip glowing
    let tip = blob_beam_start(size);
    let shoulder = v3(sx * 2 / 5, sy * 3 / 5, 0);
    let elbow = shoulder
        .lerp(tip, ONE / 2)
        .add(v3(size / 6, mul(size / 12, wob(9_000)), 0));
    bar(b, f, Mesh::Taper, shoulder, elbow, size / 4, body, ONE / 3);
    bar(b, f, Mesh::Taper, elbow, tip, size / 6, body, ONE / 3);
    put(
        b,
        f,
        Mesh::Octa,
        tip,
        v3(size / 5, size / 5, size / 5),
        spell.colour(),
        ONE,
    );
    halo_ball(b, f, tip, size / 2, spell.colour(), ONE * 2 / 3);
    f.apply(tip)
}

/// Where a sea monster's vine starts, in its frame: the glowing orb on
/// its coral staff.
pub const MONSTER_STAFF_TIP: V3 = v3(cm(150), cm(560), cm(50));

/// A Davy-Jones-like sea monster about 5 m tall, its base at y 0 facing
/// +z, floating: a skirt of tentacles below a barnacled sea-captain's
/// coat, coral growing from one shoulder, a squid face with a beard of
/// tentacles swaying and glowing sea-green eyes, a battered tricorn, a
/// great crab claw for its left hand snapping, and in its right a coral
/// staff topped by an orb in `spell`'s colour, where its vine starts.
/// Returns that point in world space. About 75 shapes.
pub fn sea_monster(b: &mut Batch, f: &Affine, spell: Spell, tick: u32, k: u32) -> V3 {
    let sway = |p: i32| wave(tick, 3, p + k as i32 * 5_000);
    // the tentacle skirt, curling out and up at the tips
    for t in 0..6 {
        let a = TURN * t / 6 + deg(30);
        let mut p = round(cm(40), a, cm(160));
        let mut w = cm(36);
        for j in 1..=3 {
            let out = cm(30) + cm(20) * j;
            let q = round(
                cm(40) + out,
                a + mul(sway(t * 7_000 + j * 3_000), deg(15)),
                cm(160) - cm(50) * j + cm(15) * (j / 3),
            );
            bar(
                b,
                f,
                Mesh::Taper,
                p,
                q,
                w,
                if j == 3 { SKIN } else { SKIN_DARK },
                0,
            );
            p = q;
            w = w * 3 / 4;
        }
    }
    // the coat and its brass buttons
    put(
        b,
        f,
        Mesh::Barrel,
        v3(0, cm(240), 0),
        v3(cm(150), cm(190), cm(110)),
        COAT,
        0,
    );
    put(
        b,
        f,
        Mesh::Cone,
        v3(0, cm(165), -cm(10)),
        v3(cm(160), cm(70), cm(120)),
        COAT_DARK,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, cm(190), 0),
        v3(cm(156), cm(18), cm(116)),
        BRASS,
        0,
    );
    for i in 0..3 {
        put(
            b,
            f,
            Mesh::Octa,
            v3(cm(20), cm(220) + cm(30) * i, cm(55)),
            v3(cm(10), cm(10), cm(6)),
            BRASS,
            ONE / 4,
        );
    }
    // barnacles over the shoulders and coat
    for j in 0..8u32 {
        let a = deg(pick(k * 9 + j, 4, -150, 150));
        let y = cm(pick(k * 9 + j, 5, 220, 320));
        let s = cm(pick(k * 9 + j, 6, 12, 24));
        put(
            b,
            f,
            Mesh::Cone,
            round(cm(70), a, y),
            v3(s, s, s),
            BARNACLE,
            0,
        );
    }
    // coral growing from the right shoulder
    for j in 0..4 {
        let root = v3(cm(55), cm(320), -cm(20));
        let tip = root.add(v3(
            cm(30) + cm(15) * j,
            cm(60) + cm(20) * (j % 2),
            -cm(40) + cm(25) * j,
        ));
        bar(
            b,
            f,
            Mesh::Spike,
            root,
            tip,
            cm(18),
            if j % 2 == 0 { CORAL } else { CORAL_VIOLET },
            ONE / 6,
        );
    }
    // the head, eyes and brows
    let head = v3(0, cm(370), cm(5));
    put(b, f, Mesh::Ico, head, v3(cm(95), cm(90), cm(85)), SKIN, 0);
    for s in [-1, 1] {
        put(
            b,
            f,
            Mesh::Octa,
            head.add(v3(s * cm(22), cm(10), cm(40))),
            v3(cm(16), cm(12), cm(8)),
            SEA_EYE,
            ONE,
        );
        put_turned(
            b,
            f,
            Mesh::Wedge,
            head.add(v3(s * cm(22), cm(24), cm(36))),
            &Affine::rot_z(s * deg(15)),
            v3(cm(30), cm(10), cm(14)),
            SKIN_DARK,
            0,
        );
    }
    // the beard of tentacles, swaying
    for t in 0..8 {
        let x = -cm(35) + cm(10) * t;
        let mut p = head.add(v3(x, -cm(20), cm(30)));
        let mut w = cm(14);
        for j in 1..=3 {
            let s = sway(t * 6_000 + j * 4_000);
            let q = p.add(v3(mul(s, cm(12)), -cm(30), cm(6) - mul(s, cm(6))));
            bar(
                b,
                f,
                Mesh::Taper,
                p,
                q,
                w,
                if j == 3 { SUCKER } else { SKIN },
                0,
            );
            p = q;
            w = w * 3 / 4;
        }
    }
    // the battered tricorn
    let hat = f
        .then(&Affine::translate(head.add(v3(0, cm(48), 0))))
        .then(&Affine::rot_z(deg(-8)));
    put(
        b,
        &hat,
        Mesh::Cyl,
        V3::ZERO,
        v3(cm(150), cm(10), cm(130)),
        HAT,
        0,
    );
    put(
        b,
        &hat,
        Mesh::Bevel,
        v3(0, cm(25), 0),
        v3(cm(80), cm(45), cm(70)),
        HAT,
        0,
    );
    for j in 0..3 {
        let a = TURN * j / 3;
        let g = hat.then(&Affine::rot_y(a));
        put_turned(
            b,
            &g,
            Mesh::Cube,
            v3(0, cm(20), cm(58)),
            &Affine::rot_x(-deg(30)),
            v3(cm(110), cm(40), cm(6)),
            HAT,
            0,
        );
    }
    // the left arm: a great crab claw, snapping
    let shoulder = v3(-cm(70), cm(300), 0);
    let elbow = v3(-cm(130), cm(230), cm(40));
    let wrist = v3(-cm(140), cm(250), cm(110));
    bar(b, f, Mesh::Taper, shoulder, elbow, cm(40), COAT, 0);
    bar(b, f, Mesh::Taper, elbow, wrist, cm(46), CLAW_DARK, 0);
    put(
        b,
        f,
        Mesh::Ico,
        wrist.add(v3(0, 0, cm(25))),
        v3(cm(70), cm(60), cm(80)),
        CLAW,
        0,
    );
    let snap = mul((sway(20_000) + ONE) / 2, deg(25));
    for (s, len) in [(1, cm(100)), (-1, cm(70))] {
        let g = f
            .then(&Affine::translate(wrist.add(v3(0, 0, cm(55)))))
            .then(&Affine::rot_x(-s * (deg(10) + snap)));
        put_turned(
            b,
            &g,
            Mesh::Spike,
            v3(0, 0, len / 2),
            &Affine::rot_x(deg(90)),
            v3(cm(40), len, cm(30)),
            CLAW,
            0,
        );
    }
    // the right arm up, holding the coral staff
    let rs = v3(cm(70), cm(300), 0);
    let hand = v3(cm(135), cm(330), cm(40));
    bar(b, f, Mesh::Taper, rs, hand, cm(36), COAT, 0);
    put(b, f, Mesh::Ico, hand, v3(cm(28), cm(28), cm(28)), SKIN, 0);
    bar(
        b,
        f,
        Mesh::Taper,
        v3(cm(130), cm(80), cm(35)),
        MONSTER_STAFF_TIP,
        cm(14),
        STAFF,
        0,
    );
    for j in 0..2 {
        let at = MONSTER_STAFF_TIP.add(v3(0, -cm(40), 0));
        bar(
            b,
            f,
            Mesh::Spike,
            at,
            at.add(v3(cm(30) - cm(60) * j, cm(30), 0)),
            cm(10),
            CORAL,
            ONE / 6,
        );
    }
    put(
        b,
        f,
        Mesh::Octa,
        MONSTER_STAFF_TIP,
        v3(cm(36), cm(36), cm(36)),
        spell.colour(),
        ONE,
    );
    halo_ball(
        b,
        f,
        MONSTER_STAFF_TIP,
        int(1) + cm(40),
        spell.colour(),
        ONE * 2 / 3,
    );
    halo_ball(
        b,
        f,
        head.add(v3(0, cm(10), cm(40))),
        cm(80),
        SEA_EYE,
        ONE / 3,
    );
    f.apply(MONSTER_STAFF_TIP)
}

/// The descent's vine: a spell beam (one of `Spell::DEEP`) with bubbles
/// wobbling up round it. About 3 shapes a 3 m plus 7.
pub fn sea_beam(b: &mut Batch, f: &Affine, a: V3, z: V3, spell: Spell, tick: u32, k: u32) {
    spell_beam(b, f, a, z, spell, tick, k);
    for j in 0..4u32 {
        let t = ((tick.wrapping_mul(ONE as u32 / 240))
            .wrapping_add(j * ONE as u32 / 4 + k * 5_555))
            % ONE as u32;
        let wob = mul(wave(tick, 2, j as i32 * 16_000), cm(40));
        let s = cm(16) + cm(8) * (j % 2) as i32;
        put(
            b,
            f,
            Mesh::Sphere,
            a.lerp(z, t as i32).add(v3(wob, cm(30), -wob / 2)),
            v3(s, s, s),
            mix(BUBBLE, spell.colour(), ONE / 4),
            ONE / 2,
        );
    }
}
