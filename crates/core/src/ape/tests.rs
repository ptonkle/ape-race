use super::*;
use crate::course::{ground, Course};
use crate::fx::{cm, deg, int};
use crate::rules::Kind;

/// A course with one vine 50 m along, its anchor 30 m up, 12 m long.
fn one_vine() -> Course {
    Course::test_line(&[(int(50), int(30), int(12))])
}

fn held() -> Controls {
    Controls {
        grab: true,
        ..Controls::default()
    }
}

/// An ape hanging from the vine at `angle` from straight down (toward +z),
/// still, already holding it.
fn hanging(c: &Course, angle: i32) -> Ape {
    let a = c.vines[0].anchor;
    let rope = int(12);
    let pos = a.add(v3(0, -mul(rope, fx::cos(angle)), mul(rope, fx::sin(angle))));
    let mut ape = Ape::new(Kind::Chimpanzee, true, pos, 0);
    ape.mode = Mode::Swinging { vine: 0, rope };
    ape.held = true;
    ape
}

#[test]
fn a_free_swing_keeps_its_height() {
    let c = one_vine();
    let mut ape = hanging(&c, deg(60));
    ape.phys.drag = 0;
    let start = ape.pos.y;
    let mut top_far_side = i32::MIN;
    let mut top_late = i32::MIN;
    for t in 0..1200 {
        ape.step(&held(), &c, t);
        assert!(ape.swinging().is_some(), "let go at {t}");
        if ape.pos.z < c.vines[0].anchor.z {
            top_far_side = top_far_side.max(ape.pos.y);
        }
        if t > 900 {
            top_late = top_late.max(ape.pos.y);
        }
    }
    let rise = c.vines[0].anchor.y - start; // how far below the anchor it started
    for top in [top_far_side, top_late] {
        assert!(
            (top - start).abs() < rise * 3 / 100,
            "top {} vs start {}",
            top / 655,
            start / 655
        );
    }
}

#[test]
fn pumping_with_the_swing_raises_it() {
    let c = one_vine();
    let run = |pump: bool| {
        let mut ape = hanging(&c, deg(20));
        let mut top = i32::MIN;
        for t in 0..1200 {
            // push the way the ape is moving: forward (+z) or back
            let fwd = if !pump {
                0
            } else if ape.vel.z >= 0 {
                1
            } else {
                -1
            };
            let ctl = Controls { fwd, ..held() };
            ape.step(&ctl, &c, t);
            if t > 600 {
                top = top.max(ape.pos.y);
            }
        }
        top
    };
    let pumped = run(true);
    let still = run(false);
    assert!(
        pumped > still + int(2),
        "{} vs {}",
        pumped / 655,
        still / 655
    );
}

#[test]
fn letting_go_at_the_bottom_keeps_the_speed() {
    let c = one_vine();
    let mut ape = hanging(&c, deg(70));
    let mut t = 0;
    // swing down to the bottom
    while ape.pos.z > c.vines[0].anchor.z {
        ape.step(&held(), &c, t);
        t += 1;
    }
    let before = ape.speed();
    ape.step(&Controls::default(), &c, t);
    assert_eq!(ape.mode, Mode::Flying);
    assert!(ape.happened.contains(&Happening::Release));
    let after = ape.speed();
    assert!((after - before).abs() < before / 50, "{before} -> {after}");
}

#[test]
fn letting_go_on_the_upswing_adds_the_release_boost() {
    let c = one_vine();
    let mut ape = hanging(&c, deg(70));
    let mut t = 0;
    // past the bottom and rising
    while !(ape.pos.z < c.vines[0].anchor.z - int(3) && ape.vel.y > 0) {
        ape.step(&held(), &c, t);
        t += 1;
    }
    let before = ape.speed();
    ape.step(&Controls::default(), &c, t);
    let after = ape.speed();
    assert!(after > before + RELEASE_BOOST / 2, "{before} -> {after}");
}

#[test]
fn a_grab_keeps_most_of_the_speed() {
    let c = one_vine();
    let a = c.vines[0].anchor;
    // flying in at 20 m/s, falling a little, just short of the vine
    let mut ape = Ape::new(
        Kind::Chimpanzee,
        true,
        v3(a.x, a.y - int(8), a.z - int(1)),
        0,
    );
    ape.mode = Mode::Flying;
    ape.vel = v3(0, -int(3), int(20));
    let before = ape.speed();
    ape.step(&held(), &c, 0);
    assert!(ape.swinging().is_some());
    assert!(
        ape.speed() >= mul(before, GRAB_KEEP) - ONE / 2,
        "{before} -> {}",
        ape.speed()
    );
}

#[test]
fn speed_has_a_soft_top() {
    let c = one_vine();
    let mut ape = hanging(&c, deg(40));
    let mut fastest = 0;
    for t in 0..120 * 60 {
        let fwd = if ape.vel.z >= 0 { 1 } else { -1 };
        let ctl = Controls { fwd, ..held() };
        ape.step(&ctl, &c, t);
        fastest = fastest.max(ape.speed());
    }
    assert!(fastest < int(60), "{}", fastest / ONE);
    assert!(fastest > int(15), "{}", fastest / ONE);
}

/// A flier passing the vine at hand height, moving along +z.
fn passing(c: &Course, behind: i32) -> Ape {
    let a = c.vines[0].anchor;
    let mut ape = Ape::new(
        Kind::Chimpanzee,
        true,
        v3(a.x, a.y - int(6), a.z - behind),
        0,
    );
    ape.mode = Mode::Flying;
    ape.vel = v3(0, 0, int(12));
    ape
}

/// Runs ticks with Space held or not; true when the ape grabbed.
fn fly_with(ape: &mut Ape, c: &Course, ticks: u32, grab: bool, from: u32) -> bool {
    for t in from..from + ticks {
        let ctl = Controls {
            grab,
            ..Controls::default()
        };
        ape.step(&ctl, c, t);
        if ape.swinging().is_some() {
            return true;
        }
        // keep it level for the test
        ape.vel.y = 0;
        ape.pos.y = c.vines[0].anchor.y - int(6);
    }
    false
}

#[test]
fn a_press_with_a_vine_in_reach_grabs() {
    let c = one_vine();
    let mut ape = passing(&c, int(1));
    assert!(fly_with(&mut ape, &c, 1, true, 0));
}

#[test]
fn a_press_into_empty_air_is_a_missed_grab_with_a_cooldown() {
    let c = one_vine();
    // far from the vine: the window runs out
    let mut ape = passing(&c, int(30));
    assert!(!fly_with(&mut ape, &c, GRAB_WINDOW as u32 + 1, true, 0));
    assert!(ape.cooldown > 0);
    // let go and press again, now in reach: the cooldown refuses it
    fly_with(&mut ape, &c, 1, false, 100);
    ape.pos.z = c.vines[0].anchor.z - int(1);
    assert!(!fly_with(&mut ape, &c, 1, true, 101));
    assert!(ape.swinging().is_none());
}

#[test]
fn after_the_cooldown_a_press_grabs_again() {
    let c = one_vine();
    let mut ape = passing(&c, int(30));
    fly_with(&mut ape, &c, GRAB_WINDOW as u32 + 1, true, 0);
    fly_with(&mut ape, &c, GRAB_COOLDOWN as u32, false, 100);
    assert_eq!(ape.cooldown, 0);
    ape.pos.z = c.vines[0].anchor.z - int(1);
    assert!(fly_with(&mut ape, &c, 1, true, 400));
}

#[test]
fn holding_space_through_a_vine_does_not_grab_it() {
    let c = one_vine();
    let mut ape = passing(&c, int(8));
    // pressed far before the vine; the window closes, then the vine passes
    assert!(!fly_with(&mut ape, &c, 120, true, 0));
}

#[test]
fn a_late_press_inside_the_window_still_grabs() {
    let c = one_vine();
    // the vine comes into reach a few ticks after the press
    let mut ape = passing(&c, int(1) + ape_reach());
    assert!(fly_with(&mut ape, &c, GRAB_WINDOW as u32, true, 0));
}

fn ape_reach() -> i32 {
    Kind::Chimpanzee.race_stats(true).physics().reach
}

#[test]
fn landing_and_running() {
    let c = one_vine();
    let p = c.point(int(100));
    let gy = ground(p.x, p.z);
    let mut ape = Ape::new(Kind::Chimpanzee, true, v3(p.x, gy + int(10), p.z), 0);
    ape.mode = Mode::Flying;
    ape.vel = v3(0, 0, int(25));
    let mut landed = false;
    for t in 0..240 {
        ape.step(&Controls::default(), &c, t);
        landed |= ape.happened.contains(&Happening::Land);
    }
    assert!(landed);
    assert_eq!(ape.mode, Mode::Grounded);
    // the ground has taken the extra speed
    assert!(
        ape.speed() <= ape.phys.run_speed + ONE / 4,
        "{}",
        ape.speed() / 655
    );
    // and WASD run at Run Speed
    for t in 240..480 {
        let ctl = Controls {
            fwd: 1,
            ..Controls::default()
        };
        ape.step(&ctl, &c, t);
    }
    assert!((ape.speed() - ape.phys.run_speed).abs() < ONE / 4);
}

#[test]
fn a_jump_under_a_low_vine_then_a_grab_takes_it() {
    // a vine whose bottom is 4.5 m above the ground: out of reach standing
    let c = Course::test_line(&[(int(50), int(14), cm(950))]);
    let a = c.vines[0].anchor;
    let gy = ground(a.x, a.z);
    let mut ape = Ape::new(
        Kind::Chimpanzee,
        true,
        v3(a.x, gy + BODY_DROP, a.z - cm(50)),
        0,
    );
    ape.step(&Controls::default(), &c, 0);
    assert_eq!(ape.mode, Mode::Grounded);
    assert!(
        ape.vine_in_reach(&c, 1).is_none(),
        "in reach from the ground"
    );
    // Space jumps; the left button, pressed near the top, grabs
    let jump = Controls {
        jump: true,
        ..Controls::default()
    };
    ape.step(&jump, &c, 1);
    assert!(ape.happened.contains(&Happening::Jump));
    let mut t = 2;
    while ape.vel.y > int(2) {
        ape.step(&jump, &c, t);
        t += 1;
    }
    let mut grabbed = false;
    for k in 0..20 {
        ape.step(&held(), &c, t + k);
        grabbed |= ape.swinging().is_some();
    }
    assert!(grabbed);
}

#[test]
fn space_jumps_but_never_grabs() {
    let c = Course::test_line(&[(int(50), int(13), int(10))]);
    let a = c.vines[0].anchor;
    let gy = ground(a.x, a.z);
    let mut ape = Ape::new(
        Kind::Chimpanzee,
        true,
        v3(a.x, gy + BODY_DROP, a.z - cm(50)),
        0,
    );
    let jump = Controls {
        jump: true,
        ..Controls::default()
    };
    for t in 0..120 {
        ape.step(&jump, &c, t);
        assert!(ape.swinging().is_none());
    }
    // a jump is not a Missed Grab
    assert_eq!(ape.cooldown, 0);
    // and Space in the air does nothing
    let mut flier = passing(&c, int(1));
    flier.step(&jump, &c, 0);
    assert_eq!(flier.mode, Mode::Flying);
    assert!(flier.swinging().is_none());
}

#[test]
fn a_grab_press_on_the_ground_takes_a_vine_in_reach() {
    let c = Course::test_line(&[(int(50), int(13), int(10))]);
    let a = c.vines[0].anchor;
    let gy = ground(a.x, a.z);
    let mut ape = Ape::new(
        Kind::Chimpanzee,
        true,
        v3(a.x, gy + BODY_DROP, a.z - cm(50)),
        0,
    );
    ape.step(&Controls::default(), &c, 0);
    ape.step(&held(), &c, 1);
    assert!(ape.swinging().is_some());
}

#[test]
fn a_hit_knocks_an_ape_off_its_vine() {
    let c = one_vine();
    let mut ape = hanging(&c, deg(30));
    ape.vel = v3(0, 0, int(10));
    ape.hit(ONE / 2, 60, V3::ZERO);
    assert_eq!(ape.mode, Mode::Flying);
    assert!(ape.stun > 0);
    assert!(ape.speed() < int(6));
    // a stunned ape cannot grab
    let mut grabbed = false;
    for t in 0..10 {
        ape.step(&held(), &c, t);
        grabbed |= ape.swinging().is_some();
    }
    assert!(!grabbed);
}

#[test]
fn the_same_controls_give_the_same_path() {
    let c = Course::jungle();
    let run = |kind: Kind, equal: bool| {
        let mut ape = Ape::new(kind, equal, c.starts[0], 0);
        for t in 0..600u32 {
            let ctl = Controls {
                fwd: 1,
                side: if t % 200 < 100 { 1 } else { 0 },
                grab: t % 90 > 20,
                ..Controls::default()
            };
            ape.step(&ctl, &c, t);
        }
        (ape.pos, ape.vel)
    };
    assert_eq!(run(Kind::Gibbon, false), run(Kind::Gibbon, false));
    // with Equal Stats a Gorilla and a Lemur are the same ape
    assert_eq!(run(Kind::Gorilla, true), run(Kind::Lemur, true));
}
