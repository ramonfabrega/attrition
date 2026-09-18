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

*2026-09-18, the fifth Fable pass (`docs/audit/2026-09-18-fable-pass-5.md`),
in conversation: **two tracks** (DECISIONS 41). No score moved, none meant
to. The loop before it landed 26 items, 20 word-moving, GL +1,503, EI
+1,899, at 31–39 USD a word-moving landing. Next: 363 and 362, side by side.*

- **The rules track is a golden record** — one staged game, AI off, in
  chapters, on the lab's click-free lane with `rontrace.cmd` and native
  issuers. 363 takes the first staged run and its three answers; 364
  replays it in the harness and pins chapter one; 365 writes the script.
  Until 364 lands the `Golden:` line says so, and the guard reads it.
- **The AI track is unchanged.** Great Lakes 9182 (item 360 left
  `Unit::move_step`'s snap arm clean: point-and-goal 6, walk-slot 0, off
  position 4 over `[8029, 9182)`); East Indies 9711 on the draw stream.
  362 is the market at 9182. Both frontiers have runway: run99 East
  Indies `[8780, 10400)`, run100 Great Lakes `[9340, 10900)`.
- **The census is the fourth counter** (`tools/census.py`,
  `docs/CENSUS.md`): 862 of 48,233 functions cited; the 31 order classes
  44 cited of 410, six untouched — air, cast, trade, special anim. The
  estimate on record: four to six months to every family diff-backed.
- Four items open, 33 parked. **Fable backlog: eight Loop items** (251,
  335, 341, 343, 356, 321, 313, 367); 367 is the AI proxy table.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w9182 of 24,000
Golden: none pinned · 363 → 364 → 365
Endpoint 24001: EastIndies 58 off, 14 unlinked · GreatLakes 57 off, 10 unlinked

**Opener: 363 on the capture lane and 362 in the loop, side by side —
the golden record's first staged run, and the market at 9182.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

363. **The golden record's first staged run** (capture lane, Opus, its own
    session; DECISIONS 41 §5): `live_session.py stage` with a `rontrace.cmd`
    — `!ai off` at frame 0, `age` to a late age, spawns for two players,
    `war`, a fight, `!quit` — the lane's window and categories exposed
    (parked 341). Products: the draw-stream trace, a windowed dump, the
    script, and three answers: auto-engage survives AI-off; two launches
    give one draw stream; a late-age dump parses through rondata. Then the
    held-out third map, one scored capture, measured and never debugged.

364. **Replay 363's script in the harness and pin chapter one** (takes
    363): an interpreter for the record's cheat set and the command
    dispatch of `docs/COMMANDS.md` §3 onto the sim's entry points —
    `add_unit`, `add_building`, `gain_tech`, `set_age`, the diplomacy
    state — the orders from the `.rcx`, the cheats from the script. Names
    the golden word; its first pin rewrites `Golden:`, which the guard reads.

365. **`docs/GOLDEN.md`, the script itself** (a reading and a design, Opus):
    chapters from `docs/CENSUS.md`'s order family and `docs/COMMANDS.md`
    §3 — every order class, each unit line, an age jump between chapters,
    the Temple chapter for attrition, a war — each chapter naming the
    records that would falsify it. Written before 364's second chapter.

362. **Great Lakes 9182 is the market's, and 358 left it there** — three
    `use_market+0x1ed` draws against this crate's one, plus an animation
    tail of one `set_anim+0x104b` against three. `docs/AI.md` §41 names the
    `LEADERS=9` window that reads it. Named on the frame, so **no widening
    owed**. run97 dumps 9182 with 167 frames above, and run100 carries
    the same game from 9340 to 10899.

342. **Host choice seats the scholar on the wrong university** — 338's
    residue, re-pinned three times. **Re-measure before diagnosing**: the
    vector `1/2020 − 1/2019` is the claim, never the count.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you are touching, never another
  author's item, and never widen a line to beat a count.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names a headline's frame, a floor, or
  a takes-chain to one. **Neither headline slot is ever empty**: no item
  on the AI word's frame means the widening of that frame; no rules item
  means the next unpinned chapter of the golden record. Loop items —
  tooling, guards, these rules — are the steering pass's, never a worker's.
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes the
  `Scoreboard:` line; the worker's story is `docs/journal/<date>-item-<N>.md`.
- **A number is measured on the tip** — after the worker's last `ccc
  update`, or the report says which tree. 301's +381 was attributed to the
  wrong half against a stale base and would have landed wired.
- **Merge, gate, push, reap — one chain.** A reap left for "before the
  next spawn" was forgotten twice in one session.
- **A floor that moves** moves `FLOORS`, the assert reading it, and the
  `Scoreboard:` line together; the guard parses the line against the floors.
  The `Golden:` line is the same shape for the rules track.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours on purpose: six blocks each end (run83); `samegame.py` exits
  0 when nothing is in common, so assert the count, not the verdict.
- **Grep for the derived quantity, the SHAPE and the GATE — not only for
  coverage of a frame.** A term zero in every dump is either zero-valued or
  switched off: 117 and 178 were the second.
- **A check on a never-cleared field asserts a CHANGE, not a value** —
  `collide_frame` is a permanent stamp (run85). Test both directions first.
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A commander's clear is
  free when every landed branch is merged, gated, pushed and reaped,
  nothing is in flight, and both headlines are measured. **Before a blind
  fan-out**, grep `CLAUDE.md` and the memory index: a subagent inherits both.
