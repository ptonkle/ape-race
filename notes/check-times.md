# Check times

One row per run of `scripts/timed-checks.sh`, newest last. Seconds per step; `load` is the 1-minute load average on this machine (16 cores) when the run started.

| When | Version | Commit | Load | Check | Steps (s) | Total (s) | Result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 2026-10-14 23:31 | 1.10.1 | f28d992 | 15.83 | check | format=0.6 lint=2.9 core=87.6 release=0.3 WebAssembly=0.1 browser=409.3 | 500.9 | pass |
| 2026-10-14 23:40 | 1.10.1 | f28d992 | 38.62 | online | release=9.2 transport=22.9 a=197.5 | 229.6 | pass |
