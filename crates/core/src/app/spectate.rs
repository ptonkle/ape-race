//! The race camera: following a racer, switching and spectating (TRIAGE T24, T34, T53), and the title's flyover.

use super::*;

impl App {
    /// Follows the next or previous racer (`next`): in a replay any of
    /// them, live only once the local ape is out of the race, and then
    /// those still racing (TRIAGE T24, T34).
    pub(super) fn switch_focus(&mut self, next: bool) {
        let alpha = self.alpha;
        let Some(p) = &mut self.play else {
            return;
        };
        let live = p.replay.is_none();
        if live && focus::racing(&p.race.racers[p.local]) {
            return;
        }
        let from = play_camera(p, alpha);
        let moved = if next {
            p.focus.next(&p.race, live)
        } else {
            p.focus.prev(&p.race, live)
        };
        if moved {
            refocus(p, from, camera::PAN_QUICK);
        }
    }

    pub(super) fn camera(&self) -> Camera {
        if let Some(g) = &self.gallery {
            return g.camera();
        }
        match self.stage {
            Stage::Race => match &self.play {
                Some(p) => {
                    let cam = play_camera(p, self.alpha);
                    match &p.pan {
                        Some(pan) => pan.blend(cam, self.alpha),
                        None => cam,
                    }
                }
                None => self.flyover(),
            },
            Stage::Flyover => self.flyover(),
            // on the map, framed beside the docked menus (TRIAGE T47)
            Stage::Showcase => self.preview_camera(),
            Stage::Lineup => models::lineup_camera(self.aspect()),
        }
    }

    /// A slow glide down the chosen map's course, for the title.
    pub(super) fn flyover(&self) -> Camera {
        let Some(c) = self.course_of(self.map()) else {
            return models::showcase_camera();
        };
        scene::flyover_camera(c, self.ticks, self.fly_from)
    }

    /// The racer the camera follows, with buttons either side to switch
    /// (A/D or the arrow keys do the same).
    pub(crate) fn focus_picker(&self, p: &Play) -> Value {
        let name = p
            .race
            .racers
            .get(p.focus.racer)
            .map(|r| r.name.clone())
            .unwrap_or_default();
        json!({ "type": "picker", "id": "focus",
                "text": format!("{} {}", self.t("focus.watching"), name) })
    }
}

/// The course's way forward a little ahead of centreline sample `hint`.
pub(super) fn ahead_of(c: &Course, hint: usize) -> V3 {
    c.heading[(hint + 8).min(c.heading.len() - 1)]
}

/// The camera moves to the focused racer, gliding from `from`.
pub(super) fn refocus(p: &mut Play, from: Camera, ticks: u32) {
    let f = &p.race.racers[p.focus.racer];
    p.chase = Chase::new(f.ape.pos, p.chase.yaw, p.race.course.terrain)
        .with_crowns(f.ape.pos, p.chase.crowns.clone())
        .with_temple(f.ape.pos, p.chase.temple.clone())
        .with_hulls(f.ape.pos, p.chase.hulls.clone())
        .with_keeps(f.ape.pos, p.chase.keeps.clone());
    p.prev[p.focus.racer] = f.ape.pos;
    p.out_for = 0;
    p.danced = 0;
    p.pan = Some(Pan::new(from, ticks));
}

/// Live, after a tick: once the followed ape has been out of the race for
/// [`SPECTATE_AFTER`], the camera pans to the leading ape still racing
/// (TRIAGE T34).
pub(super) fn spectate(p: &mut Play, alpha: i32) {
    let followed = &p.race.racers[p.focus.racer];
    if focus::racing(followed) {
        p.out_for = 0;
        p.danced = 0;
        return;
    }
    p.out_for = p.out_for.saturating_add(1);
    if crate::dance::dancing(&p.race, p.focus.racer) {
        p.danced = p.danced.saturating_add(1);
    }
    if p.out_for < spectate_after(followed) {
        return;
    }
    // a finisher's dance is watched first (TRIAGE T53), unless it never
    // gets to dance
    if followed.finish.is_some() && p.danced < DANCE_SHOWN && p.out_for < FINISH_WATCH_MOST {
        return;
    }
    let from = play_camera(p, alpha);
    if p.focus.follow_racing(&p.race) {
        refocus(p, from, camera::PAN_TICKS);
    }
}

/// How long the camera stays on a racer out of the race: a finisher a
/// moment, an eaten one until the jaguar has eaten (TRIAGE T37).
pub(super) fn spectate_after(r: &crate::race::Racer) -> u32 {
    if r.eaten.is_some() && r.finish.is_none() {
        EATEN_SPECTATE_AFTER
    } else {
        SPECTATE_AFTER
    }
}

/// The race camera as it is, before any pan.
pub(super) fn play_camera(p: &Play, alpha: i32) -> Camera {
    match p.replay.as_ref().map(|pb| pb.view) {
        None | Some(replay::View::Chase) => p.chase.camera(alpha),
        Some(v) => replay_camera(p, v, alpha),
    }
}
