//! Online play over a simulated network (PLAN P13): 2–4 sessions of
//! `tool_core::online::Online`, each with its own `Race`, joined by a
//! loopback with latency, jitter and lost lobby messages (adapted from
//! Mandible's `lockstep/loopback.rs`).

use std::collections::{BTreeMap, BTreeSet};

use tool_core::ape::Controls;
use tool_core::course::Course;
use tool_core::map::Map;
use tool_core::online::{Fin, Hit, Online, OnlineError, Outgoing, Refusal, Status, MAX_APES};
use tool_core::race::{Driver, Item, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

// ---- the loopback ----------------------------------------------------------

#[derive(Clone, Debug)]
enum Event {
    Connected {
        at: usize,
        peer: String,
        host: bool,
    },
    Disconnected {
        at: usize,
        peer: String,
    },
    Message {
        at: usize,
        from: String,
        text: String,
    },
}

impl Event {
    fn node(&self) -> usize {
        match self {
            Event::Connected { at, .. }
            | Event::Disconnected { at, .. }
            | Event::Message { at, .. } => *at,
        }
    }
}

/// A full mesh of `count` nodes named `peer0`, `peer1`, ...; node 0 is the
/// host. Each link keeps order, as a reliable ordered data channel does.
/// Race messages (states, hits) are never lost; lobby-kind messages may be,
/// at a chosen rate, or one at a time by `drop_next`.
struct Loopback {
    count: usize,
    now_ms: i64,
    rng: u64,
    latency_ms: i64,
    jitter_ms: i64,
    loss_permille: u64,
    gone: BTreeSet<usize>,
    /// pairs of nodes with no path between them (no TURN, TRIAGE T71):
    /// they never connect, and nothing between them arrives
    cut: BTreeSet<(usize, usize)>,
    /// how many states each node sent of apes it does not own (the host
    /// passing them on, TRIAGE T71)
    passed_on: BTreeMap<usize, usize>,
    drops: Vec<(usize, usize, String)>,
    queue: BTreeMap<(i64, u64), (usize, Event)>,
    seq: u64,
    link_last: BTreeMap<(usize, usize), i64>,
}

fn peer_id(i: usize) -> String {
    format!("peer{i}")
}

fn index_of(peer: &str) -> usize {
    peer.trim_start_matches("peer").parse().expect("peer name")
}

impl Loopback {
    fn new(count: usize, seed: u64, latency_ms: i64, jitter_ms: i64) -> Loopback {
        Loopback {
            count,
            now_ms: 0,
            rng: seed ^ 0x9E37_79B9_7F4A_7C15,
            latency_ms,
            jitter_ms,
            loss_permille: 0,
            gone: BTreeSet::new(),
            cut: BTreeSet::new(),
            passed_on: BTreeMap::new(),
            drops: Vec::new(),
            queue: BTreeMap::new(),
            seq: 0,
            link_last: BTreeMap::new(),
        }
    }

    fn next_random(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn arrival(&mut self, from: usize, to: usize) -> i64 {
        let jitter = if self.jitter_ms > 0 {
            (self.next_random() % (self.jitter_ms as u64 + 1)) as i64
        } else {
            0
        };
        let earliest = self.now_ms + self.latency_ms + jitter;
        let last = self.link_last.get(&(from, to)).copied().unwrap_or(0);
        let at = earliest.max(last);
        self.link_last.insert((from, to), at);
        at
    }

    fn push(&mut self, at: i64, from: usize, event: Event) {
        self.seq += 1;
        self.queue.insert((at, self.seq), (from, event));
    }

    /// No path between nodes `a` and `b`, from now on.
    fn cut(&mut self, a: usize, b: usize) {
        self.cut.insert((a.min(b), a.max(b)));
    }

    fn is_cut(&self, a: usize, b: usize) -> bool {
        self.cut.contains(&(a.min(b), a.max(b)))
    }

    /// Node `a` sees node `b` connect (and the other way), after a latency.
    fn connect(&mut self, a: usize, b: usize) {
        if self.is_cut(a, b) {
            return;
        }
        for (x, y) in [(a, b), (b, a)] {
            let at = self.arrival(y, x);
            self.push(
                at,
                y,
                Event::Connected {
                    at: x,
                    peer: peer_id(y),
                    host: y == 0,
                },
            );
        }
    }

    fn connect_all(&mut self) {
        for a in 0..self.count {
            for b in a + 1..self.count {
                self.connect(a, b);
            }
        }
    }

    fn disconnect(&mut self, i: usize) {
        if !self.gone.insert(i) {
            return;
        }
        self.queue.retain(|_, (_, e)| e.node() != i);
        for j in 0..self.count {
            if j != i && !self.gone.contains(&j) {
                let at = self.arrival(i, j);
                self.push(
                    at,
                    i,
                    Event::Disconnected {
                        at: j,
                        peer: peer_id(i),
                    },
                );
            }
        }
    }

    fn lossy(text: &str) -> bool {
        !(text.starts_with("{\"t\":\"state\"") || text.starts_with("{\"t\":\"hit\""))
    }

    fn send(&mut self, from: usize, outgoing: Vec<Outgoing>) {
        if self.gone.contains(&from) {
            return;
        }
        for out in outgoing {
            let targets: Vec<usize> = match &out.to {
                None => (0..self.count).filter(|j| *j != from).collect(),
                Some(p) => vec![index_of(p)],
            };
            for to in targets {
                if to >= self.count || self.gone.contains(&to) || self.is_cut(from, to) {
                    continue;
                }
                if let Some(k) = self
                    .drops
                    .iter()
                    .position(|(f, t, n)| *f == from && *t == to && out.text.contains(n.as_str()))
                {
                    self.drops.remove(k);
                    continue;
                }
                if Loopback::lossy(&out.text)
                    && self.loss_permille > 0
                    && self.next_random() % 1000 < self.loss_permille
                {
                    continue;
                }
                if out.text.starts_with("{\"t\":\"state\"")
                    && !out.text.contains(&format!("\"i\":{from},"))
                {
                    *self.passed_on.entry(from).or_default() += 1;
                }
                let at = self.arrival(from, to);
                self.push(
                    at,
                    from,
                    Event::Message {
                        at: to,
                        from: peer_id(from),
                        text: out.text.clone(),
                    },
                );
            }
        }
    }

    fn advance(&mut self, now_ms: i64) -> Vec<Event> {
        self.now_ms = self.now_ms.max(now_ms);
        let later = self.queue.split_off(&(self.now_ms + 1, 0));
        let due = std::mem::replace(&mut self.queue, later);
        due.into_values().map(|(_, e)| e).collect()
    }
}

// ---- sessions --------------------------------------------------------------

/// Each node's session, its race (once started) and when that race began.
struct Net {
    lb: Loopback,
    nodes: Vec<Online>,
    races: Vec<Option<(Race, i64)>>,
    alive: Vec<bool>,
    course: Course,
    now: i64,
    /// Hits each node took and applied, in order.
    applied: Vec<Vec<Hit>>,
    /// The seed each node's last race was built from.
    seeds: Vec<Option<u64>>,
    /// the map each node's race start named
    maps: Vec<Option<Map>>,
}

const KINDS: [Kind; 5] = [
    Kind::Gorilla,
    Kind::Lemur,
    Kind::Gibbon,
    Kind::Proboscis,
    Kind::Orangutan,
];

impl Net {
    fn new(count: usize, seed: u64, latency_ms: i64, jitter_ms: i64) -> Net {
        let nodes = (0..count)
            .map(|i| {
                let id = format!("client{i}");
                let name = format!("Player {i}");
                if i == 0 {
                    Online::host(&id, &name, KINDS[i], 0)
                } else {
                    Online::join(&id, &name, KINDS[i], 0)
                }
            })
            .collect();
        Net {
            lb: Loopback::new(count, seed, latency_ms, jitter_ms),
            nodes,
            races: (0..count).map(|_| None).collect(),
            alive: vec![true; count],
            course: Course::jungle(),
            now: 0,
            applied: vec![Vec::new(); count],
            seeds: vec![None; count],
            maps: vec![None; count],
        }
    }

    /// Joiners arrive one at a time, so seat `i` is node `i`.
    fn arrive_in_order(&mut self) {
        for b in 1..self.nodes.len() {
            for a in 0..b {
                self.lb.connect(a, b);
            }
            self.run(300);
        }
        self.run(1000);
    }

    /// A seated, settled lobby; seat (and ape) `i` is node `i`.
    fn lobby(count: usize) -> Net {
        let mut net = Net::new(count, 7, 30, 20);
        net.arrive_in_order();
        net
    }

    fn deliver(&mut self, events: Vec<Event>) {
        for e in events {
            let i = e.node();
            if !self.alive[i] {
                continue;
            }
            match e {
                Event::Connected { peer, host, .. } => {
                    self.nodes[i].peer_connected(&peer, host, self.now)
                }
                Event::Disconnected { peer, .. } => self.nodes[i].peer_left(&peer, self.now),
                Event::Message { from, text, .. } => self.nodes[i].receive(&from, &text, self.now),
            }
        }
    }

    /// One frame on node `i`: the session, then the race (stepped to its
    /// clock, the local ape and the host's AI holding W), then the sync.
    fn frame(&mut self, i: usize) {
        let now = self.now;
        let node = &mut self.nodes[i];
        node.update(now);
        if let Some(start) = node.take_race_start() {
            let race = Race::new(
                self.course.clone(),
                &start.entries,
                start.equal_stats,
                start.seed,
            );
            self.races[i] = Some((race, now));
            self.seeds[i] = Some(start.seed);
            self.maps[i] = Some(start.map);
        }
        if let Some((race, t0)) = &mut self.races[i] {
            if node.local_index().is_none() {
                // back in the lobby (a rematch)
                self.races[i] = None;
            } else {
                let due = ((now - *t0) * TICKS_PER_SECOND as i64 / 1000) as u32;
                let push = Controls {
                    fwd: 1,
                    yaw: self.course.start_yaw,
                    ..Controls::default()
                };
                while race.tick < due {
                    let controls: Vec<Controls> = race
                        .racers
                        .iter()
                        .map(|r| {
                            if r.driver == Driver::Remote {
                                Controls::default()
                            } else {
                                push
                            }
                        })
                        .collect();
                    race.step(&controls);
                }
                node.sync_race(now, race);
                // items::apply_hit's stand-in: the test keeps what it was given
                self.applied[i].extend(node.take_hits());
            }
        }
        let out = self.nodes[i].take_outbox();
        self.lb.send(i, out);
    }

    /// Runs every node for `ms` more milliseconds, in 10 ms frames.
    fn run(&mut self, ms: i64) {
        let end = self.now + ms;
        while self.now < end {
            self.now += 10;
            let events = self.lb.advance(self.now);
            self.deliver(events);
            for i in 0..self.nodes.len() {
                if self.alive[i] {
                    self.frame(i);
                }
            }
        }
    }

    fn drop_node(&mut self, i: usize) {
        self.alive[i] = false;
        self.lb.disconnect(i);
    }

    fn ready_all(&mut self) {
        for i in 1..self.nodes.len() {
            if self.alive[i] {
                self.nodes[i].set_ready(true, self.now).expect("ready");
            }
        }
        self.run(1000);
    }

    fn race(&self, i: usize) -> &Race {
        &self.races[i].as_ref().expect("a race").0
    }

    fn race_mut(&mut self, i: usize) -> &mut Race {
        &mut self.races[i].as_mut().expect("a race").0
    }

    /// Every live node's results, without the per-browser `me` mark.
    fn results(&self) -> Vec<Vec<(usize, Option<usize>, Fin)>> {
        (0..self.nodes.len())
            .filter(|i| self.alive[*i])
            .map(|i| {
                self.nodes[i]
                    .results()
                    .into_iter()
                    .map(|r| (r.index, r.place, r.fin))
                    .collect()
            })
            .collect()
    }
}

// ---- the lobby -------------------------------------------------------------

#[test]
fn the_lobby_seats_two_to_four_players() {
    for count in 2..=MAX_APES {
        let net = Net::lobby(count);
        for (i, node) in net.nodes.iter().enumerate() {
            assert_eq!(*node.status(), Status::Lobby, "{count} players, node {i}");
            let view = node.lobby();
            assert_eq!(view.seats.len(), count, "{count} players, node {i}");
            let names: Vec<&str> = view.seats.iter().map(|s| s.name.as_str()).collect();
            let want: Vec<String> = (0..count).map(|k| format!("Player {k}")).collect();
            assert_eq!(names, want, "everyone sees the seats in join order");
            assert_eq!(view.seats[i].kind, KINDS[i]);
            assert!(view.seats[i].me && view.seats.iter().filter(|s| s.me).count() == 1);
            assert!(view.seats[0].host);
            assert_eq!(view.is_host, i == 0);
        }
        // pings were measured both ways
        assert!(net.nodes[0].lobby().seats[1].ping_ms.is_some());
        assert!(net.nodes[1].lobby().seats[0].ping_ms.is_some());
    }
}

#[test]
fn a_fifth_player_is_refused_and_so_is_another_protocol() {
    let mut net = Net::new(5, 3, 20, 10);
    net.arrive_in_order();
    assert_eq!(
        *net.nodes[4].status(),
        Status::Refused {
            reason: Refusal::Full
        }
    );
    assert_eq!(net.nodes[0].lobby().seats.len(), 4);

    let mut net = Net::new(2, 3, 20, 10);
    net.nodes[1] = Online::join("client1", "Old", Kind::Lemur, 0).with_protocol(99);
    net.lb.connect_all();
    net.run(1000);
    assert_eq!(
        *net.nodes[1].status(),
        Status::Refused {
            reason: Refusal::WrongVersion {
                host: tool_core::online::PROTOCOL,
                mine: 99
            }
        }
    );
    assert_eq!(net.nodes[0].lobby().seats.len(), 1);
}

#[test]
fn ready_up_gates_the_start_and_a_settings_change_keeps_it() {
    let mut net = Net::lobby(3);
    let now = net.now;
    assert_eq!(net.nodes[0].start(1, now), Err(OnlineError::NotReady));
    assert_eq!(net.nodes[1].start(1, now), Err(OnlineError::NotHost));
    assert_eq!(
        net.nodes[0].set_ready(true, now),
        Err(OnlineError::HostStarts)
    );
    net.ready_all();
    assert!(net.nodes[0].lobby().can_start);
    assert!(
        net.nodes[2].lobby().seats[1].ready,
        "joiners see each other's marks"
    );
    // TRIAGE T61: no setting the host changes clears a Ready mark
    let now = net.now;
    net.nodes[0].set_equal_stats(true, now).unwrap();
    net.nodes[0].set_lava(true, now).unwrap();
    net.nodes[0].set_ai_count(1, now).unwrap();
    net.nodes[0].set_map(Map::City, now).unwrap();
    net.run(500);
    assert!(
        net.nodes[0].lobby().can_start,
        "a new setting keeps Ready: {:?}",
        net.nodes[0].lobby().seats
    );
    for i in 1..3 {
        let view = net.nodes[i].lobby();
        assert!(view.ready, "node {i} still ready");
        assert!(view.equal_stats && view.lava, "node {i} sees the settings");
        assert_eq!((view.ai_count, view.map), (1, Map::City), "node {i}");
    }
    // cancelling takes the mark away
    let now = net.now;
    net.nodes[1].set_ready(false, now).unwrap();
    net.run(500);
    assert!(!net.nodes[0].lobby().can_start);
    net.ready_all();
    assert!(net.nodes[0].lobby().can_start);
    // the race starts on the settings the joiners saw
    let now = net.now;
    net.nodes[0].start(3, now).unwrap();
    net.run(1000);
    for i in 0..3 {
        assert_eq!(net.maps[i], Some(Map::City), "node {i}");
        assert_eq!(net.nodes[i].entries().len(), 4, "node {i}: three and an AI");
    }
}

#[test]
fn the_host_fills_seats_with_ai_up_to_four_apes() {
    let mut net = Net::lobby(2);
    let now = net.now;
    net.nodes[0].set_ai_count(3, now).unwrap();
    net.run(500);
    let view = net.nodes[1].lobby();
    assert_eq!(view.ai_count, 2, "two humans leave room for two AI apes");
    assert_eq!(view.seats.len(), 4);
    assert!(view.seats[2].ai && view.seats[3].ai);
    let kinds: BTreeSet<Kind> = view.seats.iter().map(|s| s.kind).collect();
    assert_eq!(kinds.len(), 4, "AI apes take kinds no human chose");

    // a lone host needs an AI to start
    let mut solo = Net::lobby(1);
    let now = solo.now;
    assert_eq!(solo.nodes[0].start(1, now), Err(OnlineError::TooFew));
    solo.nodes[0].set_ai_count(1, now).unwrap();
    assert_eq!(solo.nodes[0].start(1, now), Ok(()));
}

#[test]
fn lost_lobby_messages_are_retried() {
    let mut net = Net::new(4, 11, 40, 30);
    net.lb.loss_permille = 400;
    // and the very first Hello and Ready are lost for certain
    net.lb.drops.push((1, 0, "\"hello\"".into()));
    net.lb.drops.push((2, 0, "\"ready\"".into()));
    net.lb.connect_all();
    net.run(6000);
    for node in &net.nodes {
        assert_eq!(*node.status(), Status::Lobby);
        assert_eq!(node.lobby().seats.len(), 4);
    }
    let now = net.now;
    net.nodes[3].set_kind(Kind::Chimpanzee, now).unwrap();
    for i in 1..4 {
        net.nodes[i].set_ready(true, now).unwrap();
    }
    net.run(6000);
    let host = net.nodes[0].lobby();
    let seat = host.seats.iter().find(|s| s.name == "Player 3").unwrap();
    assert_eq!(seat.kind, Kind::Chimpanzee);
    assert!(host.can_start, "every Ready got through: {:?}", host.seats);
    let now = net.now;
    net.nodes[0].start(5, now).unwrap();
    net.run(6000);
    for i in 0..4 {
        assert!(net.races[i].is_some(), "node {i} got the Start");
    }
}

#[test]
fn players_rename_themselves_in_the_lobby_even_when_messages_are_lost() {
    let mut net = Net::lobby(3);
    net.lb.loss_permille = 400;
    // the first rename is lost for certain: it is repeated until seen
    net.lb.drops.push((2, 0, "\"name\"".into()));
    let now = net.now;
    net.nodes[2]
        .set_name("  Koko the Great Ape!!  ", now)
        .unwrap();
    net.nodes[0].set_name("Boss", now).unwrap();
    net.run(5000);
    for (i, node) in net.nodes.iter().enumerate() {
        let names: Vec<String> = node.lobby().seats.into_iter().map(|s| s.name).collect();
        assert_eq!(names, ["Boss", "Player 1", "Koko the Great A"], "node {i}");
    }
    // an empty name changes nothing
    let now = net.now;
    net.nodes[1].set_name("   ", now).unwrap();
    net.run(2000);
    assert_eq!(net.nodes[0].lobby().seats[1].name, "Player 1");
}

#[test]
fn a_screech_plays_for_everyone_in_the_lobby_and_the_sender_too() {
    let mut net = Net::lobby(3);
    let now = net.now;
    net.nodes[1].screech(now).unwrap();
    net.nodes[1].screech(now).unwrap();
    net.nodes[0].screech(now).unwrap();
    net.run(500);
    let heard: Vec<u32> = net.nodes.iter_mut().map(|n| n.take_screeches()).collect();
    assert_eq!(heard, [3, 3, 3], "no cooldown, and nobody is left out");
    assert_eq!(net.nodes[2].take_screeches(), 0, "each plays once");
    // ready or not, everyone hears it
    let now = net.now;
    net.nodes[2].set_ready(true, now).unwrap();
    net.run(500);
    let now = net.now;
    net.nodes[0].screech(now).unwrap();
    net.run(500);
    let heard: Vec<u32> = net.nodes.iter_mut().map(|n| n.take_screeches()).collect();
    assert_eq!(heard, [1, 1, 1]);
}

#[test]
fn every_seats_colours_reach_every_browser_and_its_race() {
    // TRIAGE T49: a palette chosen before joining and one changed in the
    // lobby (its first message lost) show on every browser, and every
    // race draws each ape in its own
    let mut net = Net::new(3, 5, 30, 20);
    net.nodes[2] = Online::join("client2", "Player 2", KINDS[2], 0).with_palette(3);
    net.arrive_in_order();
    net.lb.loss_permille = 300;
    net.lb.drops.push((1, 0, "\"pick\"".into()));
    let now = net.now;
    net.nodes[1].set_palette(2, now).unwrap();
    net.nodes[0].set_palette(1, now).unwrap();
    net.run(5000);
    for (i, node) in net.nodes.iter().enumerate() {
        let worn: Vec<u8> = node.lobby().seats.iter().map(|s| s.palette).collect();
        assert_eq!(worn, [1, 2, 3], "node {i}");
    }
    net.lb.loss_permille = 0;
    let now = net.now;
    net.nodes[0].set_ai_count(1, now).unwrap();
    net.ready_all();
    let now = net.now;
    net.nodes[0].start(9, now).unwrap();
    net.run(500);
    for i in 0..3 {
        let worn: Vec<u8> = net.race(i).racers.iter().map(|r| r.palette).collect();
        assert_eq!(&worn[..3], [1, 2, 3], "node {i}");
        assert_eq!(
            worn[3], 0,
            "the AI ape (a kind no one chose) in its classic"
        );
    }
}

// ---- the race --------------------------------------------------------------

/// A lobby of `humans` with `ai` AI apes, started and running.
fn racing(humans: usize, ai: usize) -> Net {
    let mut net = Net::lobby(humans);
    let now = net.now;
    net.nodes[0].set_ai_count(ai, now).unwrap();
    net.ready_all();
    let now = net.now;
    net.nodes[0].start(42, now).unwrap();
    net.run(500);
    net
}

#[test]
fn every_session_builds_the_same_race() {
    let net = racing(3, 1);
    let first = net.nodes[0].entries().to_vec();
    assert_eq!(first.len(), 4);
    for i in 0..3 {
        let race = net.race(i);
        assert_eq!(*net.nodes[i].status(), Status::Racing);
        assert_eq!(net.nodes[i].local_index(), Some(i));
        let names: Vec<_> = race.racers.iter().map(|r| (&r.name, r.ape.kind)).collect();
        let want: Vec<_> = first.iter().map(|e| (&e.name, e.kind)).collect();
        assert_eq!(names, want, "node {i} has the same apes in the same order");
        assert_eq!(net.seeds[i], Some(42), "and the same seed");
        for (k, r) in race.racers.iter().enumerate() {
            let want = if k == i {
                Driver::Local
            } else if k == 3 && i == 0 {
                Driver::Ai
            } else {
                Driver::Remote
            };
            assert_eq!(r.driver, want, "node {i}, ape {k}");
        }
    }
}

#[test]
fn remote_apes_move_on_other_sessions() {
    let mut net = racing(2, 1);
    // past the countdown and well into the race
    net.run(6000);
    let start = net.course.starts[0];
    let real = net.race(0).racers[0].ape.pos;
    let seen = net.race(1).racers[0].ape.pos;
    assert_ne!(seen, start, "the host's ape moved on the joiner's screen");
    let lag = real.dist(seen);
    assert!(
        lag < tool_core::fx::int(4),
        "drawn about 100 ms behind: {} m apart",
        lag / tool_core::fx::ONE
    );
    // the joiner's ape on the host, and the host's AI on the joiner
    assert_ne!(net.race(0).racers[1].ape.pos, net.course.starts[1]);
    assert_ne!(net.race(1).racers[2].ape.pos, net.course.starts[2]);
    assert!(
        net.race(1).racers[2]
            .ape
            .pos
            .dist(net.race(0).racers[2].ape.pos)
            < tool_core::fx::int(4)
    );
}

#[test]
fn a_hit_reaches_the_victims_browser_only() {
    let mut net = racing(3, 1);
    // joiner 1 hits the host's ape, joiner 2's ape and the AI ape
    net.nodes[1].send_hit(0, 1, Item::Banana);
    net.nodes[1].send_hit(2, 1, Item::Stick);
    net.nodes[1].send_hit(3, 1, Item::Hook);
    // the host hits its own AI: no message needed
    net.nodes[0].send_hit(3, 0, Item::Banana);
    net.run(500);
    let hit = |victim, by, item| Hit { victim, by, item };
    assert_eq!(
        net.applied[0],
        vec![
            hit(3, 0, Item::Banana),
            hit(0, 1, Item::Banana),
            hit(3, 1, Item::Hook)
        ]
    );
    assert_eq!(net.applied[1], vec![]);
    assert_eq!(net.applied[2], vec![hit(2, 1, Item::Stick)]);
}

#[test]
fn a_summoned_vine_reaches_every_browser_at_the_same_index() {
    let mut net = racing(3, 0);
    net.run(4000);
    // joiner 1's ape summons a vine in its own browser
    {
        let race = net.race_mut(1);
        race.racers[1].item = Some(Item::Vine);
        let press = Controls {
            item: true,
            ..Controls::default()
        };
        tool_core::items::use_item(race, 1, &press);
    }
    let mine = net.race(1).summons.clone();
    assert_eq!(mine.len(), 1);
    let slot = mine[0].slot;
    assert_eq!(slot, net.race(1).summon_slot(1, 0));
    net.run(300);
    let key = |s: &tool_core::race::Summon| (s.slot, s.by, s.serial, s.anchor, s.length);
    for i in [0, 2] {
        let race = net.race(i);
        let theirs: Vec<_> = race.summons.iter().map(key).collect();
        assert_eq!(theirs, vec![key(&mine[0])], "browser {i} has it");
        assert_eq!(race.course.vines[slot], net.race(1).course.vines[slot]);
    }
    // sent once: more frames add nothing
    net.run(300);
    assert_eq!(net.race(0).summons.len(), 1);
}

#[test]
fn finish_times_reach_the_host_and_every_results_screen_agrees() {
    let mut net = racing(3, 1);
    net.run(4000);
    // each browser's own ape finishes in its own race; the AI on the host
    for (i, clock) in [(1usize, 500u32), (2, 300), (0, 700)] {
        net.race_mut(i).racers[i].finish = Some(clock);
    }
    net.run(1000);
    assert!(!net.nodes[0].results_final(), "the AI ape is still racing");
    net.race_mut(0).racers[3].finish = Some(900);
    net.run(1000);
    let all = net.results();
    let want = vec![
        (2, Some(1), Fin::Done(300)),
        (1, Some(2), Fin::Done(500)),
        (0, Some(3), Fin::Done(700)),
        (3, Some(4), Fin::Done(900)),
    ];
    for (i, r) in all.iter().enumerate() {
        assert_eq!(*r, want, "node {i}");
        assert!(net.nodes[i].results_final());
    }
    // and each local Race knows the remote finishes, for its places
    assert_eq!(net.race(1).racers[2].finish, Some(300));
    assert_eq!(net.race(2).racers[3].finish, Some(900));
}

#[test]
fn a_player_who_leaves_mid_race_is_dnf_and_stops() {
    let mut net = racing(3, 0);
    net.run(4000);
    net.drop_node(2);
    net.run(300);
    let stopped = net.race(1).racers[2].ape.pos;
    net.run(1000);
    assert_eq!(
        net.race(1).racers[2].ape.pos,
        stopped,
        "the leaver's ape stands still"
    );
    net.race_mut(0).racers[0].finish = Some(800);
    net.race_mut(1).racers[1].finish = Some(600);
    net.run(1000);
    let want = vec![
        (1, Some(1), Fin::Done(600)),
        (0, Some(2), Fin::Done(800)),
        (2, None, Fin::Dnf),
    ];
    for r in net.results() {
        assert_eq!(r, want);
    }
    assert!(net.nodes[0].results_final() && net.nodes[1].results_final());
}

#[test]
fn the_jaguar_eats_on_every_browser_even_when_messages_are_lost() {
    // TRIAGE T37: the host's Floor is Lava reaches every lobby; an eaten
    // ape is eaten in every race, and the results list the eaten after
    // the finishers, the furthest first
    let mut net = Net::lobby(3);
    let now = net.now;
    net.nodes[0].set_lava(true, now).unwrap();
    assert_eq!(net.nodes[1].set_lava(true, now), Err(OnlineError::NotHost));
    net.run(1500);
    assert!((0..3).all(|i| net.nodes[i].lobby().lava));
    let now = net.now;
    net.nodes[0].set_ai_count(1, now).unwrap();
    net.ready_all();
    let now = net.now;
    net.nodes[0].start(42, now).unwrap();
    net.run(4500);
    net.lb.loss_permille = 500;
    // joiner 1 eaten early on, joiner 2 further on, each in its own race
    for (i, s) in [(1usize, 120), (2, 400)] {
        let race = net.race_mut(i);
        let p = race.course.point(tool_core::fx::int(s));
        race.racers[i].ape.pos = p;
        race.racers[i].ape.hint = race.course.nearest_anywhere(p);
        race.eat(i);
    }
    net.run(3000);
    net.lb.loss_permille = 0;
    for i in 0..3 {
        for j in [1, 2] {
            let e = net.race(i).racers[j].eaten;
            assert_eq!(
                e.map(|e| e.at),
                net.race(j).racers[j].eaten.map(|e| e.at),
                "{i} sees {j}"
            );
        }
    }
    net.race_mut(0).racers[0].finish = Some(900);
    net.race_mut(0).racers[3].finish = Some(800);
    net.run(2000);
    for (i, r) in net.results().iter().enumerate() {
        let order: Vec<usize> = r.iter().map(|x| x.0).collect();
        assert_eq!(order, vec![3, 0, 2, 1], "node {i}: {r:?}");
        assert!(matches!(r[2].2, Fin::Eaten(..)) && r[2].1.is_none());
        assert!(net.nodes[i].results_final());
    }
}

#[test]
fn rematch_returns_everyone_to_the_lobby_for_a_new_round() {
    let mut net = racing(3, 1);
    net.run(1000);
    net.drop_node(2);
    net.run(500);
    let now = net.now;
    assert_eq!(net.nodes[1].rematch(now), Err(OnlineError::NotHost));
    net.nodes[0].rematch(now).unwrap();
    net.run(1000);
    for i in 0..2 {
        let view = net.nodes[i].lobby();
        assert_eq!(*net.nodes[i].status(), Status::Lobby);
        assert_eq!(net.nodes[i].round(), 1);
        assert_eq!(view.seats.len(), 3, "two humans who stayed, and the AI");
        // TRIAGE T56: everyone who raced comes back ready
        assert!(
            view.seats.iter().all(|s| s.ready || s.host),
            "node {i}: {:?}",
            view.seats
        );
        assert!(view.ready || view.is_host, "node {i}'s own mark");
        assert!(net.races[i].is_none());
    }
    assert!(net.nodes[0].lobby().can_start, "Start at once");
    // and may un-ready
    let now = net.now;
    net.nodes[1].set_ready(false, now).unwrap();
    net.run(500);
    assert!(!net.nodes[0].lobby().can_start);
    assert!(!net.nodes[1].lobby().ready);
    net.ready_all();
    let now = net.now;
    net.nodes[0].start(9, now).unwrap();
    net.run(4000);
    assert_eq!(net.race(1).racers.len(), 3);
    assert_ne!(net.race(1).racers[0].ape.pos, net.course.starts[0]);
}

#[test]
fn a_player_who_joins_between_rounds_starts_not_ready() {
    // TRIAGE T56: only those who raced come back ready
    let mut net = Net::new(3, 7, 30, 20);
    net.lb.connect(0, 1);
    net.run(1300);
    let now = net.now;
    net.nodes[1].set_ready(true, now).unwrap();
    net.run(500);
    let now = net.now;
    net.nodes[0].start(4, now).unwrap();
    net.run(1000);
    assert!(net.races[1].is_some());
    let now = net.now;
    net.nodes[0].rematch(now).unwrap();
    net.run(1000);
    net.lb.connect(0, 2);
    net.lb.connect(1, 2);
    net.run(1500);
    let seats = net.nodes[0].lobby().seats;
    let ready: Vec<(&str, bool)> = seats.iter().map(|s| (s.name.as_str(), s.ready)).collect();
    assert_eq!(
        ready,
        [("Player 0", false), ("Player 1", true), ("Player 2", false)],
        "the host's own seat has no mark"
    );
    assert!(!net.nodes[2].lobby().ready);
    assert!(
        !net.nodes[0].lobby().can_start,
        "the newcomer must ready up"
    );
    net.ready_all();
    assert!(net.nodes[0].lobby().can_start);
}

#[test]
fn a_host_who_leaves_ends_the_lobby_and_a_race_goes_on_without_it() {
    let mut net = Net::lobby(2);
    net.drop_node(0);
    net.run(500);
    assert_eq!(*net.nodes[1].status(), Status::Lost);

    let mut net = racing(3, 1);
    net.run(3000);
    net.drop_node(0);
    net.run(500);
    assert!(net.nodes[1].host_gone());
    net.race_mut(1).racers[1].finish = Some(400);
    net.race_mut(2).racers[2].finish = Some(500);
    net.run(1000);
    let want = vec![
        (1, Some(1), Fin::Done(400)),
        (2, Some(2), Fin::Done(500)),
        (0, None, Fin::Dnf),
        (3, None, Fin::Dnf),
    ];
    for r in net.results() {
        assert_eq!(r, want);
    }
}

#[test]
fn the_hosts_weather_reaches_every_lobby_and_rides_with_the_start() {
    use tool_core::snow::Weather;
    // TRIAGE T67: one weather for everyone, the host's
    let mut net = Net::lobby(3);
    let now = net.now;
    net.nodes[0].set_map(Map::Ski, now).unwrap();
    net.nodes[0].set_weather(Weather::Snowing, now).unwrap();
    assert_eq!(
        net.nodes[1].set_weather(Weather::Clear, now),
        Err(OnlineError::NotHost)
    );
    net.run(1000);
    for i in 0..3 {
        assert_eq!(net.nodes[i].lobby().weather, Weather::Snowing, "node {i}");
    }
    // back to Clear, every roster to node 2 lost: its Start still carries it
    let now = net.now;
    net.nodes[0].set_weather(Weather::Clear, now).unwrap();
    for _ in 0..40 {
        net.lb.drops.push((0, 2, "\"roster\"".into()));
    }
    net.ready_all();
    assert_eq!(net.nodes[2].weather(), Weather::Snowing, "node 2 missed it");
    let now = net.now;
    net.nodes[0].start(9, now).unwrap();
    net.run(1000);
    for i in 0..3 {
        assert_eq!(*net.nodes[i].status(), Status::Racing);
        assert_eq!(net.nodes[i].weather(), Weather::Clear, "node {i}");
    }
}

#[test]
fn the_hosts_map_reaches_every_lobby_and_rides_with_the_start() {
    let mut net = Net::lobby(3);
    let now = net.now;
    net.nodes[0].set_map(Map::City, now).unwrap();
    // a joiner may not choose
    assert!(net.nodes[1].set_map(Map::Jungle, now).is_err());
    net.run(1000);
    for i in 0..3 {
        assert_eq!(net.nodes[i].lobby().map, Map::City, "node {i}");
    }
    // nobody has readied yet (a map change keeps marks, TRIAGE T61)
    assert!(!net.nodes[0].lobby().can_start);
    // back to the Jungle, every roster to node 2 lost: its Start still
    // carries the host's map
    let now = net.now;
    net.nodes[0].set_map(Map::Jungle, now).unwrap();
    for _ in 0..40 {
        net.lb.drops.push((0, 2, "\"roster\"".into()));
    }
    net.ready_all();
    assert_eq!(
        net.nodes[2].lobby().map,
        Map::City,
        "node 2 missed the change"
    );
    let now = net.now;
    net.nodes[0].start(9, now).unwrap();
    net.run(1000);
    for i in 0..3 {
        assert_eq!(net.maps[i], Some(Map::Jungle), "node {i}");
    }
}

#[test]
fn the_ski_slope_rides_with_the_start_and_is_raced_on_every_browser() {
    // the Ski Slope (2026-10-13): the start carries its key, `ski`
    let mut net = Net::lobby(3);
    let now = net.now;
    net.nodes[0].set_map(Map::Ski, now).unwrap();
    net.run(1000);
    for i in 0..3 {
        assert_eq!(net.nodes[i].lobby().map, Map::Ski, "node {i}");
    }
    net.course = Map::Ski.course();
    net.ready_all();
    let now = net.now;
    net.nodes[0].start(5, now).unwrap();
    net.run(6000);
    for i in 0..3 {
        assert_eq!(net.maps[i], Some(Map::Ski), "node {i}");
        assert_eq!(net.race(i).course.name, "ski");
    }
    // the host's ape moved down the hill on a joiner's screen too
    let real = net.race(0).racers[0].ape.pos;
    let seen = net.race(1).racers[0].ape.pos;
    assert_ne!(seen, net.course.starts[0]);
    assert!(real.dist(seen) < tool_core::fx::int(4));
}

#[test]
fn a_client_from_before_the_ski_slope_is_refused_not_misled() {
    // PROTOCOL 5 knew only the Jungle and the City: a `ski` in its Roster
    // or Start would not read, so it is told to update instead
    let mut net = Net::new(2, 3, 20, 10);
    net.nodes[1] = Online::join("client1", "Old", Kind::Lemur, 0).with_protocol(5);
    net.lb.connect_all();
    net.run(1000);
    assert_eq!(
        *net.nodes[1].status(),
        Status::Refused {
            reason: Refusal::WrongVersion {
                host: tool_core::online::PROTOCOL,
                mine: 5
            }
        }
    );
    assert_eq!(net.nodes[0].lobby().seats.len(), 1);
}

#[test]
fn a_client_from_before_sunset_ruins_is_refused_not_misled() {
    // PROTOCOL 9 did not know `ruins`: it is told to update instead
    let mut net = Net::new(2, 3, 20, 10);
    net.nodes[1] = Online::join("client1", "Old", Kind::Lemur, 0).with_protocol(9);
    net.lb.connect_all();
    net.run(1000);
    assert_eq!(
        *net.nodes[1].status(),
        Status::Refused {
            reason: Refusal::WrongVersion {
                host: tool_core::online::PROTOCOL,
                mine: 9
            }
        }
    );
    assert_eq!(net.nodes[0].lobby().seats.len(), 1);
}

// ---- no path between two joiners (TRIAGE T71) ----------------------------

#[test]
fn joiners_with_no_path_between_them_see_each_other_through_the_host() {
    let mut net = Net::new(3, 7, 30, 20);
    // joiners 1 and 2 both reach the host, never each other
    net.lb.cut(1, 2);
    net.arrive_in_order();
    net.ready_all();
    let now = net.now;
    net.nodes[0].start(42, now).unwrap();
    net.run(6000);
    for (watcher, ape) in [(2, 1), (1, 2), (0, 1), (0, 2)] {
        let seen = net.race(watcher).racers[ape].ape.pos;
        assert_ne!(
            seen, net.course.starts[ape],
            "browser {watcher} sees ape {ape} move"
        );
        let lag = net.race(ape).racers[ape].ape.pos.dist(seen);
        assert!(
            lag < tool_core::fx::int(6),
            "browser {watcher} draws ape {ape} near where it is: {} m apart",
            lag / tool_core::fx::ONE
        );
        assert!(net.nodes[watcher].stalled(net.now).is_empty());
    }
    // a hit between them goes through the host, once
    net.nodes[1].send_hit(2, 1, Item::Banana);
    net.run(500);
    assert_eq!(
        net.applied[2],
        vec![Hit {
            victim: 2,
            by: 1,
            item: Item::Banana
        }]
    );
}

#[test]
fn a_full_mesh_passes_nothing_on() {
    let mut net = racing(3, 0);
    net.run(6000);
    // every joiner hears every other: the host passes no states on
    assert_eq!(net.lb.passed_on.get(&0), None);
    assert!(net.race(2).racers[1].ape.pos != net.course.starts[1]);
}

#[test]
fn an_ape_whose_states_stop_is_marked_stalled() {
    let mut net = racing(2, 0);
    net.run(3000);
    assert!(net.nodes[0].stalled(net.now).is_empty());
    // the link goes quiet without closing
    net.lb.cut(0, 1);
    net.run(tool_core::online::STALL_MS + 200);
    assert_eq!(net.nodes[0].stalled(net.now), vec![1]);
    assert_eq!(net.nodes[1].stalled(net.now), vec![0]);
}

// ---- a refresh (TRIAGE T72) -------------------------------------------------

/// Three seated players (nodes 0–2) and node 3: node 1's browser after a
/// refresh, the same client id on a new transport id, not yet connected.
fn with_refresh() -> Net {
    let mut net = Net::new(4, 7, 30, 20);
    net.nodes[3] = Online::join("client1", "Player 1", KINDS[1], 0);
    net.alive[3] = false;
    net.lb.gone.insert(3);
    for b in 1..3 {
        for a in 0..b {
            net.lb.connect(a, b);
        }
        net.run(300);
    }
    net.run(1000);
    net
}

impl Net {
    /// Node 1 refreshes: its page stops (`leaves` says whether its links
    /// close at once, as `pagehide` does, or linger, as after a crash) and
    /// node 3 comes up in its place.
    fn refresh_node_1(&mut self, leaves: bool) {
        if leaves {
            self.drop_node(1);
        } else {
            self.alive[1] = false;
        }
        self.lb.gone.remove(&3);
        self.alive[3] = true;
        self.lb.connect(0, 3);
        self.lb.connect(2, 3);
        self.run(1500);
    }

    fn seat_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.nodes[0]
            .lobby()
            .seats
            .iter()
            .filter(|s| !s.ai)
            .map(|s| s.name.clone())
            .collect();
        names.sort();
        names
    }
}

#[test]
fn a_player_back_from_a_refresh_keeps_one_seat_in_the_lobby() {
    for leaves in [true, false] {
        let mut net = with_refresh();
        net.refresh_node_1(leaves);
        assert_eq!(*net.nodes[3].status(), Status::Lobby, "leaves: {leaves}");
        // the crashed tab's links time out later: still one seat
        if !leaves {
            net.drop_node(1);
            net.run(1500);
        }
        assert_eq!(
            net.seat_names(),
            vec!["Player 0", "Player 1", "Player 2"],
            "leaves: {leaves}"
        );
        assert_eq!(*net.nodes[3].status(), Status::Lobby);
    }
}

#[test]
fn a_player_back_from_a_refresh_mid_race_waits_for_the_next_round() {
    for leaves in [true, false] {
        let mut net = with_refresh();
        net.ready_all();
        let now = net.now;
        net.nodes[0].start(42, now).unwrap();
        net.run(4000);
        net.refresh_node_1(leaves);
        assert_eq!(*net.nodes[3].status(), Status::Joining);
        assert!(net.nodes[3].underway(), "told the race is under way");
        // its old ape is out of this race, so the race can end
        net.race_mut(0).racers[0].finish = Some(800);
        net.race_mut(2).racers[2].finish = Some(600);
        net.run(1000);
        assert!(net.nodes[0].results_final(), "leaves: {leaves}");
        let fin: Vec<Fin> = net.nodes[0].results().iter().map(|r| r.fin).collect();
        assert_eq!(fin[2], Fin::Dnf);
        // the next round seats it again, ready, and it races as ape 1
        let now = net.now;
        net.nodes[0].rematch(now).unwrap();
        net.run(1500);
        assert_eq!(*net.nodes[3].status(), Status::Lobby);
        assert!(!net.nodes[3].underway());
        let now = net.now;
        net.nodes[0].start(43, now).unwrap();
        net.run(500);
        assert_eq!(*net.nodes[3].status(), Status::Racing, "leaves: {leaves}");
        assert_eq!(net.nodes[3].local_index(), Some(1));
    }
}
