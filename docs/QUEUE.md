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

*2026-09-22, the tenth Fable pass: no score moved and none was meant to.
Great Lakes **10277**, chapter two **683**, both inside their first fight.*

- **Fourteen landings since the ninth pass, every worker on Opus 5, the
  chain run unattended to its stop** — the ninth pass's clauses held.
  Both words moved. Workers 26 USD a landing against 20, 53 a word
  against 46, and **4 USD a frame against 0.19**: a fight draws on every
  frame, and a residue there buys a frame where one between fights
  bought 651.
- **Ten of fourteen landings turned on an instrument defect**, six of
  them a field the original prints that nothing read. That is a guard
  now: `rondata::diff::coverage` records what the parser asks each
  block for and pins the dump's own keys against it — **20 paths, 239
  keys unread** on the two headline windows, three record families
  never opened (`DEATH_OBJS`, the frame-level `GUY` list, per-frame
  `WORLD`). A key a landing reads is deleted from the pin in that
  landing. The count is the next pass's measure.
- **Struck text no longer counts** against a section's ceiling (480): a
  correction is not an addition. Four pins fell; `AI.md` §15 left the table.
- **491 landed both of 683's sides**; 476 and 342 stay parked.
  **Fable backlog: 6 Loop items** (313, 503, 507, 508, 509, 513).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10817 of 24,000
Golden: w626 of 901 (ch1) · ch2 w725 · 510 next
Endpoint 24001: EastIndies 64 off, 10 unlinked · GreatLakes 53 off, 3 unlinked

**Opener: 502 and 506 in flight. Five landings; Great Lakes 10277 → 10582
in four, ch2 695 and its comparator. Merge, book, gate, push, reap, spawn.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

515. **10817's eight rows, all one raider `1/28`** (the AI headline's
    own frame; 506 moved the word 10582 → 10817). A collision **this
    crate takes and the original does not** — `collide` 1 v 0,
    `collide_who` 8 v −1 — with the stand behind it (`g.cur_anim` 0 v
    8, `g.stopped` 1 v 0) and the clock triple under that. The draw
    stream parts on the same frame and says the same thing: ours
    **three** draws against **two**, the extra `Guy::set_anim+0x97a <
    Unit::move_step+0x823`. COLLISION §11's family. No mechanism.

510. **725's attack-end wrap costs the original two draws and this
    crate one** (the rules headline's own frame; 502 moved the word
    695 → 725). Ours 7 draws against 9, parting at draw 1; the delta
    is two `Guy::set_anim+0xf2f < Guy::inc_time+0x271`. Three wraps in
    two frames (`0/7`, `0/8` on 725, `0/6` on 726); the **same** wrap
    on 695 cost one, with `hold_attack 1` in the dump. Start:
    `Sim::guy_inc_time` marks `SITE_ATTACK_WRAP` on the `slot < 2` arm
    only and hard-codes ATTACK2. COMBAT §44.3. No mechanism.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you are touching, never another
  author's item, and never widen a line to beat a count.
- **A residue item is booked by its frame and its draw delta** (DECISIONS
  42); a mechanism in its title is the previous item's hypothesis.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names a headline's frame, a floor, or
  a takes-chain to one. **Neither headline slot is ever empty**: no item
  on the AI word's frame means the widening of that frame; no rules item
  means the next unpinned chapter; **a worker parks what names the
  headline's frame and the commander books it at the merge.**
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes these
  lines; the story is `docs/journal/<date>-item-<N>.md`. **A pinned
  constant is the worker's to re-pin; the line is the commander's.** The
  chain — merge, book, gate, push, reap, spawn — is `CLAUDE.md`'s. **A
  number is measured on the tip**, after the last `ccc update`.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS` with `Scoreboard:`, `Long captures:`,
  `Golden:`, `Endpoint <frame>:`, `Fable backlog: N Loop items` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours: six blocks each end (run83); `samegame.py` exits 0 when
  nothing is in common, so assert the count, not the verdict.
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in
  every dump is either zero-valued or switched off (117, 178). **A
  never-cleared field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A commander's clear is
  free when every landed branch is merged, gated, pushed and reaped, and
  nothing is in flight. **Before a blind fan-out**, grep `CLAUDE.md` and
  the memory index: a subagent inherits both.
