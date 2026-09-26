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

*2026-09-26, the commander (Opus 5.5): **twenty landings since 96136251,
by the log; the sixteenth Fable pass is due.** Nothing is in flight; no
lane was spawned past the twentieth.*

- **East Indies 17403 → 20007** in eight (800, 811, 822, 829, 837, 839,
  850, 857: GROUPS §27, ARMY §21–§22, AI §76, ANIM §14–§15, ECONOMY §16,
  COLLISION §18); 865 and 870 named 20007's cause (GROUPS §28–§29), and
  the pool agrees to 20257. **Great Lakes held at 20568**; 795 was never
  spawned, East Indies being the lower map throughout.
- **Chapters twenty to twenty-four closed** (803, 813/824, 836/842 with
  853/854's floors, 867, 877), 832 a reading; GOLDEN §13 has no
  `unresolved` row left; the order row 60 → 64.
- **For the pass**: nine one-function grants across the lanes, none
  collided; four commander bookings failed a guard (the handoff's length,
  846, a capture's citation, 831's ledger); `viadriver.sh`'s silent drop
  (810); the decimal false-bank (820); a journal's stray tags (841).
- **Fable backlog: 28 Loop items** (677, 685, 697, 745, 775, 799, 810,
  820, 821, 823, 828, 830, 834, 835, 838, 841, 845, 846, 852, 856, 860,
  864, 866, 869, 875, 879, 881, 885).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w20007 of 24,000 · GreatLakes w20568 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · ch23 closed · ch24 closed · 882 next
Endpoint 24001: EastIndies 40 off, 2 unlinked · GreatLakes 11 off, 0 unlinked

**Opener: the sixteenth Fable pass (twenty landings since 96136251).
After it, 880 on the AI lane (East Indies is lower), 795 after it, and
882 on the rules lane. A capture is sized to its word and waited on with
`tools/gamelog/waitrun.sh`; a journal's "for the Loop" line is filed at
its merge.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — East Indies (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

880. **East Indies' word is 20007: the slot records' history** (870:
    every `group` pointer agrees to 20257, 831 built; 30e5b464's kept
    fields still drop the words to 17530/7213). run277 holds the word;
    the slot records' `order_num` agree to 6215 and part by 15894, and
    no dump prints `GROUPDATA` in 6216..15893. A `GROUPS=1` capture over
    that gap, walked in record mode; then the kept fields (GROUPS §29.4).

795. **Great Lakes' word is 20568** (785 moved it 17181 → 20568:
    `Wonders::init_wonder` raises `wonder_mark`, AI §75). On 20568 ours 37
    draws against 38 at index 31: the original's is `1/40`'s blocked step
    by `8/0`. `1/40` is already off on 20500, run243's first block (the
    gap, 796). Inside run243 ([20500, 20819)), widened. No mechanism.

882. **Chapter twenty-four's floor: a trained squad's pool push** (877
    closed the chapter at 1560, GOLDEN §33). 30 of its 34 standing rows
    are `Unit::come_out`'s human push of a trained squad into the pool:
    `group`, `form` 0, and the followers' `orders_x/y`. In `group.rs`,
    the pool's module; 689's family, beside 880. Inside run285.

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
