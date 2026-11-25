# The pass of 2026-11-25: a game one line from whole is seated by a guard, the stop counts builds, and a lane lands on a verdict its gate printed

*2026-11-25, from 08:45 −05:00, the `steer` row on Fable 5.2, the
backstop's opener. `commander` (Opus 5.5) handed over at `2a2faa6ca`
after twenty-three first-parent merges (items 4636, 4618, 4608, 4635,
4650, 4652, 4634, 4651, 4654, 4653, 4655, 4609, 4658, 4633, 4659, 4662,
4661, 4663, 4660, 4664, 4666, 4665 and 4667), gated whole
(`gate-backstop-1125.log`), with no lane out. The charter was the
handoff's `For the pass` part. Decisions in entry 157. The pass spawned
no subagent and took no capture. The record before it is
`2026-11-24-astra-settlement.md`.*

## For Ramon

- **The headline is 51 of 63, from 50.** One more game agrees with the
  original from its first frame to its last. Twenty-three landings in
  seven and a half hours, 7.44 USD each, 171 USD in all.
- **One landing bought nine tenths of the progress.** It was on a game
  that already agreed on eleven of the twelve checksum lines. The other
  sixteen game landings gained 53 states each.
- **A second game stands in that position now, and the handoff had it
  last in line.** I gave it the first seat. The rule that such a game
  goes first existed only as a sentence; a test now holds the queue to
  it.
- **A lane landed without its gate having run its tests, and a 39-minute
  gate went red on it.** The queue file was over a size limit when the
  lane was started, which stopped the lane's gate early, and the check
  meant to catch a landing with no verdict accepted a sentence saying
  there was none. Both are fixed in the tools.
- **One game's line was stopped by a rule that counted two readings as
  two failures.** The fix was already worked out to a single term. The
  rule now counts builds, and that item is back in the order.
- **Lanes wait four times as long as they work**: 67 minutes of an 84
  minute landing, 40 of them on the lane's own gate queueing for the
  machine. That was 53 of 67 the tranche before. I changed nothing
  here; a fifth lane would make it worse.
- **The client's ground now has the wheat and the paving**, and the
  next thing the eye finds is the trees at the edge of sight, dark in
  the original and bright in ours. The seat stays on that. The pair:
  `https://cdn.ramonfabrega.com/attrition/pass-1125/`.
- **Yours to decide, as before:** the merged lane branches on origin.
  Three branches there are not in the trunk (`rescue-234`,
  `worktree-att-1565-opus`, `worktree-att-4537`).
- **The week** read 56 % at 09:23 −05:00 on its fifth day. Fable's
  week read 30 % before this pass and 33 % at that reading.

## What was measured

| what | measured |
|---|---|
| the backstop's gate (`gate-backstop-1125.log`) | `gate exit 0` on `2a2faa6ca`, HEAD = origin/main = origin/worktree-replan-pdb; whole, 9 of 9 steps, release 23.2 minutes; `Driver: 23 landings since the last pass (backstop 20)`, context 358 k |
| the merges (`git log --first-parent --merges df102cc6e..2a2faa6ca`) | 23, 00:34 to 08:08 −05:00 |
| the lanes (`tables-1125-tranche.txt`) | 23 sessions over 23 items, 171.19 USD, 7.44 a landing; a session's clock 83.5 minutes, 16.3 working and 67.2 waiting, 39.5 on its own gate and 13.5 on a suite run; peaks 201 to 366 k, two past 300 (4650 at 366, 4653 at 360); 7 capture sections |
| the gates (`tables-1125-gates.txt`) | 11 before the backstop's, 377.7 minutes: five whole, six scoped and widened; two red, `gate-4636` (8.9 minutes, the guard, on the queue's ceiling and a function's name) and `gate-4650-4652` (38.9 minutes, two client pins); admission 30,769.2 s summed, longest 1,160.9 s; four ended with the load past 16 |
| what a seat bought (the journals' score lines) | seventeen landings on a held-out row, 9,518 states before a first parting: row 128 8,663 and whole (4635); row 53 358 over two builds; row 133 226 over four; row 88 131 over three; the fifth's rows 43 and 83, 96 and 44, on row 133's fix (4655). Seven moved no first parting: four first acts landed alone (4634, 4652, 4654, 4609) and three named fixes (4618, 4608, 4651) |
| the twelve games not whole (`score_recording` at `2a2faa6ca`, `score-*.txt`) | every first parting equals its pin: row 133 units and guys 2677, row 93 units 2965, row 88 units 3033 (run1834), row 83 3706, row 53 leaders 3716 (run1835), row 43 groups 3995, row 48 ammo 5607, row 108 7421, row 13 7675, row 8 units 9896, row 148 units 10879. Three window takes agree for their whole window and exit 0: run1815 to 2991, run1812 to 3608, the sample's run1786 to 8000 |
| row 148's lines (`score-7-148.txt`) | units 10,879 of 12,001; the other eleven lines 12,001 of 12,001 |
| the leaves on disk | row 133: units range 4225, record 18, guy, `cur_anim` 0 against 1. Row 83: units range 1101, record 26, guy, `cur_anim` 13 against 11. Row 88: units range 2205, record 66, `damage` 39 against 38. Row 43: groups range 64, record 64, `army` 0 against −1. Row 53: leaders range 221, record 1, `encrypt` |
| the states short of whole (each row's length less its first parting) | 62,916 over the twelve games: the sample 10,391, row 93 9,036, row 88 8,968, row 53 8,285, row 48 6,394, row 108 4,580, row 13 4,326, row 133 3,408, row 83 2,295, row 8 2,105, row 43 2,006, row 148 1,122 |
| the seam table (`held_seams`, feature `seam-count`, seven tests, 256 s, 56 walks) | seven green, the fallback's pins empty. Six hooks no whole row enters (entry 157 §5); `muster::siege_find_building` on five open rows, before the first parting on rows 133, 93 and 83 |
| row 93's term at the address (`compare_target`'s tail, read by the pass) | a negative product becomes 9,999,999, a fifth of it out of range, `(v + 99) / 100`: 20,000 on theirs' blow of 24,132 with the journal's 84,604 and 20; ours holds the tail |
| the archives (`archive_compress.py`, one tree at a time) | nothing to split, the same files skipped over the compressor's limit; `scan: 0 damaged, 0 standing`; 121 → 125 GB |
| the debug profile (`cargo clean --profile dev`) | 2.9 GiB; the disk read 119 GB by `df -g ~` before it, under the settlement's 134 |
| the suite re-timed (`suite_timings.py --stale-only`) | 9 tests timed, then the pass's own 2: 2,717 tests, 6,815 s one at a time |
| the rubric (`rubric.py --record`) | every figure the pass before's: ported 739, open 570, lockstep bytes 78.1 %, unit types seen 296 of 302 |
| the week (`lore limits`) | 54 % at 08:18 −05:00 and 56 % at 09:23; Fable's week 30 % and 33 % |
| the rows and trees left | the old driver row reaped by `handover.py reap 096274dc`; no lane row stood; `reap_agents.py` found no builder tree; three branches on origin not in HEAD, as before |

## Rulings

- **A game one line from whole goes first, and a guard holds the
  queue's order to it** (entry 157 §2): row 148, item 4637.
- **The stop counts builds** (§3): a first act landed alone that names
  a term with both sides' values is not one of the two; item 4657 is
  back in the order as a build.
- **A lane lands on a verdict its gate printed** (§4).
- **A named fix is seated when a row enters its hook before its first
  parting or within 100 states past it** (§5): item 4672 is minted and
  seated, item 4656 waits.
- **Four seats** (§6): the chain's on 4637, parity seats on 4672 and
  4668, the client's on 4670.
- **Fourteen parked findings** (§9): four closed in the tranche, one is
  item 4670's second act, the rest stay in their journals.
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

Six of the twenty-three landings drew the capital's ground (4636, 4650,
4653, 4658, 4661, 4665). The pass set run1814b's photograph beside item
4665's still of the same camera and frame (`pair/run1814b-theirs.png`,
`pair/run1814b-ours.png` under the pass's directory, and uploaded). No
client source has changed since that still. The wheat plots stand
south-west of the city in both, with the bare plots between them, and a
sandy plate lies under each building. The mountain, the lake's edge and
the fog's soft edge fall where theirs do. The trees differ: the grove
at the lower left, the lower half of the grove at the bottom and the
trees at the right edge are dark red-brown in theirs, where the fog
begins, and bright gold in ours. Theirs carries the city's name plate
and an outline round the selected city; the still carries no HUD and no
selection. The pale ground under the lower-left grove reaches further
right in ours. Ours has a white bird at the lower left; theirs has a
small dark one left of the farms.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4637 seventh, row 148 | rules, whole; the chain's | the seventh 3 → 4 of 8, `Parity:` 52 | one take, `0x1u`, to 11,129 | two landings running move no column |
| 4672 the siege muster's seam | rules, whole; a parity seat's | the columns of rows 133, 93 and 83, each scored | the search built on the arms the crate has | it has landed |
| 4668 seventh, row 88 | rules, whole; a parity seat's | row 88's column from 3033 | the leaf on run1834 | two landings running move no column |
| 4670 the capital's ground | render, scoped; the client's | `RUN1814B_3294_FOG`, then `_PLATES` | the trees past sight darkened | the packet holds no draw for it |
| 4669 sixth, row 133 | rules, whole; unseated, first | row 133's column from 2677; row 83 scored | the leaf on run1797 | two landings running move no column |
| 4657 sixth, row 93 | rules, whole; unseated | row 93's column from 2965 | the build, one term | it moves no column |
| 4573, 4671, 4571 rows 83, 53 and 43 | rules, whole; unseated | each row's column | the leaf on run1784, run1835, run1787 | two landings running move no column |
| 4615, 4616, 4579, 4617 rows 48, 108, 13 and 8 | rules, whole; unseated | each row's column | a take, `0x8u`, `0x63u`, `0xfebu`, `0x1u` | two landings running move no column |
| 4649 the sample | rules, whole; unseated | `Parity:`, units 10653 | the tick after the 10652 package | two landings move neither word |
| 4656 the pivot's seam | rules, whole; unseated, held | row 133's column | the unpinned offsets as rows | a seam table finds it inside the mark |

## Built

- `crates/rondata/src/diff/held_out.rs`: `open_rows`,
  `unseated_row_items`, `out_of_seat_order` and two tests. The guard,
  `the_handoff_s_unseated_rows_stand_in_the_seats_order`, was seen red
  on the queue as the handoff left it, on five slips; it is one of
  `release_gate.py`'s commander's lines.
- `crates/sim/src/docs_guard.rs`: from item 4672 a lane's journal
  quotes a verdict a gate prints. Seen red on item 4650's journal with
  the mark moved back over it; a test holds the sentence itself.
- `tools/release_gate.py`: `LANE_GUARD_RED`, printed when a lane's
  guard step is red. One test, seen red twice (no constant, then no
  print).
- `tools/brief.py`: the test lister leaves out a module declared behind
  a cargo feature. One test, seen red on the seven `held_seams` names.
- The paperwork: entry 157 and its index row, this record, the queue's
  handoff, lanes line and fifteen items in the seats' order, the loop's
  clauses, the tracks' paragraph (the two oldest folded out), the
  journal's entry, one line in `docs/audit/README.md`, the suite's
  timings, the rubric's row.

## Not done, and said

- **No seam is shown to be a row's cause.** The siege muster's hook is
  entered before three first partings, and each walk agrees for 225 to
  665 states after it.
- **Whether the three named fixes of the tranche saved a landing is not
  measured.** Rows 88 and 133 walked through their hooks afterwards;
  nothing says they would have parted there.
- **The 100 states of entry 157 §5 rest on three cases**, 17, 23 and
  90 states ahead.
- **Row 148's take is not taken**, so its leaf is unknown; that units
  parts on every state after 10879 is read off the sums.
- **Row 93's build is read, not built.** The tail gives 20,000 on
  theirs' figures; ours' 594,950 was not re-derived to the digit (the
  pass's arithmetic on the journal's figures gives 594,550).
- **The lane's wait was measured and not changed.** Forty minutes of a
  lane's landing is its own gate, most of it admission.
- **The guard on the unseated order reads the file's order**, and a
  seated row is out of its reach: it does not say when a seated row
  should give its seat up.
- **Two items of the last record carried a packet frame one early**
  (4609, 4654); the frames were the pass's.
- **No still was drawn at the tip.** The pair is item 4665's still and
  the photograph on file.
- **`tools/ctx.py` still reads the driver's context from a builder**,
  the files over the compressor's limit stay raw, the driver's own cost
  is not summed, and the blind list was not scanned (open 0), all as
  before.

## The gate

Whole, by `tools/gate_run.sh`, once, on the pass's commit; the commit
after it quotes the verdict here.
