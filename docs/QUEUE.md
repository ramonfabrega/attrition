# The queue

Where the work stands and what comes next — the file a fresh session reads
first, and the one a subagent never sees. `CLAUDE.md` is the rules; this is
the state; `docs/journal/` is the story; `docs/PARKED.md` is the backlog,
read when a wave is composed and never at boot.

This file **deletes**; `queueledger.py` fails the build on a number that
leaves in silence or is referred to and never booked; `docs_guard.rs` bounds
it by items (18 at 8 lines, the handoff 32). **A finding that names no
score parks**: only the headline's frame, a floor, or a takes-chain books here.

## Where things stand

*2026-10-15, **the thirty-seventh pass (steer, Fable) after the thirty-seventh tranche: 23 landings from `23e2df12`** (DECISIONS
74), four lanes (Opus 5.5 medium on the AI word, Sonnet 5.5 medium on the rules track and two sweeps): 4.22 USD and **36.5 minutes
waiting a landing** against 3.92 / 27.0 — a fifth broken, so **the fourth lane goes** (entry 61 §5); every gate green, no kill rule, one `DEEP`.*

- **Words**: the peace pair (East Indies, the Lakota, `RUSH_RULES 10`, ends 6822) 5315 → 5564, the open word.
  Chapter 79 closed (1500), 80 staged (run877) at 1198; 69 built, 70 surveyed. Census 1068 → 1140, `port` 171 → 156. Battery 576 / 6566
  / 2974; **when the peace pair closes, the AI word's successor is its lowest lobby** (DECISIONS 72).
- **Gates (DECISIONS 72, 73)**: the booking gate `--scope <last whole gate's tip> --test-threads 8`, whole every third landing
  and at the drain; a whole gate refuses a stale `agent-*` worktree (Loop 2724). The sweep's per-function check runs in debug (2787).
- **Lanes: three, none live** (DECISIONS 74) — booked, unspawned: 2780 (AI §233, EU, run 873), 2785 (GOLDEN §103, EX, run 878),
  2782 then 2783 on the one sweep lane (EV, EW). Next 2788, run 879 (Astra 900–909). Pools: GOLDEN §104–109, AI §234–239, park EY–FC.
- **Fable backlog: 9 Loop items** (1570, 1725, 2003, 2518, 2654, 2750, 2763, 2769, 2775).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850 · GreatSahara 1850/1850 w1850
Long captures: EastIndies w24000 of 24,000 · GreatLakes w24000 of 24,000
Second pair: EastIndies w18140 of 18,140 · GreatLakes w5930 of 5,930
Third pair: EastIndiesFrench w17379 of 17,379 · GreatLakesFrench w5638 of 5,638
Coverage pair: EastIndiesPersianAllTech w4730 of 4,730
Successor pair: EastIndiesMongolAllTech w5056 of 5,056
Peace pair: EastIndiesLakotaPeace w5564 of 6,822
Census: simulation backed 1140 of 3611
Third map: GreatSahara w24000 of 24,000 · GreatSaharaToughest w15432 of 15,432 · GreatSaharaPersianAllTech w4340 of 4,340 · GreatSaharaMongolAllTech w3268 of 3,268
Golden: ch80 w1198 of 1204 · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · ch25 closed · ch26 closed · ch27 closed · ch28 closed · ch29 closed · ch30 closed · ch31 closed · ch32 closed · ch33 closed · ch34 closed · ch35 closed · ch36 closed · ch37 closed · ch38 closed · ch39 closed · ch40 closed · ch41 closed · ch42 closed · ch43 closed · ch44 closed · ch45 closed · ch46 closed · ch47 closed · ch48 closed · ch49 closed · ch50 closed · ch51 closed · ch53 closed · ch55 closed · ch56 closed · ch57 closed · ch59 closed · ch61 closed · ch62 closed · ch63 closed · ch64 closed · ch65 closed · ch67 closed · ch68 closed · ch71 closed · ch72 closed · ch73 closed · ch74 closed · ch75 closed · ch76 closed · ch77 closed · ch78 closed · ch79 closed
Endpoint 24001: EastIndies 0 off, 0 unlinked · GreatLakes 0 off, 0 unlinked · GreatSahara 0 off, 0 unlinked

**Opener: the thirty-eighth tranche (commander, Opus): three lanes from the thirty-seventh pass's commit — 2780 on
the AI word (Opus 5.5 `--effort medium`), 2785 on the rules track and 2782 on the one sweep lane (Sonnet 5.5 `--effort
medium`; 2783 is its next); the whole gate green at the pass; count landings from that commit, stop at twenty, and clear
`steer` into the thirty-eighth pass with the handoff written.**

## Sequential performance work

User-directed branch `codex/astra-windowed-reader`; commander remains paused.
Shared isolation and persistent setup observations validated on `575bad00`: both
complete required-fixture gates passed; full wall456→286s with isolation, CPU
1529→1541s, sampled tree10473→9030MiB. The workflow follow-up makes isolation the default in each
updated cooperating worktree. Third-pair reuse passes full-state differential checks: CPU37–38→5s,
RSS3.4–4.3→0.74–0.76GiB; combined validation complete (ledger correction).
Third-pair four-suite sample: CPU-32%, memory-36%, wall unchanged; see its
[journal](journal/2026-10-16-third-pair-replay.md). Coverage walk now saves21–25% CPU;
full-frame audits and exact fixture/output comparisons pass. Four-suite sample:
wall-16%, CPU-15%, memory-6%; six suites wall-18%, CPU-14%, memory flat.
Full required-fixture gate passed on `f08d0c08`; full-gate speedup unproven.
Ticking/decoder follow-up: coverage CPU−11–13%; scoped macOS reclamation cuts
four/six-suite sample memory15%/20%, CPU6%/7%; six-suite elapsed−9% (four flat).
Complete warm isolated CPU−2%; peak memory variable. Full gate passed on
`3b7e6846`: time/CPU roughly flat, peak memory−18%. See [ticking journal](journal/2026-10-16-ticking-decoding.md).
Live-unit/radius candidates rejected: full isolated165.18→165.57s and
CPU1190.57→1191.85s; all890 tests pass, exact3276 fixture requests unchanged.
No production change or new speedup claimed; see [radius journal](journal/2026-10-16-radius-scans.md).
Workflow follow-up: all gate modes default to isolated execution with unchanged
selection; standalone guards/Cargo wrapper join shared admission. Ledger history
reads retain all2,208 bodies and cut wall30.10→2.13s/CPU27.47→2.08s; RSS+5.3MiB.
Complete required-fixture gate passed on `980efbd5`:271.25s/1509.62CPU/8169MiB
sampled tree; all tests and3276 fixture requests preserved. Warm guard27.98→4.29s.
See [workflow journal](journal/2026-10-16-validation-workflow.md).
Build follow-up: incremental release compilation with256 partitions cuts measured
small sim-edit rebuild44–47→19s and CPU277–289→108–109s. Full gate passes on
`b8494d23`:266.23s/1463.74CPU/9028MiB; exact tests/fixtures unchanged. Runtime
tree peak rises10.5% versus prior run; no suite-memory reduction claimed.
See [build journal](journal/2026-10-16-build-invalidation.md).
Test-family follow-up:736 independent sweeps move into four Cargo targets;
2586 total sim names preserved exactly. Sweep edit21.95→2.21–2.78s/CPU103.49→2.76–2.83s;
full required-fixture gate passed on `69e343a3`:262.15s/1443.50CPU/8360MiB,
all checks and3276 fixture requests retained. See [family journal](journal/2026-10-16-test-families.md).
Public-API follow-up:196 more unchanged sweeps use the existing four partitions;
corner-tile edit14.37→2.47s/CPU16.59→14.00s/build memory1071→400–404MiB.
Full gate passed on `f4e683b6`:255.00s/1396.42CPU/8369MiB;2586 sim tests and
3276 fixture requests preserved. See [journal](journal/2026-10-16-public-sweeps.md).
Golden replay follow-up:211 unchanged bodies split into ten chapter modules with
original test-name wrappers. Case rebuilds2.4–3.4× faster, CPU74–85% lower;
shared-helper CPU flat. Combined validation: source guards corrected on `a30a36bd`
after full run on `e8404b99`; all original checks/3276 fixture requests retained,
one guard added. See [journal](journal/2026-10-16-replay-boundary.md).
Harness follow-up:85 exact bodies in nine groups; measured case rebuild19.52→15.30s,
CPU52.64→20.95s, sampled tree1566→1209MiB. Full required-fixture gate passes on
`5491fe87`:271.28s/1472.54CPU/8864MiB; exact3276 requests retained. No whole-suite
gain. See [journal](journal/2026-10-16-harness-boundary.md). Next: profile repeated
golden/harness replay work (47% of rondata CPU); finer harness groups plateau.
Golden/harness observation follow-up: indexed near spans and bounded East Indies
setup/decoded observations; all45 affected tests pass complete differential audits.
Local warm examples10.60→0.59s and7.63→1.95s; two fail-first guards added.
Full required-fixture gate passes on `55793bfc`; exact3276 requests retained.
Warm rondata161.22→145.58s, CPU1158.05→1037.78 (about10% lower). Whole-suite10×
remains unmet. Next: complete golden civilian observations (283.12CPU family). See
[journal](journal/2026-10-16-golden-replay.md).
Golden civilian follow-up: complete auxiliary observations cached with distinct
ammunition modes; all72 complete-report differential tests pass. Warm chapter36
8.89→1.72s, CPU8.22→1.67; warm memory flat and first-use cost higher. Full gate
pending; see [journal](journal/2026-10-16-golden-civilian.md).
Each revision keeps its own test verdict.
Independent review of earlier landed optimizations remains owed.
Pins/scoreboard unchanged; 10× remains unmet. See
[journal](journal/2026-10-16-shared-runner.md) for evidence and validation status.

## The queue

In dependency order, headline-nearest first. **The roster is Ramon's**:
2780 the AI word's (Opus), 2785 the rules track's, 2782 then 2783 the
one sweep lane's (Sonnet) — lower map first — East Indies (Lakota), the
peace pair's only map. Take the first unstarted item unless a better
order is clear, and say why. The backlog is `docs/PARKED.md`.

2780. **The peace pair's word 5564: ours 26 draws, theirs 24, index
    7** (2761; ours `Unit::think_scout+0xaba`, theirs `Guy::set_anim+
    0x97a < Guy::move+0x19f`; on 5565 who=1's scout `1/0` holds a move
    in ours alone, `group` 65 against 75, group 65 ours alone since
    5542). East Indies run867 (5401..5657) holds it, so no capture
    first: the widening is every record on 5564, both directions,
    `WIDENINGS` naming it, then group 65 walked back to 5542. AI §233.

2785. **Chapter eighty's word 1198: `world:danger` who 1 cell 235 ours
    60, theirs 120** (2755, run877; `0/7` weighed whole at the 1000
    pass in theirs and halved in ours, `0/6` lit by the same Airbase
    agreeing at 60, `seen3` agreeing on 1198 and 1199). The 1000 pass
    is undumped: run 878 captures 998–1002 on `chapter80.cmd` at
    `WORLD=6`, `UNITS=3` (parked 2757), then the value diff on `0/7`,
    both directions. GOLDEN §103.

2782. **The sweep's hundred-and-seventh batch: ten never-backed simulation
    functions, `--ptrtab` first** (DECISIONS 63 (iv); the score
    `Census:` 1129 of 3611): 2768 swept four `GroupData` lists by sixty
    hand flags apiece (parked 2772, Loop 2769); a pointer-array shape
    built with a test made to fail first, then what it and slots ES and
    ET hold, each checked against `census.py --never`. `--kind sweep`.

2783. **The sweep's hundred-and-eighth batch: ten never-backed simulation
    functions a built shape already holds** (DECISIONS 63 (iv), 73; the
    score `Census:` 1140 of 3611): 2774's harvest (ten of ten), off
    `tools/census.py --unbacked`, the `port`-marked rows first, each
    generated by `sweep_gen.py` as it stands — no generator change; a
    function no built shape holds parks under slot EW for 2782's lane.
    The two sweep lanes share `census.rs`'s pin, the commander's at
    each booking (Loop 2367). `--kind sweep`.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you touch; never widen a line to beat a count.
- **A residue item is booked by its frame and its draw delta** (DECISIONS 42); a mechanism in its title is the previous
  item's hypothesis, not a build order, and an absence it cites ("ours has no X") names the probe that showed it (Loop 2704).
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names a headline's frame, a floor, or
  a takes-chain to one. **Neither headline slot is ever empty**: no item
  on the AI word's frame means the widening of that frame; no rules item
  means the next chapter without a capture (GOLDEN §13, §14; the run
  number is minted at booking); **a worker parks what names the
  headline's frame and the commander books it at the merge.**
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes these
  lines; the story is `docs/journal/<date>-item-<N>.md`. **A pinned
  constant is the worker's to re-pin; the line is the commander's**, and
  so is `census::SIMULATION_BACKED`, re-pinned at booking (Loop 2367).
  The chain — merge, book, spawn, gate, push, reap — is `CLAUDE.md`'s.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS`, `AI_WORDS` with `Scoreboard:`, `Long captures:`,
  `Golden:` (`<name> closed`, or `every chapter closed` first), `Endpoint <frame>:`,
  `Second pair:`, `Third map:`, `Census:`, `Fable backlog: N Loop items` and `lower map first — <map>` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours: six blocks each end (run83); `samegame.py` exits 0 when nothing
  is in common, so assert the count, not the verdict; **a detached capture
  waits on `waitrun.sh`** (565, 656). **A value question takes a packet at
  the block before the first print** (EMULATOR §8, 2303).
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in every dump is
  zero-valued or switched off (117, 178); **a never-cleared field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 8 [--scope <tip>]`** (DECISIONS 72), to
  a file, never piped — a pipe launders the 137 — ending `Gate steps:`; **a worker's takes `--lane --tests
  --test-threads 4`** (the brief's, 1623), quotes `Lane verdict:` (969), and widens on a branch that moves a pin (2415).
