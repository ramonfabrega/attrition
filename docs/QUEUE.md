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
**19 landings since 29a46bd**, every one a word or a chapter. Lane
att-718 (chapter thirteen) is live; the AI lane is held for the stop.*

- **Great Lakes 12536 → 15384 in eight landings**: the retry's gate, a
  resumed search, a city's halving, the stray-road sweep, the gather park,
  the graft and patriot, the target's mirror, the site recruiter (715).
- **East Indies 13640 → 15985 in two**: animation names case-folded (643,
  ANIM §12), then 706's graft and patriot. The word is past run78 (708).
- **Chapter six-b closed** at 1250 (651, 680): a captain's attack on a
  building re-runs `find_new_target` every frame (COMBAT §62).
- **The issuer chapters**: nine (move), ten (patrol, ORDERS §27) and
  twelve (follow, §28) closed; the census's order row moved for the first
  time, 44 → 49 cited of 410. Eleven, the guard, closed at 1250 in five landings: four
  collision arms, a leash, vision's resync, a per-guy danger flag (ANIM §13).
- **Fable backlog: 6 Loop items** (527, 677, 685, 687 a capture's cost,
  692 the census's row, 697 `cover=1` hangs); no `FABLE:` marker.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w15985 of 24,000 · GreatLakes w15384 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · 718 next
Endpoint 24001: EastIndies 55 off, 3 unlinked · GreatLakes 32 off, 0 unlinked

**Opener: 718 is live on the rules lane and the AI lane is held: the
twentieth landing is due. After the pass, 722 then 708 (Great Lakes is
lower). A detached capture waits on `tools/gamelog/waitrun.sh`; a report
quotes the gate's `Gate steps:` line.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — Great Lakes (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

722. **Great Lakes' word is 15384: who=1's wonder arm** (715 moved it 15383
    → 15384: `Wall::process`'s site recruiter re-recruits a citizen for a
    wonder or an undamaged fort, AI §69). On 15384 ours 51 draws against
    46 at index 4: a third `Leader::create_buildings+0xffb`/`+0x1017` pair
    (the wonder arm's `% 1000`, `% 300`), theirs `Animal::think_bird
    +0x82`. On 15386 `MAKE[1]` is `SIEGEFACTORY` here, empty there. Inside
    run202, widened. No mechanism.

708. **East Indies' word is 15985, past every capture** (706 moved it 15782
    → 15985 and closed 694: `1/60` is The Senator). On 15985 ours 5 draws
    against 6 at index 4: ours `Unit::think_scout+0x941`, theirs
    `Leader::make_stuff+0x63d`. run78 ends on 15900. The `WIDENINGS` row
    names this item: capture over the word, sized to it, and widen whole.
    No mechanism.

718. **Chapter thirteen, the garrison line — an issuer the AI never uses,
    no capture yet** (DECISIONS 49; GOLDEN §13; 714 closed chapter twelve
    at 1150 and moved the census's order row 47 → 49 cited).
    `CommandManager::issue_garrison@00941a70` from the tracer DLL, one unit
    and a squad garrisoning a building, then ungarrisoning, 714's harness.
    The issuer under the emulator first; the premise names its killer.
    Takes GOLDEN §21; run208, at `cover=0` (Loop 697).

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
