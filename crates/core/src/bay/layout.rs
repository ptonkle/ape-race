//! Banana Bay Battle's course (2026-10-15): the layout.
//!
//! - **The dock** (0 m): a tall wooden pier 54 m out from the shore, 12 m
//!   over the water; the start row's seven ropes hang from a gantry across its
//!   end (`Hanger::Shore`, the gantry's two posts).
//! - **The fleet** (from about 25 m; the overhaul of 2026-10-15, BO):
//!   `SHIPS` still ships, about 1.35× their first size (a galleon 70 m), in
//!   two battle lines facing each other across a channel: pirates left of
//!   the line, the navy right, alternating, each turned 5–10° off the line.
//!   They pair off, pirate and navy, side by side at least `BESIDE` along
//!   (broadside to broadside, so their cannon fire crosses the channel);
//!   in its own line each ship keeps `SAME_SIDE_GAP` clear of the bowsprit
//!   ahead. Two big ships (galleons, frigates), then two small ones (brigs,
//!   sloops), and so on. Every mast carries ropes: four from its swing yard
//!   (one each side of the mast, one halfway out toward the channel, one
//!   at the yardarm over the channel), plus one low rope per ship from the
//!   foremast's swing yard to 3 m over the deck. The two lines' masts
//!   interleave, so a row of ropes comes every 12–29 m along, alternately
//!   on each side. Sails: none is set (the ships lie at anchor): every one
//!   is furled, torn or shot away, since an ape flies as high as the
//!   royals.
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

/// The open water between a ship's inner side and the course's middle:
/// pirates lie left of the line (−), the navy right (+), their keels
/// `beam / 2 + SHIP_OFFSET` out, so the two battle lines face each other
/// across a channel about twice this wide.
pub const SHIP_OFFSET: i32 = int(11);

/// Half the course's width: the two battle lines, their yardarms and the
/// ropes on them fit inside it; the boundary barrels float on it.
pub const HALF_WIDTH: i32 = int(42);

/// The least water between a ship's stern and the bow (bowsprit counted)
/// of the ship ahead of it in the same line.
const SAME_SIDE_GAP: i32 = int(6);

/// The longest gap along the course between one row of ropes and the next.
const HOP_MOST: i32 = int(26);

/// How far each ship lies side by side with the enemy before it, at least.
const BESIDE: i32 = int(12);

/// How many passes push the ships into place. The rules pull against each
/// other (a pair side by side, the bowsprit ahead, the longest hop), so the
/// passes stop short of a fixed point: three settle every layout tried
/// within a few metres; more creep the fleet forward off the dock.
const RELAX: usize = 3;

/// From one ship's foremast to the next ship's aftmost mast, across the
/// channel (metres, least and most).
const HALF_HOP: (i32, i32) = (11, 15);

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
    /// its keel's turn off the course's heading (`TURN` units)
    turn: i32,
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
        ShipKind::Galleon => (70, 19, 9, &[(-35, -16, 4), (22, 35, 3)], 15),
        ShipKind::Frigate => (60, 16, 8, &[(-30, -14, 3)], 13),
        ShipKind::Brig => (44, 12, 7, &[], 10),
        ShipKind::Sloop => (35, 11, 5, &[], 10),
    }
}

/// The masts of each kind, fore first (every mast carries ropes now the
/// ships are big). On each mast the swing ropes hang from the lowest yard
/// at least `SWING_YARD` over the deck, the low rope from the yard below
/// that (or the same yard on a sloop).
fn masts(kind: ShipKind) -> Vec<MastSpec> {
    const G_TALL: &[(i32, i32)] = &[(19, 19), (35, 21), (51, 22), (63, 12)];
    const G_MIZ: &[(i32, i32)] = &[(19, 14), (34, 19), (49, 21)];
    const F_TALL: &[(i32, i32)] = &[(18, 16), (33, 19), (48, 21), (59, 11)];
    const F_MIZ: &[(i32, i32)] = &[(18, 12), (32, 17), (47, 20)];
    const B_TALL: &[(i32, i32)] = &[(16, 13), (31, 17), (48, 20), (58, 9)];
    const S_TALL: &[(i32, i32)] = &[(31, 15), (46, 15)];
    let m = |along, height, yards, ropes| MastSpec {
        along: int(along),
        height: int(height),
        yards,
        ropes,
    };
    match kind {
        ShipKind::Galleon => vec![
            m(23, 70, G_TALL, true),
            m(0, 74, G_TALL, true),
            m(-23, 57, G_MIZ, true),
        ],
        ShipKind::Frigate => vec![
            m(20, 65, F_TALL, true),
            m(1, 69, F_TALL, true),
            m(-19, 54, F_MIZ, true),
        ],
        ShipKind::Brig => vec![m(12, 62, B_TALL, true), m(-9, 63, B_TALL, true)],
        ShipKind::Sloop => vec![m(3, 59, S_TALL, true)],
    }
}

/// The swing ropes hang from the lowest yard at least this high over the
/// deck, so their bottoms sit about as high over the water as before the
/// ships grew.
const SWING_YARD: i32 = int(28);

/// The yard a mast's ropes hang from (the low rope's too: from a lower
/// yard it was too short to swing up to the next row).
fn swing_yard(yards: &[(i32, i32)]) -> usize {
    yards
        .iter()
        .position(|&(h, _)| int(h) >= SWING_YARD)
        .unwrap_or(yards.len() - 1)
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

    // two battle lines, along the course (1-D): pirates left, the navy
    // right, alternating. A first guess puts each ship's aftmost mast half
    // a hop past the last ship's foremast, across the channel; then the
    // places are pushed forward until (1) in its own line each ship's
    // stern keeps `SAME_SIDE_GAP` clear of the bowsprit ahead, (2) the
    // ships pair off, pirate and navy, side by side for at least `BESIDE`
    // (broadside to broadside), and (3) no gap between rows of ropes is
    // longer than `HOP_MOST`.
    let kinds = kinds(&mut dice);
    let len = |k: ShipKind| int(hull(k).0);
    let sprit = |k: ShipKind| int(hull(k).4);
    let mut plans: Vec<Plan> = Vec::new();
    for (k, &kind) in kinds.iter().enumerate() {
        let (lo, _) = roped_span(kind);
        let first = match plans.last() {
            None => int(34),
            Some(prev) => {
                let (_, phi) = roped_span(prev.kind);
                prev.mid + phi + int(dice.range(HALF_HOP.0, HALF_HOP.1))
            }
        };
        // each ship turned a little off the line (5–10°, either way), so
        // the lines read as ships fighting, not a convoy
        let turn = TURN / 360 * dice.range(5, 10) * if dice.below(2) == 0 { -1 } else { 1 };
        plans.push(Plan {
            kind,
            side: if k % 2 == 0 { Side::Pirate } else { Side::Navy },
            mid: first - lo,
            turn,
        });
    }
    for pass in 0..=RELAX {
        // the last pass keeps the hard rules only (order, the line, the
        // pairs); it never breaks one it has already met
        let last = pass == RELAX;
        let mut moved = false;
        let mut push = |plans: &mut Vec<Plan>, k: usize, least: i32| {
            if plans[k].mid < least {
                plans[k].mid = least;
                moved = true;
            }
        };
        for k in 1..plans.len() {
            let (kind, prev) = (plans[k].kind, plans[k - 1].kind);
            // in course order
            let least = plans[k - 1].mid + int(1);
            push(&mut plans, k, least);
            // (1) its own line
            if k >= 2 {
                let ahead = plans[k - 2].kind;
                let least = plans[k - 2].mid
                    + len(ahead) / 2
                    + sprit(ahead)
                    + SAME_SIDE_GAP
                    + len(kind) / 2;
                push(&mut plans, k, least);
            }
            // (2) the pair side by side
            if k % 2 == 1 && k > 1 {
                let least = plans[k].mid - len(kind) / 2 + BESIDE - len(prev) / 2;
                push(&mut plans, k - 1, least);
            }
            // (3) no long gap: bring the ship before forward
            let first = plans[k].mid + roped_span(kind).0;
            let most = plans[..k]
                .iter()
                .map(|p| p.mid + roped_span(p.kind).1)
                .max()
                .unwrap_or(first);
            if first - most > HOP_MOST && k > 1 && !last {
                let least = first - HOP_MOST - roped_span(prev).1;
                push(&mut plans, k - 1, least);
            }
        }
        if !moved {
            break;
        }
    }
    let last_bow = plans
        .iter()
        .map(|p| p.mid + len(p.kind) / 2)
        .max()
        .unwrap_or(DOCK_END);
    let last_station = plans
        .iter()
        .map(|p| p.mid + roped_span(p.kind).1)
        .max()
        .unwrap_or(int(9));
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
    c.half_width = HALF_WIDTH;
    let length = c.length;

    let mut put = Placing {
        vines: Vec::new(),
        hangers: Vec::new(),
    };
    let mut stations: Vec<(i32, Vec<usize>)> = Vec::new();

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
            .add(right_of(h).scale(side_sign * (beam / 2 + SHIP_OFFSET)));
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
                    a += int(5);
                }
            }
        }
        let ship = Ship {
            kind: p.kind,
            side: p.side,
            at: v3(mid.x, WATER_LEVEL, mid.z),
            yaw: yaw_of(h) + p.turn,
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

        // the ropes: a station at each roped mast, four ropes across its
        // swing yard (one each side of the mast, one at each yardarm);
        // the low rope from the foremast over the deck
        for (m, spec) in specs.iter().enumerate() {
            if !spec.ropes {
                continue;
            }
            let sy = swing_yard(spec.yards);
            let yard = ship.masts[m].yards[sy];
            let mut here = Vec::new();
            // across the ship from its outer side to the channel: one
            // each side of the mast, one halfway out, one at the yardarm
            // over the channel (none at the outer yardarm: an ape out
            // there is far off the line)
            let c_in = -side_sign;
            let inner = OFF_MAST + int(2) + cm(dice.range(-50, 50));
            let arm = yard.half_span - int(2) - cm(dice.range(0, 100));
            let half = (inner + arm) / 2;
            let channel = if m == 0 { inner + int(3) } else { inner };
            for out in [-inner, channel, half, arm].map(|o| o * c_in) {
                let hanger = Hanger::Yard {
                    ship: k,
                    mast: m,
                    yard: sy,
                    out,
                };
                let anchor = ship.point(spec.along, out, deck + yard.height);
                // bottoms 15–22 m over the water, and never lower than 6 m
                // over the deck
                let longest = yard.height - int(6);
                let length = int(dice.range(23, 31)).min(longest);
                let at = c.progress(anchor, c.nearest_anywhere(anchor)).0;
                here.push(put.push(anchor, length, at, hanger));
            }
            let at = put.vines[here[0]].at;
            stations.push((at, here));
            if m == 0 {
                let at = c
                    .progress(ship.point(spec.along, 0, 0), c.nearest_anywhere(ship.at))
                    .0;
                // the low rope: 3 m over the deck, toward the course's
                // middle, 3 m off the mast
                // (from the swing yard: a rope from the low yard would be
                // too short to swing up to the next row; KK whirled on one)
                let out = -side_sign * OFF_MAST;
                let anchor = ship.point(spec.along, out, deck + yard.height);
                put.push(
                    anchor,
                    yard.height - int(3),
                    at,
                    Hanger::Yard {
                        ship: k,
                        mast: m,
                        yard: sy,
                        out,
                    },
                );
            }
        }

        // a bouncy barrel on its open side, toward the course's middle
        // (in the channel, off its inner side)
        let lateral = side_sign * (SHIP_OFFSET - BARREL_RADIUS - int(2));
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
    stations.push((palms_at, here));
    let chest = c.path[last].add(lh.scale(int(28)));
    let island = Island {
        centre: v3(centre.x, ISLAND_TOP, centre.z),
        radius,
        chest: v3(chest.x, ISLAND_TOP, chest.z),
        chest_yaw: yaw_of(lh.neg()),
        chest_size: v3(int(16), int(8), int(10)),
    };

    c.vines = put.vines;
    // the two lines' stations interleave: in course order, and every
    // other one counted, as many arcs as one line of stations gave
    stations.sort_by_key(|s| s.0);
    let stations: Vec<Vec<usize>> = stations
        .into_iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .map(|(_, s)| s.1)
        .collect();
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
