# The pass of 2026-11-23, second: the seventh ratified, a game near its end seated first, and the client's seat on the fog

*2026-11-23, from 14:00 −05:00, the `steer` row on Fable 5.2, called on
Ramon's word. `commander` (Opus 5.5) handed over at `d399cae86` after
twelve first-parent merges (items 4596, 4598, 4597, 4599, 4601, 4592,
4602, 4603, 4590, 4604, 4605 and 4606), gated whole
(`gate-handoff-1123b.log`), with three lanes out. All three had landed
when the pass began; it merged and booked them (4600, 4578, 4607). The
charter was the handoff's `For the pass` part. Decisions in entry 155.
The pass spawned no subagent and took no capture. The pass before it
is `2026-11-23-fable-pass-seams.md`.*

## For Ramon

- **The headline is 48 of 55, up from 47.** One more game agrees with
  the original on every state. Fifteen landings in five hours, 6.87 USD
  each.
- **A landing bought less this time: 163 agreeing states, against 621
  last tranche.** The games still open are late-age fights. Each fix
  was a real rule the original has and ours lacked, but the game goes
  wrong again a few hundred states later. The one game that finished
  was the one that had a single line left.
- **The new battery, with a human who keeps giving orders, read 1 of 8
  as found**, two games not yet played for lack of disk. Inside the
  first 6,001 states it agrees 87 % of the way, where the last battery
  read 59 %. A person's orders did not break a game early.
- **I seated a game that goes wrong 292 states before it ends ahead of
  games that go wrong earlier**, because a game that close can finish
  in one fix. It is a bet on two cases; the next pass scores it.
- **The client's seat draws the fog of war next.** Our window shows the
  whole map lit, which is the first thing a person playing would
  notice, and no capture on file was ever taken with the fog on. The
  seat starts with that capture. Say the word to put the seat on a
  parity game instead.
- **The disk.** It read 56 GB at the handoff and 84 after this pass
  compressed the archive and removed three finished lane trees. Every
  capture was being stored twice; new ones are stored once from now.
  The copies already there are 19 GiB, and 10 GiB more is the same
  executables saved 4,151 times. Folding those needs your word, and
  the larger sums are outside this project: see the disk report in the
  session.
- **Yours to decide:** 335 merged lane branches stand on origin.
- **The week** read 36 % at 13:48 −05:00 on its third day, six points
  under the day's line. Fable's week read 14 % before this pass.

## What was measured

| what | measured |
|---|---|
| the handoff's gate (`gate-handoff-1123b.log`) | `gate exit 0` on `d399cae86`, HEAD = origin/main = origin/worktree-replan-pdb; whole, 9 of 9 steps, 35.6 minutes, release 23.6; `Driver: 12 landings since the last pass (backstop 20) · context 278 k`; 66 GB free after it |
| the three lanes out | 4600 `f2b5b57d5`, 4578 `ff0f26fed`, 4607 `8eb3fb72e`, each on origin before its merge; 4607's merge backed out on one `docs/RECGAME.md` hunk (4600's §14 beside its paragraph), the lane took `ccc update --keep-conflicts` and pushed `78436f39d`; booked at `a3304119b` and `274d282e8` |
| the merges (`git log --first-parent --merges ebc1b9d00..274d282e8`) | 15, 09:16 to 14:10 −05:00 |
| the lanes (`tools/tranche.py --base ebc1b9d00`, after `lore index`; `tables-1123c-tranche.txt`) | 15 sessions over 15 items, 103.10 USD, 6.87 a landing; a session's clock 69.0 minutes, 13.9 working and 55.1 waiting, 27.3 on its own gate and 9.4 on a suite run; peaks 163 to 316 k, two past 300 (4601 at 313, 4603 at 316); 10 capture sections, seven of them 4600's |
| the gates (`tools/gates.py … --since gate-pass-1123.log`; `tables-1123c-gates.txt`) | 10, 252.0 minutes: nine scoped (eight widened) and the handoff's whole gate; one red, `gate-4596`, 2.3 minutes, on the queue's own line; admission 15,399.5 s summed, longest 678.5 s; two ended with the load past 16 |
| what a seat bought (the bookings' own lines) | eight landings on a held-out row: 1,302 states before a first parting (row 133 445 over four, row 113 578 and whole, row 93 279 over two, row 53 none), 163 a landing |
| the thirteen games not whole (`score_recording` at `274d282e8`) | entry 155 §3's table; exit 1 on each, a measured difference, 11 to 78 seconds; every report's first parting equals its pin; under `~/ron-data/lab-experiments/2026-11-23-pass-second/` |
| the seam table (`held_seams`, feature `seam-count`, seven tests, 207 s, the seventh added) | five green; the sixth red, `left: Some([(133, 455)])`, `right: Some([(133, 471)])`; the seventh red on its empty pin, `left: Some([(28, 41), (48, 20), (88, 2092)])`. Row 88: `kill_contents_no_come_out` entered first on state 2741, its first parting, marked `=`, 210 entries; row 53 enters it first on 3707. Row 93: `attack_ground_aim` first on 2962, marked `<`, 318 entries, the one hook on or before a first parting that none of the 43 whole rows enters. Run again on the re-pinned figures (`held-seams-2.log`): seven green |
| the bay probe (`PASSFALLBACK` on `launch_point`'s fallback, `the_seventh_held_out_games_score_as_pinned`, exit 0, 198 s) | 2,153 lines: pieces 128 (835) and 292 (1,257) on row 88, 1812 (19) and 1849 (22) on row 28, 10 (20) on row 48; nineteen keys of piece, anim and node; the print taken out after |
| the archives (`split.py --archive`, then `afsctool` over both) | nothing to split; four files skipped over the compressor's limit, the same four; 62 → 68 GB |
| the disk (`df -g ~`) | 56 GB at the handoff's last reading; 62 at the pass's start; 68 after the compression; 78 after reaping 4578's and 4600's trees; 84 after 4607's |
| the archive's twins (`stat` over `Logs` and `~/ron-golden`) | `Logs`: 4,301 files, 29.8 GiB on disk, 401.2 apparent; 1,785 of them the same size as a file under `~/ron-golden/<run>/map-*`, 19.2 GiB on disk, none a hard link; `~/ron-golden`: packets 26.9 GiB, traces 12.8, ranges 11.7, dumps 11.6, 4,151 executables 10.2 |
| the temp directory (`du` over `$TMPDIR`) | 47 GiB in all: 440 `attrition-gate-*` report directories 21.3 GiB, `attrition-diff-failures` 851 exports 8.9 GiB, 136,820 unnamed entries 6.5 GiB |
| the compressor and a hard link | a 25.6 MB file under two names: one inode before and after `afsctool -c -f -T LZFSE`, 25,000 → 80 KiB on disk, `cmp` equal |
| the week (`lore limits`) | 36 % at 13:48 −05:00, reset at 06:00 −05:00 on 2026-11-21; Fable's week 14 % |
| the suite re-timed (`suite_timings.py --stale-only`) | 13 tests timed: 2,682 tests, 6,728 s one at a time |
| the rubric (`rubric.py --record`) | every figure the pass before's: ported 739, open 570, lockstep bytes 78.1 %, unit types seen 296 of 302 |
| the rows and trees left | three lane rows reaped at their bookings (`ccc rm`, each tree clean and its tip on origin); the old driver row reaped by `handover.py reap 9a338be3`; `reap_agents.py` found no builder tree; no Codex branch ahead of HEAD; 335 merged `worktree-att-*` branches on origin |

## Rulings

- **The seats go by first parting, still, and a row within 600 states
  of its last state ranks with a row one line from whole** (entry 155
  §1, §4).
- **The seventh is ratified as found** and folds at its first leaf on
  the rows taken; an eighth is named, rows 18 to 158 (§2).
- **Rows 93 and 83 keep one seat; rows 133 and 43 name one shape**, and
  item 4610 reads the group first (§3).
- **A seam hook whose first entry is a row's first parting is pasted
  whatever the whole rows enter** (§9): item 4614 opens on row 88's,
  item 4608 carries row 93's, item 4618 builds the seventh's five
  pieces with no bay.
- **The client's seat takes the fog**, a photograph first (§5; item
  4612). The fourth seat is the client's again.
- **A capture is filed as a hard link** (§6). The twins already filed
  are Ramon's. **A gate keeps two days of its reports**, sweeps the
  diff viewer's exports at that age, and the viewer's test removes its
  own.
- **Four seats**: the chain's on 4613, parity seats on 4610 and 4614,
  the client's on 4612.
- **Eleven parked findings and notes**: one built (4600a), the rest
  stay in their journals (§8).
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

Four of the fifteen landings drew the minimap (4596, 4601, 4603,
4605). The pass set run1564's photograph beside item 4603's still of
the same camera, full frame and at the minimap's crop
(`run1564-theirs-half.png`, `run1564-ours-half.png` under the pass's
directory; `side.png` under item 4603's). The lake, its surf, the
sand, the cracked ground and the fish sit where theirs do. The minimap
carries the lakes, forests, borders, blips, pings and the view's
rectangle where theirs does; ours has cyan specks in its lakes that
theirs lacks, and no frame. Everything else the photograph has and the
still lacks is the HUD: the bar and its panels, the resource counts,
the message log, the banner. The window carries a plain HUD and the
still does not, so those were not judged. What neither shows is fog:
both are a console that sees everything. Nothing was uploaded.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4613 seventh, row 28 | rules, whole; the chain's | the seventh 1 → 2 of 8; the fold, `Parity:` 49 of 61 | a take, `0x843u`, `--allow-early-end` | two landings running move no column |
| 4610 sixth, row 133 | rules, whole; a parity seat's | row 133's column from 2351 | group 132's record on 2350 and 2351 | two landings running move no column |
| 4614 seventh, row 88 | rules, whole; a parity seat's | row 88's column from 2741 | the seam's fix: a rider comes out before it dies | two landings running move no column |
| 4612 the fog | render, scoped; the client's | a new row on a new photograph | one take with the reveal off | no setting the capture reaches draws fog |
| 4608 sixth, row 93 | rules, whole; unseated, first | row 93's column from 2965; row 83 scored again | the leaf on run1799 | two landings running move no column |
| 4609 sixth, row 53 | rules, whole; unseated | row 53's column from 3358 | a packet at 3356 | two landings running move no column |
| 4618 the seventh's bays | rules, whole; unseated, behind 4614 | none of its own: 2,153 rounds past three first partings | the nineteen keys measured | it has landed its act |
| 4573 fifth, row 83 | rules, whole; rides 4608 | row 83's column from 3662 | the leaf on run1784 | two landings running move no column |
| 4571 fifth, row 43 | rules, whole; behind 4610's reading | row 43's column from 3833 | the leaf on run1787 | two landings running move no column |
| 4615, 4616, 4617 seventh, rows 48, 108, 8 | rules, whole; unseated | each row's column | a take, `0x8u`, `0x63u`, `0x1u` | two landings running move no column |
| 4569 the sample | rules, whole; unseated | `Parity:`, leaders 7502 | the leaf on run1786 | two landings running leave the first parting and the draw word |
| 4579 sixth, row 13 | rules, whole; unseated | row 13's column from 7675 | a take, `0xfebu` | two landings running move no column |
| 4611 seventh, rows 128 and 148 | rules, whole; with the first seat that frees | the seventh's found column, eight of eight taken | `df -g ~` above 60 | both rows are pinned |

## Built

- `tools/gamelog/filerun.py`: the archive's name is a hard link to the
  take's file, a copy only where a link cannot be made, and the line it
  prints says which. Two tests in `tools/explore/test_filerun.py`,
  seen red on the copy (`gamelog-run893c-tag.txt is a second copy`).
- `tools/release_gate.py`: `REPORT_KEEP_DAYS` 4 → 2, and the sweep
  takes `attrition-diff-failures/*.html` of that age. One test, seen
  red (`4 != 2`). `crates/rondata/src/debug_view.rs`: the acceptance
  test removes its export unless `RON_KEEP_EXPORT` is set.
- `crates/rondata/src/diff/held_seams.rs`: the seventh's walks are
  counted (`the_seventh_held_out_walks_list_the_seams_they_enter`),
  seen red on an empty pin; the fallback's pins read `(133, 455)` and
  `(28, 41), (48, 20), (88, 2092)`.
- The paperwork: entry 155 and its index row, this record, the queue's
  handoff, lanes line and fifteen items, the loop's clauses (the
  near-end rank, the eighth's name, the filed link), the tracks'
  paragraph, the journal's entry, the suite's timings, the rubric's
  row.

## Not done, and said

- **The fog item is a design and a look, not a reading.** Nobody has
  read how the original turns its reveal off in a capture, or what the
  ground's fog stage lays. Its first act may find the capture lane
  cannot reach a fogged view at all.
- **That one writer parts rows 133 and 43 is not shown.** Two leaves
  have one shape. No function was read.
- **Neither seam is shown to be its row's cause.** A hook entered on
  the parting's state, and a note that says what the original does
  there. Item 4597's was the same evidence and held.
- **The near-end rank rests on two games.**
- **Nothing of the seventh's partings is read.** Six takes hold sums.
  Whether the human's orders caused any of them is unknown.
- **The link, as built, has filed no real capture.** The tests file a
  36-byte trace in a temp directory.
- **The twins already filed were counted by size, not by content.** A
  fold would `cmp` each pair first.
- **The 44 P notes item 4599 left unhooked stay unhooked**, and the 210
  notes classed as unreachable are still one Sonnet reading.
- **No still was drawn at the tip and nothing was uploaded.** The pair
  is item 4603's still and the photograph on file.
- **`tools/ctx.py` still reads the driver's context from a builder**,
  the four files over the compressor's limit stay raw, the driver's own
  cost is not summed, and the blind list was not scanned (open 0), all
  as before.

## The gate

Whole, by `tools/gate_run.sh`, once, on the pass's commit; the commit
after it quotes the log (`gate-pass-1123b.log`).
