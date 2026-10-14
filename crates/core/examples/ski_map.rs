//! The Ski Slope from above as an SVG on stdout: the lanes (the walls, the
//! forks' islands), the lifts (line, towers, terminals), the conifers, the
//! vines by hanger (chair blue, tree green, wire grey, helicopter red, low
//! vines small), crates, landmarks, and contour lines of the hill every
//! 10 m. On stderr: the run's profile (height and grade along it). For
//! checking the layout by eye.
//!
//! `cargo run --release -p tool_core --example ski_map > ski.svg`
use tool_core::fx::ONE;
use tool_core::geom::right_of;
use tool_core::ski::{ski, HangerKind};

fn main() {
    let c = ski();
    let lay = c.ski.as_ref().unwrap();
    let m = |v: i32| v as f64 / ONE as f64;
    let (mut x0, mut x1, mut z0, mut z1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for p in &c.path {
        x0 = x0.min(m(p.x));
        x1 = x1.max(m(p.x));
        z0 = z0.min(m(p.z));
        z1 = z1.max(m(p.z));
    }
    let pad = 100.0;
    let (x0, z0, w, h) = (x0 - pad, z0 - pad, x1 - x0 + 2.0 * pad, z1 - z0 + 2.0 * pad);
    println!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{x0} {z0} {w} {h}" width="{}" height="{}" style="background:#eef">"#,
        w * 1.2,
        h * 1.2
    );
    // contours
    let step = 4;
    let mut gx = x0 as i32;
    while gx < (x0 + w) as i32 {
        let mut gz = z0 as i32;
        while gz < (z0 + h) as i32 {
            let y = c.terrain(gx * ONE, gz * ONE) / ONE;
            let y2 = c.terrain((gx + step) * ONE, gz * ONE) / ONE;
            let y3 = c.terrain(gx * ONE, (gz + step) * ONE) / ONE;
            if y.div_euclid(10) != y2.div_euclid(10) || y.div_euclid(10) != y3.div_euclid(10) {
                println!(r##"<rect x="{gx}" y="{gz}" width="2" height="2" fill="#99a"/>"##);
            }
            gz += step;
        }
        gx += step;
    }
    // lanes, as strips
    let mut s = -30 * ONE;
    while s < c.length + 150 * ONE {
        let p = c.point(s);
        let r = right_of(c.heading_at(s));
        for (mid, half) in c.lanes(s.clamp(0, c.length)) {
            for e in [-1, 1] {
                let q = p.add(r.scale(mid + e * half));
                println!(
                    r##"<circle cx="{}" cy="{}" r="0.8" fill="#e60"/>"##,
                    m(q.x),
                    m(q.z)
                );
            }
        }
        s += 2 * ONE;
    }
    // centreline
    let pts: Vec<String> = c
        .path
        .iter()
        .step_by(5)
        .map(|p| format!("{},{}", m(p.x), m(p.z)))
        .collect();
    println!(
        r##"<polyline points="{}" fill="none" stroke="#88a" stroke-width="0.6" stroke-dasharray="3 3"/>"##,
        pts.join(" ")
    );
    for k in (0..=c.length / ONE).step_by(100) {
        let p = c.point(k * ONE);
        println!(
            r##"<text x="{}" y="{}" font-size="9" fill="#446">{k}</text>"##,
            m(p.x) + 2.0,
            m(p.z)
        );
    }
    for t in &lay.trees {
        println!(
            r##"<circle cx="{}" cy="{}" r="{}" fill="#273" fill-opacity="0.5"/>"##,
            m(t.at.x),
            m(t.at.z),
            m(t.radius) / 3.0
        );
    }
    for (i, l) in lay.lifts.iter().enumerate() {
        println!(
            r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#333" stroke-width="1.2"/>"##,
            m(l.bottom.x),
            m(l.bottom.z),
            m(l.top.x),
            m(l.top.z)
        );
        for t in &l.towers {
            println!(
                r##"<rect x="{}" y="{}" width="3" height="3" fill="#333"/>"##,
                m(t.x) - 1.5,
                m(t.z) - 1.5
            );
        }
        for (p, lab) in [(l.bottom, "B"), (l.top, "T")] {
            println!(
                r##"<rect x="{}" y="{}" width="8" height="8" fill="#a33"/><text x="{}" y="{}" font-size="10">L{}{lab}</text>"##,
                m(p.x) - 4.0,
                m(p.z) - 4.0,
                m(p.x) + 5.0,
                m(p.z) + 4.0,
                i + 1
            );
        }
    }
    for hg in &lay.hangers {
        let v = &c.vines[hg.vine];
        let col = match hg.kind {
            HangerKind::Chair { .. } => "#24c",
            HangerKind::Tree { .. } => "#2a2",
            HangerKind::Wire => "#777",
            HangerKind::Helicopter => "#e22",
        };
        let g = c.terrain(v.anchor.x, v.anchor.z);
        let low = v.anchor.y - v.length - g < 5 * ONE;
        if let HangerKind::Tree { .. } | HangerKind::Wire = hg.kind {
            println!(
                r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{col}" stroke-width="0.4"/>"##,
                m(hg.from.x),
                m(hg.from.z),
                m(hg.to.x),
                m(hg.to.z)
            );
        }
        println!(
            r##"<circle cx="{}" cy="{}" r="{}" fill="{col}"/>"##,
            m(v.anchor.x),
            m(v.anchor.z),
            if low { 1.0 } else { 2.0 }
        );
    }
    for k in &c.crates {
        println!(
            r##"<rect x="{}" y="{}" width="2" height="2" fill="#c80"/>"##,
            m(k.x) - 1.0,
            m(k.z) - 1.0
        );
    }
    for l in &lay.landmarks {
        println!(
            r##"<circle cx="{}" cy="{}" r="7" fill="#90c"/><text x="{}" y="{}" font-size="12" fill="#609">{:?}</text>"##,
            m(l.at.x),
            m(l.at.z),
            m(l.at.x) + 8.0,
            m(l.at.z),
            l.kind
        );
    }
    println!("</svg>");
    // the profile
    let mut last = 0;
    for k in (0..=(c.length / ONE + 150)).step_by(50) {
        let p = c.point(k * ONE);
        let y = c.terrain(p.x, p.z);
        let d = c.terrain.downhill(p.x, p.z).len();
        eprintln!(
            "{k:>5} m  height {:>7.1}  drop {:>5.1}  grade {:>5.1}%  along {:>5.1}%",
            m(y),
            m(last - y),
            m(d) * 100.0,
            m(last - y) * 2.0
        );
        last = y;
    }
}
