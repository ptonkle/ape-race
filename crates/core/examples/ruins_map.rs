//! Sunset Ruins from above as an SVG on stdout: the river, the ravine and
//! the bridge, the corridor, the traps and spikes, the chamber's walls,
//! the pile, vines (by hanger), turtles' swims and the centreline. For
//! checking the layout by eye.
//!
//! `cargo run --release -p tool_core --example ruins_map > ruins.svg`
use tool_core::fx::ONE;
use tool_core::geom::V3;
use tool_core::ruins::{ruins, Hanger};

fn main() {
    let c = ruins();
    let lay = c.ruins.as_ref().unwrap();
    let m = |v: i32| v as f64 / ONE as f64;
    let (mut x0, mut x1, mut z0, mut z1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for p in &c.path {
        x0 = x0.min(m(p.x));
        x1 = x1.max(m(p.x));
        z0 = z0.min(m(p.z));
        z1 = z1.max(m(p.z));
    }
    let pad = 140.0;
    let (x0, z0, w, h) = (x0 - pad, z0 - pad, x1 - x0 + 2.0 * pad, z1 - z0 + 2.0 * pad);
    println!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{x0} {z0} {w} {h}" width="{w}" height="{h}" style="background:#2a1a14">"#
    );
    let line = |a: V3, b: V3, col: &str, wd: f64| {
        println!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{col}" stroke-width="{wd}"/>"#,
            m(a.x),
            m(a.z),
            m(b.x),
            m(b.z)
        );
    };
    // the river
    for p in lay.river.points.windows(2) {
        line(p[0].at, p[1].at, "#2d7fa8", m(p[0].half_width) * 2.0);
    }
    // the corridor
    for s in &lay.corridor {
        line(s.from, s.to, "#6b5a3e", m(s.half_width) * 2.0);
    }
    // the chamber: its wall and the invisible inner wall
    let ch = lay.chamber;
    println!(
        r##"<circle cx="{}" cy="{}" r="{}" fill="#4a3c28" stroke="#c9a46a" stroke-width="2"/>"##,
        m(ch.centre.x),
        m(ch.centre.z),
        m(ch.radius)
    );
    println!(
        r##"<circle cx="{}" cy="{}" r="{}" fill="#2a1a14" stroke="#888" stroke-dasharray="4 4"/>"##,
        m(ch.centre.x),
        m(ch.centre.z),
        m(ch.inner)
    );
    println!(
        r##"<circle cx="{}" cy="{}" r="{}" fill="#ffd23a"/>"##,
        m(lay.pile.centre.x),
        m(lay.pile.centre.z),
        m(lay.pile.radius)
    );
    // the lip, the bridge, the mouth, the reset spot
    line(lay.ravine.lip.0, lay.ravine.lip.1, "#ffffff", 2.0);
    line(
        lay.bridge.from,
        lay.bridge.to,
        "#c08040",
        m(lay.bridge.width),
    );
    let o = lay.mouth.opening;
    let across = tool_core::geom::right_of(o.fwd).scale(o.half.x);
    line(o.centre.sub(across), o.centre.add(across), "#e05030", 3.0);
    println!(
        r##"<circle cx="{}" cy="{}" r="3" fill="#ff40ff"/>"##,
        m(lay.ravine.spot.x),
        m(lay.ravine.spot.z)
    );
    // traps: wire, door; spikes
    for t in &lay.traps {
        let d = t.door;
        let a = tool_core::geom::right_of(d.fwd).scale(d.half.x);
        line(
            d.centre.sub(a),
            d.centre.add(a),
            "#ff8800",
            m(d.half.z) * 2.0,
        );
        line(t.wire.0, t.wire.1, "#ff2020", 1.0);
    }
    for s in &lay.spikes {
        let a = s.area;
        println!(
            r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#d02020"/>"##,
            m(a.centre.x - a.half.x),
            m(a.centre.z - a.half.z),
            m(a.half.x) * 2.0,
            m(a.half.z) * 2.0
        );
    }
    // turtles' swims
    for t in &lay.turtles {
        line(t.from, t.to, "#3a8a3a", 2.0);
    }
    // the centreline
    let pts: Vec<String> = c
        .path
        .iter()
        .step_by(2)
        .map(|p| format!("{},{}", m(p.x), m(p.z)))
        .collect();
    println!(
        r##"<polyline points="{}" fill="none" stroke="#fff" stroke-width="0.6" stroke-dasharray="3 3"/>"##,
        pts.join(" ")
    );
    // vines by hanger
    for (i, v) in c.vines.iter().enumerate() {
        let col = match lay.hangers[i] {
            Hanger::Tree { .. } => "#7fdc4a",
            Hanger::Ruin { .. } => "#b0b0a0",
            Hanger::Bridge => "#ffcc00",
            Hanger::Ceiling => "#a070ff",
            Hanger::Pit { .. } => "#ff5050",
            Hanger::Skylight => "#ffffff",
            Hanger::Mouth => "#ff9040",
        };
        println!(
            r#"<circle cx="{}" cy="{}" r="1.6" fill="{col}"/>"#,
            m(v.anchor.x),
            m(v.anchor.z)
        );
    }
    println!("</svg>");
}
