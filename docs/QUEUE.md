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
(`docs/audit/2026-09-25-fable-pass-14.md`, DECISIONS 50): **12 landings
since 7958738**, eight words and four chapters. Lane att-767 (East Indies) is live.*

- **Great Lakes 15384 → 17099** in four: `num_wonders` counts a site
  (722), `take_damage` stamps `frame_attacked` (729), the siege sub-group's
  own record (736, GROUPS §26), and `move_step`'s give-up takes no step
  (742, COLLISION §17; run226 to 17350). **East Indies is the lower map.**
- **East Indies 15985 → 16982** in two: a republic raises the commerce
  cap (708, AI §72), and the census counts a gatherer in its building's
  city (752, AI §73; run227 captured [16230, 16935)).
- **Chapters fourteen to sixteen closed** (723, 731, 738), **seventeen
  open, 642 → 821** (746, 759, 763; §25, ORDERS §33–§34): the flight home,
  the landing and the pair's patrol are built. Order row 50 → 57.
- **Ruled at the pass**: the compared recorder (`diff::compared`, a
  parsed field nobody compares fails until pinned); runway 250 blocks
  (687); `waitrun.sh` reads the click-free receipts; a brief reserves the
  code module; the chain deletes the lane's remote branch.
- **Fable backlog: 16 Loop items** (677, 685, 697, 727 the remote sweep
  is the user's, 730, 735, 737, 745, 751, 755, 756, 758, 762, 766, 769, 772).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w16982 of 24,000 · GreatLakes w17099 of 24,000
Golden: ch17 w821 of 1399 · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · 770 next
Endpoint 24001: EastIndies 48 off, 0 unlinked · GreatLakes 44 off, 0 unlinked

**Opener: 767 is live on the AI lane (East Indies is lower), 757 after
it; 770 on the rules lane; landings count from 7958738. A capture carries
250 blocks of runway and is waited on with `waitrun.sh`; a report quotes
`Gate steps:`; a journal's "for the Loop" line is filed at its merge.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — East Indies (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

767. **East Indies' word is 16982, past every capture** (752 moved it
    16683 → 16982: the census counts a gatherer in its building's city, AI
    §73). On 16982 ours 10 draws against 15 at index 4: theirs
    `Leader::make_stuff+0x63d`, ours `Animal::do_idle+0x83`. run227 ends
    on 16934; on it who=1's `MAKE[1].val` is 22784 here, 91136 there (×4)
    from 16779. `WIDENINGS` names this item: capture over the word.

757. **Great Lakes' word is 17099** (742 moved it 16460 → 17099:
    `move_step`'s give-up takes no step, COLLISION §17). On 17099 ours 217
    draws against 225 at index 0: theirs `Guy::set_anim+0x97a <
    Unit::move_step+0x823`, ours `< Unit::do_move+0x11cf`. Under it on
    17086 who=1's `resource_cap` 4800 there, 4000 here, and the gatherers
    re-dealt. Inside run226, widened (17098..17102). No mechanism.

770. **Chapter seventeen's word is 821: the strafe's round** (763 moved it
    805 → 821: the pair's patrol, ORDERS §34). On 821 ours 4 draws against
    5 at index 0: theirs `Object::take_damage+0xe1 < Object::do_damage <
    Ammo::do_damage`. On 822 the Barracks' damage is 45 there, 0 here.
    763 read `guy_release_events` (`anim.rs`) firing no round under a
    `STRAFE` front order, as a hypothesis. Inside run223, widened.

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
