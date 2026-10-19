//! Standard checks every map gets for free (`Map::ALL`): found by hand
//! after Rattlesnake Ravine shipped (TRIAGE T108, T110), so a future map
//! cannot ship the same bugs.
//!
//! 1. The start: every start slot's feet rest on a solid top, and the
//!    drawing has a top there too (the platform is drawn and solid).
//! 2. The finish: an ape crossing the finish line anywhere it can
//!    physically be (across the open width, at any reachable height)
//!    finishes, not only one on the narrow path.
//! 3. Drawn walls are solid: no generic form. Only some maps keep wall or
//!    solid lists (Rattlesnake Ravine's `walls`/`Solids`, the tower map's
//!    stone, Sunset Ruins' facade, Banana Bay's chest), each its own type,
//!    and the drawing is unlabelled shapes in the scenery, so nothing
//!    common says which drawn shape is a wall. Each map's own tests cover
//!    it (`tower/audit.rs`, the ravine's solids tests).
//!
//! Each map has its own test of each, named `<map>_<check>`, so
//! tests/tags.toml tags each with its own map. A map that fails a check
//! today is listed in `KNOWN` with its TRIAGE row: its test then asserts
//! that it still fails (so a fix shows up and the row is taken out), and
//! the test passes on main.

use tool_core::course::Course;
use tool_core::fx::{int, ONE};
use tool_core::geom::{right_of, v3, V3};
use tool_core::map::Map;
use tool_core::mesh::{vertices, Mesh, VERTEX_VALUES};
use tool_core::render::{GROUND, INSTANCE_VALUES};
use tool_core::rules::{BODY_DROP, BODY_RADIUS};
use tool_core::scene::{scenery, Scenery};

/// Checks a map fails today, with the TRIAGE row that tracks the bug.
/// Each still fails here, so its test passes; once fixed the test fails
/// and says to take the row out.
const KNOWN: &[(Map, Check, &str)] = &[
    // the start's platform is solid at 16 m but nothing is drawn there
    // (the canyon floor's ground is the nearest drawn top)
    (Map::Ravine, Check::Start, "TRIAGE T108"),
    // wide left past the line, on the open mesa, the nearest centreline
    // sample is one before the last corner: no finish there
    (Map::Ravine, Check::Finish, "TRIAGE T110"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Check {
    Start,
    Finish,
}

fn known(map: Map, check: Check) -> Option<&'static str> {
    KNOWN
        .iter()
        .find(|(m, c, _)| *m == map && *c == check)
        .map(|k| k.2)
}

/// Passes when `problems` is empty, or the check is a known failure and
/// still fails.
fn verdict(map: Map, check: Check, problems: Vec<String>) {
    match known(map, check) {
        None => assert!(
            problems.is_empty(),
            "{map:?} {check:?}: {} problems, first {:?}",
            problems.len(),
            &problems[..problems.len().min(6)]
        ),
        Some(row) => assert!(
            !problems.is_empty(),
            "{map:?} {check:?} passes now: take {row} out of KNOWN"
        ),
    }
}

fn m(x: i32) -> f64 {
    x as f64 / ONE as f64
}

// ------------------------------------------------------------------ start

/// How far a start slot's feet may be off the top they stand on.
const FEET_SLACK: i32 = ONE / 5;
/// How far the drawn top may be from the solid top under the feet.
const DRAWN_SLACK: i32 = ONE / 2;
/// How far round a slot the feet reach (any drawn top in it counts).
const FEET_PATCH: i32 = ONE / 4;

/// The drawn surfaces' heights straight under/over (`x`, `z`), from the
/// map's still scenery (shapes and ground): every triangle facing up that
/// a vertical line there passes through.
fn drawn_tops(s: &Scenery, x: i32, z: i32) -> Vec<f64> {
    let (px, pz) = (m(x), m(z));
    let mut tops = Vec::new();
    let mut tri = |a: [f64; 3], b: [f64; 3], cc: [f64; 3]| {
        // facing up (counter-clockwise from above, seen from +y)
        let (ux, uz) = (b[0] - a[0], b[2] - a[2]);
        let (vx, vz) = (cc[0] - a[0], cc[2] - a[2]);
        let den = ux * vz - uz * vx;
        if den.abs() < 1e-9 {
            return;
        }
        let (wx, wz) = (px - a[0], pz - a[2]);
        let s1 = (wx * vz - wz * vx) / den;
        let s2 = (ux * wz - uz * wx) / den;
        if s1 < -1e-6 || s2 < -1e-6 || s1 + s2 > 1.0 + 1e-6 {
            return;
        }
        tops.push(a[1] + s1 * (b[1] - a[1]) + s2 * (cc[1] - a[1]));
    };
    let unit: Vec<Vec<i32>> = Mesh::ALL.iter().map(|&k| vertices(k)).collect();
    for ch in &s.chunks {
        for d in ch.draws.chunks(7) {
            let (mesh, offset, count, first, n, kind) = (d[1], d[2], d[3], d[4], d[5], d[6]);
            if kind == GROUND || mesh == Mesh::Terrain as u32 {
                let (from, to) = (first as usize, (first + n) as usize);
                let v = &s.terrain[from * VERTEX_VALUES..to * VERTEX_VALUES];
                for t in v.chunks(VERTEX_VALUES * 3) {
                    let p = |k: usize| {
                        let q = &t[k * VERTEX_VALUES..];
                        [m(q[0]), m(q[1]), m(q[2])]
                    };
                    tri(p(0), p(1), p(2));
                }
                continue;
            }
            let base = offset as usize / 4;
            for i in 0..count as usize {
                let a = &s.values[base + i * INSTANCE_VALUES..];
                // rows: x = c0.x c1.x c2.x t.x, then y, then z
                let world = |q: &[i32]| {
                    let (lx, ly, lz) = (m(q[0]), m(q[1]), m(q[2]));
                    let row = |r: usize| {
                        m(a[r * 4]) * lx
                            + m(a[r * 4 + 1]) * ly
                            + m(a[r * 4 + 2]) * lz
                            + m(a[r * 4 + 3])
                    };
                    [row(0), row(1), row(2)]
                };
                // (quick reject: the shape's translation and reach)
                let reach: f64 = (0..3)
                    .map(|r| (0..3).map(|k| m(a[r * 4 + k]).abs()).sum::<f64>())
                    .sum();
                if (m(a[3]) - px).abs() > reach || (m(a[11]) - pz).abs() > reach {
                    continue;
                }
                for t in unit[mesh as usize].chunks(VERTEX_VALUES * 3) {
                    let w: Vec<[f64; 3]> = (0..3).map(|k| world(&t[k * VERTEX_VALUES..])).collect();
                    tri(w[0], w[1], w[2]);
                }
            }
        }
    }
    tops
}

fn start_problems(map: Map) -> Vec<String> {
    let c = map.course();
    let drawn = scenery(&c);
    let mut out = Vec::new();
    for (k, s) in c.starts.iter().enumerate() {
        let feet = s.y - BODY_DROP;
        let top = c.support(*s);
        if (feet - top).abs() > FEET_SLACK {
            out.push(format!(
                "slot {k}: feet at {:.2} m, the solid top under them at {:.2} m",
                m(feet),
                m(top)
            ));
            continue;
        }
        // (the feet's patch, not one point: Banana Bay's slots stand on the
        // 8 cm gaps between the pier's planks)
        let r = FEET_PATCH;
        let tops: Vec<f64> = [(0, 0), (r, 0), (-r, 0), (0, r), (0, -r)]
            .iter()
            .flat_map(|&(dx, dz)| drawn_tops(&drawn, s.x + dx, s.z + dz))
            .collect();
        let near = tops.iter().any(|&h| (h - m(top)).abs() <= m(DRAWN_SLACK));
        if !near {
            let best = tops
                .iter()
                .copied()
                .min_by(|a, b| (a - m(top)).abs().total_cmp(&(b - m(top)).abs()));
            out.push(format!(
                "slot {k}: solid top {:.2} m, nearest drawn top {best:.2?} m",
                m(top)
            ));
        }
    }
    out
}

// ----------------------------------------------------------------- finish

/// How far either side of the finish point the probe looks (it stops at
/// the first wall), and how high over the floor or a top (an ape flies
/// well over the vines' bottoms).
const PROBE_SIDE: i32 = 150;
const PROBE_UP: i32 = 32;

/// Whether a body could be at `p`: not pushed back by the corridor's walls
/// and not inside a map's solids.
fn open(c: &Course, p: V3, hint: usize) -> bool {
    let (q, _, _) = c.contain(p, V3::ZERO, hint);
    if q.sub(p).len() > ONE / 100 {
        return false;
    }
    if let Some(s) = c.ravine_solids() {
        if s.holds(p, BODY_RADIUS) {
            return false;
        }
    }
    if let Some(t) = c.tower.as_deref() {
        if t.stone_push(p, V3::ZERO).is_some() {
            return false;
        }
    }
    true
}

/// A body crossing the finish line `side` right of the finish point and
/// `up` over what is under it: from 1 m before the line to 1 m past it.
/// None when it cannot be there (a wall, a solid); else whether the race
/// counts it finished (its progress reaches the course's length, its
/// nearest sample found as the race finds it).
fn crosses(c: &Course, side: i32, up: i32) -> Option<bool> {
    let line = c.point(c.length);
    let h = c.heading_at(c.length);
    let r = right_of(h);
    let flat = |along: i32| line.add(h.scale(along)).add(r.scale(side));
    let ground = |q: V3| {
        c.support(v3(q.x, q.y + int(300), q.z))
            .max(c.floor(q.x, q.z))
    };
    let (a, b) = (flat(-ONE), flat(ONE));
    let y = ground(a).max(ground(b)) + BODY_DROP + up;
    let at_y = |q: V3| v3(q.x, y, q.z);
    let (a, b) = (at_y(a), at_y(b));
    let hint = c.nearest_anywhere(a);
    if !open(c, a, hint) {
        return None;
    }
    let hint = c.nearest(at_y(flat(0)), hint);
    let hint_b = c.nearest(b, hint);
    if !open(c, b, hint_b) {
        return None;
    }
    Some(c.progress(b, hint_b).0 >= c.length)
}

/// Past the line (`PAST` m deep, `PAST_STEP` apart): every spot a body
/// can reach from the finish point without going through a wall or a
/// solid, at heights `up` over the ground or a top there, counts as
/// finished. (TRIAGE T110: Rattlesnake Ravine's mesa opens wide past the
/// line, and wide on the left the nearest centreline sample is one before
/// the corner, so an ape that comes round there has not finished.)
const PAST: i32 = 45;
const PAST_STEP: i32 = 3;

fn past_problems(c: &Course, up: i32, out: &mut Vec<String>) {
    let line = c.point(c.length);
    let h = c.heading_at(c.length);
    let r = right_of(h);
    let rows = (PAST / PAST_STEP) as usize + 1;
    let cols = (2 * PROBE_SIDE + 1) as usize;
    // (row, col) -> Some(finished) where a body can be, None where it cannot
    let cell = |row: usize, col: usize| {
        let along = 1 + row as i32 * PAST_STEP;
        let side = col as i32 - PROBE_SIDE;
        let q = line.add(h.scale(int(along))).add(r.scale(int(side)));
        let ground = c
            .support(v3(q.x, q.y + int(300), q.z))
            .max(c.floor(q.x, q.z));
        let p = v3(q.x, ground + BODY_DROP + int(up), q.z);
        let hint = c.nearest_anywhere(p);
        open(c, p, hint).then(|| c.progress(p, hint).0 >= c.length)
    };
    let grid: Vec<Vec<Option<bool>>> = (0..rows)
        .map(|row| (0..cols).map(|col| cell(row, col)).collect())
        .collect();
    // reached from the finish point, a step at a time
    let mut seen = vec![vec![false; cols]; rows];
    let mut todo = vec![(0usize, PROBE_SIDE as usize)];
    while let Some((row, col)) = todo.pop() {
        if seen[row][col] || grid[row][col].is_none() {
            continue;
        }
        seen[row][col] = true;
        if grid[row][col] == Some(false) {
            out.push(format!(
                "{} m past the line, {} m right, {up} m up: no finish",
                1 + row as i32 * PAST_STEP,
                col as i32 - PROBE_SIDE
            ));
        }
        if row > 0 {
            todo.push((row - 1, col));
        }
        if row + 1 < rows {
            todo.push((row + 1, col));
        }
        if col > 0 {
            todo.push((row, col - 1));
        }
        if col + 1 < cols {
            todo.push((row, col + 1));
        }
    }
}

fn finish_problems(map: Map) -> Vec<String> {
    let c = map.course();
    let mut out = Vec::new();
    for up in (0..=PROBE_UP).step_by(4) {
        // across the line, out from the finish point either way, to the
        // first wall
        for dir in [1, -1] {
            for k in 0..=PROBE_SIDE {
                let side = dir * k;
                match crosses(&c, int(side), int(up)) {
                    None => break,
                    Some(true) => {}
                    Some(false) => out.push(format!("{side} m right, {up} m up: no finish")),
                }
            }
        }
    }
    for up in [0, 12, 24] {
        past_problems(&c, up, &mut out);
    }
    assert!(
        crosses(&c, 0, 0) == Some(true),
        "{map:?}: on the path itself the finish does not count"
    );
    out
}

macro_rules! per_map {
    ($($name:ident => $map:expr),* $(,)?) => {
        mod start {
            use super::*;
            $(#[test] fn $name() { verdict($map, Check::Start, start_problems($map)); })*
        }
        mod finish {
            use super::*;
            $(#[test] fn $name() { verdict($map, Check::Finish, finish_problems($map)); })*
        }
        /// Every map in `Map::ALL` has its tests here.
        #[test]
        fn every_map_is_checked() {
            let listed = [$($map),*];
            for m in Map::ALL {
                assert!(listed.contains(&m), "{m:?}: add it to per_map! in map_checks.rs");
            }
        }
    };
}

per_map! {
    jungle => Map::Jungle,
    city => Map::City,
    ski => Map::Ski,
    sector => Map::Sector,
    ruins => Map::Ruins,
    bay => Map::Bay,
    tower => Map::Tower,
    ravine => Map::Ravine,
    grocery => Map::Grocery,
    spindle => Map::Spindle,
}
