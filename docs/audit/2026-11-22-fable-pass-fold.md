# The pass of 2026-11-22: the fifth battery folds, the sample's seat reopens, and the gate asks for the trunk

*2026-11-22, from 04:40 −05:00, the `steer` row on Fable 5.2, called by
the backstop. `commander` (Opus 5.5) handed over at `252444c84` after
twenty first-parent merges (items 4517, 4526, 4509, 4528, 4518, 4530,
4529, 4531, 4532, 4527, 4533, 4519, 4485, 4534, 4537, 4535, 4539, 4536,
4540 and 4538), gated whole (`gate-drain-1122.log`). The charter was the
handoff's: seat items 4541 and 4542, and rule on the handoff's line
bound. Decisions in entry 151. The pass spawned no subagent and took no
capture. The pass before it is `2026-11-21-fable-pass-pieces.md`.*

## For Ramon

- **The headline now reads 39 of 47, and nothing got better or worse.**
  It read 37 of 39 at the handoff. Your fold rule says a held-out
  battery joins the count once a lane has worked on one of its games.
  That happened to the fifth battery seven landings ago and nobody
  added it. Its eight games are in now, two of them whole.
- **The one game with a person's orders had lost its seat by mistake.**
  The driver stopped it for "two landings that moved nothing". Both had
  moved it. The seat is back. That game agrees for 6,108 of 21,044
  states, up from 5,401, at about 177 states and 7.69 USD a landing.
- **`origin/main` was 137 commits behind** when I started, since
  midnight. I fast-forwarded it. Your root checkout pulls from it. The
  gate now refuses to run while main is behind.
- **You asked this pass to confirm your two answers of yesterday.**
  Both are built as you said. The fifth battery's games each have three
  players and one team of two. Three is the most those maps hold.
- **The team games are much harder for us than the two-player ones.**
  An unseen two-player game ran 91 % of its states before first
  differing. An unseen team game ran 48 %. Six of the eight are open
  and each differs on nearly every line.
- **A sixth unseen battery is named** (rows 13, 33, … 153), taken when
  the fifth reads 4 of 8. **One question:** in all five batteries the
  human player does nothing. I recommend the sixth's human plays from a
  scripted list of orders. Say so if you want it; otherwise it is idle
  like the rest.
- **The tranche:** twenty landings in seven and a quarter hours. Two
  gates were red, each fixed by a second commit; nothing red was
  pushed. A landing cost 8.17 USD, up from 7.18, and waited 90 minutes,
  up from 73. Fifty-five of those minutes are its own gate, up from 40
  and 31 the two tranches before.
- **The week** read 14 % at 04:45 −05:00, the day after its reset.
  Fable's week read 5 %.

## What was measured

| what | measured |
|---|---|
| the drain's gate (`gate-drain-1122.log`) | `gate exit 0` on `252444c84`, the tip, HEAD = origin/worktree-replan-pdb; whole, 9 of 9 steps, 26.7 minutes, release 22.4; `Driver: 20 landings since the last pass (backstop 20)`, context 276 k |
| the trunk (`git rev-parse origin/main`, the two refs' reflogs) | `a5c5f8204`, 137 commits behind the tip; the branch was pushed 13 times in the tranche and main 6, the last at 00:07 −05:00 |
| the merges (`git log --first-parent --merges 43ee59434..252444c84`) | 20, 20:56 to 04:09 −05:00 |
| the lanes (`tools/tranche.py --base 43ee59434`, after `lore index`) | 12 sessions over 12 items, 98.01 USD, 8.17 a landing; a session's clock 109.7 minutes, 20.3 working and 89.5 waiting, 54.8 on its own gate and 14.5 on a suite run; peaks 199 to 305 k, two at or past 300 (4527, 4534); 13 captures. Items 4517, 4528, 4530, 4532, 4485, 4537, 4539 and 4540 were in-session builders and have no session |
| the gates (`tools/gates.py … --since gate-pass-1121c.log`) | 16, 364.6 minutes: 1 whole, 15 scoped (15 widened), 2 red, 1 stopped; admission 22,411.7 s summed, longest 804.9 s; 3 ended with the load past 16 |
| the two red gates | `gate-4532` on run1153c's widened word, re-pinned 112 → 82 by `a5c5f8204`, green as `gate-4532b`; `gate-4534` on seven melee pins and two packets that item 4537 improved, re-pinned on the merged tree by lane 4535 (`6276ae9ea`), green as `gate-4535` |
| the `Parity:` line at each booking (`git show <booking>:docs/QUEUE.md`) | world 5401, leaders 5403 (4526), leaders 5486 (4529), units 6000 (4533), units 6108 (4535); draws 5460, 5460, 6015, 6015, 6114; the count 6 of 7, then 34, 35, 36, 36, 36 and 37 of 39 |
| the seven rows not whole (`score_recording` at the tip) | entry 151 §5's table; exit 1 on each, a measured difference. Row 110 names three leaves off run1742; the six fifth rows hold sums and each prints one take |
| the fifth's declaration against its files (`git show -s aece988eb`, `stat` with `%z`) | committed 21:55:27 −05:00; the eight files written 21:59:43 to 22:01:52 |
| the spent list (`held_out::SPENT`) | four batteries; row 23's two leaf tests are the fifth's |
| a builder's tree left standing | `agent-a2df0f7d276d00354`, branch `worktree-att-4537`, tip `9533fbeb3`, one commit past the merge, clean, locked by pid 34070, which was gone. Its seven pins and two packet values are HEAD's, value for value |
| the builders' bases (`git merge-base` of each builder's tip and its merge's first parent) | each is the driver's tip of its minute; none is main's stale tip but item 4485's, cut while that was still the tip |
| the archives (`split.py --archive`, then `afsctool` over both) | 31 records, nothing to split; three text dumps skipped as before; 94 GB free to 102 |
| the disk (`df -g ~`) | 94 GB at the start; 102 after the archives; 109 after `cargo clean --profile dev` (12.7 GiB); 106 with the pass's builds |
| the week (`lore limits`, the reading of 04:45 −05:00) | 14 %, reset at 06:00 −05:00 on 2026-11-21; Fable's week 5 % |
| the suite re-timed (`suite_timings.py --stale-only`) | 24 tests timed: 2,629 tests, 6,235 s one at a time |
| the rubric (`rubric.py --record`) | every figure the pass before's: ported 738, replaced 331, open 571 |
| the rows and trees left | no lane row stood; the old driver row reaped by `handover.py reap ed650492`; Codex's branch is not ahead of HEAD; 272 `worktree-att-*` branches on origin, which are Ramon's |

## Rulings

- **The sample's seat was stopped against its rule and reopens** (entry
  151 §1). All four of its landings moved the `Parity:` line. The count
  of games whole is never the number.
- **The fifth battery is spent and folds: 39 of 47** (§2). It was spent
  at item 4534. A guard reads the row tests against the spent list.
- **Entry 150 §1 to §3 are confirmed** (§2), as Ramon asked.
- **The gate asks for the trunk** (§3): exit 92 while `origin/main` is
  behind the pushed branch.
- **The handoff's bound is 36, ratified, and the pass's to move** (§4).
  The queue's preamble is read against the constants.
- **Four seats** (§5): the sample's on item 4543, the chain's on row
  110 (4541), parity seats on rows 63 (4542) and 103 (4544). Rows 83,
  123, 43 and 3 wait behind them in that order.
- **A fifth row's take keeps every line it parts on, which is eleven of
  twelve** (§5). Entry 148 §4's rule stands as written.
- **The sixth battery is rows 13 to 153**, named before any take, taken
  when the fifth reads 4 of 8 (§6). It adds no line to the handoff.
- **The eleven parked findings stay in their journals** (§8). None
  names a score. Two closed in the tranche: 4527a by item 4534, 4528a
  by §2's guard.
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche.

## What the pass looked at

Nothing was drawn. The tranche has no render landing: all twenty are
rules items. The pairs of 2026-11-20 were not redrawn or uploaded.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4543 the sample, units 6108 | rules, whole; the sample's | `Parity:`, units 6108 | the leaf on run1772, no take | two landings running leave the first parting and the draw word as they are |
| 4541 second, row 110 | rules, whole; the chain's | the second's 7 → 8 of 8 | the leaves on run1742, no take | two landings running move no column |
| 4542 fifth, row 63 | rules, whole; a parity seat's | row 63's column, eleven lines | score it again, then a take, `RNG_MASK=0xffbu`, `--allow-early-end` | two landings running move no column |
| 4544 fifth, row 103 | rules, whole; a parity seat's | row 103's column, eleven lines | the same, after 4542's on the game lane | two landings running move no column |
| 4545 fifth, row 83 | rules, whole; unseated | row 83's column, eleven lines | score it again, then a take, `0xffbu` | two landings running move no column |
| 4546 fifth, row 123 | rules, whole; unseated | row 123's column, nine lines | score it again, then a take, `0xfe3u` | two landings running move no column |
| 4547 fifth, row 43 | rules, whole; unseated | row 43's column, eleven lines | score it again, then a take, `0xffbu` | two landings running move no column |
| 4548 fifth, row 3 | rules, whole; unseated | row 3's column, eleven lines | score it again, then a take, `0xffbu` | two landings running move no column |

## Built

- `held_out::a_battery_with_a_row_s_leaf_test_is_spent`, seen red on
  the handed-over tree naming row 23; `SPENT` takes the fifth.
- `tools/handover.py`: `trunk_behind`, `trunk_line`, the `Trunk:` line
  of `check` and its exit 5; `tools/gate_run.sh` writes `gate exit 92`.
  Two tests in `tools/explore/test_handover.py`, seen red, one on a
  repository and a bare origin made there.
- `docs_guard::the_queue_names_the_bounds_the_guard_holds`, seen red on
  the preamble's 33.
- The paperwork: entry 151 and its index row, this record, the queue's
  handoff, lanes line and eight items, the loop's four clauses, the
  tracks' paragraph, the journal's entry, the suite's timings, the
  rubric's row.

## Not done, and said

- **No leaf is read for any fifth row.** All six hold sums. Each item
  takes one take.
- **Why the fast-forward of main stopped at midnight is not
  established.** The branch was pushed after each later gate.
- **Why a landing's wait on its own gate rose for a third tranche is
  measured, not explained**: 30.5, 39.6 and 54.8 minutes. Every lane
  held a rules item that moves the sim, and each landing re-pins what
  the others then re-gate.
- **`tools/ctx.py` still reads the driver's context from a builder**
  (4530). No probe was run to find what a subagent's own id is.
- **The readers of `leaders.defeated` are not swept** (4529, 4525a).
  Rows 63 and 103 end on a defeat our walk does not reach.
- **The closed form for the whole batteries' lines is not built.** The
  sixth's item builds it.
- **Whether a scripted human can be recorded on a held-out lobby is
  unread.** Step 4 mints a person's orders on one lobby; the
  recommendation to Ramon rests on that alone.
- **A fourth seat is unmeasured**, and no donor holds one (4527b).
- **The three text dumps over the compressor's limit stay raw**, as
  before.
- **The driver's own cost is not summed**, as before.
- **The blind list was not scanned**: open 0.
- **The handed-over queue's lead said every gate was green.** Two were
  red and mended; the lead is rewritten.
- **The guards caught the pass twice**: the handoff and the tracks'
  section were each over the ceiling, 264 and 682 bytes. The handoff's
  lead was shortened. The tracks' paragraphs for the passes of
  2026-11-16, second and third, were retired; their story is entries
  137 and 138 and the journal's.

## The gate

Whole, by `tools/gate_run.sh`.
