use tool_core::mesh::Mesh;
use tool_core::render::DRAW_VALUES;
fn main() {
    let c = tool_core::map::Map::Ruins.course();
    let s = tool_core::scene::scenery(&c);
    for k in (0..c.path.len()).step_by(25) {
        let d = s.draws_near(c.path[k]);
        let m: u32 = d
            .chunks(DRAW_VALUES)
            .filter(|d| d[1] != Mesh::Terrain as u32)
            .map(|d| d[3])
            .sum();
        print!("{}m:{} ", k * 2, m);
    }
    println!();
    for (i, ch) in s.chunks.iter().enumerate() {
        let m: u32 = ch
            .draws
            .chunks(DRAW_VALUES)
            .filter(|d| d[1] != Mesh::Terrain as u32)
            .map(|d| d[3])
            .sum();
        print!("c{i}:{m} ");
    }
    println!();
}
