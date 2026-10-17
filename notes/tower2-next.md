# Tower v2: the next round (queued 2026-10-17, for a fresh session)

Branch `tower2`, worktree `~/slop/ape-race-tower`. 1.14.0 is built there and not deployed (main is 1.13.4). Read this, then the TRIAGE rows named below. Local build: `./scripts/build.sh`, served at http://127.0.0.1:8000/ by `python3 scripts/serve.py 8000`.

## Rules for this round (the person's)

- **No AI or medal re-runs or retuning unless the person asks** (memory `ai-times-on-request`). The AI-order tests and the medal test may fail after these changes: name them, leave them.
- One agent at a time, small briefs, grep and ranged reads (memory `map-build-cost`). No parallel side work.
- Deploy only when the person says.

## The queue, in this order

| # | TRIAGE | Change | Where |
| --- | --- | --- | --- |
| 1 | T100 | The tube 2× the diameter (radius 17.5 → 35 m); the 22 ring/slab pads replaced by **150** discs **10 m across**, each tilted a random 0–25° in a random direction (bounce off the face, so sideways too), floating free, spread evenly through the whole volume (min distance; ~15 per 30 m), one fixed placement; one count constant, and the first N stay even for any N. Hitting one bounces you up and kills most of the fall speed (~2 s). Placement confirmed by the person; build exactly as the row says. | `towermodels/tube.rs` (`TUBE_RADIUS`), `tower/layout.rs` (`TUBE_PADS`, `tube_pad`), `tower.rs` (`TubePad`, `PadShape`), `tower/rules.rs` (`tube_pads`), `towerscape/art.rs` (`void_pad`), the sling row and tunnel mouth (they meet the tube's foot) |
| 2 | T99 | Entering the void (crossing the portal), zero the ape's horizontal speed, so the fall starts straight down whatever the entry. | `tower/rules.rs`, the portal crossing; TC2-9 |
| 3 | T98 | Less fog: tower 2 visible from tower 1 (~380 m apart; fog thick by ~330 m). Watch the worst frame (5,197 of 5,500). | `towerscape.rs` sky/fog by height, cull distances |
| 4 | T97 | Start with a short climb round tower 1's outside before the lower bridge (a new leg after `Leg::Start`). AI times don't matter. | `tower/layout.rs` `LEGS` |
| 5 | T96 | About a third as many vines inside tower 1. | `tower/layout.rs` ~528, the rings every `LAYER_RISE` |

T100 and T99 both touch the tube, so do them in one agent (or T99 inline: it is small). T96–T98 are small enough to do inline or in one agent together.

## After the queue

Rebuild, the person plays it locally, shots of the new tube at 1920×1080 on the journal `#tower` (one batched publish), quick check (`BUILDER_PYTHON=/home/peter/slop/ape-race/.venv/bin/python ./scripts/timed-checks.sh quick`), then the deploy question. Still open from the night: `notes/tower2-morning.md` questions (the inside climb's spread is partly answered by T96; the tube's green glow is replaced by T100's new pads).
