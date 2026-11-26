# The pass of 2026-11-25, second: a row's take runs to its last state, the pivot's seam is seated, and three tools refuse what broke them

*2026-11-25, from 18:45 −05:00, the `steer` row on Fable 5.2, the
backstop's opener. `commander` (Opus 5.5) handed over at `f49417655`
after twenty-three first-parent merges (items 4672, 4670, 4668, 4669,
4674, 4673, 4637, 4677, 4675, 4676, 4657, 4678, 4680, 4679, 4681, 4684,
4682, 4685, 4683, 4687, 4686, 4688 and 4689), gated whole
(`gate-backstop-1125b.log`), with no lane out. The charter was the
handoff's `For the pass` part. Decisions in entry 158. The pass spawned
no subagent and took no capture. The record before it is
`2026-11-25-fable-pass-seats.md`.*

## For Ramon

- **The headline is 52 of 63, from 51.** The game that stood one line
  from whole this morning is whole. Twenty-three landings in seven and
  a half hours, 6.09 USD each, 140 USD in all.
- **Every landing on a game moved that game this time.** Sixteen of
  them, 207 agreeing states each. The tranche before had seven that
  moved nothing.
- **Eleven games are still open, 59,253 states short between them.** At
  this tranche's rate that is about 290 more landings. The rate has
  read 163, 43, 560 and 207 over four tranches, so the figure is loose.
- **One game was captured six times for six fixes; the other two were
  captured once, days ago, and needed nothing.** A game is now captured
  once, to its end. It costs about 14 GB until the next pass compresses
  it to under 2.
- **I broke my own run and fixed the cause.** The first step of a pass
  compresses the archives, and I had pointed its log into the archive.
  The compressor took the log and two scripts died silently. Nothing
  was damaged. The tool now refuses to start that way.
- **The one red gate of the day was a real bug in a script, not the
  machine being busy.** The script that waits on a capture could call a
  running game dead. Fixed, with a test that reproduces it.
- **The client's picture of the capital now matches the photograph
  outside the HUD.** What is still missing is the city's name plate and
  a raised log on the Woodcutter's Camp. The pair:
  `https://cdn.ramonfabrega.com/attrition/pass-1125b/`.
- **Yours to decide, as before:** the merged lane branches on origin;
  438 branches stand there. Three are not in the trunk (`rescue-234`,
  `worktree-att-1565-opus`, `worktree-att-4537`).
- **The week** read 68 % at 18:58 −05:00 on its fifth day, under the
  day's line of 70. Fable's week read 45 % before this pass and 46 % at
  that reading.

## What was measured

| what | measured |
|---|---|
| the backstop's gate (`gate-backstop-1125b.log`) | `gate exit 0` on `f49417655`, HEAD = origin/main = origin/worktree-replan-pdb; `Driver: 23 landings since the last pass (backstop 20)` |
| the merges (`git log --first-parent --merges 99c66271e..f49417655`) | 23, 10:49 to 18:12 −05:00 |
| the lanes (`tables-1125b-tranche.txt`) | 23 sessions over 23 items, 140.07 USD, 6.09 a landing; a session's clock 74.2 minutes, 12.5 working and 61.7 waiting, 36.9 on its own gate and 15.9 on a suite run; peaks 175 to 333 k, three past 300 (4637 at 311, 4675 at 333, 4689 at 310); 6 capture sections |
| the gates (`tables-1125b-gates.txt`) | 12 before the backstop's, 342.5 minutes, all scoped and widened; one red, `gate-4685-4683` (1.4 minutes, the offline step, the waiter's test); admission 28,335.5 s summed, longest 1,224.8 s; three ended with the load past 16 |
| what a seat bought (the booking commits' figures) | sixteen landings on a held-out row, 3,308 states before a first parting: row 148 1,122 and whole (4637); row 133 943 over five (802, 83, 51, 6, 1); row 88 684 over six (36, 26, 17, 329, 133, 143); row 93 559 over four (93, 31, 180, 255). Rows 43 and 83 of the fifth moved 346 and 9 on rows 88's and 93's rules. One landing moved nothing (4672, the siege muster's search) |
| the eleven games not whole (`score_recording` at `f49417655`, `score-*.txt`) | every first parting equals its pin: row 93 leaders 3524, row 133 units, guys and world 3620, row 83 deaths 3715, row 53 leaders 3716 (run1835), row 88 builds 3717 (the pin; run1843 agrees for its window, to 3590), row 43 six lines 4341, row 48 ammo 5607, row 108 7421, row 13 7675, row 8 units 9896. Row 148 exits 0, twelve lines of 12,001. Three window takes agree for their whole window and exit 0: run1815 to 2991, run1812 to 3608, the sample's run1786 to 8000 |
| the leaves on disk | row 93: leaders range 266, record 2, `who..ally_mask`, byte 1052, 0x01 against 0x00. Row 133: units range 1530, record 54, at (19944, 25627) against (19964, 25632); world range 1247, record 1223. Row 83: deaths range 7, record 3, byte 4, 0x12 against 0x11. Row 53: leaders range 221, record 1, `encrypt`. Row 43: units range 854, record 28; groups range 113, record 71, `num` 0 against 1; leaders range 128, record 1, `flags`; cities range 37, record 10, `city_flags` 0x01 against 0x41 |
| the states short of whole (each row's length less its first parting) | 59,253 over the eleven games: the sample 10,391, row 93 8,477, row 53 8,285, row 88 8,284, row 48 6,394, row 108 4,580, row 13 4,326, row 133 2,465, row 83 2,286, row 8 2,105, row 43 1,660 |
| the takes a row's words asked for (`docs/RUNS.md`, `du`) | row 88: six window takes for six words (run1834, run1838, run1839, run1840, run1842, run1843), 1,557.8 s of game, 18.8 GB, 1.8 GiB compressed. Row 93: run1799 to 12,000, 988.8 s, 13.1 GB, 1.6 GiB compressed, four words with no take. Row 133: run1797 to the game's end, 446.5 s, 6.3 GB, 0.8 GiB, five words with no take |
| the seam table (`held_seams`, feature `seam-count`, seven tests, 252 s, 56 walks) | six green and one red on the fallback's pin: the seventh's row 88 fires nine rounds from 7769, re-pinned `(88, 9)`. Four hooks no whole row enters (entry 158 §3); `pivot::offset_at_unpinned` on row 133 from 3685, 65 past its first parting, 385 entries |
| the still at the tip (`the_plates_on_run1814b`, `the_ring_round_the_capital_on_run1814b`) against run1814b's photograph | by region, theirs and ours: the left grove's trees (89, 64, 19) and (88, 65, 20), from (142, 113, 24) at item 4665's still; the right grove's lower trees (92, 66, 16) and (91, 69, 18); open grass (186, 190, 75) and (186, 189, 74); the fog at the lower right (124, 122, 44) and (122, 121, 41); the ground under the left grove (141, 125, 56) and (144, 133, 62). 371 of 1,792 blocks of 25 pixels differ by 25 levels or more, all but seven under the HUD's frame and text: four on the name plate, three on the camp |
| the archives (`archive_compress.py`, one tree at a time) | nothing to split; `scan: 0 damaged, 0 standing` (`--check`, after the run); the run's own log and the acts' log were compressed while open (entry 158 §5); 116 → 118 GB |
| the debug profile (`cargo clean --profile dev`) | 5.9 GiB; the disk read 112 GB by `df -g ~` before it, under the last pass's 119 |
| the suite re-timed (`suite_timings.py --stale-only`) | 11 tests timed, the tranche's leaf tests: 2,728 tests, 6,834 s one at a time |
| the rubric (`rubric.py --record`) | every figure the pass before's: ported 739, open 570, lockstep bytes 78.1 %, unit types seen 296 of 302 |
| the week (`lore limits`) | 67 % at 18:22 −05:00 and 68 % at 18:58; Fable's week 45 % and 46 % |
| the rows and trees left | the old driver row reaped by `handover.py reap bc3c5d56`; no lane row stood; no builder tree; three branches on origin not in HEAD, as before |

## Rulings

- **A row's take runs to its last state, by the scorer's own line, and
  is taken once** (entry 158 §2): the items of rows 88, 48, 108, 13 and
  8 are rewritten to it.
- **The pivot's seam is inside the mark and takes a seat** (§3): item
  4656, beside item 4691 on the same row.
- **Four seats** (§3): the chain's on 4692, parity seats on 4691 and
  4656, the client's on 4690; the unseated in the guard's order, 4573
  first.
- **The waiter reads the log before it asks for the process** (§4).
- **The compressor refuses its own output under its trees, and a pass
  writes nothing there until it exits** (§5).
- **`mutate.py` refuses a tree another cargo is compiling** (§6).
- **Ten parked findings** (§7): one closed in the tranche, one is item
  4690's first act, 4687a stays a rules gap no row has parted on, the
  rest stay in their journals.
- **The client's seat stays on 4690, and the name plate is minted
  behind it** (§8): item 4694.
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

The pass drew two stills at `f49417655` at run1814b's camera, frame
3294, and set them beside the photograph
(`pair/run1814b-theirs.png`, `run1814b-ours.png`,
`run1814b-ours-selected.png` and `run1814b-difference-x3.png` under
the pass's directory, and uploaded). The trees at the fog's edge are
dark red-brown in both now, at the lower left, the bottom and the
right. The sandy plates lie under the three blocks of the city, the
wheat stands south-west of it, and the mountain, the lake's edge and
the fog's soft edge fall where theirs do. With the selection drawn,
ours carries the red outline round the selected city and the green bar
above one of its buildings, as theirs does. Outside the HUD's frame
two things differ. Theirs has the city's name plate under the selected
city, a dark box with a gold rim, the capital's star and `Napata`.
And the Woodcutter's Camp at the right grove carries a log raised on
its hoist in theirs and stands at rest in ours; the two citizens beside
it stand in slightly different places. The HUD's own frame, panels and
top bars are not in either still. Ours has a white bird at the lower
left where theirs shows a small dark one left of the farms, as before.

The pass's first reading of the pair was that a pale band of ground
ran further right from the left grove in ours. Measured, the band
reads (176, 170, 76) in theirs and (175, 170, 75) in ours, and the
difference map is empty there. It is not a finding.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4692 sixth, row 93 | rules, whole; the chain's | row 93's column from 3524 | the leaders leaf on run1799 | two landings running move no column |
| 4691 sixth, row 133 | rules, whole; a parity seat's | row 133's column from 3620 | the units leaf on run1797 | two landings running move no column |
| 4656 the pivot's seam | rules, whole; a parity seat's | the hook's 385 entries, row 133's column | the unpinned offsets off a packet, as rows | it has landed |
| 4690 the capital's ground | render, scoped; the client's | run1559b's pieces, 471 of 490 | the client picks on the mesh's lines | it has landed |
| 4573, 4671 rows 83 and 53 | rules, whole; unseated | each row's column | the leaf on run1784, run1835 | two landings running move no column |
| 4693 seventh, row 88 | rules, whole; unseated | row 88's column from 3717 | the row's one whole take, `0xffbu` | two landings running move no column |
| 4571 fifth, row 43 | rules, whole; unseated | row 43's column from 4341 | the leaves on run1787 | two landings running move no column |
| 4615, 4616, 4579, 4617 rows 48, 108, 13 and 8 | rules, whole; unseated | each row's column | the row's one whole take, `0xffbu`, `0xfebu`, `0xfebu`, `0xdfbu` | two landings running move no column |
| 4649 the sample | rules, whole; unseated | `Parity:`, units 10653 | the tick after the 10652 package | two landings move neither word |
| 4694 the name plate | render, scoped; behind 4690 | a new row on the plate's box | the plate's draw in run1833's stream | the stream holds no draw for it |

## Built

- `tools/gamelog/waitrun.sh`: the log is read for a start before
  `pgrep` is asked. One test in `tools/explore/test_waitrun.py`, a
  `pgrep` on the path that answers "no runner" as the runner's first
  line lands; seen red on the script as it stood.
- `tools/archive_compress.py`: `output_under`, and the refusal, exit 2.
  Two tests, seen red (no function; exit 0).
- `tools/mutate.py`: `building`, and the refusal while a build
  directory's `.cargo-lock` is held. One test, seen red.
- `crates/rondata/src/diff/held_seams.rs`: the seventh's fallback pin,
  `(88, 9)`.
- The paperwork: entry 158 and its index row, this record, the queue's
  handoff, lanes line and fourteen items, two clauses of
  `docs/LOOP.md`, the tracks' paragraph (the oldest folded out), the
  journal's entry, the suite's timings, the rubric's row.

## Not done, and said

- **Nothing holds the take rule but a clause and the items' words.**
  The dumps are outside the repo, and no guard reads which rows hold a
  record to their last state.
- **The compressor's refusal covers its own output only.** Another
  script's log under the trees is still taken; that half is the clause.
- **`mutate.py` does not see a cargo started after the mutant is
  applied**, item 4655's case.
- **What made every row landing move its row is not separated**: the
  stop's count of builds and the leaves pasted into the items both
  changed at the pass before.
- **Rows 53 and 88 are scored on windows**, so how many lines they
  part on by their last state is read off the pins, not this pass's
  scores.
- **Item 4656 may land with no column moved.** Row 133 parts 65 states
  before the hook's first entry, and the fix to that leaf is another
  item's.
- **The name plate and the hoist have no row.** Item 4694 opens on
  whether run1833's stream holds the plate's draw, which the pass did
  not check.
- **The photograph on file is the 1,400-wide copy** the pass before
  drew; the still was scaled to it for the measures.
- **The nine fallback rounds on row 88 were not printed by piece.**
  They fall 4,052 states past its first parting.
- **Fable's week read 45 % against 33 % at the last pass's reading,
  with the tranche's lanes on Opus.** Where the twelve points went was
  not traced.
- **The lane's wait was measured and not changed**, the files over the
  compressor's limit stay raw, the driver's own cost is not summed, and
  the blind list was not scanned (open 0), all as before.

## The gate

Whole, by `tools/gate_run.sh`, to `gate-pass-1125c.log`; its verdict is
quoted here by the commit after the pass's own.
