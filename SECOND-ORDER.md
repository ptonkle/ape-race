# Second-order notebook

Stage 6, kept by the agent as the work happens, not afterwards. `B3 Build in self-questioning`.

One row per assumption, surprise or open item. **Kind** is *divergence* (what was assumed, what turned out to be true, what changed), *finding*, or *worklist* (the person's to do). **Status** is *done*, *open*, or *person*.

Closed rows (done, decided, or about the Monkey and the first Mape's Tower) are in `notes/second-order-archive.md`; agents need not read it. Move a row there when it closes.

| # | Kind | What was assumed or found | What is true | What changed | Status |
| --- | --- | --- | --- | --- | --- |
| HK3 | decision | T90 (1.13.x) changed replay hashes on purpose: saved replays where a Hook was caught (or, on the City, an AI was overdue on a vine) no longer play back the same. | `replay::FORMAT` is still 2, never bumped through earlier AI and item changes either; a bump marks old replays "older version" (R55-3). | Ask: bump `FORMAT` (perhaps with T104)? | person |
| V6 | measurement | T37: how often the AI is eaten (not retuned, as asked). | `cargo test --release -p tool_core --test ai -- --ignored --nocapture lava_report`, four apes (Chimpanzee, Gorilla, Gibbon, Lemur) × eight layouts, 150 s limit: Sloth finished 13/32, eaten 19/32; Monkey 16 / 16; Ape 12 / 20; King Kong (the default) 5 / 27. King Kong is eaten mostly 7–20 s in (150–500 m): its fast line lands and rolls on. So in a lava race the AI is usually gone early and the player races the floor. | If the person wants rivals who last, the AI needs a lava-aware planner (avoid flights that land); a follow-up, not done. | open: reopened 2026-10-17 by the person; wants a lava-aware AI |
