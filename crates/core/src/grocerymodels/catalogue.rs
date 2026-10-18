//! The store's products, aisle by aisle (the brief: "every aisle should
//! have 5-10 uniquely sized, colored, and textured boxes ... repeated /
//! ordered to mimic the real inside of a grocery store"): 94 products in
//! 14 aisles, each with its own shape, size in real centimetres, three
//! colours and a front. The shelves (`groceryscape`) fill them by the
//! planogram rule. The journal's catalogue figure is drawn from the same
//! table.

use super::{Front, Product, Shape};
use crate::render::rgb;

/// One aisle's products: its name (for the docs and the tests; never shown)
/// and its products.
pub struct Aisle {
    pub name: &'static str,
    pub products: &'static [Product],
}

const fn p(
    name: &'static str,
    shape: Shape,
    size: [i32; 3],
    colours: [crate::render::Rgb; 3],
    front: Front,
) -> Product {
    Product {
        name,
        shape,
        size,
        colours,
        front,
    }
}

/// The fourteen aisles, west to east (`grocery::layout::RUNS` names them
/// by index for each run's faces).
pub const AISLES: [Aisle; 14] = [
    Aisle {
        name: "Bread and tortillas",
        products: &[
            p(
                "white loaf",
                Shape::Bag,
                [12, 12, 28],
                [rgb(244, 241, 232), rgb(43, 108, 196), rgb(232, 194, 58)],
                Front::Dots,
            ),
            p(
                "wheat loaf",
                Shape::Bag,
                [12, 12, 28],
                [rgb(199, 154, 94), rgb(46, 125, 58), rgb(244, 241, 232)],
                Front::Band,
            ),
            p(
                "hamburger buns",
                Shape::Bag,
                [22, 9, 22],
                [rgb(242, 226, 192), rgb(212, 58, 44), rgb(244, 241, 232)],
                Front::Window,
            ),
            p(
                "tortillas",
                Shape::Pouch,
                [22, 3, 22],
                [rgb(245, 239, 226), rgb(47, 154, 74), rgb(212, 58, 44)],
                Front::Band,
            ),
            p(
                "bagels",
                Shape::Bag,
                [16, 10, 24],
                [rgb(217, 163, 90), rgb(31, 79, 154), rgb(244, 241, 232)],
                Front::Window,
            ),
            p(
                "english muffins",
                Shape::Box,
                [16, 7, 16],
                [rgb(240, 230, 207), rgb(194, 42, 42), rgb(43, 95, 174)],
                Front::Stripe,
            ),
        ],
    },
    Aisle {
        name: "Canned goods and soup",
        products: &[
            p(
                "red soup can",
                Shape::Can,
                [7, 10, 7],
                [rgb(200, 38, 44), rgb(244, 241, 232), rgb(216, 178, 58)],
                Front::Split,
            ),
            p(
                "green beans",
                Shape::Can,
                [7, 11, 7],
                [rgb(47, 138, 62), rgb(244, 241, 232), rgb(233, 210, 74)],
                Front::Band,
            ),
            p(
                "corn",
                Shape::Can,
                [7, 11, 7],
                [rgb(240, 198, 46), rgb(47, 138, 62), rgb(244, 241, 232)],
                Front::Band,
            ),
            p(
                "tomatoes",
                Shape::Can,
                [10, 12, 10],
                [rgb(216, 56, 42), rgb(59, 138, 58), rgb(244, 241, 232)],
                Front::Band,
            ),
            p(
                "chili",
                Shape::Can,
                [8, 11, 8],
                [rgb(122, 42, 30), rgb(233, 178, 58), rgb(244, 241, 232)],
                Front::Chevron,
            ),
            p(
                "broth carton",
                Shape::Box,
                [9, 19, 6],
                [rgb(233, 178, 58), rgb(47, 95, 168), rgb(244, 241, 232)],
                Front::Band,
            ),
            p(
                "tuna",
                Shape::Can,
                [8, 4, 8],
                [rgb(43, 95, 174), rgb(244, 241, 232), rgb(216, 56, 42)],
                Front::Band,
            ),
        ],
    },
    Aisle {
        name: "Pasta, rice and sauce",
        products: &[
            p(
                "spaghetti",
                Shape::Box,
                [6, 26, 4],
                [rgb(31, 79, 154), rgb(244, 241, 232), rgb(216, 56, 42)],
                Front::Window,
            ),
            p(
                "elbow macaroni",
                Shape::Box,
                [12, 18, 5],
                [rgb(43, 108, 196), rgb(240, 198, 46), rgb(244, 241, 232)],
                Front::Window,
            ),
            p(
                "pasta sauce",
                Shape::Jar,
                [9, 17, 9],
                [rgb(163, 38, 28), rgb(244, 239, 226), rgb(47, 138, 62)],
                Front::Lid,
            ),
            p(
                "alfredo",
                Shape::Jar,
                [9, 15, 9],
                [rgb(242, 234, 216), rgb(31, 79, 154), rgb(216, 178, 58)],
                Front::Lid,
            ),
            p(
                "long-grain rice",
                Shape::Sack,
                [14, 22, 7],
                [rgb(244, 241, 232), rgb(216, 56, 42), rgb(47, 138, 62)],
                Front::Band,
            ),
            p(
                "mac and cheese",
                Shape::Box,
                [15, 9, 4],
                [rgb(246, 162, 30), rgb(43, 95, 174), rgb(244, 241, 232)],
                Front::Split,
            ),
            p(
                "ramen",
                Shape::Pouch,
                [11, 3, 15],
                [rgb(227, 58, 42), rgb(240, 198, 46), rgb(28, 27, 28)],
                Front::Stripe,
            ),
        ],
    },
    Aisle {
        name: "Cereal and breakfast",
        products: &[
            p(
                "corn flakes",
                Shape::Box,
                [20, 30, 7],
                [rgb(232, 69, 44), rgb(240, 198, 46), rgb(46, 125, 58)],
                Front::Split,
            ),
            p(
                "honey o's",
                Shape::Box,
                [19, 28, 7],
                [rgb(240, 198, 46), rgb(43, 95, 174), rgb(232, 69, 44)],
                Front::Band,
            ),
            p(
                "family crunch",
                Shape::Box,
                [24, 34, 8],
                [rgb(43, 95, 174), rgb(244, 241, 232), rgb(232, 69, 44)],
                Front::Chevron,
            ),
            p(
                "frosted squares",
                Shape::Box,
                [19, 29, 7],
                [rgb(122, 78, 160), rgb(240, 198, 46), rgb(244, 241, 232)],
                Front::Dots,
            ),
            p(
                "oatmeal canister",
                Shape::Canister,
                [13, 24, 13],
                [rgb(31, 63, 122), rgb(216, 56, 42), rgb(244, 241, 232)],
                Front::Band,
            ),
            p(
                "pancake mix",
                Shape::Box,
                [17, 23, 7],
                [rgb(246, 198, 58), rgb(43, 95, 174), rgb(138, 74, 32)],
                Front::Stripe,
            ),
            p(
                "maple syrup",
                Shape::Bottle,
                [8, 22, 8],
                [rgb(122, 58, 20), rgb(244, 239, 226), rgb(200, 38, 44)],
                Front::Lid,
            ),
            p(
                "ground coffee",
                Shape::Can,
                [15, 17, 15],
                [rgb(200, 38, 44), rgb(28, 27, 28), rgb(232, 194, 58)],
                Front::Band,
            ),
            p(
                "toaster pastries",
                Shape::Box,
                [22, 14, 7],
                [rgb(58, 122, 214), rgb(244, 241, 232), rgb(232, 69, 44)],
                Front::Window,
            ),
            p(
                "granola bars",
                Shape::Box,
                [18, 13, 6],
                [rgb(47, 138, 62), rgb(240, 198, 46), rgb(138, 74, 32)],
                Front::Stripe,
            ),
        ],
    },
    Aisle {
        name: "Baking and spices",
        products: &[
            p(
                "flour",
                Shape::Sack,
                [18, 28, 10],
                [rgb(244, 241, 232), rgb(216, 56, 42), rgb(43, 95, 174)],
                Front::Band,
            ),
            p(
                "sugar",
                Shape::Sack,
                [15, 24, 9],
                [rgb(244, 241, 232), rgb(43, 95, 174), rgb(232, 194, 58)],
                Front::Stripe,
            ),
            p(
                "cake mix",
                Shape::Box,
                [15, 22, 5],
                [rgb(232, 90, 160), rgb(244, 241, 232), rgb(122, 58, 20)],
                Front::Window,
            ),
            p(
                "chocolate chips",
                Shape::Bag,
                [14, 20, 5],
                [rgb(90, 46, 26), rgb(240, 198, 46), rgb(244, 241, 232)],
                Front::Split,
            ),
            p(
                "vegetable oil",
                Shape::Jug,
                [12, 30, 9],
                [rgb(242, 210, 74), rgb(47, 138, 62), rgb(216, 56, 42)],
                Front::Lid,
            ),
            p(
                "spice jars",
                Shape::Jar,
                [5, 11, 5],
                [rgb(200, 38, 44), rgb(244, 241, 232), rgb(28, 27, 28)],
                Front::Lid,
            ),
        ],
    },
    Aisle {
        name: "Coffee, tea and juice",
        products: &[
            p(
                "orange juice",
                Shape::Jug,
                [11, 26, 11],
                [rgb(246, 154, 30), rgb(244, 241, 232), rgb(47, 138, 62)],
                Front::Lid,
            ),
            p(
                "apple juice",
                Shape::Bottle,
                [10, 28, 10],
                [rgb(233, 178, 58), rgb(200, 38, 44), rgb(47, 138, 62)],
                Front::Band,
            ),
            p(
                "tea bags",
                Shape::Box,
                [15, 10, 8],
                [rgb(47, 138, 62), rgb(240, 198, 46), rgb(244, 241, 232)],
                Front::Split,
            ),
            p(
                "coffee bag",
                Shape::Bag,
                [12, 22, 7],
                [rgb(58, 36, 24), rgb(232, 194, 58), rgb(200, 38, 44)],
                Front::Stripe,
            ),
            p(
                "cocoa canister",
                Shape::Canister,
                [11, 16, 11],
                [rgb(90, 46, 26), rgb(244, 241, 232), rgb(200, 38, 44)],
                Front::Band,
            ),
            p(
                "juice boxes",
                Shape::Case,
                [20, 12, 15],
                [rgb(122, 78, 160), rgb(240, 198, 46), rgb(58, 122, 214)],
                Front::Dots,
            ),
        ],
    },
    Aisle {
        name: "Frozen",
        products: &[
            p(
                "vanilla ice cream",
                Shape::Tub,
                [16, 12, 16],
                [rgb(246, 237, 212), rgb(122, 58, 20), rgb(58, 122, 214)],
                Front::Lid,
            ),
            p(
                "chocolate ice cream",
                Shape::Tub,
                [16, 12, 16],
                [rgb(90, 46, 26), rgb(246, 237, 212), rgb(232, 90, 160)],
                Front::Lid,
            ),
            p(
                "pizza",
                Shape::Box,
                [30, 5, 30],
                [rgb(200, 38, 44), rgb(240, 198, 46), rgb(47, 138, 62)],
                Front::Window,
            ),
            p(
                "frozen peas",
                Shape::Bag,
                [18, 24, 5],
                [rgb(47, 154, 74), rgb(244, 241, 232), rgb(240, 198, 46)],
                Front::Window,
            ),
            p(
                "dinner tray",
                Shape::Box,
                [22, 5, 16],
                [rgb(31, 63, 122), rgb(232, 69, 44), rgb(244, 241, 232)],
                Front::Window,
            ),
            p(
                "waffles",
                Shape::Box,
                [20, 17, 8],
                [rgb(240, 198, 46), rgb(43, 95, 174), rgb(200, 38, 44)],
                Front::Chevron,
            ),
            p(
                "ice pops",
                Shape::Box,
                [24, 9, 12],
                [rgb(58, 122, 214), rgb(232, 90, 160), rgb(240, 198, 46)],
                Front::Dots,
            ),
            p(
                "crinkle fries",
                Shape::Bag,
                [20, 28, 6],
                [rgb(216, 56, 42), rgb(240, 198, 46), rgb(244, 241, 232)],
                Front::Stripe,
            ),
        ],
    },
    Aisle {
        name: "Frozen breakfast and meals",
        products: &[
            p(
                "breakfast sandwiches",
                Shape::Box,
                [20, 6, 14],
                [rgb(232, 69, 44), rgb(244, 241, 232), rgb(240, 198, 46)],
                Front::Window,
            ),
            p(
                "pot pie",
                Shape::Box,
                [14, 6, 14],
                [rgb(43, 95, 174), rgb(240, 198, 46), rgb(138, 74, 32)],
                Front::Window,
            ),
            p(
                "fish sticks",
                Shape::Box,
                [22, 8, 14],
                [rgb(31, 111, 176), rgb(244, 241, 232), rgb(240, 198, 46)],
                Front::Band,
            ),
            p(
                "burritos",
                Shape::Pouch,
                [16, 4, 20],
                [rgb(47, 138, 62), rgb(232, 69, 44), rgb(240, 198, 46)],
                Front::Stripe,
            ),
            p(
                "ice bag",
                Shape::Sack,
                [30, 40, 12],
                [rgb(230, 242, 250), rgb(43, 108, 196), rgb(244, 241, 232)],
                Front::Band,
            ),
        ],
    },
    Aisle {
        name: "Soda and water",
        products: &[
            p(
                "cola 12-pack",
                Shape::Case,
                [40, 13, 27],
                [rgb(200, 38, 44), rgb(244, 241, 232), rgb(28, 27, 28)],
                Front::Band,
            ),
            p(
                "lemon-lime 12-pack",
                Shape::Case,
                [40, 13, 27],
                [rgb(47, 154, 74), rgb(240, 198, 46), rgb(244, 241, 232)],
                Front::Band,
            ),
            p(
                "water case",
                Shape::Case,
                [40, 22, 27],
                [rgb(58, 122, 214), rgb(230, 242, 250), rgb(244, 241, 232)],
                Front::Stripe,
            ),
            p(
                "2-litre orange",
                Shape::Bottle,
                [11, 33, 11],
                [rgb(246, 154, 30), rgb(244, 241, 232), rgb(31, 79, 154)],
                Front::Lid,
            ),
            p(
                "sport drink",
                Shape::Bottle,
                [8, 24, 8],
                [rgb(74, 184, 232), rgb(28, 27, 28), rgb(240, 198, 46)],
                Front::Band,
            ),
            p(
                "gallon water",
                Shape::Jug,
                [15, 30, 15],
                [rgb(207, 230, 242), rgb(43, 108, 196), rgb(244, 241, 232)],
                Front::Lid,
            ),
        ],
    },
    Aisle {
        name: "Snacks, cookies and crackers",
        products: &[
            p(
                "potato chips",
                Shape::Bag,
                [26, 36, 9],
                [rgb(240, 198, 46), rgb(216, 56, 42), rgb(244, 241, 232)],
                Front::Split,
            ),
            p(
                "sour cream chips",
                Shape::Bag,
                [26, 36, 9],
                [rgb(47, 138, 62), rgb(244, 241, 232), rgb(31, 79, 154)],
                Front::Split,
            ),
            p(
                "tortilla chips",
                Shape::Bag,
                [28, 38, 9],
                [rgb(122, 78, 160), rgb(240, 198, 46), rgb(232, 69, 44)],
                Front::Chevron,
            ),
            p(
                "pretzels",
                Shape::Bag,
                [22, 30, 8],
                [rgb(31, 79, 154), rgb(216, 163, 90), rgb(244, 241, 232)],
                Front::Window,
            ),
            p(
                "popcorn",
                Shape::Box,
                [17, 15, 9],
                [rgb(200, 38, 44), rgb(244, 241, 232), rgb(240, 198, 46)],
                Front::Stripe,
            ),
            p(
                "sandwich cookies",
                Shape::Pouch,
                [20, 6, 8],
                [rgb(31, 79, 154), rgb(244, 241, 232), rgb(28, 27, 28)],
                Front::Band,
            ),
            p(
                "saltine crackers",
                Shape::Box,
                [13, 20, 8],
                [rgb(216, 56, 42), rgb(244, 241, 232), rgb(43, 95, 174)],
                Front::Band,
            ),
            p(
                "cheese crackers",
                Shape::Box,
                [18, 20, 6],
                [rgb(246, 130, 30), rgb(200, 38, 44), rgb(244, 241, 232)],
                Front::Dots,
            ),
            p(
                "peanut butter cups",
                Shape::Bag,
                [18, 24, 6],
                [rgb(246, 130, 30), rgb(58, 36, 24), rgb(240, 198, 46)],
                Front::Split,
            ),
            p(
                "mixed nuts",
                Shape::Canister,
                [11, 13, 11],
                [rgb(43, 95, 174), rgb(216, 163, 90), rgb(200, 38, 44)],
                Front::Lid,
            ),
        ],
    },
    Aisle {
        name: "Candy and nuts",
        products: &[
            p(
                "gummy bears",
                Shape::Bag,
                [15, 20, 5],
                [rgb(232, 90, 160), rgb(240, 198, 46), rgb(58, 122, 214)],
                Front::Dots,
            ),
            p(
                "chocolate bars",
                Shape::Box,
                [15, 2, 6],
                [rgb(90, 46, 26), rgb(244, 241, 232), rgb(200, 38, 44)],
                Front::Band,
            ),
            p(
                "licorice",
                Shape::Bag,
                [12, 26, 4],
                [rgb(200, 38, 44), rgb(244, 241, 232), rgb(28, 27, 28)],
                Front::Window,
            ),
            p(
                "peanuts",
                Shape::Jar,
                [11, 15, 11],
                [rgb(216, 163, 90), rgb(31, 79, 154), rgb(200, 38, 44)],
                Front::Lid,
            ),
            p(
                "mints",
                Shape::Tray,
                [6, 2, 9],
                [rgb(244, 241, 232), rgb(47, 138, 62), rgb(58, 122, 214)],
                Front::Stripe,
            ),
            p(
                "trail mix",
                Shape::Bag,
                [18, 22, 6],
                [rgb(138, 74, 32), rgb(240, 198, 46), rgb(47, 138, 62)],
                Front::Window,
            ),
        ],
    },
    Aisle {
        name: "Paper and cleaning",
        products: &[
            p(
                "paper towels",
                Shape::Case,
                [30, 28, 22],
                [rgb(244, 241, 232), rgb(58, 122, 214), rgb(232, 194, 58)],
                Front::Split,
            ),
            p(
                "toilet paper",
                Shape::Case,
                [40, 24, 22],
                [rgb(244, 241, 232), rgb(232, 90, 160), rgb(58, 122, 214)],
                Front::Band,
            ),
            p(
                "laundry jug",
                Shape::Jug,
                [18, 30, 14],
                [rgb(246, 130, 30), rgb(31, 79, 154), rgb(244, 241, 232)],
                Front::Lid,
            ),
            p(
                "dish soap",
                Shape::Bottle,
                [8, 26, 5],
                [rgb(74, 184, 74), rgb(244, 241, 232), rgb(240, 198, 46)],
                Front::Lid,
            ),
            p(
                "bleach",
                Shape::Jug,
                [15, 28, 10],
                [rgb(244, 241, 232), rgb(58, 122, 214), rgb(200, 38, 44)],
                Front::Lid,
            ),
            p(
                "trash bags",
                Shape::Box,
                [24, 14, 11],
                [rgb(28, 27, 28), rgb(232, 194, 58), rgb(244, 241, 232)],
                Front::Stripe,
            ),
        ],
    },
    Aisle {
        name: "Pet and baby",
        products: &[
            p(
                "dog food",
                Shape::Sack,
                [40, 55, 14],
                [rgb(200, 38, 44), rgb(240, 198, 46), rgb(90, 46, 26)],
                Front::Split,
            ),
            p(
                "cat litter",
                Shape::Tub,
                [28, 30, 22],
                [rgb(58, 122, 214), rgb(244, 241, 232), rgb(240, 198, 46)],
                Front::Lid,
            ),
            p(
                "cat food cans",
                Shape::Can,
                [7, 4, 7],
                [rgb(122, 78, 160), rgb(244, 241, 232), rgb(232, 69, 44)],
                Front::Band,
            ),
            p(
                "diapers",
                Shape::Case,
                [34, 30, 18],
                [rgb(230, 242, 250), rgb(58, 122, 214), rgb(240, 198, 46)],
                Front::Window,
            ),
            p(
                "baby wipes",
                Shape::Pouch,
                [20, 8, 11],
                [rgb(244, 241, 232), rgb(74, 184, 232), rgb(47, 138, 62)],
                Front::Band,
            ),
            p(
                "formula",
                Shape::Canister,
                [13, 17, 13],
                [rgb(242, 210, 74), rgb(58, 122, 214), rgb(244, 241, 232)],
                Front::Lid,
            ),
        ],
    },
    Aisle {
        name: "Chips, dips and salsa",
        products: &[
            p(
                "salsa",
                Shape::Jar,
                [9, 13, 9],
                [rgb(200, 38, 44), rgb(240, 198, 46), rgb(47, 138, 62)],
                Front::Lid,
            ),
            p(
                "queso",
                Shape::Jar,
                [9, 11, 9],
                [rgb(246, 162, 30), rgb(28, 27, 28), rgb(200, 38, 44)],
                Front::Lid,
            ),
            p(
                "corn chips",
                Shape::Bag,
                [24, 34, 8],
                [rgb(240, 198, 46), rgb(47, 138, 62), rgb(200, 38, 44)],
                Front::Chevron,
            ),
            p(
                "cheese puffs",
                Shape::Bag,
                [26, 36, 9],
                [rgb(246, 130, 30), rgb(31, 79, 154), rgb(244, 241, 232)],
                Front::Dots,
            ),
            p(
                "pork rinds",
                Shape::Bag,
                [22, 30, 8],
                [rgb(138, 74, 32), rgb(240, 198, 46), rgb(200, 38, 44)],
                Front::Band,
            ),
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The brief: every aisle has five to ten products, each its own.
    #[test]
    fn every_aisle_has_five_to_ten_products_of_its_own() {
        let mut names = Vec::new();
        for a in &AISLES {
            let n = a.products.len();
            assert!((5..=10).contains(&n), "{}: {n} products", a.name);
            for p in a.products {
                assert!(p.size.iter().all(|&c| c > 0), "{}: no size", p.name);
                assert!(!names.contains(&p.name), "{} twice", p.name);
                names.push(p.name);
            }
            // each aisle's products differ from each other in size or colour
            for (i, p) in a.products.iter().enumerate() {
                for q in &a.products[i + 1..] {
                    assert!(
                        p.size != q.size || p.colours != q.colours,
                        "{} and {} look alike",
                        p.name,
                        q.name
                    );
                }
            }
        }
        assert_eq!(names.len(), 94);
    }
}
