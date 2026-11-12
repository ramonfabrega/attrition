# The pass of 2026-11-12: the render track's next step, designed from the client's own picture

*2026-11-12 05:30 to about 06:45 −05:00, the `steer` row on Fable 5.2, armed by
`commander` on supply (`docs/DECISIONS.md` entry 126 §4) with the tree banked
at `3d7cf1a94`. The charter: design two or three render items with scores
that can fail, rule on the two frame rows lanes 4165 and 4168 named, leave
step III of item 4159 as designed. Decisions in entry 127. The pass before
it is entry 126's; the last with a record here is
`2026-11-09-fable-pass-76.md`.*

## For Ramon

- **The renderer's scores were not measuring the picture.** Every
  photograph row is counted on a CPU copy of the client's drawing, never
  on the client's own frame, and compares hue only. The rows read whole or
  nearly so while the client's frame, set beside the original's, shows
  figures washed in team colour, flat green trees, a mountain a third as
  bright and a lake that tiles.
- **What changed**: five render items booked (4171 to 4175), the first two
  being the instrument — the rows counted on the client's still, and a
  brightness reader. The colour row's chase of one tank is stopped.
- **The tranche**: nine lane landings, 75.21 USD at list, 8.36 a landing;
  the window read 5 % at 10:29Z, six and a half hours after the reset and
  48 before it closes: about nine points in twelve hours, against the
  twenty entry 126 §2 planned on. Three workers are not spending it.
- **One tool built**: a booking gate no longer runs every timed test for a
  merge that touches only the client crate or the painter.
- **Yours, unchanged**: the 163 GB of decoded caches (`--gc-oracle`), the
  merged `agent-*` trees, and whether a fourth worker stands. On the
  window's reading a fourth is affordable; the host's gate queue is what
  binds, and the gate change above is aimed at it.
- **Not designed here**: the lobby screen and the client's input past
  item 4149. They are the next undesigned step after the leaders line.

## What was measured

| what | measured |
|---|---|
| the tranche (`tranche.py --base 6bebf0788`, after `lore index`) | 9 lane landings, 75.21 USD over 9 sessions, 8.36 a landing; mean session 50.8 minutes clock, 23.7 working, 27.2 waiting (gate 17.8); two deep (4165 at 427 k, 4166 at 326 k); 4 takes of the original over 9 items. Items 4154, 4162 and 4167 were in-session builders and have no lane session |
| the gates (`gates.py ~/Library/Caches/attrition/gates`) | 14 gates, 12 green and 2 red at their second step and re-run; **every green one ran every timed test**, the eight render bookings among them, each `scoped` and widened by a path under `crates/`; 262.2 minutes; admission 13,357 s summed, longest 1,005.7 s (4164's) |
| the window (`lore limits`) | the whole week 5 % at 10:29Z, the Fable week 3 % at 09:10Z, both closing 2026-11-14 06:00 −05:00 |
| the rubric (`rubric.py --record`) | the territory 669 · 336 · 1066 from 550 · 318 · 1285 (item 4039); lockstep bytes 78.1 %, ages 8 of 8, unit types 296 of 302, all unmoved |
| the first acts | 27 GB free at the start (the handoff said 38); `cargo clean --profile dev` 24,184 files and 15.0 GiB, 35 after; `afsctool` over both archives, 853,791 items, stopped through its own signal handler at 435,375 (51 %) after 55 minutes so the pass's gate would not run beside it, 42 GB free after; the rest is the next pass's |

## What the pass looked at

The client's still, `client --frame 3279 --still`, from the gated binary at
`3d7cf1a94`, at run1559b's pinned camera and at run1564's (`--look
36960,38496`), each set beside its photograph. The stills and the two crops
are under `~/ron-data/lab-experiments/2026-11-12-pass-render/`, outside the
repository, with `meas.py`, the scratch that took the boxes in entry 127's
table. The boxes are by eye on the 1920-wide frames; they are evidence that
a row is owed, not a row.

By eye, on run1559b's frame: every figure is its owner's colour with a
trace of texture under it; the trees are flat pale green; the mountain is
dark brown where the photograph's is pale rock; the city stands on bare
grass where the photograph has its paved base, smoke, fire and a label;
the original's muzzle flashes, dust under the tanks and the whole interface
are absent, as designed for now. On run1564's: the lake is the floor's
texture tiling in cyan where the photograph is deep blue with foam; the
beach is a hard sawtooth a tile wide where the photograph's fades into
grass; the rock patch and a border line at the right edge are not drawn.

How the figures come to be washed was read, not built: the client's main
pass draws the textured figures, then the same meshes under the silhouette
pipeline at depth Greater with no depth write. A closed mesh has a far side
behind its own near side, so the second draw passes on the figure itself.
`docs/RENDERER.md` §5.3's reading has the original draw the silhouette
state before the lit one. The client's only test of the pass asserts the
shader's text. This is item 4171's first hypothesis.

## Rulings

- **A render row is the client's pixels, and it reads brightness** (entry
  127 §1, §2). Items 4171 and 4172.
- **The colour row rests at 18 of 33** (§3). Rows 4165a, 4165b and 4168b
  stay in their journals. The lit figure is drawn when a brightness count
  says so.
- **The two frame rows are taken, as one rung each** of the render
  oracle in `tools/brief/frame.md`: the packet's heap from `graphsys`
  before a probe is booked (4168's), the card's setters on one take before
  a second kill from pixels (4165's). Both lanes were right, and the order
  between them is the cost: the packet is on disk, the card is a take.
- **Two rows of the pass's own** in the same section: look at the still
  beside the photograph before the first probe and at the landing; and a
  row is the client's pixels. The section read 2,580 characters against its
  page of 2,400 with them in, and 2,393 after two example paths were struck
  and the rows tightened.
- **The water's normal (4168a) is not an item.** It is the last hypothesis
  of item 4174, behind the floor's vertex colour and the shore's blend,
  which the still shows are the larger misses.
- **The corpses and rounds (4169's) are item 4175**, booked last: its
  count rises on any piece that decodes, so it carries a value diff from a
  packet, and it gates whole beside the chain.
- **Step III of 4159 stands as designed.** Nothing in this pass reads on
  the leaders line.
- **`docs/LOOP.md`'s roster line for the render track said one lane at
  medium**; it now says what entry 126 §2 decided, two at high.

## The items, and what each can fail on

| item | the score | how it fails today |
|---|---|---|
| 4171 the still under the readers | the four photograph rows counted on `client::gpu`'s still, pinned beside the painter's | the colour row on the still, by eye near zero |
| 4172 a luminance reader | three new pins, rule before count; the lit and unlit figure both counted | the mountain at a third, the lake at 1.6 times |
| 4173 the trees' texture | a new row over the trees in view | flat green against orange, expected 0 |
| 4174 the water and the shore | `RUN1564_3279_GROUND`'s 527 of 1,209 and its brightness twin | the twin, from its first count |
| 4175 corpses and rounds | `resolved` up from 1,167,081; K = M past 98 of 110 | the value diff on a packet's own piece |

Order: 4171 and 4172 side by side; 4173 on either's landing; 4174 takes
4172; 4175 between the chain's steps.

## Built

- **`tools/release_gate.py`**: `crates/client/` and
  `crates/render/src/silhouette.rs` join the paths that never widen a
  scoped or lane gate. The timed tests are the data crate's alone; nothing
  depends on the client, and no file of `rondata`, `sim` or `render` but
  `lib.rs`'s `pub mod` line names the painter. The test
  (`test_a_leaf_of_the_data_crate_does_not_widen_a_gate`) asserts both
  facts from the tree and was red before the rule. A dry run over the
  tranche's twelve merges spares one (4160); items 4171 and 4172 sit on
  exactly these paths.

## Not done, and said

- **No render code was touched.** The wash, the trees and the brightness
  are the lanes' to build against their rows.
- **The finer gate rule is not built**: six more of the eight render
  bookings widened on `rondata`'s `artdata`, `trace` and `free` and on
  `render::clock`. Which of those can move a timed test is a reading of the
  data crate's module graph, and is the next pass's if the queue of gates
  still binds.
- **The pace**: 5 % of the week in the first six and a half hours on
  three workers. Entry 126 §2 leaves a fourth to `commander` if the first
  twelve hours read under 20 points; it is on course to, and the gate
  queue, not the window, is what a fourth would meet.
- **The lobby screen and the input's next step** are undesigned.
- **`afsctool` covered half the archives.** It was six minutes on
  2026-10-17 and was at 51 % after 55 here; a pass's first act that takes
  two hours is not one. The next pass starts it on the captures added
  since, not on both trees whole.

## The gate

Scoped to `3d7cf1a94`, on the pass's commit; its log is
`~/Library/Caches/attrition/gates/gate-pass-1112.log`.
