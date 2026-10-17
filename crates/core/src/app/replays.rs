//! The replays (TRIAGE T15, T23, T55): keeping runs, the library, watching one, its viewer and its cameras.

use super::*;

impl App {
    /// The replays' buttons: watch the race just run, the best, a kept
    /// one; leave; and the viewer's controls.
    pub(super) fn replays_act(&mut self, id: &str, value: &str) {
        match id {
            "menu.replay" => {
                // the race just run: watch it, then come back to its results
                if let Some(mut p) = self.play.take() {
                    self.keep_recording(&mut p);
                    let rec = p.record.clone();
                    let exit = self.screen;
                    self.stash = Some(p);
                    if !rec.is_some_and(|r| self.watch(r, exit)) {
                        self.play = self.stash.take();
                    }
                }
            }
            "replay.best" => {
                if let Some(rec) = self.best_replay().cloned() {
                    self.watch(rec, Screen::Setup);
                }
            }
            _ if id.starts_with("replays.watch.") => {
                let k: usize = id["replays.watch.".len()..].parse().unwrap_or(usize::MAX);
                let rec = self.library.listing().get(k).map(|(r, _)| (*r).clone());
                if let Some(rec) = rec {
                    self.watch(rec, Screen::Replays);
                }
            }
            "replay.exit" => self.leave_replay(),
            _ if id.starts_with("replay.") => self.replay_act(id, value),
            _ => {}
        }
    }
    /// Puts a finished run into the replay library, or brings the one kept
    /// up to date (the race goes on being recorded while the rest finish).
    pub(super) fn keep_recording(&mut self, p: &mut Play) {
        if p.replay.is_some() {
            return;
        }
        let me = &p.race.racers[p.local];
        let (finish, eaten) = (me.finish, me.eaten.is_some());
        if let (Some(rec), true) = (&p.record, finish.is_some() || eaten) {
            let mut rec = rec.clone();
            rec.finish = finish;
            rec.eaten = eaten;
            match p.kept {
                Some(serial) => {
                    rec.serial = serial;
                    self.save_replays |= self.library.refresh(&rec);
                }
                None => {
                    p.kept = Some(self.library.serial_of_next());
                    self.library.add(rec);
                    self.save_replays = true;
                }
            }
        }
    }

    /// A drag across the 3D view while not captured, in pixels: in a
    /// replay it steers the orbit camera, switching to it (TRIAGE T23).
    pub fn drag(&mut self, dx: i32, dy: i32) {
        let alpha = self.alpha;
        let Some(p) = &mut self.play else {
            return;
        };
        let Some(view) = p.replay.as_ref().map(|pb| pb.view) else {
            return;
        };
        if view != replay::View::Orbit {
            // start from the view as it is
            let cam = play_camera(p, alpha);
            let at = p.race.racers[p.focus.racer].ape.pos;
            if let Some(pb) = &mut p.replay {
                pb.orbit.yaw = crate::geom::yaw_of(cam.fwd);
                pb.orbit.pitch = crate::geom::pitch_of(cam.fwd);
                pb.orbit.dist = at.dist(cam.eye);
                pb.view = replay::View::Orbit;
            }
        }
        if let Some(pb) = &mut p.replay {
            pb.orbit.drag(dx, dy);
        }
    }

    /// The mouse wheel over the 3D view: in a replay's orbit, zoom (only
    /// its direction counts, so pixel and line wheels agree).
    pub fn wheel(&mut self, dy: i32) {
        if let Some(pb) = self.play.as_mut().and_then(|p| p.replay.as_mut()) {
            if pb.view == replay::View::Orbit {
                pb.orbit.zoom(dy.signum());
            }
        }
    }

    /// The replays kept in browser storage, from the page at start.
    pub fn load_replays(&mut self, text: &str) {
        self.library = Library::parse(text);
    }

    /// The page could not store the last `save_replays` text (storage full
    /// or blocked; what was stored before is still there). Asks to save
    /// again with one recent replay fewer, down to the bests alone, which
    /// are never shed (TRIAGE T55). The library itself keeps them all.
    pub fn replays_not_saved(&mut self) {
        let room = self.replay_room.min(self.library.recent.len());
        if room > 0 {
            self.replay_room = room - 1;
            self.save_replays = true;
        }
    }

    /// Starts watching `rec`; leaving goes back to `exit` (or to the
    /// stashed race's results).
    pub(super) fn watch(&mut self, rec: Recording, exit: Screen) -> bool {
        let Some(map) = Map::from_key(&rec.course) else {
            return false;
        };
        self.ensure_world(map);
        let Some(race) = self
            .course_of(map)
            .filter(|c| rec.watchable(c))
            .map(|c| rec.race(c))
        else {
            return false;
        };
        let local = rec.local;
        self.begin(race, local);
        self.replay_exit = exit;
        if let Some(p) = &mut self.play {
            p.trial = rec.mode == replay::Mode::Trial;
            p.replay = Some(Playback::new(rec));
        }
        true
    }

    /// The replay viewer's controls: play/pause, speed, camera, seek.
    pub(super) fn replay_act(&mut self, id: &str, value: &str) {
        let Some(p) = &mut self.play else {
            return;
        };
        let Some(pb) = &mut p.replay else {
            return;
        };
        match id {
            "replay.play" if pb.length() <= p.race.tick => {
                // at the end: Play starts again
                pb.playing = true;
                if let Some(save) = pb.seek_to(0, p.race.tick) {
                    restore(p, save);
                }
            }
            "replay.play" => pb.playing = !pb.playing,
            "replay.speed" => {
                if let Some(sp) = replay::Speed::ALL.into_iter().find(|s| s.key() == value) {
                    pb.speed = sp;
                }
            }
            "replay.camera" => {
                if let Some(v) = replay::View::ALL.into_iter().find(|v| v.key() == value) {
                    pb.view = v;
                }
            }
            "replay.seek" => {
                let permille = value.parse::<u32>().unwrap_or(0);
                if let Some(save) = pb.seek_to(permille, p.race.tick) {
                    restore(p, save);
                }
            }
            _ => {}
        }
    }

    pub(super) fn leave_replay(&mut self) {
        match self.stash.take() {
            Some(p) => self.play = Some(p),
            None => self.go(self.replay_exit),
        }
    }

    pub(super) fn mode(&self) -> replay::Mode {
        if self.trial {
            replay::Mode::Trial
        } else {
            replay::Mode::Race
        }
    }

    /// The best kept run of the setup's mode on its map, when it fits the
    /// map's course as it is now.
    pub(super) fn best_replay(&self) -> Option<&Recording> {
        let course = self.course_of(self.settings.map)?;
        self.library.best_for(course, self.mode())
    }

    /// The kept replays, newest first, the best per course and mode marked.
    pub(super) fn replays_panels(&self) -> Vec<Value> {
        let mut items: Vec<Value> = self
            .library
            .listing()
            .iter()
            .enumerate()
            .map(|(k, (r, best))| {
                let mut words = Vec::new();
                if *best {
                    words.push(self.t("replays.best"));
                }
                let map = Map::from_key(&r.course);
                words.push(match map {
                    Some(m) => self.map_name(m),
                    None => r.course.clone(),
                });
                words.push(self.t(&format!("mode.{}", r.mode.key())));
                words.push(self.t(&format!("ape.{}", r.kind().key())));
                if r.lava {
                    words.push(self.t("setup.lava"));
                }
                words.push(match r.finish {
                    Some(t) => clock_text(t),
                    None if r.eaten => self.t("results.eaten"),
                    None => String::new(),
                });
                let course = map.and_then(|m| self.course_of(m));
                // a run of an older layout or version: watch-only, noted (T55)
                if let Some(note) = course.and_then(|c| r.older(c)) {
                    words.push(self.t(note));
                }
                json!({ "type": "button", "id": format!("replays.watch.{k}"),
                        "text": words.join(" · "), "quiet": !*best,
                        "disabled": !course.is_some_and(|c| r.watchable(c)) })
            })
            .collect();
        if items.is_empty() {
            items.push(json!({ "type": "list", "rows": [[self.t("replays.none"), ""]] }));
        }
        items.push(self.button("menu.back"));
        vec![json!({ "kind": "replays", "heading": self.t("replays.heading"), "items": items })]
    }

    /// The replay viewer's controls; the seek bar is in the HUD.
    pub(super) fn replay_panel(&self, p: &Play, pb: &Playback) -> Value {
        let play = if pb.playing && p.race.tick < pb.length() {
            "replay.pause"
        } else {
            "replay.play"
        };
        let speeds: Vec<Value> = replay::Speed::ALL
            .iter()
            .map(|s| json!({ "value": s.key(), "text": self.t(&format!("replay.speed.{}", s.key())) }))
            .collect();
        let views: Vec<Value> = replay::View::ALL
            .iter()
            .map(|v| json!({ "value": v.key(), "text": self.t(&format!("replay.camera.{}", v.key())) }))
            .collect();
        json!({ "kind": "replay", "heading": self.t("replay.heading"), "items": [
            { "type": "button", "id": "replay.play", "text": self.t(play) },
            { "type": "select", "id": "replay.speed", "label": self.t("replay.speed"),
              "value": pb.speed.key(), "options": speeds },
            { "type": "select", "id": "replay.camera", "label": self.t("replay.camera"),
              "value": pb.view.key(), "options": views },
            self.focus_picker(p),
            { "type": "button", "id": "replay.exit", "text": self.t("replay.exit"), "quiet": true },
        ]})
    }
}

/// Puts a save point back (scrubbing a replay).
pub(super) fn restore(p: &mut Play, save: replay::SavePoint) {
    replay::put_back(save, &mut p.race, &mut p.brains);
    p.prev = p.race.racers.iter().map(|r| r.ape.pos).collect();
    p.since_out = None;
    let f = &p.race.racers[p.focus.racer];
    p.chase = Chase::new(f.ape.pos, p.chase.yaw, p.race.course.terrain)
        .with_crowns(f.ape.pos, p.chase.crowns.clone())
        .with_temple(f.ape.pos, p.chase.temple.clone())
        .with_hulls(f.ape.pos, p.chase.hulls.clone());
}

/// One page tick of a replay: as many race ticks as the speed (or a seek)
/// wants, the player's recorded camera with them when it follows the
/// player. Returns the sounds of the followed ape and the racers that
/// called (TRIAGE T58), none while seeking.
pub(super) fn replay_tick(p: &mut Play) -> (Vec<Cue>, Vec<usize>) {
    let Some(mut pb) = p.replay.take() else {
        return (Vec::new(), Vec::new());
    };
    let n = pb.due(p.race.tick);
    let seeking = pb.seek.is_some();
    let mut cues = Vec::new();
    let mut calls = Vec::new();
    for _ in 0..n {
        pb.keep(&mut p.race, &p.brains);
        let c = pb.rec.at(p.race.tick);
        p.prev = p.race.racers.iter().map(|r| r.ape.pos).collect();
        pb.rec.play(&mut p.race, &mut p.brains);
        pb.orbit.tick();
        follow_speed(p);
        let f = &p.race.racers[p.focus.racer];
        let ahead = ahead_of(&p.race.course, f.ape.hint);
        // the player's own camera when it follows the player; else by itself
        let own = p.focus.racer == p.local && pb.rec.track.is_none();
        if own {
            p.chase.yaw = c.yaw;
            p.chase.pitch = c.pitch;
        }
        p.chase.follow(f.ape.pos, f.ape.vel, ahead, !own);
        if p.race.racers[p.local].out() {
            p.since_out = Some(p.since_out.map(|t| t + 1).unwrap_or(0));
        }
        if !seeking {
            cues.extend(
                p.race
                    .events
                    .iter()
                    .filter_map(|e| cue_for(e, p.focus.racer)),
            );
            calls.extend(pb.rec.calls_at(p.race.tick));
        }
    }
    p.replay = Some(pb);
    if n > 0 {
        step_chains(p);
    }
    (cues, calls)
}

/// The orbit and trackside replay cameras, on the followed ape.
pub(super) fn replay_camera(p: &Play, view: replay::View, alpha: i32) -> Camera {
    let i = p.focus.racer;
    let me = &p.race.racers[i];
    let at = p
        .prev
        .get(i)
        .copied()
        .unwrap_or(me.ape.pos)
        .lerp(me.ape.pos, alpha);
    match view {
        replay::View::Trackside => {
            // a camera every 60 m, beside the course and a little ahead
            let course = &p.race.course;
            let gap = int(60);
            let spot = ((me.progress / gap + 1) * gap).min(course.length);
            let q = course.point(spot);
            let side = crate::geom::right_of(course.heading_at(spot));
            let flip = if (spot / gap) % 2 == 0 { 1 } else { -1 };
            let foot = q.add(side.scale(int(14) * flip));
            let eye = v3(foot.x, course.terrain(foot.x, foot.z) + int(16), foot.z);
            let d = at.sub(eye);
            // zoom in on a far ape
            let fov =
                fx::mul(deg(70), fx::div(int(12), d.len().max(int(1)))).clamp(deg(10), deg(70));
            Camera {
                eye,
                fwd: d.norm(),
                fov,
            }
        }
        _ => {
            let cam = match &p.replay {
                Some(pb) => pb.orbit.camera(at, p.race.course.terrain),
                None => camera::Orbit::default().camera(at, p.race.course.terrain),
            };
            // on the bay, out of the sea and the ships (BP1)
            match &p.chase.hulls {
                Some(h) => {
                    let eye = h.kept_out(cam.eye);
                    Camera {
                        eye,
                        fwd: at.sub(eye).norm(),
                        ..cam
                    }
                }
                None => cam,
            }
        }
    }
}
