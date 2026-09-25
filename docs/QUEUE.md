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

*2026-09-25, the commander (Opus 5.5) after the fourteenth Fable pass
(`docs/audit/2026-09-25-fable-pass-14.md`, DECISIONS 50): **7 landings
since 7958738**, four words and three chapters. Lane att-746 (chapter seventeen) is live.*

- **Great Lakes 15384 → 16460** in three: `num_wonders` counts a site
  (722, AI §70), `take_damage` stamps `frame_attacked` (729, AI §71), and
  the siege sub-group lays out on its own cleared record (736, GROUPS §26;
  run218 to 16711). **Great Lakes is the lower map again.**
- **East Indies 15985 → 16683** (708, AI §72): a republic raises the
  commerce cap, +50 a capped good; run221 captured [15894, 16237).
- **Chapters fourteen to sixteen closed** (723, 731, 738; GOLDEN
  §22–§24): no issuer builds a `FormOrder` or a `GroupAttackOrder`; a
  copied plain move replays to its `orig`. The census's order row 50 → 55.
- **Ruled at the pass**: the compared recorder (`diff::compared`, a
  parsed field nobody compares fails until pinned); runway 250 blocks
  (687); `waitrun.sh` reads the click-free receipts; a brief reserves the
  code module; the chain deletes the lane's remote branch.
- **Fable backlog: 10 Loop items** (677, 685, 697, 727 the remote sweep
  is the user's, 730, 735, 737, 745, 751, 755 `leader.py` on a slice).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w16683 of 24,000 · GreatLakes w16460 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · 746 next
Endpoint 24001: EastIndies 50 off, 0 unlinked · GreatLakes 44 off, 0 unlinked

**Opener: 742 on the AI lane (Great Lakes is lower), 752 after it; 746
is live on the rules lane; landings count from 7958738. A capture carries
250 blocks of runway and is waited on with `waitrun.sh`; a report quotes
`Gate steps:`; a journal's "for the Loop" line is filed at its merge.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — Great Lakes (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

742. **Great Lakes' word is 16460: `1/23`'s blocked step** (736 moved it
    15619 → 16460: the siege sub-group's own `Group::clear` record, GROUPS
    §26). On 16460 ours 1 draw against 3 at index 1: theirs a blocked
    `Unit::move_step+0x823`. On 16460 `1/23` stands stopped there and
    walks here; on 16461 the original's is blocked by The Despot
    (`collide_o 79`). Inside run218, widened (16459..16463). No mechanism.

752. **East Indies' word is 16683, past every capture** (708 moved it
    15985 → 16683: a republic raises the commerce cap, AI §72). On 16683
    ours 6 draws against 7 at index 5: theirs six `Guy::set_anim+0x97a <
    Guy::inc_time+0x271` wraps before `Farms::inc_time+0x1ae`, ours five
    (`1/18` twice, `1/19`, `1/20`, `1/54`). run221 ends on 16236. The
    `WIDENINGS` row names this item: capture over the word, widen whole.

746. **Chapter seventeen, the flight line — an issuer, no capture yet**
    (DECISIONS 49; GOLDEN §13's next `—` row; 738 closed chapter sixteen
    at 1250, the census's order row 52 → 55).
    `CommandManager::issue_flight@00941d40` on an aircraft from the tracer
    DLL, 738's harness and chapter six's airbase staging. It should enter
    `AirOrder`. The issuer under the emulator first; the premise names its
    killer. Takes GOLDEN §25; run223 at `GUYS=4`, `cover=0`.

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
