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

*2026-09-19, the first tranche after the sixth Fable pass. **Two landings,
no score moved**: Great Lakes stands at 9510, golden at 626.*

- **408 widened the AI headline's own frame and the word did not move** —
  that is the result. 172 blocks of run100 `[9340, 9511]` compared whole,
  and exactly two keys first part *inside* the window, both on **9382**:
  `1/2007` Village `queued ours 2 theirs 0`, `1/2019` University `0 / 1`.
  Asserted, so 9382 is the frame rather than one of 172. `docs/AI.md` §47.
- **A purchase draws nothing** (§47.3): 9382 costs eight draws a side entry
  for entry and the stream stayed blind the 128 frames to the word. So a
  draw-stream word is a **lower bound** on when a decision parted, never an
  estimate, and `BUILDQUEUE` — from `BUILDS=1`, already on disk — is the
  oracle for an AI purchase. A DECISIONS entry is owed: parked 416.
- **The chapters cannot deliver "every order class"**: the channel stages
  state, so seven of 365's eight add none. `docs/GOLDEN.md` §13 is the
  order-class-to-issuer table DECISIONS 41 §5 awaited, and order coverage
  is a separate axis whose unit is a native issuer (418). `bird` (case 82)
  is the one console command that orders; 365 corrects the three that deny it.
- **Both lanes alive**, holding 414 and 415. Five open, 59 parked; **Fable
  backlog: seven Loop items** (251, 335, 313, 367, 375, 416, 417).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w9510 of 24,000
Golden: w626 of 901 · chapter one pinned · 415 next
Endpoint 24001: EastIndies 64 off, 5 unlinked · GreatLakes 51 off, 11 unlinked

**Opener: 414 on the AI lane — the `LEADERS=9` window over [9375, 9390],
run111; 415 on the rules lane — chapter two staged and pinned, run112.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

414. **A `LEADERS=9` window over `[9375, 9390]`, and the two queue rows on
    9382** (capture lane, run111): the AI headline's own cause frame, named
    by 408 and unresolvable from this disk. The original queues one Scholar
    on University `1/2019`; this crate queues two Citizens on Village
    `1/2007`, same city, same frame. Our census of that city matches the
    dump field for field (`free 0 busy 15 gatherers 15`), so it is the
    **decision** and not the value — and `LEADERS=1` omits the make list.
    Assert the block count against the window asked for (373).

415. **Chapter two staged and pinned** (capture lane, run112): the golden
    record's second chapter, `tools/gamelog/golden/chapter2.cmd`, from
    365's `docs/GOLDEN.md` §14 — which assigns run112–run119 by what a
    failure would teach, chapter five and chapter seven early because each
    can invalidate work built on it. Pin chapter two's word in its **own**
    constant: `floors.rs` holds only `GOLDEN_WORD_CHAPTER_ONE`, and the
    composed `Golden:` line is a guard change parked to the pass (417).
    Lowest chapter first, ruled — the AI track's rule, one map down.

390. **A `LEADERS≥2` window anywhere in (7616, 8174]** (capture lane;
    takes-chain to the met bit): nothing on disk dumps a leader between
    7600 and 8174, so this crate's CONTACT flip frame **7944 is checked
    against a bracket, not a frame**. The cheapest capture left on the
    mechanism 385 just landed. Assert the block count against the window
    asked for — run97's truncation is why (373).

370. **Type the ladder's twenty-five extras** (names rung C's pinned
    `extra` floor, 13→25 under 362's batch fix; rung B +13 the same way,
    compared/off/unlinked unmoved). All scholars and caravans means the
    batch overshoots where the word is long past; a mixed bag is sibling
    divergence on borrowed captures 7,590 and 8,700 frames past the word.
    **No dump can answer it** — an `extra` is ours and not theirs — so
    `docs/journal/2026-09-18-item-362.md` names the one line inside
    `the_east_indies_ladder_is_pinned` that types all twenty-five.

342. **Host choice seats the scholar on the wrong university** — 338's
    residue, re-pinned three times, and now `1/55`. **Re-measure before
    diagnosing**: the vector `(768, 9984)` is the claim, never the count.

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
  means the next unpinned chapter of the golden record. Loop items —
  tooling, guards, these rules — are the steering pass's, never a worker's.
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes these
  lines; the story is `docs/journal/<date>-item-<N>.md`. **A pinned
  constant is the worker's to re-pin; the line is the commander's.** The
  chain — merge, book, gate, push, reap — is `CLAUDE.md`'s. **A number is
  measured on the tip**, after the last `ccc update`, or names its tree.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS` with `Scoreboard:`, `Long captures:`,
  `Golden:`, `Endpoint <frame>:` — and the guards read those phrases and
  `Fable backlog: N Loop items` literally. The length guard counts every
  line under "Where things stand" to the next `## `, blanks included.
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
  free when every landed branch is merged, gated, pushed and reaped,
  nothing is in flight, and both headlines are measured. **Before a blind
  fan-out**, grep `CLAUDE.md` and the memory index: a subagent inherits both.
