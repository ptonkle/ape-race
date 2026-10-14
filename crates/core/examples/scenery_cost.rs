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
}
