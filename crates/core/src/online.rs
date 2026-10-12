//! Online play for up to four browsers (PLAN P13): the lobby, the start, the
//! apes' states, item hits, finish times and the results. A pure state
//! machine: text comes in with [`Online::receive`], text goes out through
//! [`Online::take_outbox`], and time comes in as `now_ms` arguments. The
//! transport is `web/online.js` (WebRTC, copied from Mandible); it only
//! carries the text.
//!
//! # No lockstep
//!
//! Each browser simulates its own ape with no delay and sends that ape's
//! state about 20 times a second ([`STATE_EVERY_MS`]). The host also runs the
//! AI apes and sends theirs. Every other ape is drawn where its owner said it
//! was about [`INTERP_DELAY_MS`] ago, interpolated between two states, or
//! carried forward along its velocity for at most [`EXTRAPOLATE_MS`] when
//! states are late. Item hits are sent to the victim's owner, which applies
//! them to its own ape. Finish times are reported to the host, which
//! publishes the results.
//!
//! # Roles
//!
//! The **host** owns the lobby: who sits where, each seat's ape, the AI count
//! (0–3, never more than four apes in all), Equal Stats, and each joiner's
//! Ready mark. It rebroadcasts the roster every second, since lobby messages
//! may be lost. A **joiner** repeats its Hello until seated (or refused: a
//! full lobby, a race under way, or another [`PROTOCOL`]), and repeats an
//! ape choice or a Ready mark until the roster shows it. Everyone pings
//! everyone every half second; the round trips appear in the lobby view.
//!
//! The host's Start is repeated to each joiner until it is acknowledged. Every
//! browser builds the same entries (humans in seat order, then the AI apes),
//! so ape `i` is the same ape everywhere. A **Rematch** (host only) goes back
//! to the lobby with the same seats and settings, every Ready mark cleared;
//! each Rematch is a new *round*, and a race message of an old round is
//! ignored.
//!
//! # For the integrator (app.rs and app.js)
//!
//! 1. **Open.** Host: `Online::host(client_id, name, kind, now_ms)` and give
//!    the page a code: [`room_code`]`(seed)` for a room code, or the pasted
//!    code path. Joiner: `Online::join(client_id, name, kind, now_ms)`;
//!    [`clean_code`] checks a typed room code. `client_id` is any id that is
//!    stable for this browser tab and differs between browsers (online.js's
//!    `myId()` once it resolves, or a random id the page made with
//!    `crypto.randomUUID()`).
//! 2. **Transport events** (online.js's `onEvent`): `connected` →
//!    [`Online::peer_connected`]`(peer, host)`; `message` →
//!    [`Online::receive`]`(peer, text, now_ms)`; `disconnected` →
//!    [`Online::peer_left`]`(peer, now_ms)`.
//! 3. **Every frame:** [`Online::update`]`(now_ms)`, then send everything in
//!    [`Online::take_outbox`]: `to: None` → `online.broadcast(text)`,
//!    `to: Some(peer)` → `online.send(peer, text)`.
//! 4. **Lobby screen:** draw [`Online::lobby`]. Buttons call
//!    [`Online::set_kind`], [`Online::set_ready`], and on the host
//!    [`Online::set_ai_count`], [`Online::set_equal_stats`] and
//!    [`Online::start`]`(seed, now_ms)` (refused unless every joined human is
//!    Ready and there are at least two apes).
//! 5. **Race start:** when [`Online::take_race_start`] gives a [`RaceStart`],
//!    build `Race::new(Course::jungle(), &start.entries, start.equal_stats,
//!    start.seed)`. The local player drives `start.local_index`; on the host,
//!    AI apes are `Driver::Ai`; everything else is `Driver::Remote`.
//! 6. **During the race, every frame after stepping the race:**
//!    [`Online::sync_race`]`(now_ms, &mut race)`. It sends this browser's
//!    apes' states (throttled to 20 Hz), reports their finishes, moves the
//!    remote apes (`Race::set_remote`) and records remote finishes
//!    (`Race::remote_finish`). Then, for each `Event::Hit { who, by, item }`
//!    the race raised whose victim `who` this browser does not own
//!    (`!online.owns(who)`), call [`Online::send_hit`]`(who, by, item)`; and
//!    apply every hit from [`Online::take_hits`] to the local race with
//!    `items::apply_hit(race, hit.victim, hit.by, hit.item)`. A hit this
//!    browser sends to an ape it owns comes straight back from `take_hits`.
//! 7. **Results:** [`Online::results`] (rows in place order) and
//!    [`Online::results_final`]. A player who left mid-race is a DNF.
//! 8. **Rematch / leave:** host [`Online::rematch`]`(now_ms)`; joiners follow
//!    the roster back to the lobby on their own ([`Online::status`] becomes
//!    `Lobby`). [`Online::leave`] then online.js `leave()`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::ape::{Ape, Mode};
use crate::fx::{self, Dice, ONE};
use crate::geom::V3;
use crate::race::{Driver, Entry, Item, Race};
use crate::rules::{Kind, TICKS_PER_SECOND};

/// The protocol version. A joiner with another one is refused.
pub const PROTOCOL: u32 = 1;
/// The most apes in a race, humans and AI together.
pub const MAX_APES: usize = 4;
/// The most AI apes the host may add.
pub const MAX_AI: usize = 3;
/// How often a joiner repeats a Hello, an ape choice, a Ready mark or a
/// finish, and the host repeats a Start, until it is seen.
pub const RETRY_MS: i64 = 500;
/// How often the host rebroadcasts the roster, and the results.
pub const ROSTER_EVERY_MS: i64 = 1000;
/// How often everyone pings everyone.
pub const PING_EVERY_MS: i64 = 500;
/// How often an ape's state is sent: 20 times a second.
pub const STATE_EVERY_MS: i64 = 50;
/// How far behind its owner a remote ape is drawn.
pub const INTERP_DELAY_MS: i64 = 100;
/// How long a remote ape is carried along its velocity when states are late.
pub const EXTRAPOLATE_MS: i64 = 200;
/// The most the host waits for its Start to reach the joiners before its own
/// countdown begins (half the slowest round trip, at most this).
pub const MAX_START_LEAD_MS: i64 = 250;
/// States kept per remote ape: two seconds at 20 Hz.
const STATES_KEPT: usize = 40;
/// Round trips kept per peer.
const RTT_KEPT: usize = 10;
/// The longest player name, in characters.
pub const MAX_NAME_CHARS: usize = 16;
/// A room code's length and letters: no 0, O, 1 or I.
pub const CODE_LEN: usize = 6;
pub const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// The transport's name for another browser (online.js's peer id).
pub type PeerId = String;

/// A room code from a seed the caller chose: six letters and digits, with
/// none of 0, O, 1 or I.
pub fn room_code(seed: u64) -> String {
    let mut dice = Dice(seed);
    (0..CODE_LEN)
        .map(|_| CODE_ALPHABET[dice.below(CODE_ALPHABET.len() as u32) as usize] as char)
        .collect()
}

/// A room code as a player typed it: case, spaces and dashes are forgiven.
/// `None` when it is not six code letters.
pub fn clean_code(text: &str) -> Option<String> {
    let code: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .map(|c| c.to_ascii_uppercase())
        .collect();
    let ok = code.len() == CODE_LEN && code.bytes().all(|b| CODE_ALPHABET.contains(&b));
    ok.then_some(code)
}

/// A player name as the lobby shows it: printable, trimmed, at most
/// [`MAX_NAME_CHARS`] characters.
pub fn clean_name(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .chars()
        .take(MAX_NAME_CHARS)
        .collect::<String>()
        .trim()
        .to_string()
}

/// A message for the transport: `to: None` is every connected peer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Outgoing {
    pub to: Option<PeerId>,
    pub text: String,
}

/// A human's seat.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Human {
    pub id: String,
    pub name: String,
    pub kind: Kind,
    /// Ready Up. The host's own seat is never marked: its Start is its Ready.
    pub ready: bool,
}

/// The AI apes' kinds: the kinds no human chose first, in the usual order,
/// then repeats. Every browser works them out the same way.
pub fn ai_kinds(humans: &[Human], count: usize) -> Vec<Kind> {
    let free: Vec<Kind> = Kind::ALL
        .into_iter()
        .filter(|k| humans.iter().all(|h| h.kind != *k))
        .collect();
    let pool = if free.is_empty() {
        Kind::ALL.to_vec()
    } else {
        free
    };
    (0..count).map(|i| pool[i % pool.len()]).collect()
}

/// Why the host refused a joiner.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "why", rename_all = "snake_case")]
pub enum Refusal {
    /// The host runs another protocol version.
    WrongVersion { host: u32, mine: u32 },
    /// Four humans are seated already.
    Full,
    /// The race has started.
    Started,
}

/// What this browser's online session is doing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Status {
    /// A joiner waiting for its seat.
    Joining,
    /// Seated in the lobby.
    Lobby,
    /// The host refused this joiner.
    Refused { reason: Refusal },
    /// In a race (or its results). After a Rematch, back to `Lobby`.
    Racing,
    /// The host closed the lobby.
    Dissolved,
    /// A joiner lost its link to the host before the race.
    Lost,
    /// This browser left.
    Left,
}

/// Why a lobby action was not taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OnlineError {
    /// Only the host may do this.
    NotHost,
    /// The host does not mark Ready; its Start is its Ready.
    HostStarts,
    /// Not seated in a lobby.
    NotInLobby,
    /// A joined human has not pressed Ready Up.
    NotReady,
    /// A race needs at least two apes.
    TooFew,
    /// Not in a race.
    NotRacing,
}

/// One ape's finish as the results know it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fin {
    Racing,
    /// Finished at this race clock (ticks since Go, on its owner's browser).
    Done(u32),
    /// Did not finish: its player left.
    Dnf,
}

/// Everything needed to build this browser's `Race`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RaceStart {
    pub seed: u64,
    pub entries: Vec<Entry>,
    /// The ape this browser's player drives.
    pub local_index: usize,
    pub equal_stats: bool,
    pub round: u32,
}

/// An item hit for this browser to apply to an ape it owns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Hit {
    pub victim: usize,
    pub by: usize,
    pub item: Item,
}

/// Where a remote ape should be drawn now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RemoteApe {
    pub index: usize,
    pub pos: V3,
    pub vel: V3,
    pub mode: Mode,
    pub yaw: i32,
}

/// One seat as the lobby screen draws it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SeatView {
    pub name: String,
    pub kind: Kind,
    pub ai: bool,
    pub host: bool,
    pub me: bool,
    pub ready: bool,
    /// The round trip to this seat's browser, when measured.
    pub ping_ms: Option<i64>,
}

/// The lobby screen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LobbyView {
    pub status: Status,
    pub is_host: bool,
    pub seats: Vec<SeatView>,
    pub ai_count: usize,
    /// The most AI apes the host may set now.
    pub max_ai: usize,
    pub equal_stats: bool,
    /// This browser's Ready mark, as last asked for.
    pub ready: bool,
    /// The host may press Start.
    pub can_start: bool,
}

/// One row of the results, in place order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ResultRow {
    pub index: usize,
    pub name: String,
    pub kind: Kind,
    /// 1-based, for finishers only.
    pub place: Option<usize>,
    pub fin: Fin,
    pub me: bool,
}

/// Every message, as small JSON.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
enum Msg {
    Hello {
        proto: u32,
        id: String,
        name: String,
        kind: Kind,
    },
    Refuse {
        reason: Refusal,
    },
    Roster {
        round: u32,
        humans: Vec<Human>,
        ai: usize,
        equal: bool,
    },
    Pick {
        kind: Kind,
    },
    Ready {
        ready: bool,
    },
    Leave,
    Dissolve,
    Ping {
        id: String,
        stamp: i64,
    },
    Pong {
        stamp: i64,
    },
    Start {
        round: u32,
        seed: u64,
        humans: Vec<Human>,
        ai: usize,
        equal: bool,
    },
    Started {
        round: u32,
    },
    /// An ape's state: round, index, the owner's race tick, position,
    /// velocity, mode, yaw.
    State {
        r: u32,
        i: usize,
        k: u32,
        p: V3,
        v: V3,
        m: Mode,
        y: i32,
    },
    Hit {
        r: u32,
        victim: usize,
        by: usize,
        item: Item,
    },
    Finish {
        r: u32,
        i: usize,
        clock: u32,
    },
    Results {
        r: u32,
        fin: Vec<Fin>,
    },
}

impl Msg {
    fn encode(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    fn decode(text: &str) -> Option<Msg> {
        serde_json::from_str(text).ok()
    }
}

/// Who owns an ape: the human with this id, or (AI) the host.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Owner {
    Human(String),
    Ai,
}

#[derive(Clone, Debug, Default)]
struct Peer {
    id: Option<String>,
    rtts: VecDeque<i64>,
}

impl Peer {
    /// The 90th-percentile round trip (nearest rank).
    fn rtt_p90(&self) -> Option<i64> {
        if self.rtts.is_empty() {
            return None;
        }
        let mut sorted: Vec<i64> = self.rtts.iter().copied().collect();
        sorted.sort_unstable();
        let rank = (sorted.len() * 9).div_ceil(10);
        Some(sorted[rank.max(1) - 1])
    }
}

/// One state received for a remote ape.
#[derive(Clone, Copy, Debug)]
struct Sample {
    /// the owner's race tick, as milliseconds
    sent: i64,
    /// when it arrived here
    got: i64,
    pos: V3,
    vel: V3,
    mode: Mode,
    yaw: i32,
}

#[derive(Clone, Debug)]
struct RaceState {
    round: u32,
    seed: u64,
    owners: Vec<Owner>,
    entries: Vec<Entry>,
    local_index: usize,
    /// Host: joiners that have not acknowledged the Start.
    unacked: BTreeSet<PeerId>,
    last_start_ms: i64,
    last_sent: BTreeMap<usize, i64>,
    samples: BTreeMap<usize, VecDeque<Sample>>,
    hits: Vec<Hit>,
    /// The results: the host's own on the host, the host's latest on a joiner.
    fin: Vec<Fin>,
    /// Finishes reported by any browser (including this one).
    heard: BTreeMap<usize, u32>,
    /// This browser's apes' finishes, sent until the host's results show them.
    reported: BTreeMap<usize, u32>,
    last_finish_ms: i64,
    last_results_ms: i64,
    /// Apes whose owner has left.
    gone: BTreeSet<usize>,
    host_gone: bool,
    /// Remote finishes already given to the local `Race`.
    applied: BTreeSet<usize>,
}

/// One browser's online session.
#[derive(Clone, Debug)]
pub struct Online {
    proto: u32,
    client_id: String,
    name: String,
    kind: Kind,
    is_host: bool,
    host_peer: Option<PeerId>,
    status: Status,
    round: u32,
    humans: Vec<Human>,
    ai_count: usize,
    equal_stats: bool,
    peers: BTreeMap<PeerId, Peer>,
    outbox: Vec<Outgoing>,
    last_hello_ms: i64,
    last_roster_ms: i64,
    last_ping_ms: i64,
    wanted_kind: Option<Kind>,
    last_pick_ms: i64,
    wanted_ready: Option<bool>,
    last_ready_ms: i64,
    race: Option<RaceState>,
    /// Host: its own start, held until the joiners have theirs.
    host_start_at: Option<i64>,
    start: Option<RaceStart>,
}

impl Online {
    fn blank(client_id: &str, name: &str, kind: Kind, now_ms: i64) -> Online {
        Online {
            proto: PROTOCOL,
            client_id: client_id.to_string(),
            name: clean_name(name),
            kind,
            is_host: false,
            host_peer: None,
            status: Status::Joining,
            round: 0,
            humans: Vec::new(),
            ai_count: 0,
            equal_stats: false,
            peers: BTreeMap::new(),
            outbox: Vec::new(),
            last_hello_ms: now_ms,
            last_roster_ms: now_ms,
            last_ping_ms: now_ms - PING_EVERY_MS,
            wanted_kind: None,
            last_pick_ms: now_ms,
            wanted_ready: None,
            last_ready_ms: now_ms,
            race: None,
            host_start_at: None,
            start: None,
        }
    }

    /// Opens a lobby as its host, in seat 0.
    pub fn host(client_id: &str, name: &str, kind: Kind, now_ms: i64) -> Online {
        let mut o = Online::blank(client_id, name, kind, now_ms);
        o.is_host = true;
        o.status = Status::Lobby;
        o.humans = vec![Human {
            id: client_id.to_string(),
            name: o.name.clone(),
            kind,
            ready: false,
        }];
        o
    }

    /// Joins a lobby; the Hello goes when the transport connects the host.
    pub fn join(client_id: &str, name: &str, kind: Kind, now_ms: i64) -> Online {
        Online::blank(client_id, name, kind, now_ms)
    }

    /// For tests: speaks another protocol version.
    #[doc(hidden)]
    pub fn with_protocol(mut self, proto: u32) -> Online {
        self.proto = proto;
        self
    }

    fn send(&mut self, to: &PeerId, msg: &Msg) {
        self.outbox.push(Outgoing {
            to: Some(to.clone()),
            text: msg.encode(),
        });
    }

    fn broadcast(&mut self, msg: &Msg) {
        self.outbox.push(Outgoing {
            to: None,
            text: msg.encode(),
        });
    }

    /// Everything to send since the last call.
    pub fn take_outbox(&mut self) -> Vec<Outgoing> {
        std::mem::take(&mut self.outbox)
    }

    pub fn is_host(&self) -> bool {
        self.is_host
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// The round: 0, then one more for each Rematch.
    pub fn round(&self) -> u32 {
        self.round
    }

    fn in_lobby(&self) -> bool {
        self.status == Status::Lobby
    }

    fn hello(&self) -> Msg {
        Msg::Hello {
            proto: self.proto,
            id: self.client_id.clone(),
            name: self.name.clone(),
            kind: self.kind,
        }
    }

    fn roster(&self) -> Msg {
        Msg::Roster {
            round: self.round,
            humans: self.humans.clone(),
            ai: self.ai_count,
            equal: self.equal_stats,
        }
    }

    fn roster_changed(&mut self, now_ms: i64) {
        self.ai_count = self.ai_count.min(self.max_ai());
        self.last_roster_ms = now_ms;
        let roster = self.roster();
        self.broadcast(&roster);
    }

    fn max_ai(&self) -> usize {
        MAX_AI.min(MAX_APES.saturating_sub(self.humans.len()))
    }

    fn clear_ready(&mut self) {
        for h in &mut self.humans {
            h.ready = false;
        }
    }

    fn peer_of_client(&self, id: &str) -> Option<PeerId> {
        self.peers
            .iter()
            .find(|(_, p)| p.id.as_deref() == Some(id))
            .map(|(k, _)| k.clone())
    }

    // ---- transport events ---------------------------------------------------

    /// The transport connected a peer. `host` is what that peer said of
    /// itself (online.js's `connected` event).
    pub fn peer_connected(&mut self, peer: &str, host: bool, now_ms: i64) {
        self.peers.entry(peer.to_string()).or_default();
        if !self.is_host && host && self.host_peer.is_none() {
            self.host_peer = Some(peer.to_string());
            if self.status == Status::Joining {
                self.last_hello_ms = now_ms;
                let hello = self.hello();
                self.send(&peer.to_string(), &hello);
            }
        }
    }

    /// The transport lost a peer.
    pub fn peer_left(&mut self, peer: &str, now_ms: i64) {
        let info = self.peers.remove(peer);
        let client = info.and_then(|p| p.id);
        if self.host_peer.as_deref() == Some(peer) {
            match self.status {
                Status::Joining | Status::Lobby => self.status = Status::Lost,
                Status::Racing => {
                    if let Some(race) = &mut self.race {
                        race.host_gone = true;
                        for (i, o) in race.owners.iter().enumerate() {
                            let host_owned = match o {
                                Owner::Ai => true,
                                Owner::Human(id) => Some(id) == client.as_ref() || i == 0,
                            };
                            if host_owned {
                                race.gone.insert(i);
                            }
                        }
                    }
                }
                _ => {}
            }
            return;
        }
        let Some(client) = client else { return };
        self.client_left(&client, now_ms);
        if let Some(race) = &mut self.race {
            race.unacked.remove(peer);
        }
    }

    fn client_left(&mut self, client: &str, now_ms: i64) {
        if self.status == Status::Racing {
            let Some(race) = &mut self.race else { return };
            let Some(i) = race
                .owners
                .iter()
                .position(|o| *o == Owner::Human(client.to_string()))
            else {
                return;
            };
            race.gone.insert(i);
            if self.is_host && race.fin[i] == Fin::Racing {
                race.fin[i] = Fin::Dnf;
                self.send_results(now_ms);
            }
            return;
        }
        if self.is_host && self.in_lobby() {
            let before = self.humans.len();
            self.humans
                .retain(|h| h.id != client || h.id == self.client_id);
            if self.humans.len() != before {
                self.roster_changed(now_ms);
            }
        }
    }

    /// Handles one message from a peer. Unknown text is ignored.
    pub fn receive(&mut self, peer: &str, text: &str, now_ms: i64) {
        let Some(msg) = Msg::decode(text) else {
            return;
        };
        let peer = peer.to_string();
        self.peers.entry(peer.clone()).or_default();
        let from_host = self.host_peer.as_ref() == Some(&peer);
        match msg {
            Msg::Ping { id, stamp } => {
                if let Some(p) = self.peers.get_mut(&peer) {
                    p.id = Some(id);
                }
                self.send(&peer, &Msg::Pong { stamp });
            }
            Msg::Pong { stamp } => {
                if let Some(p) = self.peers.get_mut(&peer) {
                    p.rtts.push_back((now_ms - stamp).max(0));
                    while p.rtts.len() > RTT_KEPT {
                        p.rtts.pop_front();
                    }
                }
            }
            Msg::Hello {
                proto,
                id,
                name,
                kind,
            } if self.is_host => self.on_hello(&peer, proto, id, &name, kind, now_ms),
            Msg::Pick { kind } if self.is_host => self.on_pick(&peer, kind, now_ms),
            Msg::Ready { ready } if self.is_host => self.on_ready(&peer, ready, now_ms),
            Msg::Leave => {
                let client = self.peers.get(&peer).and_then(|p| p.id.clone());
                if let Some(client) = client {
                    self.client_left(&client, now_ms);
                }
            }
            Msg::Started { round } if self.is_host => {
                if let Some(race) = &mut self.race {
                    if race.round == round {
                        race.unacked.remove(&peer);
                    }
                }
            }
            Msg::Refuse { reason } if from_host => {
                if self.status == Status::Joining {
                    self.status = Status::Refused { reason };
                }
            }
            Msg::Roster {
                round,
                humans,
                ai,
                equal,
            } if from_host => self.on_roster(round, humans, ai, equal),
            Msg::Dissolve if from_host => {
                if matches!(self.status, Status::Joining | Status::Lobby) {
                    self.status = Status::Dissolved;
                }
            }
            Msg::Start {
                round,
                seed,
                humans,
                ai,
                equal,
            } if from_host => {
                let waiting = matches!(self.status, Status::Lobby | Status::Joining);
                if waiting && humans.iter().any(|h| h.id == self.client_id) && round >= self.round {
                    self.round = round;
                    self.humans = humans;
                    self.ai_count = ai;
                    self.equal_stats = equal;
                    self.begin_race(seed, now_ms);
                }
                if self.race.as_ref().is_some_and(|r| r.round == round) {
                    self.send(&peer, &Msg::Started { round });
                }
            }
            Msg::State {
                r,
                i,
                k,
                p,
                v,
                m,
                y,
            } => self.on_state(r, i, k, p, v, m, y, now_ms),
            Msg::Hit {
                r,
                victim,
                by,
                item,
            } => {
                let mine = self.owns(victim);
                if let Some(race) = &mut self.race {
                    if race.round == r && mine {
                        race.hits.push(Hit { victim, by, item });
                    }
                }
            }
            Msg::Finish { r, i, clock } => self.on_finish(r, i, clock, now_ms),
            Msg::Results { r, fin } if from_host => {
                if let Some(race) = &mut self.race {
                    if race.round == r && fin.len() == race.fin.len() {
                        race.fin = fin;
                        let shown: Vec<usize> = race
                            .reported
                            .keys()
                            .copied()
                            .filter(|i| matches!(race.fin[*i], Fin::Done(_)))
                            .collect();
                        for i in shown {
                            race.reported.remove(&i);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn on_hello(
        &mut self,
        peer: &PeerId,
        proto: u32,
        id: String,
        name: &str,
        kind: Kind,
        now: i64,
    ) {
        if proto != PROTOCOL {
            let reason = Refusal::WrongVersion {
                host: PROTOCOL,
                mine: proto,
            };
            self.send(peer, &Msg::Refuse { reason });
            return;
        }
        if let Some(p) = self.peers.get_mut(peer) {
            p.id = Some(id.clone());
        }
        if self.status == Status::Racing {
            let racing = self
                .race
                .as_ref()
                .is_some_and(|r| r.owners.contains(&Owner::Human(id.clone())));
            if !racing {
                self.send(
                    peer,
                    &Msg::Refuse {
                        reason: Refusal::Started,
                    },
                );
            }
            return;
        }
        if !self.in_lobby() {
            return;
        }
        if self.humans.iter().any(|h| h.id == id) {
            let roster = self.roster();
            self.send(peer, &roster);
            return;
        }
        if self.humans.len() >= MAX_APES {
            self.send(
                peer,
                &Msg::Refuse {
                    reason: Refusal::Full,
                },
            );
            return;
        }
        self.humans.push(Human {
            id,
            name: clean_name(name),
            kind,
            ready: false,
        });
        self.roster_changed(now);
    }

    fn seat_of_peer(&self, peer: &PeerId) -> Option<usize> {
        let id = self.peers.get(peer)?.id.as_deref()?;
        self.humans.iter().position(|h| h.id == id)
    }

    fn on_pick(&mut self, peer: &PeerId, kind: Kind, now_ms: i64) {
        if !self.in_lobby() {
            return;
        }
        match self.seat_of_peer(peer) {
            Some(i) if i != 0 && self.humans[i].kind != kind => {
                self.humans[i].kind = kind;
                self.roster_changed(now_ms);
            }
            _ => {}
        }
    }

    fn on_ready(&mut self, peer: &PeerId, ready: bool, now_ms: i64) {
        if !self.in_lobby() {
            return;
        }
        match self.seat_of_peer(peer) {
            Some(i) if i != 0 && self.humans[i].ready != ready => {
                self.humans[i].ready = ready;
                self.roster_changed(now_ms);
            }
            _ => {}
        }
    }

    fn on_roster(&mut self, round: u32, humans: Vec<Human>, ai: usize, equal: bool) {
        if round < self.round {
            return;
        }
        match self.status {
            Status::Joining | Status::Lobby => {}
            // A rematch: back to the lobby.
            Status::Racing if round > self.round => {
                self.race = None;
                self.start = None;
                self.wanted_ready = None;
                self.wanted_kind = None;
            }
            _ => return,
        }
        self.round = round;
        self.humans = humans;
        self.ai_count = ai;
        self.equal_stats = equal;
        let seated = self.humans.iter().any(|h| h.id == self.client_id);
        self.status = if seated {
            Status::Lobby
        } else {
            Status::Joining
        };
    }

    fn on_state(
        &mut self,
        r: u32,
        i: usize,
        k: u32,
        pos: V3,
        vel: V3,
        mode: Mode,
        yaw: i32,
        now_ms: i64,
    ) {
        if self.owns(i) {
            return;
        }
        let Some(race) = &mut self.race else { return };
        if race.round != r || i >= race.owners.len() {
            return;
        }
        let sent = k as i64 * 1000 / TICKS_PER_SECOND as i64;
        let list = race.samples.entry(i).or_default();
        if list.back().is_some_and(|s| s.sent >= sent) {
            return;
        }
        list.push_back(Sample {
            sent,
            got: now_ms,
            pos,
            vel,
            mode,
            yaw,
        });
        while list.len() > STATES_KEPT {
            list.pop_front();
        }
    }

    fn on_finish(&mut self, r: u32, i: usize, clock: u32, now_ms: i64) {
        let is_host = self.is_host;
        let Some(race) = &mut self.race else { return };
        if race.round != r || i >= race.fin.len() {
            return;
        }
        race.heard.entry(i).or_insert(clock);
        if is_host && race.fin[i] == Fin::Racing {
            race.fin[i] = Fin::Done(clock);
            self.send_results(now_ms);
        } else if is_host {
            // a repeat: the joiner has not seen the results yet
            self.send_results(now_ms);
        }
    }

    // ---- the lobby ----------------------------------------------------------

    /// Chooses this browser's ape. A joiner's choice is repeated until the
    /// roster shows it.
    pub fn set_kind(&mut self, kind: Kind, now_ms: i64) -> Result<(), OnlineError> {
        if !matches!(self.status, Status::Lobby | Status::Joining) {
            return Err(OnlineError::NotInLobby);
        }
        self.kind = kind;
        if self.is_host {
            if self.humans[0].kind != kind {
                self.humans[0].kind = kind;
                self.roster_changed(now_ms);
            }
            return Ok(());
        }
        self.wanted_kind = Some(kind);
        self.last_pick_ms = now_ms;
        if let Some(host) = self.host_peer.clone() {
            if self.status == Status::Lobby {
                self.send(&host, &Msg::Pick { kind });
            }
        }
        Ok(())
    }

    /// Joiner: presses Ready Up (or undoes it). Repeated until the roster
    /// shows it.
    pub fn set_ready(&mut self, ready: bool, now_ms: i64) -> Result<(), OnlineError> {
        if self.is_host {
            return Err(OnlineError::HostStarts);
        }
        if self.status != Status::Lobby {
            return Err(OnlineError::NotInLobby);
        }
        self.wanted_ready = Some(ready);
        self.last_ready_ms = now_ms;
        if let Some(host) = self.host_peer.clone() {
            self.send(&host, &Msg::Ready { ready });
        }
        Ok(())
    }

    fn host_lobby(&self) -> Result<(), OnlineError> {
        if !self.is_host {
            return Err(OnlineError::NotHost);
        }
        if !self.in_lobby() {
            return Err(OnlineError::NotInLobby);
        }
        Ok(())
    }

    /// Host: the number of AI apes, clamped to 0–3 and to four apes in all.
    /// Clears every Ready mark.
    pub fn set_ai_count(&mut self, count: usize, now_ms: i64) -> Result<(), OnlineError> {
        self.host_lobby()?;
        self.ai_count = count.min(self.max_ai());
        self.clear_ready();
        self.roster_changed(now_ms);
        Ok(())
    }

    /// Host: Equal Stats on or off. Clears every Ready mark.
    pub fn set_equal_stats(&mut self, equal: bool, now_ms: i64) -> Result<(), OnlineError> {
        self.host_lobby()?;
        self.equal_stats = equal;
        self.clear_ready();
        self.roster_changed(now_ms);
        Ok(())
    }

    /// True when every joined human but the host has pressed Ready Up.
    pub fn all_ready(&self) -> bool {
        self.humans.iter().skip(1).all(|h| h.ready)
    }

    fn apes(&self) -> usize {
        self.humans.len() + self.ai_count
    }

    /// Host: starts the race with a seed the caller chose. Each joiner gets
    /// a Start (repeated until acknowledged); the host's own start follows
    /// once the Start has had time to arrive (see [`MAX_START_LEAD_MS`]).
    pub fn start(&mut self, seed: u64, now_ms: i64) -> Result<(), OnlineError> {
        self.host_lobby()?;
        if !self.all_ready() {
            return Err(OnlineError::NotReady);
        }
        if self.apes() < 2 {
            return Err(OnlineError::TooFew);
        }
        self.begin_race(seed, now_ms);
        let lead = self
            .peers
            .values()
            .filter_map(|p| p.rtt_p90())
            .max()
            .unwrap_or(0)
            / 2;
        self.host_start_at = Some(now_ms + lead.clamp(0, MAX_START_LEAD_MS));
        self.resend_start(now_ms);
        Ok(())
    }

    fn begin_race(&mut self, seed: u64, now_ms: i64) {
        let mut owners: Vec<Owner> = self
            .humans
            .iter()
            .map(|h| Owner::Human(h.id.clone()))
            .collect();
        let mut entries: Vec<Entry> = self
            .humans
            .iter()
            .map(|h| Entry {
                name: h.name.clone(),
                kind: h.kind,
                driver: if h.id == self.client_id {
                    Driver::Local
                } else {
                    Driver::Remote
                },
            })
            .collect();
        for (n, kind) in ai_kinds(&self.humans, self.ai_count)
            .into_iter()
            .enumerate()
        {
            owners.push(Owner::Ai);
            entries.push(Entry {
                name: format!("AI {}", n + 1),
                kind,
                driver: if self.is_host {
                    Driver::Ai
                } else {
                    Driver::Remote
                },
            });
        }
        let local_index = self
            .humans
            .iter()
            .position(|h| h.id == self.client_id)
            .unwrap_or(0);
        let unacked: BTreeSet<PeerId> = if self.is_host {
            self.humans
                .iter()
                .skip(1)
                .filter_map(|h| self.peer_of_client(&h.id))
                .collect()
        } else {
            BTreeSet::new()
        };
        let count = entries.len();
        let start = RaceStart {
            seed,
            entries: entries.clone(),
            local_index,
            equal_stats: self.equal_stats,
            round: self.round,
        };
        self.race = Some(RaceState {
            round: self.round,
            seed,
            owners,
            entries,
            local_index,
            unacked,
            last_start_ms: now_ms - RETRY_MS,
            last_sent: BTreeMap::new(),
            samples: BTreeMap::new(),
            hits: Vec::new(),
            fin: vec![Fin::Racing; count],
            heard: BTreeMap::new(),
            reported: BTreeMap::new(),
            last_finish_ms: now_ms,
            last_results_ms: now_ms,
            gone: BTreeSet::new(),
            host_gone: false,
            applied: BTreeSet::new(),
        });
        self.start = Some(start);
        self.status = Status::Racing;
        self.wanted_ready = None;
        self.wanted_kind = None;
    }

    fn resend_start(&mut self, now_ms: i64) {
        let Some(race) = &mut self.race else { return };
        race.last_start_ms = now_ms;
        let msg = Msg::Start {
            round: race.round,
            seed: race.seed,
            humans: self.humans.clone(),
            ai: self.ai_count,
            equal: self.equal_stats,
        };
        let to: Vec<PeerId> = race.unacked.iter().cloned().collect();
        for peer in to {
            self.send(&peer, &msg);
        }
    }

    /// Once the race starts: what this browser needs to build its `Race`.
    /// Given once per race. (On the host it waits for the Start's lead.)
    pub fn take_race_start(&mut self) -> Option<RaceStart> {
        if self.host_start_at.is_some() {
            return None;
        }
        self.start.take()
    }

    /// Leaves. On the host this dissolves the lobby (or, mid-race, ends the
    /// session for everyone). The page then calls online.js `leave()`.
    pub fn leave(&mut self) {
        if matches!(self.status, Status::Left) {
            return;
        }
        if self.is_host {
            self.broadcast(&Msg::Dissolve);
        } else if let Some(host) = self.host_peer.clone() {
            self.send(&host, &Msg::Leave);
        }
        self.status = Status::Left;
    }

    /// Host, during or after a race: back to the lobby with the same seats
    /// (players who left are gone) and settings, every Ready mark cleared.
    pub fn rematch(&mut self, now_ms: i64) -> Result<(), OnlineError> {
        if !self.is_host {
            return Err(OnlineError::NotHost);
        }
        if self.status != Status::Racing {
            return Err(OnlineError::NotRacing);
        }
        let connected: BTreeSet<String> =
            self.peers.values().filter_map(|p| p.id.clone()).collect();
        let me = self.client_id.clone();
        self.humans
            .retain(|h| h.id == me || connected.contains(&h.id));
        self.clear_ready();
        self.round += 1;
        self.race = None;
        self.start = None;
        self.host_start_at = None;
        self.status = Status::Lobby;
        self.roster_changed(now_ms);
        Ok(())
    }

    /// Repeats what may have been lost, and pings. Call every frame.
    pub fn update(&mut self, now_ms: i64) {
        if matches!(
            self.status,
            Status::Left | Status::Dissolved | Status::Lost | Status::Refused { .. }
        ) {
            return;
        }
        if let Some(at) = self.host_start_at {
            if now_ms >= at {
                self.host_start_at = None;
            }
        }
        if self.is_host {
            if self.in_lobby() && now_ms - self.last_roster_ms >= ROSTER_EVERY_MS {
                self.last_roster_ms = now_ms;
                let roster = self.roster();
                self.broadcast(&roster);
            }
            let due = self.race.as_ref().map(|r| {
                (
                    !r.unacked.is_empty() && now_ms - r.last_start_ms >= RETRY_MS,
                    now_ms - r.last_results_ms >= ROSTER_EVERY_MS,
                )
            });
            if let Some((start, results)) = due {
                if start {
                    self.resend_start(now_ms);
                }
                if results {
                    self.send_results(now_ms);
                }
            }
        } else if let Some(host) = self.host_peer.clone() {
            self.joiner_retries(&host, now_ms);
        }
        if now_ms - self.last_ping_ms >= PING_EVERY_MS {
            self.last_ping_ms = now_ms;
            let ping = Msg::Ping {
                id: self.client_id.clone(),
                stamp: now_ms,
            };
            self.broadcast(&ping);
        }
    }

    fn joiner_retries(&mut self, host: &PeerId, now_ms: i64) {
        if self.status == Status::Joining && now_ms - self.last_hello_ms >= RETRY_MS {
            self.last_hello_ms = now_ms;
            let hello = self.hello();
            self.send(host, &hello);
        }
        if self.status == Status::Lobby {
            let mine = self.humans.iter().find(|h| h.id == self.client_id).cloned();
            if let (Some(kind), Some(me)) = (self.wanted_kind, &mine) {
                if me.kind == kind {
                    self.wanted_kind = None;
                } else if now_ms - self.last_pick_ms >= RETRY_MS {
                    self.last_pick_ms = now_ms;
                    self.send(host, &Msg::Pick { kind });
                }
            }
            if let (Some(ready), Some(me)) = (self.wanted_ready, &mine) {
                if me.ready == ready {
                    self.wanted_ready = None;
                } else if now_ms - self.last_ready_ms >= RETRY_MS {
                    self.last_ready_ms = now_ms;
                    self.send(host, &Msg::Ready { ready });
                }
            }
        }
        let resend: Vec<Msg> = match &mut self.race {
            Some(race)
                if !race.host_gone
                    && !race.reported.is_empty()
                    && now_ms - race.last_finish_ms >= RETRY_MS =>
            {
                race.last_finish_ms = now_ms;
                race.reported
                    .iter()
                    .map(|(i, clock)| Msg::Finish {
                        r: race.round,
                        i: *i,
                        clock: *clock,
                    })
                    .collect()
            }
            _ => Vec::new(),
        };
        for msg in resend {
            self.broadcast(&msg);
        }
    }

    // ---- the lobby view -----------------------------------------------------

    fn ping_of(&self, id: &str) -> Option<i64> {
        let peer = self.peer_of_client(id)?;
        self.peers.get(&peer)?.rtt_p90()
    }

    /// The lobby as the screen draws it.
    pub fn lobby(&self) -> LobbyView {
        let mut seats: Vec<SeatView> = self
            .humans
            .iter()
            .enumerate()
            .map(|(i, h)| SeatView {
                name: h.name.clone(),
                kind: h.kind,
                ai: false,
                host: i == 0,
                me: h.id == self.client_id,
                ready: h.ready,
                ping_ms: if h.id == self.client_id {
                    None
                } else {
                    self.ping_of(&h.id)
                },
            })
            .collect();
        for (n, kind) in ai_kinds(&self.humans, self.ai_count)
            .into_iter()
            .enumerate()
        {
            seats.push(SeatView {
                name: format!("AI {}", n + 1),
                kind,
                ai: true,
                host: false,
                me: false,
                ready: true,
                ping_ms: None,
            });
        }
        let ready = self.wanted_ready.unwrap_or_else(|| {
            self.humans
                .iter()
                .any(|h| h.id == self.client_id && h.ready)
        });
        LobbyView {
            status: self.status.clone(),
            is_host: self.is_host,
            seats,
            ai_count: self.ai_count,
            max_ai: self.max_ai(),
            equal_stats: self.equal_stats,
            ready,
            can_start: self.is_host && self.in_lobby() && self.all_ready() && self.apes() >= 2,
        }
    }

    // ---- the race -----------------------------------------------------------

    /// True when this browser simulates ape `index`: its own, and on the
    /// host the AI apes.
    pub fn owns(&self, index: usize) -> bool {
        let Some(race) = &self.race else {
            return false;
        };
        match race.owners.get(index) {
            Some(Owner::Human(id)) => *id == self.client_id,
            Some(Owner::Ai) => self.is_host,
            None => false,
        }
    }

    /// The ape this browser's player drives, during a race.
    pub fn local_index(&self) -> Option<usize> {
        self.race.as_ref().map(|r| r.local_index)
    }

    /// The race's entries, during a race.
    pub fn entries(&self) -> &[Entry] {
        self.race
            .as_ref()
            .map(|r| r.entries.as_slice())
            .unwrap_or(&[])
    }

    /// True on a joiner whose host has left mid-race: the results are then
    /// what this browser heard, and the host's and AI apes are DNF.
    pub fn host_gone(&self) -> bool {
        self.race.as_ref().is_some_and(|r| r.host_gone)
    }

    /// Sends the state of ape `index` (one this browser owns), at most every
    /// [`STATE_EVERY_MS`]. `tick` is the race's `tick` (ticks since setup).
    pub fn send_state(&mut self, now_ms: i64, tick: u32, index: usize, ape: &Ape) {
        if !self.owns(index) {
            return;
        }
        let Some(race) = &mut self.race else { return };
        if race
            .last_sent
            .get(&index)
            .is_some_and(|t| now_ms - t < STATE_EVERY_MS)
        {
            return;
        }
        race.last_sent.insert(index, now_ms);
        let msg = Msg::State {
            r: race.round,
            i: index,
            k: tick,
            p: ape.pos,
            v: ape.vel,
            m: ape.mode,
            y: ape.yaw,
        };
        self.broadcast(&msg);
    }

    /// Where each remote ape should be drawn now: about
    /// [`INTERP_DELAY_MS`] behind its owner, interpolated between states, or
    /// carried along its velocity for at most [`EXTRAPOLATE_MS`] when states
    /// are late. An ape whose owner left stands still where it was last seen.
    pub fn remote_states(&self, now_ms: i64) -> Vec<RemoteApe> {
        let Some(race) = &self.race else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (index, list) in &race.samples {
            if self.owns(*index) || list.is_empty() {
                continue;
            }
            let newest = list[list.len() - 1];
            if race.gone.contains(index) {
                out.push(RemoteApe {
                    index: *index,
                    pos: newest.pos,
                    vel: V3::ZERO,
                    mode: newest.mode,
                    yaw: newest.yaw,
                });
                continue;
            }
            // The quickest delivery seen maps the owner's clock onto ours.
            let offset = list.iter().map(|s| s.got - s.sent).min().unwrap_or(0);
            let target = now_ms - offset - INTERP_DELAY_MS;
            out.push(sample_at(*index, list, target));
        }
        out
    }

    /// Sends an item hit on ape `victim` to the browser that owns it (the
    /// host for an AI ape). A hit on an ape this browser owns is kept for
    /// [`Online::take_hits`].
    pub fn send_hit(&mut self, victim: usize, by: usize, item: Item) {
        let Some(race) = &mut self.race else { return };
        let Some(owner) = race.owners.get(victim).cloned() else {
            return;
        };
        let mine = match &owner {
            Owner::Human(id) => *id == self.client_id,
            Owner::Ai => self.is_host,
        };
        if mine {
            race.hits.push(Hit { victim, by, item });
            return;
        }
        let msg = Msg::Hit {
            r: race.round,
            victim,
            by,
            item,
        };
        let to = match owner {
            Owner::Human(id) => self.peer_of_client(&id),
            Owner::Ai => self.host_peer.clone(),
        };
        match to {
            Some(peer) => self.send(&peer, &msg),
            // Not yet known by id: everyone gets it, and only the owner keeps it.
            None => self.broadcast(&msg),
        }
    }

    /// Hits for this browser to apply to its own apes, since the last call.
    pub fn take_hits(&mut self) -> Vec<Hit> {
        self.race
            .as_mut()
            .map(|r| std::mem::take(&mut r.hits))
            .unwrap_or_default()
    }

    /// Reports that ape `index` (one this browser owns) finished at race
    /// clock `clock`. The host records it and publishes the results; a
    /// joiner repeats it until the host's results show it.
    pub fn report_finish(&mut self, index: usize, clock: u32, now_ms: i64) {
        if !self.owns(index) {
            return;
        }
        let is_host = self.is_host;
        let Some(race) = &mut self.race else { return };
        if race.heard.contains_key(&index) {
            return;
        }
        race.heard.insert(index, clock);
        let msg = Msg::Finish {
            r: race.round,
            i: index,
            clock,
        };
        if is_host {
            if race.fin[index] == Fin::Racing {
                race.fin[index] = Fin::Done(clock);
            }
            self.send_results(now_ms);
        } else {
            race.reported.insert(index, clock);
            race.last_finish_ms = now_ms;
            self.broadcast(&msg);
        }
    }

    fn send_results(&mut self, now_ms: i64) {
        let Some(race) = &mut self.race else { return };
        race.last_results_ms = now_ms;
        let msg = Msg::Results {
            r: race.round,
            fin: race.fin.clone(),
        };
        self.broadcast(&msg);
    }

    /// One call per frame during a race, after stepping it: sends this
    /// browser's apes' states and finishes, moves the remote apes and
    /// records remote finishes. Hits are separate: see the module docs.
    pub fn sync_race(&mut self, now_ms: i64, race: &mut Race) {
        if self.race.is_none() {
            return;
        }
        for i in 0..race.racers.len() {
            if self.owns(i) {
                self.send_state(now_ms, race.tick, i, &race.racers[i].ape);
                if let Some(clock) = race.racers[i].finish {
                    self.report_finish(i, clock, now_ms);
                }
            }
        }
        for a in self.remote_states(now_ms) {
            race.set_remote(a.index, a.pos, a.vel, a.mode, a.yaw);
        }
        let fin = self.final_fin();
        let Some(state) = &mut self.race else { return };
        for (i, f) in fin.iter().enumerate() {
            if let Fin::Done(clock) = f {
                let remote = race
                    .racers
                    .get(i)
                    .is_some_and(|r| r.driver == Driver::Remote);
                if remote && state.applied.insert(i) && race.racers[i].finish.is_none() {
                    race.remote_finish(i, *clock);
                }
            }
        }
    }

    /// The results as this browser knows them: the host's, plus finishes
    /// heard and not yet in them; an ape whose owner left is DNF.
    fn final_fin(&self) -> Vec<Fin> {
        let Some(race) = &self.race else {
            return Vec::new();
        };
        let mut fin = race.fin.clone();
        for (i, f) in fin.iter_mut().enumerate() {
            if *f == Fin::Racing {
                if let Some(clock) = race.heard.get(&i) {
                    *f = Fin::Done(*clock);
                } else if race.gone.contains(&i) {
                    *f = Fin::Dnf;
                }
            }
        }
        fin
    }

    /// The results in place order: finishers by clock (ties by index), then
    /// apes still racing, then DNFs.
    pub fn results(&self) -> Vec<ResultRow> {
        let Some(race) = &self.race else {
            return Vec::new();
        };
        let fin = self.final_fin();
        let rank = |i: &usize| match fin[*i] {
            Fin::Done(c) => (0, c),
            Fin::Racing => (1, 0),
            Fin::Dnf => (2, 0),
        };
        let mut order: Vec<usize> = (0..fin.len()).collect();
        order.sort_by_key(|i| (rank(i), *i));
        let mut place = 0;
        order
            .into_iter()
            .map(|i| {
                let done = matches!(fin[i], Fin::Done(_));
                if done {
                    place += 1;
                }
                ResultRow {
                    index: i,
                    name: race.entries[i].name.clone(),
                    kind: race.entries[i].kind,
                    place: done.then_some(place),
                    fin: fin[i],
                    me: i == race.local_index,
                }
            })
            .collect()
    }

    /// True when no ape is still racing.
    pub fn results_final(&self) -> bool {
        self.race.is_some() && self.final_fin().iter().all(|f| *f != Fin::Racing)
    }
}

/// A remote ape at the owner's moment `target` (milliseconds of its race).
fn sample_at(index: usize, list: &VecDeque<Sample>, target: i64) -> RemoteApe {
    let at = |s: &Sample, pos: V3| RemoteApe {
        index,
        pos,
        vel: s.vel,
        mode: s.mode,
        yaw: s.yaw,
    };
    let first = list[0];
    if target <= first.sent {
        return at(&first, first.pos);
    }
    for k in 1..list.len() {
        let (a, b) = (list[k - 1], list[k]);
        if target <= b.sent {
            let span = (b.sent - a.sent).max(1);
            let t = ((target - a.sent) * ONE as i64 / span) as i32;
            return RemoteApe {
                index,
                pos: a.pos.lerp(b.pos, t),
                vel: a.vel.lerp(b.vel, t),
                mode: if t < ONE / 2 { a.mode } else { b.mode },
                yaw: a.yaw + fx::mul(fx::wrap(b.yaw - a.yaw), t),
            };
        }
    }
    let last = list[list.len() - 1];
    let ahead = (target - last.sent).min(EXTRAPOLATE_MS);
    let step = |v: i32| (v as i64 * ahead / 1000) as i32;
    let moved = V3 {
        x: step(last.vel.x),
        y: step(last.vel.y),
        z: step(last.vel.z),
    };
    at(&last, last.pos.add(moved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::int;
    use crate::geom::v3;

    #[test]
    fn room_codes_are_six_clear_letters_from_the_seed() {
        let a = room_code(1);
        assert_eq!(a.len(), CODE_LEN);
        assert!(a.bytes().all(|b| CODE_ALPHABET.contains(&b)));
        for bad in *b"0O1I" {
            assert!(!CODE_ALPHABET.contains(&bad));
        }
        assert_eq!(room_code(1), a);
        assert_ne!(room_code(2), a);
        assert_eq!(clean_code(&format!(" {} ", a.to_lowercase())), Some(a));
        assert_eq!(clean_code("ABC-DEF"), Some("ABCDEF".to_string()));
        assert_eq!(clean_code("ABCDE0"), None);
        assert_eq!(clean_code("ABCDE"), None);
    }

    #[test]
    fn ai_apes_take_kinds_no_human_chose() {
        let h = |kind| Human {
            id: String::new(),
            name: String::new(),
            kind,
            ready: false,
        };
        let humans = [h(Kind::Orangutan), h(Kind::Gibbon)];
        assert_eq!(ai_kinds(&humans, 2), vec![Kind::Gorilla, Kind::Chimpanzee]);
    }

    fn sample(sent: i64, x: i32) -> Sample {
        Sample {
            sent,
            got: sent,
            pos: v3(x, 0, 0),
            vel: v3(int(10), 0, 0),
            mode: Mode::Flying,
            yaw: 0,
        }
    }

    #[test]
    fn interpolation_blends_and_extrapolation_stops() {
        let list: VecDeque<Sample> = [sample(0, 0), sample(100, int(1))].into_iter().collect();
        assert_eq!(sample_at(0, &list, 50).pos.x, int(1) / 2);
        assert_eq!(sample_at(0, &list, -20).pos.x, 0);
        // 100 ms past the newest at 10 m/s: one more metre
        assert_eq!(sample_at(0, &list, 200).pos.x, int(2));
        // but never more than EXTRAPOLATE_MS ahead
        let far = sample_at(0, &list, 5000).pos.x;
        assert_eq!(far, int(1) + int(10) * EXTRAPOLATE_MS as i32 / 1000);
    }
}
