# The pass of 2026-11-16, second: the mover for the leaders line's last six leaves, and the two seats' items

*2026-11-16, from 14:13 −05:00, the `steer` row on Fable 5.2, called by
supply. `commander` (Opus 5.5) handed over at `32ee8bba1` after twelve
merges, 10:07 to 13:41, drained and gated whole (`gate-drain-1116b.log`).
The charter was the handoff's "For the pass": design the mover for item
4159 (parked 4159r18b), the input and render seats' next items, and the
lanes line. Decisions in entry 137. The pass spawned two read-only scouts
on Opus 5.5, cut no builder and took no capture. The pass before it is
`2026-11-16-fable-pass-wake.md`.*

## For Ramon

- **The milestone's last line is four steps away, and the first needs no
  capture.** Six leaves are still held out of the leaders line. One moves
  on a rehearsal that was already on disk; three move on one diplomacy
  deal; one needs you seated in another lobby slot; the last needs the
  computer to demand tribute, which happens only in a Diplomacy-style
  game past frame 6,000.
- **Yours to weigh, after the measurement:** that last leaf
  (`tribute_demanded`) may cost a port of the computer's style-0
  diplomacy, five random draws deep. I booked two short recons that
  measure it before anything is built. If it is large, the choice is
  whether the milestone waits for it or is called with that one leaf held
  by name. I did not make that choice.
- **The tranche:** twelve merges in 3.6 hours for three points of the
  week (29 → 32 %), 74.54 USD at list, one red gate (a test fixture, not
  the code).
- **The host:** since about 13:00 this Mac kills some executables that
  were rewritten in place and run again. It still happens (4 in about 60
  trials at 14:25) and I could not find the cause. If you installed or
  updated anything around 13:00, that is the suspect. The capture driver
  itself ran a probe script.
- **What the pass saw in the picture:** no national border is drawn
  anywhere, the ground past the map's edge is black, and the lake's fish
  are white. All three are booked, borders first.
- **Yours, carried:** ten minutes playing the client; the merged branches
  on origin; the README's line 12.

## What was measured

| what | measured |
|---|---|
| the merges since the pass before (`git rev-list --first-parent --merges c72bf06ac..32ee8bba1`) | 12: nine lanes' items (4289 to 4297) and three in-session builders (4159 residue 17, residue 18, the `test_viadriver` fixture) |
| the lanes (`tools/tranche.py --base c72bf06ac`, after `lore index`) | 9 landings, 47.92 USD, 5.32 a landing (6.59 the tranche before); a lane's clock 37.4 minutes, 13.5 working and 23.9 waiting, 14.8 of them on its own gate; none past 259 k; 1 capture |
| the driver and its builders (`lore api job commander`) | 223 requests, 13.52 USD; three builders 13.10 USD |
| the gates (`tools/gates.py ~/Library/Caches/attrition/gates --since gate-pass-1116b.log`) | 9, 161.4 minutes: 1 whole, 8 scoped (8 widened), 1 red (gate-4297, the fixture, 3.1 minutes); admission 5,451.1 s summed, longest 580.0 s; the clock step 0.1 to 0.2 minutes on every gate |
| the six held leaves on disk (scout, `rg` over 3,683 gamelogs in both archives) | 1,660 print the diplomacy rows, 2,015 print `tributes`; one file moves one leaf: `~/ron-golden/recon4206-4/map-18/gamelog.txt`, the French `good_deeds[0]` 1328 on blocks 1032 to 1150 (119 lines, re-counted by the pass); `hire_stamp`, `hire_who`, `tribute_demanded` and `tributes` never leave their reset in any file |
| the instrument (the same pattern on run1575's log) | 38,250 zero lines and 10,200 `hire_who` −1 lines match; the non-reset pattern none |
| recon4206-4 against run1575 (`tools/gamelog/rngcmp.py`, the pass) | 1,151 frames in common, 49 differing, the first 1100; harness word 1099 |
| recon4206-4's commands (its `COMMANDMANAGER` lines and `rontrace.cmd`) | run1575's through `process_treaty 1 1 1021`, then `process_tribute 1 4` on 1024 to 1027 (four clicks at (858, 405) under a held shift), `process_accept 1 1031`, `process_treaty 1 2 1061` |
| style-0 games on disk (scout) | 17 logs, all map 18; none past frame 1,150 |
| the human's seat (scout) | who 0 on 5,027 PLAYER records of 2,458 logs and in our 49 recordings; who 3 in the 2017 sample, alone on its team |
| the arms no walk enters (scout, listing ranges of the computer's diplomacy pass) | the attack terms 262 instructions, no draw; the demand arm 548, two draw sites; the short-with-a-give arm 327, one draw; the top-scorer coin 89, one draw; the alliance proposal naming a target 351, no draw; the peace proposal 290, one draw. Four of the pass's nine draws are ported |
| the gates on run1575's lobby (scout, one reading each) | the demand's frame gate 6,000 (who 1) and 6,300 (who 2); the computer's alliance proposal 19,800 and 26,100; `aggression` 0 between all three leaders on every frame of run1575 |
| the host's kill (the pass, 14:25) | a script written, run, rewritten in place and run again from Python: 1 of 15 and 1 of 15 pairs, then 2 of 22 and 1 of 1, exit −9; from a shell loop 0 of 10; nothing named in the readable system log |
| the capture driver (`file`, and a probe through `viadriver.sh` at 14:23) | a Mach-O arm64 binary dated 2026-09-03; the probe script ran and printed its line |
| the week (`lore limits`) | 32 % at 14:12 and at 14:25 (29 at the tranche's start); 27.06 USD a point all-in; Fable's week 12 → 13 % |
| the suite re-timed (`suite_timings.py --stale-only`) | 7 stale tests timed: 2,478 tests, 4,985 s one at a time |
| the disk (`df -g ~`) | 193 GB at the start; 198 after `cargo clean --profile dev` (10.0 GiB); 206 after `afsctool` over both archives |
| the rubric (`rubric.py --record`) | every figure the pass before's: the territory 736 · 331 · 572, 44 of 112 groups whole, 1,456 of 3,611 backed, ages 8 of 8, unit types 296 of 302, lockstep bytes 78.1 % |
| the rows and trees left | no lane row and no `agent-*` tree (`ccc list`, `git worktree list`, `reap_agents.py --yes` printed nothing) |

The scouts' reports, with their commands and quoted lines, are
`~/Library/Caches/attrition/pass-1116c/scout-disk.md` and
`scout-arms.md`. Both ran on Opus 5.5 (11 and 14 minutes, 147 k and 219 k
tokens).

## Rulings

- **The six take four movers, cheapest first** (entry 137 §1). The
  handoff said a mover needs a game the disk does not hold. For
  `good_deeds` the disk holds it: an unfiled rehearsal of run1575's
  script. The other five need three new games.
- **Entry 131 §1 stands.** A release for a leaf "frozen under the lobby's
  style" was weighed and refused.
- **A released leaf is compared by the record diff** (§2): parked
  4159r18a rides with the first release.
- **The input seat** (§3): the four arms by orders, then the panel's
  predicates with its row minted first.
- **The render seat** (§4): borders, the map's edge, the fish, the decal,
  the deep tiles.
- **The host's kill** (§5): the two fixture mends stand; the cause is not
  established; the first capture is the test.
- **Marked rows:** the tranche took no second reading and left no
  `FABLE:` marker; nothing to ratify. No journal carries a "for the Loop"
  line.

## What the pass looked at

The client's still at the tip, at each photograph's camera, stacked under
the photograph (`~/ron-data/lab-experiments/2026-11-16-pass-2/`:
`pair-<run>-half.png`, six `crop-*.png` at 1:1, and `lobby-600-half.png`,
the minted lobby's frame 600).

- **run1559b and run1570.** The land, the mountain, the trees, the city
  and the figures read as the photograph's at half scale and at 1:1. The
  ground past the map's edge is black where theirs is dark terrain. The
  photograph has a pale beam over the mounted flag-bearer, dust under the
  moving tanks, muzzle flashes and a red trim on the roofs that ours
  lacks. The red line round the plaza is the selection's, and our still
  selects nothing.
- **run1564.** The lake is lighter than theirs and has few whitecaps. Its
  fish are white shapes on the surface where theirs are dark shapes under
  it beside a dark blot. A lavender band crosses the photograph's east
  edge, a national border, and ours draws none. A patch of cracked ground
  with dark pits south-east of the lake has plain grass under it in ours.
  The south shore's sand and grass agree.
- **The minted lobby at 600.** The player's capital sits by the map's
  corner with a quarter of the screen black beside it, no border round
  it.

The photographs are of frames 3294 and 3331 and the stills of 3279 and
3327, as the rows read them, so a unit's place differs by fifteen frames
and was not compared.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4159r19 `good_deeds` off the rehearsal | the chain's builder | `LEADERS_HELD_OUT` 6 → 5 | file recon4206-4, walk it | ours is not 1328 |
| 4298 the attack deal | rules, whole | 3 leaves | a recon photographing the negotiation window | no rehearsal is accepted |
| 4299 a taunt from a reseated human | rules, whole | 1 leaf | a recon with `LAST_WHO0` set | the key does not seat the human |
| 4300 the demand | rules, whole | 1 leaf | two recons to 7,000 at two difficulties | (nothing is built before the driver reads both) |
| 4301 `gather_point` on an object | input, scoped | missed 118 → 70 | the census of the 48 | an action's issuer is not the right press |
| 4302 two issues in one turn | input, scoped | missed, 38 less | the recorded turns' bytes | |
| 4303 `spell` on an object | input, scoped | missed, 22 less | the census of the 22 | |
| 4304 `build` along a dragged line | input, scoped | missed, 10 less, to 0 | the census of the 10 | |
| 4305 the panel's leader predicates | input, scoped | a row minted first | find the oracle for the offered options | no state both sides hold names a panel |
| 4306 the national borders | render, scoped | a row minted at 0 | the crop beside the sim's border cells | the band is off the sim's cells |
| 4307 past the map's edge | render, scoped | the pinned row, 0 of 162 | the card probe | |
| 4308 the lake's fish | render, scoped | texture ocean 234 of 437 | the crop at 1:1 | |
| 4309 run1564's cracked ground | render, scoped | a row minted at 0 | the crop, and what draws there | |
| 4310 the deep tiles | render, scoped | sandy 650 → 653, surf 688 → 702 | the nine tiles' water | |

## Built

Nothing. The pass changed no tool and no code.

## Not done, and said

- **The value 1328 is not computed here.** The pass showed the rehearsal
  is run1575's game through the mover; whether our valuation gives 1328
  for 2,000 metal is the builder's first number.
- **Three of the four movers rest on one reading each**: that a deal with
  an attack request can be accepted at peace, that `LAST_WHO0` seats the
  human, that a harder difficulty turns `aggression` on. Each item's
  first act is the recon that tests its own.
- **The demand's real size is unknown.** A style-0 walk to 6,000 frames
  may need every arm in the table above.
- **A game at Diplomacy style cannot be played in our client**: the nine
  diplomacy orders are minted by nothing and no recording carries one, so
  no input row counts them. Not booked.
- **The host's kill is not explained**, and no guard keeps a tool's test
  from rewriting an executable in place.
- **`INPUT_WHOLE` will not move** on the five input items; its next step,
  a form order shown with its point and group, is not designed.
- **The look compared no unit's place** (the frames differ), and the
  window with its HUD was not drawn.
- **The blind list was not scanned again**: open 9, the rows of the two
  passes before.
- **Carried from the pass before:** the wake from `commander` itself is
  unexercised; a red clock beside load is unrun; whether a builder's
  handback survives its driver's clear; `tools/tranche.py` tables no
  in-session builder.
