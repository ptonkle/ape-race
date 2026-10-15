//! The app's online screens and race hooks (PLAN P13): the Online screen
//! (name, Host, Join), the Lobby, the start of an online race, the per-tick
//! sync and hits, the online results, and the transport requests the page
//! carries out with `web/online.js`. The protocol itself is
//! `crate::online`; this file only connects it to the screens.

use serde_json::{json, Value};

use super::{ordinal, App, Play, Screen};
use crate::online::{self, Fin, Online, Refusal, Status};
use crate::race::{Driver, Event, Race};
use crate::rules::Kind;

/// How long the Copy button reads Copied, in milliseconds.
const COPIED_MS: i64 = 1500;

/// The lobby's Screech! button (TRIAGE T44) is hidden for now (T57). The
/// `lobby.screech` action, the online message and the sound stay, so it
/// comes back by setting this to true with the person's sound file
/// (SECOND-ORDER L4).
pub(crate) const SCREECH_BUTTON: bool = false;

/// The online part of the app.
#[derive(Default)]
pub struct OnlinePlay {
    /// this browser's id, from the page (`crypto.randomUUID()`)
    pub client_id: String,
    pub session: Option<Online>,
    /// the room code typed, or made by Host
    pub room: String,
    /// `?codes`: the pasted-code path, for the automated check
    pub codes: bool,
    pub invite: String,
    pub reply: String,
    /// a key in data/strings.json, or empty
    pub status: String,
    copied_ms: Option<i64>,
    link_copied_ms: Option<i64>,
    /// the page's own address (no query, no hash), for invite links
    pub page_url: String,
    /// transport requests for the page: {"op": ...}
    requests: Vec<Value>,
    /// the last messages of a session that has been left (its Leave or
    /// Dissolve), still to send
    parting: Vec<online::Outgoing>,
}

impl App {
    pub(super) fn online_flag(&mut self, id: &str, value: &str) {
        match id {
            "flag.codes" => self.online.codes = value == "on",
            "flag.client" => self.online.client_id = value.to_string(),
            "flag.page_url" => self.online.page_url = value.to_string(),
            // opened from an invite link: straight to the room
            "flag.room_link" if online::clean_code(value).is_some() => {
                self.go(super::Screen::Online);
                self.online_act("online.room", value);
                self.online_act("online.join", "");
            }
            _ => {}
        }
    }

    fn player_name(&self) -> String {
        let name = online::clean_name(&self.settings.name);
        if name.is_empty() {
            self.t("online.default_name")
        } else {
            name
        }
    }

    fn hosting(&self) -> bool {
        self.online.session.as_ref().is_some_and(|o| o.is_host())
    }

    fn open_session(&mut self, host: bool, request: Value) {
        let id = self.online.client_id.clone();
        let name = self.player_name();
        let kind = self.settings.kind;
        let mut o = if host {
            Online::host(&id, &name, kind, self.now_ms)
        } else {
            Online::join(&id, &name, kind, self.now_ms)
        }
        .with_palette(self.settings.palette());
        if host {
            let _ = o.set_map(self.settings.map, self.now_ms);
            let _ = o.set_random_map(self.settings.random_map, self.now_ms);
            let _ = o.set_ai_count(self.settings.ai_count as usize, self.now_ms);
            let _ = o.set_equal_stats(self.settings.equal_stats, self.now_ms);
            let _ = o.set_lava(self.settings.lava, self.now_ms);
        }
        self.online.session = Some(o);
        self.online.status = String::new();
        self.online.requests.push(request);
        self.go(Screen::Lobby);
    }

    /// The transport room for a code: the protocol version keeps browsers
    /// of different versions apart.
    fn room_name(code: &str) -> String {
        format!("ape-race-p{}-{code}", online::PROTOCOL)
    }

    /// Leaves the session: its last messages go out before the transport
    /// closes (the page flushes the outbox before the requests).
    fn leave_online(&mut self) {
        if let Some(o) = &mut self.online.session {
            o.leave();
            self.online.parting.extend(o.take_outbox());
            self.online.requests.push(json!({"op": "leave"}));
        }
        self.online.invite.clear();
        self.online.reply.clear();
    }

    pub(super) fn online_act(&mut self, id: &str, value: &str) {
        let now = self.now_ms;
        match id {
            // the Online screen's field, and the lobby's (TRIAGE T35)
            "online.name" | "lobby.name" => {
                self.settings.name = value
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(online::MAX_NAME_CHARS)
                    .collect();
                self.save = true;
                let name = self.player_name();
                if let Some(o) = &mut self.online.session {
                    let _ = o.set_name(&name, now);
                }
            }
            "lobby.screech" => {
                if let Some(o) = &mut self.online.session {
                    let _ = o.screech(now);
                }
            }
            "online.room" => {
                self.online.room = value
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .map(|c| c.to_ascii_uppercase())
                    .take(online::CODE_LEN)
                    .collect();
            }
            "online.host" => {
                let seed = self.seed ^ (now as u64).rotate_left(17) ^ self.ticks as u64;
                self.online.room = online::room_code(seed);
                let room = App::room_name(&self.online.room);
                self.open_session(true, json!({"op": "host_room", "room": room}));
            }
            "online.join" => match online::clean_code(&self.online.room) {
                Some(code) => {
                    self.online.room = code.clone();
                    let room = App::room_name(&code);
                    self.open_session(false, json!({"op": "join_room", "room": room}));
                }
                None => self.online.status = "online.bad_room".to_string(),
            },
            "online.copy" => self.online.copied_ms = Some(now),
            "online.copy_link" => self.online.link_copied_ms = Some(now),
            "online.invite" => self.online.invite = value.trim().to_string(),
            "online.reply" => self.online.reply = value.trim().to_string(),
            "online.host_code" => {
                if self.hosting() {
                    self.online.requests.push(json!({"op": "invite_code"}));
                } else {
                    self.online.room.clear();
                    self.open_session(true, json!({"op": "host_code"}));
                }
            }
            "online.join_code" => {
                self.online.room.clear();
                let invite = self.online.invite.clone();
                self.open_session(false, json!({"op": "join_code", "invite": invite}));
            }
            "online.connect" => {
                let reply = self.online.reply.clone();
                self.online
                    .requests
                    .push(json!({"op": "accept_reply", "reply": reply}));
            }
            "lobby.ape" | "lobby.ape.next" | "lobby.ape.prev" => {
                let n = Kind::ALL.len();
                let i = self.settings.kind.index();
                let k = match id {
                    "lobby.ape.next" => Kind::ALL[(i + 1) % n],
                    "lobby.ape.prev" => Kind::ALL[(i + n - 1) % n],
                    _ => Kind::from_key(value).unwrap_or(self.settings.kind),
                };
                self.settings.kind = k;
                self.save = true;
                if let Some(o) = &mut self.online.session {
                    let _ = o.set_kind(k, now);
                    // the new ape's own colours (TRIAGE T49)
                    let _ = o.set_palette(self.settings.palette(), now);
                }
            }
            "lobby.palette" => {
                if let Ok(p) = value.parse::<u8>() {
                    self.settings.set_palette(p);
                    self.save = true;
                    if let Some(o) = &mut self.online.session {
                        let _ = o.set_palette(self.settings.palette(), now);
                    }
                }
            }
            "lobby.ready" => {
                if let Some(o) = &mut self.online.session {
                    let ready = !o.lobby().ready;
                    let _ = o.set_ready(ready, now);
                }
            }
            "lobby.ai" => {
                if let (Ok(n), Some(o)) = (value.parse::<usize>(), &mut self.online.session) {
                    let _ = o.set_ai_count(n, now);
                    self.settings.ai_count = n.min(online::MAX_AI) as u8;
                    self.save = true;
                }
            }
            "lobby.map" => {
                if let Some(o) = &mut self.online.session {
                    if value == crate::map::RANDOM_MAP {
                        if o.set_random_map(true, now).is_ok() {
                            self.settings.random_map = true;
                            self.save = true;
                        }
                    } else if let Some(m) = crate::map::Map::from_key(value) {
                        if o.set_map(m, now).is_ok() && o.set_random_map(false, now).is_ok() {
                            self.settings.map = m;
                            self.settings.random_map = false;
                            self.save = true;
                        }
                    }
                }
            }
            "lobby.equal" => {
                let on = value == "on";
                if let Some(o) = &mut self.online.session {
                    let _ = o.set_equal_stats(on, now);
                    self.settings.equal_stats = on;
                    self.save = true;
                }
            }
            "lobby.lava" => {
                let on = value == "on";
                if let Some(o) = &mut self.online.session {
                    let _ = o.set_lava(on, now);
                    self.settings.lava = on;
                    self.save = true;
                }
            }
            "lobby.start" => {
                let seed = self.seed ^ (now as u64).rotate_left(29) ^ self.ticks as u64;
                if let Some(o) = &mut self.online.session {
                    o.set_items(self.settings.items);
                    let _ = o.start(seed, now);
                }
            }
            "lobby.leave" => {
                self.leave_online();
                self.online.session = None;
                self.go(Screen::Online);
            }
            _ => {}
        }
    }

    /// Rematch (host) or Main Menu from an online race.
    pub(super) fn online_race_menu(&mut self, id: &str) {
        if id == "menu.rematch" {
            let now = self.now_ms;
            if let Some(o) = &mut self.online.session {
                if o.rematch(now).is_ok() {
                    self.go(Screen::Lobby);
                }
            }
            return;
        }
        self.leave_online();
        self.online.session = None;
        self.go(Screen::Title);
    }

    /// A transport event from the page: `kind` is "connected", "message",
    /// "disconnected" or "error" (then `text` is a key in data/strings.json).
    pub fn net_event(&mut self, kind: &str, peer: &str, text: &str, host: bool) {
        let now = self.now_ms;
        if kind == "error" {
            self.online.status = text.to_string();
            return;
        }
        let Some(o) = &mut self.online.session else {
            return;
        };
        match kind {
            "connected" => o.peer_connected(peer, host, now),
            "message" => o.receive(peer, text, now),
            "disconnected" => o.peer_left(peer, now),
            _ => {}
        }
    }

    /// A code for the pasted-code path, from the page's transport.
    pub fn net_code(&mut self, which: &str, code: &str) {
        match which {
            "invite" => self.online.invite = code.to_string(),
            "reply" => self.online.reply = code.to_string(),
            _ => {}
        }
    }

    /// Messages for the page to send: [{"to": peer or null (everyone), "text"}].
    pub fn take_outbox(&mut self) -> String {
        let mut out = std::mem::take(&mut self.online.parting);
        if let Some(o) = &mut self.online.session {
            out.extend(o.take_outbox());
        }
        serde_json::to_string(&out).unwrap_or_else(|_| "[]".to_string())
    }

    /// Once a frame: the session's retries, the race start, a rematch, and
    /// the end of a lobby.
    pub(super) fn online_frame(&mut self) {
        let now = self.now_ms;
        let Some(o) = &mut self.online.session else {
            return;
        };
        o.update(now);
        for _ in 0..o.take_screeches() {
            self.sounds.push(crate::sound::SCREECH.to_string());
        }
        if let Some(start) = o.take_race_start() {
            // the host's map, built here if this browser has not shown it
            self.ensure_world(start.map);
            let course = self
                .course_of(start.map)
                .cloned()
                .unwrap_or_else(|| start.map.course());
            let mut race = Race::new(course, &start.entries, start.equal_stats, start.seed);
            // only the host runs the AI apes, at its own difficulty
            race.ai_level = self.settings.ai_level;
            // the host's items (TRIAGE T42) and Floor is Lava (T37)
            race.items = start.items;
            race.lava = start.lava;
            self.begin(race, start.local_index);
            if let Some(p) = &mut self.play {
                p.online = true;
                p.random_map = start.random_map;
            }
            return;
        }
        let status = o.status().clone();
        match status {
            // the host pressed Rematch
            Status::Lobby if self.screen == Screen::Race => self.go(Screen::Lobby),
            Status::Refused { .. } | Status::Dissolved | Status::Lost
                if self.screen == Screen::Lobby =>
            {
                self.online.status = status_key(&status).to_string();
                self.online.requests.push(json!({"op": "leave"}));
                self.online.session = None;
                self.go(Screen::Online);
            }
            _ => {}
        }
    }

    /// After each tick of an online race: hits this browser's items made on
    /// apes it does not own go to their owners; states and finishes go out;
    /// remote apes move; hits from elsewhere land on this browser's apes.
    pub(super) fn online_step(&mut self, p: &mut Play) {
        let now = self.now_ms;
        let Some(o) = &mut self.online.session else {
            return;
        };
        for e in &p.race.events {
            if let Event::Hit { who, by, item } = *e {
                if !o.owns(who) {
                    o.send_hit(who, by, item);
                }
            }
        }
        o.sync_race(now, &mut p.race);
        for h in o.take_hits() {
            crate::items::apply_hit(&mut p.race, h.victim, h.by, h.item);
        }
    }

    // ---- what the page shows ----

    fn status_text(&self) -> String {
        if !self.online.status.is_empty() {
            return self.t(&self.online.status);
        }
        match &self.online.session {
            None => self.t("online.idle"),
            Some(o) => self.t(status_key(o.status())),
        }
    }

    /// The room as a link: opening it joins the room (TRIAGE row 27).
    pub(super) fn invite_link(&self) -> String {
        format!("{}#room={}", self.online.page_url, self.online.room)
    }

    fn link_item(&self) -> Value {
        let copied = self
            .online
            .link_copied_ms
            .is_some_and(|t| self.now_ms - t < COPIED_MS);
        json!({
            // just the button: the link itself is not shown (TRIAGE row 27)
            "type": "copy", "id": "online.copy_link", "label": "", "bare": true,
            "value": "", "copy": self.invite_link(),
            "text": self.t(if copied { "online.copied" } else { "online.copy_link" }),
            "disabled": self.online.room.is_empty() || self.online.page_url.is_empty(),
        })
    }

    fn copy_item(&self) -> Value {
        let copied = self
            .online
            .copied_ms
            .is_some_and(|t| self.now_ms - t < COPIED_MS);
        json!({
            "type": "copy", "id": "online.copy", "label": self.t("online.room_code"),
            "value": self.online.room, "copy": self.online.room,
            "text": self.t(if copied { "online.copied" } else { "online.copy" }),
            "disabled": self.online.room.is_empty(),
        })
    }

    fn status_list(&self) -> Value {
        json!({ "type": "list", "rows": [[self.t("online.status"), self.status_text()]] })
    }

    fn codes_panel(&self, host: bool, joiner: bool) -> Value {
        let mut items = vec![
            json!({"type": "field", "id": "online.invite", "label": self.t("online.invite_code"),
                   "value": self.online.invite, "multiline": true}),
            json!({"type": "field", "id": "online.reply", "label": self.t("online.reply_code"),
                   "value": self.online.reply, "multiline": true}),
        ];
        if !host && !joiner {
            items.push(self.button("online.host_code"));
            items.push(self.button("online.join_code"));
        }
        if host {
            items.push(self.button("online.host_code"));
            items.push(self.button("online.connect"));
        }
        json!({ "kind": "codes", "heading": self.t("online.codes"), "items": items })
    }

    pub(super) fn online_panels(&self) -> Vec<Value> {
        let mut out = vec![
            json!({ "kind": "setup", "heading": self.t("menu.online"), "items": [
                {"type": "field", "id": "online.name", "label": self.t("online.name"),
                 "value": self.settings.name, "placeholder": self.t("online.default_name")},
                {"type": "field", "id": "online.room", "label": self.t("online.room_code"),
                 "value": self.online.room, "placeholder": self.t("online.room_hint")},
                self.status_list(),
                self.button("online.host"),
                self.button("online.join"),
                self.button("menu.back"),
            ]}),
        ];
        if self.online.codes {
            out.push(self.codes_panel(false, false));
        }
        out
    }

    pub(super) fn lobby_panels(&self) -> Vec<Value> {
        let Some(o) = &self.online.session else {
            return vec![
                json!({ "kind": "setup", "heading": self.t("online.lobby"), "items": [
                    self.status_list(), self.button("lobby.leave")
                ]}),
            ];
        };
        let view = o.lobby();
        let ape = |k: Kind| self.t(&format!("ape.{}", k.key()));
        let rows: Vec<Value> = view
            .seats
            .iter()
            .map(|s| {
                let ready = if s.ai {
                    self.t("player.ai")
                } else if s.host {
                    self.t("online.host_seat")
                } else if s.ready {
                    self.t("online.readied")
                } else {
                    self.t("online.not_ready")
                };
                let ping = match s.ping_ms {
                    Some(ms) => format!("{ms} {}", self.t("online.ms")),
                    None => self.t("item.none"),
                };
                // the ape, and its colours when not the classic (TRIAGE T49)
                let worn = if s.palette == 0 {
                    ape(s.kind)
                } else {
                    format!("{} ({})", ape(s.kind), self.palette_name(s.kind, s.palette))
                };
                json!([s.name, worn, ready, ping, s.me])
            })
            .collect();
        let mut seats = vec![
            json!({ "type": "table", "head": [
                self.t("results.name"), self.t("results.ape"), self.t("online.ready"), self.t("online.ping")
            ], "rows": rows }),
            self.status_list(),
        ];
        if !self.online.room.is_empty() {
            seats.insert(0, self.copy_item());
            seats.insert(1, self.link_item());
        }
        let kinds: Vec<Value> = Kind::ALL
            .iter()
            .map(|k| json!({ "value": k.key(), "text": ape(*k) }))
            .collect();
        let stats = self.settings.kind.race_stats(view.equal_stats);
        let mut mine = vec![
            json!({"type": "field", "id": "lobby.name", "label": self.t("online.name"),
                   "value": self.settings.name, "placeholder": self.t("online.default_name")}),
            json!({ "type": "picker", "id": "lobby.ape", "label": self.t("setup.ape"),
                    "value": self.settings.kind.key(), "options": kinds,
                    "text": ape(self.settings.kind) }),
            self.palette_select("lobby.palette"),
            json!({ "type": "stats", "label": self.t("setup.stats"), "rows": self.stats_rows(&stats) }),
        ];
        if view.is_host {
            mine.insert(
                0,
                self.map_tiles("lobby.map", view.map, view.random_map, false),
            );
            let options: Vec<Value> = (0..=view.max_ai)
                .map(|n| json!({ "value": n.to_string(), "text": n.to_string() }))
                .collect();
            mine.push(
                json!({ "type": "select", "id": "lobby.ai", "label": self.t("online.ai_count"),
                              "value": view.ai_count.to_string(), "options": options }),
            );
            mine.push(
                json!({ "type": "toggle", "id": "lobby.equal", "label": self.t("setup.equal"),
                              "value": view.equal_stats }),
            );
            mine.push(self.lava_toggle("lobby.lava"));
            mine.extend(self.item_toggles());
            let mut start = self.button("lobby.start");
            start["disabled"] = json!(!view.can_start);
            mine.push(start);
        } else {
            // the host's pick, read-only (TRIAGE T54)
            mine.insert(
                0,
                self.map_tiles("lobby.map", view.map, view.random_map, true),
            );
            mine.push(json!({ "type": "list", "rows": [[self.t("setup.equal"),
                self.t(if view.equal_stats { "on" } else { "off" })]] }));
            mine.push(json!({ "type": "list", "rows": [[self.t("setup.lava"),
                self.t(if view.lava { "on" } else { "off" })]] }));
            let mut ready = self.button("lobby.ready");
            ready["text"] = json!(self.t(if view.ready {
                "online.cancel_ready"
            } else {
                "online.ready_up"
            }));
            ready["disabled"] = json!(view.status != Status::Lobby);
            mine.push(ready);
        }
        // the screech: anyone, any time in the lobby (TRIAGE T44); hidden
        // for now (T57) until the person's sound file arrives (SECOND-ORDER L4)
        if SCREECH_BUTTON {
            let mut screech = self.quiet_button("lobby.screech");
            screech["disabled"] = json!(view.status != Status::Lobby);
            mine.push(screech);
        }
        mine.push(self.button("lobby.leave"));
        let mut out = vec![
            json!({ "kind": "setup", "heading": self.t("online.lobby"), "items": seats }),
            json!({ "kind": "setup", "heading": self.t("setup.heading"), "items": mine }),
        ];
        if self.online.codes {
            out.push(self.codes_panel(view.is_host, !view.is_host));
        }
        out
    }

    /// The menu overlay (Esc; the race runs on) and the results.
    pub(super) fn online_race_panels(&self, p: &Play) -> Vec<Value> {
        let host = self.hosting();
        let finished = super::results_due(p);
        if p.paused {
            return vec![
                json!({ "kind": "overlay", "heading": self.t("menu.online"), "items": [
                    self.button("menu.resume"),
                    self.button("menu.main"),
                ]}),
            ];
        }
        if !finished {
            return Vec::new();
        }
        // spectating while others race (TRIAGE T34); the replay once all
        // are in (TRIAGE T24: watching it stops this race's sending)
        let done = self
            .online
            .session
            .as_ref()
            .is_some_and(|o| o.results_final());
        let mut items = vec![self.online_results()];
        if !done {
            items.push(self.focus_picker(p));
        }
        if host {
            items.push(self.button("menu.rematch"));
        } else {
            items.push(json!({ "type": "list", "rows": [[self.t("online.status"), self.t("online.wait_rematch")]] }));
        }
        if done {
            items.push(self.quiet_button("menu.replay"));
        }
        items.push(self.button("menu.main"));
        let kind = if done { "results" } else { "spectate" };
        vec![json!({ "kind": kind, "heading": self.t("results.heading"), "items": items })]
    }

    fn online_results(&self) -> Value {
        let rows: Vec<Value> = self
            .online
            .session
            .as_ref()
            .map(|o| o.results())
            .unwrap_or_default()
            .into_iter()
            .map(|r| {
                let place = r.place.map(ordinal).unwrap_or_else(|| self.t("item.none"));
                let shown = match r.fin {
                    Fin::Done(t) => super::clock_text(t),
                    Fin::Racing => self.t("results.racing"),
                    Fin::Dnf => self.t("results.out"),
                    Fin::Eaten(..) => self.t("results.eaten"),
                };
                json!([
                    place,
                    r.name,
                    self.t(&format!("ape.{}", r.kind.key())),
                    shown,
                    r.me
                ])
            })
            .collect();
        json!({ "type": "table", "head": [
            self.t("results.place"), self.t("results.name"), self.t("results.ape"), self.t("results.time")
        ], "rows": rows })
    }

    /// Adds the transport requests, and for the online check whether a
    /// remote ape has left its start.
    pub(super) fn online_ui(&mut self, v: &mut Value) {
        if !self.online.requests.is_empty() {
            v["net_requests"] = Value::Array(std::mem::take(&mut self.online.requests));
        }
        if let Some(p) = &self.play {
            if p.online {
                let moved = p.race.racers.iter().enumerate().any(|(i, r)| {
                    r.driver == Driver::Remote && r.ape.pos != p.race.course.starts[i % 4]
                });
                v["remote_moved"] = json!(moved);
            }
        }
    }
}

/// The strings key for a session's status.
fn status_key(s: &Status) -> &'static str {
    match s {
        Status::Joining => "online.connecting",
        Status::Lobby => "online.in_lobby",
        Status::Racing => "online.racing",
        Status::Refused { reason } => match reason {
            Refusal::WrongVersion { .. } => "online.wrong_version",
            Refusal::Full => "online.full",
            Refusal::Started => "online.already_started",
        },
        Status::Dissolved => "online.dissolved",
        Status::Lost => "online.lost",
        Status::Left => "online.idle",
    }
}

#[cfg(test)]
#[path = "online_play_tests.rs"]
mod tests;
