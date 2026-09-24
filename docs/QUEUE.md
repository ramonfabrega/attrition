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

*2026-09-24, the commander (Opus 5.5) after the thirteenth Fable pass:
**6 landings since 29a46bd**, every one a word or a chapter. Lanes
att-688 (Great Lakes) and att-693 (chapter ten) are live.*

- **Great Lakes 12536 → 14382 in two landings**: the retry gates on
  `is_move` (673), and a resumed search reads its blocks as suspended
  (678, PATHFINDER §26, run178). **East Indies 13640 → 15782** (643):
  animation names resolve case-folded, first match (ANIM §12, run166).
- **Chapter six-b staged and closed** at 1250 (651, 680): each aircraft
  walks at the enemy Airbase, and a captain's attack on a building re-runs
  `find_new_target` every frame (COMBAT §62, run177's packet).
- **Chapter nine, the first issuer chapter, closed at 1100** (676, run180,
  GOLDEN §17). The census's order row stayed 44 of 410 (692), so chapter
  ten is patrol, an issuer the AI never uses.
- **Fable backlog: 5 Loop items** (527, 677, 685, 687 a capture's cost,
  692 the census's order row); no `FABLE:` marker was filed.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w15782 of 24,000 · GreatLakes w14382 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · 693 next
Endpoint 24001: EastIndies 46 off, 10 unlinked · GreatLakes 33 off, 0 unlinked

**Opener: 693 is live on the rules lane and 688 on the AI lane (Great
Lakes is the lower map); 694 follows 688. A detached capture waits on
`tools/gamelog/waitrun.sh`; a report quotes the gate's `Gate steps:` line.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — Great Lakes (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

688. **Great Lakes' word is 14382: who=1's building placement** (678 moved
    it 12897 → 14382: a resumed search reads its blocks as suspended,
    PATHFINDER §26, AI §66). On 14382 ours 11 draws against 13 at index 2:
    ours `Leader::make_stuff+0x221`, theirs `Leader::produce_building
    +0x1805`. On 14383 who=1 places `1/2025` at (39552, 17472), the
    original at (42624, 19776). The nearest input the dump prints is
    `SITE[1]`, parting on 12976. Inside run178 (to 14899). No mechanism.

694. **East Indies' word is 15782: who=1's birth of `1/60`** (643 moved it
    13640 → 15782: animation names resolve case-folded, first match, ANIM
    §12). On 15782 the original spends three `Guy::init_real+0x52` and a
    wrap before the bird's `set_anim+0x104b`; ours spends only the bird
    and the farms. The three are one birth, `1/60` with three figures
    (item 227). Inside run78 [15700, 15900], never widened: widen it
    first, and ask whether `LEADERS=1` prints the make list. No mechanism.

693. **Chapter ten, the patrol line — an issuer the AI never uses, no
    capture yet** (DECISIONS 49; GOLDEN §13; 676 closed chapter nine at
    1100 and the census's order row stayed 44 of 410, parked 692).
    `CommandManager::issue_patrol@00941800` from the tracer DLL on one unit
    and on a squad, 676's harness. The issuer under the emulator first,
    then the pair; the premise names its killer. Takes GOLDEN §18; run184.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you touch; never widen a line to beat a count.
- **A residue item is booked by its frame and its draw delta** (DECISIONS
  42); a mechanism in its title is the previous item's hypothesis, not a build order.
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
  constant is the worker's to re-pin; the line is the commander's.** The
  chain — merge, book, gate, push, reap, spawn — is `CLAUDE.md`'s.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS` with `Scoreboard:`, `Long captures:`,
  `Golden:` (`<name> closed`, or `every chapter closed` first), `Endpoint <frame>:`,
  `Fable backlog: N Loop items` and `lower map first — <map>` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours: six blocks each end (run83); `samegame.py` exits 0 when
  nothing is in common, so assert the count, not the verdict; **a detached
  capture waits on `tools/gamelog/waitrun.sh`** (565, 656). **A value question
  takes a packet at the word's own frame, not a detail capture** (EMULATOR §8).
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in
  every dump is zero-valued or switched off (117, 178); **a never-cleared
  field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137 — ending `Gate steps:`. A
  clear is free once every landed branch is merged, gated, pushed and reaped.
