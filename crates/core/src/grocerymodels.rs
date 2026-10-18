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
//! The builders live in `products.rs` (the 12 shapes and 8 fronts),
//! `people.rs` (shoppers and carts), `props.rs` (balloons, the ball bin,
//! produce tables, display bins, misters, the stock cart, the pallet) and
//! `signs.rs` (aisle signs, sale banners, department pictures), and
//! `outside.rs` (the second pass, GR-T1–T8: the banana logo, the shopping
//! list's items, the parking lot, the plane, the drop ceiling, the ice, the
//! odd vines' fittings); the model gallery's keys are in `gallery.rs`.

pub mod catalogue;
mod gallery;
pub mod outside;
mod people;
mod products;
mod props;
mod signs;
#[cfg(test)]
mod tests;

pub use catalogue::{Aisle, AISLES};
pub use gallery::{framing, gallery, gallery_look, GALLERY};
pub use outside::{
    balloon_weight, banana_logo, canopy, car, corral, doors, ice_wall, lamp_post, list_item,
    low_ceiling, order_bag, plane, sign_arm,
};

use crate::fx::{self, cm, QUARTER};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{rgb, rod, Batch, Rgb};

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

impl Shape {
    pub const ALL: [Shape; 12] = [
        Shape::Box,
        Shape::Bag,
        Shape::Can,
        Shape::Bottle,
        Shape::Jar,
        Shape::Tub,
        Shape::Jug,
        Shape::Case,
        Shape::Canister,
        Shape::Pouch,
        Shape::Tray,
        Shape::Sack,
    ];
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

impl Front {
    pub const ALL: [Front; 8] = [
        Front::Band,
        Front::Stripe,
        Front::Split,
        Front::Dots,
        Front::Window,
        Front::Chevron,
        Front::Lid,
        Front::Plain,
    ];
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
    products::product(b, p, foot, yaw, detail);
}

/// A shopper standing at `feet`, facing `yaw`, walking (`stride` 0..ONE
/// round one step cycle; 0 standing). `look` (0..7) picks their clothes,
/// skin and hair. With `string`, whichever hand is nearer the end is raised
/// and holds the balloon string there (a balloon shopper walking back has
/// it on their left); with `cart`, they push a shopping cart ahead of them,
/// both hands on it (but a string hand). 20.4 m tall (`grocery::SHOPPER_HEIGHT`).
pub fn shopper(
    b: &mut Batch,
    feet: V3,
    yaw: i32,
    look: u8,
    stride: i32,
    string: Option<V3>,
    cart: bool,
) {
    people::shopper(b, feet, yaw, look, stride, string, cart);
}

/// A bunch of helium balloons whose strings meet at `anchor` (the vine's
/// anchor: their knot, under the balloons), bobbing with `tick`; `k` picks
/// the bunch's colours.
pub fn balloons(b: &mut Batch, anchor: V3, tick: u32, k: u32) {
    props::balloons(b, anchor, tick, k);
}

/// A bin of bouncy balls (a bounce bouncer): its top's middle at `top`,
/// `radius` round. It stands on the floor (y 0), so it is `top.y` tall.
pub fn ball_bin(b: &mut Batch, top: V3, radius: i32) {
    props::ball_bin(b, top, radius);
}

/// A produce table's pile (`stock`: 0 apples, 1 oranges, 2 bananas, 3
/// melons, 4 lettuce, 5 tomatoes, 6 peppers) on a dark wooden round table
/// `radius` round, `height` tall, standing on `foot`.
pub fn table(b: &mut Batch, foot: V3, radius: i32, height: i32, stock: u8) {
    props::table(b, foot, radius, height, stock);
}

/// A kraft cardboard display bin full of product `stock` (an aisle of
/// `AISLES`), `radius` round, `height` tall, on `foot`.
pub fn bin(b: &mut Batch, foot: V3, radius: i32, height: i32, stock: u8) {
    props::bin(b, foot, radius, height, stock);
}

/// A mister's puff of spray from `at` (a nozzle over a produce table), a
/// soft white burst that grows and fades over the first third of each
/// `period` ticks (at most 1.5 s), `phase` in; nothing between puffs.
/// Glows (halos only).
pub fn mister(b: &mut Batch, at: V3, tick: u32, period: u32, phase: u32) {
    props::mister(b, at, tick, period, phase);
}

/// A navy aisle sign hanging from the ceiling at `at` (its middle), facing
/// `yaw` both ways, showing `number` in white digits built from shapes.
pub fn aisle_sign(b: &mut Batch, at: V3, yaw: i32, number: u32) {
    signs::aisle_sign(b, at, yaw, number);
}

/// A red-and-yellow sale banner hanging at `at`, facing `yaw`; `k` picks
/// its price digits.
pub fn banner(b: &mut Batch, at: V3, yaw: i32, k: u32) {
    signs::banner(b, at, yaw, k);
}

/// A department's sign: its picture (`dept`: 0 dairy milk bottle and cheese,
/// 1 meat steak and fish, 2 deli, 3 bakery bread loaf, 4 produce carrot and
/// apple, 5 frozen snowflake) on the soffit at `at`, facing `yaw`, `size`
/// across.
pub fn dept_sign(b: &mut Batch, at: V3, yaw: i32, dept: u8, size: i32) {
    signs::dept_sign(b, at, yaw, dept, size);
}

/// A shopping cart on `foot` (its middle), facing `yaw`.
pub fn cart(b: &mut Batch, foot: V3, yaw: i32) {
    people::cart(b, foot, yaw);
}

/// The start's stock cart (a U-boat) loaded with shipping cases: `centre`
/// on the floor, the way along it `yaw`, `size` (across, the load's top,
/// along): its top is the start platform.
pub fn stock_cart(b: &mut Batch, centre: V3, yaw: i32, size: V3) {
    props::stock_cart(b, centre, yaw, size);
}

/// A pallet display of water and soda cases filling exactly the box the
/// Ravine's sphinx is solid as (`ravinemodels::SPHINX_SIZE`): its front
/// (the sphinx's front feet's middle) at `front` on the floor, facing
/// `yaw`, its body behind.
pub fn pallet(b: &mut Batch, front: V3, yaw: i32) {
    props::pallet(b, front, yaw);
}

// ---------------------------------------------------------------------------
// The palette (the reference photos: warm neutrals; the colour comes from
// the products, banners and signs).

pub const CREAM: Rgb = rgb(236, 232, 220);
pub const PRICE_YELLOW: Rgb = rgb(243, 210, 43);
pub const NAVY: Rgb = rgb(31, 44, 92);
pub const WHITE: Rgb = rgb(246, 246, 242);
pub const BANNER_RED: Rgb = rgb(215, 50, 43);
pub const BANNER_YELLOW: Rgb = rgb(245, 209, 30);
pub const BOARD: Rgb = rgb(42, 38, 38);
pub const KRAFT: Rgb = rgb(184, 149, 106);
pub const HANDLE: Rgb = rgb(28, 27, 28);
pub const WOOD: Rgb = rgb(75, 58, 45);
pub const CHROME: Rgb = rgb(206, 211, 216);
pub const WIRE: Rgb = rgb(150, 156, 162);
pub const STEEL: Rgb = rgb(124, 128, 132);
pub const ROD: Rgb = rgb(160, 160, 158);
pub const PICTURE: Rgb = rgb(61, 43, 31);
pub const PANEL: Rgb = rgb(240, 234, 216);
pub const INK: Rgb = rgb(40, 30, 30);

/// Birthday balloons and bouncy balls.
pub const PARTY: [Rgb; 8] = [
    rgb(226, 40, 52),
    rgb(40, 110, 220),
    rgb(250, 206, 40),
    rgb(60, 180, 80),
    rgb(240, 100, 170),
    rgb(140, 80, 200),
    rgb(250, 140, 40),
    rgb(80, 200, 220),
];

// ---------------------------------------------------------------------------
// Helpers.

/// A deterministic mix of an index (no randomness in the core).
pub(crate) fn hash(n: u32) -> u32 {
    let mut x = n.wrapping_mul(0x9E37_79B1) ^ 0x5bd1_e995;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2c1b_3c6d);
    x ^= x >> 12;
    x
}

/// `a × num / den`, for proportions.
pub(crate) const fn part(a: i32, num: i32, den: i32) -> i32 {
    ((a as i64 * num as i64) / den as i64) as i32
}

/// A model's own frame: its origin `o`, turned `yaw`: x across (to the
/// right of someone looking at its front), y up, z out of its front.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Fr {
    pub o: V3,
    pub rot: Affine,
}

impl Fr {
    pub fn new(o: V3, yaw: i32) -> Fr {
        Fr {
            o,
            rot: Affine::rot_y(yaw),
        }
    }

    /// The world point of local `l`.
    pub fn pt(&self, l: V3) -> V3 {
        self.o.add(self.rot.apply_dir(l))
    }

    /// The transform of a unit shape at local `at`, turned by `turn`
    /// (local), scaled to `size`.
    pub fn aff(&self, at: V3, turn: &Affine, size: V3) -> Affine {
        Affine::translate(self.pt(at))
            .then(&self.rot)
            .then(turn)
            .then(&Affine::scale(size))
    }

    pub fn put(&self, b: &mut Batch, m: Mesh, at: V3, size: V3, c: Rgb, glow: i32) {
        b.push(m, &self.aff(at, &Affine::IDENTITY, size), c, glow);
    }

    pub fn turned(
        &self,
        b: &mut Batch,
        m: Mesh,
        at: V3,
        turn: Affine,
        size: V3,
        c: Rgb,
        glow: i32,
    ) {
        b.push(m, &self.aff(at, &turn, size), c, glow);
    }

    /// A rod `w` thick from local `a` to local `z`.
    pub fn rod(&self, b: &mut Batch, m: Mesh, a: V3, z: V3, w: i32, c: Rgb, glow: i32) {
        b.push(m, &rod(self.pt(a), self.pt(z), w), c, glow);
    }

    /// A frame at local `at` turned a further `yaw` about its up.
    pub fn sub(&self, at: V3, yaw: i32) -> Fr {
        Fr {
            o: self.pt(at),
            rot: self.rot.then(&Affine::rot_y(yaw)),
        }
    }
}

/// How far print stands proud of the face it is printed on.
pub(crate) const PROUD: i32 = cm(3);

/// A flat face to print on: `a` is its frame (the face's middle, x across
/// it, y up it, z out of it), `w` by `h`; `sink` is how far the print must
/// reach back into the body (a curved face falls away at its sides).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Face {
    pub a: Affine,
    pub w: i32,
    pub h: i32,
    pub sink: i32,
}

impl Face {
    /// The front face of a body in frame `fr`: middle at local (0, y, z).
    pub fn front(fr: &Fr, y: i32, z: i32, w: i32, h: i32, sink: i32) -> Face {
        Face {
            a: Affine::translate(fr.pt(v3(0, y, z))).then(&fr.rot),
            w,
            h,
            sink,
        }
    }

    /// A top face at local height `y`, its middle `z` along: its "up" runs
    /// toward the back, so it reads from the front.
    pub fn top(fr: &Fr, y: i32, z: i32, w: i32, h: i32) -> Face {
        Face {
            a: Affine::translate(fr.pt(v3(0, y, z)))
                .then(&fr.rot)
                .then(&Affine::rot_x(-QUARTER)),
            w,
            h,
            sink: cm(3),
        }
    }

    fn depth(&self, layer: i32) -> (i32, i32) {
        let out = PROUD + layer * cm(1);
        (self.sink + out, (out - self.sink) / 2)
    }

    /// A shape `m` at (u, v) on the face, `su` by `sv`, turned `angle` in
    /// the face's plane; `layer` 1 or 2 stands it over layer 0's print.
    #[allow(clippy::too_many_arguments)]
    pub fn mark(
        &self,
        b: &mut Batch,
        m: Mesh,
        uv: (i32, i32),
        su: i32,
        sv: i32,
        angle: i32,
        c: Rgb,
        layer: i32,
    ) {
        let (d, zc) = self.depth(layer);
        let a = self
            .a
            .then(&Affine::translate(v3(uv.0, uv.1, zc)))
            .then(&Affine::rot_z(angle))
            .then(&Affine::scale(v3(su, sv, d)));
        b.push(m, &a, c, 0);
    }

    /// A flat slab of print.
    pub fn slab(&self, b: &mut Batch, uv: (i32, i32), su: i32, sv: i32, c: Rgb, layer: i32) {
        self.mark(b, Mesh::Cube, uv, su, sv, 0, c, layer);
    }

    /// A round dot of print, `dia` across (a cylinder facing out).
    pub fn dot(&self, b: &mut Batch, uv: (i32, i32), dia: i32, c: Rgb, layer: i32) {
        let (d, zc) = self.depth(layer);
        let a = self
            .a
            .then(&Affine::translate(v3(uv.0, uv.1, zc)))
            .then(&Affine::rot_x(QUARTER))
            .then(&Affine::scale(v3(dia, d, dia)));
        b.push(Mesh::Cyl, &a, c, 0);
    }

    /// A bar of print from `p` to `q` on the face, `thick` wide.
    pub fn bar(&self, b: &mut Batch, p: (i32, i32), q: (i32, i32), thick: i32, c: Rgb, layer: i32) {
        let (dx, dy) = (q.0 - p.0, q.1 - p.1);
        let len = v3(dx, dy, 0).len();
        let mid = (p.0 + dx / 2, p.1 + dy / 2);
        self.mark(
            b,
            Mesh::Cube,
            mid,
            len + thick / 2,
            thick,
            fx::atan2(dy, dx),
            c,
            layer,
        );
    }
}

/// The segments a seven-segment digit lights: a b c d e f g.
const SEGMENTS: [u8; 10] = [
    0b011_1111, 0b000_0110, 0b101_1011, 0b100_1111, 0b110_0110, 0b110_1101, 0b111_1101, 0b000_0111,
    0b111_1111, 0b110_1111,
];

/// Digit `n` (0..9) in bars on `face`, its middle at `c`, `dw` by `dh`,
/// its bars `t` thick. Returns the shapes drawn.
#[allow(clippy::too_many_arguments)]
pub(crate) fn digit(
    b: &mut Batch,
    face: &Face,
    c: (i32, i32),
    dw: i32,
    dh: i32,
    t: i32,
    n: u32,
    colour: Rgb,
    layer: i32,
) {
    let mask = SEGMENTS[(n % 10) as usize];
    let (x, y) = c;
    let side = dw / 2 - t / 2;
    let tall = dh / 2 - t / 2;
    // a, b, c, d, e, f, g: (middle, horizontal)
    let segs: [((i32, i32), bool); 7] = [
        ((x, y + tall), true),
        ((x + side, y + dh / 4), false),
        ((x + side, y - dh / 4), false),
        ((x, y - tall), true),
        ((x - side, y - dh / 4), false),
        ((x - side, y + dh / 4), false),
        ((x, y), true),
    ];
    for (i, (at, flat)) in segs.into_iter().enumerate() {
        if mask & (1 << i) == 0 {
            continue;
        }
        if flat {
            face.slab(b, at, dw, t, colour, layer);
        } else {
            face.slab(b, at, t, dh / 2 + t / 2, colour, layer);
        }
    }
}
