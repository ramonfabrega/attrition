# The pass of 2026-11-16: the wake that fired and did not spawn, the clock that drained the host on every gate, and the two seats' items

*2026-11-16, from 09:07 −05:00, the `steer` row on Fable 5.2, called at a
supply boundary. `commander` (Opus 5.5) handed over at `03dfe1a18` after
ten merges, 22:12 on 2026-11-15 to 02:24 on 2026-11-16, gated whole
(`gate-drain-1116.log`). The charter was the handoff's "For the pass" (two
design calls, a trade, the gate table's start parse) and one thing more:
the morning's wake did not fire. Decisions in entry 136. The pass cut one
in-session builder on Opus 5.5, spawned and reaped two throwaway rows as a
probe, and took no capture. The pass before it is
`2026-11-15-fable-pass-driver-lines.md`.*

## For Ramon

- **The wake:** it was armed and it fired. `handover.py wake` then died on
  ccc 0.1.50 refusing to spawn `commander` from `commander`; nobody was
  told, and the loop sat from 06:07 until you woke it near 09:03. Mended,
  and a failed wake now sends you a push notification. Tomorrow's wake is
  the first real test of it.
- **The tranche:** ten landings in 4.2 hours for three points of the week
  (25 → 28 %), 72 USD at list (lanes 39.52, four builders 24.27, the
  driver 8.37), every booking gate green.
- **What the pass changed:** the booking gate's clock step had waited 60.6
  of eight gates' 193.3 minutes for the host to drain; it now drains only
  after a red. A gate log says when it began.
- **Decided, yours to reverse** (entry 136): `leader_options` comes from
  the client's own prefs with no control drawn; the land's light is taken
  on the plain ground, with run1564's sand and surf and both plumes
  falling by name; the auto-explored file's playback is a rules item.
- **A count to read with care:** `INPUT_WHOLE` will read 42 of 50 after
  item 4289, and 38 of those 42 recordings hold no order but the one every
  file carries. The orders are the count: 363 of 2,429 shown, 990 missed.
- **Yours, carried:** ten minutes playing the client; the merged
  branches on origin (62 of 66); the README's line 12.

## What was measured

| what | measured |
|---|---|
| the merges since the pass before (`git rev-list --first-parent --merges c1300355f..03dfe1a18`) | 10: six lanes' items (4283 to 4288) and four in-session builders (4159r12a, r12b, r12c, r15a) |
| the lanes (`tools/tranche.py --base c1300355f`, after `lore index`) | 6 landings, 39.52 USD, 6.59 a landing (6.67 the tranche before); a lane's clock 48.3 minutes, 19.0 working and 29.2 waiting, 17.3 of them on its own gate; none past 281 k; 3 captures |
| the builders (`lore spawns --session 0cc9a398`) | 4, all Opus 5.5, 453 requests, 24.27 USD |
| the driver (`lore api job`, the `commander` job) | 142 requests over two sessions, 8.37 USD; 243 k at the drain's gate |
| the gates (`tools/gates.py ~/Library/Caches/attrition/gates --since gate-pass-1115c.log`, after this pass's fix) | 9, 219.4 minutes: 1 whole, 8 scoped (7 widened), 0 red; admission 7,780.5 s summed |
| the clock step on the eight scoped gates (each log's `Gate minutes:` and its `"admission": "drain"` line) | the drained lease waited 157, 0, 50, 774, 80, 1,494, 0 and 1,082 s: 60.6 of 193.3 minutes; the test ran 8 s each time |
| gate-4159r12b and -r12c | 43.5 and 33.8 minutes (the table read 443.1 and 481.9; the handoff said about twenty); both logs born 2026-11-15 17:34 and 17:48, written again at 00:14 and 01:16 |
| the wake (the cleared session's transcript, `c7c06f04`) | the `--then` prompt typed 02:55:52; `CronCreate` 02:56:04, one-shot `7 6 16 11 *`; fired 06:07:00; `handover.py wake` exit 1 at 06:07:11 on ccc's refusal of the name; no row spawned |
| the probe (two throwaway rows in this tree, Haiku 5.5, reaped) | a second `ccc spawn` under a live row's name: exit 1, the morning's sentence; with `--allow-duplicate`: exit 0 |
| the week (`lore limits`) | 28 % at 09:13 on 2026-11-16 (25 at the tranche's start); 27.72 USD a point all-in; Fable's week 10 % |
| the suite re-timed (`suite_timings.py --stale-only`) | 7 stale tests timed: 2,471 tests, 4,981 s one at a time |
| the disk (`df -g ~`) | 202 GB at the start, 204 after `afsctool` over both archives; the debug profile 5 GB and left |
| the rubric (`rubric.py --record`) | every figure the pass before's: the territory 736 · 331 · 572, 44 of 112 groups whole, 1,456 of 3,611 backed, ages 8 of 8, unit types 296 of 302, lockstep bytes 78.1 % |
| the rows and trees left | no lane row and no `agent-*` tree: the drain's gate had reaped the last |
| the blind list | never 345 · residue 336 · open 9, the same nine rows the pass before scanned; not scanned again |

## Rulings

- **A wake into its own row says so, and a failed wake reaches Ramon**
  (entry 136 §1). Built. The handoff's hypothesis, that ccc typed no
  prompt onto the cleared row, was wrong: the prompt was typed and the
  cron fired.
- **The clock drains the host only after a red beside load** (§2). Built
  by a builder. This is the finding the charter did not carry: the pass
  before named the drain's wait as unmeasured, and the logs held it.
- **A gate log says when it began** (§3), the handoff's fourth. Built.
- **`leader_options` is the client's own prefs** (§4), the handoff's
  first. A lobby field was the offer; the original takes the order from
  the profile, and a control whose other settings the sim ignores is
  worse than none. Item 4289.
- **`INPUT_WHOLE` is booked with its orders beside it** (§4). A count of
  files, most of them empty, is not a person's whole game.
- **The panel's roster next** (§5). Item 4290.
- **The auto-explored file's playback** (§6), the handoff's second: a
  rules item, whole, the take first. Item 4291.
- **The land is lit** (§7), the handoff's third. Looked at first. Item
  4292.
- **The journals' "for the Loop" lines** (§8): three, each method for its
  own kind; they stay where they are.
- **Marked rows:** the tranche took no second reading and left no
  `FABLE:` marker; nothing to ratify.

## What the pass looked at

`client --frame 3279 --still`, with and without `--land-light`, beside
run1559b's photograph (`~/ron-data/lab-experiments/2026-11-16-pass/`:
`triple-1559b-half.png`, and `crop-west-foot.png` at 1:1). At the range's
west foot the photograph's shadow fades smoothly onto the grass. With the
switch off ours draws a stair along tile edges where lit tiles under the
range meet flat ones. With it on ours draws the photograph's fade, and the
grass loses its flat yellow. Nothing in the frame reads worse with it on.
run1564 and run1570 were not drawn; the lane's own table carries them.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4159 the leaders line | the chain's builders | `LEADERS_HELD_OUT` 9 | 4159r5a, then the stamps | as each row says |
| 4289 `leader_options` from the client's prefs | input, scoped | `INPUT_KINDS` 25 → 26 of 26; `INPUT_WHOLE` 0 → 42 of 50 with its orders beside it | whether the prefix's fresh profile is the original's default | a byte of the payload is one the reading does not explain |
| 4290 the panel's whole roster | input, scoped | `INPUT_WHOLE_ORDERS` missed 990, 432 of them a type no key binds | the census of recorded types against the panel's buttons | a type's button is on no panel the HUD draws |
| 4291 the auto-explored file, played | rules, whole | `Recording:` gains the auto-explore's frames | the take of the minted file, per frame | the first parted draw is none of the five readers of the bit |
| 4292 the land's light on the plain ground | render, scoped | run1559b's luma 1,072 → 1,109 of 1,109; `_TRACED_STILL` 32 → 33 | the three pairs beside their photographs | a row falls that the lane's table did not name |

## Built

By the pass, in this session, each red first.

- **The spawn into its own name** (`tools/handover.py`, `spawn_argv`).
  Red: no such function; and the live refusal, reproduced.
- **The gate's start** (`tools/gate_run.sh`, the `Gate start:` line;
  `tools/gates.py`, the start, the steps' sum for an older log, and a
  `clk` column). Red: two cases on a log whose name was written before,
  each reading 443.1 where 20.0 and 43.5 were asked.
- **The wake's prompt** (`docs/LOOP.md`, "A wake is a spawn"): a push
  notification on a non-zero exit. A clause; the prompt is typed by the
  driver that arms it.

By one in-session builder on Opus 5.5, merged by the pass:

- **The clock beside load first** (`tools/release_gate.py`, `_run_clock`,
  `CLOCK_BESIDE_LOAD`, two slots and 2 GiB; a `Clock:` line). Red: five
  cases, on names that did not exist and on one clock call where two were
  asked. The builder broke the finished code twice to see each case fail.
  It ran 8.9 minutes on 125 k tokens.

## Not done, and said

- **The wake from `commander` itself is not exercised.** The probe used a
  throwaway name; the first real one is the next morning's.
- **The push notification is a line of a prompt.** No tool sends it, and a
  wake that fires into a row whose model ignores the line is silent again.
- **A red beside load can cost more than the drain did.** The test's own
  rule waits up to 600 s for quiet before its second session, and the
  drain follows. No gate has run it.
- **What held a lease for 25 minutes is not named.** A lane's or a
  builder's whole test run is the likeliest; no log says which.
- **The look was one photograph of three.**
- **The blind list was not scanned again**, on its nine rows being the
  pass before's.
- **The debug profile was not cleaned**: 5 GB.
- **`tools/tranche.py` tables no in-session builder** (`4159: no session`):
  their price is the driver job's `spawnUsd`.
- **Carried:** whether a builder's handback survives its driver's clear;
  the cut past 350 k is prose; no guard reads a count in the handoff's
  prose against its pin.
