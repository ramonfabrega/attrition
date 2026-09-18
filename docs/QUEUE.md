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

*2026-09-17 into 09-18, an Opus commander loop: eighteen landings, twelve
word-moving. **GL +703, EI +683.** Next: 344. The capture lane closed.*

- **Great Lakes 8374 → 8382** (item 340), still the same scholar 338
  seated. `Guy::init_real` sets `guy_flags & 0x80` for type 0x34/0x35, and
  `set_anim`'s arm on that bit makes the idle variant an **offset** —
  `+0x19` for the head of the host's inside chain, `+0x1d` below it. Plus
  `is_peasant` is `{0x32,0x33}`, not "any worker", so the scholar was
  taking the peasant collapse. Only both together land on the word.
- **Both maps now have a value diff at their word, with runway** — the
  lane's three captures, 1.2 GiB, all 0 differing against their long runs.
  **run97** GL `[8030, 9349]`, 1,320 blocks no gap, **979 past the word**;
  **run98** EI `[7880, 8789]`, truncated, successor owed; **run96** EI's
  first late census. run98 was 340's oracle within the hour.
- **339 bit exactly as parked, and the brief caught it.** 340's workspace
  gate stopped at the red rondata binary and never ran sim; told to run it
  separately, it got **831 passed** with `no_float`, `soak` and
  `docs_guard` green. Until 339 lands, every brief must say so.
- Two items open, thirty parked. **Fable backlog: eleven Loop items**
  (313, 318, 321, 330, 331, 332, 335, 339, 341, 343, 345). 318+339 are one
  question; 341+343 are one; 345 says the commander's own verbs belong in
  `CLAUDE.md` rather than invisible memory.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w8495 of 24,000 · GreatLakes w8382 of 24,000
Endpoint 24001: EastIndies 61 off, 11 unlinked · GreatLakes 62 off, 15 unlinked

**Opener: 344 — 340 named it on the frame, so this is a strict successor,
not a widening: 46 draws against 865 at `Leader::make_stuff` →
`produce_building`.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable; the
backlog is `docs/PARKED.md`, and an item comes back from it only when a
score names it.

344. **Great Lakes 8382 is a build-order frame, and 340 named it** — no
    widening owed. 46 draws against the original's **865**:
    `Leader::make_stuff` then `Leader::produce_building`, where the
    original leaves `+0xc99` after **twelve** draws for `+0x1805` and this
    crate spends **twenty-one** at `+0xc99` first. A count that large is a
    loop bound or an exit predicate, not an arithmetic slip. Build it, then
    ratify (`DECISIONS` 23).

342. **Host choice seats the scholar on the wrong university** — 338's
    residue, exact: four scholars off by precisely `1/2020 − 1/2019` at
    run80's 24001, six of eleven already agreeing. 340's widening did not
    land on it. A predicate, not a formula; book after 344 unless 344 names
    it.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you are touching, never another
  author's item, and never widen a line to beat a count.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names the headline's frame, a floor, or
  a takes-chain to one. **The headline slot is never empty**: no item on
  the word's frame means the next brief is the widening of that frame.
  Loop items — tooling, guards, these rules — go to the parked file's Loop
  section and are the steering pass's, never a worker's.
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes the
  `Scoreboard:` line; the worker's story is `docs/journal/<date>-item-<N>.md`.
- **A number is measured on the tip** — after the worker's last `ccc
  update`, or the report says which tree. 301's +381 was attributed to the
  wrong half against a stale base and would have landed wired.
- **Merge, gate, push, reap — one chain.** A reap left for "before the
  next spawn" was forgotten twice in one session.
- **A floor that moves** moves `FLOORS`, the assert reading it, and the
  `Scoreboard:` line together; the guard parses the line against the floors.
- **Overlap a capture's neighbours on purpose.** Six blocks each end is the
  floor (run83); a containing neighbour is free with `--exclude <the
  category you raised>`; `samegame.py` exits 0 when nothing is in common,
  so assert the count, not the verdict.
- **Grep for the derived quantity, the SHAPE and the GATE — not only for
  coverage of a frame.** A term zero in every dump is either zero-valued or
  switched off: 117 and 178 were the second.
- **A check on a never-cleared field asserts a CHANGE, not a value** —
  `collide_frame` is a permanent stamp (run85). Test both directions first.
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A commander's clear is
  free when every landed branch is merged, gated, pushed and reaped,
  nothing is in flight, and this handoff is current with the headline
  measured. **Before a blind fan-out**, grep `CLAUDE.md` and the memory
  index: a subagent inherits both, and no brief quotes this file.
