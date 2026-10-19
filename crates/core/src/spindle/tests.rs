use super::*;
use crate::ape::{Ape, Controls, Happening, Mode};
use crate::course::Course;
use crate::fx::{cm, int, ONE};
use crate::geom::{v3, Affine};
use crate::rules::{Kind, BODY_DROP, BODY_RADIUS};

fn near(a: V3, b: V3, within: i32) -> bool {
    a.sub(b).len() <= within
}

/// The course with `blocks` and `spinners` in place of its own.
fn with(blocks: Vec<Block>, spinners: Vec<Spinner>) -> Course {
    let mut c = spindle();
    let s = c.spindle.as_deref_mut().unwrap();
    s.blocks = blocks;
    s.spinners = spinners;
    c
}

fn run(a: &mut Ape, c: &Course, ctl: &Controls, ticks: u32) {
    for t in 0..ticks {
        a.step(ctl, c, t);
    }
}

#[test]
fn the_floor_bends_up_overhead_and_back_round() {
    let w = Warp::SPINDLE;
    // the floor under the route stays where it would be flat
    assert!(near(w.point(v3(0, 0, int(50))), v3(0, 0, int(50)), cm(1)));
    // half way round, the floor is overhead: the tube's diameter up
    let over = w.point(v3(CIRCUMFERENCE / 2, 0, int(50)));
    assert!(near(over, v3(0, 2 * RADIUS, int(50)), cm(5)), "{over:?}");
    // a quarter round, on the side wall at the axis' height
    let side = w.point(v3(CIRCUMFERENCE / 4, 0, 0));
    assert!(near(side, v3(RADIUS, RADIUS, 0), cm(5)), "{side:?}");
    // height climbs toward the axis
    assert!(near(w.point(v3(0, RADIUS, 0)), v3(0, RADIUS, 0), cm(1)));
    // a full turn round comes back
    let p = v3(int(17), int(5), int(9));
    assert!(near(
        w.point(p.add(v3(CIRCUMFERENCE, 0, 0))),
        w.point(p),
        cm(5)
    ));
}

#[test]
fn bending_turns_a_shape_with_the_floor_and_flat_of_undoes_it() {
    let w = Warp::SPINDLE;
    // a post standing on the floor overhead points down, toward the axis
    let post = Affine::translate(v3(CIRCUMFERENCE / 2, int(3), 0));
    let bent = w.affine(&post);
    assert!(bent.c1.y < -ONE * 99 / 100, "{:?}", bent.c1);
    // flat_of is point's inverse, and affine_of affine's
    for p in [
        v3(int(30), int(10), int(4)),
        v3(-int(100), int(25), 0),
        v3(int(130), cm(50), int(7)),
    ] {
        assert!(near(w.flat_of(w.point(p)), p, cm(20)), "{p:?}");
    }
    let world = Affine::translate(v3(int(20), int(70), int(5)));
    let back = w.affine(&w.affine_of(&world));
    assert!(near(back.t, world.t, cm(20)));
    assert!(near(back.c1, world.c1, ONE / 100));
}

#[test]
fn running_round_the_tube_comes_back_where_it_began() {
    let c = spindle();
    // (between the warm-up's slime pads)
    let mut a = Ape::new(Kind::Chimpanzee, true, v3(0, BODY_DROP, int(140)), 0);
    // run sideways (yaw a quarter turn) for long enough to pass the seam
    let ctl = Controls {
        fwd: 1,
        yaw: crate::fx::QUARTER,
        ..Controls::default()
    };
    let mut crossed = false;
    let mut last = a.pos.x;
    for t in 0..120 * 40 {
        a.step(&ctl, &c, t);
        assert!(a.pos.x.abs() <= CIRCUMFERENCE / 2, "x {}", a.pos.x);
        if (a.pos.x - last).abs() > CIRCUMFERENCE / 2 {
            crossed = true;
        }
        last = a.pos.x;
    }
    assert!(crossed, "never went round");
}

#[test]
fn a_swinging_ape_is_not_wrapped_off_its_vine() {
    let c = spindle();
    let s = c.spindle.as_deref().unwrap();
    // swinging past the seam on a vine short of it: kept on this side
    let mut a = Ape::new(
        Kind::Chimpanzee,
        true,
        v3(CIRCUMFERENCE / 2 + int(4), int(16), int(100)),
        0,
    );
    a.mode = Mode::Swinging {
        vine: 0,
        rope: int(22),
    };
    a.hit_spindle(s, 0, a.pos, int(17));
    assert!(a.pos.x > CIRCUMFERENCE / 2, "wrapped while swinging");
    // let go, it is brought round
    a.mode = Mode::Flying;
    a.hit_spindle(s, 0, a.pos, int(17));
    assert!(a.pos.x < 0, "not wrapped once let go");
}

#[test]
fn the_axle_is_the_ceiling() {
    let c = spindle();
    let mut a = Ape::new(Kind::Chimpanzee, true, v3(0, int(30), int(100)), 0);
    a.mode = Mode::Flying;
    a.vel = v3(0, int(40), 0);
    let mut top = 0;
    for t in 0..60 {
        a.step(&Controls::default(), &c, t);
        top = top.max(a.pos.y);
    }
    assert!(top <= AXLE_UNDER - BODY_RADIUS, "rose to {}", top / ONE);
}

#[test]
fn a_block_stops_a_runner_and_holds_one_on_top() {
    let block = Block {
        lo: v3(-int(10), 0, int(120)),
        hi: v3(int(10), int(6), int(124)),
        kind: BlockKind::Wool(1),
    };
    let c = with(vec![block], Vec::new());
    let mut a = Ape::new(Kind::Chimpanzee, true, v3(0, BODY_DROP, int(100)), 0);
    let ctl = Controls {
        fwd: 1,
        ..Controls::default()
    };
    let mut hit = false;
    for t in 0..240 {
        a.step(&ctl, &c, t);
        hit |= a.happened.contains(&Happening::Wall);
        assert!(
            a.pos.z <= int(120) - BODY_RADIUS + cm(1),
            "ran into the block to {}",
            a.pos.z
        );
    }
    assert!(hit);
    // dropped onto it, it stands on its top
    let mut b = Ape::new(Kind::Chimpanzee, true, v3(0, int(10), int(122)), 0);
    b.mode = Mode::Flying;
    run(&mut b, &c, &Controls::default(), 120);
    assert_eq!(b.mode, Mode::Grounded);
    assert_eq!(b.pos.y - BODY_DROP, int(6));
}

#[test]
fn slime_throws_a_landing_ape_back_up() {
    let slime = Block {
        lo: v3(-int(6), 0, int(118)),
        hi: v3(int(6), int(2), int(126)),
        kind: BlockKind::Slime,
    };
    let c = with(vec![slime], Vec::new());
    let mut a = Ape::new(Kind::Chimpanzee, true, v3(0, int(12), int(122)), 0);
    a.mode = Mode::Flying;
    let mut bounced = false;
    let mut top = 0;
    for t in 0..240 {
        a.step(&Controls::default(), &c, t);
        bounced |= a.happened.contains(&Happening::Bounce);
        if bounced {
            top = top.max(a.pos.y);
        }
    }
    assert!(bounced);
    assert!(top > int(12), "only rose to {}", top / ONE);
}

#[test]
fn a_spinning_column_carries_an_ape_round_with_it() {
    let sp = Spinner {
        z: int(150),
        depth: int(4),
        width: int(6),
        count: 1,
        period: 120 * 10,
        phase: -int(20),
        kind: BlockKind::Obsidian,
    };
    let c = with(Vec::new(), vec![sp]);
    // the column moves round at its speed, a pure function of the tick
    assert_eq!(sp.x_at(0, 0), -int(20));
    assert!(sp.x_at(0, 120) > sp.x_at(0, 0));
    assert_eq!(sp.x_at(0, 1200), sp.x_at(0, 0));
    // an ape standing in its way is pushed round, not passed through
    let mut a = Ape::new(Kind::Chimpanzee, true, v3(0, BODY_DROP, int(150)), 0);
    let mut pushed = false;
    for t in 0..360 {
        a.step(&Controls::default(), &c, t);
        let dx = Warp::SPINDLE.wrap_x(a.pos.x - sp.x_at(0, t));
        assert!(
            dx.abs() >= sp.width / 2 + BODY_RADIUS - cm(1),
            "inside the column at {t}"
        );
        pushed |= a.happened.contains(&Happening::Wall);
    }
    assert!(pushed);
    assert!(a.pos.x > 0);
}

#[test]
fn the_vines_hang_all_round_the_tube_and_clear_of_the_seam() {
    let c = spindle();
    assert!(!c.vines.is_empty());
    let round: Vec<i32> = c.vines.iter().map(|v| v.anchor.x).collect();
    assert!(round.iter().any(|x| *x > CIRCUMFERENCE / 3));
    assert!(round.iter().any(|x| *x < -CIRCUMFERENCE / 3));
    for v in &c.vines {
        assert!(v.anchor.x.abs() < CIRCUMFERENCE / 2 - layout::SEAM_CLEAR + int(2));
        assert!(v.anchor.y < AXLE_UNDER);
        assert!(v.anchor.y - v.length > 0, "a vine to the floor");
    }
}
