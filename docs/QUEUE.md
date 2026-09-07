# The queue

Where the work stands and what comes next — the file a fresh session reads
first, and the one a subagent never sees. `CLAUDE.md` is the rules; this is
the state; `docs/JOURNAL.md` is the story; `docs/PARKED.md` is the backlog,
read when a wave is composed and never at boot.

This file **deletes**, numbers are never reused, and `queueledger.py` fails
the build if one leaves in silence. `docs_guard.rs` bounds the file by
**items, not lines** (262): 18 open at 8 lines each, the handoff at 32,
each section's non-item prose pinned and falling only.

## Where things stand

*2026-09-07, the commander's loop unattended (Opus) — both words moved,
130 frames and 277, and the maps swapped twice.*

- **East Indies 7806** (was 7529), **Great Lakes 7585** (was 7455) and
  leading, so 290 — its word's cause — is headline-nearest.
- **Three words fell to one method**: widen the whole record, never the
  field the brief names — an age snaps every figure (271),
  `compute_form`'s tail has a second reverse test (267), and a bird's
  figure lags its unit (284, `do_air_physics` passing **0 as `param_3`**).
  Each time the brief's own named mechanism was wrong.
- **run90 was written to be refused, and was** — 286 put this crate's
  collide/pause/wait rows down as predictions *before* the capture; the
  original refused two of four. The cause is a **waypoint the original
  abandons and this crate walks to** (289), and `1/7`'s `pause 8` is the
  disk's first non-zero `MOVEORDER` pause, settling COLLISION §9.
- **Great Lakes' word is one number** — `88 < 55 + 43 + 4` in `make_stuff`
  step 6, so it is the AI's **stockpile, not its rules** (290), and a
  leader's `bucket` has never been compared on this map. 291 is a live
  candidate: the AI's caravans are unlinked on 246 of 246 blocks.
- **260 halved the gate**; **280 took the mapping back out**, one `forbid`
  again. `FABLE:` owed — 37, 280's UB ground, 283, CLAUDE.md's wait rule.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7806 of 24,000 · GreatLakes w7585 of 24,000
Endpoint 24001: EastIndies 79 off, 0 unlinked · GreatLakes 84 off, 0 unlinked

**Opener: commander on Opus (DECISIONS 34), spawning — never working.
Nothing in flight, board clean. Next: 290 (Great Lakes' word, wanting a
`LEADERS=9` window — the lane's) with 291 or 289 beside it; then 285, 283.
Every brief ends with a "done" ping, and `landed` beats it (entry 34).**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable.

289. **The original abandons a sidestep waypoint this crate walks to** —
    run90 refused 286's candidate and moved the cause two blocks below the
    word. On 7805 `1/6` is (39720, 38808) against the original's (39729,
    38802); **both sides push the same waypoint**, and the original drops
    it after one step where this crate walks all the way to it — a
    four-block cycle against five. §8.5's turn-versus-step reading is
    **dead**, not merely unconfirmed. run90 is on disk, so this is a diff.
    East Indies' word holds at 7806 and this is what sits under it.

278. **`Unit::work@0060d180:440` is a second `set_new_location(…, 1, 1)`**
    and nothing models it — two units of one type within `0x180` are
    pushed apart by half their separation and **both snapped**, gated on
    `field_0x82 < 0` and the order's `+0x30` vcall (read, never run; the
    gate's meaning is the unread half). Ruled out for 271's 6937 — nothing
    within 700 units of `1/13` — but `Guy::last_pos` now makes the
    signature searchable on every dump: a unit that moved whose figure has
    `last_x == x`. run79's squad first. Falsified by no pair closing.

283. **The ratchet wants a `#[global_allocator]`, not a mapping** — 280's
    +1,563 MiB is *not* live data: a mapping is `munmap`ed at drop, while a
    freed `String` of a capture's size is kept by macOS's allocator and
    cannot serve the next capture's different size (260's own 5,332-MiB-
    with-nothing-alive probe, again). An allocator that returns large
    blocks recovers most of it with **no `unsafe` in this tree** — the
    crate carries it — and closes 235's ratchet for every large owned
    buffer, not the two that were mapped. Take it after the score moves.

279. **`queueledger.py` reads a coordinate as a booking** — its group
    regex `\(([\d/,\s]+)\)` then pulls every `\d{1,3}`, so item 253's
    "off by (11,7)" booked numbers 7 and 11, and deleting 253 reported 7
    as having left in silence (ledger, 2026-09-07). No queue item has ever
    written two numbers in one paren group, so the fix is likely a single
    number per group — but make it fail on purpose first, both ways: a
    real deletion must still be caught. The guard's loudness is correct.

226. **`cover=1` dies in the wow64 bop** (ORACLE), an x86 host the
    falsifier. (220) TECH §13's twenty range blocks. (209) **the
    once-per-game events a dump install swallows** — a `set_*` whose
    **return value** drives an irreversible record; **takes 224**. (203)
    `mark_behind_tiles`' `0x4` is a building *finishing*; (240) the
    merchant's `gather_down`/`special` −1 at 6929, **takes 184**; (216)
    `create_buildings` offers a gather building the original does not.

(242) **The crate's group id is not the original's** — `GroupData +0x4 =
64` against `group_id`'s stand-in, the army group's *slot* 1; the other
five agree on all 630 and 237 reports the row without scoring it. (243)
`UnitDump::group` is parsed and **nothing compares it**. (244) **1,428
grouped order records no test windows** — run79's 453, run31's 945.
(245) `focus.sh` matched a concurrent worker's shell on run84.

255. **`go_to_unit`'s `0x480` and `go_to`'s `MOVE_TO` arm are read and
    never run** — every joiner on disk is farther than `0x480` from its
    army's `get_unit(0)`, so the falsifier is one born beside its army.
    (256) `come_out`'s three `action_move_to` sites stay unreached; (254)
    its ring has one sample (7284) and wants a second disembark. (167)
    run61's two (SYNC §3.9). All four came off 253, which 271 closed:
    `1/13`'s 6938 parting was the age snap, not a movement fault.

247. **An upgrade is an in-place guy-type change on the standing unit** —
    run76's **6737**, three Archers going guy **170 → 177** keeping
    `(who, o)` and `group 64`; East Indies' `1/32` does 340 → 341, the
    danger row moving by `(110 − 100) / 2`. (181) CARAVAN §7.2–§7.3.

**The widening ledger** (87, DATALAYER §4, §4.1): **19** fields the
harness names nowhere, **45** one capture names; blind spot `avg_speed`
(210) — it counts *fields* and cannot see a record the parser never
visits, which is what hid 252's dumps. Uncounted: (88) the blind list, 101
of 617; (72) every `+0xNN` a document pins vs its module; (89) the guard;
(35) VISION §7.

269. **The third axis: a field compared at the wrong width** (DATALAYER
    §4.2, from 265). Ten `CityData` counters are `uchar` in the type record
    and `i32` in `ai::CityAi` — `busy` and `gatherers` have bare byte
    writers of their own — and `pop` is an eleventh on `sim::City`. None
    has been seen to wrap; the falsifier is a capture where the sweep's
    count and the producers' decrements cross zero. Same question one
    record up, for every `char`/`short` of `LeaderData` held as an `i32`.
    A guard wants the PDB's widths beside the sim's structs, not a grep.

270. **The fourth axis: a word compared one bit at a time** (DATALAYER
    §4.3, from 267). The ledger scans the differ for a field's *name*, so
    `unit_masks` has been on neither list since the packed bit got a row
    while one of at least eight modelled bits was actually compared —
    and the missing one, `0x100000`, named Great Lakes' run-up cause on
    its first run. Nine dumped masks want a per-**bit** census:
    `unit_masks`, `unit_masks2`, `guy_flags`, `node_flags`, `city_flags`,
    `leader_flags`, `leader_flags2`, `build_flags`, `role`. Same tool 269.

257. **Nineteen kept tests compare a torn block** (252): a closing dump is
    frame n but for the one unit the quit caught mid-update, and every
    nested archive's last `FRAME n` body *is* its closing dump. No live
    case found, unaudited; fix is `compare_shutdown`'s n−1 allowance.

175. **The uber chain past its birth**: `Objects::init_unit` threads
    `uber_size` objects (CITIES §4.3); nothing else reads it. Takes (48)
    COLLISION §3/§7, (73) `UnitData::group`'s back-pointer, (56) ARMY §13.

161. **The make-list block is 2,500 frames behind East Indies' word**:
    `create_buildings` first runs on 9982 (AI §25), so `building_value`,
    `gather_value` and §24.4's arm wait on it. (195) `find_repair_spot`.

211. **`get_speed`'s three remaining arms** (MOVEMENT): `unit_masks &
    0x10`, set and cleared inside a frame; `has_general(0, 0x162)`'s siege
    doubling; the group cap, gated on `action_type == 0` (219) — it wants
    a grouped unit holding no action. (229) a figure in melee does not
    step its clock — `unit_masks2 & 0x10` freezes `Guy::inc_time` (ANIM
    §5), and that arm waits on a melee frame.

251. **`memcap.sh` has four doors and no fixture**: its `ps | awk` reads
    `0 0` when `ps` answers nothing, so a refused sample reads as no memory
    used and the ceiling never fires (lore); it was mode 644 until 09-07;
    it takes a **GiB cap as its first argument**; and (281) it reads `ps
    rss`, which **over-counts a mapped capture** — macOS refuses to hand a
    private file mapping back, so ~1.3 GB of 260's 8,816 is run58's clean
    text, and a 2 s poll under-reports a sawtooth besides. Wants a fixture:
    past the cap, dead in N seconds, exit 137.

290. **Nothing has ever compared a leader's ledger, and Great Lakes' word
    is in it** — 287 reduced 7585 to one number: every gate of
    `make_stuff` step 6 agrees but the good loop over food,
    `88 < 55 + 43 + 4`. Give the AI **98** food on that frame and 7585
    agrees 9 draws for 9, the bird at index 4 included. So the word is the
    AI's **stockpile, not its rules** — 98 ≤ food < 160 against this
    crate's 88. `bucket` is written only at `LEADERS=9` and no Great Lakes
    capture carries it past setup: the falsifier is one such window.

291. **The AI's caravans are not linked** — `vans.length` 0 against 1 and
    `trade_val` 0 against 128 on **both** of player 1's cities, 246 of 246
    blocks (287). That names two of 285's eighteen fields and gives them a
    mechanism, and wealth is what a market buy spends, so it is a live
    candidate for 290's shortfall rather than a separate residue. (288)
    the gull's `do_strafe`, unmodelled. (274) run87's `1/26`. (268)
    `AnimalData::ox`/`whom`/`aid` carry nothing — −1 on all forty animals
    on all 247 blocks. Closed as answered, not open.

285. **The CITY record parts on every block and nothing asserts it** — 18
    fields on 246 of 246 of run89's window, found by widening the whole
    record, and **no window test on either map asserts `city_diverged`**.
    Nine tenths of a dumped record once went uncompared for a month and
    the first widening failed on its first run; this is that shape one
    record up. Two of the eighteen are named now (291, the unlinked
    caravans). `1/3`'s `(+192, +192)` `MOVEORDER` was booked here and
    **closed itself** when 284 landed the air-physics fix.

273. **`refresh_group_order` re-origins on the order's `form_id`, not the
    member's list position** — `713ac3` reads `[eax+0x10]` off the
    `GroupOrder` the `+0x94` vcall returns, where `Sim::group_refresh_order`
    uses `g.list.iter().position(member)`. `do_group_move` step 2 rewrites
    `form_id` every frame, so the two agree except where membership changed
    and the follower arm has not run: wants a `GROUPS=1` window across a
    death or a join in a marching formation. (275) `MoveOrder::facing`
    still does not score; 267 split the two mechanisms, so re-read that.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items to the journal. **A new
  item costs nothing** — shorten only what you are touching, never another
  author's item, and never widen a line to beat a count.
- **A floor that moves** moves `FLOORS`, the assert reading it, and the
  `Scoreboard:` line together. **Start of session:** merge the lanes, then
  "Where things stand", the item, its document.
- **A worker never books a number; it reports and the commander books.**
  Two live workers both take "the next number" and collide — 264 and the
  commander both booked a 265 on 09-07 — and two findings under one number
  breaks the index numbers exist for. Scoreboard line and journal entry
  stay the worker's own.
- **Overlap a capture's neighbours on purpose.** The run-up is free, so an
  overlap costs seconds and turns "same seed, therefore same game" into a
  state check. Six blocks each end is the floor (run83); a neighbour that
  **contains** yours is free and total (run84) with `--exclude <the
  category you raised>`. **The trap**: `samegame.py` exits 0 when nothing
  is in common, so assert the count, not the verdict.
- **Grep for the derived quantity, the mechanic's SHAPE and the GATE — not
  only for coverage of a frame.** `BUILDQUEUE` prints `queue[scan].cost`,
  so a price the original *paid* is on disk wherever `BUILDS` is on (238),
  and 178's two passes write different footprints. A term zero in every
  dump is either zero-valued or switched off: 117 and 178 were the second.
- **A check on a field never cleared must assert a CHANGE, not a value** —
  `collide_frame` is a permanent stamp, so "some unit has one in the band"
  is true of any window, and run85's first teeth check passed on a band
  where nothing happened. Test both directions on real data first.
- **A commander's clear is free when**: every landed branch merged, gated
  and pushed; nothing in flight owing its result elsewhere; this handoff
  current with the headline *measured*; nothing unfiled. `ccc clear <ref>
  --then "continue"` arms it, and takes a session name from 0.1.31.
- **Run the diff suite with `--release`** under `tools/memcap.sh 20` — the
  cap is its first argument — to a file, never piped (a pipe launders the
  137), keeping the `peak` line. **Before a blind fan-out**, grep
  `CLAUDE.md` and the memory index: a subagent inherits both, and neither
  they nor a brief may quote this file or the journal.
