//! Clean-Up on Aisle Ape's Floor is Lava cat, the gummy jaguar
//! (`GUMMY_JAGUAR`, GR-Q11): raspberry gummy, glossy (`gummy_cat`). The
//! map's apes wore food costumes here until GR-T1 put them in the store's
//! uniforms (`models::grocer`); only the cat is left.

use super::*;

/// the gummy jaguar
pub(super) const GUMMY_COAT: Rgb = rgb(214, 30, 58);
pub(super) const GUMMY_DEEP: Rgb = rgb(168, 16, 42);
pub(super) const GUMMY_CREAM: Rgb = rgb(246, 110, 130);
pub(super) const GUMMY_SPOT: Rgb = rgb(116, 8, 28);
pub(super) const GUMMY_EYE: Rgb = rgb(255, 236, 140);
pub(super) const GUMMY_SHINE: Rgb = rgb(255, 214, 224);
pub(super) const GUMMY_GLOSS: Rgb = rgb(255, 150, 170);

/// A soft glow over `a` (a unit ball's frame), drawn last and added over
/// the shape (`Batch::push_halo`): a strength from 2 to 3 is glass, a
/// crisp bright rim (the gloss); up to 1 a light from inside (the gummy's
/// translucence).
fn sheen(rig: &mut Rig, a: Affine, c: Rgb, strength: i32) {
    let a = rig.base.then(&a);
    rig.b.push_halo(Mesh::Sphere, &a, c, strength);
}

/// The gummy jaguar's gloss over its raspberry body (`GUMMY_JAGUAR`): a
/// light from inside, a glass rim and two pale highlights along its back.
pub(super) fn gummy_cat(rig: &mut Rig, body: &Frame) {
    let p = |x: i32, y: i32, z: i32| v3(cm(x), cm(y), cm(z));
    sheen(
        rig,
        body.shape(p(0, 0, 0), p(50, 122, 54)),
        GUMMY_GLOSS,
        GLASS + ONE * 3 / 5,
    );
    sheen(
        rig,
        body.shape(p(0, 0, 0), p(40, 104, 44)),
        GUMMY_COAT,
        ONE * 2 / 5,
    );
    for (along, x, len) in [(24, -8, 40), (-30, 8, 30)] {
        rig.glow(
            Mesh::Geo,
            body.shape(p(x, along, 24), p(8, len, 4)),
            GUMMY_SHINE,
            ONE * 2 / 3,
        );
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::super::tests::Shape;
    use super::*;
    use crate::render::{DYNAMIC_BUFFER, INSTANCE_VALUES};

    /// A batch's shapes (halos aside), as `models::tests` reads them.
    pub(in crate::models) fn read(b: Batch) -> Vec<Shape> {
        let (values, draws) = b.finish(DYNAMIC_BUFFER);
        let mut out = Vec::new();
        for d in draws.chunks(crate::render::DRAW_VALUES) {
            let first = d[2] as usize / 4 / INSTANCE_VALUES;
            for i in first..first + d[3] as usize {
                let v = &values[i * INSTANCE_VALUES..(i + 1) * INSTANCE_VALUES];
                out.push(Shape {
                    mesh: d[1],
                    axes: [
                        v3(v[0], v[4], v[8]),
                        v3(v[1], v[5], v[9]),
                        v3(v[2], v[6], v[10]),
                    ],
                    at: v3(v[3], v[7], v[11]),
                    colour: [v[12], v[13], v[14]],
                    glow: v[15],
                });
            }
        }
        out
    }

    #[test]
    fn only_the_grocerys_cat_is_the_gummy_jaguar() {
        use crate::map::Map;
        for map in Map::ALL {
            assert_eq!(BigCat::on(map).gummy, map.key() == "grocery", "{map:?}");
            assert_eq!(BigCat::on(map) == GUMMY_JAGUAR, map.key() == "grocery");
        }
        const { assert!(GUMMY_JAGUAR.gummy && !JAGUAR.gummy && !SPIDER_JAGUAR.gummy) };
        let j = crate::jaguar::Jaguar {
            feet: V3::ZERO,
            yaw: deg(30),
            gait: deg(50),
            stride: ONE,
            leap: 0,
            head_down: 0,
            jaw: ONE / 2,
        };
        let draw = |cat: &BigCat| {
            let mut b = Batch::new();
            jaguar(&mut b, &j, 100, cat);
            let halos = b.take_halos().count();
            (read(b), halos)
        };
        let (s, halos) = draw(&GUMMY_JAGUAR);
        // still a jaguar, in gummy: its coat and rosettes red, glossy
        assert!(s.iter().any(|x| x.colour == GUMMY_COAT));
        assert!(s.iter().any(|x| x.colour == GUMMY_SPOT));
        assert!(!s.iter().any(|x| x.colour == JAG_COAT));
        assert_eq!(s.iter().filter(|x| x.colour == GUMMY_SHINE).count(), 2);
        assert_eq!(halos, 2);
        let (plain, plain_halos) = draw(&JAGUAR);
        assert_eq!(plain.len() + 2, s.len());
        assert_eq!(plain_halos, 0);
        for other in [
            JAGUAR,
            SPACE_JAGUAR,
            RUINS_JAGUAR,
            NAVY_JAGUAR,
            SNOW_LEOPARD,
            FLAME_CAT,
            SPIDER_JAGUAR,
        ] {
            let (o, _) = draw(&other);
            assert!(!o.iter().any(|x| x.colour == GUMMY_SHINE), "{other:?}");
        }
    }
}
