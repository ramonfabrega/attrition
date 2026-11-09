# The guard of guards — 2026-11-09

Loop rows 4040, 4041 and 4044 (the seventy-sixth pass) have one shape: an instrument that reads nothing and
stays green. This file is the inventory 4041 asked for: every pinned guard that reads a set of source files or
documents by path, whether it could go blind before this landing, and what it asserts now. Built by Opus 5.5.

A guard **could go blind** when an empty or moved set makes it pass. A missing directory usually panics
(`read_dir(..).expect`), so the case that matters is a set that is still there but empty, or that is still read
but is no longer where the rows are. A guard already **holds** when it asserts a floor or a count, when a pinned
table's rows would report "gone" or "mended", or when a single file it needs panics when missing.

The floors sit about a tenth under the day's count, and the count is written beside each one. Journals and pass
records are never deleted, so those floors only fall further behind. The sweep and production counts can move
with structure work, and the margin is there for that.

## Red first

- **4040.** Before the fix, `tools/guard.sh the_scanner_follows_the_harness_into_a_directory` (a `rondata` test)
  printed `test result: ok. 0 passed; ... 2286 filtered out` for `sim` and exited 0. After the fix the same filter
  runs in `rondata` and exits 0 on the one test it ran. `no_such_test_anywhere_4040` exits 1 with
  `guard.sh: the filter(s) matched no test in sim or rondata: no_such_test_anywhere_4040`.
  `test_release_gate.SharedEntryPoints.test_a_guard_filter_runs_on_both_crates_and_one_that_matches_nothing_is_named`
  failed first on `'-p rondata' not found in 'test -q -p sim rondata_only'`.
- **4041.** With `docs/journal/` emptied, `docs_guard::a_journal_carries_no_tool_call_markup` and
  `a_journal_quotes_its_lane_verdict_and_gate_minutes` both passed (`2 passed`). After the fix all three journal
  guards fail with `docs/journal/*.md: read 0, under the floor of 1250 — the set moved, was renamed or emptied, and
  this guard is checking nothing (Loop 4041)`. `test_queueledger.test_a_ledger_that_reads_no_queue_fails` failed
  first on `0 != 1 : queue ledger: 0 numbers ever booked, 0 live, 0 ledgered`.
- **4044.** With `crates/sim/tests/sweep/` hidden (the `sim` lib build does not compile it), the hex-constant
  scan, the spawn-site walk and the real-tree classification all fail with `the sweep's module tree holds 150
  files under src/ (floor 120) and 0 under tests/ (floor 1000)`. `no_float::a_module_s_files_are_read_from_the_tree_in_either_layout`
  checks that one `sweep` module gives the same set, its own file included, whether it is laid out as
  `sweep/mod.rs` or as `sweep.rs` beside `sweep/`.

## `crates/sim/src/docs_guard.rs`

| guard | set it reads | could go blind before | asserts now |
|---|---|---|---|
| `a_capture_booked_on_a_map_cites_the_window_the_disk_holds` | `tools/gamelog/captures.txt` | no: `windows.len() >= 40` | unchanged |
| `an_item_number_is_minted_once_and_in_its_file_s_form` | `docs/journal/*-item-*.md`, the landed items | **yes**: no journals meant nothing landed and nothing failed | `JOURNALS_AT_LEAST` 1250 (1,367) |
| `no_runs_section_heading_stands_twice`, `every_runs_section_closes_its_fences` | `docs/RUNS.md`, one file | only if the file's form changed; a missing file panics | unchanged (one file, not a set) |
| `claude_md_carries_no_findings` | `CLAUDE.md`, one file | no: a missing file panics | unchanged |
| `no_conflict_marker_survives_in_the_tree` | text files under `docs/`, `tools/`, `crates/` and the root | **yes** | `TREE_TEXT_FILES_AT_LEAST` 4000 (4,938) |
| `every_cited_address_names_its_function` | `docs/*.md` and their `name@address` citations | **yes** (when the export is present) | 40 documents (48), 6000 citations (6,735) |
| `a_section_the_code_cites_is_one_the_document_holds` | `docs/*.md` and `crates/**/*.rs` | partly: `NO_SUCH_SECTION` rows go "mended" | 40 documents, `CRATE_SOURCES_AT_LEAST` 1800 (2,072) |
| `a_dead_listed_address_is_cited_only_where_pinned` | `docs/*.md` | no: `DEAD_CITED` rows go "gone" | plus 40 documents |
| `a_constant_a_document_names_is_built_or_pinned` | `docs/*.md`, `crates/**/*.rs` less the sweep | no: `UNBUILT` rows go "gone"; an empty crate set fails the pin | its sweep skip now comes from the module tree, with the size asserted (4044, below) |
| `a_section_over_the_ceiling_may_not_grow` | `docs/*.md` | no: `OVER` rows go "renamed or removed" | plus 40 documents |
| `every_shell_script_under_tools_is_executable` | `tools/**/*.sh` with a shebang | **yes** | `TOOL_SCRIPTS_AT_LEAST` 35 (44) |
| `a_journal_carries_no_tool_call_markup` | `docs/journal/*.md` | **yes** (made red first) | 1250 (1,391) |
| `a_journal_quotes_its_lane_verdict_and_gate_minutes` | journals from item 3265 | **yes** (made red first) | 500 (575) |
| `a_journal_s_loop_line_is_filed_or_says_none` | `docs/audit/*fable-pass*.md`, journals dated from 2026-10-18 | **yes** | `PASS_RECORDS_AT_LEAST` 60 (74), 550 journals (625) |
| `a_loop_line_under_its_own_heading_is_read_off_the_next_line` | two named journals | no: missing files panic | unchanged |
| `every_cited_sim_path_names_a_file` (runs `tools/paths.py --check`) | `docs/**/*.md`, `crates/**/*.rs` | no: `CITATIONS_NAMING_NOTHING` rows go "mended" | unchanged |
| `a_seam_that_cites_the_disk_names_its_scan` | `crates/**/*.rs` | no: `UNSCANNED_SEAMS` rows go "gone" | plus 1800 sources |
| the queue, handoff and park-pool tests | `docs/QUEUE.md`, `docs/PARKED.md` | no: a missing file panics; several have their own non-empty asserts | unchanged |
| `every_decision_has_an_index_row` | `docs/DECISIONS.md` | no: `!entries.is_empty()`, `!rows.is_empty()` | unchanged |
| `a_track_s_stub_pool_holds_two_unassigned_sections` | `docs/AI.md`, `docs/GOLDEN.md` | no: `free.len() >= 2` | unchanged |

## `crates/sim/src/no_float.rs` and the other `sim` readers

| guard | set it reads | could go blind before | asserts now |
|---|---|---|---|
| `the_simulation_holds_no_floating_point` | production files from `source_files()`, a recursive walk classified by `module_tree` | partly: the floor was 15 against 243 | `SCANNED_AT_LEAST` 215 (243) |
| `the_scan_reaches_a_nested_module`, `every_source_file_is_mounted` | the walk, the module trees | no | unchanged |
| `the_real_tree_classifies_its_test_files` | sweep files by the `sweep/` prefix | **yes**: `filter(..).all(..)` is true on nothing | `sweep_files()`: at least 120 in `src/`, its own module file among them, all test |
| `economy::rares::tests::a_unit_s_speed_is_never_set_from_the_raw_moves` | production files less `sweep/` and `sweep.rs`, by name | **yes** | reads `source_files()` less `sweep_files()`, floor 220 (248) |

## `crates/rondata/src/` (inventory by an Explore reader on Opus 5.5, spot-checked)

| guard | set it reads | could go blind before | asserts now |
|---|---|---|---|
| `writers::sim_files` (`compared_fields_have_writers`, `a_field_the_sim_writes_is_read_by_something`) | `crates/sim/src` less `sweep/`, `sweep.rs`, `test_support/` | the skip, yes; the read set is held by the `NO_READER` and exemption rows | `MIN_SWEEP_SKIPPED` 120 (150), `MIN_SIM_FILES` 320 (363) |
| `writers::diff_sources_under` | `src/diff.rs`, `src/diff/**` | no: `!out.is_empty()`, `MIN_ROWS`, `MIN_FIELDS` | unchanged |
| `diff::floors::a_golden_widening_s_window_reaches_its_trace_s_end` | `golden.rs`, `golden/*.rs` | it did, until Loop 4029 | `unread.is_empty()` (4029) |
| `ledger::every_differ_module_is_on_the_ledger` | `src/diff/**` | no: pinned `DIFF_FILES`, `gone` | unchanged |
| `testenv::source::{family,golden,harness}` | the registered `cases_*` modules | no: `expect`, `!cases.is_empty()` | unchanged |
| `golden::every_chapter_stages_what_it_says_it_stages`, `no_chapter_script_writes_a_frame_below_its_predecessor` | `tools/gamelog/golden/*.cmd` | no: `== CHAPTER_DEBT.len()`, `read >= 38` | unchanged |
| `trace::every_site_label_the_simulation_marks_has_a_row` | `crates/sim/src/**` | no: `labels.len() > 50` | unchanged |
| `blind::the_blind_list_is_the_pinned_residue` | `docs/**/*.md` | no: pinned rows go "no longer cited" | unchanged |
| `diff::slots`, `diff::floors` (AI words, widenings, guard pins) | `src/diff/*.rs` | no: pinned tables, `found == named` | unchanged |
| `diff::harness` deferred labels and widened bits | `include_str!("harness.rs")` | no: `> 50`, `>= 10` | unchanged |
| `diff::coverage::a_unit_key_the_simulation_carries_is_read_or_owed` | `sim/src/units/unit.rs` | no: `fields.len() > 20` | unchanged |
| **`diff::oracle::a_leader_compare_walks_the_lobby_s_seats_never_a_literal`** | a fixed `include_str!` list: `harness.rs`, `golden.rs`, … | **yes, and blind today**: the harness and golden bodies moved to `harness/tests/cases_*.rs` and `golden/cases_*.rs` before this guard was written, and they hold nine literal seat loops (e.g. `harness/tests/cases_02.rs:1628`) | **not done**, see below |

## The Python guards the gate runs

| guard | set it reads | could go blind before | asserts now |
|---|---|---|---|
| `tools/queueledger.py` (`guard.sh`) | `docs/QUEUE.md`, `docs/PARKED.md`, each read only if it existed | **yes**: no queue gave "0 numbers ever booked", exit 0 | a missing `docs/QUEUE.md` fails, naming Loop 4041 (the parked file may be absent: fixtures hold the queue alone) |
| `tools/nations_census.py --check` | `crates/sim/src/**`, `crates/sim/tests/sweep/**` | no for the source set (a `KeyError` on `tuning.rs`, drift against `NATIONS.md`); yes for the sweep skip | `SWEEP_ONLY_AT_LEAST` 1200 (1,459) |
| `tools/shape.py` via `diff::floors::the_pinned_shape_is_the_tree_s` | `crates/sim/src`, `crates/rondata/src` | no: an exact `STRUCTURE` pin; `sweep-lines` moves if the sweep skip breaks | unchanged; the booking's empty-directory fix to `flat` (`p.with_suffix('').rglob('*.rs')`) is in place, and was checked, not redone |
| `tools/seams.py --views` (a reporting tool) | `crates/sim/src` less `sweep` | yes | on the live tree, at least 120 sweep files skipped (`SWEEP_SKIPPED_AT_LEAST`) |
| `test_seams` live-tree tests | `seams.SIM` | no: `DOORS = 12`, `> 150` | unchanged |
| `test_offline_tests.test_every_explore_suite_is_run_or_named` | `tools/explore/test_*.py` | no: `NOT_RUN` rows | unchanged |
| `test_brief.test_the_live_queue_composes` | open items in `docs/QUEUE.md` | yes: zero items, zero iterations | **not done**: a drained queue can hold no open item, so a floor would be red at a drain |
| `test_census` `hooked_in_sweeps(...) == []` and `census.py --hooked` | sweep sources plus `tools/emu` | yes: the pin is zero | **not done** |

## 4044: the readers that knew the sweep by its directory's name

Item 4032 listed seven: `shape.py`'s `in_sweep`, `seams.py`, `nations_census.py`'s `sweep_only`, `docs_guard`'s
hex-constant scan, `economy::rares`'s spawn-site walk, `rondata::writers`'s `sim_files`, and `sweep_gen.py`'s
registration. The `sim` readers now ask the module tree. `no_float::module_files(under, root, name)` returns the
files of the module `name` that a crate root declares, whatever its layout. `no_float::sweep_files()` is `lib.rs`'s
`sweep` module plus each integration target's `sweep` module under `tests/`, and it asserts both halves' sizes once
for every reader. The hex scan leaves the same files out as before: its `UNBUILT` pin did not move. The other
crate and the Python readers keep their name-keyed skip, which is the cheapest truth they have, and each now
asserts the size of what it skips. `sweep_gen.py` writes registrations rather than reading a set, so it was left
alone. `shape.py` is held by its exact pin.

## Not done

- **`diff::oracle::a_leader_compare_walks_the_lobby_s_seats_never_a_literal` is blind to the moved bodies, and
  fixing the reader turns it red.** Reading `testenv::source::{harness,golden}()` (the expanded sources
  `floors.rs` already uses) would find nine `for who in 0..2usize` seat loops over `flog.leader_block` in
  `harness/tests/cases_02.rs`, `cases_03.rs`, `cases_04.rs` and `golden/cases_01_09.rs`. Whether those loops are
  wrong (a two-seat lobby) or exempt is a ruling on the leader compare, not on the guard. The next item should
  rule on it and then point the reader at the expanded sources.
- `test_brief.test_the_live_queue_composes` and the `hooked_in_sweeps` pin of zero: both pass on an empty set,
  and in both a floor would conflict with a legitimate empty state. Reading the set's source count instead of its
  matches would fix them; not built here.
- The `SUITES` members the reader did not open: `test_release_gate`, `test_sweep_gen`, `test_repin`,
  `test_runwiring`, `test_retire`, `test_archive_census`, `test_rubric`, `test_gates`, `test_emu_env`,
  `test_filerun`, `test_runtime_policy`, `test_waitrun`, `test_lane_lock`, `test_capture_lanes`. Most of them run
  on authored temporary trees, which cannot go blind in this way. That is unchecked.
- Crates other than `sim` and `rondata` (`render`, `client`, `fixed`).
