// Tests for Rattlesnake Ravine's sky and light (step 5, SECOND-ORDER RK).

use super::*;
use crate::map::Map;

/// Brightness, 0..=255.
fn luma(c: Rgb) -> i32 {
    (c[0] * 30 + c[1] * 59 + c[2] * 11) / 100 * 255 / ONE
}

/// A colour lit as `scene.frag` roughly lights a wall facing the light
/// half-on: the bounce plus half the light.
fn lit(c: Rgb, l: &Look) -> Rgb {
    [0, 1, 2].map(|k| {
        let light = l.ground[k] + l.light[k] / 2;
        fx::mul(c[k], light).min(ONE)
    })
}

fn ravine() -> (Course, Box<RavineLayout>) {
    let c = Map::Ravine.course();
    let r = c.ravine.clone().expect("the ravine's layout");
    (c, r)
}

#[test]
fn the_mine_tunnel_and_the_caves_are_lit_and_their_fog_pushed_out() {
    let (c, r) = ravine();
    let outside = look_for(&c);
    let mut seen = 0;
    for room in r
        .rooms
        .iter()
        .filter(|m| matches!(m.kind, RoomKind::Tunnel | RoomKind::Cave))
    {
        let mid = room
            .from
            .lerp(room.to, ONE / 2)
            .add(v3(0, room.height / 2, 0));
        let k = inside_share(room, mid);
        assert!(k > ONE - ONE / 100, "{:?} at {mid:?}: {k}", room.kind);
        let l = look_at(&c, &r, mid);
        assert_ne!(l, outside, "{:?} keeps the outside light", room.kind);
        // never near-black (MAPS gotcha 7): a strong bounce and light, and
        // the fog far enough out to see the next lantern
        assert!(
            luma(l.ground) >= 90,
            "{:?} bounce {:?}",
            room.kind,
            l.ground
        );
        assert!(luma(l.light) >= 150, "{:?} light {:?}", room.kind, l.light);
        assert!(
            l.fog_range.1 >= int(240),
            "{:?} fog {:?}",
            room.kind,
            l.fog_range
        );
        assert!(luma(l.fog) >= 40, "{:?} fog colour {:?}", room.kind, l.fog);
        seen += 1;
    }
    assert!(seen >= 2, "a tunnel and a cave at least: {seen}");
}

#[test]
fn the_light_eases_in_over_a_mouth() {
    let (_, r) = ravine();
    let room = r
        .rooms
        .iter()
        .find(|m| m.kind == RoomKind::Tunnel)
        .expect("the mine tunnel");
    let run = room.to.sub(room.from).flat();
    let into = |m: i32| {
        room.from
            .add(run.norm().scale(m))
            .add(v3(0, room.height / 2, 0))
    };
    assert_eq!(inside_share(room, into(-int(2))), 0);
    let a = inside_share(room, into(int(2)));
    let b = inside_share(room, into(int(5)));
    assert!(0 < a && a < b && b < ONE, "{a} {b}");
}

#[test]
fn the_mesa_opens_the_fog_out_round_the_finish() {
    let (c, r) = ravine();
    let start = look_at(&c, &r, c.path[0].add(v3(0, int(3), 0)));
    let finish = look_at(&c, &r, r.finish.sphinx.add(v3(0, int(3), int(20))));
    assert_eq!(start.fog_range, look_for(&c).fog_range);
    assert!(
        finish.fog_range.1 > start.fog_range.1 + int(150),
        "{finish:?}"
    );
}

#[test]
fn the_night_sky_has_hundreds_of_stars_the_milky_way_and_the_moon() {
    let (c, r) = ravine();
    let eye = c.path[0].add(v3(0, int(4), 0));
    let mut b = Batch::new();
    sky(&mut b, eye, moon_yaw(&c), 0);
    // every field and band star above the horizon, and the haze
    assert!(
        b.count_of(Mesh::Ico) >= 550,
        "stars {}",
        b.count_of(Mesh::Ico)
    );
    // the moon and its seas, and the Milky Way's soft haze
    assert!(
        b.count_of(Mesh::Sphere) >= 4,
        "{}",
        b.count_of(Mesh::Sphere)
    );
    let haze = b.take_halos();
    assert!(haze.count() >= 30, "haze and corona {}", haze.count());
    // the moon stands ahead of the start, low over the rim
    let moon = facing(moon_yaw(&c), MOON_HIGH);
    assert!(moon.dot(c.heading[0].flat().norm()) > ONE * 3 / 4);
    assert!((deg(20)..deg(45)).contains(&MOON_HIGH));
    // the whole draw stays within the frame's budget (with the pools)
    let mut all = Batch::new();
    dynamic(&mut all, &c, &r, eye, 0);
    assert!(all.count() < 900, "{}", all.count());
}

#[test]
fn every_glow_throws_a_pool_of_light_near_the_eye() {
    let (_, r) = ravine();
    assert!(!r.glows.is_empty());
    for g in &r.glows {
        let mut b = Batch::new();
        pools(&mut b, &r, g.at.add(v3(0, int(3), 0)), 0);
        assert!(b.take_halos().count() >= 1, "{:?}", g.kind);
    }
}

#[test]
fn the_vines_read_against_the_night_walls_in_every_mode() {
    let l = look();
    let wall = lit(crate::ravinemodels::SANDSTONE, &l);
    let moss = lit(crate::ravinemodels::MOSS, &l);
    // normal: the pale moss-green vine is far brighter than the lit walls
    // and their moss (it glows, so it is drawn near its own colour)
    for w in [wall, moss] {
        assert!(luma(VINE) > luma(w) * 2, "vine {VINE:?} wall {w:?}");
    }
    // colourblind: the walls go toward grey, the vivid vine stays bright
    let vivid = crate::scene::vine_look_on(Map::Ravine, false, true, 0).colour;
    for w in [wall, moss] {
        let g = crate::sky::grey(w, crate::sky::MUTE);
        assert!(luma(vivid) > luma(g) * 2, "vivid {vivid:?} wall {g:?}");
    }
    // JOJO MODE: the walls take the wild colours (lit as the walls), the
    // vines keep theirs: always apart by brightness or colour
    for m in (0..300).step_by(15) {
        let (_, wild) = crate::sky::world(0, int(m));
        for w in wild {
            let w = lit(w, &l);
            let d: i32 = (0..3).map(|k| (VINE[k] - w[k]).abs()).sum::<i32>() * 255 / ONE;
            assert!(d > 150, "at {m} m: vine {VINE:?} wild wall {w:?}");
        }
    }
}
