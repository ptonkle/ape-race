//! Sounds (PLAN P14), made by the core: a small integer synthesiser that
//! writes each cue as a complete WAV file (16-bit mono PCM, 22 050 Hz). The
//! page decodes the bytes and plays them; it holds no frequencies, lengths or
//! levels. The approach follows Mandible's `sound.rs`, with frequency sweeps
//! and low-passed noise added for the jungle cues, and a seamless wind loop.

use serde::{Deserialize, Serialize};

use crate::fx::{self, Dice, ONE};

pub const SAMPLE_RATE: u32 = 22_050;

/// The 16-bit level of a full-scale sound at 100% Volume. Mandible's
/// players found 26 000 too loud; this is its corrected level.
pub const MASTER: i64 = 13_000;

/// The Volume (percent) of a new player. Saved settings keep theirs.
pub const DEFAULT_VOLUME: u8 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cue {
    Grab,
    Release,
    Miss,
    Jump,
    Land,
    Pickup,
    Throw,
    Hit,
    Boost,
    Beep,
    Go,
    Finish,
    Click,
    /// course rewards (group K): a mushroom's bounce, a course banana, a
    /// catch-up vine
    Bounce,
    Banana,
    Catchup,
}

impl Cue {
    pub const ALL: [Cue; 16] = [
        Cue::Grab,
        Cue::Release,
        Cue::Miss,
        Cue::Jump,
        Cue::Land,
        Cue::Pickup,
        Cue::Throw,
        Cue::Hit,
        Cue::Boost,
        Cue::Beep,
        Cue::Go,
        Cue::Finish,
        Cue::Click,
        Cue::Bounce,
        Cue::Banana,
        Cue::Catchup,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Cue::Grab => "grab",
            Cue::Release => "release",
            Cue::Miss => "miss",
            Cue::Jump => "jump",
            Cue::Land => "land",
            Cue::Pickup => "pickup",
            Cue::Throw => "throw",
            Cue::Hit => "hit",
            Cue::Boost => "boost",
            Cue::Beep => "beep",
            Cue::Go => "go",
            Cue::Finish => "finish",
            Cue::Click => "click",
            Cue::Bounce => "bounce",
            Cue::Banana => "banana",
            Cue::Catchup => "catchup",
        }
    }

    pub fn from_key(k: &str) -> Option<Cue> {
        Cue::ALL.into_iter().find(|c| c.key() == k)
    }
}

#[derive(Debug, Clone, Copy)]
enum Wave {
    Sine,
    Square,
    Triangle,
    /// White noise through a one-pole low-pass with this coefficient in
    /// 1/256 (256 is unfiltered; smaller is duller), then normalised.
    Noise(i64),
}

/// How a note's loudness moves over its length.
#[derive(Debug, Clone, Copy)]
enum Env {
    /// 5 ms attack, held, then a linear release over the last third.
    Flat,
    /// 3 ms attack, then a quadratic fall: plucks, bonks and thuds.
    Perc,
    /// Rises over the first 40%, then falls quadratically: whooshes.
    Swell,
}

/// One note: a frequency swept from `hz0` to `hz1` (Hz), start and length in
/// milliseconds, loudness in percent of full scale.
#[derive(Debug, Clone, Copy)]
struct Note {
    hz0: u32,
    hz1: u32,
    at_ms: u32,
    ms: u32,
    loud: i32,
    wave: Wave,
    contour: Env,
}

const fn n(hz: (u32, u32), at_ms: u32, ms: u32, loud: i32, wave: Wave, contour: Env) -> Note {
    Note {
        hz0: hz.0,
        hz1: hz.1,
        at_ms,
        ms,
        loud,
        wave,
        contour,
    }
}

/// The notes of each cue. The loudness of notes that overlap sums to at
/// most 100, so no cue can clip.
fn notes(cue: Cue) -> Vec<Note> {
    use Env::*;
    use Wave::*;
    const AIR: (u32, u32) = (0, 0);
    match cue {
        // A rubbery "thwip": a fast falling pluck with a bright edge.
        Cue::Grab => vec![
            n((950, 240), 0, 95, 60, Sine, Perc),
            n((1500, 520), 0, 50, 20, Triangle, Perc),
            n(AIR, 0, 20, 15, Noise(160), Perc),
        ],
        // A short airy whoosh, lifting slightly.
        Cue::Release => vec![
            n(AIR, 0, 170, 50, Noise(70), Swell),
            n((480, 760), 0, 140, 15, Sine, Swell),
        ],
        // A dull, low whiff: heavier filtered air and a sagging low tone.
        Cue::Miss => vec![
            n(AIR, 0, 220, 55, Noise(22), Swell),
            n((190, 105), 20, 170, 30, Sine, Perc),
        ],
        // A small springy hop.
        Cue::Jump => vec![
            n((280, 720), 0, 120, 45, Triangle, Perc),
            n((560, 1100), 0, 70, 15, Sine, Perc),
        ],
        // A soft thud.
        Cue::Land => vec![
            n((150, 55), 0, 150, 60, Sine, Perc),
            n(AIR, 0, 70, 30, Noise(18), Perc),
        ],
        // A bright, fruity two-step bling.
        Cue::Pickup => vec![
            n((1047, 1047), 0, 90, 40, Sine, Perc),
            n((1568, 1568), 70, 240, 40, Sine, Perc),
            n((2093, 2093), 70, 120, 15, Triangle, Perc),
        ],
        // A longer whoosh with a falling edge.
        Cue::Throw => vec![
            n(AIR, 0, 230, 50, Noise(90), Swell),
            n((720, 300), 20, 190, 18, Sine, Swell),
        ],
        // A cartoon bonk.
        Cue::Hit => vec![
            n((430, 150), 0, 160, 55, Sine, Perc),
            n((860, 300), 0, 80, 20, Triangle, Perc),
            n(AIR, 0, 15, 25, Noise(256), Perc),
        ],
        // A rising zoom.
        Cue::Boost => vec![
            n((150, 600), 0, 420, 22, Square, Flat),
            n((300, 1200), 0, 420, 30, Sine, Flat),
            n(AIR, 0, 420, 18, Noise(60), Swell),
        ],
        Cue::Beep => vec![
            n((660, 660), 0, 150, 50, Triangle, Flat),
            n((660, 660), 0, 150, 20, Sine, Flat),
        ],
        // An octave above the countdown beep, held longer.
        Cue::Go => vec![
            n((1320, 1320), 0, 340, 50, Triangle, Flat),
            n((660, 660), 0, 340, 25, Sine, Flat),
        ],
        // A short rising fanfare ending on a held chord.
        Cue::Finish => vec![
            n((523, 523), 0, 90, 50, Triangle, Flat),
            n((659, 659), 90, 90, 50, Triangle, Flat),
            n((784, 784), 180, 90, 50, Triangle, Flat),
            n((1047, 1047), 270, 300, 55, Triangle, Flat),
            n((523, 523), 270, 300, 30, Sine, Flat),
        ],
        Cue::Click => vec![
            n(AIR, 0, 15, 25, Noise(256), Perc),
            n((1200, 1200), 0, 28, 20, Sine, Perc),
        ],
        Cue::Bounce | Cue::Banana | Cue::Catchup => reward_notes(cue),
    }
}

/// The course rewards' cues (group K: TRIAGE T48, T43, T45).
fn reward_notes(cue: Cue) -> Vec<Note> {
    use Env::*;
    use Wave::*;
    const AIR: (u32, u32) = (0, 0);
    match cue {
        // A springy "boing" that keeps rising into a little zoom.
        Cue::Bounce => vec![
            n((180, 820), 0, 260, 50, Triangle, Perc),
            n((360, 1640), 0, 170, 20, Sine, Perc),
            n((420, 980), 110, 280, 25, Sine, Swell),
        ],
        // A quick, high double blip, lighter than a crate's Pickup.
        Cue::Banana => vec![
            n((1760, 1760), 0, 60, 40, Sine, Perc),
            n((2637, 2637), 45, 120, 40, Sine, Perc),
        ],
        // A short rising sparkle.
        Cue::Catchup => vec![
            n((620, 1480), 0, 220, 40, Sine, Swell),
            n((1240, 2960), 60, 170, 20, Triangle, Perc),
            n(AIR, 0, 180, 15, Noise(120), Swell),
        ],
        _ => Vec::new(),
    }
}

fn ms_to_samples(ms: u32) -> usize {
    (ms as u64 * SAMPLE_RATE as u64 / 1000) as usize
}

/// White noise in [-ONE, ONE] from the dice.
fn white(dice: &mut Dice) -> i64 {
    dice.below(2 * ONE as u32 + 1) as i64 - ONE as i64
}

/// Scales a buffer so its peak is ONE (no change if it is all zero).
fn normalise(buf: &mut [i64]) {
    let peak = buf.iter().map(|v| v.abs()).max().unwrap_or(0);
    if peak > 0 {
        for v in buf.iter_mut() {
            *v = *v * ONE as i64 / peak;
        }
    }
}

/// One note's waveform, before the envelope, as Q16 in [-ONE, ONE].
fn wave(note: &Note, count: usize, dice: &mut Dice) -> Vec<i64> {
    let mut out = Vec::with_capacity(count);
    match note.wave {
        Wave::Noise(k) => {
            let mut y: i64 = 0;
            for _ in 0..count {
                y += ((white(dice) - y) * k) >> 8;
                out.push(y);
            }
            normalise(&mut out);
        }
        shape => {
            // Phase in 1/2^32 of a cycle; the top 16 bits are fx angle units.
            let mut phase: u32 = 0;
            let span = count.max(1) as i64;
            for i in 0..count {
                let hz = note.hz0 as i64 + (note.hz1 as i64 - note.hz0 as i64) * i as i64 / span;
                let step = ((hz << 32) / SAMPLE_RATE as i64) as u32;
                phase = phase.wrapping_add(step);
                let a = (phase >> 16) as i32;
                let s = match shape {
                    Wave::Square => {
                        if a < fx::HALF {
                            ONE
                        } else {
                            -ONE
                        }
                    }
                    Wave::Triangle => {
                        let t = if a < fx::HALF { a } else { fx::TURN - 1 - a };
                        t * 4 - ONE
                    }
                    _ => fx::sin(a),
                };
                out.push(s as i64);
            }
        }
    }
    out
}

/// The envelope at sample `i` of `count`, in 1/1024.
fn envelope(contour: Env, i: usize, count: usize) -> i64 {
    let count = count.max(1) as i64;
    let i = i as i64;
    let fall = |from: i64| {
        let len = (count - from).max(1);
        let rem = count - i;
        rem * rem * 1024 / (len * len)
    };
    match contour {
        Env::Flat => {
            let attack = (SAMPLE_RATE as i64 / 200).max(1);
            let release = (count / 3).max(1);
            if i < attack {
                i * 1024 / attack
            } else if i + release > count {
                (count - i) * 1024 / release
            } else {
                1024
            }
        }
        Env::Perc => {
            let attack = (SAMPLE_RATE as i64 * 3 / 1000).max(1);
            if i < attack {
                i * 1024 / attack
            } else {
                fall(attack)
            }
        }
        Env::Swell => {
            let peak = (count * 2 / 5).max(1);
            if i < peak {
                i * 1024 / peak
            } else {
                fall(peak)
            }
        }
    }
}

/// Converts a Q16 mix (ONE is full scale) to 16-bit samples at `volume`
/// percent of `MASTER`.
fn to_pcm(mix: &[i64], volume: u8) -> Vec<i16> {
    let vol = volume.min(100) as i64;
    mix.iter()
        .map(|v| {
            let s = v * MASTER / ONE as i64 * vol / 100;
            s.clamp(i16::MIN as i64, i16::MAX as i64) as i16
        })
        .collect()
}

/// The 16-bit samples of a cue at `volume` percent.
pub fn samples(cue: Cue, volume: u8) -> Vec<i16> {
    let ns = notes(cue);
    let end_ms = ns.iter().map(|n| n.at_ms + n.ms).max().unwrap_or(0) + 15;
    let len = ms_to_samples(end_ms);
    let mut mix = vec![0i64; len];
    let mut dice = Dice(0x5EED_A9E0 + cue as u64);
    for note in &ns {
        let start = ms_to_samples(note.at_ms);
        let count = ms_to_samples(note.ms);
        let w = wave(note, count, &mut dice);
        for (i, s) in w.into_iter().enumerate() {
            if let Some(m) = mix.get_mut(start + i) {
                *m += s * envelope(note.contour, i, count) / 1024 * note.loud as i64 / 100;
            }
        }
    }
    to_pcm(&mix, volume)
}

/// The wind loop's samples: one second of low-passed noise with slow gusts.
/// The filters run round the buffer twice, so the state at the end carries
/// into the start, and the gusts complete whole cycles: the loop has no
/// seam.
pub fn wind_samples(volume: u8) -> Vec<i16> {
    let len = SAMPLE_RATE as usize;
    let mut dice = Dice(0x00A9_E71D);
    let raw: Vec<i64> = (0..len).map(|_| white(&mut dice)).collect();
    let circular = |src: &[i64], k: i64| {
        let mut out = vec![0i64; src.len()];
        let mut y: i64 = 0;
        for _ in 0..2 {
            for (o, x) in out.iter_mut().zip(src) {
                y += ((x - y) * k) >> 8;
                *o = y;
            }
        }
        out
    };
    // Two passes: a low rumble, about 300 Hz down, softened again.
    let mut buf = circular(&circular(&raw, 22), 48);
    normalise(&mut buf);
    for (i, v) in buf.iter_mut().enumerate() {
        // Gusts at 1 and 3 cycles a second around 3/4.
        let a1 = (i as i64 * fx::TURN as i64 / len as i64) as i32;
        let gust =
            ONE as i64 * 3 / 4 + fx::sin(a1) as i64 / 6 + fx::sin(a1.wrapping_mul(3)) as i64 / 12;
        // 60% of full scale at the loudest.
        *v = *v * gust / ONE as i64 * 6 / 10;
    }
    to_pcm(&buf, volume)
}

/// A complete WAV file (RIFF, 16-bit mono PCM) holding `pcm`.
fn wav_file(pcm: &[i16]) -> Vec<u8> {
    let data_len = (pcm.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in pcm {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

/// A cue as a complete WAV file at `volume` percent.
pub fn wav(cue: Cue, volume: u8) -> Vec<u8> {
    wav_file(&samples(cue, volume))
}

/// The wind loop at `volume` percent: one second that loops cleanly. The
/// page plays it continuously with the gain `wind_gain` gives.
pub fn wind(volume: u8) -> Vec<u8> {
    wav_file(&wind_samples(volume))
}

/// Speed (Q16 m/s) below which the wind is silent.
pub const WIND_QUIET: i32 = fx::int(8);
/// Speed (Q16 m/s) at which the wind is at full gain.
pub const WIND_FULL: i32 = fx::int(45);

/// The wind loop's gain (Q16, 0..=ONE) at a speed in Q16 m/s: silent below
/// `WIND_QUIET`, full from `WIND_FULL`, smoothstep-eased between.
pub fn wind_gain(speed: i32) -> i32 {
    let t = fx::div(speed - WIND_QUIET, WIND_FULL - WIND_QUIET).clamp(0, ONE);
    fx::mul(fx::mul(t, t), 3 * ONE - 2 * t).clamp(0, ONE)
}

// ---- the lobby screech (TRIAGE T44) ----

/// The lobby screech's sound key (`App::sound`).
pub const SCREECH: &str = "screech";

/// The lobby's chimp screech at `volume` percent, as a WAV file: a
/// placeholder, synthesised (two rising hoots, then three shrieks with a
/// raspy edge) until the person's own recording arrives.
///
/// **Swapping in the real file** (see SECOND-ORDER L4): convert it to 16-bit
/// mono PCM at 22 050 Hz (`ffmpeg -i in.mp3 -ac 1 -ar 22050 -sample_fmt s16
/// crates/core/src/screech.wav`), then make this function's body
/// `scaled_wav(include_bytes!("screech.wav"), volume)`: the core keeps the
/// bytes in the wasm, Volume still applies, and nothing else changes.
pub fn screech(volume: u8) -> Vec<u8> {
    use Env::*;
    use Wave::*;
    let ns = [
        // hoo-hoo, rising
        n((420, 620), 0, 150, 60, Triangle, Swell),
        n((520, 760), 190, 150, 60, Triangle, Swell),
        // three shrieks, each sweeping up and falling off, with rasp
        n((900, 1700), 400, 210, 55, Square, Flat),
        n((0, 0), 400, 210, 25, Noise(150), Flat),
        n((1000, 1850), 650, 210, 55, Square, Flat),
        n((0, 0), 650, 210, 25, Noise(150), Flat),
        n((1100, 2000), 900, 320, 55, Square, Perc),
        n((0, 0), 900, 320, 25, Noise(150), Perc),
    ];
    let end_ms = ns.iter().map(|n| n.at_ms + n.ms).max().unwrap_or(0) + 15;
    let mut mix = vec![0i64; ms_to_samples(end_ms)];
    let mut dice = Dice(0x5C2E_EC40);
    for note in &ns {
        let start = ms_to_samples(note.at_ms);
        let count = ms_to_samples(note.ms);
        for (i, v) in wave(note, count, &mut dice).into_iter().enumerate() {
            if let Some(m) = mix.get_mut(start + i) {
                *m += v * envelope(note.contour, i, count) / 1024 * note.loud as i64 / 100;
            }
        }
    }
    wav_file(&to_pcm(&mix, volume))
}

/// A supplied 16-bit mono WAV at `volume` percent: its samples scaled as
/// the synthesised cues are (full scale is `MASTER` at 100%), in a fresh
/// header at `SAMPLE_RATE`. For the screech once its file arrives. A file
/// with no `data` chunk gives silence.
pub fn scaled_wav(file: &[u8], volume: u8) -> Vec<u8> {
    // walk the RIFF chunks after "RIFF", size, "WAVE" to the "data" chunk
    let mut at = 12;
    let mut data: &[u8] = &[];
    while let Some(head) = file.get(at..at + 8) {
        let len = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
        let body = file
            .get(at + 8..(at + 8 + len).min(file.len()))
            .unwrap_or(&[]);
        if &head[0..4] == b"data" {
            data = body;
            break;
        }
        at += 8 + len + len % 2;
    }
    let mix: Vec<i64> = data
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| i16::from_le_bytes(*b) as i64 * ONE as i64 / i16::MAX as i64)
        .collect();
    wav_file(&to_pcm(&mix, volume))
}

// ---- ape voices: the hype call-outs (TRIAGE T22) ----
//
// A tiny formant voice: a sawtooth "throat" at the syllable's pitch, with a
// little vibrato and breath, through three resonators (the vowel's
// formants) in a row. Hoots are "oo", the climax opens to "ah"; a second
// voice a fifth below joins the top call, so it sounds like a troop.

/// A vowel's three formants, Hz.
#[derive(Debug, Clone, Copy)]
struct Vowel(u32, u32, u32);

const OO: Vowel = Vowel(320, 800, 2300);
const OH: Vowel = Vowel(480, 900, 2400);
const AH: Vowel = Vowel(760, 1250, 2550);
const EE: Vowel = Vowel(330, 2100, 2900);

/// The formants' bandwidths, Hz.
const BANDS: [u32; 3] = [90, 120, 180];

/// One voiced syllable: pitch from `hz0` to `hz1`, the vowel from `from`
/// to `to`, start and length in ms, loudness in percent (the whole call is
/// normalised afterwards), and a growl (percent of amplitude wobble).
#[derive(Debug, Clone, Copy)]
struct Syllable {
    at_ms: u32,
    ms: u32,
    hz0: u32,
    hz1: u32,
    from: Vowel,
    to: Vowel,
    loud: i32,
    growl: i32,
}

const fn syl(at_ms: u32, ms: u32, hz: (u32, u32), from: Vowel, to: Vowel, loud: i32) -> Syllable {
    Syllable {
        at_ms,
        ms,
        hz0: hz.0,
        hz1: hz.1,
        from,
        to,
        loud,
        growl: 0,
    }
}

/// The hype levels' calls: 1 "hoo-HOO", 2 "hoo-hoo-WAH", 3 a pant-hoot
/// climax "hoo-hoo-hoo-WAAH-HAH" with a troop joining in.
fn voice_syllables(level: u8) -> Vec<Syllable> {
    match level {
        1 => vec![
            syl(0, 140, (300, 360), OO, OO, 70),
            syl(170, 220, (380, 470), OO, OH, 90),
        ],
        2 => vec![
            syl(0, 110, (330, 390), OO, OO, 70),
            syl(130, 110, (400, 470), OO, OO, 80),
            syl(260, 260, (480, 620), OO, AH, 95),
        ],
        _ => {
            let mut v = vec![
                syl(0, 80, (360, 430), OO, OO, 60),
                syl(100, 80, (440, 520), OO, OO, 70),
                syl(200, 80, (520, 610), OO, OO, 80),
                Syllable {
                    growl: 25,
                    ..syl(300, 260, (650, 980), OO, AH, 100)
                },
                Syllable {
                    growl: 30,
                    ..syl(580, 170, (960, 820), AH, EE, 90)
                },
            ];
            // the troop: the last two a fifth below, a little late
            v.push(Syllable {
                growl: 20,
                ..syl(320, 250, (430, 650), OO, AH, 55)
            });
            v.push(Syllable {
                growl: 20,
                ..syl(600, 160, (640, 550), AH, OH, 50)
            });
            v
        }
    }
}

/// Coefficients in 1/2^20.
const Q20: i64 = 1 << 20;

/// A syllable's envelope at sample `i` of `count`, in 1/1024: 12 ms in,
/// held, then a quadratic fall over the last 45%.
fn voice_envelope(i: usize, count: usize) -> i64 {
    let count = count.max(1) as i64;
    let i = i as i64;
    let attack = (SAMPLE_RATE as i64 * 12 / 1000).max(1);
    let fall = (count * 45 / 100).max(1);
    if i < attack {
        i * 1024 / attack
    } else if i + fall > count {
        let rem = count - i;
        rem * rem * 1024 / (fall * fall)
    } else {
        1024
    }
}

/// One syllable as Q16 samples, normalised to a peak of ONE.
fn syllable(s: &Syllable, dice: &mut Dice) -> Vec<i64> {
    let count = ms_to_samples(s.ms);
    let span = count.max(1) as i64;
    let lerp = |a: u32, b: u32, i: usize| a as i64 + (b as i64 - a as i64) * i as i64 / span;
    // the throat: a sawtooth with 5.5 Hz vibrato of 3%, and some breath
    let mut buf = Vec::with_capacity(count);
    let mut phase: u32 = 0;
    for i in 0..count {
        let vib = fx::sin((i as i64 * 11 * fx::TURN as i64 / 2 / SAMPLE_RATE as i64) as i32) as i64;
        let hz = lerp(s.hz0, s.hz1, i) * (ONE as i64 + vib * 3 / 100) / ONE as i64;
        phase = phase.wrapping_add(((hz << 32) / SAMPLE_RATE as i64) as u32);
        let a = (phase >> 16) as i64;
        let saw = a * 2 - ONE as i64;
        buf.push(saw + white(dice) / 12);
    }
    // the vowel: three resonators in a row, each gliding from `from` to `to`
    let from = [s.from.0, s.from.1, s.from.2];
    let to = [s.to.0, s.to.1, s.to.2];
    for k in 0..3 {
        // r ≈ 1 - π·bandwidth / rate
        let r = Q20 - BANDS[k] as i64 * 355 * Q20 / 113 / SAMPLE_RATE as i64;
        let c = -(r * r / Q20);
        let (mut y1, mut y2) = (0i64, 0i64);
        for (i, x) in buf.iter_mut().enumerate() {
            let f = lerp(from[k], to[k], i);
            let theta = (f * fx::TURN as i64 / SAMPLE_RATE as i64) as i32;
            let b = 2 * r * (fx::cos(theta) as i64 * (Q20 / ONE as i64)) / Q20;
            let a = Q20 - b - c;
            let y = (a * *x + b * y1 + c * y2) / Q20;
            y2 = y1;
            y1 = y;
            *x = y;
        }
        normalise(&mut buf);
    }
    for (i, x) in buf.iter_mut().enumerate() {
        // a growl: the loudness wobbling at 32 Hz
        let wob = fx::sin((i as i64 * 32 * fx::TURN as i64 / SAMPLE_RATE as i64) as i32) as i64;
        let g = ONE as i64 - s.growl as i64 * (ONE as i64 - wob) / 200;
        *x = *x * g / ONE as i64 * voice_envelope(i, count) / 1024;
    }
    buf
}

/// The peak of a call-out, percent of full scale.
const VOICE_LOUD: i64 = 90;

/// The 16-bit samples of hype level `level`'s call-out (1 to 3) at `volume`
/// percent.
pub fn voice_samples(level: u8, volume: u8) -> Vec<i16> {
    let ss = voice_syllables(level.clamp(1, 3));
    let end_ms = ss.iter().map(|s| s.at_ms + s.ms).max().unwrap_or(0) + 15;
    let mut mix = vec![0i64; ms_to_samples(end_ms)];
    let mut dice = Dice(0x00A9_E5A1 + level as u64);
    for s in &ss {
        let start = ms_to_samples(s.at_ms);
        for (i, v) in syllable(s, &mut dice).into_iter().enumerate() {
            if let Some(m) = mix.get_mut(start + i) {
                *m += v * s.loud as i64 / 100;
            }
        }
    }
    normalise(&mut mix);
    for m in mix.iter_mut() {
        *m = *m * VOICE_LOUD / 100;
    }
    to_pcm(&mix, volume)
}

/// Hype level `level`'s call-out as a complete WAV file.
pub fn voice(level: u8, volume: u8) -> Vec<u8> {
    wav_file(&voice_samples(level, volume))
}

/// The level of a call-out's sound key (`hype1` to `hype3`, as
/// `hype::voice_key` names them).
pub fn voice_level(key: &str) -> Option<u8> {
    match key {
        "hype1" => Some(1),
        "hype2" => Some(2),
        "hype3" => Some(3),
        _ => None,
    }
}

#[cfg(test)]
mod voice_tests {
    use super::*;

    fn peak(s: &[i16]) -> i32 {
        s.iter().map(|v| (*v as i32).abs()).max().unwrap_or(0)
    }

    #[test]
    fn each_hype_level_has_its_own_short_call_at_a_safe_level() {
        let calls: Vec<Vec<i16>> = (1..=3).map(|l| voice_samples(l, 100)).collect();
        for (k, c) in calls.iter().enumerate() {
            let ms = c.len() * 1000 / SAMPLE_RATE as usize;
            assert!((250..=800).contains(&ms), "level {}: {ms} ms", k + 1);
            let p = peak(c);
            assert!(
                p > 8_000 && p < i16::MAX as i32,
                "level {}: peak {p}",
                k + 1
            );
            let quiet = peak(&voice_samples(k as u8 + 1, DEFAULT_VOLUME));
            assert!(quiet <= i16::MAX as i32 / 5, "level {}: {quiet}", k + 1);
            assert!(voice_samples(k as u8 + 1, 0).iter().all(|s| *s == 0));
            check(&voice(k as u8 + 1, 70), k + 1);
        }
        assert_ne!(calls[0], calls[1]);
        assert_ne!(calls[1], calls[2]);
        // the top call is the longest
        assert!(calls[2].len() > calls[1].len() && calls[1].len() > calls[0].len());
        for l in 1..=3u8 {
            assert_eq!(voice_level(&format!("hype{l}")), Some(l));
        }
        assert_eq!(voice_level("grab"), None);
    }

    fn check(w: &[u8], level: usize) {
        assert_eq!(&w[0..4], b"RIFF", "level {level}");
        let data_len = u32::from_le_bytes([w[40], w[41], w[42], w[43]]) as usize;
        assert_eq!(w.len(), 44 + data_len, "level {level}");
    }

    #[test]
    fn a_call_is_voiced_not_noise() {
        // a voice repeats at its pitch: neighbouring samples are close, far
        // more than noise's would be
        let s = voice_samples(2, 100);
        let rough: i64 = s
            .windows(2)
            .map(|w| (w[0] as i64 - w[1] as i64).abs())
            .sum::<i64>()
            / s.len() as i64;
        let level: i64 = s.iter().map(|v| (*v as i64).abs()).sum::<i64>() / s.len() as i64;
        assert!(rough * 3 < level, "step {rough} vs level {level}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_header(w: &[u8], what: &str) {
        assert_eq!(&w[0..4], b"RIFF", "{what}");
        let riff = u32::from_le_bytes([w[4], w[5], w[6], w[7]]) as usize;
        assert_eq!(riff + 8, w.len(), "{what} RIFF size");
        assert_eq!(&w[8..16], b"WAVEfmt ", "{what}");
        assert_eq!(u16::from_le_bytes([w[20], w[21]]), 1, "{what} PCM");
        assert_eq!(u16::from_le_bytes([w[22], w[23]]), 1, "{what} mono");
        let rate = u32::from_le_bytes([w[24], w[25], w[26], w[27]]);
        assert_eq!(rate, SAMPLE_RATE, "{what}");
        assert_eq!(u16::from_le_bytes([w[34], w[35]]), 16, "{what} 16-bit");
        assert_eq!(&w[36..40], b"data", "{what}");
        let data_len = u32::from_le_bytes([w[40], w[41], w[42], w[43]]) as usize;
        assert!(data_len > 0, "{what} has samples");
        assert_eq!(w.len(), 44 + data_len, "{what} data size");
    }

    fn peak(s: &[i16]) -> i32 {
        s.iter().map(|v| (*v as i32).abs()).max().unwrap_or(0)
    }

    #[test]
    fn every_cue_and_the_wind_are_valid_wav_files() {
        for c in Cue::ALL {
            check_header(&wav(c, DEFAULT_VOLUME), c.key());
            assert_eq!(Cue::from_key(c.key()), Some(c));
        }
        let w = wind(DEFAULT_VOLUME);
        check_header(&w, "wind");
        assert_eq!(w.len(), 44 + 2 * SAMPLE_RATE as usize, "wind is one second");
    }

    #[test]
    fn cues_are_short() {
        for c in Cue::ALL {
            let len = samples(c, 100).len();
            assert!(
                len < ms_to_samples(600),
                "{c:?} lasts {} ms",
                len * 1000 / SAMPLE_RATE as usize
            );
        }
    }

    #[test]
    fn volume_zero_is_silent() {
        for c in Cue::ALL {
            assert!(samples(c, 0).iter().all(|s| *s == 0), "{c:?}");
        }
        assert!(wind_samples(0).iter().all(|s| *s == 0), "wind");
    }

    #[test]
    fn volume_scales_amplitude_and_nothing_clips() {
        let mut all: Vec<(&str, Vec<i16>, Vec<i16>)> = Cue::ALL
            .iter()
            .map(|c| (c.key(), samples(*c, 100), samples(*c, 25)))
            .collect();
        all.push(("wind", wind_samples(100), wind_samples(25)));
        for (k, loud, quiet) in all {
            let p = peak(&loud);
            let q = peak(&quiet);
            assert!(p > 3_000, "{k} is audible: {p}");
            assert!(p < i16::MAX as i32, "{k} never clips: {p}");
            // A quarter of the volume is about a quarter of the amplitude.
            assert!((q * 4 - p).abs() <= p / 20, "{k}: {q} at 25% vs {p}");
        }
    }

    #[test]
    fn sounds_are_quiet_at_the_default_volume() {
        // Mandible's lesson: no cue peaks above a fifth of full scale at
        // the default Volume.
        for c in Cue::ALL {
            let p = peak(&samples(c, DEFAULT_VOLUME));
            assert!(p <= i16::MAX as i32 / 5, "{c:?} peaks at {p}");
        }
    }

    #[test]
    fn the_wind_loops_without_a_click() {
        let s = wind_samples(100);
        let step = |a: i16, b: i16| (a as i32 - b as i32).abs();
        let seam = step(s[s.len() - 1], s[0]);
        let body = s.windows(2).map(|w| step(w[0], w[1])).max().unwrap();
        assert!(seam <= body / 2, "seam jump {seam} vs largest step {body}");
    }

    #[test]
    fn the_same_cue_gives_the_same_bytes() {
        for c in Cue::ALL {
            assert_eq!(wav(c, 70), wav(c, 70), "{c:?}");
        }
        assert_eq!(wind(70), wind(70));
    }

    #[test]
    fn cues_differ() {
        for (i, a) in Cue::ALL.iter().enumerate() {
            for b in &Cue::ALL[i + 1..] {
                assert_ne!(samples(*a, 100), samples(*b, 100), "{a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn wind_gain_follows_speed() {
        assert_eq!(wind_gain(0), 0);
        assert_eq!(wind_gain(fx::int(5)), 0);
        assert_eq!(wind_gain(fx::int(45)), ONE);
        assert_eq!(wind_gain(fx::int(200)), ONE);
        assert_eq!(wind_gain(-fx::int(30)), 0);
        let mut last = 0;
        for v in 0..=60 * 4 {
            let g = wind_gain(fx::int(v) / 4);
            assert!(g >= last, "falls at {v}/4 m/s: {g} < {last}");
            assert!((0..=ONE).contains(&g));
            last = g;
        }
        // Eased: a quarter of the way in is under a quarter of the gain.
        let q = WIND_QUIET + (WIND_FULL - WIND_QUIET) / 4;
        assert!(wind_gain(q) < ONE / 4);
    }
}
