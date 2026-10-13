//! How many scenery shapes the jungle has, and the most drawn at once
//! (TRIAGE T31; SECOND-ORDER W8 and G2).
use tool_core::course::{Course, Prop};
use tool_core::mesh::Mesh;
use tool_core::render::{DRAW_VALUES, INSTANCE_VALUES};

fn main() {
    let c = Course::jungle();
    let s = tool_core::scene::scenery(&c);
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
        "scenery instances {n}, most drawn at once {most}, props {}, trees {trees}",
        c.props.len()
    );
}
