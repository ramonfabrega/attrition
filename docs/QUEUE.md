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

*2026-09-25, the fifteenth Fable pass (Fable 5.1): **the count restarts at
96136251, this pass's commit** (`docs/audit/2026-09-25-fable-pass-15.md`,
DECISIONS 51; the gate green on 9bd334e8). Since it: 800, 811, 822, 829, 837, 839,
850 (East Indies 17403 → 19509), 803, 813, 824, 832, 836, 842, 853 (ch20
to ch22 closed); 857 in flight.*

- **Great Lakes 20568 of 24,000; East Indies 19509 and the lower map.**
  The AI lane's frame cost fell 0.055 → 0.038 (eight packets replaced by
  a grep); the rules lane rose to 48.0 a landing on three stalls.
- **Landed by the pass, each made to fail first**: the execute-bit guard
  (756; 751 with it), `RON_LANE_WAIT` (758), the pool receipt (735),
  `--stall-seconds` (762), `frame.py`'s indentation (755), the
  comment-blind constant guard (802; 72 → 104, the arrivals are 809).
  The remote swept (727). **The brief checklist** is in
  `docs/audit/README.md`; a brief is composed against it.
- **For the next pass**: the value diff beside every word (eight of
  twenty reported a row count); the checklist's rows in the briefs;
  `UNBUILT` at 104; the order row at 60.
- **Fable backlog: 22 Loop items** (677, 685, 697, 745, 775, 799, 810, 820, 821, 823, 828, 830, 834, 835, 838, 841, 845, 846, 852, 856, 860, 864).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w19509 of 24,000 · GreatLakes w20568 of 24,000
Golden: every chapter closed · ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch6 closed · ch7 closed · ch8 closed · restage closed · ch7b closed · ch7b-control closed · ch6b closed · ch9 closed · ch10 closed · ch11 closed · ch12 closed · ch13 closed · ch14 closed · ch15 closed · ch16 closed · ch17 closed · ch18 closed · ch19 closed · ch20 closed · ch21 closed · ch22 closed · 854 next
Endpoint 24001: EastIndies 38 off, 2 unlinked · GreatLakes 11 off, 0 unlinked

**Opener: 857 on the AI lane (East Indies is lower), 795 then 831 after
it, and 854 on the rules lane; the briefs against the checklist. A capture is
sized to its word — six blocks before, 250 after — and waited on with
`tools/gamelog/waitrun.sh`; a word lands with the field and both sides'
values, never a row count.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — East Indies (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

857. **East Indies' word is 19509** (850 moved it 19413 → 19509: a
    unit that goes inside in its own work takes `Guy::process` that
    frame, ANIM §15). It is `1/71`'s walk start; its first parting is on
    19499, `collide` 0 against 11 and `collide_o` −1 against 75. Inside
    run269, widened. No mechanism.

795. **Great Lakes' word is 20568** (785 moved it 17181 → 20568:
    `Wonders::init_wonder` raises `wonder_mark`, AI §75). On 20568 ours 37
    draws against 38 at index 31: the original's is `1/40`'s blocked step
    by `8/0`. `1/40` is already off on 20500, run243's first block (the
    gap, 796). Inside run243 ([20500, 20819)), widened. No mechanism.

831. **Twelve long floors: `do_non_flat_gather`'s `group` write** (824
    measured it: `Unit::do_non_flat_gather@005f0170` writes `group` −1
    on every call, AI included, `5f023e`). Both words and `ENDPOINTS`
    hold; twelve floors fall (run257 320 → 306, run227 293 → 283), and
    pool slots renumber (run78 `1/60` 69 → 68 against 70). Journal
    824's table. The AI lane's, after its word; its pins are that lane's.

854. **Chapter twenty-two's floor: the landed patrol** (842). `0/6` on
    1385 and `0/7` on 1489: `land_plane` keeps the order under a home
    whose `build_masks & 0x80` is set (`WallData::has_repeat_air@00472410`;
    run265's Airbase reads 4232), and `Building` carries no
    `build_masks` (`lib.rs`); the bit's writer is unread.

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
