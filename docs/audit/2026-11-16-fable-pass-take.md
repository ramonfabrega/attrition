# The pass of 2026-11-16, third: the demand's own game, parity's leaders line, and the panel on held packets

*2026-11-16, from 21:14 −05:00, the `steer` row on Fable 5.2, called by
the backstop. `commander` (Opus 5.5) handed over at `d2b00ab36` after
twenty first-parent merges, 15:58 to 20:40, drained and gated whole
(`gate-drain-1116c.log`). The charter was the handoff's: score the twenty,
design item 4316 (the last held leaf) and item 4315 (the panel's filter),
and refill the seats. Decisions in entry 138. The pass spawned one
in-session builder on Opus 5.5 (verified from its transcript: 135 records,
all `claude-opus-5-5`), which took one capture. The pass before it is
`2026-11-16-fable-pass-mover.md`.*

## For Ramon

- **The last leaf has its game now.** The chain was walking a game on
  which the computer never demands tribute. I had the real one captured
  tonight: you allied with the French, on Tough. The French ask for 700
  wealth on frame 6081, as the emulator predicted. Our sim follows that
  game to frame 2082 of the 6,081 it needs.
- **The milestone and parity are two different finish lines, and the
  queue had merged them.** When that leaf is released, the checksum
  agrees on the one frame the playback prints, which closes the
  milestone's step 5. The leaders line still differs from the original's
  on 152 of the first 300 frames of the minted game, on fields nobody
  held an item for. That is now item 4317.
- **Yours to weigh:** whether the milestone is called at step 5's close,
  or held until the leaders line agrees on all 24,000 frames. I booked
  both and did not choose.
- **The tranche:** twenty merges in 4.7 hours for seven points of the week
  (32 → 39 %), 157.06 USD at list (lanes 116.19, the driver 17.13, its
  four builders 23.74). Three red gates of fifteen: two were the
  `syspolicyd` fault you cleared, one a queue line.
- **The panel needs no capture.** Sixteen snapshots of the minted game
  were already on disk; the original's build panel was run on four.
- **Yours, carried:** ten minutes playing the client; the merged branches
  on origin; the README's line 12.

## What was measured

| what | measured |
|---|---|
| the merges since the pass before (`git rev-list --first-parent --merges 4446872ec..d2b00ab36`) | 20: sixteen lanes' (4298 twice, 4299, 4300b1, 4301 to 4308, 4311 to 4314) and four in-session builders' (4159r19, 4300's first act, 4300a, 4300b) |
| the lanes (`tools/tranche.py --base 4446872ec`, after `lore index` and the parser's mend) | 15 items over 17 sessions, 116.19 USD, 7.75 an item, 6.83 a session; a session's clock 47.9 minutes, 18.0 working and 29.9 waiting, 19.6 of them on its own gate; three past 300 k (4298b at 408 k, 4314 at 332 k, 4313 at 322 k); 4 captures |
| the driver and its builders (`lore api job commander`) | 257 requests, 17.13 USD; four builders 23.74 USD |
| the gates (`tools/gates.py ~/Library/Caches/attrition/gates --since gate-pass-1116e.log`) | 15, 267.4 minutes: 1 whole, 14 scoped (13 widened), 3 red (gate-4298a and gate-4299 on `memcap`, the host's fault of entry 137 §5; gate-4300a on the queue's checksum line); admission 10,843.1 s summed, longest 615.1 s; 4 ended past the cores |
| the drain's gate (`gate-drain-1116c.log`) | `gate exit 0` on `d2b00ab36`, 9 of 9 steps, 27.9 minutes, release 22.9; `Clock: green beside load`; HEAD and `origin/main` both `d2b00ab36` |
| the week (`lore limits`) | 39 % at 21:12 and at 21:35 (32 at the tranche's start; the day's line 42); 27.32 USD a point all-in; Fable's week 15 % |
| recon4316-1 (the pass's builder; `~/Library/Caches/attrition/pass-1116d/recon4316.md`) | run1575's lobby at `DIFFICULTY 4` with `tools/gamelog/coverage/alliance4316.cmd`, to 6,600, 400.0 s, exit 0, no kill; the alliance set on 902 and `diplos` 2 both ways on every block from 5,900 |
| the leaf on it (the pass, `rg` on the gamelog) | `tribute_demanded[scan] 1` on 520 rows, blocks 6081 to 6600; `[scan] 0` on 27,520; no row at 1 in recon4300-2 |
| the rows beside it on 6081 (the builder's reader) | French `aggression[0]` 0 → 1, `agendas[0]` 64 → 72, `dip[0]` `agree` 1, `treaty` −1, `offers[WEALTH]` −700; the human's `offers[WEALTH]` +700, `agendas[1]` 4; `diplos` 2, `blacken` 0 |
| our walk of it (`recon4300_walked_at_its_difficulty`, `RON_RECON_SCRIPT`) | `count 2082 sequence 2082 of 6600`, the head word agreeing on every frame walked; on 2082 ours 3,924 draws, theirs 3,907, at 3,894: ours `PathFinder::calc_road_cost+0x46`, theirs `Guy::set_anim+0x97a < Guy::inc_time+0x271`; with no script staged it parts on 957 |
| the two games (`tools/gamelog/rngcmp.py`, recon4300-2 against recon4316-1) | 6,601 frames in common, harness word 957, identical through 973 but for 958 |
| new on recon4316-1 | the Russians at war with the human by 5,900 (`diplos` 0 both ways, the human's `wars` 1); `Leader::action_declare` first entered on 5496 (4984 on recon4300-2) |
| the leaders line on the minted game (`CHECKSUM_FRAMES`, `MINTED_PACKETS`, read in the source) | run1578: first parting 0, 152 of the first 300 frames (0, 100–109, 159–299); whole on the packets at 1, 61, 129, 987, 2745, 2746, 3246; parting on 100 and 3166 (`128 1 flags: +0x2`, theirs `80`), 189 (`0 0 flags: +0x2`, theirs `08`), 3596 (`+0x5d0`, ours 2, theirs 0), 6537 and 6781 (`flags: +0x3`, `+0x2`), and `+0x850`, `+0x5d8` on the record's frames |
| the minted game's packets on disk (`find ~/ron-golden -name frame-snapshot.bin`) | 16 of 133: frames 1, 61, 100, 129, 189, 987, 2745, 2746, 3166, 3246, 3596, 4030, 6537, 6781, 6907, 7195 |
| the panel's oracle on four of them (`tools/recomp/panels.py`, TSVs under `~/Library/Caches/attrition/pass-1116d/panels/`) | every run returned; 3596 and 4030: 7 buildings offering; 6907: 8 of 9; 7195: 10 of 10; the Barracks `[69, 82, 132, 177]` and 4 research, all 4 disabled; a Library 35 research, 30 to 32 disabled |
| the input census (`INPUT_WHOLE_ORDERS`, read in the source) | 987 shown, 1,442 of the layer's form, 0 missed, of 2,429; `INPUT_WHOLE` 42 of 50 |
| the suite re-timed (`suite_timings.py --stale-only`) | 9 stale tests timed: 2,487 tests, 5,020 s one at a time |
| the disk (`df -g ~`) | 188 GB at the start; 193 after `cargo clean --profile dev` (9.6 GiB); 200 after `afsctool` over both archives and the recon's 1.6 GB |
| the rubric (`rubric.py --record`) | every figure the pass before's: 736 · 331 · 572, 44 of 112 groups whole, 1,456 of 3,611 backed, ages 8 of 8, unit types 296 of 302, lockstep bytes 78.1 % |
| the rows and trees left | no lane row (`ccc list`); `reap_agents.py --yes` printed nothing at the start; the pass's builder's tree is merged |

## Rulings

- **The demand is chased on its own game** (entry 138 §1). recon4316-1 is
  the take; recon4300-2's 4202 stays a floor; a mover's game is captured
  before a walk is chased toward it.
- **Parity's leaders line is item 4317** (§2), on the suite's seat. The
  milestone's step 5 closes on the leaf; parity does not.
- **The panel's filter is diffed on the minted packets** (§3). No capture.
- **The input seat's next is the form orders** (§4), a census first.
- **The table reads the driver's subjects** (§5).
- **Three journal lines became one clause of the render brief** (§6).
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche; nothing to ratify.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4316 the demand's walk, on the take | rules, whole; the chain's | `LEADERS_HELD_OUT` 1 → 0; the walk 2082 → 6,081 | file recon4316-1; the value diff on 2082 | two acts move the word by nothing, or a parting needs an arm no document holds |
| 4317 parity's leaders line on the minted game | rules, whole; the suite's seat | a row minted first: frames agreeing of 24,000 (152 of the first 300 part) | Step R's record on 0, 100 and 159 | a field's writer is interface code the sim does not hold |
| 4315 the panel's filter | input, scoped | `PANEL_LEADER` re-minted over the held packets, 0 of its buildings equal → all | `panels.py` over the sixteen; the row re-minted | a predicate reads leader state the sim lacks (then it says `crates/sim`) |
| 4318 the form orders shown | input, scoped | `INPUT_WHOLE_ORDERS` form 1,442 → less; `INPUT_WHOLE` 42 → 50 | the census: point reachable, group selectable | |
| 4309, 4310 | render, scoped | as the pass before booked them | | |

## Built

- `tools/tranche.py`: `booked_text` reads `QUEUE: book N (…)` and `The
  drain …: book N`, and an item's step letter; red first in
  `tools/explore/test_tranche.py` (`- []` against six items).
- `tools/brief/frame.md`: one clause on the render oracle's row.
- `tools/gamelog/coverage/alliance4316.cmd`: the take's three lines.
- Merged from the builder (`9154f98e2`): the recon probe stages
  `RON_RECON_SCRIPT`.

## Not done, and said

- **No render step was designed, so the pass did not look.** 4309 and
  4310 stand as the pass before booked them.
- **Recon B was not taken**: one human and one computer at `DIFFICULTY
  2`, which item 4300's second act predicts demands with no click. The
  builder stopped at its context bar. It is the fallback if the take's
  walk stalls, and which profile key seats two is not established.
- **Nothing on recon4316-1 is computed by us yet.** The leaf's rows are
  the original's; our walk ends 3,999 frames short of them.
- **The leaders line past frame 300 is not counted.** The 152 is `DIAG`'s
  window; item 4317 mints the 24,000-frame row.
- **The blind list was not scanned again**: open 9, the input track's
  issuers.
- **A lane waited 15.2 minutes on `ScheduleWakeup`** (4298b's second
  session, at 408 k). Not ruled on.
- **Carried:** the wake from `commander` itself is unexercised; a red
  clock beside load is unrun; whether a builder's handback survives its
  driver's clear; `tools/tranche.py` tables no in-session builder.

## The gate

Whole, by `tools/gate_run.sh`; quoted here once it has run.
