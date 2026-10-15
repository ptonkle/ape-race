# Check times

One row per run of `scripts/timed-checks.sh`, newest last. Seconds per step; `load` is the 1-minute load average on this machine (16 cores) when the run started.

| When | Version | Commit | Load | Check | Steps (s) | Total (s) | Result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 2026-10-14 23:31 | 1.10.1 | f28d992 | 15.83 | check | format=0.6 lint=2.9 core=87.6 release=0.3 WebAssembly=0.1 browser=409.3 | 500.9 | pass |
| 2026-10-14 23:40 | 1.10.1 | f28d992 | 38.62 | online | release=9.2 transport=22.9 a=197.5 | 229.6 | pass |
| 2026-10-15 00:23 | 1.11.3 | c4317fd | 24.58 | check | format=0.8 lint=0.1 core=73.4 release=0.4 WebAssembly=0.1 browser=255.8 | 330.6 | pass |
| 2026-10-15 00:28 | 1.11.3 | c4317fd | 28.00 | online | release=0.2 transport=22.6 a=89.5 | 112.3 | pass |
| 2026-10-15 09:13 | 1.11.4 | a71701c | 13.08 | check | (stopped early) | - | FAIL (exit 1) |
| 2026-10-15 09:18 | 1.11.4 | a71701c | 13.78 | online | release=0.2 transport=17.1 a=80.3 | 97.6 | pass |
| 2026-10-15 09:20 | 1.11.4 | 2121281 | 10.01 | check | format=0.5 lint=0.1 core=61.3 release=0.2 WebAssembly=0.1 browser=327.2 | 389.4 | pass |
| 2026-10-15 10:05 | 1.11.4 | c73ac85+ | 10.92 | check | (stopped early) | - | FAIL (exit 1) |
| 2026-10-15 10:06 | 1.11.4 | c73ac85+ | 10.86 | check | format=0.5 lint=0.1 core=58.6 release=0.2 WebAssembly=0.1 browser=219.9 | 279.4 | pass |
| 2026-10-15 10:11 | 1.11.4 | 7bb3409 | 4.82 | check | format=0.5 lint=2.2 core=73.2 release=5.3 WebAssembly=0.1 browser=226.4 | 307.6 | pass |
| 2026-10-15 10:17 | 1.11.4 | 7bb3409 | 11.96 | online | release=0.3 transport=16.8 a=76.9 | 94.0 | pass |
