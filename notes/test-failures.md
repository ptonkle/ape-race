# Test failures

One row per failing run of `scripts/timed-checks.sh`, newest last: the tests and browser steps that failed (from `scripts/tags.py failures`). Passing runs add nothing; notes/check-times.md has every run.

| When | Version | Commit | Check | Failing |
| --- | --- | --- | --- | --- |
| 2026-10-16 00:48 | 1.12.8 | 5d6b12c | online (chromium+firefox+webkit) | browser[firefox]: B's lobby panels have no sideways scroll [['panel setup', 407, 398]]; browser[webkit]: B's lobby panels have no sideways scroll [['panel setup', 407, 398]] |
| 2026-10-16 10:18 | 1.12.8 | 135c0d1 | check (chromium+firefox+webkit) | cargo:app::notes_tests::minor_patches_hidden_until_ticked_and_remembered; cargo:patch_notes::tests::newest_entry_is_this_version |
| 2026-10-16 19:19 | 1.13.3 | 9bbf96f | quick (chromium; tags all, boundary, camera, costumes, items, menus, notes, online, page, race, replay, sound, weather) | browser: ModuleNotFoundError: No module named 'playwright' |
| 2026-10-17 01:17 | 1.14.0 | 9192d6e | check (chromium+firefox+webkit) | cargo:tower::audit::every_vine_is_clear_of_the_stone |
| 2026-10-17 09:17 | 1.14.0 | 8b79781 | check (chromium+firefox+webkit) | cargo:tower::audit::every_vine_is_clear_of_the_stone |
| 2026-10-17 09:57 | 1.14.2 | 4b4b3cc | quick (chromium; tags ai, all, boundary, camera, costumes, items, maps-tower, menus, notes, online, page, physics, race, render, replay, sound, weather) | cargo:tower::audit::every_vine_is_clear_of_the_stone |
| 2026-10-17 10:11 | 1.14.2 | fc084f7 | quick (chromium; tags boundary, camera, costumes, items, menus, notes, online, page, race, replay, sound, weather) | cargo:tower::audit::every_vine_is_clear_of_the_stone |
| 2026-10-17 10:11 | 1.14.2 | 69eca4c | quick (chromium; tags boundary, camera, costumes, items, menus, notes, online, page, race, replay, sound, weather) | cargo:every_difficulty_climbs_falls_and_finishes; cargo:the_tower_medals_follow_its_time_trial; cargo:every_level_flies_through_the_tear; cargo:tower_difficulties_are_ordered_and_spaced |
| 2026-10-17 10:14 | 1.14.2 | c30a124 | quick (chromium; tags boundary, camera, costumes, items, menus, notes, online, page, race, replay, sound, weather) | browser [chromium]/weather: playwright._impl._errors.TimeoutError: Page.wait_for_selector: Timeout 5000ms exceeded. |
| 2026-10-17 11:12 | 1.14.3 | 553486f | quick (chromium; tags ai, boundary, camera, costumes, dock, gear, hud, hype, items, lava, maps-tower, medals, menus, models, notes, online, race, render, replay, results, scenery, settings, setup, sky, sound, trial, vines, weather) | cargo:tower_difficulties_are_ordered_and_spaced |
| 2026-10-17 15:17 | 1.15.0 | 3ab9c3c | quick (chromium+firefox+webkit; tags all, boundary, notes, online, page) | browser [chromium]/race: playwright._impl._errors.TimeoutError: Page.wait_for_selector: Timeout 5000ms exceeded. |
| 2026-10-18 21:06 | 1.17.0 | 01064ba+ | quick (chromium; tags boundary, camera, items, menus, notes, online, race, replay) | cargo:app::settings_tests::settings_round_trip_through_storage |
