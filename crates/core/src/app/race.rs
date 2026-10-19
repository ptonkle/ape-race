//! The race flow: starting a race, showing it, and each tick of it.

use super::*;

impl App {
    pub(super) fn start_race(&mut self) {
        let mut entries = vec![Entry {
            name: self.t("player.you"),
            kind: self.settings.kind,
            driver: Driver::Local,
            palette: self.settings.palette(),
            costume: self.settings.costume,
        }];
        let mut d = fx::Dice(self.seed ^ self.ticks as u64);
        // the AI picks kinds other than the player's where it can
        let mut pool: Vec<Kind> = Kind::ALL
            .into_iter()
            .filter(|k| *k != self.settings.kind)
            .collect();
        let opponents = if self.trial {
            0
        } else {
            // the lobby may have set 0 (online, people race people)
            self.settings.ai_count.max(1)
        };
        for _ in 0..opponents {
            let k = pool.remove(d.below(pool.len() as u32) as usize);
            entries.push(Entry {
                name: self.t(&format!("ape.{}", k.key())),
                kind: k,
                driver: Driver::Ai,
                palette: crate::race::palette_apart(crate::race::worn(&entries), k),
                costume: crate::costume::Costume::Own,
            });
        }
        // a tournament (TRIAGE T103) races the same lineup every race: it is
        // fixed when the setup starts it
        if !self.trial
            && self.settings.tournament
            && self.tour.is_none()
            && self.screen == Screen::Setup
        {
            self.tour = Some(self.new_tour(&entries));
        }
        if let Some(t) = &self.tour {
            entries = App::tour_entries(t);
        }
        // Random (TRIAGE T64): a map other than the last, from the page's
        // seed and the clock; it becomes the last map (the preview's)
        if self.settings.random_map {
            self.settings.map = Map::random(d.next(), self.settings.map);
            self.save = true;
        }
        let map = self.settings.map;
        self.ensure_world(map);
        let course = self.course_of(map).cloned().unwrap_or_else(|| map.course());
        let mut race = Race::new(course, &entries, self.settings.equal_stats, d.next());
        // a Time Trial: crates give only the Speed Boost (TRIAGE T13);
        // otherwise the items ticked on the setup (T42)
        race.items = if self.trial {
            crate::items::ItemSet::only(crate::race::Item::Boost)
        } else {
            self.settings.items
        };
        race.ai_level = self.settings.ai_level;
        // Floor is Lava (TRIAGE T37); never in a Time Trial
        race.lava = self.settings.lava && !self.trial;
        let record = Recording::of(&race, 0, self.mode());
        // (after `begin`, which keeps the run being left: it may be the best)
        self.begin(race, 0);
        let ghost = self
            .best_replay()
            .filter(|_| self.trial && self.settings.ghost)
            .and_then(|r| self.course_of(map).and_then(|c| Ghost::new(r, c)));
        if let Some(p) = &mut self.play {
            p.trial = self.trial;
            p.record = Some(record);
            p.ghost = ghost;
            p.random_map = self.settings.random_map;
        }
    }

    /// Starts showing a race in which this browser plays ape `local`.
    pub fn begin(&mut self, race: Race, local: usize) {
        let at = race.racers[local].ape.pos;
        let mut chase = Chase::new(at, race.course.start_yaw, race.course.terrain);
        // on the Ski Slope the eye keeps out of the conifers (SW9)
        if let Some(ski) = &race.course.ski {
            let crowns = crate::skiscape::crowns(&race.course, ski);
            chase = chase.with_crowns(at, crowns.into());
        }
        // on Sunset Ruins the eye keeps inside the temple (RS25)
        if let Some(r) = &race.course.ruins {
            let temple = crate::camera::Temple::of(r);
            chase = chase.with_temple(at, Some(std::sync::Arc::new(temple)));
        }
        // on Rattlesnake Ravine the eye keeps inside the rooms and out of
        // the rock
        if let Some(k) = crate::camera::Keeps::of(&race.course) {
            chase = chase.with_keeps(at, Some(std::sync::Arc::new(k)));
        }
        // on Banana Bay Battle the eye keeps out of the ships (BP1)
        if let Some(h) = crate::camera::Hulls::of(&race.course) {
            chase = chase.with_hulls(at, Some(std::sync::Arc::new(h)));
        }
        // Sector Ape runs about twice as fast: the view opens wider
        if race.course.sector.is_some() {
            chase.pace = crate::camera::SPACE_PACE;
        }
        let prev = race.racers.iter().map(|r| r.ape.pos).collect();
        // each AI ape thinks at the race's difficulty, with its own dice
        // from the race's seed, so a replay rebuilds the same brains
        let brains = (0..race.racers.len())
            .map(|i| Brain::new(race.ai_level, race.dice.0 ^ (0x9E37_79B9 * (i as u64 + 1))))
            .collect();
        if let Some(mut old) = self.play.take() {
            self.keep_recording(&mut old);
        }
        self.go(Screen::Race);
        self.play = Some(Play {
            race,
            brains,
            chase,
            local,
            paused: false,
            online: false,
            prev,
            since_out: None,
            hype_speed: 0,
            colour_speed: 0,
            hype: crate::hype::Hype::default(),
            new_best: false,
            new_costume: None,
            chains: Vec::new(),
            flashes: Vec::new(),
            trial: false,
            record: None,
            kept: None,
            focus: Focus::on(local),
            out_for: 0,
            danced: 0,
            pan: None,
            replay: None,
            ghost: None,
            random_map: false,
            still: None,
        });
        self.settle_scene();
    }

    pub(super) fn step_race(&mut self, p: &mut Play) {
        p.prev = p.race.racers.iter().map(|r| r.ape.pos).collect();
        let mine = self.controls(p.chase.yaw, p.chase.pitch);
        // an online race is recorded as a track of every ape (TRIAGE T24)
        if p.online && p.record.is_none() && p.race.tick == 0 {
            p.record = Some(Recording::online(&p.race, p.local));
        }
        if let Some(rec) = &mut p.record {
            if rec.track.is_none() && rec.controls.len() == p.race.tick as usize && rec.has_room() {
                rec.controls.push(mine);
            }
        }
        replay::drive(&mut p.race, &mut p.brains, p.local, mine);
        if let Some(g) = &mut p.ghost {
            g.step();
        }
        if p.online {
            self.online_step(p);
        }
        if let Some(track) = p.record.as_mut().and_then(|r| r.track.as_mut()) {
            track.record(&p.race);
        }
        follow_speed(p);
        self.hype_tick(p);
        step_chains(p);
        // the camera follows the focused ape; on another's, by itself
        let spectating = p.focus.racer != p.local;
        let f = &p.race.racers[p.focus.racer];
        let ahead = ahead_of(&p.race.course, f.ape.hint);
        p.chase.follow(
            f.ape.pos,
            f.ape.vel,
            ahead,
            self.settings.auto_camera || spectating,
        );
        spectate(p, self.alpha);
        let me = &p.race.racers[p.local];
        if me.out() {
            if let (None, Some(t)) = (p.since_out, me.finish) {
                // a personal best is kept with the settings, one per map
                // and mode
                let best = self.settings.best_mut(Map::of(&p.race.course), p.trial);
                p.new_best = best.map(|b| t < b).unwrap_or(true);
                if p.new_best {
                    *best = Some(t);
                    self.save = true;
                }
                // under the map's Author time, in any mode: its costume
                // (TRIAGE T89)
                if let Some(c) = Costume::earned(Map::of(&p.race.course), t) {
                    if self.settings.unlock(c) {
                        p.new_costume = Some(c);
                        self.save = true;
                    }
                }
            }
            let since = p.since_out.map(|t| t + 1).unwrap_or(0);
            p.since_out = Some(since);
            if since == KEEP_AFTER_FINISH
                || (since > KEEP_AFTER_FINISH && since.is_multiple_of(KEEP_AGAIN))
            {
                self.keep_recording(p);
            }
        }
        // the sounds of the ape being watched
        for e in p.race.events.clone() {
            if let Some(cue) = cue_for(&e, p.focus.racer) {
                self.sounds.push(cue.key().to_string());
            }
        }
        // Sector Ape's flashes: an ape put back on a deck, the door shut
        if let Some(at) = crate::sectorscape::flash_of(&p.race) {
            p.flashes.extend(at);
            let tick = p.race.tick;
            p.flashes
                .retain(|f| tick.saturating_sub(f.tick) < crate::sectorscape::FLASH_TICKS);
        }
    }
}

/// The followed ape's horizontal speed, smoothed: the speedometer, the
/// colours and the wind follow it (TRIAGE rows 19 and 22, T30).
pub(super) fn follow_speed(p: &mut Play) {
    let flat = p.race.racers[p.focus.racer].ape.vel.len_xz();
    p.hype_speed += (flat - p.hype_speed) / 30;
    p.colour_speed += (flat - p.colour_speed) / 120;
}

/// Vines near the local ape are chains that bend (TRIAGE row 12); those
/// further off keep their last shape until the ape comes near.
pub(super) fn step_chains(p: &mut Play) {
    let c = &p.race.course;
    // a summoned vine (TRIAGE T12) adds a chain or, put where its slot's
    // last one was, starts its own: the other chains keep their shape
    p.chains.truncate(c.vines.len());
    for i in 0..c.vines.len() {
        let a = c.anchor(i, p.race.tick);
        let fresh = || crate::vinesim::Chain::hanging_along(a, c.vines[i].length, c.hang(i));
        match p.chains.get(i) {
            None => p.chains.push(fresh()),
            Some(ch) if i >= p.race.base_vines && ch.pts[0].dist(a) > int(2) => {
                p.chains[i] = fresh()
            }
            Some(_) => {}
        }
    }
    let me = p.race.racers[p.focus.racer].ape.pos;
    for i in 0..c.vines.len() {
        let a = c.anchor(i, p.race.tick);
        if a.flat().sub(me.flat()).len() > int(170) {
            continue;
        }
        let hands = p
            .race
            .racers
            .iter()
            .find(|r| r.ape.swinging() == Some(i))
            .map(|r| r.ape.pos);
        p.chains[i].step(a, c.vines[i].length, hands, p.race.tick, i as u32);
    }
}
