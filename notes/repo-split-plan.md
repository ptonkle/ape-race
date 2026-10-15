# Plan: one public repo per game (queued 2026-10-14)

The person asked for each game to be its own public GitHub repo, set up like sgilson7's (e.g. github.com/sgilson7/ribbit-king). **Wait until the next deploy has gone through** (the person: "this seems like it will be disruptive"), i.e. after the six-agent batch of 2026-10-14 (T60–T83) is merged and live.

## How sgilson7's repos are set up
- One public repo per game, the full source (crates, web, scripts, tests, BRIEF/PLAN/HANDOFF/SECOND-ORDER, AGENTS.md, CLAUDE.md).
- MIT `LICENSE` plus `LICENSES.md` (where every asset comes from; vendored Trystero credited).
- A README that opens with what the game is and a **Play:** link, then features and a controls table.
- Repo description set, homepage = the game's Pages URL.
- Pages served by the repo itself via `.github/workflows/deploy.yml`: on push to main it runs `BUILDER_ENGINES=chromium ./scripts/check.sh` (Rust stable + wasm32 target, rust-cache, wasm-bindgen pinned via taiki-e/install-action, Python 3.12 + Playwright Chromium), uploads `dist/` and deploys. So a push to main is the deploy.
- The user site repo (`sgilson7.github.io`) only holds the home page.

## Steps for ptonkle
1. `gh repo create ptonkle/ape-race --public --source . --push` (main only; the agent branches are merged and deleted by then). Description and homepage https://ptonkle.github.io/ape-race/.
2. Add LICENSE (MIT: the person confirmed 2026-10-14, for both games), LICENSES.md, a README Play link, `.github/workflows/deploy.yml` adapted from ribbit-king (check our wasm-bindgen version in Cargo.lock; nettest.html/nettest.js are removed from the deploy as `pages.sh` does).
3. Turn on Pages with build type "workflow" (`gh api -X POST repos/ptonkle/ape-race/pages -f build_type=workflow`).
4. Once the project site serves /ape-race/, remove `ape-race/` from `ptonkle.github.io` (keep `img/ape-race/` for the home page card; `pages.sh` then only refreshes the card pictures). The URL stays the same, so room links and saved settings (same origin) keep working.
5. Mandible: its source is in `~/slop/game1` (no remote), the built game in `ptonkle.github.io/mandible/`. Same steps as `ptonkle/mandible`; also rename the folder to `~/slop/mandible` (the slop CLAUDE.md naming rule).
6. Update the deploy instructions in HANDOFF (both games) and the memory notes: deploying is now "push main, watch the Actions run".
