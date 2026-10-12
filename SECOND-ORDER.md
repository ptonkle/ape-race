# Second-order notebook

Stage 6, kept by the agent as the work happens, not afterwards. `B3 Build in self-questioning`.

One row per assumption, surprise or open item. **Kind** is *divergence* (what was assumed, what turned out to be true, what changed), *finding*, or *worklist* (the person's to do). **Status** is *done*, *open*, or *person*.

| # | Kind | What was assumed or found | What is true | What changed | Status |
| --- | --- | --- | --- | --- | --- |
| 1 | | | | | |
| S1 | divergence | Sound would reuse Mandible's synth as is. | Mandible has only fixed-pitch notes and raw noise; vine/whoosh/bonk cues need sweeps and duller air. | `sound.rs` adds per-note frequency sweeps (hz0→hz1), low-passed noise (one-pole, normalised), and three envelopes (Flat, Perc, Swell). MASTER 13 000 and DEFAULT_VOLUME 50 kept from Mandible's loudness fix. | done |
| S2 | finding | The wind loop needs crossfading to loop without a click. | Running each one-pole filter round the buffer twice makes the end state carry into the start; gusts use whole cycles per second (1 and 3). | No crossfade. Test: the seam jump is at most half the largest sample step in the body. | done |
| S3 | divergence | Wind volume follows speed (P14). | The page cannot compute it (rule 4). | `sound::wind_gain(speed)` gives Q16 0..ONE: silent below 8 m/s (`WIND_QUIET`), full at 45 m/s (`WIND_FULL`), smoothstep between. The app must send it each frame; the page multiplies its wind gain node by it. Thresholds are guesses until tuned in play. | open |
| S4 | finding | A field named `env` for envelopes was harmless. | The boundary test bans `env` as an identifier (input/output). | Renamed to `contour`. | done |
| S5 | finding | Rule 6 break checks for sound tests. | Each failed as expected: RIFF size +2 → "grab RIFF size left 4896 right 4894"; volume clamped to ≥1 → "Grab" not silent at 0; volume ignored → "grab: 10276 at 25% vs 10276"; MASTER 40 000 → "hit never clips: 32767"; MASTER 20 000 → "Grab peaks at 7905" (default-volume cap); single filter pass → "seam jump 1063 vs largest step 844"; Finish note 400 ms → "Finish lasts 684 ms"; Go same as Beep → "Beep vs Go"; seed from a counter → "Grab" bytes differ; no clamp in wind_gain → "left: 10513 right: 0". (MASTER 15 000 did not trip the default-volume cap; peaks sit below the loudness sum.) | Restored after each. | done |
| S6 | worklist | The cues sound pleasant. | Made from numbers only; nobody has listened yet. | The person should listen in the browser and say which cue is wrong; each cue is a few lines in `notes()`. | person |
