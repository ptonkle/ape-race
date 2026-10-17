//! Prints Mape's Tower's layout (the art agent's look at the data).
use tool_core::fx::ONE;
use tool_core::geom::V3;

fn p(v: V3) -> String {
    format!(
        "({:.1},{:.1},{:.1})",
        v.x as f64 / ONE as f64,
        v.y as f64 / ONE as f64,
        v.z as f64 / ONE as f64
    )
}

fn f(x: i32) -> f64 {
    x as f64 / ONE as f64
}

fn main() {
    let c = tool_core::map::Map::Tower.course();
    let t = c.tower.as_ref().unwrap();
    let sc = tool_core::scene::scenery(&c);
    for ch in &sc.chunks {
        let n: u32 = ch
            .draws
            .chunks(tool_core::render::DRAW_VALUES)
            .map(|d| d[3])
            .sum();
        println!(
            "chunk {} n {} bounds {:?} h {:?}",
            p(ch.centre),
            n,
            ch.bounds.map(|b| b.map(f)),
            ch.heights.map(|b| b.map(f))
        );
    }
    for (n, k) in t.towers.iter().enumerate() {
        println!(
            "len {} path {} tower {n} {} r {} in {} h {} levels {}",
            f(c.length),
            c.path.len(),
            p(k.centre),
            f(k.radius),
            f(k.inner),
            f(k.height),
            k.levels.len()
        );
    }
    for b in &t.bridges {
        println!(
            "bridge {} -> {} hw {} deck {:?}",
            p(b.from),
            p(b.to),
            f(b.half_width),
            b.deck
        );
    }
    for b in &t.balconies {
        println!(
            "balc {} out {} hw {} d {} mage {} deck {:?}",
            p(b.at),
            p(b.out),
            f(b.half_width),
            f(b.depth),
            b.mage,
            b.deck
        );
    }
    for b in &t.clouds {
        println!("cloud band {} {}", f(b.bottom), f(b.top));
    }
    let s = &t.portal;
    println!(
        "portal wiz {} yaw {} tear {} half {} {} through {} banana {} len {}",
        p(s.wizard),
        s.wizard_yaw,
        p(s.tear),
        f(s.tear_half.0),
        f(s.tear_half.1),
        p(s.through),
        p(s.banana),
        f(s.banana_len)
    );
    let z = &t.tube;
    println!(
        "tube axis {} top {} bottom {} r {} discs {}",
        p(z.axis),
        f(z.top),
        f(z.bottom),
        f(z.radius),
        z.discs.len()
    );
    let bz = &t.banana_zone;
    println!(
        "bz {} r {} plat {}",
        p(bz.centre),
        f(bz.radius),
        bz.platform
    );
    for pd in &t.pads {
        println!(
            "pad at {} plat {} boost {}",
            f(pd.at),
            pd.platform,
            p(pd.boost)
        );
    }
    for (i, pl) in c.platforms.iter().enumerate() {
        println!(
            " plat {i} c {} top {} hw {} hl {} fwd {}",
            p(pl.centre),
            f(pl.top),
            f(pl.half_width),
            f(pl.half_len),
            p(pl.fwd)
        );
    }
    for (i, m) in c.mushrooms.iter().enumerate() {
        println!(" mush {i} {:?}", m);
    }
    for (i, h) in t.hangers.iter().enumerate() {
        println!(
            " vine {i} {:?} anchor {} len {} hang {}",
            h,
            p(c.anchor(i, 0)),
            f(c.vines[i].length),
            p(t.hangs[i])
        );
    }
}
