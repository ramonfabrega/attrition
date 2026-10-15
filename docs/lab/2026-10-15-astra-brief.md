# A brief for Astra, 2026-10-15 — the replay oracle, and a wishlist the next steer would merge

Written at the thirty-fifth steering pass (Fable, with Ramon on device;
`docs/audit/2026-10-15-fable-pass-35.md`, DECISIONS 72). Astra works under
`AGENTS.md` — one agent, sequential, its own `codex/*` branch cut from
`main` after Ramon fast-forwards it to the pass's tip — and is settled by
DECISIONS 60's routine: fast-forward when the branch descends from the
tip, the whole booking gate here, pushed, the review debt filed. Nothing
below is a queue item; the commander's lanes run beside this on the
integration branch, and the two never share a file by arrangement. Where
they collide, `ccc update` is the loop's problem and a rebase is Astra's.

## The one thing worth the whole session: a replay oracle on games nobody here played

**Why.** Every pair the loop mints closes, and the held-out battery — three
lobbies nobody tuned for — has not moved in eleven tranches (576 / 6566 /
2974 frames of 13519 / 7303 / 20380). The loop measures generalisation
and never works on it. banteg's Snail Mail (2026-10-08) closed the same
gap with two tick-by-tick oracles on recordings he did not author. We
have the raw material and half the reader:

- `ron.heavengames.com`'s downloads hold about 245 recorded games
  (`docs/ORACLE.md` Part 1); a 48-file `.rcx` corpus already sits beside
  the gamelogs (`$RON_RECGAME_DIR`, then `dump`'s directory;
  `docs/INPUT.md`'s corpus table; the `~/Documents` TCC stall is solved).
- `docs/RECGAME.md` is the container, derived and sample-checked:
  header, types, the full initial-state snapshot, and the order stream —
  21,884 `CommandPackage`s over 21,044 frames on the sample, every one
  from the human (the AIs are recomputed on playback). `docs/COMMANDS.md`
  decodes every payload. `rondata <install> --recgame <file>` parses the
  whole stream and pins the embedded combat table against the install's.
- `crates/rondata/src/diff/order.rs`'s
  `the_recorded_order_stream_drives_the_units_it_names` already feeds a
  recording's orders to the sim for the units it names.
- The original's logger runs under `Game::run_playback` as it does under
  `run_solo` (`docs/ORACLE.md`, "It runs in single player"): a recording
  played back with the end-frame categories on is a per-frame dump of a
  real match, the same file shape every pinned widening reads.

**What to deliver, in order; stop at a natural end and hand off.**

1. **One recording played back under the logger**, end to end, on the
   click-free lane or the mouse lane, with its `docs/RUNS.md` section
   (the ledger's form; a run number from Ramon, the loop's next is 869
   and the commander mints upward — take 900–909 for this work so the
   two never collide). The questions a first playback answers: does
   playback honour `rise.ini`'s fixed seed or carry its own from the
   file (the recording embeds the seeds, RECGAME §3); what `GAME INFO`
   line it prints; whether the dump's per-frame records match
   `run_solo`'s in shape; and the frame the recording's own sync
   checksums agree with the dump's.
2. **The sim driven from the recording**: the harness loads the initial
   state the container carries (RECGAME §4 — the part no widening has
   needed, since every capture so far starts from `Game::init` on a lobby
   this crate already reproduces), feeds the order stream frame by
   frame, and `first_parting` scores it against the dump of step 1. The
   first version may well part at frame 0 or 1; **that parting frame and
   its value diff are the deliverable**, pinned as a floor in
   `rondata::diff` the way every lobby's is (`FLOORS`, a `Pair` or the
   battery's form in `diff/battery.rs`), and the handoff names the
   mechanism by frame and draw delta, never a build order (DECISIONS 42).
3. **Ten recordings, scored on first try**, as the battery is: a table
   of parting frames, no debugging against them. That table is the
   number this project has been missing — lockstep length on human
   games — and the steer reads it before anything else.

**What is known to be in the way.** A recording is worthless without
its map (`docs/FORMATS.md` §1): playback needs the scenario the game was
made on, and heavengames recordings are mostly random-map skirmishes
whose map the container regenerates from its seeds — check that before
downloading more. The `Types` section's class sequence is the loader's
knowledge, not the file's (RECGAME §5), which the snapshot loader of
step 2 will meet first. A 2017-build recording against the 2024 install
drifts narrowly at the `Constants` block (RECGAME §5). Playback checks
no version, so an old recording is misread rather than refused.

**Rules that apply, with their reasons.**
- No float in `crates/sim`, ever (`CLAUDE.md`; `no_float.rs` enforces).
- Nothing from the install or the downloads enters git: a recording is a
  user file; its parsed facts and its parting frames are ours.
- A new reader of a dumped field goes in the coverage driver with its
  window, or the pin stays blind (`rondata::diff::coverage`).
- The capture lane is pooled and locked (`ron_lane_state`,
  `tools/gamelog/winelaunch.sh`); the loop's lanes capture on it too.
  Take it, never remove a lock by hand, and wait on `waitrun.sh`.
- `python3 tools/release_gate.py <install> --test-threads 8` before the
  branch is called done, to a file; its `Gate steps:` line in the
  handoff. A worker's gate is `--lane`; Astra's is the whole one.
- Journals as `docs/journal/<date>-astra-<topic>.md`; `docs/QUEUE.md`,
  `CLAUDE.md` and the pinned floors of existing lobbies are the loop's
  and are not edited here.

## The wishlist — tools the next steer merges on sight if they land right

Each is a parked Loop row the passes have carried; each is small, each
wants a test made to fail first, and each is merged by the next pass
without a reading. Numbers are `docs/PARKED.md`'s.

- **A progress page** (Ramon's ask; no row). `tools/progress.py` reads
  the booking commits (`git log --format=%B` of every `Book NNNN`
  commit) and the queue's `Scoreboard:`, `Census:`, `Battery` and
  `Fable backlog:` lines over time, and writes one HTML page: every
  lobby's word against its length, the census against 3611, the three
  battery lobbies, price and landings a tranche. decomp.dev's shape,
  from our own numbers. Held by a test on a fixture of three commits.
- **The battery's partings side by side** (DECISIONS 72's open row).
  `tools/partings.py`: every lobby's `first_parting` on the tree, both
  sides' draw sites at the parting, grouped by site — so an item can be
  booked by the mechanism that parts the most lobbies, not the nearest.
- **2422 `tools/drawdiff.py <pair> <frame>`**: both sides' draws on a
  frame aligned by site (`RON_DEBUG_SITES` beside `report.py draws`, now
  `draws lo-hi`), red first on the peace pair's 3302.
- **2410 `tools/leaderdiff.py <dump> <frame> <who>`**: both leaders'
  whole `LEADERDATA` across a block, unread fields included; red first
  on run849's block 1985.
- **2459 `tools/gamelog/buildat.py <dump> <frames> <who>/<o>`**: a
  building's queue across frames, printed on change and cut at `queued`
  (2407 folded in); red first on run853's `1/2018`.
- **2418 `tools/gamelog/rareheld.py <good>`**: every dump's holders and
  deposits of a rare by frame (`rares_collected` slot = good − 6); red
  first on one dump's record.
- **2405 `tools/census.py --frontier`** and **2435 `--callees
  <function>`**: never-entered simulation functions with an entered
  direct `E8` caller (the frontier a new lobby enters), and the
  cited-and-never-entered rows a function's call graph reaches; red
  first on `Leader::diplomacy`.
- **2505 and 2518, `stage_walk`'s two prints**: every placement's refusal
  with its reason, and `queue=[]` once on a dequeue; red first on a
  blocked site and on run864's shape.
- **2506**: `tools/emu/callfn.py` seeds the stack with a marker per row,
  so a port reading a stale word fails where it should; red first on
  `issue_cannon_time`.
- **The 14 GiB test binary**: the timing run's new memory column
  (`docs/audit/2026-10-15-suite-memory.txt`, when it lands) names the
  tests; if one widening holds a whole 24,000-frame dump, stream it.
- **A build cache across worktrees** (2408): `sccache` with
  `--remap-path-prefix` on a lane-shaped worktree, hit rate and minutes
  measured on a clean `cargo test --release --no-run` twice; a number,
  not a flip — the pass flips it.

## What a handoff looks like

One file, `docs/journal/<date>-astra-replay-oracle.md` or the topic's
name: what landed by commit, the parting frames and value diffs, the
`Gate steps:` line and the tree it ran on, what is not established, and
one line headed "for the Loop" per finding the loop's tools should take.
The steer fast-forwards and gates; a branch that does not descend from
the tip is rebased by Astra first.
