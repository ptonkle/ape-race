use tool_core::fx::{int, ONE};
use tool_core::geom::Affine;
use tool_core::render::Batch;
use tool_core::ruinsmodels as m;
fn main() {
    let f = Affine::IDENTITY;
    let mut out = vec![];
    macro_rules! c {
        ($n:expr, $e:expr) => {{
            let mut b = Batch::new();
            $e(&mut b);
            out.push(($n, b.count()));
        }};
    }
    c!("statue", |b: &mut Batch| m::banana_statue(b, &f, 1, false));
    c!("statue big", |b: &mut Batch| m::banana_statue(b, &f, 1, true));
    c!("moai big", |b: &mut Batch| m::moai_head(b, &f, 1, true));
    c!("turtle", |b: &mut Batch| m::turtle(b, &f, 0));
    c!("bridge 40", |b: &mut Batch| m::rope_bridge(b, &f, int(40)));
    c!("posts", |b: &mut Batch| m::bridge_posts(b, &f));
    c!("mouth", |b: &mut Batch| m::temple_mouth(b, &f, 0));
    c!("waterfall", |b: &mut Batch| m::waterfall(b, &f, int(28), int(90), 0));
    c!("panel 10x8", |b: &mut Batch| m::stone_panel(b, &f, int(10), int(8)));
    c!("panel 30x32", |b: &mut Batch| m::stone_panel(b, &f, int(30), int(32)));
    c!("glyph", |b: &mut Batch| m::banana_glyph(b, &f, 0, 0));
    c!("spike", |b: &mut Batch| m::spike_trap(b, &f, ONE));
    c!("wire", |b: &mut Batch| m::trip_wire(b, &f, int(30), false));
    c!("trapdoor", |b: &mut Batch| m::trapdoor(b, &f, int(30), int(12), 0));
    c!("pit", |b: &mut Batch| m::pit_spikes(b, &f, int(30), int(12), 0));
    c!("pile", |b: &mut Batch| m::banana_pile(b, &f, 0));
    c!("skylight", |b: &mut Batch| m::skylight(b, &f, int(14), int(15), int(46), 0));
    c!("kapok", |b: &mut Batch| m::jungle_tree(b, &f, m::Tree::Kapok, 1));
    c!("palm", |b: &mut Batch| m::jungle_tree(b, &f, m::Tree::Palm, 1));
    c!("fig", |b: &mut Batch| m::jungle_tree(b, &f, m::Tree::Fig, 1));
    c!("blossom", |b: &mut Batch| m::jungle_tree(b, &f, m::Tree::Blossom, 1));
    c!("sky", |b: &mut Batch| m::sky(b, tool_core::geom::V3::ZERO, 0));
    for (n, k) in out {
        println!("{n}: {k}");
    }
    println!(
        "spots 10x8 {}  30x32 {}",
        m::glyph_spots(int(10), int(8)).len(),
        m::glyph_spots(int(30), int(32)).len()
    );
    println!("spots10x8 {:?}", m::glyph_spots(int(10), int(8)));
    let c = tool_core::map::Map::Ruins.course();
    println!(
        "half_width {} len {}",
        c.half_width as f64 / 65536.0,
        c.length as f64 / 65536.0
    );
    let r = c.ruins.as_ref().unwrap();
    println!(
        "bridge {:?} {:?} mouth {:?} chamber {:?} vines {} turtles {} path0 {:?} h0 {:?} lobby {:?}",
        r.bridge.from,
        r.bridge.to,
        r.mouth.opening,
        r.chamber,
        c.vines.len(),
        r.turtles.len(),
        c.path[0],
        c.heading[0],
        r.lobby
    );
}
