//! Banana Bay Battle's course (2026-10-15): the first real layout.
//!
//! - **The dock** (0 m): a tall wooden pier 54 m out from the shore, 12 m
//!   over the water; the start row's seven ropes hang from a gantry across its
//!   end (`Hanger::Shore`, the gantry's two posts).
//! - **The fleet** (from about 25 m): `SHIPS` still ships in one line
//!   through the bay, bow to stern along the course, pirates a little left
//!   of the line and the navy a little right, alternating, so each pair
//!   duels broadside to broadside. Two big ships (galleons, frigates), then
//!   two small ones (brigs, sloops) that fill the gap, and so on. The swing stations
//!   are the masts: every mast that carries ropes hangs two or three from
//!   its swing yard across the course (lanes 9 m apart, never on the mast
//!   itself), every 12–28 m along; one low rope per ship over its deck, 3 m
//!   above it, for a grounded ape to jump to. A galleon's or frigate's
//!   mainmast carries none. Sails: none is set (the ships lie at anchor):
//!   every one is furled, torn or shot away, since an ape flies as high as
//!   the royals.
//! - **The island** (the finish): its beach starts just past the last
//!   ship's bow; a row of palms over the beach, then the finish, then the
//!   treasure chest on past it.
//!
//! New placement draws from `Dice(seed ^ 0xBA7_5EA)`.

use super::*;
use crate::course::{banana_arcs, centreline, Course, Leg, Mushroom, Platform, Vine};
use crate::fx::Dice;
use crate::geom::{right_of, yaw_of};
use crate::rules::BODY_DROP;
use crate::ski::Terrain;

const SEED: u64 = 0xBA7_BA77_0001;

/// How many ships the line has.
pub const SHIPS: usize = 30;

/// The still water's level (the floor).
pub const WATER_LEVEL: i32 = 0;

/// The dock's deck over the water, and how far it runs back from the
/// course's start toward the shore, and on past it.
pub const DOCK_TOP: i32 = int(12);
const DOCK_BACK: i32 = int(40);
const DOCK_END: i32 = int(14);

/// The start row's ropes, as the Jungle's: from a gantry 30 m over the
/// dock, to a metre over it. (The dock stands 12 m up so that their bottoms
/// are level with the first ship's: from a 5 m dock the weaker AI levels
/// swung to and fro on the start row for seconds.)
const START_ROPE: i32 = int(29);

/// The island's beach top over the water.
pub const ISLAND_TOP: i32 = int(2);

/// How far left (pirates, −) or right (navy, +) of the line a ship's keel
/// lies.
pub const SHIP_OFFSET: i32 = int(5);

/// A bouncy barrel's top over the still water, and its radius.
pub const BARREL_TOP: i32 = cm(130);
pub const BARREL_RADIUS: i32 = cm(220);

/// A rope never hangs closer than this to its mast (across the ship).
const OFF_MAST: i32 = int(3);

/// A reset puts an ape on its ship's main deck this far aft of the
/// foremast (its low rope hangs from the foremast).
pub const RESET_AFT: i32 = int(8);

/// The legs' bends; the last straight is as long as the line needs.
const BENDS: &[Leg] = &[
    Leg(90, 0),
    Leg(220, 24),
    Leg(160, 0),
    Leg(260, -40),
    Leg(140, 0),
    Leg(220, 28),
];

/// Banana Bay Battle's course, from its fixed seed.
pub fn bay() -> Course {
    bay_seeded(SEED)
}

/// One ship's plan along the course (metres along, 1-D), before the
/// centreline exists.
struct Plan {
    kind: ShipKind,
    side: Side,
    /// the hull's middle, metres along (fixed point)
    mid: i32,
}

/// A mast's place along the ship, its height over the deck, its yards
/// (height over the deck, half span), and whether ropes hang from it.
struct MastSpec {
    along: i32,
    height: i32,
    yards: &'static [(i32, i32)],
    ropes: bool,
}

/// A kind's hull, in metres.
type Hull = (i32, i32, i32, &'static [(i32, i32, i32)], i32);

/// Length, beam, deck height, raised decks (from, to, height over the
/// deck) and bowsprit of each kind, in metres.
fn hull(kind: ShipKind) -> Hull {
    match kind {
        ShipKind::Galleon => (52, 14, 7, &[(-26, -12, 3), (16, 26, 2)], 14),
        ShipKind::Frigate => (44, 12, 6, &[(-22, -10, 2)], 12),
        ShipKind::Brig => (32, 9, 5, &[], 9),
        ShipKind::Sloop => (26, 8, 4, &[], 10),
    }
}

/// The masts of each kind, fore first. On each roped mast the swing ropes
/// hang from the second-highest yard of four (the highest of three), the
/// low rope from the yard below that.
fn masts(kind: ShipKind) -> Vec<MastSpec> {
    const G_TALL: &[(i32, i32)] = &[(14, 14), (26, 15), (38, 16), (47, 9)];
    const G_MIZ: &[(i32, i32)] = &[(14, 10), (25, 14), (36, 16)];
    const F_TALL: &[(i32, i32)] = &[(13, 12), (24, 14), (36, 16), (44, 8)];
    const F_MIZ: &[(i32, i32)] = &[(13, 9), (24, 13), (35, 16)];
    const B_TALL: &[(i32, i32)] = &[(12, 10), (23, 13), (36, 16), (43, 7)];
    const S_TALL: &[(i32, i32)] = &[(25, 10), (37, 11)];
    let m = |along, height, yards, ropes| MastSpec {
        along: int(along),
        height: int(height),
        yards,
        ropes,
    };
    match kind {
        ShipKind::Galleon => vec![
            m(17, 52, G_TALL, true),
            m(0, 55, G_TALL, false),
            m(-17, 42, G_MIZ, true),
        ],
        ShipKind::Frigate => vec![
            m(15, 48, F_TALL, true),
            m(1, 51, F_TALL, false),
            m(-14, 40, F_MIZ, true),
        ],
        ShipKind::Brig => vec![m(9, 46, B_TALL, true), m(-6, 47, B_TALL, true)],
        ShipKind::Sloop => vec![m(2, 44, S_TALL, true)],
    }
}

/// The yard a roped mast's swing ropes hang from, and its low rope's.
fn swing_yard(yards: &[(i32, i32)]) -> (usize, usize) {
    let n = yards.len();
    if n >= 4 {
        (n - 2, n - 3)
    } else if n == 3 {
        (2, 1)
    } else {
        (n - 1, 0)
    }
}

/// The aft-most and fore-most roped masts' places along the ship.
fn roped_span(kind: ShipKind) -> (i32, i32) {
    let ms = masts(kind);
    let roped = ms.iter().filter(|m| m.ropes).map(|m| m.along);
    let lo = roped.clone().min().unwrap_or(0);
    let hi = roped.max().unwrap_or(0);
    (lo, hi)
}

/// Where a vine hangs and what from, as it is placed.
struct Placing {
    vines: Vec<Vine>,
    hangers: Vec<Hanger>,
}

impl Placing {
    fn push(&mut self, anchor: V3, length: i32, at: i32, hanger: Hanger) -> usize {
        self.vines.push(Vine {
            anchor,
            length,
            slide: None,
            at,
        });
        self.hangers.push(hanger);
        self.vines.len() - 1
    }
}

/// The kinds of the line, start first: two big ships (a pirate and a
/// navy ship dueling), then two small ones filling the gap, and so on.
fn kinds(dice: &mut Dice) -> Vec<ShipKind> {
    (0..SHIPS)
        .map(|k| {
            let big = k % 4 < 2;
            match (big, dice.below(5)) {
                (true, 0..=1) => ShipKind::Galleon,
                (true, _) => ShipKind::Frigate,
                (false, 0..=2) => ShipKind::Brig,
                (false, _) => ShipKind::Sloop,
            }
        })
        .collect()
}

/// Banana Bay Battle's course from `seed`.
pub fn bay_seeded(seed: u64) -> Course {
    let mut dice = Dice(seed ^ 0xBA7_5EA);

    // the line, along the course (1-D): each ship's first roped mast a
    // station's hop past the last one's, its stern at least 3 m past the
    // last bow
    let kinds = kinds(&mut dice);
    let mut plans = Vec::new();
    let mut last_station = int(9);
    let mut last_bow = DOCK_END + int(6);
    for (k, &kind) in kinds.iter().enumerate() {
        let (len, ..) = hull(kind);
        let (lo, hi) = roped_span(kind);
        let hop = if k == 0 {
            int(25)
        } else {
            int(dice.range(23, 28))
        };
        let first = (last_station + hop).max(last_bow + int(3) + int(len) / 2 + lo);
        let mid = first - lo;
        plans.push(Plan {
            kind,
            side: if k % 2 == 0 { Side::Pirate } else { Side::Navy },
            mid,
        });
        last_station = mid + hi;
        last_bow = mid + int(len) / 2;
    }
    // the island: its beach from 4 m past the last bow; a row of palms
    // over it a hop past the last station, the finish 22 m on
    let beach = last_bow + int(4);
    let palms_at = (last_station + int(24)).max(beach + int(8));
    let length = palms_at + int(22);

    let mut legs: Vec<Leg> = BENDS.iter().map(|l| Leg(l.0, l.1)).collect();
    let used: i32 = BENDS.iter().map(|l| l.0).sum();
    legs.push(Leg((length / ONE - used).max(40), 0));
    let mut c = centreline("bay", &legs);
    c.terrain = Terrain::Flat(WATER_LEVEL);
    let length = c.length;

    let mut put = Placing {
        vines: Vec::new(),
        hangers: Vec::new(),
    };
    let mut stations: Vec<Vec<usize>> = Vec::new();

    // the dock, the start platform; the starters stand across its end
    let fwd = c.heading_at(0);
    let right = right_of(fwd);
    let o = c.point(0);
    let back = o.sub(fwd.scale(DOCK_BACK));
    let end = o.add(fwd.scale(DOCK_END));
    let dock_mid = back.add(end).scale(ONE / 2);
    c.platforms.push(Platform {
        centre: v3(dock_mid.x, DOCK_TOP, dock_mid.z),
        fwd,
        half_len: (DOCK_BACK + DOCK_END) / 2,
        half_width: int(9),
        top: DOCK_TOP,
    });
    for k in 0..4 {
        let at = c.point(int(2)).add(right.scale(int(-6) + k * int(4)));
        c.starts[k as usize] = v3(at.x, DOCK_TOP + BODY_DROP, at.z);
    }
    c.start_yaw = 0;
    // the start row: seven ropes from the gantry across the dock's end,
    // hanging to a metre over the deck
    let row_at = DOCK_END + cm(150);
    for k in -3..=3 {
        let base = c.point(row_at).add(right.scale(int(3) * k));
        let side = if k < 0 { -1 } else { 1 };
        let post = c.point(row_at).add(right.scale(int(12) * side));
        put.push(
            v3(base.x, DOCK_TOP + START_ROPE + int(1), base.z),
            START_ROPE,
            row_at,
            Hanger::Shore {
                foot: v3(post.x, WATER_LEVEL, post.z),
            },
        );
    }

    // the ships
    let mut ships = Vec::new();
    let mut fighters = Vec::new();
    let mut bouncy = Vec::new();
    for (k, p) in plans.iter().enumerate() {
        let (len, beam, deck, raised, sprit) = hull(p.kind);
        let (len, beam, deck) = (int(len), int(beam), int(deck));
        let h = c.heading_at(p.mid);
        let side_sign = match p.side {
            Side::Pirate => -1,
            Side::Navy => 1,
        };
        let mid = c
            .point(p.mid)
            .add(right_of(h).scale(side_sign * SHIP_OFFSET));
        let specs = masts(p.kind);
        let mut ship_masts = Vec::new();
        for spec in &specs {
            let yards = spec
                .yards
                .iter()
                .map(|&(height, half)| Yard {
                    height: int(height),
                    half_span: int(half),
                    // none set: an ape flies as high as the royals (a
                    // set royal on a bare mainmast was touched, 44 m up)
                    sail: match dice.below(10) {
                        0..=4 => Sail::Furled,
                        5..=7 => Sail::Torn,
                        _ => Sail::Missing,
                    },
                })
                .collect();
            ship_masts.push(Mast {
                along: spec.along,
                height: spec.height,
                yards,
                gaff: None,
            });
        }
        // the fore-and-aft sails: furled or shot through, never set
        let clear = |d: &mut Dice| match d.below(3) {
            0 => Sail::Furled,
            1 => Sail::Torn,
            _ => Sail::Missing,
        };
        let aft = ship_masts.len() - 1;
        ship_masts[aft].gaff = Some(clear(&mut dice));
        let jib = Some(clear(&mut dice));
        // the gun ports, both sides; a galleon has two tiers
        let mut ports = Vec::new();
        let tiers: &[i32] = if p.kind == ShipKind::Galleon {
            &[cm(160), cm(360)]
        } else {
            &[cm(150)]
        };
        for side in [-1, 1] {
            for &down in tiers {
                let mut a = -len / 2 + int(6);
                while a <= len / 2 - int(6) {
                    ports.push(Port {
                        along: a,
                        side,
                        up: deck - down,
                    });
                    a += int(4);
                }
            }
        }
        let ship = Ship {
            kind: p.kind,
            side: p.side,
            at: v3(mid.x, WATER_LEVEL, mid.z),
            yaw: yaw_of(h),
            length: len,
            beam,
            deck,
            raised: raised
                .iter()
                .map(|&(f, t, up)| (int(f), int(t), deck + int(up)))
                .collect(),
            bowsprit: int(sprit),
            masts: ship_masts,
            jib,
            ports,
            decks: Vec::new(),
        };
        // its decks: the main deck, then the raised ones
        let mut decks = Vec::new();
        c.platforms.push(Platform {
            centre: ship.point(0, 0, deck),
            fwd: ship.fwd(),
            half_len: len / 2 - int(1),
            half_width: beam / 2 - cm(50),
            top: deck,
        });
        decks.push(c.platforms.len() - 1);
        for &(f, t, top) in &ship.raised {
            c.platforms.push(Platform {
                centre: ship.point((f + t) / 2, 0, top),
                fwd: ship.fwd(),
                half_len: (t - f) / 2 - cm(50),
                half_width: beam / 2 - cm(80),
                top,
            });
            decks.push(c.platforms.len() - 1);
        }
        let ship = Ship { decks, ..ship };

        // the ropes: a station at each roped mast, lanes 9 m apart across
        // the course; the low rope from the foremast over the deck
        for (m, spec) in specs.iter().enumerate() {
            if !spec.ropes {
                continue;
            }
            let (sy, ly) = swing_yard(spec.yards);
            let yard = ship.masts[m].yards[sy];
            let at = p.mid + spec.along;
            let skip = if dice.below(4) == 0 {
                3
            } else {
                2 * dice.below(2) as i32
            };
            let mut here = Vec::new();
            for lane in 0..3 {
                if lane == skip {
                    continue;
                }
                let course_off = int((lane - 1) * 9 + dice.range(-1, 1));
                let mut out = course_off - side_sign * SHIP_OFFSET;
                if out.abs() < OFF_MAST {
                    out = if out < 0 { -OFF_MAST } else { OFF_MAST };
                }
                if out.abs() > yard.half_span - cm(50) {
                    continue;
                }
                let hanger = Hanger::Yard {
                    ship: k,
                    mast: m,
                    yard: sy,
                    out,
                };
                let anchor = ship.point(spec.along, out, deck + yard.height);
                // bottoms 10–22 m over the water, as the Jungle's over its
                // floor, and never lower than 6 m over the deck
                let longest = yard.height - int(6);
                let length = int(dice.range(23, 31)).min(longest);
                here.push(put.push(anchor, length, at, hanger));
            }
            stations.push(here);
            if m == 0 {
                // the low rope: 3 m over the deck, toward the course's
                // middle, 3 m off the mast
                let low = ship.masts[m].yards[ly];
                let out = -side_sign * OFF_MAST;
                let anchor = ship.point(spec.along, out, deck + low.height);
                put.push(
                    anchor,
                    low.height - int(3),
                    at,
                    Hanger::Yard {
                        ship: k,
                        mast: m,
                        yard: ly,
                        out,
                    },
                );
            }
        }

        // a bouncy barrel on its open side, toward the course's middle
        let edge = -side_sign * (beam / 2 - SHIP_OFFSET);
        let lateral = edge - side_sign * (BARREL_RADIUS + int(2));
        let along = p.mid + int(dice.range(-8, 8));
        let bp = c
            .point(along)
            .add(right_of(c.heading_at(along)).scale(lateral));
        c.mushrooms.push(Mushroom {
            top: v3(bp.x, WATER_LEVEL + BARREL_TOP, bp.z),
            radius: BARREL_RADIUS,
        });
        bouncy.push(BouncyBarrel {
            bouncer: c.mushrooms.len() - 1,
            yaw: dice.range(0, TURN - 1),
        });

        // the boarding fight on its main deck: pirates against sailors in
        // pairs
        let pairs = match p.kind {
            ShipKind::Galleon => 3,
            ShipKind::Frigate => 3,
            ShipKind::Brig => 2,
            ShipKind::Sloop => 1,
        };
        for _ in 0..pairs {
            let a = int(dice.range(-(len / ONE) / 2 + 12, len / ONE / 2 - 12));
            let x = cm(dice.range(-(beam / ONE - 3) * 50, (beam / ONE - 3) * 50));
            let me = ship.point(a, x, deck);
            let foe = ship.point(a + cm(160), x, deck);
            let phase = dice.below(240);
            let n = fighters.len();
            fighters.push(Fighter {
                ship: k,
                at: me,
                yaw: yaw_of(foe.sub(me)),
                side: Side::Pirate,
                foe: Some(n + 1),
                phase,
            });
            fighters.push(Fighter {
                ship: k,
                at: foe,
                yaw: yaw_of(me.sub(foe)),
                side: Side::Navy,
                foe: Some(n),
                phase: phase + 37,
            });
        }
        ships.push(ship);
    }

    // the cannon fire: each neighbouring pair trades shots broadside to
    // broadside, two or three each way
    let mut volleys = Vec::new();
    for k in 0..ships.len().saturating_sub(1) {
        for (from, at) in [(k, k + 1), (k + 1, k)] {
            let s = &ships[from];
            let t = &ships[at];
            // the side facing the other ship
            let to = t.at.sub(s.at);
            let face = if to.dot(s.right()) >= 0 { 1 } else { -1 };
            let ahead = to.dot(s.fwd()) >= 0;
            let facing: Vec<usize> = s
                .ports
                .iter()
                .enumerate()
                .filter(|(_, p)| p.side == face && (p.along >= 0) == ahead)
                .map(|(i, _)| i)
                .collect();
            if facing.is_empty() {
                continue;
            }
            let back = t.at.sub(s.at);
            let t_face = if back.dot(t.right()) >= 0 { -1 } else { 1 };
            for _ in 0..2 + dice.below(2) {
                let port = facing[dice.below(facing.len() as u32) as usize];
                let hit = dice.below(3) != 0;
                let along = int(dice.range(-(t.length / ONE) / 3, t.length / ONE / 3));
                let target = if hit {
                    t.point(along, t_face * t.beam / 2, t.deck - int(2))
                } else {
                    t.point(along, t_face * (t.beam / 2 + int(dice.range(3, 8))), 0)
                };
                volleys.push(Volley {
                    from,
                    port,
                    at,
                    target,
                    hit,
                    fire: dice.below(VOLLEY_CYCLE),
                    flight: 90 + dice.below(70),
                });
            }
        }
    }

    // the island: a beach from just past the last bow on past the finish
    // (a finished ape coasts on); a row of palms over the beach
    let last = c.path.len() - 1;
    let lh = c.heading[last];
    let reach = crate::course::OUTRO;
    let centre = c.path[last].add(lh.scale(int(50)));
    let radius = length + int(50) - beach;
    let from = c.point(beach);
    let to = c.path[last].add(lh.scale(reach));
    let mid = from.add(to).scale(ONE / 2);
    c.platforms.push(Platform {
        centre: v3(mid.x, ISLAND_TOP, mid.z),
        fwd: lh,
        half_len: to.sub(from).dot(lh) / 2,
        half_width: int(42),
        top: ISLAND_TOP,
    });
    let mut here = Vec::new();
    {
        let r = right_of(c.heading_at(palms_at));
        for lane in -1..=1 {
            let base = c
                .point(palms_at)
                .add(r.scale(int(lane * 9 + dice.range(-1, 1))));
            let lean = if lane < 0 { -1 } else { 1 };
            let foot = base.add(r.scale(int(4) * lean));
            here.push(put.push(
                v3(base.x, ISLAND_TOP + int(32), base.z),
                int(dice.range(23, 26)),
                palms_at,
                Hanger::Shore {
                    foot: v3(foot.x, ISLAND_TOP, foot.z),
                },
            ));
        }
    }
    stations.push(here);
    let chest = c.path[last].add(lh.scale(int(28)));
    let island = Island {
        centre: v3(centre.x, ISLAND_TOP, centre.z),
        radius,
        chest: v3(chest.x, ISLAND_TOP, chest.z),
        chest_yaw: yaw_of(lh.neg()),
        chest_size: v3(int(6), int(4), int(4)),
    };

    c.vines = put.vines;
    banana_arcs(&mut c, &stations);

    // item crates: rows of three across the course, every 240 m
    let mut at = 150;
    while at < length / ONE - 60 {
        let h = c.heading_at(int(at));
        let r = right_of(h);
        let base = c.point(int(at));
        for off in [-9, 0, 9] {
            let p = base.add(r.scale(int(off)));
            c.crates
                .push(v3(p.x, WATER_LEVEL + int(dice.range(16, 24)), p.z));
        }
        at += 240;
    }

    // the boundary barrels: both edges, every 10 m, from beside the dock
    // to the beach
    let mut boundary = Vec::new();
    let mut s = -int(10);
    while s < beach {
        let p = c.point(s.max(0)).add(fwd.scale(s.min(0)));
        let r = right_of(c.heading_at(s.max(0)));
        for edge in [-1, 1] {
            let q = p.add(r.scale(edge * c.half_width));
            boundary.push(BoundaryBarrel {
                at: v3(q.x, WATER_LEVEL, q.z),
                edge,
            });
        }
        s += int(10);
    }

    let water = Water {
        level: WATER_LEVEL,
        waves: vec![
            Wave {
                amp: cm(22),
                dir: v3(fx::sin(9000), 0, fx::cos(9000)),
                length: int(38),
                period: 840,
            },
            Wave {
                amp: cm(12),
                dir: v3(fx::sin(-14000), 0, fx::cos(-14000)),
                length: int(17),
                period: 516,
            },
            Wave {
                amp: cm(6),
                dir: v3(fx::sin(30000), 0, fx::cos(30000)),
                length: int(8),
                period: 348,
            },
        ],
    };
    let lobby_at = o.sub(fwd.scale(int(28)));
    c.bay = Some(Box::new(BayLayout {
        ships,
        hangers: put.hangers,
        boundary,
        bouncy,
        water,
        dock: Dock {
            from: v3(back.x, DOCK_TOP, back.z),
            to: v3(end.x, DOCK_TOP, end.z),
            half_width: int(9),
            platform: 0,
        },
        island,
        volleys,
        fighters,
        lobby: (
            v3(lobby_at.x, DOCK_TOP + BODY_DROP, lobby_at.z),
            yaw_of(fwd),
        ),
    }));
    c
}
