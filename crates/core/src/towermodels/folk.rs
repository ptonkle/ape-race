//! The school: mages (apes in robes and pointed hats, a staff with the
//! spell glowing at its tip) and dragons with mage riders. Low detail,
//! since there are many: about 14 shapes a mage, 60 a dragon and rider.

use super::{bar, dm, halo_ball, put, put_turned, wave, Spell};
use super::{BANNER, BEARD, FACE, FUR, GOLD, WOOD};
use crate::fx::{cm, deg, int, mul, ONE, QUARTER};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{rgb, shade, Batch, Rgb};

/// What a mage is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagePose {
    /// staff raised high, the other hand thrown forward: casting
    Cast,
    /// staff levelled forward like a lance, aiming the spell
    Aim,
    /// staff planted upright beside, watching
    Idle,
    /// seated astride (a dragon); the staff raised; the origin is the seat
    Ride,
}

/// How far a seated rider's body is lowered from a standing one.
const RIDE_DROP: i32 = -cm(85);

/// The staff's foot and tip, in the mage's frame.
fn staff(pose: MagePose) -> (V3, V3) {
    let (a, z) = match pose {
        MagePose::Cast | MagePose::Ride => (v3(dm(3), dm(12), -dm(2)), v3(dm(5), dm(31), dm(3))),
        MagePose::Aim => (v3(dm(3), dm(12), -dm(4)), v3(dm(3), dm(15), dm(17))),
        MagePose::Idle => (v3(dm(5), 0, dm(1)), v3(dm(5), dm(24), dm(1))),
    };
    if pose == MagePose::Ride {
        (a.add(v3(0, RIDE_DROP, 0)), z.add(v3(0, RIDE_DROP, 0)))
    } else {
        (a, z)
    }
}

/// Where a mage's spell leaves the staff, in the mage's frame: a beam (a
/// vine) starts here.
pub fn staff_tip(pose: MagePose) -> V3 {
    staff(pose).1
}

/// A mage of `spell`'s school in `pose`, feet at the origin facing +z
/// (or seated, `Ride`), about 2.4 m to the hat's tip. `k` varies the
/// beard and the pulse.
pub fn mage(b: &mut Batch, f: &Affine, spell: Spell, pose: MagePose, tick: u32, k: u32) {
    let robe = spell.robe();
    let dark = shade(robe, 70);
    let lift = if pose == MagePose::Ride { RIDE_DROP } else { 0 };
    let up = |p: V3| p.add(v3(0, lift, 0));
    if pose == MagePose::Ride {
        // the skirts over the saddle and the legs down its sides
        put(
            b,
            f,
            Mesh::Cone,
            v3(0, cm(15), 0),
            v3(cm(95), cm(50), cm(90)),
            robe,
            0,
        );
        for s in [-1, 1] {
            bar(
                b,
                f,
                Mesh::Cube,
                v3(s * dm(2), cm(5), dm(1)),
                v3(s * dm(5), -dm(6), dm(3)),
                cm(18),
                dark,
                0,
            );
        }
    } else {
        put(
            b,
            f,
            Mesh::Cone,
            v3(0, cm(55), 0),
            v3(cm(95), cm(110), cm(80)),
            robe,
            0,
        );
        put(
            b,
            f,
            Mesh::Cyl,
            v3(0, cm(4), 0),
            v3(cm(96), cm(8), cm(82)),
            GOLD,
            ONE / 6,
        );
    }
    put(
        b,
        f,
        Mesh::Ico,
        up(v3(0, cm(120), 0)),
        v3(cm(62), cm(55), cm(50)),
        robe,
        0,
    );
    put(
        b,
        f,
        Mesh::Ico,
        up(v3(0, cm(162), 0)),
        v3(cm(50), cm(48), cm(48)),
        FUR,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        up(v3(0, cm(158), cm(22))),
        v3(cm(34), cm(24), cm(10)),
        FACE,
        0,
    );
    if k.is_multiple_of(2) {
        put(
            b,
            f,
            Mesh::Shard,
            up(v3(0, cm(138), cm(24))),
            v3(cm(30), cm(46), cm(10)),
            BEARD,
            0,
        );
    }
    // the hat: a wide brim and a tall cone, its tip bent back a little
    put(
        b,
        f,
        Mesh::Cyl,
        up(v3(0, cm(188), 0)),
        v3(cm(86), cm(6), cm(86)),
        dark,
        0,
    );
    put_turned(
        b,
        f,
        Mesh::Cone,
        up(v3(0, cm(234), -cm(5))),
        &Affine::rot_x(-deg(12 + (k % 3) as i32 * 4)),
        v3(cm(50), cm(95), cm(50)),
        dark,
        0,
    );
    // the staff and the arms holding it
    let (foot, tip) = staff(pose);
    bar(b, f, Mesh::Hex, foot, tip, cm(8), WOOD, 0);
    let at = |t: i32| foot.lerp(tip, t);
    let (right, left) = match pose {
        MagePose::Cast | MagePose::Ride => (at(ONE * 45 / 100), up(v3(-dm(5), cm(160), dm(5)))),
        MagePose::Aim => (at(ONE * 3 / 10), at(ONE * 6 / 10)),
        MagePose::Idle => (at(ONE * 55 / 100), up(v3(-dm(4), cm(90), cm(5)))),
    };
    for (s, hand) in [(1, right), (-1, left)] {
        bar(
            b,
            f,
            Mesh::Cube,
            up(v3(s * cm(33), cm(135), 0)),
            hand,
            cm(17),
            robe,
            0,
        );
    }
    // the spell at the tip (and in the free hand, casting)
    let c = spell.colour();
    let p = super::pulse(tick, 1, k, ONE * 3 / 5);
    let d = mul(cm(34), p);
    put(b, f, Mesh::Ico, tip, v3(d, d, d), c, ONE);
    halo_ball(b, f, tip, mul(cm(110), p), c, ONE * 3 / 5);
    if matches!(pose, MagePose::Cast | MagePose::Ride) {
        put(
            b,
            f,
            Mesh::Octa,
            left.add(v3(0, cm(8), cm(8))),
            v3(cm(22), cm(22), cm(22)),
            c,
            ONE,
        );
    }
}

/// A dragon's colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hide {
    Emerald,
    Crimson,
    Bronze,
    Midnight,
}

impl Hide {
    /// Back, belly and wing membrane.
    fn colours(self) -> (Rgb, Rgb, Rgb) {
        match self {
            Hide::Emerald => (rgb(44, 128, 82), rgb(186, 204, 120), rgb(74, 160, 112)),
            Hide::Crimson => (rgb(150, 32, 34), rgb(232, 172, 96), rgb(196, 70, 56)),
            Hide::Bronze => (rgb(156, 108, 52), rgb(232, 204, 146), rgb(194, 142, 82)),
            Hide::Midnight => (rgb(42, 42, 86), rgb(116, 112, 176), rgb(76, 64, 136)),
        }
    }
}

/// A dragon's wings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wings {
    /// beating (with the tick)
    Flap,
    /// spread level
    Glide,
    /// spread, the whole dragon rolled into a turn
    Bank,
    /// half folded and swept back, nose down
    Dive,
}

/// The rider's seat on a dragon's back, in the dragon's frame.
pub const SADDLE: V3 = v3(0, dm(13), dm(6));

/// A dragon flying with a mage of `spell` riding it: about 11 m nose to
/// tail and 19 m across the wings, its middle at the origin, nose +z.
/// The wings beat with the tick (`Flap`) or hold a pose; the tail sways.
/// Returns where the rider's spell leaves the staff, in `f`'s space
/// (the beam's start). About 60 shapes.
pub fn dragon(
    b: &mut Batch,
    f: &Affine,
    hide: Hide,
    wings: Wings,
    spell: Spell,
    tick: u32,
    k: u32,
) -> V3 {
    let (back, belly, skin) = hide.colours();
    let g = match wings {
        Wings::Bank => f.then(&Affine::rot_z(deg(if k.is_multiple_of(2) {
            26
        } else {
            -26
        }))),
        Wings::Dive => f.then(&Affine::rot_x(deg(24))),
        _ => *f,
    };
    let along = Affine::rot_x(QUARTER);
    // body, belly, chest and haunch
    put_turned(
        b,
        &g,
        Mesh::Barrel,
        v3(0, 0, 0),
        &along,
        v3(int(2), int(5), cm(190)),
        back,
        0,
    );
    put_turned(
        b,
        &g,
        Mesh::Barrel,
        v3(0, -cm(35), cm(10)),
        &along,
        v3(cm(160), cm(440), cm(150)),
        belly,
        0,
    );
    put(
        b,
        &g,
        Mesh::Ico,
        v3(0, cm(30), int(2)),
        v3(cm(190), cm(180), int(2)),
        back,
        0,
    );
    put(
        b,
        &g,
        Mesh::Ico,
        v3(0, 0, -cm(180)),
        v3(cm(190), cm(170), cm(220)),
        back,
        0,
    );
    // the neck and head, the head turning a little with the tick
    let look = mul(wave(tick, 5, k as i32 * 3_000), cm(40));
    let neck = [
        v3(0, cm(60), cm(260)),
        v3(look / 3, cm(120), cm(360)),
        v3(look * 2 / 3, cm(170), cm(450)),
        v3(look, cm(190), cm(540)),
    ];
    for (i, w) in [cm(110), cm(85), cm(70)].into_iter().enumerate() {
        bar(b, &g, Mesh::Taper, neck[i], neck[i + 1], w, back, 0);
    }
    let h = neck[3];
    put(b, &g, Mesh::Bevel, h, v3(cm(90), cm(80), cm(110)), back, 0);
    put(
        b,
        &g,
        Mesh::Bevel,
        h.add(v3(0, -cm(15), cm(85))),
        v3(cm(65), cm(50), cm(120)),
        back,
        0,
    );
    put_turned(
        b,
        &g,
        Mesh::Cube,
        h.add(v3(0, -cm(48), cm(70))),
        &Affine::rot_x(deg(16)),
        v3(cm(55), cm(18), cm(110)),
        belly,
        0,
    );
    for s in [-1, 1] {
        bar(
            b,
            &g,
            Mesh::Spike,
            h.add(v3(s * cm(30), cm(35), -cm(20))),
            h.add(v3(s * cm(45), cm(90), -cm(110))),
            cm(22),
            BEARD,
            0,
        );
        put(
            b,
            &g,
            Mesh::Octa,
            h.add(v3(s * cm(38), cm(20), cm(35))),
            v3(cm(16), cm(12), cm(16)),
            rgb(255, 220, 60),
            ONE,
        );
    }
    // spines down the back
    for i in 0..5 {
        let z = int(2) - cm(120) * i;
        put(
            b,
            &g,
            Mesh::Spike,
            v3(0, cm(110) - cm(10) * i, z),
            v3(cm(20), cm(55), cm(40)),
            BEARD,
            0,
        );
    }
    // the tail, swaying
    let mut p = v3(0, 0, -cm(260));
    let mut w = int(1);
    for i in 1..=6 {
        let sway = mul(wave(tick, 3, k as i32 * 2_000 + i * 5_000), cm(30) * i);
        let q = v3(sway, -cm(15) * i, -cm(260) - cm(120) * i);
        bar(b, &g, Mesh::Taper, p, q, w, back, 0);
        p = q;
        w = w * 3 / 4;
    }
    put(
        b,
        &g,
        Mesh::Shard,
        p.add(v3(0, 0, -cm(40))),
        v3(cm(90), cm(110), cm(12)),
        skin,
        0,
    );
    // legs tucked up
    for s in [-1, 1] {
        bar(
            b,
            &g,
            Mesh::Taper,
            v3(s * cm(70), -cm(60), cm(160)),
            v3(s * cm(80), -cm(140), cm(200)),
            cm(45),
            back,
            0,
        );
        bar(
            b,
            &g,
            Mesh::Taper,
            v3(s * cm(80), -cm(50), -cm(160)),
            v3(s * cm(90), -cm(150), -cm(220)),
            cm(50),
            back,
            0,
        );
    }
    // the wings
    let (lift, sweep) = match wings {
        Wings::Flap => (deg(8) + mul(wave(tick, 1, k as i32 * 7_000), deg(38)), 0),
        Wings::Glide | Wings::Bank => (deg(6), deg(4)),
        Wings::Dive => (deg(18), deg(48)),
    };
    for s in [-1, 1] {
        let wf = g
            .then(&Affine::translate(v3(s * cm(80), cm(80), cm(120))))
            .then(&Affine::rot_y(s * sweep))
            .then(&Affine::rot_z(s * lift));
        let x = |m: i32| s * m;
        let elbow = v3(x(cm(380)), cm(30), cm(40));
        let wrist = v3(x(cm(650)), cm(10), -cm(40));
        let tip = v3(x(cm(950)), -cm(20), -cm(160));
        let f1 = v3(x(cm(850)), -cm(10), -cm(320));
        let f2 = v3(x(cm(600)), 0, -cm(380));
        bar(b, &wf, Mesh::Taper, V3::ZERO, elbow, cm(40), back, 0);
        bar(b, &wf, Mesh::Taper, elbow, wrist, cm(30), back, 0);
        bar(b, &wf, Mesh::Taper, wrist, tip, cm(18), back, 0);
        bar(b, &wf, Mesh::Taper, wrist, f1, cm(12), back, 0);
        bar(b, &wf, Mesh::Taper, wrist, f2, cm(12), back, 0);
        // the membrane in three panels
        put(
            b,
            &wf,
            Mesh::Cube,
            v3(x(cm(190)), cm(10), -cm(120)),
            v3(cm(380), cm(6), cm(300)),
            skin,
            ONE / 8,
        );
        let back_by = |p: V3, d: i32| p.add(v3(0, 0, -d));
        super::keep::slab(
            b,
            &wf,
            Mesh::Cube,
            back_by(elbow, cm(170)),
            back_by(wrist, cm(170)),
            cm(320),
            cm(6),
            skin,
            ONE / 8,
        );
        super::keep::slab(
            b,
            &wf,
            Mesh::Cube,
            back_by(wrist, cm(130)),
            back_by(tip.lerp(f1, ONE / 2), cm(40)),
            cm(180),
            cm(6),
            skin,
            ONE / 8,
        );
    }
    // the saddle and the rider
    put(
        b,
        &g,
        Mesh::Cube,
        SADDLE.add(v3(0, -cm(15), 0)),
        v3(int(1), cm(30), cm(120)),
        BANNER,
        0,
    );
    let seat = g.then(&Affine::translate(SADDLE));
    mage(b, &seat, spell, MagePose::Ride, tick, k);
    seat.apply(staff_tip(MagePose::Ride))
}
