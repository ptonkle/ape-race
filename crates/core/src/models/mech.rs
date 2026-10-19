//! The Spindle's outfit (`Gear::Mech`, 2026-10-19; Sam: "the monkey skins
//! are terrible. create mech / armored core 6 / gundam versions of each ape
//! in the design log for the costumes for the apes instead of the neon
//! suits, then implement it in the game"). The concept sheet is
//! `notes/spindle-shots/mech/concepts.svg`.
//!
//! Each ape is drawn as it always is, then everything it is made of (the
//! fur, the face, the hands, the tail) takes the dark frame colour of its
//! scheme (`Batch::recolour_since`): the mech's inner frame, showing at the
//! shoulders, elbows, wrists, hips, knees and ankles. Over that go hard
//! plates in the scheme's main and secondary colours on every limb
//! segment, a cuirass and a hip skirt on the torso, pauldrons on the
//! shoulders, boots, a helmet with a faceplate, and a backpack with two
//! boosters. The visor, mono-eye or eye lenses and the boosters' nozzles
//! are drawn at full glow (the renderer has no specular). Every ape wears
//! a caution stripe (sheared blocks, `stripe`) and a unit number painted
//! as a block with dark bars (`decal`), never as text.
//!
//! Each kind keeps its silhouette, as the concept sheet draws it:
//!
//! | kind | frame | what makes it that kind |
//! | --- | --- | --- |
//! | Orangutan | OR-02 Warden | bulky; cheek-flange armour round the visor; long arms, heavy fists |
//! | Gorilla | GR-01 Bulwark | the heavy tank: the widest plates and pauldrons, fist guards, a mono-eye under a crest |
//! | Chimpanzee | CH-03 Vanguard | the balanced middleweight: a V-fin crest and twin eyes |
//! | Gibbon | GB-04 Longreach | the light frame: slim plates on its long arms, forearm blade fins, a slim V-fin and a visor slit |
//! | Proboscis | PB-05 Ramnose | an armoured nose (ring segments and a ram plate), a pot-belly plate, a mono-eye |
//! | Lemur | LM-06 Ringtail | light and fast: a segmented ringed tail, antenna ears, twin round lenses |
//!
//! The racer's palette picks one of four schemes (`scheme`): RX White
//! (Gundam), AC Rust (Armored Core), Field Olive (military) and Black
//! Crimson; none near the vines' green or the catch-up magenta. The map's
//! Floor is Lava cat is `BLOCK_OCELOT`.

use super::*;

/// One colour scheme: the main armour, the secondary plates, an accent
/// (trim, the stripes on the pauldrons), the dark frame between the
/// plates, the glow of the visor and the boosters, and the V-fin's and
/// the antennae's colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Scheme {
    pub main: Rgb,
    pub second: Rgb,
    pub accent: Rgb,
    pub frame: Rgb,
    pub glow: Rgb,
    pub fin: Rgb,
}

/// Gundam: white, blue, red, a yellow glow and a gold V-fin.
pub(super) const RX_WHITE: Scheme = Scheme {
    main: rgb(238, 240, 246),
    second: rgb(36, 84, 214),
    accent: rgb(220, 34, 40),
    frame: rgb(44, 48, 60),
    glow: rgb(255, 210, 40),
    fin: rgb(240, 196, 40),
};
/// Armored Core: rusty orange over gunmetal, a cyan glow.
pub(super) const AC_RUST: Scheme = Scheme {
    main: rgb(214, 108, 36),
    second: rgb(92, 98, 108),
    accent: rgb(240, 190, 40),
    frame: rgb(36, 36, 40),
    glow: rgb(80, 220, 255),
    fin: rgb(240, 190, 40),
};
/// Military: olive and tan, an amber glow.
pub(super) const FIELD_OLIVE: Scheme = Scheme {
    main: rgb(128, 140, 64),
    second: rgb(196, 176, 128),
    accent: rgb(226, 110, 30),
    frame: rgb(42, 44, 36),
    glow: rgb(255, 140, 30),
    fin: rgb(196, 176, 128),
};
/// The ace's custom: crimson over black, gold trim, a red glow.
pub(super) const BLACK_CRIMSON: Scheme = Scheme {
    main: rgb(184, 22, 40),
    second: rgb(46, 44, 52),
    accent: rgb(220, 180, 70),
    frame: rgb(24, 22, 26),
    glow: rgb(255, 48, 36),
    fin: rgb(220, 180, 70),
};
pub(super) const SCHEMES: [Scheme; 4] = [RX_WHITE, AC_RUST, FIELD_OLIVE, BLACK_CRIMSON];

/// The caution stripes' yellow (on the frame's dark), the decal's light
/// block, and the visor's dark glass.
pub(super) const HAZARD: Rgb = rgb(244, 196, 36);
pub(super) const DECAL: Rgb = rgb(236, 236, 228);
pub(super) const VISOR: Rgb = rgb(12, 14, 18);

/// The plates: lit, a little brighter than paint so they read in the
/// tube's even light (`Batch::push`).
pub(super) const ARMOUR_GLOW: i32 = ONE / 4;
/// The visor, the eyes and the boosters: full colour whatever the light.
pub(super) const GLOW: i32 = ONE;

/// How far paint and trim stand out of the plate under them: faces closer
/// than this flicker through each other in the depth test.
pub(super) const PROUD: i32 = cm(1) / 2;

/// The scheme each kind wears in its classic palette (the concept sheet's):
/// the orangutan olive, the gorilla and the proboscis monkey rust, the
/// chimpanzee and the lemur white, the gibbon crimson.
fn classic(kind: Kind) -> usize {
    match kind {
        Kind::Orangutan => 2,
        Kind::Gorilla | Kind::Proboscis => 1,
        Kind::Chimpanzee | Kind::Lemur => 0,
        Kind::Gibbon => 3,
    }
}

/// `kind`'s scheme in `palette` (0 to `PALETTES` - 1; past that it wraps):
/// each kind's four palettes are the four schemes.
pub(super) fn scheme(kind: Kind, palette: u8) -> Scheme {
    SCHEMES[(classic(kind) + (palette % PALETTES) as usize) % SCHEMES.len()]
}

/// How each kind's armour is cut, percent of its limbs' widths: the
/// pauldrons, the plates across the limbs, and the plates' depth front to
/// back.
struct Cut {
    pauldron: i32,
    across: i32,
    deep: i32,
}

fn cut(kind: Kind) -> Cut {
    match kind {
        Kind::Gorilla => Cut {
            pauldron: 250,
            across: 150,
            deep: 156,
        },
        Kind::Orangutan => Cut {
            pauldron: 230,
            across: 146,
            deep: 150,
        },
        Kind::Chimpanzee => Cut {
            pauldron: 210,
            across: 140,
            deep: 146,
        },
        Kind::Proboscis => Cut {
            pauldron: 200,
            across: 138,
            deep: 144,
        },
        Kind::Gibbon => Cut {
            pauldron: 190,
            across: 132,
            deep: 140,
        },
        Kind::Lemur => Cut {
            pauldron: 180,
            across: 136,
            deep: 140,
        },
    }
}

/// The armour over the ape `kind` drawn since `from` (its body, limbs,
/// head and tail), posed as `s` (`pose` for the nose and the tail's sway):
/// every one of those shapes takes the frame colour, then the plates, the
/// helmet, the boosters, the stripe and the decal go on.
#[allow(clippy::too_many_arguments)]
pub(super) fn mech(
    rig: &mut Rig,
    kind: Kind,
    palette: u8,
    l: &Look,
    m: &Dims,
    s: &Skel,
    pose: &Pose,
    from: &[usize],
) {
    let c = scheme(kind, palette);
    rig.b.recolour_since(from, |_, _| c.frame);
    rig.b.reglow_since(from, 0);
    torso(rig, kind, m, s, &c);
    limbs(rig, kind, m, s, &c);
    boosters(rig, kind, m, s, &c);
    helmet(rig, kind, l, &s.head, pose, &c);
    tail(rig, kind, m, s, pose.tick, &c);
    markings(rig, kind, m, s, &c);
}

/// A plate.
fn plate(rig: &mut Rig, mesh: Mesh, a: Affine, c: Rgb) {
    rig.glow(mesh, a, c, ARMOUR_GLOW);
}

/// A frame halfway along `a`–`b`, its y along it and its z as near to
/// `fwd` as it can be.
fn along(a: V3, b: V3, fwd: V3) -> Frame {
    Frame::upright(a.lerp(b, ONE / 2), b.sub(a), fwd)
}

/// A plate round the limb segment `a`–`b` (`w` across at `a`, as
/// `Rig::bone` draws it), from `from` to `to` percent along it, `across`
/// and `deep` percent of `w`, its middle `ahead` percent of `w` toward
/// `fwd`.
#[allow(clippy::too_many_arguments)]
fn sleeve(
    rig: &mut Rig,
    a: V3,
    b: V3,
    fwd: V3,
    span: (i32, i32),
    w: i32,
    size: (i32, i32),
    ahead: i32,
    mesh: Mesh,
    c: Rgb,
) {
    let len = a.dist(b);
    if len < ONE / 100 {
        return;
    }
    let (from, to) = span;
    let f = along(a.lerp(b, ONE * from / 100), a.lerp(b, ONE * to / 100), fwd);
    plate(
        rig,
        mesh,
        f.shape(
            v3(0, 0, pct(w, ahead)),
            v3(pct(w, size.0), len * (to - from) / 100, pct(w, size.1)),
        ),
        c,
    );
}

/// The cuirass over the chest, a ridge and vents on it, the hip skirt and
/// each kind's own plate (the gorilla's slab chest, the orangutan's back
/// plate over its long hair, the proboscis monkey's pot belly).
fn torso(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, c: &Scheme) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    let wide = match kind {
        Kind::Gorilla => 118,
        Kind::Lemur => 121,
        _ => 113,
    };
    plate(
        rig,
        Mesh::Bevel,
        body.shape(at(0, 64, 2), at(wide, 60, 118)),
        c.main,
    );
    // the ridge down the middle of the chest and two vents either side
    plate(
        rig,
        Mesh::Wedge,
        body.shape(at(0, 70, 60), at(18, 44, 8)),
        c.second,
    );
    for side in SIDES {
        plate(
            rig,
            Mesh::Cube,
            body.shape(at(side * 28, 74, 62), at(26, 12, 4)),
            c.second,
        );
    }
    // the hip skirt
    plate(
        rig,
        Mesh::Bevel,
        body.shape(at(0, 14, -2), at(104, 26, 104)),
        c.second,
    );
    plate(
        rig,
        Mesh::Wedge,
        body.shape(at(0, 10, 52), at(40, 30, 10)),
        c.accent,
    );
    match kind {
        Kind::Gorilla => {
            for side in SIDES {
                plate(
                    rig,
                    Mesh::Bevel,
                    body.shape(at(side * 24, 76, 50), at(46, 30, 20)),
                    c.main,
                );
            }
        }
        Kind::Orangutan => {
            // a broad back plate over the long hair
            plate(
                rig,
                Mesh::Bevel,
                body.shape(at(0, 58, -56), at(118, 74, 34)),
                c.main,
            );
        }
        Kind::Proboscis => {
            plate(
                rig,
                Mesh::Barrel,
                body.shape(at(0, 34, 32), at(100, 76, 94)),
                c.second,
            );
        }
        _ => {}
    }
}

/// Pauldrons, plates on every limb segment, knee guards and boots; the
/// gorilla's fist guards, the gibbon's forearm blades.
fn limbs(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, c: &Scheme) {
    let k = cut(kind);
    let fwd = s.head.z;
    let aw = m.arm_w;
    let lw = m.leg_w;
    for (i, side) in SIDES.into_iter().enumerate() {
        let body = &s.body;
        let out = body.x.scale(side);
        // the pauldron, a stripe of the accent across its top
        let sh = shoulder_of(body, m, side);
        let p = k.pauldron;
        let mid = sh
            .add(out.scale(pct(aw, 40)))
            .add(body.y.scale(pct(aw, 26)));
        plate(
            rig,
            Mesh::Bevel,
            body.shape(
                V3::ZERO,
                v3(pct(aw, p), pct(aw, p * 4 / 5), pct(aw, p * 11 / 10)),
            )
            .at(mid),
            c.main,
        );
        plate(
            rig,
            Mesh::Cube,
            body.shape(
                V3::ZERO,
                v3(pct(aw, p + 16), pct(aw, p / 6), pct(aw, p * 7 / 10)),
            )
            .at(mid.add(body.y.scale(pct(aw, p * 7 / 20)))),
            c.accent,
        );
        // the arm
        let (elbow, wrist) = bend(sh, s.hands[i], m.arm, s.arm_hint[i]);
        let uw = pct(aw, 130);
        sleeve(
            rig,
            sh,
            elbow,
            fwd,
            (24, 80),
            uw,
            (k.across - 20, k.deep - 20),
            0,
            Mesh::Cube,
            c.second,
        );
        let fw = pct(aw, 112);
        sleeve(
            rig,
            elbow,
            wrist,
            fwd,
            (14, 84),
            fw,
            (k.across, k.deep),
            0,
            Mesh::Bevel,
            c.main,
        );
        if kind == Kind::Gibbon {
            // a blade fin down the outside of the long forearm
            let f = along(elbow, wrist, fwd);
            let len = elbow.dist(wrist);
            plate(
                rig,
                Mesh::Wedge,
                Affine {
                    // (mirrored on the left, but kept right-handed)
                    c0: f.z.scale(-side * pct(fw, 60)),
                    c1: f.y.scale(pct(len, 70)),
                    c2: f.x.scale(side * pct(fw, 90)),
                    t: f.at(side * pct(fw, 70), 0, 0),
                },
                c.second,
            );
        }
        if kind == Kind::Gorilla {
            // knuckle guards over the fists it walks on
            let f = Frame {
                o: wrist,
                ..s.feet_frame
            };
            plate(
                rig,
                Mesh::Bevel,
                f.shape(
                    v3(0, pct(aw, 10), pct(aw, 40)),
                    v3(pct(aw, 178), pct(aw, 100), pct(aw, 178)),
                ),
                c.second,
            );
        }
        // the leg
        let hip = hip_of(body, m, side);
        let (knee, ankle) = bend(hip, s.feet[i], m.leg, s.leg_hint[i]);
        let tw = pct(lw, 130);
        sleeve(
            rig,
            hip,
            knee,
            fwd,
            (22, 80),
            tw,
            (k.across - 16, k.deep - 16),
            0,
            Mesh::Cube,
            c.main,
        );
        let sw = pct(lw, 110);
        sleeve(
            rig,
            knee,
            ankle,
            fwd,
            (16, 86),
            sw,
            (k.across, k.deep),
            0,
            Mesh::Bevel,
            c.main,
        );
        // the knee guard, in front of the joint
        sleeve(
            rig,
            knee,
            ankle,
            fwd,
            (-6, 22),
            sw,
            (80, 46),
            62,
            Mesh::Cube,
            c.second,
        );
        // the boot over the foot
        let f = Frame {
            o: ankle,
            ..s.skis.map_or(s.feet_frame, |f| f[i])
        };
        plate(
            rig,
            Mesh::Bevel,
            f.shape(
                v3(0, pct(lw, 8), pct(lw, 56)),
                v3(pct(lw, 184), pct(lw, 82), pct(lw, 222)),
            ),
            c.second,
        );
    }
}

/// The backpack and two boosters on it: pods, dark nozzles pointing down
/// the back (behind, on all fours), and the glow in each nozzle with a
/// short flame.
fn boosters(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, c: &Scheme) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| body.at(pct(w, x), pct(t, y), pct(d, z));
    let back = if kind == Kind::Orangutan { -76 } else { -66 };
    plate(
        rig,
        Mesh::Cube,
        body.shape(
            v3(0, pct(t, 66), pct(d, back + 6)),
            v3(pct(w, 70), pct(t, 46), pct(d, 26)),
        ),
        c.second,
    );
    let pod = pct(w, if kind == Kind::Gorilla { 28 } else { 24 });
    for side in SIDES {
        let x = side * 30;
        let top = at(x, 92, back - 8);
        let bottom = at(x, 50, back - 10);
        plate(rig, Mesh::Hex, rod(top, bottom, pod), c.main);
        let nozzle = bottom.add(body.y.scale(-pct(t, 10)));
        rig.put(Mesh::Hex, rod(bottom, nozzle, pct(pod, 80)), c.frame);
        // the glow in the nozzle, and a short flame out of it
        let lip = nozzle.add(body.y.scale(-pct(t, 2)));
        rig.glow(Mesh::Hex, rod(nozzle, lip, pct(pod, 66)), c.glow, GLOW);
        let tip = lip.add(body.y.scale(-pct(t, 22)));
        rig.glow(Mesh::Spike, rod(tip, lip, pct(pod, 56)), c.glow, GLOW);
    }
}

/// The helmet over the head, a faceplate over the muzzle, the visor and
/// what glows in it, and each kind's crest: the gorilla's ridge, the V-fins
/// of the chimpanzee and the gibbon, the orangutan's cheek flanges, the
/// lemur's antenna ears and snout guard, the proboscis monkey's nose.
fn helmet(rig: &mut Rig, kind: Kind, l: &Look, h: &Frame, pose: &Pose, c: &Scheme) {
    let d = cm(l.head);
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    // (the gorilla's helmet in the secondary, so its head, low between
    // its shoulders, reads apart from its plates)
    let (helm, face) = if kind == Kind::Gorilla {
        (c.second, c.main)
    } else {
        (c.main, c.second)
    };
    plate(
        rig,
        Mesh::Bevel,
        h.shape(p(0, 8, -2), p(108, 96, 106)),
        helm,
    );
    // the faceplate over the muzzle, a dark vent across it
    if l.muzzle > 0 {
        let mz = l.muzzle;
        plate(
            rig,
            Mesh::Bevel,
            h.shape(
                p(0, -26, 44),
                p(pct(mz, 110) + 10, pct(mz, 74) + 10, pct(mz, 90) + 10),
            ),
            face,
        );
        rig.put(
            Mesh::Cube,
            h.shape(p(0, -30, 49 + pct(mz, 45)), p(pct(mz, 60), 4, 4)),
            c.frame,
        );
    }
    // the visor: dark glass across the eyes
    let (ey, ex) = eye_spot(kind);
    let span = 2 * ex + l.eye + 14;
    rig.put(
        Mesh::Cube,
        h.shape(p(0, ey, 50), p(span, pct(l.eye, 80) + 4, 12)),
        VISOR,
    );
    let z = 56;
    match kind {
        Kind::Gorilla | Kind::Proboscis => {
            // the mono-eye, sweeping along its rail
            let x = mul(fx::sin(spin(pose.tick, deg(2))), pct(d, ex));
            let e = pct(d, l.eye * 2 / 3);
            rig.glow(
                Mesh::Hex,
                rod(
                    h.at(x, pct(d, ey), pct(d, z - 2)),
                    h.at(x, pct(d, ey), pct(d, z + 2)),
                    e,
                ),
                c.glow,
                GLOW,
            );
        }
        Kind::Chimpanzee | Kind::Lemur => {
            // twin eyes
            for side in SIDES {
                let e = if kind == Kind::Lemur {
                    l.eye * 3 / 4
                } else {
                    l.eye / 2
                };
                rig.glow(
                    Mesh::Hex,
                    rod(
                        h.at(side * pct(d, ex), pct(d, ey), pct(d, z - 2)),
                        h.at(side * pct(d, ex), pct(d, ey), pct(d, z + 2)),
                        pct(d, e),
                    ),
                    c.glow,
                    GLOW,
                );
            }
        }
        Kind::Orangutan | Kind::Gibbon => {
            // a visor bar
            rig.glow(
                Mesh::Cube,
                h.shape(p(0, ey, z), p(span - 10, pct(l.eye, 36) + 2, 4)),
                c.glow,
                GLOW,
            );
        }
    }
    match kind {
        Kind::Gorilla => {
            // a crest ridge over the brow, front to back
            plate(
                rig,
                Mesh::Bevel,
                h.shape(p(0, 54, -6), p(26, 24, 92)),
                c.main,
            );
            plate(
                rig,
                Mesh::Wedge,
                h.shape(p(0, 34, 52), p(104, 14, 18)),
                c.main,
            );
        }
        Kind::Chimpanzee | Kind::Gibbon => {
            // the V-fin, and a small crest between its arms
            let (reach, rise, wide) = if kind == Kind::Gibbon {
                (44, 96, 6)
            } else {
                (50, 92, 9)
            };
            for side in SIDES {
                let root = h.at(side * pct(d, 4), pct(d, 34), pct(d, 54));
                let tip = h.at(side * pct(d, reach), pct(d, rise), pct(d, 46));
                plate(rig, Mesh::Cube, rod(root, tip, pct(d, wide)), c.fin);
            }
            plate(
                rig,
                Mesh::Bevel,
                h.shape(p(0, 34, 54), p(12, 14, 8)),
                c.accent,
            );
        }
        Kind::Orangutan => {
            // the cheek flanges' armour: a broad plate round the face
            let a = h.at(0, -pct(d, 6), pct(d, 16));
            plate(
                rig,
                Mesh::Hex,
                rod(a, a.add(h.z.scale(pct(d, 8))), pct(d, 206)),
                c.second,
            );
        }
        Kind::Lemur => {
            // antennae along the pricked ears, and a guard over the snout
            for side in SIDES {
                let base = h.at(side * pct(d, 28), pct(d, 40), -pct(d, 4));
                let tip = h.at(side * pct(d, 70), pct(d, 110), -pct(d, 16));
                plate(rig, Mesh::Cube, rod(base, tip, pct(d, 8)), c.fin);
            }
            let a = h.at(0, -pct(d, 14), pct(d, 40));
            let b = h.at(0, -pct(d, 20), pct(d, 84));
            plate(
                rig,
                Mesh::Bevel,
                along(a, b, h.y).shape(V3::ZERO, v3(pct(d, 40), a.dist(b), pct(d, 34))),
                c.main,
            );
        }
        Kind::Proboscis => {
            // the nose in armour: three rings, a dark joint between each,
            // and a ram plate over the bulb in the accent, so it reads
            // against the chest
            let n = nose_points(h, d, pose.vel, pose.tick);
            let widths = [pct(d, 34), pct(d, 40), pct(d, 50)];
            for i in 0..3 {
                let a = n[i + 1].lerp(n[i], ONE / 8);
                let b = n[i + 1].lerp(n[i], ONE * 7 / 8);
                plate(rig, Mesh::Hex, rod(a, b, widths[i]), c.second);
            }
            plate(
                rig,
                Mesh::Bevel,
                h.shape(V3::ZERO, p(66, 60, 62)).at(n[3]),
                c.accent,
            );
        }
    }
}

/// Plates on every other joint of the tail (the lemur's rings, in armour;
/// the proboscis monkey's tail too), on the joints `models::tail` draws.
fn tail(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, tick: u32, c: &Scheme) {
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
        if i % 2 == 0 {
            let wide = if kind == Kind::Lemur { w } else { pct(w, 120) };
            plate(
                rig,
                Mesh::Hex,
                rod(
                    at.lerp(next, ONE / 10),
                    at.lerp(next, ONE * 9 / 10),
                    pct(wide, 134),
                ),
                c.main,
            );
        }
        at = next;
    }
}

/// A caution stripe (dark, with three sheared yellow bars) and the unit
/// number (a light block with dark bars, one more for each kind), each
/// where the concept sheet puts it.
fn markings(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, c: &Scheme) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let fwd = s.head.z;
    // the stripe: on the hip skirt, the belly, a shin, a forearm or a thigh
    let (f, size) = match kind {
        Kind::Gorilla => (
            Frame {
                o: body.at(-pct(w, 26), pct(t, 14), pct(d, 54)),
                ..*body
            },
            v3(pct(w, 34), pct(t, 16), pct(d, 4)),
        ),
        Kind::Orangutan | Kind::Proboscis => (
            Frame {
                o: body.at(
                    0,
                    pct(t, 30),
                    pct(d, if kind == Kind::Proboscis { 80 } else { 60 }),
                ),
                ..*body
            },
            v3(pct(w, 60), pct(t, 10), pct(d, 4)),
        ),
        _ => {
            let side = 1;
            let (a, b, w, from) = match kind {
                Kind::Gibbon => {
                    let sh = shoulder_of(body, m, side);
                    let (elbow, wrist) = bend(sh, s.hands[1], m.arm, s.arm_hint[1]);
                    (elbow, wrist, pct(m.arm_w, 112), 50)
                }
                Kind::Lemur => {
                    let hip = hip_of(body, m, side);
                    let (knee, _) = bend(hip, s.feet[1], m.leg, s.leg_hint[1]);
                    (hip, knee, pct(m.leg_w, 130), 50)
                }
                _ => {
                    let hip = hip_of(body, m, side);
                    let (knee, ankle) = bend(hip, s.feet[1], m.leg, s.leg_hint[1]);
                    (knee, ankle, pct(m.leg_w, 110), 40)
                }
            };
            let k = cut(kind);
            let f = along(
                a.lerp(b, ONE * from / 100),
                a.lerp(b, ONE * (from + 20) / 100),
                fwd,
            );
            let len = a.dist(b);
            (
                Frame {
                    o: f.at(0, 0, pct(w, k.deep / 2 + 2)),
                    ..f
                },
                v3(pct(w, k.across - 16), pct(len, 18), pct(w, 6)),
            )
        }
    };
    stripe(rig, &f, size, c.frame);
    // the unit number on the left pauldron's outside
    let sh = shoulder_of(body, m, -1);
    let p = cut(kind).pauldron;
    let aw = m.arm_w;
    let face = sh
        .add(body.x.scale(-(pct(aw, 40) + pct(aw, p / 2) + pct(aw, 2))))
        .add(body.y.scale(pct(aw, 20)));
    let f = Frame {
        o: face,
        x: body.z,
        y: body.y,
        z: body.x.neg(),
    };
    decal(rig, &f, pct(aw, p * 6 / 10), kind.index() as i32 + 1, c);
}

/// A caution stripe in `f`'s plane (x across, y up, z out of it), `size`:
/// a dark band and three yellow bars sheared across it.
fn stripe(rig: &mut Rig, f: &Frame, size: V3, dark: Rgb) {
    rig.put(Mesh::Cube, f.shape(V3::ZERO, size), dark);
    let bar = size.x / 7;
    for i in -1..=1 {
        rig.glow(
            Mesh::Cube,
            Affine {
                c0: f.x.scale(bar),
                // (the bar leans across the band: sheared, not turned)
                c1: f.y.scale(size.y * 4 / 5).add(f.x.scale(size.y * 4 / 5)),
                c2: f.z.scale(size.z + PROUD * 2),
                t: f.at(i * size.x * 2 / 7, 0, 0),
            },
            HAZARD,
            ARMOUR_GLOW,
        );
    }
}

/// The unit number as paint: a light block `across` wide on `f` (z out of
/// the plate), with `n` dark bars on it.
fn decal(rig: &mut Rig, f: &Frame, across: i32, n: i32, c: &Scheme) {
    let tall = across * 2 / 3;
    let thick = (across / 12).max(PROUD);
    rig.glow(
        Mesh::Cube,
        f.shape(V3::ZERO, v3(across, tall, thick)),
        if c.main == RX_WHITE.main {
            c.second
        } else {
            DECAL
        },
        ARMOUR_GLOW,
    );
    let gap = across / (n + 1);
    for i in 0..n {
        rig.put(
            Mesh::Cube,
            f.shape(
                v3(-across / 2 + gap * (i + 1), 0, thick / 2 + PROUD / 4),
                v3(across / 10, tall * 3 / 5, PROUD),
            ),
            c.frame,
        );
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
pub(super) mod tests {
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

    fn mech_pose(p: Pose) -> Pose {
        Pose {
            gear: Gear::Mech,
            ..p
        }
    }

    /// How far apart two colours are (the largest channel's difference,
    /// 0 to 255).
    fn apart(a: Rgb, b: Rgb) -> i32 {
        (0..3).map(|i| (a[i] - b[i]).abs()).max().unwrap() * 255 / ONE
    }

    pub(in crate::models) fn same(x: &Shape, y: &Shape) -> bool {
        y.mesh == x.mesh && y.at == x.at && y.axes == x.axes
    }

    /// The bare ape's shapes the armour leaves out: the orangutan's arm
    /// hair and crest and the gibbon's crown (as under a hat), hair blades
    /// all.
    fn shaved(kind: Kind, bare: &[Shape], s: &[Shape]) -> usize {
        let lost: Vec<&Shape> = bare
            .iter()
            .filter(|x| !s.iter().any(|y| same(x, y)))
            .collect();
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

    /// The shapes the armour added: those not in the bare ape.
    fn added<'a>(bare: &[Shape], s: &'a [Shape]) -> Vec<&'a Shape> {
        s.iter()
            .filter(|y| !bare.iter().any(|x| same(x, y)))
            .collect()
    }

    #[test]
    fn on_the_spindle_every_kind_wears_mech_armour_in_every_pose() {
        // Sam: "create mech / armored core 6 / gundam versions of each ape
        // ... for the costumes for the apes instead of the neon suits"
        assert_eq!(Gear::from_key("mech"), Some(Gear::Mech));
        for map in Map::ALL {
            assert_eq!(
                Gear::on(map) == Gear::Mech,
                map.key() == "spindle",
                "{map:?}"
            );
        }
        for kind in Kind::ALL {
            for palette in 0..PALETTES {
                let c = scheme(kind, palette);
                let lp = look_in(kind, palette);
                let paints = [
                    c.main, c.second, c.accent, c.frame, c.glow, c.fin, HAZARD, DECAL, VISOR,
                ];
                for p in poses() {
                    let s = drawn_in(kind, palette, &mech_pose(p));
                    let bare = drawn_in(kind, palette, &p);
                    // the ape underneath, every shape of it in place (but
                    // the hair a helmet flattens) and in the dark frame
                    shaved(kind, &bare, &s);
                    for x in bare.iter().filter(|x| x.colour != STAR) {
                        if let Some(y) = s.iter().find(|y| same(x, y)) {
                            assert!(
                                y.colour == c.frame && y.glow == 0,
                                "{kind:?} {palette}: mesh {} not in the frame",
                                x.mesh
                            );
                        }
                    }
                    // nothing of the ape's own colours left
                    for own in [
                        lp.fur, lp.limb, lp.chest, lp.face, lp.skin, EYE_WHITE, LEMUR_EYE, NOSE,
                        FACE_RING, TAIL_GREY,
                    ] {
                        assert!(
                            !s.iter().any(|x| x.colour == own) || paints.contains(&own),
                            "{kind:?} {palette}: {own:?} shows"
                        );
                    }
                    // the armour: only the scheme's paints, plates enough
                    // to cover it, nothing floating off the body
                    let more = added(&bare, &s);
                    for x in more.iter().filter(|x| x.colour != STAR) {
                        assert!(
                            paints.contains(&x.colour),
                            "{kind:?} {palette}: mesh {} colour {:?}",
                            x.mesh,
                            x.colour
                        );
                        let near = bare
                            .iter()
                            .filter(|b| b.colour != STAR)
                            .map(|b| b.at.dist(x.at))
                            .min()
                            .unwrap();
                        assert!(
                            near <= pct(cm(36), look(kind).size),
                            "{kind:?} {palette}: mesh {} colour {:?} floats {near} off the body",
                            x.mesh,
                            x.colour
                        );
                        for a in x.axes {
                            assert!(
                                a.len() >= ONE / 1000 && a.len() <= int(2),
                                "{kind:?} mesh {} axis {}",
                                x.mesh,
                                a.len()
                            );
                        }
                    }
                    let count = |col: Rgb| more.iter().filter(|x| x.colour == col).count();
                    assert!(
                        count(c.main) >= 8,
                        "{kind:?}: {} main plates",
                        count(c.main)
                    );
                    assert!(count(c.second) >= 6, "{kind:?}: {} second", count(c.second));
                    assert!(more.len() <= 90, "{kind:?}: {} shapes added", more.len());
                }
            }
        }
    }

    #[test]
    fn each_kind_keeps_its_shape_in_armour() {
        // the concept sheet's frames: what makes each one its kind
        let pose = mech_pose(at_rest());
        let fwd = facing(pose.yaw, 0);
        let side = V3::UP.cross(fwd).norm();
        let mut widths = Vec::new();
        for kind in Kind::ALL {
            let c = scheme(kind, 0);
            let bare = drawn_in(kind, 0, &at_rest());
            let s = drawn_in(kind, 0, &pose);
            let gone = shaved(kind, &bare, &s);
            assert!(gone <= 18, "{kind:?}: {gone} shapes gone");
            let more = added(&bare, &s);
            let n = |mesh: Mesh, col: Rgb| {
                more.iter()
                    .filter(|x| x.mesh == mesh as u32 && x.colour == col)
                    .count()
            };
            // (rods in the fin's colour on the helmet)
            let head = cm(look(kind).head) * look(kind).size / 100;
            let visor = s.iter().find(|v| v.colour == VISOR).unwrap().at;
            let fins = more
                .iter()
                .filter(|x| x.mesh == Mesh::Cube as u32 && x.colour == c.fin)
                .filter(|x| x.at.dist(visor) < head * 2)
                // (long and thin, not a stripe across a pauldron)
                .filter(|x| x.axes[1].len() > x.axes[0].len() * 3)
                .count();
            // (lights in the visor, not the boosters')
            let glowing = |mesh: Mesh| {
                more.iter()
                    .filter(|x| x.mesh == mesh as u32 && x.glow == GLOW)
                    .filter(|x| x.at.dist(visor) < head)
                    .count()
            };
            match kind {
                Kind::Gorilla | Kind::Proboscis => {
                    assert_eq!(glowing(Mesh::Hex), 1, "{kind:?}: a mono-eye")
                }
                Kind::Chimpanzee | Kind::Lemur => {
                    assert_eq!(glowing(Mesh::Hex), 2, "{kind:?}: twin eyes")
                }
                Kind::Orangutan | Kind::Gibbon => {
                    assert_eq!(glowing(Mesh::Cube), 1, "{kind:?}: a visor bar")
                }
            }
            match kind {
                // the V-fin
                Kind::Chimpanzee | Kind::Gibbon => {
                    assert_eq!(fins, 2, "{kind:?}: the V-fin")
                }
                // the antenna ears, and the ringed tail behind
                Kind::Lemur => {
                    assert_eq!(fins, 2, "{kind:?}: the antennae");
                    assert_eq!(n(Mesh::Hex, c.main), 2 + 6, "{kind:?}: tail plates");
                }
                // the cheek flanges: one plate wider than the helmet
                Kind::Orangutan => {
                    let flange = more
                        .iter()
                        .filter(|x| x.mesh == Mesh::Hex as u32 && x.colour == c.second)
                        .map(|x| x.axes[0].len())
                        .max()
                        .unwrap();
                    assert!(flange > head * 3 / 2, "{kind:?}: flange {flange}");
                }
                // the nose: three rings and a ram plate, below the visor
                Kind::Proboscis => {
                    assert_eq!(n(Mesh::Hex, c.second), 3, "{kind:?}: nose rings");
                    let visor = more.iter().find(|x| x.colour == VISOR).unwrap();
                    let ram = more
                        .iter()
                        .find(|x| x.mesh == Mesh::Bevel as u32 && x.colour == c.accent)
                        .unwrap();
                    assert!(ram.at.y < visor.at.y, "{kind:?}: the nose hangs");
                }
                // the knuckle guards over the fists it walks on (with the
                // hip skirt, the helmet and the two boots)
                Kind::Gorilla => {
                    assert_eq!(n(Mesh::Bevel, c.second), 4 + 2, "{kind:?}: knuckles")
                }
            }
            if kind == Kind::Gibbon {
                assert_eq!(n(Mesh::Wedge, c.second), 2 + 1, "{kind:?}: the blade fins");
            }
            let across: Vec<i32> = s.iter().map(|x| x.at.dot(side)).collect();
            widths.push((
                kind,
                across.iter().max().unwrap() - across.iter().min().unwrap(),
            ));
        }
        // the gorilla the widest: the heavy tank
        let gorilla = widths.iter().find(|w| w.0 == Kind::Gorilla).unwrap().1;
        for (kind, w) in &widths {
            assert!(
                *kind == Kind::Gorilla || *w < gorilla,
                "{widths:?}: {kind:?} as wide as the gorilla"
            );
        }
    }

    #[test]
    fn the_visor_and_the_boosters_glow() {
        for kind in Kind::ALL {
            for palette in 0..PALETTES {
                let c = scheme(kind, palette);
                let pose = mech_pose(at_rest());
                let s = drawn_in(kind, palette, &pose);
                let fwd = facing(pose.yaw, 0);
                let middle = s
                    .iter()
                    .fold(V3::ZERO, |a, x| a.add(x.at.scale(ONE / s.len() as i32)));
                let glowing: Vec<&Shape> = s.iter().filter(|x| x.glow == GLOW).collect();
                // only the visor and the boosters glow, in the scheme's glow
                assert!(
                    glowing.iter().all(|x| x.colour == c.glow),
                    "{kind:?} {palette}"
                );
                // the visor's light: ahead of the body's middle, by the top
                let top = s.iter().map(|x| x.at.y).max().unwrap();
                let visor = s.iter().find(|x| x.colour == VISOR).unwrap();
                let eyes: Vec<&&Shape> = glowing
                    .iter()
                    .filter(|x| x.at.dist(visor.at) < cm(12))
                    .collect();
                assert!(!eyes.is_empty(), "{kind:?}: no light in the visor");
                for e in &eyes {
                    assert!(e.at.sub(middle).dot(fwd) > 0, "{kind:?}: visor behind");
                    assert!(top - e.at.y < cm(70), "{kind:?}: visor low");
                }
                // the boosters: two nozzles and two flames, behind
                let behind = glowing
                    .iter()
                    .filter(|x| x.at.sub(middle).dot(fwd) < 0)
                    .count();
                assert!(behind >= 4, "{kind:?}: {behind} booster lights");
                assert_eq!(glowing.len(), eyes.len() + 4, "{kind:?}");
            }
        }
    }

    #[test]
    fn the_schemes_are_vivid_and_apart_from_the_vines_and_the_catch_up_glow() {
        // the Spindle's vines (bright green) and the catch-up magenta
        let (vine, magenta) = (rgb(70, 255, 120), rgb(255, 30, 160));
        assert_eq!(SCHEMES.len(), 4);
        for (i, a) in SCHEMES.iter().enumerate() {
            for paint in [a.main, a.second, a.accent, a.glow, a.fin, HAZARD] {
                assert!(apart(paint, vine) >= 100, "{paint:?} near the vines");
                assert!(apart(paint, magenta) >= 100, "{paint:?} near magenta");
            }
            // the glow vivid, one channel near full
            assert!(a.glow.iter().max().unwrap() * 255 / ONE >= 230, "{a:?}");
            // the frame dark, the plates bright enough to read over it
            assert!(a.frame.iter().max().unwrap() * 255 / ONE <= 64, "{a:?}");
            assert!(a.main.iter().max().unwrap() * 255 / ONE >= 120, "{a:?}");
            assert!(
                apart(a.main, a.frame) >= 90,
                "{a:?}: plates lost on the frame"
            );
            assert!(apart(a.main, a.second) >= 60, "{a:?}: one colour");
            assert!(apart(a.glow, a.main) >= 60, "{a:?}: the glow lost");
            assert!(apart(a.glow, VISOR) >= 200, "{a:?}: the visor's light");
            for b in &SCHEMES[i + 1..] {
                assert!(apart(a.main, b.main) >= 80, "{a:?} near {b:?}");
            }
        }
        // the classic palette shows all four; each kind's four palettes are
        // the four schemes, and past the last it wraps
        let classic: Vec<Scheme> = Kind::ALL.iter().map(|k| scheme(*k, 0)).collect();
        for sc in SCHEMES {
            assert!(classic.contains(&sc), "{sc:?}");
        }
        for kind in Kind::ALL {
            let mine: Vec<Scheme> = (0..PALETTES).map(|p| scheme(kind, p)).collect();
            for (i, a) in mine.iter().enumerate() {
                assert!(!mine[i + 1..].contains(a), "{kind:?}: {a:?} twice");
            }
            assert_eq!(scheme(kind, PALETTES), scheme(kind, 0));
        }
    }

    /// The cost bound (SP-M6): the armour is a plate on every limb
    /// segment and a helmet and boosters, so it costs more than an outfit
    /// of cloth; it is held to half as much again as the kind's cowboy
    /// outfit (well inside the brief's limit of twice).
    #[test]
    fn mech_armour_costs_at_most_half_again_the_cowboy_outfits() {
        let mut lines = Vec::new();
        let mut over = Vec::new();
        for kind in Kind::ALL {
            let p = at_rest();
            let tris = |g: Gear| triangles(&drawn_in(kind, 0, &Pose { gear: g, ..p }));
            let bare = tris(Gear::None);
            let mech = tris(Gear::Mech) - bare;
            let cowboy = tris(Gear::Cowboy) - bare;
            lines.push(format!("{kind:?}: mech +{mech}, cowboy +{cowboy}"));
            if mech * 2 > cowboy * 3 {
                over.push(kind);
            }
        }
        assert!(over.is_empty(), "{}", lines.join("; "));
    }

    /// A flat face of a box-like shape: its middle, its two half-spans and
    /// its outward normal (length `ONE`).
    pub(in crate::models) struct Face {
        mid: V3,
        u: V3,
        v: V3,
        n: V3,
    }

    /// The flat faces of a cube, a bevelled box (its flat middles, a third
    /// of the way out each way) or a wedge (its bottom and back).
    pub(in crate::models) fn faces(x: &Shape) -> Vec<Face> {
        let all = [(0, 1), (0, -1), (1, 1), (1, -1), (2, 1), (2, -1)];
        let (which, span): (&[(usize, i32)], i32) = match x.mesh {
            m if m == Mesh::Cube as u32 => (&all, ONE / 2),
            m if m == Mesh::Bevel as u32 => (&all, ONE * 33 / 100),
            m if m == Mesh::Wedge as u32 => (&[(1, -1), (2, -1)], ONE / 2),
            _ => (&[], 0),
        };
        let ax = x.axes;
        which
            .iter()
            .map(|&(k, s)| {
                let (a, b) = (ax[(k + 1) % 3], ax[(k + 2) % 3]);
                let n = a.norm().cross(b.norm()).norm();
                let n = if n.dot64(ax[k]) * (s as i64) < 0 {
                    n.neg()
                } else {
                    n
                };
                Face {
                    mid: x.at.add(ax[k].scale(s * ONE / 2)),
                    u: a.scale(span),
                    v: b.scale(span),
                    n,
                }
            })
            .collect()
    }

    /// Whether `p` (on `f`'s plane) is inside `f`.
    fn on_face(f: &Face, p: V3) -> bool {
        let d = p.sub(f.mid);
        let (a11, a12, a22) = (
            f.u.dot64(f.u) as i128,
            f.u.dot64(f.v) as i128,
            f.v.dot64(f.v) as i128,
        );
        let (b1, b2) = (d.dot64(f.u) as i128, d.dot64(f.v) as i128);
        let det = a11 * a22 - a12 * a12;
        if det == 0 {
            return false;
        }
        let s = b1 * a22 - b2 * a12;
        let t = a11 * b2 - a12 * b1;
        s.abs() < det.abs() && t.abs() < det.abs()
    }

    /// Whether two faces lie in one plane (within `gap`), face the same
    /// way and overlap: the depth test cannot tell them apart, and the
    /// two colours flicker through each other.
    pub(in crate::models) fn fight(a: &Face, b: &Face, gap: i32) -> bool {
        if a.n.dot(b.n) < ONE * 998 / 1000 || b.mid.sub(a.mid).dot(a.n).abs() > gap {
            return false;
        }
        let steps = [-ONE * 4 / 5, -ONE * 2 / 5, 0, ONE * 2 / 5, ONE * 4 / 5];
        let inside = |x: &Face, y: &Face| {
            steps.iter().any(|&i| {
                steps
                    .iter()
                    .any(|&j| on_face(y, x.mid.add(x.u.scale(i)).add(x.v.scale(j))))
            })
        };
        inside(a, b) || inside(b, a)
    }

    /// The armour's flicker (Sam: "theres a lot of texture glitching on
    /// these outfits"): no plate the armour adds has a face in the plane of
    /// another shape's face, facing the same way and overlapping it, in
    /// another colour.
    #[test]
    fn no_plate_shares_a_face_with_another_colour() {
        let gap = cm(1) / 5;
        let mut bad = Vec::new();
        for kind in Kind::ALL {
            for palette in 0..PALETTES {
                for p in poses() {
                    let bare = drawn_in(
                        kind,
                        palette,
                        &Pose {
                            gear: Gear::None,
                            ..p
                        },
                    );
                    let s = drawn_in(kind, palette, &mech_pose(p));
                    let new: Vec<bool> =
                        s.iter().map(|y| !bare.iter().any(|x| same(x, y))).collect();
                    let f: Vec<Vec<Face>> = s.iter().map(faces).collect();
                    for i in 0..s.len() {
                        for j in i + 1..s.len() {
                            if !(new[i] || new[j])
                                || (s[i].colour == s[j].colour && s[i].glow == s[j].glow)
                            {
                                continue;
                            }
                            if f[i].iter().any(|a| f[j].iter().any(|b| fight(a, b, gap))) {
                                bad.push(format!(
                                    "{kind:?} {palette}: mesh {} at {:?} / mesh {} at {:?}",
                                    s[i].mesh, s[i].at, s[j].mesh, s[j].at
                                ));
                            }
                        }
                    }
                }
            }
        }
        assert!(bad.is_empty(), "{} fights:\n{}", bad.len(), bad.join("\n"));
    }

    #[test]
    fn mech_armour_is_a_costume_unlocked_on_its_map() {
        assert!(Costume::ALL.contains(&Costume::Mech));
        assert_eq!(Costume::from_key("mech"), Some(Costume::Mech));
        assert_eq!(Costume::Mech.key(), "mech");
        assert_eq!(Gear::from_key(Costume::Mech.key()), Some(Gear::Mech));
        assert_eq!(Costume::of_map(Map::Spindle), Some(Costume::Mech));
        assert_eq!(Costume::Mech.map(), Some(Map::Spindle));
        assert!(!Costume::Mech.free());
        for map in Map::ALL {
            assert_eq!(Costume::Mech.gear(map), Gear::Mech);
        }
        // the morph suit is gone
        assert_eq!(Gear::from_key("morph"), None);
        assert_eq!(Costume::from_key("morph"), None);
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
