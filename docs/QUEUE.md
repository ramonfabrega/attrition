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
**15 landings since 29a46bd**, every one a word or a chapter. Lanes
att-711 (Great Lakes) and att-713 (chapter eleven's word) are live.*

- **Great Lakes 12536 → 15175 in six landings**: the retry's gate, a
  resumed search, a city's halving, the stray-road sweep, the gather park
  (COLLISION §15), and the graft table with the Senate's patriot (706).
- **East Indies 13640 → 15985 in two**: animation names case-folded (643,
  ANIM §12), then 706's graft and patriot. The word is past run78 (708).
- **Chapter six-b closed** at 1250 (651, 680): a captain's attack on a
  building re-runs `find_new_target` every frame (COMBAT §62).
- **The issuer chapters**: nine (move) and ten (patrol, ORDERS §27)
  closed, and the census's order row moved for the first time, 44 → 47
  cited of 410. Eleven, the guard, is open at 1139: four collision arms,
  an attack leashed to its post (COMBAT §63), and vision's resync (§10).
- **Fable backlog: 6 Loop items** (527, 677, 685, 687 a capture's cost,
  692 the census's row, 697 `cover=1` hangs); no `FABLE:` marker.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w15985 of 24,000 · GreatLakes w15175 of 24,000
Golden: ch11 w1139 of 1250 · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed
Endpoint 24001: EastIndies 47 off, 7 unlinked · GreatLakes 47 off, 0 unlinked

**Opener: 713 is live on the rules lane and 711 on the AI lane (Great
Lakes is the lower map); 708 follows 711. A detached capture waits on
`tools/gamelog/waitrun.sh`; a report quotes the gate's `Gate steps:` line.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — Great Lakes (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

711. **Great Lakes' word is 15175: the free Longbowmen's guard posts** (706
    moved it 14982 → 15175: `Tribe::graft` and the Senate's patriot, TECH).
    On 15175 ours 4 draws against 3 at index 2: an extra `Guy::set_anim
    +0x97a < Unit::do_guard+0x7f4`. Inside run196, widened (block 15176).
    Under it on 15095 the three free Longbowmen take their guard posts
    the other way round, `1/77` and `1/78` swapped. No mechanism.

708. **East Indies' word is 15985, past every capture** (706 moved it 15782
    → 15985 and closed 694: `1/60` is The Senator). On 15985 ours 5 draws
    against 6 at index 4: ours `Unit::think_scout+0x941`, theirs
    `Leader::make_stuff+0x63d`. run78 ends on 15900. The `WIDENINGS` row
    names this item: capture over the word, sized to it, and widen whole.
    No mechanism.

713. **Chapter eleven's word is 1139: the guard's idle stand** (709 moved
    it 1133 → 1139: the hundredth-frame resync and the whole-disc relight
    of `visible`, and a building seen through `ever_seen`, VISION §10). On
    1139 ours 4 draws against 5 at index 0: the original's guard rolls
    `Guy::set_anim+0x97a < Unit::do_guard+0x7f4` and ours does not, the
    site 711's word spends extra. `GUYS=2` prints no animation state: a
    packet at logger 1139 (run200). No mechanism.

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
