// Tests for Rattlesnake Ravine's course art (step 8, SECOND-ORDER RA-n).

use super::*;
use crate::map::Map;
use crate::ravine::solids::{Shape, SolidKind, Solids};
use std::cell::RefCell;

fn ravine() -> (Course, Box<RavineLayout>) {
    let c = Map::Ravine.course();
    let r = c.ravine.clone().expect("the ravine's layout");
    (c, r)
}

fn close(a: V3, b: V3, within: i32) -> bool {
    !a.sub(b).longer_than(within)
}

/// The frame's budget (MAPS.md, `notes/ravine-pitfalls.md`): the scenery
/// drawn at once plus what moves round the eye.
const MOST_AT_ONCE: usize = 5_500;

#[test]
fn every_wall_stretch_is_drawn_from_its_solids_foot_to_its_top() {
    // (RC3: the fin's pieces over and under the skull's mouth too, so no
    // solid slot shows as an opening)
    let (_, r) = ravine();
    let solids = Solids::of(&r);
    let mut walls = solids.list.iter().filter(|s| s.kind == SolidKind::Wall);
    let mut fin = 0;
    for w in &r.walls {
        for k in w.points.windows(2) {
            if k[1].at.sub(k[0].at).flat().len() <= 0 {
                continue;
            }
            let s = walls.next().expect("a solid for every stretch");
            let Shape::Box { mid, half, .. } = s.shape else {
                panic!("a wall solid is a box");
            };
            let (bottom, top) = stretch_heights(&k[0], &k[1]);
            // (to a 32,000th of a metre: the solid halves its height)
            assert!(
                (bottom - (mid.y - half.y)).abs() <= 2 && (top - (mid.y + half.y)).abs() <= 2,
                "a stretch at {:?} drawn {bottom}..{top}, solid {}..{}",
                k[0].at,
                mid.y - half.y,
                mid.y + half.y
            );
            if k[0].height < int(20) {
                fin += 1;
            }
        }
    }
    assert!(fin > 0, "the fin's pieces round the skull's mouth");
}

#[test]
fn the_skull_and_the_sphinx_are_drawn_where_they_are_solid() {
    let (_, r) = ravine();
    let s = r.skull.as_ref().expect("the skull");
    let f = skull_frame(s);
    let model = m::rex_teeth(crate::ravine::MOUTH_GAP);
    assert_eq!(model.len(), s.teeth.len());
    for (t, lay) in model.iter().zip(&s.teeth) {
        assert!(
            close(f.apply(t.base), lay.root, cm(2)) && close(f.apply(t.tip), lay.tip, cm(2)),
            "a tooth drawn at {:?} is solid at {:?}",
            f.apply(t.tip),
            lay.tip
        );
    }
    let solids = Solids::of(&r);
    let sphinx = solids
        .list
        .iter()
        .find(|s| s.kind == SolidKind::Sphinx)
        .expect("the sphinx's solid");
    let Shape::Box { mid, fwd, .. } = sphinx.shape else {
        panic!("the sphinx's solid is a box");
    };
    let f = sphinx_frame(&r);
    let drawn = f.apply(v3(0, m::SPHINX_SIZE.y / 2, 0));
    assert!(close(drawn, mid, cm(5)), "drawn {drawn:?}, solid {mid:?}");
    assert!(
        close(f.apply_dir(v3(0, 0, ONE)), fwd, cm(1)),
        "it faces its solid's way"
    );
}

#[test]
fn a_snapped_snake_falls_to_the_floor_and_lies_there() {
    let most = int(9);
    assert_eq!(snake_fall(0, most), 0);
    let mut last = 0;
    for t in (0..240).step_by(10) {
        let d = snake_fall(t, most);
        assert!(d >= last && d <= most, "{t}: {d}");
        last = d;
    }
    assert_eq!(snake_fall(240, most), most, "on the floor within 2 s");
    assert_eq!(snake_fall(240, -int(1)), 0, "never falls up");
}

#[test]
fn every_snake_vine_is_drawn_with_a_warning_glow_and_black_and_white_bands_when_colourblind() {
    let (c, r) = ravine();
    let snakes: Vec<usize> = (0..r.hangers.len()).filter(|&i| r.is_snake(i)).collect();
    assert!(!snakes.is_empty());
    for vivid in [false, true] {
        for &i in snakes.iter().step_by(5) {
            let eye = c.vines[i].anchor;
            let mut b = Batch::new();
            dynamic(&mut b, &c, &r, eye, 0, 0, vivid);
            let mut glows = b.take_halos();
            let seen = RefCell::new(Vec::new());
            glows.restyle(|col, g| {
                seen.borrow_mut().push(col);
                (col, g)
            });
            let warn = if vivid { SNAKE_GLOW_VIVID } else { SNAKE_GLOW };
            assert!(
                seen.borrow().iter().filter(|&&col| col == warn).count() >= 2,
                "snake {i} (vivid {vivid}): no warning glow at its head and rattle"
            );
            if vivid {
                // every unlit shape round a snake's anchor is black or
                // white (lizards, carts, glows and the skull's torches are elsewhere)
                let mut only = Batch::new();
                let mut alone = r.clone();
                alone.lizards.clear();
                alone.carts.clear();
                alone.glows.clear();
                // (and the skull's torches, RA-12)
                alone.skull = None;
                alone.finish.campfire = v3(int(9000), 0, 0);
                dynamic(&mut only, &c, &alone, eye, 0, 0, true);
                let cols = RefCell::new(Vec::new());
                only.restyle(|col, g| {
                    cols.borrow_mut().push((col, g));
                    (col, g)
                });
                let bad = cols
                    .borrow()
                    .iter()
                    .filter(|(col, g)| *g == 0 && *col != [ONE; 3] && *col != [ONE / 20; 3])
                    .count();
                assert_eq!(bad, 0, "snake {i}: colourblind bands not black and white");
            }
        }
    }
}

#[test]
fn the_ravine_frame_stays_within_budget() {
    use crate::render::DRAW_VALUES;
    let (c, r) = ravine();
    let s = scenery(&c, &r);
    let mut worst = (0, 0, 0);
    for k in 0..c.path.len() / 5 {
        let eye = c.path[k * 5].add(v3(0, int(12), 0));
        let still: usize = s
            .draws_in(eye, int(260))
            .chunks(DRAW_VALUES)
            .filter(|d| d[1] != Mesh::Terrain as u32)
            .map(|d| d[3] as usize)
            .sum();
        let mut b = Batch::new();
        dynamic(&mut b, &c, &r, eye, 100, 100, false);
        let moving = b.count() + b.take_halos().count();
        if still + moving > worst.0 + worst.1 {
            worst = (still, moving, k * 5);
        }
    }
    assert!(
        worst.0 + worst.1 <= MOST_AT_ONCE,
        "{} still + {} moving shapes at sample {}",
        worst.0,
        worst.1,
        worst.2
    );
}

/// Every shape's middle in `b` (the instance's translation).
fn middles(b: Batch) -> Vec<V3> {
    let (values, _) = b.finish(0);
    values
        .chunks(crate::render::INSTANCE_VALUES)
        .map(|s| v3(s[3], s[7], s[11]))
        .collect()
}

#[test]
fn the_skull_is_torch_lit_from_the_approach() {
    // (RA-12: the fin's hairpin was near-black; the teeth and the gap
    // must read from the in-leg)
    let (c, r) = ravine();
    let s = r.skull.as_ref().expect("the skull");
    let out = crate::geom::facing(s.yaw, 0).neg();
    let approach = s.at.add(out.scale(int(25)));
    let mut b = Batch::new();
    skull_lights(&mut b, s, 0);
    let mut halos = b.take_halos();
    let fire = RefCell::new(0);
    halos.restyle(|col, g| {
        if col == SKULL_FIRE {
            *fire.borrow_mut() += 1;
        }
        (col, g)
    });
    assert!(
        *fire.borrow() > SKULL_TORCHES.len(),
        "{} torchlight glows round the skull",
        fire.borrow()
    );
    // the torches stand on the fin's face, beside the skull, not in its mouth
    for (side, _) in SKULL_TORCHES {
        assert!(side.abs() > int(7), "a torch {side} across is in the skull");
    }
    let warm = |l: crate::scene::Look| l.light[0] > l.light[2];
    assert!(
        warm(crate::ravinescape::look_at(&c, &r, approach)),
        "the approach is lit by the torches"
    );
    let far = s.at.add(out.scale(int(200)));
    assert!(
        crate::ravinescape::skull_share(s, far) == 0,
        "the torchlight reaches 200 m"
    );
}

#[test]
fn the_canyon_mouths_steps_are_heaped_with_rock_outside_the_canyon() {
    // (RA-11, RA-12: the walls fell to the mesa as bare slabs; the rock
    // dresses the steps and slopes away outside, never into the way)
    let (_, r) = ravine();
    let mut steps = 0;
    for w in &r.walls {
        for k in w.points.windows(2) {
            let (a, z) = (k[0], k[1]);
            if (a.height - z.height).abs() <= MOUTH_STEP {
                continue;
            }
            steps += 1;
            let fwd = z.at.sub(a.at).flat().norm();
            let into = into_rock(w.side, fwd);
            let (bottom, top) = stretch_heights(&a, &z);
            let mut b = Batch::new();
            mouth_rock(
                &mut b,
                (a, z),
                fwd,
                into,
                w.depth,
                (bottom + WALL_BELOW, top),
                0,
            );
            for p in middles(b) {
                let back = p.sub(a.at).flat().dot(into);
                assert!(
                    back > int(1),
                    "a mouth rock at {p:?} sits {back} into the rock from the face at {:?}",
                    a.at
                );
            }
        }
    }
    assert!(steps >= 6, "{steps} canyon-mouth steps (both sides)");
}

#[test]
fn a_wall_lizard_glows_white_when_colourblind() {
    let (c, r) = ravine();
    let (at, _) = r.lizard_at(0, 0);
    for vivid in [false, true] {
        let mut b = Batch::new();
        dynamic(&mut b, &c, &r, at, 0, 0, vivid);
        let mut halos = b.take_halos();
        let white = RefCell::new(0);
        halos.restyle(|col, g| {
            if col == LIZARD_GLOW_VIVID {
                *white.borrow_mut() += 1;
            }
            (col, g)
        });
        assert_eq!(*white.borrow() > 0, vivid, "lizard glow (vivid {vivid})");
    }
}
