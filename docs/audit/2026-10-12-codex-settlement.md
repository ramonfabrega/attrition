# The Codex settlement, 2026-10-12 — Astra's second direct tranche taken whole, and what it says about the loop

**Fable 5.1, the `steer` session, in the main thread with Ramon.** Not a
numbered pass: a pass runs every twenty landings or when the headline has
not moved for two sessions, and this is nine landings since the
twenty-seventh with the headline moved. What it did is what DECISIONS 60
says a direct tranche gets — fast-forwarded, gated here, pushed, its
review debt filed — plus the steer Ramon asked for.

## For Ramon

- **Where things stand.** The twenty-seventh pass left the commander
  held for a talk (2026-10-07). Since then the loop's branch did not
  move; Astra's Codex session worked `codex/core-loop-investigation` off
  the pass's tip, 21 commits over 2026-10-09 to 10-11, its worktree
  clean. Nine landings: two on the headline word (1634, 1647: coverage
  pair **3395 → 3439**), five on the harness (1640–1646), two REA trials.
- **Taken whole, by entry 60's rule.** Fast-forward, then the booking
  gate on the result: **exit 0, six of six**, rondata 778 passed in
  492.3 s (619 s for 779 at the pass — the speedups are real), sim 1477,
  fixed 13, memcap 12,174 MiB, fixtures required and none missing. The
  weakened-check scan: no float in `sim`, `#[ignore]` five before and
  after, two clippy `allow`s on harness signatures, assertions added and
  none removed. Pushed to the branch and fast-forwarded to `main`.
- **The review debt is 1649**, two code-changing claims (the road flag
  before the upgrade sweep; the fleet in the live caravan count and the
  census's asymmetry) — the twenty-eighth pass's to ratify, with the
  twenty-three Loop items already on the ledger.
- **Resume the commander.** The queue's opener stands as the pass wrote
  it, with 1648 in 1634's place: four lanes, Opus 5.5 on the word, Sonnet
  5.5 on the sweep and the rules track, 1639's three racers. The row is
  `stopped`; `ccc attach commander` resumes it and `ccc clear commander
  --then "<the opener>"` fires on its first idle. The spend is yours: the
  last tranche read 414.72 USD list for twenty-two landings.
- **Astra's throughput, for the roster question.** Nine landings in
  three days of a sequential arm, two on the headline; the last Codex
  tranche did twenty in fifteen hours. `lore` does not index Codex, so
  there is no price beside it but your subscription's word. Entry 60's
  question — is the architecture the variable, not the model — still has
  one data point on the Claude side (1448's arm) and now two on Codex's.
- **The disk is at the line**: 61 GiB free of 926; the pass's rule asks
  you before deleting under 60. Sixteen of those GiB are build output in
  the four Codex worktrees under `~/.codex/worktrees/` (fc0b 6, cecf 5,
  28ba 3, 2a94 2); `cargo clean` there is yours, not the loop's.
- **1641's LZ4 is a lever on the archive** (191 GiB): opt-in, 18.5×
  smaller at replay speed. Re-archiving the long captures is yours (1650
  carries the pointer); nothing was compressed.

## What was checked, and how

Branches not merged into the tip: four `codex/*` (three are
September's lab branches, 3,096 commits behind, kept as records),
`rescue-234` (a WIP rescue, kept), and `worktree-att-1565-opus` — the
race's Opus arm, kept on purpose as the record (`docs/audit/2026-10-07-race-1565.md`).
Only `codex/core-loop-investigation` descends from the tip; its base is
the pass's second gate commit, so the fast-forward is exact and no merge
was needed. The Codex journals' own validation (full gates, exit 1 twice
on stale pins and corrected by `tools/repin.py`, then combined
validation) was not taken on trust: the gate here ran on the
fast-forwarded tree and is the verdict above.

The Codex queue carried a "Branch checkpoint" section of nine landings'
prose under the handoff; folded into "Where things stand" and deleted,
the stories being in `docs/journal/`. Its scoreboard line already read
w3439 and the floors pin it; the ledger guard holds every number.

## Outside: banteg, read for what transfers

Ramon asked that banteg's work stay in view. Read this steer: the
Crimsonland post and methodology, the `harvest` repository (Harvest:
Massive Encounter, a byte-matching decomp of the 2008 game plus a Zig and
SDL3 port that keeps the recovered source untouched), "Binary Ninja for
Agents" (2026-03), and the posts list (no "harvest" post; "Translating
Fermion", 2026-09, is a script-bytecode translation, not a decomp).

- **Crimsonland reached a 100 % matching decomp** (858 functions, VC6
  compiling back to the original bytes) *after* the Python rewrite
  shipped on differential replay. The order is ours too — rules first on
  diffs, the matching-decomp as a later oracle if ever — and the
  methodology page is our method without the PDB: Ghidra for structure,
  Frida for runtime evidence, tick-by-tick checkpoints for the verdict.
  Nothing to adopt; a confirmation.
- **`harvest` scores itself with one public number** — the decomp.dev
  match percentage on the README — and a `just match` that rebuilds and
  diffs every unit. Ours are `Census: simulation backed 68 of 3611` and
  the ticks before divergence; both live in the queue and in test
  constants, neither on the README. A candidate for a pass, not a lane:
  a README badge or line that prints the scoreboard, so the project's
  state is readable without opening `docs/QUEUE.md`. Filed nowhere yet;
  Ramon's call whether the repo is public enough to want it.
- **"Binary Ninja for Agents" is the tool-shape lesson**: a shell CLI
  over a live session, text and JSON outputs that spill to disk with a
  token count when large, a preview mode that applies, diffs and reverts,
  and verification that a write landed. `tools/ghidra/` already answers
  by `grep` over a static export, which is the same choice for a
  different reason (no live session to hold). What transfers is the
  spill-with-count: our probe outputs go to `target/item-N-investigation/`
  by convention, and the Codex REA trial's bundle (143 kB full, 2.4 kB
  summary) is that shape done by hand. 1638 (the reading-heavy half in a
  subagent) is the same economy from the other side.
- **The matching toolchain** (a 2010 container with GCC 4.4.3, delinked
  objects, objdiff) is the one thing we cannot borrow: our executable is
  MSVC 2002–2003 and we are not matching bytes. `tools/recomp/lift.py` is
  the nearest cousin and stays a reading tool.

## Not done here

The Loop ledger is unruled (twenty-five items after 1649 and 1650); the
twenty-eighth pass takes it at the tranche's end. No hard-constraint, no
roster, no phase and no finish line was touched. `CLAUDE.md` unchanged.

## The disk, after the talk

Ramon's word on the cleanups and the archive, the same afternoon.

| step | freed |
|---|---|
| `cargo clean --profile dev` in the integration worktree | 43 GiB |
| ten `agent-*` worktrees of 1619's sweep, every patch in HEAD, removed with their branches | in the above |
| `cargo clean` in the four Codex worktrees | 32 GiB |
| the archive, APFS-compressed in place with `afsctool -c -f -T LZFSE -J8 ~/ron-data` | 205 GB → 22 GiB, 13,248 files |
| **free** | **258 GiB**, from 61 |

Why APFS and not Astra's `.rcap`: the harness opens a `.rcap` only
where a path names one, and 58 Python probes under `tools/` read the
dumps raw, so that route is a migration; filesystem compression is
invisible to every reader. Measured before the word was given: a 1.41
GB dump to 36 MB (39×), an 822 MB packet payload to 196 MB (4.2×),
both byte-identical; the consolidated Great Lakes widening 46 s and
1,433 MiB raw against 46 s and 1,426 MiB compressed, twice each. The
proof after: the full booking gate on the compressed archive, exit
0. A new capture lands raw; the Loop line (1650) says who compresses it.

## Decided with Ramon, after the settlement

- **The twenty-eighth tranche is the race, on the headline word** —
  four arms (Opus 5.5 high and medium, Sonnet 5.5 high and medium) on
  1648, five landings each, one session a landing, nothing else out;
  DECISIONS 64 amended, the queue and opener rewritten. The commander
  was attached by Ramon and the clear armed with the opener from this
  session.
- **After the race, a talk**: the pass scores the 2×2 and holds the
  commander; the roster is decided then, and the twenty-ninth tranche
  runs the four lanes on it.
