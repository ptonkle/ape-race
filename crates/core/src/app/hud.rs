//! The race HUD: place, time, best, the speedometer, the item and the minimap.

use super::*;

impl App {
    pub(super) fn hud(&self) -> Value {
        let Some(p) = &self.play else {
            return Value::Null;
        };
        let race = &p.race;
        let me = &race.racers[p.local];
        let big = match race.phase {
            Phase::Countdown(left) => {
                let n = (left + rules::TICKS_PER_SECOND - 1) / rules::TICKS_PER_SECOND;
                n.to_string()
            }
            Phase::Running if race.clock < rules::TICKS_PER_SECOND as u32 => self.t("hud.go"),
            _ => match (me.finish, me.eaten) {
                (Some(_), _) if p.since_out.unwrap_or(0) < rules::TICKS_PER_SECOND as u32 * 2 => {
                    self.t("hud.finished")
                }
                // the jaguar (TRIAGE T37): until the camera moves on
                (None, Some(_)) if p.since_out.unwrap_or(0) < EATEN_SPECTATE_AFTER => {
                    self.t("hud.eaten")
                }
                _ => String::new(),
            },
        };
        // place, time and item are the followed ape's (TRIAGE T24, T34)
        let me = &race.racers[p.focus.racer];
        let shown_clock = me.finish.unwrap_or(race.clock);
        let dial = crate::speedo::Dial::on(Map::of(&race.course));
        let item = me
            .item
            .map(|i| self.t(&format!("item.{}", i.key())))
            .unwrap_or_else(|| self.t("item.none"));
        json!({
            "place": [self.t("hud.place"), format!("{}/{}", me.place, race.racers.len())],
            "time": [self.t("hud.time"), clock_text(shown_clock)],
            "best": self.hud_best(p),
            "map": [self.t("setup.map"), self.map_name(Map::of(&p.race.course))],
            "speed": [self.t("hud.speed"), format!("{} {}", p.hype_speed / ONE, self.t("hud.unit"))],
            "speedo": {
                "value": crate::speedo::kmh(p.hype_speed).to_string(),
                "unit": self.t("hud.kmh"),
                "needle": dial.needle(crate::speedo::kmh(p.hype_speed)),
                "hype": dial.hype(crate::speedo::kmh(p.hype_speed)),
                "amount": decimal(&dial.amount_text(crate::speedo::kmh(p.hype_speed))),
                "shout": self.t(&format!("hud.hype{}", dial.hype(crate::speedo::kmh(p.hype_speed)))),
                // the splash of a call-out (TRIAGE T22): how strongly the
                // shout pops (0 to 1), the screen flash's opacity, and the
                // level whose shout pops while it shows
                "flash": decimal(&p.hype.flash_text()),
                "screen": decimal(&p.hype.screen_text()),
                "burst": p.hype.burst(),
            },
            "item": [self.t("hud.item"), item],
            // the course's bananas taken (TRIAGE T43)
            "bananas": [self.t("hud.bananas"), format!("{}/{}", me.ape.whole_bananas(), rules::BANANAS_FULL)],
            // (Aisle Ape: the shopping list in the bananas' place, each item
            // struck through once taken, GR-T2, GR-T9)
            "list": race.course.grocery.as_ref().map(|_| json!({
                "title": self.t("hud.list"),
                "items": (0..crate::grocery::LIST_ZONES).map(|z| json!({
                    "name": self.t(&format!("list.item{z}")),
                    "got": me.ape.list & (1 << z) != 0,
                })).collect::<Vec<_>>(),
            })),
            "item_key": me.item.map(|i| i.key()).unwrap_or(""),
            // the held item as a picture (TRIAGE T19)
            "item_icon": me.item.map(crate::items::icon_svg).unwrap_or_default(),
            "big": big,
            // a whole message (the jaguar, TRIAGE T37) is drawn smaller
            "big_class": if big.chars().count() > 10 { "big message" } else { "big" },
            "reward": self.random_map_flash(p).unwrap_or_else(|| self.reward_flash(me, race.tick)),
            "hint": if p.replay.is_some() {
                self.t("hud.replay")
            } else if p.focus.racer != p.local {
                self.t("hud.spectate")
            } else if let Some(names) = self.stalled_names(p) {
                format!("{} {names}", self.t("hud.stalled"))
            } else if self.pointer_locked {
                String::new()
            } else {
                self.t("hud.click")
            },
            "minimap": self.minimap(p),
            "replay": p.replay.as_ref().map(|pb| json!({
                "at": clock_text(race.clock),
                "of": clock_text(pb.length().saturating_sub(crate::race::COUNTDOWN as u32)),
                "value": (race.tick as u64 * 1000 / pb.length().max(1) as u64).min(1000),
                "max": 1000,
            })),
        })
    }

    /// A course reward's name, for a second after it came (group K: the
    /// mushroom's push, a catch-up vine).
    pub(super) fn reward_flash(&self, me: &crate::race::Racer, tick: u32) -> String {
        match me.reward {
            Some((r, at)) if tick < at + REWARD_FLASH => self.t(&format!("hud.reward.{}", r.key())),
            _ => String::new(),
        }
    }

    /// The HUD's Best: the race best, or in a Time Trial the next medal to
    /// aim for (TRIAGE T1).
    pub(super) fn hud_best(&self, p: &Play) -> Value {
        let map = Map::of(&p.race.course);
        if !p.trial {
            let best = self.settings.best(map, false).map(clock_text);
            return json!([
                self.t("results.best"),
                best.unwrap_or_else(|| self.t("item.none"))
            ]);
        }
        let b = self.settings.best(map, true);
        let medals = medals::of(map);
        match medals.next_target(b) {
            Some(trial::Medal::Author) if !medals.author_shown(b) => {
                json!([self.t("medal.author"), self.t("trial.hidden")])
            }
            Some(m) => json!([
                self.t(&format!("medal.{}", m.key())),
                clock_text(medals.target(m))
            ]),
            None => json!([
                self.t("results.best"),
                b.map(clock_text).unwrap_or_default()
            ]),
        }
    }

    /// The overhead minimap (decision 4): the course as a line and each ape
    /// as a dot, in a 0–1000 box, north up along the course's own extent.
    pub(super) fn minimap(&self, p: &Play) -> Value {
        let c = &p.race.course;
        let (mut lo_x, mut hi_x, mut lo_z, mut hi_z) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
        for q in &c.path {
            lo_x = lo_x.min(q.x);
            hi_x = hi_x.max(q.x);
            lo_z = lo_z.min(q.z);
            hi_z = hi_z.max(q.z);
        }
        let span = (hi_x - lo_x).max(hi_z - lo_z).max(ONE);
        let pad = 60;
        let to = |q: V3| {
            // seen from above with z (the way the course runs) up the map:
            // the world is right-handed, so facing +z your right is -x,
            // and +x goes to the LEFT of the map (TRIAGE T93: it was drawn
            // to the right, the view from underneath, a mirror image)
            let x =
                1000 - pad - ((q.x - lo_x) as i64 * (1000 - 2 * pad) as i64 / span as i64) as i32;
            let y =
                1000 - pad - ((q.z - lo_z) as i64 * (1000 - 2 * pad) as i64 / span as i64) as i32;
            (x, y)
        };
        // the run: on the Ski Slope down each fork's left branch, with the
        // right branches drawn beside (`branches`)
        let lane = |s: i32, k: usize| {
            c.point(s)
                .add(crate::geom::right_of(c.heading_at(s)).scale(c.lanes(s)[k].0))
        };
        let line: Vec<i32> = (0..c.path.len())
            .step_by(10)
            .flat_map(|i| {
                let (x, y) = to(lane(i as i32 * crate::course::STEP, 0));
                [x, y]
            })
            .collect();
        let branches: Vec<Vec<i32>> = c
            .ski
            .as_ref()
            .map(|ski| {
                ski.forks
                    .iter()
                    .map(|f| {
                        let step = int(10);
                        (0..=(f.to - f.from) / step)
                            .flat_map(|k| {
                                let (x, y) = to(lane(f.from + k * step, 1));
                                [x, y]
                            })
                            .collect()
                    })
                    .collect()
            })
            .unwrap_or_default();
        // Sector Ape's routes through space: each drawn through its vines'
        // anchors from where it splits to where it joins (the hidden one
        // through the wreck stays hidden)
        let mut branches = branches;
        if let Some(sl) = &c.sector {
            for path in crate::sectorscape::route_lines(c, sl) {
                branches.push(
                    path.into_iter()
                        .flat_map(|q| {
                            let (x, y) = to(q);
                            [x, y]
                        })
                        .collect(),
                );
            }
        }
        let apes: Vec<Value> = p
            .race
            .racers
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let (x, y) = to(r.ape.pos);
                let me = i == p.local;
                let colour = colour_of(r.ape.kind);
                let mut dot = json!({ "x": x, "y": y, "me": me, "colour": colour });
                // the colourblind mode tells the apes apart by shape too
                // (TRIAGE T33)
                if self.settings.colourblind {
                    let size = if me { MINIMAP_ME_DOT } else { MINIMAP_DOT };
                    dot["shape"] = json!(minimap_shape(r.ape.kind, x, y, size * 5 / 4));
                }
                dot
            })
            .collect();
        let mut apes = apes;
        if let Some(g) = &p.ghost {
            let (x, y) = to(g.race.racers[0].ape.pos);
            apes.insert(
                0,
                json!({ "x": x, "y": y, "me": false, "colour": "#96e1ff" }),
            );
        }
        let (fx_, fy) = to(*c.path.last().unwrap_or(&V3::ZERO));
        json!({
            "size": 1000, "stride": 2, "line": line, "branches": branches, "apes": apes, "finish": [fx_, fy],
            "dot": MINIMAP_DOT, "me_dot": MINIMAP_ME_DOT, "width": 22, "ring_width": 8,
            "track": "rgba(240, 230, 190, 0.75)", "finish_colour": "#ffffff",
            "ring": "rgba(0, 0, 0, 0.6)", "me_ring": "#ffe066",
            "circle": decimal("6.2832"),
        })
    }

    /// The map a race on Random runs on, its name shown at the start
    /// (TRIAGE T64).
    pub(crate) fn random_map_flash(&self, p: &Play) -> Option<String> {
        let early = match p.race.phase {
            Phase::Countdown(_) => true,
            Phase::Running => p.race.clock < rules::TICKS_PER_SECOND as u32 * 2,
            _ => false,
        };
        (p.random_map && early && p.replay.is_none()).then(|| {
            format!(
                "{}: {}",
                self.t("hud.random_map"),
                self.map_name(Map::of(&p.race.course))
            )
        })
    }
}

/// The minimap's dots' radii, in its 0–1000 box.
pub(super) const MINIMAP_DOT: i32 = 24;

pub(super) const MINIMAP_ME_DOT: i32 = 36;

/// A kind's minimap marker in the colourblind mode (TRIAGE T33), as the
/// corners of a polygon round (`x`, `y`) of radius `r`: a triangle, a
/// square, a pentagon, a hexagon, a diamond or a star, one per kind.
pub(super) fn minimap_shape(kind: crate::rules::Kind, x: i32, y: i32, r: i32) -> Vec<i32> {
    // (corners, turn of the first corner, inner radius percent for a star)
    let (n, turn, inner) = match kind.index() {
        0 => (3, 0, 100),
        1 => (4, fx::TURN / 8, 100),
        2 => (5, 0, 100),
        3 => (6, 0, 100),
        4 => (4, 0, 70),
        _ => (10, 0, 50),
    };
    (0..n)
        .flat_map(|k| {
            let a = turn + k * fx::TURN / n;
            let rr = if k % 2 == 1 { r * inner / 100 } else { r };
            [x + fx::sin(a) * rr / ONE, y - fx::cos(a) * rr / ONE]
        })
        .collect()
}

/// An ape kind's colour on the minimap.
pub(super) fn colour_of(k: Kind) -> &'static str {
    match k {
        Kind::Orangutan => "#e0702c",
        Kind::Gorilla => "#3a3a44",
        Kind::Chimpanzee => "#6b4a30",
        Kind::Gibbon => "#d8c49a",
        Kind::Proboscis => "#c8864a",
        Kind::Lemur => "#9a9aa4",
    }
}

/// How long a course reward's name shows on the HUD, ticks.
pub(super) const REWARD_FLASH: u32 = rules::TICKS_PER_SECOND as u32;
