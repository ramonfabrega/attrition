# Replay oracle — playback preflight, 2026-10-15

Branch `codex/astra-replay-oracle`, cut from main `c8eb478d`, one sequential
Codex agent under AGENTS.md. No commander, queue, existing floors or other
checkout changed. The brief's first deliverable is the current checkpoint;
the ten-game table and sim-driven replay remain subsequent work.

## Evidence and implementation

`unattended_capture.py --recording` selects the native playback route, checks
the recording's seed/map and source hash, and reads `gamelog_p.txt`. It keeps
solo lifecycle validation unchanged and supplies separate strict playback
validation. Tests first failed on the missing playback API, prefix reader and
filename selection; they now reject solo lifecycle, missing/repeated frames,
faults, nonzero exit and truncated headers. RECGAME §7 records the format
correction and review debt. The sample test pins the absence of sync commands.

Opening evidence: `~/ron-data/lab-captures/2026-10-15-run900-playback-opening/map-7`.
37 consecutive frames, exit 0, success true, settings_restored true (5 files),
map 7, seed 340513920 despite requested ini seed 12345. Source SHA-256
`df7830d7b6c77e1318a1ef5b7725ae2f0b0c8db5957bed85db0479b506f72649`.
The first long run at
`~/ron-data/lab-captures/2026-10-15-run900-playback-full/map-7` reached
trace frame 21044 but did not exit. `wine.log` ends with a DXVK allocation
failure. It was stopped through the runner's SIGTERM handler; its failure
receipt verifies restoration of all five settings files. It also exposed a
recipe error: `end:MISC` alone emits no per-frame value blocks. This attempt
is **not** a successful full oracle. A fresh `run900-playback-full-values`
retry uses the opening's value categories and schedules quit at 21044, rather
than a frame beyond the recording's range. Whether the first attempt's failure
was caused by EOF handling or the graphics allocation is not established. That wider retry reached only frame 284
in 210 seconds, so it was stopped through the same cleanup path (five settings
restored). Full object logging at every frame is not a practical recipe at
that measured rate. The subsequent `run900-playback-full-leaders` capture
uses `end:LEADERS=2` for each frame and the wider object categories at endgame;
its receipt is successful (details below). The narrower coverage must not be
called a whole-world value comparison.

Exploratory attempts are retained under sibling `run900-playback-probe` and
`run900-playback-probe-b` directories. The first refused a filename the native
tokenizer did not accept. The second reached 37 frames and exited 0, but the
solo runner's wrong log path caused an automatic retake, which overwrote that
attempt's trace and failed with exit 40. Its first wine log and playback dump
survive. Do not use that mixed-attempt directory as a paired oracle.

The existing reader's report is `/tmp/astra-recgame-report-20261015.log`:
21,884 packages to exact EOF, frames 0..21044, all 25,779 commands decoded,
all 243,049 combat-table cells equal, zero embedded sync commands. The full
playback therefore cannot establish agreement with original sync checksums.
No sim score or value-diff improvement is claimed; all existing pins stand.

## Reproduction and validation

Launch `tools/explore/golden_capture.sh` through `tools/gamelog/viadriver.sh`,
with a fresh output, `--recording <source> --end-frame 21044
--timeout 3600 --stall-seconds 120 --log-window 0 21045 --callwin 1 0
--detail end:LEADERS=2 --detail misc:CHECKSUM=2
--detail endgame:UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=2`.
Use a fresh TMPDIR for the tracer build; this session's shim is
`/tmp/astra-replay-capture-20261015.sh`. Wait through `waitrun.sh` with process
visibility; sandbox-hidden processes make its liveness verdict unreliable.

Focused validation: playback tests 3, existing lifecycle tests 8, runner
tests 27 pass; the runner's held-parent lock test needs process visibility.
The compiled modal trampoline passes 256 Unicorn cases and rejects its
mutated argument offset. Recording regressions exit 0 under a 20 GiB cap.
The full required-fixture release gate on frozen commit `f4f004b4` exited 0:
`Gate steps: 6 of 6 ran (offline, clippy, fmt, survey, release, guard)`.
Release results: fixed 13, rondata 849, sim 2423, rondata doctests 3; zero
failures. Fixture audit: 3114 observed requests, 417 unique fixtures, zero
missing. This asserts completeness of requested evidence, not every corpus file.
The first failed gate and its correction remain recorded below.

## Corpus and direction

The catalog https://ron.heavengames.com/downloads/lister.php?category=rec
lists 245 games. The 48 archived Playback files are project captures, not
48 external matches. Three 2015 catalog candidates were downloaded and inspected by header:
2139 reports `(Versión: 00.2014.10.0200)`, 2135 reports
`(Versión: 03.02.12.0800)`, and 2133 reports `(Version: 03.02.03.2905)`.
These are distinct from the known 2017 EE sample. Compatibility is not
established; none has been played or scored. Downloads and provenance
remain outside git under `game/external-recgames/astra-20261015`; initial
`.download` responses are HTML, not recordings. No ten-game set is scored.

for the Loop: random-map RCX has setup/rules, not a full initial snapshot;
Ramon selected reconstruction from recording inputs in this chat. Existing
`diff::setup::build_sim` imports terrain/entities from `Initial`; it is not
a random-map generator. Parked 2390 also states the generator is absent.
Do not call a dump-seeded replay the selected self-contained reconstruction.

for the Loop: this external SP recording has zero embedded sync commands;
playback completion cannot certify original-match fidelity.

for the Loop: playback writes `gamelog_p.txt`; watching the solo filename
misdiagnoses a successful run and can destroy its trace by automatic retry.

## Gate registration correction

Implementation commit `7f1097e9`'s first full gate exited 1 at offline tests:
`test_every_explore_suite_is_run_or_named` found `test_replay_capture` absent
from `tools/offline_tests.py`. `Gate steps: 1 of 6 ran (offline); not reached:
clippy, fmt, survey, release, guard`. Log `/tmp/astra-replay-gate-20261015.log`.
Correction: register that suite in `SUITES` (not `NOT_RUN`). No release results
existed to reuse; the full required-fixture gate was rerun on `f4f004b4` and
passed all six steps as recorded above.

## Seed control and durable evidence

The final committed capture code was exercised in
`~/ron-data/lab-captures/2026-10-15-run900-playback-seed-control/map-7`.
Requested ini seed 54321, observed recording seed 340513920; 37 consecutive
frames, exit 0, success and five-file restoration true. Against the opening's
ini seed 12345, `rngcmp.py` finds 37 shared frames and zero differences.
`samegame.py` finds one complete common dump block (frame 1), zero differences,
with no exclusions or dropped fields; both terminal blocks are omitted by
that tool's documented rule. This is native/native reproducibility, not a
sim value score.

Completed reports, comparisons, source-page metadata and capture shims are
preserved outside git at
`~/ron-data/lab-experiments/2026-10-15-astra-replay-oracle/`.
The full successful gate's log is `astra-replay-gate-20261015-b.log` there,
and its audit directory is `gate-f4f004b4/`. No downloaded recording or
proprietary dump is committed.

## Next reconstruction checkpoint

The user's choice is **reconstruct from recording inputs**. Start with this
random-map Great Sahara recording; do not substitute the captured initial
positions or post-setup RNG word. Existing ORDERS §9 establishes the two seed
walks: `init_rules_and_teams` consumes one, then `Setup::build_game` re-seeds
before world/map creation, start permutation, empire creation and herds.
The first measurable target is the generated frame-0 world and whole entity
cast against a separately captured startup oracle, which may be read only by
the comparator. Keep unsupported map styles explicit until implemented.

Two other prerequisites remain visible. RECGAME §4.2's Types section is only
located by landmark, with its class order supplied by the loader; equality of
the combat table alone does not establish equivalence of all loaded rules.
And `the_recorded_order_stream_drives_the_units_it_names` is run7's narrower
input-wiring test: it imports a dump-built world, asserts nine applied move
orders, and permits up to 22 skipped commands. Parsing every external command
is not evidence that the simulation applies every command. A scored replay
must retain the applied/skipped report and account for those gaps.

A further acquisition lead is the organizer's
[Qt3 EE tournament replay packs](https://forum.quartertothree.com/t/qt3-multiplayer-gaming-tournament-1-rise-of-nations/133121).
The post links four public round archives and specifies random land maps,
standard rules and no mods. All four linked Discord CDN URLs returned HTTP
403 to this session's HEAD requests; no pack was acquired and no participant
was contacted. The response ledger is `astra-qt3-replay-packs-20261015.json`
in the durable evidence directory. This is a candidate source, not ten
available compatible fixtures.

The retained wide attempt and the leader-only recipe agree over all 285
shared trace frames and all 1,981 printed leader records in complete dump
frames 1..283 (seven leaders per frame). The comparison includes the preceding
`leader_flags` and `leader_flags2` lines and every indented leader field,
not a selected field list. `astra-replay-leader-overlap-20261015.py` and its
JSON result are preserved with the evidence; a synthetic changed score and
a changed preceding flag each break the comparator. This check is not a
simulation score or a blind independent review.

## Completed playback checkpoint

Implementation `7f1097e9`, offline registration `f4f004b4`; the latter is the
full-gated code tree. The successful full capture is
`~/ron-data/lab-captures/2026-10-15-run900-playback-full-leaders/map-7`.
One take, no relaunch: 21,045 consecutive trace frames 0..21044, closing dump
frame 21045, process and `waitrun.sh` exits 0, success true, five settings
restored. Launch-to-exit 1581.84 seconds, total 1600.66 seconds. The recipe
schedules `!quit` at the last recorded frame (21044); this establishes playback
through the recorded range, not automatic EOF shutdown. The terminal source
package is a camera command; the last simulation-input command printed by the
reader is at 21020.

A separate streaming audit finds every complete leader dump frame 1..21044,
147,308 records, the full printed cast `{0,1,2,3,4,8,9}` on every frame.
The closing block is excluded from this coverage count. All 21,045 shared
RNG states equal the first long attempt's trace, with zero differences.
That attempt is still a failed lifecycle capture, not a successful second
oracle. Whole leader value equality is measured only on the 283-frame overlap
with the retained wide attempt, as above. No sim word or value diff moved;
existing floors, including the held-out battery, remain unchanged.

Evidence manifests: `astra-replay-final-audit-20261015.json` (receipt, frame
coverage and SHA-256), `astra-replay-final-rng-20261015.txt`, plus their audit
script in the durable experiment directory. Final trace SHA-256:
`f0783e964b970727aacbe7601c7379bcdf2eae49e0d267596b122e70d0b732fb`;
final dump SHA-256:
`665d1664855fb4ced74069ada26aa81a5c921be40bc1d65ab2e1b82edfae5829`.

for the Loop: run900 now supplies a completed external playback through the
recorded range and reproducible native RNG/leader evidence. It supplies no
self-contained simulation floor, no original-recording sync proof, and no
ten-game score. The next checkpoint is input-only startup reconstruction;
blind review debt in RECGAME §7 remains open.

Documentation-only verdict follow-up: `RON_INSTALL=<install> zsh
tools/guard.sh --built` exited 0, and `git diff --check` passed. Log
`astra-replay-final-guard-20261015.log` is preserved with the evidence. No
simulation, comparison, or capture implementation changed after the full gate;
the required-fixture verdict remains the six-step run on `f4f004b4`.
