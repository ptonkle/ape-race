//! The Ski Slope's chunk drawing (TRIAGE T62): along the course, how many
//! scenery shapes are drawn at once, and how visible (1 - fog) the nearest
//! part of a chunk is when it starts or stops being drawn. A chunk that
//! appears while any of it shows through the fog is a pop-in.
//!
//!     cargo run --release -p tool_core --example ski_chunks [-- REACH_M|old [WEATHER [NEAR,FAR]]]
use tool_core::fx::{int, ONE};
use tool_core::mesh::Mesh;
use tool_core::render::DRAW_VALUES;

fn main() {
    let c = tool_core::map::Map::Ski.course();
    let mut s = tool_core::scene::scenery(&c);
    let mut look = tool_core::scene::look_of(&c);
    let args: Vec<String> = std::env::args().collect();
    if let Some(r) = args.get(1).and_then(|r| r.parse::<i32>().ok()) {
        if r > 0 {
            s.reach = int(r);
        }
    }
    if let Some(w) = args
        .get(2)
        .and_then(|w| tool_core::snow::Weather::from_key(w))
    {
        look = tool_core::snow::sky_in(w, look);
    }
    // `-- 0 clear 90,300`: try another fog
    if let Some((a, b)) = args.get(3).and_then(|f| f.split_once(',')) {
        look.fog_range = (int(a.parse().unwrap()), int(b.parse().unwrap()));
    }
    // `-- old`: the old rule (a chunk drawn while its centre is within
    // 260 m), to measure what it showed
    let old = args.get(1).is_some_and(|a| a == "old");
    let (near, far) = look.fog_range;
    let fog = |d: i32| -> f64 {
        let f = ((d - near) as f64 / (far - near) as f64).clamp(0.0, 1.0);
        f * f * (3.0 - 2.0 * f)
    };
    let bounded = s.chunks.iter().any(|ch| ch.bounds.is_some());
    let reach = match (bounded && !old, s.reach > 0) {
        (true, true) => s.reach.min(far),
        (true, false) => far,
        (false, true) => s.reach,
        (false, false) => tool_core::scene::DRAW_DISTANCE,
    };
    println!(
        "fog {}..{} m, reach {} m",
        near / ONE,
        far / ONE,
        reach / ONE
    );
    let mut most = 0;
    let mut best = 0;
    let mut total = 0u64;
    let mut worst_pop: f64 = 0.0;
    let mut pops = 0;
    let mut prev: Vec<bool> = vec![false; s.chunks.len()];
    // the chase camera rides a few metres up behind the ape
    for k in 0..c.path.len() {
        let eye = c.path[k].add(tool_core::geom::v3(0, int(6), 0));
        let d = if old {
            s.draws_near(eye)
        } else {
            s.draws_in(eye, far)
        };
        let m: u32 = d
            .chunks(DRAW_VALUES)
            .filter(|d| d[1] != Mesh::Terrain as u32)
            .map(|d| d[3])
            .sum();
        most = most.max(m);
        // the least that could be drawn: every shape any of which is
        // nearer than the reach, and no other
        let mut ideal = 0u32;
        for v in s.values.chunks(tool_core::render::INSTANCE_VALUES).skip(1) {
            let half = (0..3)
                .map(|k| tool_core::geom::v3(v[k], v[4 + k], v[8 + k]).len())
                .max()
                .unwrap_or(0)
                / 2;
            if tool_core::geom::v3(v[3] - eye.x, 0, v[11] - eye.z).len() - half <= reach {
                ideal += 1;
            }
        }
        best = best.max(ideal);
        total += m as u64;
        for (i, ch) in s.chunks.iter().enumerate() {
            // the nearest any of it can be (its centre, unbounded)
            let nearest = ch.nearest(eye);
            let on = if old {
                ch.centre.flat().sub(eye.flat()).len() <= reach
            } else {
                nearest <= reach
            };
            if on != prev[i] && k > 0 {
                let seen = 1.0 - fog(nearest);
                worst_pop = worst_pop.max(seen);
                if seen > 0.02 {
                    pops += 1;
                }
            }
            prev[i] = on;
        }
    }
    println!("the least that could be drawn at the busiest place: {best}");
    println!(
        "most shapes drawn at once {most}, mean {}; chunk changes visible through the fog {pops}, worst {:.0}% visible",
        total / c.path.len() as u64,
        worst_pop * 100.0
    );
}
