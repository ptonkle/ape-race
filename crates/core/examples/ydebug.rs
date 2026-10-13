use tool_core::mesh::Mesh;
use tool_core::render::DRAW_VALUES;

fn main() {
    let c = tool_core::cityscape::demo();
    let s = tool_core::scene::scenery(&c);
    for ch in &s.chunks {
        for d in ch.draws.chunks(DRAW_VALUES) {
            if d[1] == Mesh::Wedge as u32 {
                let first = d[2] as usize / 4;
                for i in s.values[first..first + d[3] as usize * 16].chunks(16) {
                    println!("{:?}", i.iter().map(|v| v * 100 / 65536).collect::<Vec<_>>());
                }
            }
        }
    }
}
