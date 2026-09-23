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

*2026-09-23, the twelfth Fable pass (Fable 5.1, the main thread): **20
landings, 17 moved a word, and the count reached the pass at twenty.**
East Indies **9711 → 11069** at 0.19 USD a frame, still the lower map;
the rules track closed chapters four (1500), seven (1200) and three
(900), and the restage stands at 782. Great Lakes held at 12038.*

- **Eight Loop items ruled** (DECISIONS 48): a capture booked on a map
  cites the ledger's window — a guard, made to fail first both ways on
  571; a falsifier names where it fires (GOLDEN §3, point 5) and
  **chapter seven-b is 628**; the one-line literal reads; the ladder
  floor is a pin; a packet is taken at the word's own logger frame.
- **598 rides in 620**: `inside_up`'s container row goes into `compare`
  inside the widening. 571 never ran — both lanes held a headline.
- **Fable backlog: 3 Loop items** (313, 527, 630); the `FABLE:` batch was empty.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w11590 of 24,000 · GreatLakes w12038 of 24,000
Golden: ch1 closed · ch2 closed · ch3 closed · ch4 closed · ch5 closed · ch7 closed · restage w792 of 1000 · 628 next
Endpoint 24001: EastIndies 52 off, 6 unlinked · GreatLakes 45 off, 0 unlinked

**Opener: 620 landed (East Indies 11069 → 11590) and 625 (the restage
782 → 792). 628 has the rules lane and 629 the AI lane; 617 follows 628,
and 571 follows 629. Two landings since the twelfth pass's commit; stop
at twenty.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden word for the rules, the long word for the AI, and
lower map first — East Indies (a guard reads this line). Take the first
unstarted on either track unless a better order is obvious, and say so; the
backlog is `docs/PARKED.md`, and an item returns only when a score names it.

629. **East Indies' word is 11590, past every capture** (620 moved it
    11069 → 11590 by capping a Mine's `dist_mod` at 3 on a gather list
    under `MTN_TINY_SIZE`, ORDERS §6.4; parked 622's tile from 10959). On
    11590 ours 6 draws against 5, parting at index 0: ours
    `Guy::set_anim+0x97a < Animal::do_idle+0x19`, theirs `Guy::set_anim+0x97a
    < Guy::inc_time+0x271`. run155 ends at 11279, so the `WIDENINGS` row
    names this item: capture **run159** over the word, overlapping
    run155, then widen it whole. No mechanism.

571. **Great Lakes: Army 1's squad stops on block 11922 in the original
    and walks on here** (566 moved the word 11903 → 12038 by carrying the
    pathfinder's validity memo across searches, PATHFINDER §24). Nine
    figures, `1/37`–`1/42` and `1/62`–`1/64`: `stopped 1`, speed 0 against
    walking; positions agree, no draw; 29 rows pinned in run136's
    widening. Then 12038's capture and widening past 11959: ours 4 draws
    against 5, `move_step+0x823`. No mechanism. Optional second run if a
    lane is free: a packet at 11922 with the nine guys' move path on it.

628. **Chapter seven-b: the computer's civilians under the cheat** (the
    twelfth pass; GOLDEN §11's restage, and §3 point 5 names where each
    falsifier fires). The same five for who=1, `!ai off` and the control
    — run156 and run157, `[605, 1200)`, chapter seven's dump set — where
    `Unit::think`'s block decides, and where this crate's one-term seam
    (INPUT §11.9, `orders.rs`) has its frame. A falsifier that fires
    stops the item; otherwise the pair pins two words or closes. Takes
    the rules lane after 625.

617. **Chapter three's restage word is 792: the arena-A hoplites attack
    three frames late** (625 moved it 782 → 792 with `Guy::move`'s walk
    gate for an unpacked packer, COMBAT §58). On 792 ours 28 draws against
    29 at draw 24: the original's hoplites take their attack on the
    catapult and one spends `Unit::fight+0x9b0`; ours take it on 795. The
    idle's phase is `(frame + o) & 15`, and ours number them 6–8 against
    the dump's 9–11. Hypothesis: COMBAT §42.5's `DEATH_OBJS` cull on 771
    (parked 617). No mechanism past the phase. Takes the rules lane after 628.

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
  takes a packet at the word's own frame, not a detail capture** (EMULATOR §8).
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in
  every dump is zero-valued or switched off (117, 178); **a never-cleared
  field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A clear is free when
  every landed branch is merged, gated, pushed and reaped and none is in flight.
