//! The temple's face (live, 1.10.2: "you can fly directly through the
//! face, it has no physics"): solid everywhere but the mouth's opening.
//! Four boxes as thick as the opening round it (left, right, over, under),
//! from under the ravine's reset height to over its ceiling, as wide as
//! the face is drawn (`ruinsmodels::temple_face`); the body's ball is swept
//! from where it was at the start of the tick, as Sector Ape's bodies
//! (`sector::sweep`), so nothing passes through however fast.

use super::*;
use crate::ape::{Ape, Happening, Mode};
use crate::course::Course;
use crate::fx::mul;
use crate::geom::right_of;
use crate::rules::{BODY_RADIUS, CEILING, WALL_BOUNCE};
use crate::sector::{sweep, Touch};

/// The face's half width, as drawn (`temple_face`'s `wide`).
pub const FACE_HALF: i32 = int(32);

/// A little off the face, so the next tick starts clear of it.
const SKIN: i32 = cm(2);

impl Mouth {
    /// The face's solid boxes round the opening (centre, half sizes; axes
    /// across, up, along `fwd`), `floor` the temple's floor.
    pub fn face_boxes(&self, floor: i32) -> [(V3, V3); 4] {
        let o = self.opening;
        let right = right_of(o.fwd);
        let top = floor + TEMPLE_BELOW_LIP + CEILING + int(10);
        let bottom = floor - RAVINE_DEPTH - int(10);
        let (lo, hi) = (o.centre.y - o.half.y, o.centre.y + o.half.y);
        let at = |x: i32, y: i32| v3(o.centre.x, y, o.centre.z).add(right.scale(x));
        let side_half = (FACE_HALF - o.half.x) / 2;
        let side_x = o.half.x + side_half;
        let tall = v3(side_half, (top - bottom) / 2, o.half.z);
        [
            (at(-side_x, (top + bottom) / 2), tall),
            (at(side_x, (top + bottom) / 2), tall),
            (
                at(0, (top + hi) / 2),
                v3(o.half.x, (top - hi) / 2, o.half.z),
            ),
            (
                at(0, (lo + bottom) / 2),
                v3(o.half.x, (lo - bottom) / 2, o.half.z),
            ),
        ]
    }
}

impl Ape {
    /// Sunset Ruins: the temple's face stops a body moving from `before`
    /// to where it is: put back against the face, the motion into it gone
    /// (a little bounce, as a wall), so it slides and drops into the
    /// ravine. A hit knocks it off a vine.
    pub(crate) fn hit_facade(&mut self, course: &Course, before: V3) {
        let Some(r) = course.ruins.as_deref() else {
            return;
        };
        let o = r.mouth.opening;
        let (p0, p1) = (before, self.pos);
        // (cheap: far from the face either side, nothing to do)
        let reach = int(4) + o.half.z;
        let side0 = p0.sub(o.centre).dot(o.fwd);
        let side1 = p1.sub(o.centre).dot(o.fwd);
        if (side0 > reach && side1 > reach) || (side0 < -reach && side1 < -reach) {
            return;
        }
        let axes = [right_of(o.fwd), v3(0, ONE, 0), o.fwd];
        let mut best: Option<Touch> = None;
        for (c, half) in r.mouth.face_boxes(r.chamber.centre.y) {
            if let Some(t) = sweep(c, &axes, half, BODY_RADIUS, p0, p1) {
                if best.is_none_or(|b| t.t < b.t) {
                    best = Some(t);
                }
            }
        }
        let Some(t) = best else {
            return;
        };
        let n = t.normal;
        self.pos = if t.depth > 0 {
            p1.add(n.scale(t.end_depth + SKIN))
        } else {
            p0.add(p1.sub(p0).scale(t.t)).add(n.scale(SKIN))
        };
        let into = self.vel.dot(n);
        if into < 0 {
            self.vel = self.vel.sub(n.scale(into + mul(into, WALL_BOUNCE)));
        }
        if let Mode::Swinging { vine, .. } = self.mode {
            self.mode = Mode::Flying;
            self.last_vine = Some((vine, 0));
            self.release_bonus = 0;
        }
        self.happened.push(Happening::Wall);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ape::Controls;
    use crate::race::{Driver, Entry, Event, Phase, Race};
    use crate::rules::Kind;

    /// Ape 0 of a running race flung from 12 m out in front of the face,
    /// `across` right of the opening's middle and `up` over the temple's
    /// floor, at 40 m/s straight at the face: the furthest it gets past
    /// the face's outside (+ inside), how fast it was going on into the
    /// face there, and whether the ravine put it back within 6 s.
    fn fling(across: i32, up: i32) -> (i32, bool, i32) {
        let entry = Entry {
            name: "P0".into(),
            kind: Kind::Chimpanzee,
            driver: Driver::Local,
            palette: 0,
        };
        let mut r = Race::new(ruins(), &[entry], false, 1);
        while r.phase != Phase::Running {
            r.step(&[Controls::default()]);
        }
        let lay = r.course.ruins.as_deref().unwrap();
        let o = lay.mouth.opening;
        let floor = lay.chamber.centre.y;
        let right = right_of(o.fwd);
        let p = v3(o.centre.x, floor + up, o.centre.z)
            .add(right.scale(across))
            .sub(o.fwd.scale(int(12)));
        let a = &mut r.racers[0].ape;
        a.pos = p;
        a.vel = o.fwd.scale(int(40));
        a.mode = Mode::Flying;
        a.hint = r.course.nearest_anywhere(p);
        let mut furthest = i32::MIN;
        let mut on = 0;
        for _ in 0..6 * TICKS_PER_SECOND {
            r.step(&[Controls::default()]);
            if r.events.iter().any(|e| matches!(e, Event::Reset { .. })) {
                return (furthest, true, on);
            }
            let a = &r.racers[0].ape;
            let past = a.pos.sub(o.centre).dot(o.fwd) + o.half.z;
            if past > furthest {
                furthest = past;
                on = a.vel.dot(o.fwd);
            }
        }
        (furthest, false, on)
    }

    #[test]
    fn an_ape_flung_at_the_face_beside_the_opening_stops_and_drops_into_the_ravine() {
        // beside the opening, over it, under it, over the face's top
        for (across, up) in [(-17, 7), (17, 7), (0, 25), (0, 70), (0, -6), (-18, 40)] {
            let (past, reset, on) = fling(int(across), int(up));
            assert!(
                past <= 0,
                "at {across} m across, {up} m up: {} cm through the face",
                past * 100 / ONE
            );
            assert!(on <= int(3), "{across}, {up}: still going in at {on}");
            assert!(
                reset,
                "{across} m across, {up} m up: never fell into the ravine"
            );
        }
    }

    #[test]
    fn an_ape_flung_at_the_opening_gets_in() {
        for (across, up) in [(0, 7), (-11, 3), (11, 11)] {
            let (past, reset, _) = fling(int(across), int(up));
            assert!(
                past > int(10) && !reset,
                "at {across} m across, {up} m up: {} cm in, reset {reset}",
                past * 100 / ONE
            );
        }
    }
}
