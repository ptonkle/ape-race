//! The chase camera on the Ski Slope keeps out of the conifers
//! (SECOND-ORDER SW9), smoothly.

use super::*;
use crate::skiscape::Crown;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

/// Each tick of an all-AI Ski Slope race (dice `seed`, four apes) seen by
/// the chase camera following ape `focus`: the ape's look-at point and the
/// eye. `trees` false takes the conifers away from the camera, as before.
fn ski_race(seed: u64, focus: usize, trees: bool) -> (Vec<(V3, V3)>, std::sync::Arc<[Crown]>) {
    let mut a = app();
    a.ensure_course(Map::Ski);
    let course = a.course_of(Map::Ski).cloned().unwrap();
    let entries: Vec<Entry> = [
        Kind::Gorilla,
        Kind::Chimpanzee,
        Kind::Orangutan,
        Kind::Lemur,
    ]
    .into_iter()
    .map(|k| Entry {
        name: k.key().into(),
        kind: k,
        driver: Driver::Ai,
        palette: 0,
        costume: crate::costume::Costume::Own,
    })
    .collect();
    let race = Race::new(course, &entries, false, seed);
    a.begin(race, focus);
    let mut p = a.play.take().unwrap();
    let crowns = p.chase.crowns.clone();
    if !trees {
        p.chase.crowns = std::sync::Arc::from(Vec::new());
    }
    let mut out = Vec::new();
    let limit = 140 * rules::TICKS_PER_SECOND as u32;
    while p.race.racers[focus].finish.is_none() && p.race.tick < limit {
        a.step_race(&mut p);
        out.push((p.chase.target, p.chase.eye));
    }
    (out, crowns)
}

/// Ticks at which the eye went into a conifer by itself: not one the ape
/// is in or brushing past (`camera::NEAR_APE`), nor one the eye was
/// already in a tick before (drawn in with the ape, it glides out).
fn eye_went_into_a_tree(run: &[(V3, V3)], crowns: &[Crown]) -> Vec<usize> {
    let mut bad = Vec::new();
    let mut prev: Option<V3> = None;
    for (k, (t, e)) in run.iter().enumerate() {
        if crowns.iter().any(|c| {
            c.holds(*e, 0) && !c.holds(*t, camera::NEAR_APE) && !prev.is_some_and(|p| c.holds(p, 0))
        }) {
            bad.push(k);
        }
        prev = Some(*e);
    }
    bad
}

/// SW9: the eye ended up inside a conifer by the fence. Checked on one
/// race (dice `seed`, chase camera on ape `focus`), with and without the
/// trees; one test per race so the test harness runs them side by side.
fn camera_keeps_out_of_the_conifers_smoothly(seed: u64, focus: usize) {
    let (run, crowns) = ski_race(seed, focus, true);
    let bad = eye_went_into_a_tree(&run, &crowns);
    assert!(
        bad.is_empty(),
        "seed {seed}: the eye went into a tree at ticks {bad:?}"
    );
    // smooth: as the camera was (its worst change in step a tick is
    // 14 cm) but for a few quick moves in front of a tree that came
    // too fast to glide ahead of
    let (old, _) = ski_race(seed, focus, false);
    let jolts = |r: &[(V3, V3)]| {
        r.windows(3)
            .filter(|w| w[2].1.sub(w[1].1).sub(w[1].1.sub(w[0].1)).len() > ONE * 3 / 10)
            .count()
    };
    assert_eq!(jolts(&old), 0);
    let n = jolts(&run);
    assert!(n <= 20, "seed {seed}: {n} jolts over 30 cm");
    // and before, it did go in
    assert!(!eye_went_into_a_tree(&old, &crowns).is_empty());
}

#[test]
fn on_the_ski_slope_the_chase_camera_keeps_out_of_the_conifers_smoothly_seed_1() {
    camera_keeps_out_of_the_conifers_smoothly(1, 0);
}

#[test]
fn on_the_ski_slope_the_chase_camera_keeps_out_of_the_conifers_smoothly_seed_2() {
    camera_keeps_out_of_the_conifers_smoothly(2, 3);
}
