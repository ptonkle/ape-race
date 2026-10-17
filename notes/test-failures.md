# Test failures

One row per failing run of `scripts/timed-checks.sh`, newest last: the tests and browser steps that failed (from `scripts/tags.py failures`). Passing runs add nothing; notes/check-times.md has every run.

| When | Version | Commit | Check | Failing |
| --- | --- | --- | --- | --- |
| 2026-10-16 00:48 | 1.12.8 | 5d6b12c | online (chromium+firefox+webkit) | browser[firefox]: B's lobby panels have no sideways scroll [['panel setup', 407, 398]]; browser[webkit]: B's lobby panels have no sideways scroll [['panel setup', 407, 398]] |
| 2026-10-16 10:18 | 1.12.8 | 135c0d1 | check (chromium+firefox+webkit) | cargo:app::notes_tests::minor_patches_hidden_until_ticked_and_remembered; cargo:patch_notes::tests::newest_entry_is_this_version |
| 2026-10-16 19:19 | 1.13.3 | 9bbf96f | quick (chromium; tags all, boundary, camera, costumes, items, menus, notes, online, page, race, replay, sound, weather) | browser: ModuleNotFoundError: No module named 'playwright' |
| 2026-10-17 01:17 | 1.14.0 | 9192d6e | check (chromium+firefox+webkit) | cargo:tower::audit::every_vine_is_clear_of_the_stone |
