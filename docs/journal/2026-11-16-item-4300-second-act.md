## 2026-11-16 — item 4300, second act: the war branch named on a packet, and the demand reached by one alliance (Opus 5.5)

*Opus 5.5 (`claude-opus-5-5`), an in-session builder for `commander`, base `3e5d56015` (origin's
`worktree-replan-pdb`). **No score moved.** It took one capture, run1640, with a packet at 6079, and ran the French
pass through `Leader::diplomacy@006bc950` on that packet under a scratch copy of `tools/recomp/step4.py`. Nothing was
built, and nothing in `crates/` changed.*

Lane verdict: not run (measure only; the driver gates at the merge).
Gate minutes: none.

## The capture

run1640 is recon4300-2's line with `--end-frame 6100`, `--log-window 5900 6101` and the packet's two defines
(`RON_STATE_FRAME=6079` and item 597's plan). `docs/RUNS.md` run1640 has the whole line. It ran 395.5 s, `success`,
with no kill. Blocks 5900 to 5902 and 6076 to 6100 are byte-identical to recon4300-2's, so the packet is that game's.
The packet's `game->frame` reads 6079. The French cadence at the human (`P = 0x800 >> 2` = 0x200, since `agendas[0]` is
0x40, `& 8 == 0`) fires when `(frame + 64) & 0x1ff == 0`, that is on 6080. So every run below pokes `game->frame` to
6080 (`--poke e38410=c0170000`) and enters `Leader::diplomacy` with `this` = the French record (`0xe4127c`). Tick 6079
is not emulated: one tick of drift, and none of the arm's operands moves in it.

**The tool.** `step4.py` copied to the job's tmp dir, with these changes:
- The hook line also prints `[ebp-0x10]` (`k`), `-0x30`, `-0x38`, `-0x3c` (the gate frame, then `D`) and `-0x50`
  (the cleared-agreement flag).
- The stack is printed at the stop.
- Stubs added for `MultiByteToWideChar` (reached through a pointer at `7beae394` by `String::char_to_wchar`),
  `wcsncmp` and its kin, `_itow_s`, `wcscpy_s` and its kin.

The chat and sound code under `action_respond` still faults on renderer state. So the demand runs also poke
`console->who` (`0xe8d800`) to 7, which keeps only the human's chat out. Logs:
`~/Library/Caches/attrition/item-4300a/step4-<name>.log`.

## (a) The war branch: local `-0x30`

The arm's predicates in their listing order, with the values the unperturbed run (`step4-f6080.log`) read at `k = 0`:

| site | predicate | read | branch |
|---|---|---|---|
| `6be639` | `is_tribute_period(me, k)` | 0 | on |
| `6be651` | `frame >= gate` | 6080 ≥ 6000 | on |
| `6be65a` | cleared-agreement flag `[ebp-0x50]` (`me.dip[k].agree == 1` or `k.dip[me].agree == 2`, `6be47a`–`6be4a8`) | 0 | on |
| `6be664` | `k.blacken >= 4` (+0x20c) | 0 | on |
| `6be676` | **`[ebp-0x30]`** | **1** | **war, `6be9ae`** |
| `6be680` | `me.hire_stamp[k]` window | not reached (0 in the packet) | — |
| `6be6c8` | `k.bucket[WEALTH] < D` | not reached (20,135 against 700) | — |

**Where `-0x30` comes from** (`6bd8cb`–`6bd937`, the non-allied path only). At `get_diff >= 4`, with `[ebp-0x38]` 0:
`atwar` is 1 if either side's `diplos` is 0, else 0. If `k.wars` (+0x9ac) `<= atwar`, then `-0x30 = 1` unless `me` is a
boomer below epoch 4. The packet reads the human's `wars` 0 and peace both ways, so `atwar` is 0. `is_boomer(French)`
returns 0, and the French epoch is 6. At difficulty 3 it is a draw (`6bdab3`): an odd draw sets it only for a rusher, an
even draw for a non-boomer. At 2 it is set only for a rusher; below 2 the block is skipped.

**The same flag sets `aggression[k]`.** At `6bde26`: `if (-0x30 && frame >= gate) aggression[k] = 1`. In the
unperturbed run the only writes to the French `aggression[0]` are the per-pass zero (`6bcd6f`) and this 1 (`6bde3c`).
So on the non-allied path, the `aggression[k] != 0` that opens the arm and the war branch inside it are the same bit,
and the arm cannot demand. That settles 4300d: `aggression` is recomputed every pass, and 5,900's 0 is the value before
the gate.

**Value diff against the record**, the unperturbed run with the stubs (`step4-baseline-console7.log`) against block 6081:

| field | 6080 | emulated write | 6081 |
|---|---|---|---|
| French `agendas[0]` | 64 | 0x48 = 72 (`6becd1`) | 72 |
| French `aggression[0]` | 0 | 1 (`6bde3c`) | 1 |
| French `diplos[0]` | 1 | 0 (`set_diplo+0x1c9`) | 0 |
| human `diplos[1]` | 1 | 0 (`set_diplo+0x1d0`) | 0 |

The run stops after these writes, on a null read at `00704a3e`, inside the declare's callees.

## (b) The state that demands instead

**One relation, at DIFFICULTY 4 on this packet: the human and France mutually allied.** The pokes are French
`diplos[0]` = 2 and human `diplos[1]` = 2 (`step4-human-ally-fr-only.log`). The allied branch sets `aggression[0] = 1`
at `6bd69c` and `6bd6c8` (the `get_diff > 3`, `k` human, one-other-live-leader store, then the team-score store). The
`-0x30` block is never reached (`6bd8cb` is not hit). The arm then reads `-0x50` 0, `blacken` 0, `-0x30` 0, and
`D` = 700 = (6 + 1)·(0 + 1)·100 against WEALTH 20,135. It takes `6be6d0`. `action_respond`'s entry zeroes
`tribute_demanded[0]` (`6d03d8`). Then **`6be85d` stores `tribute_demanded[0] = 1`**, and the pass returns.

The composed deal (`step4-B-demand.log`): French `dip[0]` = `agree 1, treaty −1, offers[WEALTH] −700`, and human
`dip[1].offers[WEALTH]` = +700. No other good is asked: every French stock that qualifies is at least 1,000. The draws
spent here are the packet's, not tick 6080's, so they move nothing on the path to the store.

Other runs, for the shape:

| run | pokes | `-0x30` | `aggression[0]` source | result |
|---|---|---|---|---|
| f6080 | none | 1 | `6bde3c` | war |
| f6080-wars1 | human `wars` 1 | 0 | none | arm shut (`aggression` 0) |
| diff2 | `DIFFICULTY` 2 | 0 | none | arm shut |
| allied-only | French and Russian `diplos` 2 each way | 1 | `6bd9bb` | war |
| allied-wars1 | the same, plus human `wars` 1 | 0 | `6bd9bb` (France's one non-ally) | **demand**, stored |
| allied-diff2 | French and Russian allied, `DIFFICULTY` 2 | 0 | `6bd9bb` | **demand**, stored |
| allied-diff3 | the same at 3 | 1 (the draw) | `6bd9bb` | war |
| solo-diff2 | Russia's `leader_flags & 3` cleared, `DIFFICULTY` 2 | 0 | `6bd9bb` | **demand**, stored |
| solo-diff4 | Russia cleared | 1 | `6bd9bb` | war |
| human-ally-fr-war-ru | human allied with France, at war with Russia, `wars` 1 | — | `6bd69c` | **demand**, stored |
| human-ally-fr-only | human allied with France | — | `6bd69c` | **demand**, stored |

No single field of the non-allied state demands. The `-0x30` route arms and wars together. The two other non-allied
routes are `6bd9bb` (`get_diff >= 2`, `k` not an ally, `me` has exactly one non-ally, `[ebp-0x70] == 1`) and the hire
contract (`6bdde7`). Each needs `-0x30` kept at 0 by a second condition. `agendas & 0x20` (`6bde20`) is only set beside
a declare. On the allied path, `LAB_006bdb8f` zeroes `aggression[k]` for a human `k` below difficulty 3, so the alliance
route needs 3 or more (`> 3` for the first store, and 3 for the team-score store alone).

## (c) What a human can reach

- **The alliance (recommended).** run1575's own lobby at `DIFFICULTY 4`, with run1575's alliance clicks and nothing
  after: the negotiation window's alliance check (770,499) and PROPOSE (1206,564). run1575 had `ally 1` on 902. The
  human holds 20,000 WEALTH at `STARTING_RESOURCES 7`, far above any `D` (`(epoch + 1)·(blacken + 1)·100`, at most
  900 with `blacken` 0). The gate for who 1 is 6,000. Before the gate, the arm only warns (`LAB_006bed3f`: a chat and
  `agendas |= 0x40`). Not established: that the alliance survives to 6,080 at Tough (the French and Russians are at
  war from 4,984, which helps), and that no tribute stamp makes `is_tribute_period` true.
- **Two seats, Moderate.** One human and one computer, `DIFFICULTY 2`, no clicks. France's only non-ally is the human
  (`6bd9bb`), and a non-rusher sets no `-0x30` at 2. That is solo-diff2 above. Not established: the lobby key that
  seats two, the French personality there, and whether anything declares before 6,000.
- At `DIFFICULTY` 0 or 1 no route opens without a hire: `6bd9bb` and `-0x30` both need 2 or more, and the allied store
  is zeroed. A walk at the lobby's difficulty never meets the leaf.

## Parked

- **4300a1. A recon of the alliance take.** This is run1575's line at `DIFFICULTY 4` with
  `tools/gamelog/coverage/diplomacy4206.cmd` cut after the alliance (no Declares), run to 6,100 and printed from 5,900.
  Expected: the French `tribute_demanded[0]` reads 1 on 6081, and `offers[WEALTH]` reads −700. Whether the alliance
  lasts is the open part. Score: `tribute_demanded` (`LEADERS_HELD_OUT` 5 → 4) once a walk reaches 6,080 on it.
- **4300a2. Every lobby that demands is at `DIFFICULTY` 2 or more.** So the take's walk length is 4300b's question:
  at 4 our walk parts on frame 5 in `Farms::inc_time`. A recon at 2 or 3 says whether the farm parting is a
  difficulty term. Score: the walk's length on a harder lobby.
- **4300a3. The `step4` stubs.** `MultiByteToWideChar` through `7beae394`, `wcsncmp` and kin, `_itow_s` and
  `wcscpy_s` and kin answered a pass that composes a chat. They live in a scratch copy. `tools/recomp/step4.py` could
  carry them, with a test. Score: none (tooling); any later diplomacy pass on a packet needs them.
- **4300a4. run1640's packet in the census.** `docs/RUNS.md` now heads run1640, and its packet is on disk. So
  `diff::checksum_lines::packets::every_packet_is_wired_to_its_game_or_named` needs a `NOT_HELD` row for
  `("run1640", 6079, …)` and `CENSUS` 114 → 115, or it goes red on a machine with `RON_GOLDEN_DIR`. That is
  `crates/`, outside this step's commit. run1640's gamelog and trace are also not filed with `filerun.py`. Score: none
  (the gate's green).
