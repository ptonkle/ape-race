# Builder specification

Builder is a reusable technical setting for small browser tools built with coding agents: application rules live in a fast-tested Rust core, a thin WebAssembly shim only moves values across the browser boundary, and a static page renders the result. The separation gives the agent fast feedback while it builds and leaves a finished artifact that can be checked at the core, the WebAssembly boundary and the browser page.

This file has three parts. **Builder requirements** define Builder; a project that drops one is no longer a Builder project, and the change should be stated as such. **Reference defaults** are what this template chose so that a copy works at once; change them by recording the change in `SECOND-ORDER.md` and in the table at the end of this file. **Project decisions** belong to the person building the tool, and they go in `BRIEF.md` or `PLAN.md`.

## Builder requirements

| # | Requirement | Enforced by |
| --- | --- | --- |
| R1 | The core (`crates/core`) holds every application rule and state transition. | review; `audit-boundary` prompt |
| R2 | The core performs no browser, filesystem, network, clock, random-device or UI input or output. `cargo test` reaches all of it without a browser. | `core_source_has_no_io_and_no_nondeterminism` in `crates/core/tests/boundary.rs` |
| R3 | The core's dependencies are on an explicit allowlist. | `core_dependencies_are_on_the_allowlist` |
| R4 | Public core functions take ordinary values and return ordinary values or serialisable structures. The template's convention is strings in, `{"ok": ...}` or `{"error": "..."}` out, in `crates/core/src/api.rs`. | review |
| R5 | The shim (`crates/shim`) uses `wasm-bindgen` and only translates values: it converts arguments, calls the core, and converts the result back. | `shim_decides_nothing` (no branch, loop or `?` in shim source) |
| R6 | The page (`web/`) is static HTML, CSS and JavaScript. It draws what the core returns and does not recompute an application rule. | `page_keeps_no_rule_and_names_no_other_origin`; review |
| R7 | The default build deploys as static files and makes no off-origin request on its default load-and-idle path. | `tests/browser/check.py` in three engines |
| R8 | `./scripts/check.sh` runs the complete pre-deploy check, and CI runs the same file. | `.github/workflows/checks.yml` in the reference repository |
| R9 | Each build follows the nine-stage procedure, and the agent does not deploy on its own judgement. | `CLAUDE.md`, `AGENTS.md`, `HANDOFF.md` |

**When a requirement seems to block necessary work, stop and say so.** Write a worklist row in `SECOND-ORDER.md` naming the requirement, what the feature needs, and the options you can see. A rule in the shim or the page is a second rulebook that the fast suite cannot see, so the answer is almost always to move the rule into the core.

## Reference defaults

| # | Default | Where | Why this default |
| --- | --- | --- | --- |
| D1 | Core allowlist: `serde`, `serde_json`. | `ALLOWED_DEPENDENCIES` in `boundary.rs` | enough for JSON in and out |
| D2 | No floats, hash maps, clocks or randomness in the core. | `NONDETERMINISM_IDENTIFIERS` in `boundary.rs` | the same input gives the same output on every machine, so tests stay stable |
| D3 | Page JavaScript holds no numeric literal other than 0 and 1, and no `Math`, `parseInt` or rounding. | `PAGE_FORBIDDEN_IDENTIFIERS` in `boundary.rs` | a number in the page is usually a copy of a rule |
| D4 | Every HTML page carries a Content-Security-Policy meta element with `default-src 'self'`, `connect-src 'self'`, `'wasm-unsafe-eval'` and `form-action 'none'`. | `every_page_sets_the_content_security_policy` | a second, browser-enforced limit on where the page can connect; it cannot stop navigation, so the browser check still watches requests |
| D5 | Reader-visible text lives in `data/`, and the page fetches it from its own origin. | convention | content changes without touching rules |
| D6 | `wasm-bindgen` pinned to `=0.2.127`; the build refuses a different CLI version. | `crates/shim/Cargo.toml`, `scripts/build.sh` | the generated glue must match the crate |
| D7 | Browser check: Playwright 1.63.0 in Chromium, Firefox and WebKit, 10 s idle, then `tests/browser/interactions.py`. | `tests/browser/check.py` | the three engines classroom machines run |
| D8 | Mutation testing with `cargo-mutants`, as an optional audit outside `check.sh`. | `scripts/mutants.sh` | it takes minutes; the everyday check must take seconds |
| D9 | `cargo fmt --check` and `cargo clippy -D warnings`. | `scripts/check.sh` | |

## Project decisions

These stay with the person and go in `BRIEF.md` or `PLAN.md`:

- what the tool does, for whom, and what "done" means;
- the content and its source, and who checks it;
- any data a person enters, where it stays, and whether the page keeps it between visits (browser storage is allowed by the requirements and is a project decision);
- any crate added to the allowlist;
- any feature that needs a server, a live model, an account, or a request to another origin. These fall outside the default build (R7). Say so in the brief and give the feature its own checks and approvals.

## Changes to the defaults in this project

| Default | Changed to | Reason | Recorded in |
| --- | --- | --- | --- |
| | | | |
