//! How many scenery shapes each map has, and the most drawn at once
//! (TRIAGE T31; SECOND-ORDER W8 and G2; the City, Y rows).
use tool_core::course::{Course, Prop};
use tool_core::mesh::{Mesh, VERTEX_VALUES};
use tool_core::render::{DRAW_VALUES, INSTANCE_VALUES};

fn report(name: &str, c: &Course) {
    let s = tool_core::scene::scenery(c);
    let n = s.values.len() / INSTANCE_VALUES;
    let mut most = 0u32;
    for k in 0..c.path.len() / 10 {
        let d = s.draws_near(c.path[k * 10]);
        let m: u32 = d
            .chunks(DRAW_VALUES)
            .filter(|d| d[1] != Mesh::Terrain as u32)
            .map(|d| d[3])
            .sum();
        most = most.max(m);
    }
    let trees = c
        .props
        .iter()
        .filter(|p| matches!(p, Prop::Tree { .. }))
        .count();
    println!(
        "{name}: scenery instances {n}, most drawn at once {most}, ground vertices {}, props {}, trees {trees}",
        s.terrain.len() / VERTEX_VALUES,
        c.props.len()
    );
}

fn main() {
    report("jungle", &Course::jungle());
    let city = tool_core::map::Map::City.course();
    report("city", &city);
    for (part, n) in tool_core::cityscape::parts(&city) {
        println!("  {part}: {n}");
    }
    let ski = tool_core::map::Map::Ski.course();
    report("ski", &ski);
    for (part, n) in tool_core::skiscape::parts(&ski) {
        println!("  {part}: {n}");
    }
    let sector = tool_core::map::Map::Sector.course();
    report("sector", &sector);
    for (part, n) in tool_core::sectorscape::parts(&sector) {
        println!("  {part}: {n}");
    }
    // Sector Ape draws much of itself each frame (the backdrop, set
    // pieces, mechs, the black hole): the most of that at once
    if let Some(sl) = &sector.sector {
        let mut most = (0, 0);
        for k in 0..sector.path.len() / 10 {
            let eye = sector.path[k * 10].add(tool_core::geom::v3(0, tool_core::fx::int(10), 0));
            let mut b = tool_core::render::Batch::new();
            for t in [0u32, 3_000, 9_000] {
                b = tool_core::render::Batch::new();
                tool_core::sectorscape::dynamic(&mut b, &sector, sl, eye, t, t);
            }
            let halos = b.take_halos().count();
            most = most.max((b.count(), halos));
        }
        println!("  per frame at most: {} shapes, {} glows", most.0, most.1);
    }
    bay_cost();
    let ruins = tool_core::map::Map::Ruins.course();
    report("ruins", &ruins);
    // Sunset Ruins draws its moving parts each frame (sky, foam, turtles,
    // falls, facade, glyphs, traps, pile, skylight): the most at once
    if let Some(rl) = &ruins.ruins {
        let mut most = (0, 0);
        for k in 0..ruins.path.len() / 10 {
            let p = ruins.path[k * 10];
            let eye =
                tool_core::geom::v3(p.x, ruins.terrain(p.x, p.z) + tool_core::fx::int(10), p.z);
            let mut b = tool_core::render::Batch::new();
            tool_core::ruinsscape::dynamic(&mut b, &ruins, rl, eye, 500, 500);
            let halos = b.take_halos().count();
            most = most.max((b.count(), halos));
        }
        println!("  per frame at most: {} shapes, {} glows", most.0, most.1);
    }
}

/// Banana Bay Battle (step 8, BA rows): the static scenery drawn at the
/// start, mid-course and the finish, and the most of it and of the
/// fleet's moving parts (flags, crews, fire, barrels, chest) at once,
/// shapes and triangles.
pub fn bay_cost() {
    use tool_core::geom::v3;
    use tool_core::mesh::{vertices, Mesh};
    use tool_core::render::Batch;
    let bay = tool_core::map::Map::Bay.course();
    report("bay", &bay);
    let Some(bl) = &bay.bay else { return };
    let s = tool_core::scene::scenery(&bay);
    let tri_of = |m: u32, n: u32| -> usize {
        let mesh = Mesh::ALL.into_iter().find(|x| *x as u32 == m).unwrap();
        n as usize * vertices(mesh).len() / VERTEX_VALUES / 3
    };
    let still = |eye: tool_core::geom::V3| -> (u32, usize) {
        let d = s.draws_near(eye);
        let mut shapes = 0;
        let mut tris = 0;
        for d in d
            .chunks(DRAW_VALUES)
            .filter(|d| d[1] != Mesh::Terrain as u32)
        {
            shapes += d[3];
            tris += tri_of(d[1], d[3]);
        }
        (shapes, tris)
    };
    let moving = |eye: tool_core::geom::V3, t: u32| -> (usize, usize) {
        let mut b = Batch::new();
        tool_core::bayscape::fleet::dynamic(&mut b, &bay, bl, eye, t, t, false);
        let tris = Mesh::ALL
            .iter()
            .map(|&m| b.count_of(m) * vertices(m).len() / VERTEX_VALUES / 3)
            .sum();
        (b.count(), tris)
    };
    let up = tool_core::fx::int(15);
    let last = bay.path.len() - 1;
    for (name, k) in [("start", 0), ("mid", last / 2), ("finish", last)] {
        let p = bay.path[k];
        let eye = v3(p.x, up, p.z);
        let (ss, st) = still(eye);
        let mut worst = (0, 0);
        for t in (0..2400).step_by(60) {
            let m = moving(eye, t);
            if m.0 > worst.0 {
                worst = m;
            }
        }
        println!(
            "  {name}: still {ss} shapes / {st} triangles; moving at most {} / {}; total {} / {}",
            worst.0,
            worst.1,
            ss as usize + worst.0,
            st + worst.1
        );
    }
    let mut worst = (0usize, 0usize, 0usize, 0);
    for k in 0..bay.path.len() / 5 {
        let p = bay.path[k * 5];
        let eye = v3(p.x, up, p.z);
        let (ss, st) = still(eye);
        for t in (0..2400).step_by(120) {
            let m = moving(eye, t);
            if ss as usize + m.0 > worst.0 {
                worst = (ss as usize + m.0, st + m.1, m.0, k * 5);
            }
        }
    }
    println!(
        "  worst frame: {} shapes / {} triangles ({} moving), at path point {} of {}",
        worst.0, worst.1, worst.2, worst.3, last
    );
}
