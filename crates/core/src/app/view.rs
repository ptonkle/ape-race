//! The 3D view: the static scenery and each frame's view, instances and draws.

use super::*;

impl App {
    pub fn resize(&mut self, w: i32, h: i32) {
        self.view_w = w.max(1);
        self.view_h = h.max(1);
    }

    pub(super) fn aspect(&self) -> i32 {
        fx::div(self.view_w, self.view_h)
    }

    pub fn scene_version(&self) -> u32 {
        self.scene_version
    }

    pub(super) fn scenery(&self) -> &scene::Scenery {
        match self.shown {
            Shown::StageSet => &self.stage_set,
            Shown::World(m) => self.worlds[slot(m)].as_ref().unwrap_or(&self.stage_set),
        }
    }

    /// The static instances and the terrain for the current stage (its
    /// draws come with each frame, only the chunks near the camera).
    pub fn static_scene(&self) -> (Vec<i32>, Vec<u32>, Vec<i32>) {
        let s = self.scenery();
        (s.values.clone(), Vec::new(), s.terrain.clone())
    }

    /// This frame's view values, dynamic instances and draws.
    pub fn frame(&self) -> (Vec<i32>, Vec<i32>, Vec<u32>) {
        if let Some(g) = &self.gallery {
            return self.gallery_frame(g);
        }
        let cam = self.camera();
        // the sky swings through colours at top speed (TRIAGE T20)
        let (speed, metres) = match (&self.play, self.stage) {
            (Some(p), Stage::Race) if self.settings.speed_colours => {
                let me = &p.race.racers[p.local];
                // Sector Ape's speeds are about double: the colours come
                // at the same share of its dial
                let dial = crate::speedo::Dial::on(Map::of(&p.race.course));
                (dial.standard(p.colour_speed), me.progress)
            }
            // the unlock: the title's jungle goes wild, colours turning about
            // once a second, until another menu opens
            (_, Stage::Flyover) if self.celebrate => (int(60), (self.ticks as i32) * (ONE / 2)),
            _ => (0, 0),
        };
        let vivid = self.settings.colourblind;
        // each map has its own sky, fog and light (the City at dusk)
        // the Ski Slope's snow greys the sky and closes the fog in (a
        // blizzard whitens it and closes it right in); Mape's Tower's sky
        // follows the height and the zipper too
        let weather = self.weather_shown();
        let look = match (&self.play, self.stage) {
            (Some(p), Stage::Race) => scene::look_in(&p.race.course, cam.eye, weather),
            _ => self
                .course_of(self.shown_map())
                .map(|c| scene::look_in(c, cam.eye, weather))
                .unwrap_or_else(|| crate::snow::sky_in(weather, scene::look(self.shown_map()))),
        };
        // a squall's lightning lights the whole sky
        let look = if weather == crate::snow::Weather::Squall {
            crate::bayscape::flashed(look, self.ticks)
        } else {
            look
        };
        // the colourblind mode mutes the sky and the fog as well (TRIAGE T33)
        let (sky, fog) = if vivid {
            (
                crate::sky::grey(look.sky, crate::sky::MUTE),
                crate::sky::grey(look.fog, crate::sky::MUTE),
            )
        } else {
            (look.sky, look.fog)
        };
        let colours = crate::sky::colours(sky, fog, speed, metres);
        let world = crate::sky::world(speed, metres);
        let mute = if vivid { ONE } else { 0 };
        let view = render::view_values(
            &cam,
            self.aspect(),
            look.fog_range,
            colours,
            world,
            mute,
            (look.light_dir, look.light, look.ground),
            look.rim,
            look.far,
        );
        // in a race the scenery between the eye and the followed ape is
        // cut away (SECOND-ORDER BX2)
        let hole = match (&self.play, self.stage) {
            (Some(p), Stage::Race) => {
                let r = &p.race.racers[p.focus.racer];
                let prev = p.prev.get(p.focus.racer).copied().unwrap_or(r.ape.pos);
                Some(crate::see::Hole::on(prev.lerp(r.ape.pos, self.alpha)))
            }
            _ => None,
        };
        let mut view = view;
        view.extend(crate::see::values(hole));
        let (vines, batch) = match self.stage {
            Stage::Race => match &self.play {
                Some(p) => {
                    let apes: Vec<V3> = p
                        .race
                        .racers
                        .iter()
                        .zip(&p.prev)
                        .map(|(r, prev)| prev.lerp(r.ape.pos, self.alpha))
                        .collect();
                    scene::dynamic(&scene::Frame {
                        race: &p.race,
                        cam: &cam,
                        apes: &apes,
                        local: Some(p.focus.racer),
                        tick: self.ticks,
                        chains: &p.chains,
                        vivid,
                        gear: self.dress,
                        flashes: &p.flashes,
                        storm: weather == crate::snow::Weather::Squall,
                    })
                }
                None => (Batch::new(), Batch::new()),
            },
            Stage::Flyover => self
                .course_of(self.map())
                .map(|c| scene::flyover_dynamic(c, self.ticks, vivid, cam.eye))
                .unwrap_or_default(),
            Stage::Showcase => self.preview_batches(vivid),
            Stage::Lineup => {
                let mut b = Batch::new();
                let gear = self.dress.unwrap_or(models::Gear::on(self.map()));
                models::lineup(&mut b, self.ticks, gear);
                (Batch::new(), b)
            }
        };
        let (mut vines, mut batch) = (vines, batch);
        // Banana Bay Battle's sky and sea round the eye, in its weather (in
        // a race the swell at the race's tick, where the physics has it)
        if self.stage != Stage::Lineup {
            let shown = match (&self.play, self.stage) {
                (Some(p), Stage::Race) => Some((&p.race.course, p.race.tick)),
                _ => self.course_of(self.shown_map()).map(|c| (c, self.ticks)),
            };
            if let Some((c, tick)) = shown {
                if let Some(bay) = &c.bay {
                    let mut own = Batch::new();
                    crate::bayscape::dynamic(&mut own, c, bay, &cam, tick, self.ticks, weather);
                    // following JOJO MODE and the colourblind mode (BP5)
                    crate::bayscape::follow(
                        &mut own, colours.0, look.sky, world.0, world.1[0], vivid,
                    );
                    batch.append(own);
                }
                // Mape's Tower's sky: its dome, stars, sun and cloud sea
                if let Some(t) = &c.tower {
                    let mut own = Batch::new();
                    crate::towerscape::dynamic(&mut own, t, cam.eye, self.ticks, weather);
                    crate::bayscape::follow(
                        &mut own, colours.0, look.sky, world.0, world.1[0], vivid,
                    );
                    batch.append(own);
                }
            }
        }
        if self.snowing() {
            crate::snow::flakes(&mut batch, cam.eye, self.ticks, weather);
            batch.restyle(|c, g| crate::snow::in_storm(c, g, weather));
        }
        // Sunset Ruins' haze, clouds and sun follow JOJO MODE and the
        // colourblind mode's sky (RS9)
        if self.shown_map() == Map::Ruins
            && colours != (crate::ruinsmodels::SKY, crate::ruinsmodels::FOG)
        {
            crate::ruinsmodels::follow_sky(&mut batch, colours.0, colours.1);
        }
        let halos = (vines.take_halos(), batch.take_halos());
        let mut draws = self.scenery().draws_in(cam.eye, look.fog_range.1);
        let mut values = Vec::new();
        draws.extend(vines.finish_as(render::DYNAMIC_BUFFER, Some(render::VINES), &mut values));
        draws.extend(batch.finish_into(render::DYNAMIC_BUFFER, &mut values));
        if let (Stage::Race, Some(p)) = (self.stage, &self.play) {
            draws.extend(ghost_draws(p, self.alpha, self.ticks, &mut values));
        }
        // the soft glows (the City's wires) over everything else
        for h in [halos.0, halos.1] {
            draws.extend(h.finish_as(render::DYNAMIC_BUFFER, Some(render::HALO), &mut values));
        }
        (view, values, draws)
    }
}

/// The ghost ape, drawn into `values` tinted and glowing (the renderer
/// has no transparency); its draws.
pub(super) fn ghost_draws(p: &Play, alpha: i32, ticks: u32, values: &mut Vec<i32>) -> Vec<u32> {
    let Some(g) = &p.ghost else {
        return Vec::new();
    };
    let r = &g.race.racers[0];
    let at = g.prev.lerp(r.ape.pos, alpha);
    let mut b = Batch::new();
    let pose = scene::racer_pose(&g.race, 0, at, ticks);
    models::draw_in(&mut b, r.ape.kind, r.palette, &pose);
    let start = values.len();
    let draws = b.finish_into(render::DYNAMIC_BUFFER, values);
    for inst in values[start..].chunks_mut(render::INSTANCE_VALUES) {
        let c = render::mix([inst[12], inst[13], inst[14]], GHOST, ONE * 3 / 4);
        inst[12..15].copy_from_slice(&c);
        inst[15] = ONE * 3 / 5;
    }
    draws
}

/// The ghost's colour.
pub(super) const GHOST: render::Rgb = render::rgb(150, 225, 255);
