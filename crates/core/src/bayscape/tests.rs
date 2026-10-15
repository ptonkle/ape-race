//! Banana Bay Battle's light, sea and squall (step 5, SECOND-ORDER BS rows).

use super::*;
use crate::map::Map;
use crate::render::INSTANCE_VALUES;

/// A camera at race height over the course `s` metres in, looking along it.
fn race_cam(c: &Course, s: i32) -> Camera {
    let p = c.point(int(s));
    let ahead = c.point(int(s + 30));
    let eye = v3(p.x, int(16), p.z);
    Camera {
        eye,
        fwd: v3(ahead.x, int(8), ahead.z).sub(eye).norm(),
        fov: deg(70),
    }
}

fn bay() -> (Course, BayLayout) {
    let c = Map::Bay.course();
    let b = (**c.bay.as_ref().unwrap()).clone();
    (c, b)
}

#[test]
fn the_bay_has_a_3pm_look_of_its_own() {
    let (c, _) = bay();
    let bay = crate::scene::look_of(&c);
    for map in Map::ALL {
        if map != Map::Bay {
            let other = crate::scene::look_of(&map.course());
            assert_ne!(bay.sky, other.sky, "{map:?}'s sky");
            assert_ne!(bay.fog, other.fog, "{map:?}'s fog");
            assert_ne!(bay.light_dir, other.light_dir, "{map:?}'s sun");
        }
    }
    // a blue sky, a turquoise haze, a warm light
    assert!(bay.sky[2] > bay.sky[1] && bay.sky[1] > bay.sky[0]);
    assert!(bay.fog[1] > bay.fog[0] && bay.fog[2] > bay.fog[0]);
    assert!(bay.light[0] > bay.light[2]);
    // the sun high (3 pm) but from the side: neither overhead nor low
    let up = bay.light_dir.y;
    assert!(up > fx::sin(deg(30)) && up < fx::sin(deg(50)), "{up}");
    let side = crate::geom::right_of(c.heading[0]);
    assert!(
        bay.light_dir.dot(side).abs() > ONE / 3,
        "the sun is off to the side of the course"
    );
}

/// The instances `dynamic` draws round the camera.
fn drawn(c: &Course, b: &BayLayout, cam: &Camera, tick: u32, w: Weather) -> Batch {
    let mut batch = Batch::new();
    dynamic(&mut batch, c, b, cam, tick, tick, w);
    batch
}

#[test]
fn the_sea_follows_the_swell_the_physics_uses_and_moves() {
    let (c, b) = bay();
    let cam = race_cam(&c, 300);
    let mut sea_now = Batch::new();
    sea(&mut sea_now, &b, &cam, 500, 500, false);
    let (v, _) = sea_now.finish(0);
    // every near tile's top centre (a slab: its centre plus half its
    // depth) lies on `Water::height` there
    let mut near = 0;
    for s in v.chunks(INSTANCE_VALUES) {
        let (x, y, z) = (s[3], s[7], s[11]);
        let half = s[5] / 2;
        if s[0] == int(4) + int(4) / 12 && s[5] == cm(30) {
            assert_eq!(y + half, b.water.height(x, z, 500), "at ({x}, {z})");
            near += 1;
        }
    }
    assert!(near > 100, "{near} near tiles");
    let mut later = Batch::new();
    sea(&mut later, &b, &cam, 560, 560, false);
    assert_ne!(later.finish(0).0, v, "the sea moves with the tick");
}

#[test]
fn the_shallows_are_turquoise_and_the_bay_deep_blue() {
    let (_, b) = bay();
    let isl = b.island.centre;
    let by_island = water_colour(&b, isl.x + b.island.radius, isl.z, 0, cm(40), false, 1);
    let out = isl.add(v3(int(400), 0, int(400)));
    assert_eq!(shallows(&b, out.x, out.z), 0);
    let open = water_colour(&b, out.x, out.z, 0, cm(40), false, 1);
    assert!(by_island[1] > open[1] + ONE / 5, "{by_island:?} {open:?}");
    let dock = b.dock.to;
    assert!(shallows(&b, dock.x, dock.z) > ONE / 2);
}

#[test]
fn the_sky_and_sea_stay_cheap() {
    // the budget (MAPS: about 6–7k shapes drawn at once): the sea and sky
    // well under a third of it, clear or squall
    let (c, b) = bay();
    for s in [0, 300, 900] {
        let cam = race_cam(&c, s);
        let clear = drawn(&c, &b, &cam, 700, Weather::Clear);
        let storm = drawn(&c, &b, &cam, 700, Weather::Squall);
        let tris = |batch: &Batch| -> usize {
            Mesh::ALL
                .iter()
                .map(|m| {
                    batch.count_of(*m) * crate::mesh::vertices(*m).len()
                        / crate::mesh::VERTEX_VALUES
                        / 3
                })
                .sum()
        };
        let msg = format!(
            "at {s} m: clear {} shapes / {} triangles, squall {} / {}",
            clear.count(),
            tris(&clear),
            storm.count(),
            tris(&storm)
        );
        assert!(clear.count() < 2200 && storm.count() < 2200, "{msg}");
        assert!(tris(&clear) < 40_000 && tris(&storm) < 40_000, "{msg}");
        assert!(!SHOW && clear.count() > 0, "{msg}");
    }
}

/// Set to show the costs in `the_sky_and_sea_stay_cheap`'s message (the
/// core may not print).
const SHOW: bool = false;

#[test]
fn a_squall_darkens_roughens_and_flashes_from_the_tick() {
    let clear = look();
    let storm = squall(clear);
    assert!(storm.fog_range.1 < clear.fog_range.1 / 2);
    let spread = |c: Rgb| c.iter().max().unwrap() - c.iter().min().unwrap();
    assert!(spread(storm.sky) < spread(clear.sky));
    assert!(storm.sky.iter().sum::<i32>() < clear.sky.iter().sum::<i32>());
    // lightning: dark most of the time, some strikes, the same each run
    let lit: Vec<u32> = (0..TICKS_PER_SECOND as u32 * 60)
        .filter(|t| lightning(*t) > 0)
        .collect();
    assert!(
        !lit.is_empty() && lit.len() < 60 * 12,
        "{} lit ticks",
        lit.len()
    );
    assert!(lit.iter().all(|t| lightning(*t) == lightning(*t)));
    let t = lit[0];
    assert_ne!(flashed(storm, t), storm);
    if t > 0 {
        assert_eq!(flashed(storm, t - 1), storm, "dark just before");
    }
    // the drawn swell is rougher; the physics' water is not touched
    let (_, b) = bay();
    let w = &b.water;
    let (mut calm, mut rough) = (0, 0);
    for k in 0..200 {
        let (x, z) = (int(k * 3), int(k * 7 % 50));
        calm = calm.max((surface(w, x, z, 400, false, 0) - w.level).abs());
        rough = rough.max((surface(w, x, z, 400, true, 0) - w.level).abs());
        assert_eq!(surface(w, x, z, 400, false, 0), w.height(x, z, 400));
    }
    assert!(rough > calm * 3 / 2, "{rough} vs {calm}");
}
