# The pass of 2026-11-17: the eye on the lake, the panel a player sees, and a person's whole game

*2026-11-17, from 08:57 −05:00, the `steer` row on Fable 5.2, called by
the backstop. `commander` (Opus 5.5) handed over at `690aa7ddf` after
nineteen first-parent merges, 22:46 on 2026-11-16 to 08:25, drained and
gated whole (`gate-1117i.log`). The charter was the handoff's: score the
nineteen, design the third seat and render's next step, and refill the
seats. Decisions in entry 139. The pass spawned no subagent and took no
capture. The pass before it is `2026-11-16-fable-pass-take.md`.*

## For Ramon

- **The two finish lines are about to meet.** The leaders line agrees with
  the original on 23,965 of the minted game's 24,000 frames, from 2,941
  the night before. The last held leaf's walk is at frame 5496 of the
  6,081 it needs. Item 4333 takes the last 35 frames and item 4331 the
  last 585. If both land, the milestone's step 5 and parity on the first
  game close in the same tranche, and the question I left you last night
  (which one to call) answers itself.
- **I stopped the render seat's chase of nine lake tiles.** Three lanes
  in a row moved nothing on them (12.19 USD). I put our picture beside the
  photograph and could not see the difference they were chasing. What I
  could see: the lake's whitecaps are faint where the original's are
  bright white, and the fish are white on top of the water where the
  original's are dark shapes under it. The seat takes those next.
- **What you would see first if you played:** select a Citizen at age 1
  and the build panel offers the Kremlin, the Eiffel Tower and the Space
  Program. The filter that hides them was finished this tranche and agrees
  with the original on 1,238 of 1,238 buttons, but the screen does not use
  it yet. That is the input seat's next item.
- **A whole game played by a person has never been run through our
  rules.** One is on disk: the 2017 recording, a person against four
  Toughest computers, 21,044 frames, 2,104 orders. The original plays it
  to the end on this machine. You named walking it as the next checkpoint
  on 2026-10-15; nothing has since. It is booked as item 4336. Its
  playback captures from October are gone from disk, and I do not know
  who removed them; the retake is 26 minutes.
- **The tranche:** nineteen landings in two sittings for five points of
  the week (39 → 44 %, the day's line 56) and 118.16 USD at list (lanes
  106.20, the driver's two rows 11.96). Twelve gates, none red. Fable's
  week reads 18 %.
- **Yours, carried:** ten minutes playing the client; the merged branches
  on origin; the README's line 12.

## What was measured

| what | measured |
|---|---|
| the merges since the pass before (`git log --first-parent --merges db2921ad2..690aa7ddf`) | 19, all lanes': 4309, 4317, 4315, 4316, 4321, 4319 (tree `att-4309b`), 4320 (tree `att-4317b`), 4322, 4323, 4310, 4324, 4326, 4327, 4329, 4328, 4325, 4318, 4330, 4332 |
| what they moved | 13 moved a score, 3 were first acts that minted a row (4309, 4317, 4315), 3 moved none (4327, 4329, 4330, all on 4310's residue) |
| by seat, lanes at list | the suite's (4317, 4320, 4324, 4328, 4332) 26.37 USD, `LEADERS_AGREE` 2,941 → 23,965; the chain's (4316, 4322, 4325) 23.73, `RUN1646_WORD` 2082 → 5496; the third (4315, 4321, 4323, 4326, 4318) 24.96, the panel 0 → 103 of 103 and `INPUT_WHOLE` 42 → 50; render (4309, 4319, 4310, 4327, 4329, 4330) 31.14, `ROUGH` 0 → 2,287 of 2,387, `SANDY` 649 → 663, the surf 688 → 708 |
| the lanes (`tools/tranche.py --base db2921ad2`, after `lore index`) | 19 sessions, 106.20 USD, 5.59 a session; a session's clock 45.9 minutes, 15.1 working and 30.8 waiting, 20.1 of them on its own gate; one past 300 k (4325 at 348 k); 1 capture; the tool counts 17 landings (entry 139 §4) |
| the driver (`lore loop <row> --since` the last pass's tip, both rows) | 11.96 USD: the evening's row 6.70 over eleven lanes, the morning's 5.26 over eight; no in-session builder in the tranche; lanes and driver 118.16 USD |
| the gates (`tools/gates.py ~/Library/Caches/attrition/gates --since gate-pass-1116g.log`) | 12, 232.2 minutes: 1 whole, 11 scoped (11 widened), 0 red; admission 7,365.9 s summed, longest 312.0 s; 3 ended past the cores |
| the drain's gate (`gate-1117i.log`) | `gate exit 0` on `690aa7ddf`, 9 of 9 steps, 28.6 minutes, release 23.3; `Clock: green beside load`; HEAD and `origin/main` both `690aa7ddf` |
| the week (`lore limits`, 08:57) | 44 % (the day's line 56, the fourth day begun); 27.40 USD a point all-in; Fable's week 18 % |
| the wake (`lore session bfc07b4d`) | the morning's row opened at 06:07:07 −05:00 on the queue's opener with a reap of `63bf762e` as its first act; its first lanes spawned 06:11 |
| the stills (`RON_STILL_DIR`, `the_ground_on_the_client_s_still`, `the_paved_base_under_a_city`, `the_surf_drawn_on_the_still`; `client --lobby --frame N --still`) | run1564, run1559b and run1570 at their photographs' cameras; the minted lobby at 600 and 6,000, and at 600 with object 0 selected; under `~/ron-data/lab-experiments/2026-11-17-pass/` |
| run900 on disk (`find ~/ron-data ~/ron-golden -maxdepth 4 -iname '*run900*'`) | nothing; `~/ron-data/lab-captures/` does not exist; the recording itself is at `game/external-recgames/` |
| the suite re-timed (`suite_timings.py --stale-only`) | 1 stale test timed (`run1646_walks_to_its_word_at_difficulty_4`, 4.99 s): 2,488 tests, 5,025 s one at a time |
| the disk (`df -g ~`) | 206 GB at the start, over the last pass's 188, so no `cargo clean`; 207 after `afsctool` over both archives |
| the rubric (`rubric.py --record`) | every figure the pass before's: 736 · 331 · 572, 44 of 112 groups whole, 1,456 of 3,611 backed, ages 8 of 8, unit types 296 of 302, lockstep bytes 78.1 % |
| the rows and trees left | no lane row (`ccc list`); `reap_agents.py --yes` printed nothing |

## Rulings

- **The render seat leaves the nine deep tiles** (entry 139 §1). Parked
  4330b is not booked; the seat takes the whitecaps and the fish, and its
  scratch probe becomes a tool.
- **The HUD draws what `offered` returns** (§2), and the Citizen's page
  is diffed as the buildings' were.
- **The 2017 sample is booked to be walked** (§3), for the first of the
  chain's and the suite's seats to free.
- **A continuation is numbered before it is spawned** (§4).
- **The wake is exercised** (§5).
- **Marked rows:** no `FABLE:` marker and no second reading in the
  tranche; nothing to ratify.

## What the pass looked at

Each photograph over the client's still at the tip, at the photograph's
camera (`pair-<run>-half.png`), three crops of run1564 at 1:1, theirs left
and ours right, and the minted lobby.

- **run1559b and run1570.** The land, the mountain, the trees, the city,
  its fire and the figures read as the photograph's. The ground past the
  map's edge is dark terrain on both sides now. The photograph has the
  selection's red line round the plaza, a pale beam over the flag-bearer,
  dust on the tanks and small flocks of birds; ours has none of them, and
  our tanks show more team blue.
- **run1564.** The lake's body and its south shore agree. The whitecaps,
  the fish and the west rim's breakers do not (entry 139 §1). The dark
  blot is drawn and sits where theirs does at half scale.
- **The minted lobby.** At 6,000 the capital, its Citizens, the trees and
  the mountain are drawn and lit, with dark terrain past the edge. No
  border is on screen. At 600 with a Citizen selected, the HUD is a line
  of text for the resources, the selection's ring, the unit's buttons and
  a build page of every building in the rules, wonders included.

## The queue's items

| item | kind, mark | the score | first act | it stops and says so when |
|---|---|---|---|---|
| 4331 the demand's walk | rules, whole; the chain's | `LEADERS_HELD_OUT` 1 → 0; `RUN1646_WORD` 5496 → 6,081 | as booked | as booked |
| 4333 parity's last flag bits | rules, whole; the suite's | `LEADERS_AGREE` 23,965 → 24,000 | as booked | as booked |
| 4334 the HUD draws the offered panel | input, scoped; the third seat | a row minted first: the HUD's drawn buttons equal to the original's panel, of 1,238 on the sixteen packets; then the Citizen's page | the HUD's buttons beside `offered`'s on one packet | the Citizen's page has no oracle in `panels.py` (then it says what the oracle needs) |
| 4335 the lake as the eye sees it | render, scoped | a row minted first: the foam share of each lake tile, theirs against ours; the fish's pixels dark or white | the card's draws over the lake, their setters beside ours, kept as a tool | the foam is a draw the card trace on disk does not hold |
| 4336 a person's whole game, walked | rules, whole; the first of the chain's and the suite's seats to free | the start block's fields equal on the playback's first frame; then the walk's word of 21,044 | the archive grepped, the opening retaken, the start built from the recording's header | the header's lobby is one the start builder does not hold, or the retake is refused |

## Built

- Nothing in the sim or the tools. The paperwork: entry 139, this record,
  the queue's three items and handoff, the tracks' paragraph, the journal's
  entry, the suite's timings.

## Not done, and said

- **`tools/tranche.py` was not mended** for a landing whose tree carries
  another number (entry 139 §4): the rule is the driver's, and the tool
  cannot tell two merges under one booking apart.
- **The whitecaps' draw was not read.** The pass saw them and booked them;
  what lays them and at what strength is the item's first act.
- **Nothing about the sample was run.** Its map is style 7 by run900's
  ledger, Great Sahara's; its seats and whether the start builder holds
  its lobby are the item's to find. The pass read the ledger and searched
  the disk.
- **The blind list was not scanned again**: open 9, the input track's
  issuers.
- **The lobby's missing border at 6,000 was not chased**: the still's
  camera may simply not reach it.
- **Carried:** a red clock beside load is unrun; whether a builder's
  handback survives its driver's clear; `tools/tranche.py` tables no
  in-session builder; a lane's wait on `ScheduleWakeup` (4298b) is not
  ruled on.

## The gate

Whole, by `tools/gate_run.sh`, on the pass's commit: `gate-pass-1117.log`.
Its lines are quoted here by the commit after it, which is gated scoped
(`gate-pass-1117b.log`).
