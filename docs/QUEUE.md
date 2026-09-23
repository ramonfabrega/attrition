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

*2026-09-23, the eleventh Fable pass: 29 landings since the tenth — 9 on
Opus 5, 20 on Opus 5.5 — and 23 of them moved a word. Great Lakes **10277
→ 12038**; chapters one, two and five **closed at 900**; chapter four (the
namesake) **pinned at 1277**. East Indies 9711 did not move, and it has
been the lower map since 09-21 — so the AI track's default flips to it.*

- **The frame was right every time again** (518 counts ten); nine of 29
  landings turned on an instrument that agreed by not looking.
- **Great Lakes left its fight**: 0.23 USD a frame against 4.07.
- **The lower-map rule was prose and went unapplied for forty landings**;
  it is a guard now, beside the closed-chapter `Golden:` rule, the
  ledger's directory check and the both-sides lint (DECISIONS 47).
- **PRs #7 and #8 merged** as opt-in rungs (CLAUDE.md Tooling, EMULATOR
  §8): a packet before the divergence and a function run on it; 571
  carries the first use. Nothing booked on the merge.
- **566's booking gate was red and its verdict arrived after the
  handoff** — the endpoint read 41 against a pin of 42 (574). Re-pinned.
- **Fable backlog: 6 Loop items** (313, 527, 574, 575, 583, 584).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w10398 of 24,000 · GreatLakes w12038 of 24,000
Golden: ch3 w621 of 900 · ch1 closed · ch2 closed · ch4 closed · ch5 closed · ch7 closed · 590 next
Endpoint 24001: EastIndies 57 off, 5 unlinked · GreatLakes 41 off, 4 unlinked

**Opener: 7 landings since the eleventh pass: 573, 576 and 579 (East
Indies 9711 → 10398), 567 and 569 (chapter four closed at 1500), 578
(chapter seven closed at 1200) and 587 (chapter three pinned at 621).
att-588 is live on the AI track with run143–144. A commander takes 590 on
the rules track, then 571; workers spawn with `claude-opus-5-5[1m]`. The
count is by the log from the pass and stops at twenty.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — East Indies (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

588. **East Indies' word is 10398: the Bark `1/34` a step behind** (579
    moved it 10232 → 10398 with the warship's birth tile and the navy's
    muster, ORDERS §25). The Bark is a step behind from its first step on
    10325 and arrives on 10398 still moving; block 10399 pins eight rows
    in run99's widening. **The word's block is run99's last**, so the item
    owes a capture: run99's line from about 10380, `LEADERS=9` if the
    navy's army record is wanted. Runs run143–144. No mechanism.

571. **Army 1's squad stops on block 11922 in the original and walks on
    here** (566 moved the word 11903 → 12038 by carrying the pathfinder's
    validity memo across searches, PATHFINDER §24). Nine figures, `1/37`–
    `1/42` and `1/62`–`1/64`: `stopped 1`, speed 0 against walking;
    positions agree, no draw; 29 rows pinned in run136's widening. Then
    12038's capture and widening past 11959: ours 4 draws against 5,
    `move_step+0x823`. No mechanism. If PRs #7/#8 land first, a packet at
    11921 with the nine guys' move path run on it is an optional second run.

590. **Golden chapter three's word is 621: +1 draw, 7 against 6** (587
    captured run145 and pinned it; run146, the restage, fired none of §7's
    falsifiers). The extra draw is `Unit::fight+0x9b0` on the catapult
    `0/9`: ours gives the packed catapult an attack order on its birth
    block at `idle 1`, the original none; values part on 622. 587's
    hypothesis: `Unit::think_attack`'s packed-unit arm. run146's own first
    parting, 633 (the chariots' chase, 8 against 9), is pinned beside it.
    The rules headline. No mechanism.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you touch; never widen a line to beat a count.
- **A residue item is booked by its frame and its draw delta** (DECISIONS
  42); a mechanism in its title is the previous item's hypothesis.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names a headline's frame, a floor, or
  a takes-chain to one. **Neither headline slot is ever empty**: no item
  on the AI word's frame means the widening of that frame; no rules item
  means the next chapter without a capture (GOLDEN §14; the run number is
  minted at booking); **a worker parks what names the headline's frame
  and the commander books it at the merge.**
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes these
  lines; the story is `docs/journal/<date>-item-<N>.md`. **A pinned
  constant is the worker's to re-pin; the line is the commander's.** The
  chain — merge, book, gate, push, reap, spawn — is `CLAUDE.md`'s.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS` with `Scoreboard:`, `Long captures:`,
  `Golden:` (a closed chapter reads `chN closed`), `Endpoint <frame>:`,
  `Fable backlog: N Loop items` and `lower map first — <map>` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours: six blocks each end (run83); `samegame.py` exits 0 when
  nothing is in common, so assert the count, not the verdict; **a wait
  keys on the runner's exit, never on a file** (565). **A value question
  takes a packet at the frame before, not a detail capture** (EMULATOR §8).
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in
  every dump is zero-valued or switched off (117, 178); **a never-cleared
  field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A clear is free when
  every landed branch is merged, gated, pushed and reaped and none is in flight.
