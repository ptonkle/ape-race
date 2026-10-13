//! The City from above as an SVG on stdout: streets (the walls), blocks,
//! vines (by hanger), bouncers, the river and the deck, and the
//! centreline. For checking the layout by eye.
//!
//! `cargo run --release -p tool_core --example city_map > city.svg`
use tool_core::city::{city, HangerKind};
use tool_core::fx::ONE;
use tool_core::geom::right_of;

fn main() {
    let c = city();
    let lay = c.city.as_ref().unwrap();
    let m = |v: i32| v as f64 / ONE as f64;
    let (mut x0, mut x1, mut z0, mut z1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for p in &c.path {
        x0 = x0.min(m(p.x));
        x1 = x1.max(m(p.x));
        z0 = z0.min(m(p.z));
        z1 = z1.max(m(p.z));
    }
    let pad = 120.0;
    let (x0, z0, w, h) = (x0 - pad, z0 - pad, x1 - x0 + 2.0 * pad, z1 - z0 + 2.0 * pad);
    println!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{x0} {z0} {w} {h}" width="{}" height="{}" style="background:#223">"#,
        w * 1.0,
        h * 1.0
    );
    if let Some(r) = lay.river {
        let o = c.point(r.from);
        let hd = c.heading_at(r.from);
        let rt = right_of(hd);
        let pts: Vec<String> = [(0, -220), (0, 220), (1, 220), (1, -220)]
            .iter()
            .map(|&(e, s)| {
                let p = o
                    .add(hd.scale(if e == 1 { r.to - r.from } else { 0 }))
                    .add(rt.scale(s * ONE));
                format!("{},{}", m(p.x), m(p.z))
            })
            .collect();
        println!(r##"<polygon points="{}" fill="#246"/>"##, pts.join(" "));
    }
    for st in &c.streets {
        let (lo, hi) = st.span(c.half_width);
        let r = right_of(st.dir);
        let pts: Vec<String> = [(lo, -1), (lo, 1), (hi, 1), (hi, -1)]
            .iter()
            .map(|&(a, s)| {
                let p = st.from.add(st.dir.scale(a)).add(r.scale(s * c.half_width));
                format!("{},{}", m(p.x), m(p.z))
            })
            .collect();
        println!(
            r##"<polygon points="{}" fill="#556" fill-opacity="0.6"/>"##,
            pts.join(" ")
        );
    }
    for b in &lay.blocks {
        let r = right_of(b.fwd);
        let pts: Vec<String> = [(-1, -1), (-1, 1), (1, 1), (1, -1)]
            .iter()
            .map(|&(i, j)| {
                let p = b
                    .centre
                    .add(b.fwd.scale(b.half_len * i))
                    .add(r.scale(b.half_width * j));
                format!("{},{}", m(p.x), m(p.z))
            })
            .collect();
        let shade = 60 + (m(b.height).min(190.0) as i32);
        let col = match b.style {
            4 => "#a83".to_string(),
            5 => "#c9c".to_string(),
            _ => format!("rgb({shade},{shade},{})", shade + 20),
        };
        println!(
            r##"<polygon points="{}" fill="{col}" stroke="#000" stroke-width="0.5"/>"##,
            pts.join(" ")
        );
    }
    for pl in &c.platforms {
        let r = right_of(pl.fwd);
        let pts: Vec<String> = [(-1, -1), (-1, 1), (1, 1), (1, -1)]
            .iter()
            .map(|&(i, j)| {
                let p = pl
                    .centre
                    .add(pl.fwd.scale(pl.half_len * i))
                    .add(r.scale(pl.half_width * j));
                format!("{},{}", m(p.x), m(p.z))
            })
            .collect();
        println!(
            r##"<polygon points="{}" fill="#987" fill-opacity="0.8"/>"##,
            pts.join(" ")
        );
    }
    let line: Vec<String> = c
        .path
        .iter()
        .map(|p| format!("{},{}", m(p.x), m(p.z)))
        .collect();
    println!(
        r##"<polyline points="{}" fill="none" stroke="#ff0" stroke-width="0.6"/>"##,
        line.join(" ")
    );
    for mu in &c.mushrooms {
        println!(
            r##"<circle cx="{}" cy="{}" r="{}" fill="#f44"/>"##,
            m(mu.top.x),
            m(mu.top.z),
            m(mu.radius)
        );
    }
    for hg in &lay.hangers {
        let v = c.vines[hg.vine];
        let col = match hg.kind {
            HangerKind::Wire => "#0ff",
            HangerKind::Crane => "#fa0",
            HangerKind::Helicopter => "#f0f",
            HangerKind::StreetLight => "#ff8",
            HangerKind::BridgeCable => "#8f8",
        };
        if hg.kind == HangerKind::Wire {
            println!(
                r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{col}" stroke-width="0.3"/>"##,
                m(hg.from.x),
                m(hg.from.z),
                m(hg.to.x),
                m(hg.to.z)
            );
        }
        let low = v.anchor.y - v.length < 4 * ONE;
        println!(
            r##"<circle cx="{}" cy="{}" r="{}" fill="{col}"/>"##,
            m(v.anchor.x),
            m(v.anchor.z),
            if low { 2.5 } else { 1.5 }
        );
    }
    println!("</svg>");
}
