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
Full run is in progress at
`~/ron-data/lab-captures/2026-10-15-run900-playback-full/map-7`;
its verdict is not yet claimed.

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
with a fresh output, `--recording <source> --end-frame 21045 --allow-early-end
--timeout 3600 --stall-seconds 120 --log-window 0 21046 --callwin 1 0
--detail end:MISC --detail misc:CHECKSUM=2
--detail endgame:UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=2`.
Use a fresh TMPDIR for the tracer build; this session's shim is
`/tmp/astra-replay-capture-20261015.sh`. Wait through `waitrun.sh` with process
visibility; sandbox-hidden processes make its liveness verdict unreliable.

Focused validation: playback tests 3, existing lifecycle tests 8, runner
tests 27 pass; the runner's held-parent lock test needs process visibility.
The compiled modal trampoline passes 256 Unicorn cases and rejects its
mutated argument offset. Recording regressions exit 0 under a 20 GiB cap.
Full required-fixture release gate is pending on the implementation commit.

## Corpus and direction

The catalog https://ron.heavengames.com/downloads/lister.php?category=rec
lists 245 games. The 48 archived Playback files are project captures, not
48 external matches. Three 2015 catalog candidates were inspected by metadata;
2133's downloaded RCX banner is `(Version: 03.02.03.2905)`, unlike the known
2017 EE sample. Compatibility is not established. Downloads and provenance
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
