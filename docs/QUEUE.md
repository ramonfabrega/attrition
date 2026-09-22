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

*2026-09-22, the tenth chain. Great Lakes **10234 → 10237**, chapter
two **645 → 683**, and `[606, 684)` is at nought — 78 frames.*

- **The defect was a brace** (481, COMBAT §38): `do_move:207` is
  `if (ptype->max_range != 0) {` and it closes **past** the captain
  retarget at `005f803f`. §37.2 said so in prose; the Rust wrote the
  gate as a conjunct of the in-range kill alone, so a **melee**
  captain reached the retarget and switched the hoplite squad off
  `0/11`. Nesting removes exactly six rows — which is the proof.
- **479's framing for 671 was falsified by a field already on disk**:
  all three hoplites hold `ox 11 whom 0 uid 18` on every block with
  `new_ord 1`, so the original never ranks them and never rewrites.
  The suspect was our own code from the first grep.
- **`crate::diff::compare` carries no hit-point row at all** — no
  `myhits`, `damage` or `hits_left` — where the run100 widening has
  had both since §34.4 and the dump prints them at every detail
  level. 684's death is visible; the wounds that caused it are not.
  **That is 484, and it gates 485.** Seventh instrument defect.
- **A ceiling hid three rows**, as 470's floor hid `0/10` at 630:
  `extra 1/8` at 684 (a death), `order 1/4`/`pos 1/4` at 685/686,
  all pre-existing on a reverted re-measure.
  Seven open, 69 parked. **Fable backlog: 2 Loop items** (313, 480).

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w9711 of 24,000 · GreatLakes w10237 of 24,000
Golden: w626 of 901 (ch1) · ch2 w683 · 484 next
Endpoint 24001: EastIndies 63 off, 11 unlinked · GreatLakes 51 off, 7 unlinked

**Opener: 483 is live; 484 takes the freed lane and gates 485. The
loop stands at 16 of 20.**

## The queue

In dependency order, headline-nearest first. **Two headlines** (DECISIONS
41): the golden record's word for the rules, the long captures' word for
the AI, lower map first — Great Lakes. Take the first unstarted on either
track unless a better order is obvious, and say so. Numbers are stable;
the backlog is `docs/PARKED.md`, and an item returns only when a score names it.

483. **`Ammo::do_damage@00678060+0xc59` is unnamed in
    `trace::SITES`** — the AI headline's own blocker. Great Lakes'
    count word is 10244 and its sequence word 10237; 10237 and 10242
    spend the **same** draws on both sides and part on a label, ours
    an unattributed `projectiles` phase. **Until the site is named
    the sequence cannot pass 10237 however the simulation behaves.**
    A `SITES` entry plus a `SITE_*` const in `sim::fight`. Reported by
    478, not taken. COMBAT §39 reserved.

476. **f10234's three value rows**: `0/5 order:length` 2/1,
    `0/5 orders.len` 2/1, `0/2001 gather:gather_down[-1]` 5/2. The
    word's own frame, value side. ORDERS §18.3.

477. **f10235's `1/28 pos`**, ours (4801,30175) theirs (4800,30175),
    with `g.x[0]` and `g.des_x[0]` the same — one world unit in x,
    said three times. ORDERS §18.3.

484. **`crate::diff::compare` has no hit-point row** — add
    `myhits`, `damage` and `hits_left`, the pair the dump prints at
    every detail level and that `run100_s_word_block_…` has carried
    since §34.4. It touches `FrameResult`/`compare` and therefore
    **every capture's diff**, so measure before and after on both
    maps and expect counts to move. **Gates 485**, which cannot name
    a mechanism without it. Reported by 481, not taken.

485. **`extra 1/8` at 684 — a death** (the rules headline's next
    frame; 481 moved the word to 683). `DEATH_OBJS` on 684,
    `first_frame 683`; `1/8`'s `damage` runs 0 → 8 (656) → 17 (657)
    → 25 (660) → 34 (682) against a figure's 40-hit share of
    `myhits 120`, and the hit on 683 takes it over. **Do not start
    before 484 lands** — without the hit-point row the diff can see
    the death and not the wounds.

342. **Host choice seats the scholar on the wrong university** — 338's
    residue, re-pinned three times, and now `1/55`. **Re-measure before
    diagnosing**: the vector `(768, 9984)` is the claim, never the count.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items — their story is in
  `docs/journal/`. Shorten only what you are touching, never another
  author's item, and never widen a line to beat a count.
- **A residue item is booked by its frame and its draw delta** (DECISIONS
  42); a mechanism in its title is the previous item's hypothesis.
- **A finding parks by default.** `docs/PARKED.md` takes what names no
  score; this file takes only what names a headline's frame, a floor, or
  a takes-chain to one. **Neither headline slot is ever empty**: no item
  on the AI word's frame means the widening of that frame; no rules item
  means the next unpinned chapter of the golden record. Loop items —
  tooling, guards, these rules — are the steering pass's, never a worker's.
- **A worker never books a number and never edits this file or
  `docs/JOURNAL.md`.** It reports; the commander books and writes these
  lines; the story is `docs/journal/<date>-item-<N>.md`. **A pinned
  constant is the worker's to re-pin; the line is the commander's.** The
  chain — merge, book, gate, push, reap — is `CLAUDE.md`'s. **A number is
  measured on the tip**, after the last `ccc update`, or names its tree.
- **The floors and these lines move together** — `FLOORS`, `LONG_WORD_*`,
  `GOLDEN_WORD_*`, `ENDPOINTS` with `Scoreboard:`, `Long captures:`,
  `Golden:`, `Endpoint <frame>:`, `Fable backlog: N Loop items` — read
  literally and **never wrapped**, or the count guard matches this line
  instead (twice, 09-19). The length guard counts every line to the next `## `.
- **A capture is a draw-stream trace first, detail on demand** (DECISIONS
  41): whole length at `cover=0`, then a windowed re-run sized to the word;
  after 363, the click-free lane unless it needs the mouse. Overlap the
  neighbours: six blocks each end (run83); `samegame.py` exits 0 when
  nothing is in common, so assert the count, not the verdict.
- **Grep for the derived quantity, the SHAPE and the GATE**: a term zero in
  every dump is either zero-valued or switched off (117, 178). **A
  never-cleared field asserts a CHANGE, not a value** (`collide_frame`, run85).
- **The gate is `python3 tools/release_gate.py <install> --test-threads 4`**,
  to a file, never piped — a pipe launders the 137. A commander's clear is
  free when every landed branch is merged, gated, pushed and reaped,
  nothing is in flight, and both headlines are measured. **Before a blind
  fan-out**, grep `CLAUDE.md` and the memory index: a subagent inherits both.
