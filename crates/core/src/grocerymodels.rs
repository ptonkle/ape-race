//! Clean-Up on Aisle Ape's models (step 4): the products, the shoppers and
//! their carts and balloons, and the store's smaller fixtures, as
//! stand-alone builders taking plain values (a point, a yaw, sizes, a look
//! number, the tick), in the store's own palette (`groceryscape`).
//!
//! **Sizes:** a product's `size` is real centimetres (width, height, depth);
//! the game is `grocery::SCALE` (12) game metres to the real metre, so a
//! 30 cm cereal box is 3.6 m tall in the game, next to a 3.3 m ape.
//! Everything else is given in game metres.
//!
//! **Frames:** a builder that takes `foot` and `yaw` stands its model on
//! `foot` (on the floor or a shelf), its front toward `facing(yaw, 0)`.
//!
//! **Budget** (the scene's 5,500 shapes a frame): a product at `Detail::Block`
//! is one shape; at `Detail::Full` at most `FULL_MOST` shapes. A shopper is
//! at most `SHOPPER_MOST`.
//!
//! CONTRACT (the skeleton): the bodies below are plain stand-ins; the
//! models step replaces them, keeping these signatures.

pub mod catalogue;

pub use catalogue::{Aisle, AISLES};

use crate::fx::{cm, int, ONE};
use crate::geom::{facing, v3, V3};
use crate::mesh::Mesh;
use crate::render::{placed, Batch, Rgb};

/// A product's shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// a printed carton (cereal, crackers, pizza)
    Box,
    /// a pillow bag (chips, frozen peas, bread)
    Bag,
    /// a tin can
    Can,
    /// a bottle with a neck and a cap
    Bottle,
    /// a glass jar with a lid
    Jar,
    /// a round tub with a lid (ice cream, cat litter)
    Tub,
    /// a jug with a handle and a cap
    Jug,
    /// a multipack case (soda, water, paper towels)
    Case,
    /// a tall round canister (oats, nuts, formula)
    Canister,
    /// a flat sealed pouch (tortillas, cookies, ramen)
    Pouch,
    /// a shallow tray (mints)
    Tray,
    /// a soft sack, wider at the bottom (flour, rice, dog food, ice)
    Sack,
}

/// A product's front: the body colour, a label colour and an accent laid
/// out as a band across it, stripes, a split, dots, a window, a chevron,
/// a coloured lid over a label, or plain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Front {
    Band,
    Stripe,
    Split,
    Dots,
    Window,
    Chevron,
    Lid,
    Plain,
}

/// A product on the store's shelves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Product {
    /// what it is (for the docs and the tests; never shown)
    pub name: &'static str,
    pub shape: Shape,
    /// width, height, depth in real centimetres
    pub size: [i32; 3],
    /// body, label, accent
    pub colours: [Rgb; 3],
    pub front: Front,
}

impl Product {
    /// Its size in game metres: across, up, deep.
    pub fn size_m(&self) -> V3 {
        let [w, h, d] = self.size;
        v3(
            crate::grocery::real_cm(w),
            crate::grocery::real_cm(h),
            crate::grocery::real_cm(d),
        )
    }
}

/// How much of a product is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detail {
    /// one shape in its body colour (far, or a whole block of facings)
    Block,
    /// its shape and front
    Full,
}

/// A product at `Detail::Full` is at most this many shapes.
pub const FULL_MOST: usize = 6;
/// A shopper (with a cart and a balloon hand) is at most this many.
pub const SHOPPER_MOST: usize = 40;

/// One facing of `p` standing on `foot` (the middle of its base's front
/// edge), its front toward `facing(yaw, 0)`.
pub fn product(b: &mut Batch, p: &Product, foot: V3, yaw: i32, detail: Detail) {
    let s = p.size_m();
    let out = facing(yaw, 0);
    let mid = foot.add(v3(0, s.y / 2, 0)).sub(out.scale(s.z / 2));
    b.push(Mesh::Cube, &placed(mid, yaw, s), p.colours[0], 0);
    let _ = detail;
}

/// A shopper standing at `feet`, facing `yaw`, walking (`stride` 0..ONE
/// round one step cycle; 0 standing). `look` (0..7) picks their clothes,
/// skin and hair. With `string`, their right hand is raised and holds a
/// balloon string whose end is there; with `cart`, they push a shopping
/// cart ahead of them. 20.4 m tall (`grocery::SHOPPER_HEIGHT`).
pub fn shopper(
    b: &mut Batch,
    feet: V3,
    yaw: i32,
    look: u8,
    stride: i32,
    string: Option<V3>,
    cart: bool,
) {
    let h = crate::grocery::SHOPPER_HEIGHT;
    b.push(
        Mesh::Barrel,
        &placed(feet.add(v3(0, h / 2, 0)), yaw, v3(int(6), h, int(4))),
        [ONE / 2; 3],
        0,
    );
    let _ = (look, stride, string, cart);
}

/// A bunch of helium balloons whose strings meet at `anchor` (the vine's
/// anchor: their knot, under the balloons), bobbing with `tick`; `k` picks
/// the bunch's colours.
pub fn balloons(b: &mut Batch, anchor: V3, tick: u32, k: u32) {
    b.push(
        Mesh::Sphere,
        &placed(anchor.add(v3(0, int(3), 0)), 0, v3(int(5), int(6), int(5))),
        [ONE, ONE / 4, ONE / 4],
        0,
    );
    let _ = (tick, k);
}

/// A bin of bouncy balls (a bounce bouncer): its top's middle at `top`,
/// `radius` round.
pub fn ball_bin(b: &mut Batch, top: V3, radius: i32) {
    b.push(
        Mesh::Cyl,
        &placed(
            top.sub(v3(0, top.y / 2, 0)),
            0,
            v3(radius * 2, top.y, radius * 2),
        ),
        [ONE, ONE / 2, 0],
        0,
    );
}

/// A produce table's pile (`stock`: 0 apples, 1 oranges, 2 bananas, 3
/// melons, 4 lettuce, 5 tomatoes, 6 peppers) on a dark wooden round table
/// `radius` round, `height` tall, standing on `foot`.
pub fn table(b: &mut Batch, foot: V3, radius: i32, height: i32, stock: u8) {
    b.push(
        Mesh::Cyl,
        &placed(
            foot.add(v3(0, height / 2, 0)),
            0,
            v3(radius * 2, height, radius * 2),
        ),
        [ONE / 3; 3],
        0,
    );
    let _ = stock;
}

/// A kraft cardboard display bin full of product `stock` (an aisle of
/// `AISLES`), `radius` round, `height` tall, on `foot`.
pub fn bin(b: &mut Batch, foot: V3, radius: i32, height: i32, stock: u8) {
    b.push(
        Mesh::Cyl,
        &placed(
            foot.add(v3(0, height / 2, 0)),
            0,
            v3(radius * 2, height, radius * 2),
        ),
        [ONE * 7 / 10, ONE * 6 / 10, ONE * 4 / 10],
        0,
    );
    let _ = stock;
}

/// A mister's puff of spray from `at` (a nozzle over a produce table), a
/// soft white burst that grows and fades over its first `cm(…)` of each
/// `period` ticks, `phase` in; nothing between puffs. Glows.
pub fn mister(b: &mut Batch, at: V3, tick: u32, period: u32, phase: u32) {
    let _ = (b, at, tick, period, phase);
}

/// A navy aisle sign hanging from the ceiling at `at` (its middle), facing
/// `yaw` both ways, showing `number` in white digits built from shapes.
pub fn aisle_sign(b: &mut Batch, at: V3, yaw: i32, number: u32) {
    b.push(
        Mesh::Cube,
        &placed(at, yaw, v3(int(8), int(4), cm(40))),
        [0, 0, ONE / 2],
        0,
    );
    let _ = number;
}

/// A red-and-yellow sale banner hanging at `at`, facing `yaw`; `k` picks
/// its price digits.
pub fn banner(b: &mut Batch, at: V3, yaw: i32, k: u32) {
    b.push(
        Mesh::Cube,
        &placed(at, yaw, v3(int(5), int(7), cm(20))),
        [ONE, 0, 0],
        0,
    );
    let _ = k;
}

/// A department's sign: its picture (`dept`: 0 dairy milk bottle and cheese,
/// 1 meat steak and fish, 2 deli, 3 bakery bread loaf, 4 produce carrot and
/// apple, 5 frozen snowflake) on the soffit at `at`, facing `yaw`, `size`
/// across.
pub fn dept_sign(b: &mut Batch, at: V3, yaw: i32, dept: u8, size: i32) {
    b.push(
        Mesh::Cube,
        &placed(at, yaw, v3(size, size, cm(30))),
        [ONE / 4; 3],
        0,
    );
    let _ = dept;
}

/// A shopping cart on `foot` (its middle), facing `yaw`.
pub fn cart(b: &mut Batch, foot: V3, yaw: i32) {
    b.push(
        Mesh::Cube,
        &placed(foot.add(v3(0, int(6), 0)), yaw, v3(int(7), int(5), int(11))),
        [ONE * 3 / 4; 3],
        0,
    );
}

/// The start's stock cart (a U-boat) loaded with shipping cases: `centre`
/// on the floor, the way along it `yaw`, `size` (across, the load's top,
/// along): its top is the start platform.
pub fn stock_cart(b: &mut Batch, centre: V3, yaw: i32, size: V3) {
    b.push(
        Mesh::Cube,
        &placed(centre.add(v3(0, size.y / 2, 0)), yaw, size),
        [ONE * 7 / 10, ONE * 6 / 10, ONE * 4 / 10],
        0,
    );
}

/// A pallet display of water and soda cases filling exactly the box the
/// Ravine's sphinx is solid as (`ravinemodels::SPHINX_SIZE`): its front
/// (the sphinx's front feet's middle) at `front` on the floor, facing
/// `yaw`, its body behind.
pub fn pallet(b: &mut Batch, front: V3, yaw: i32) {
    let size = crate::ravinemodels::SPHINX_SIZE;
    let back = facing(yaw, 0).neg();
    b.push(
        Mesh::Cube,
        &placed(
            front.add(back.scale(size.z / 2)).add(v3(0, size.y / 2, 0)),
            yaw,
            size,
        ),
        [ONE / 4, ONE / 2, ONE],
        0,
    );
}

/// The keys of the gallery's grocery models (`?gallery=<key>`).
pub const GALLERY: [&str; 0] = [];
