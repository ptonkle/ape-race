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
| 2026-10-15 17:38 | 1.12.0 | 8fe929e+ | 0.13 | check | format=0.8 lint=1.9 core=65.7 release=5.4 WebAssembly=0.1 browser=252.3 | 326.2 | pass |
| 2026-10-15 17:44 | 1.12.0 | 8fe929e+ | 25.08 | online | release=0.2 transport=18.8 a=118.1 | 137.1 | pass |
| 2026-10-15 18:14 | 1.12.0 | 3065415 | 20.57 | check | (stopped early) | - | FAIL (exit 101) |
| 2026-10-15 18:15 | 1.12.0 | 3065415 | 19.58 | online | release=9.0 transport=21.0 a=125.2 | 155.2 | pass |
| 2026-10-15 18:18 | 1.12.0 | 3ebdf65 | 20.92 | check | format=1.0 lint=2.8 core=90.6 release=8.4 WebAssembly=0.1 browser=302.4 | 405.3 | pass |
| 2026-10-15 18:25 | 1.12.0 | 3ebdf65 | 14.27 | online | release=0.3 transport=16.3 a=76.1 | 92.7 | pass |
| 2026-10-15 18:27 | 1.12.0 | 06736bb | 9.78 | check | (stopped early) | - | FAIL (exit 1) |
| 2026-10-15 18:32 | 1.12.0 | 06736bb | 11.20 | online | release=0.2 transport=16.3 a=73.9 | 90.4 | pass |
| 2026-10-15 18:37 | 1.12.0 | 2b0e996 | 13.88 | check | (stopped early) | - | FAIL (exit 1) |
| 2026-10-15 18:41 | 1.12.0 | 2b0e996 | 13.14 | online | release=0.2 transport=16.7 a=77.4 | 94.4 | pass |
| 2026-10-15 18:48 | 1.12.0 | 318a8e4 | 8.65 | check | format=0.5 lint=1.5 core=60.5 release=0.2 WebAssembly=0.1 browser=220.3 | 283.2 | pass |
| 2026-10-15 18:53 | 1.12.0 | 318a8e4 | 12.07 | online | release=0.2 transport=16.8 a=74.9 | 91.9 | pass |
| 2026-10-15 19:15 | 1.12.2 | 6198266 | 8.73 | check | format=0.5 lint=4.8 core=67.8 release=0.2 WebAssembly=0.1 browser=240.5 | 313.9 | pass |
| 2026-10-15 19:20 | 1.12.2 | 6198266 | 11.39 | online | release=0.3 transport=17.0 a=82.7 | 100.0 | pass |
