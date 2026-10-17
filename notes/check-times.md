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
| 2026-10-15 20:17 | 1.12.3 | baea429 | 48.34 | check | format=1.4 lint=0.2 core=84.5 release=10.3 WebAssembly=0.1 browser=447.0 | 543.5 | pass |
| 2026-10-15 20:41 | 1.12.4 | 0df1c7d | 33.14 | check | format=1.1 lint=0.2 core=100.0 release=13.6 WebAssembly=0.1 browser=498.5 | 613.7 | pass |
| 2026-10-15 21:02 | 1.12.5 | 4164986 | 29.91 | quick (chromium; tags all, boundary, camera, costumes, items, maps-bay, menus, notes, online, page, race, render, replay, sound, weather) | format=1.1 lint=4.4 list=0.1 core=86.5 release=16.8 WebAssembly=0.2 browser=344.2 online-transport=7.0 online-race=88.6 | 548.9 | pass |
| 2026-10-15 21:13 | 1.12.6 | 64f5e9b | 26.06 | quick (chromium; tags ai, all, boundary, maps-bay, notes, page, race, render) | format=0.9 lint=3.3 list=0.1 core=60.3 release=5.9 WebAssembly=0.1 browser=171.7 online-transport=5.1 online-race=49.2 | 296.4 | pass |
| 2026-10-15 21:19 | 1.12.7 | 155614c | 10.36 | quick (chromium; tags all, boundary, camera, costumes, items, menus, notes, online, page, race, render, replay, sound, weather) | format=0.5 lint=3.9 list=0.1 core=47.4 release=5.9 WebAssembly=0.1 browser=211.3 online-transport=5.0 online-race=66.4 | 340.6 | pass |
| 2026-10-15 21:49 | 1.12.8 | dd7320c | 12.45 | quick (chromium; tags boundary, notes, online, page) | format=0.5 lint=2.1 list=0.1 core=0.6 release=6.4 WebAssembly=0.1 browser=245.7 online-transport=5.1 online-race=87.1 | 347.6 | pass |
| 2026-10-16 00:42 | 1.12.8 | 5d6b12c | 10.78 | check (chromium+firefox+webkit) | format=0.6 lint=0.1 core=55.7 test=0.5 release=0.2 WebAssembly=0.1 browser=275.2 | 332.4 | pass |
| 2026-10-16 00:48 | 1.12.8 | 5d6b12c | 8.43 | online (chromium+firefox+webkit) | (stopped early) | - | FAIL (exit 1) |
| 2026-10-16 10:18 | 1.12.8 | 135c0d1 | 0.09 | check (chromium+firefox+webkit) | (stopped early) | - | FAIL (exit 101) |
| 2026-10-16 10:19 | 1.12.8 | 135c0d1 | 4.28 | online (chromium+firefox+webkit) | release=6.2 transport=17.8 a=127.4 | 151.5 | pass |
| 2026-10-16 10:22 | 1.13.0 | 4c75df8 | 7.71 | check (chromium+firefox+webkit) | format=0.6 lint=2.2 core=75.2 test=0.5 release=6.9 WebAssembly=0.1 browser=432.6 | 518.1 | pass |
| 2026-10-16 10:33 | 1.13.1 | 111be45 | 4.48 | quick (chromium; tags boundary, maps-tower, notes) | format=0.6 lint=2.1 list=7.7 core=15.8 release=6.2 WebAssembly=0.1 browser=61.7 | 94.2 | pass |
| 2026-10-16 10:40 | 1.13.2 | 06fd9d5 | 6.64 | quick (chromium; tags boundary, costumes, maps-tower, notes, render) | format=0.6 lint=2.1 list=11.2 core=17.2 release=0.3 WebAssembly=0.1 browser=181.9 | 213.4 | pass |
| 2026-10-16 19:19 | 1.13.3 | 9bbf96f | 7.95 | quick (chromium; tags all, boundary, camera, costumes, items, menus, notes, online, page, race, replay, sound, weather) | format=0.6 lint=10.3 list=34.1 core=66.1 release=11.1 WebAssembly=0.1 browser=0.1 | - | FAIL (exit 1) |
| 2026-10-16 19:21 | 1.13.3 | 9bbf96f | 13.55 | quick (chromium; tags all, boundary, camera, costumes, items, menus, notes, online, page, race, replay, sound, weather) | format=0.6 lint=0.1 list=0.1 core=69.8 release=0.3 WebAssembly=0.1 browser=232.1 online-transport=7.1 online-race=84.5 | 394.7 | pass |
| 2026-10-16 19:45 | 1.13.4 | 56c23e1 | 7.23 | quick (chromium; tags boundary, notes, sound) | format=1.0 lint=3.7 list=19.6 core=0.4 release=6.6 WebAssembly=0.1 browser=49.6 | 81.0 | pass |
| 2026-10-17 01:17 | 1.14.0 | 9192d6e | 1.70 | check (chromium+firefox+webkit) | (stopped early) | - | FAIL (exit 101) |
| 2026-10-17 01:17 | 1.14.0 | 9192d6e | 7.68 | online (chromium+firefox+webkit) | release=6.1 transport=16.6 a=106.6 | 129.3 | pass |
| 2026-10-17 09:17 | 1.14.0 | 8b79781 | 3.08 | check (chromium+firefox+webkit) | (stopped early) | - | FAIL (exit 101) |
| 2026-10-17 09:18 | 1.14.0 | 8b79781 | 7.97 | online (chromium+firefox+webkit) | release=6.7 transport=17.6 a=128.0 | 152.3 | pass |
| 2026-10-17 09:57 | 1.14.2 | 4b4b3cc | 11.59 | quick (chromium; tags ai, all, boundary, camera, costumes, items, maps-tower, menus, notes, online, page, physics, race, render, replay, sound, weather) | format=2.4 lint=11.3 list=29.8 core=4.9 | - | FAIL (exit 101) |
| 2026-10-17 10:11 | 1.14.2 | fc084f7 | 12.03 | quick (chromium; tags boundary, camera, costumes, items, menus, notes, online, page, race, replay, sound, weather) | format=0.6 lint=0.1 list=0.1 core=4.6 | - | FAIL (exit 101) |
| 2026-10-17 10:11 | 1.14.2 | 69eca4c | 9.63 | quick (chromium; tags boundary, camera, costumes, items, menus, notes, online, page, race, replay, sound, weather) | format=1.1 lint=0.1 list=0.1 core=67.9 | - | FAIL (exit 101) |
