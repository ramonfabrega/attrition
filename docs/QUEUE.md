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
(`docs/audit/2026-09-25-fable-pass-14.md`, DECISIONS 50): **16 landings
since 7958738**, eleven words and five chapters. Lane att-779 (chapter eighteen) is live.*

- **Great Lakes 15384 → 17181** in six: `num_wonders` (722),
  `frame_attacked` (729), the siege sub-group (736), `move_step`'s give-up
  (742; run226 to 17350), the Pyramids (757) and `invalid_loc`'s cell arm
  (776, PATHFINDER §27). **Great Lakes is the lower map.**
- **East Indies 15985 → 17189** in three: the republic's commerce cap
  (708), a gatherer counted in its building's city (752), and the British
  Taxation discount (767, AI §74; run233 captured [16929, 17441)).
- **Every golden chapter closed again**: fourteen to sixteen (723, 731,
  738), and seventeen 642 → 1400 in four (746, 759, 763, 770): the flight,
  the landing, the patrol and the bomb (§25, ORDERS §33–§35). Row 50 → 57.
- **Ruled at the pass**: the compared recorder (`diff::compared`, a
  parsed field nobody compares fails until pinned); runway 250 blocks
  (687); `waitrun.sh` reads the click-free receipts; a brief reserves the
  code module; the chain deletes the lane's remote branch.
- **Fable backlog: 21 Loop items** (677, 685, 697, 727 is the user's
  sweep, 730–789 filed from this tranche's journals).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w17189 of 24,000 · GreatLakes w17181 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · 779 next
Endpoint 24001: EastIndies 47 off, 3 unlinked · GreatLakes 43 off, 0 unlinked

**Opener: 785 on the AI lane (Great Lakes is lower), 773 after it; 779
is live on the rules lane; landings count from 7958738. A capture carries
250 blocks of runway and is waited on with `waitrun.sh`; a report quotes
`Gate steps:`; a journal's "for the Loop" line is filed at its merge.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — Great Lakes (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

785. **Great Lakes' word is 17181: who=1's wonder offers** (776 moved it
    17128 → 17181: `invalid_loc`'s cell arm under `valid_wcoord`,
    PATHFINDER §27). On 17181 ours 11 draws against 5 at index 0: three
    `Leader::create_buildings+0xffb`/`+0x1017` pairs. On 17182 who=1's
    `MAKE` list offers wonders 526/528/527 here, none there. Inside run226,
    widened. 777 (the Pyramids' readers) may be its family. No mechanism.

773. **East Indies' word is 17189** (767 moved it 16982 → 17189: the
    British Taxation discount in `get_cost`, AI §74). On 17189 ours spends
    a blocked stand (`move_step+0x823`, `1/55` by `1/60`) the original
    spends on 17190. Under it `1/55`'s `half_step` from 17182, `1/57`'s
    from 17161, `1/58`'s path from 17147, who=1's `scholars` from 16971.
    Inside run233, widened. No mechanism.

779. **Chapter eighteen, the build line — an issuer, no capture yet**
    (DECISIONS 49; GOLDEN §13's next `—` row; 770 closed chapter
    seventeen at 1400). `CommandManager::issue_build@00941c30` on a citizen
    and on a group of citizens from the tracer DLL, a building placed and
    built, 770's harness. It should enter `BuildOrder`. The issuer under
    the emulator first. Takes GOLDEN §26; run241 at `GUYS=4`, `cover=0`.

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
