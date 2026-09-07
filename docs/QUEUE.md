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

*2026-09-07, the commander's loop unattended (Opus) — both words moved in
one day, 129 frames and 277, and the maps swapped twice.*

- **East Indies is 7806** (was 7529) and **Great Lakes 7584** (was 7455),
  so Great Lakes is the lower map again and 272 is headline-nearest.
- **An age snaps every figure** (TECH): `Leader::gain_tech`, gated on
  `is_age_type`, snaps every live unit's facing and body — a free turn
  **and a frame** for one mid-turn. 271 was the countdown's seed, not its
  sum; **253 closed with it**. Great Lakes' twin is `compute_form`'s second
  reverse test (GROUPS §6.3, written down and never built). **Both were
  found by widening the whole record**, not the field the brief named.
- **260 halved the gate**, 16,061 → **8,321 MiB** at 243 tests; its ceiling
  was never a ratchet (251: `memcap.sh` under-reports a sawtooth and
  over-counts a mapping). **The mapping came back out** (280, the user's
  steer) — `runqueue.sh` renames over an archive while tests run, so the
  UB was reachable here, and the lazy half is safe Rust and most of the win.
- **A Fable pass is owed, with an agenda**: DECISIONS 37 (drafted,
  unratified), the perf/UB ground 280 opened, and 283. Nothing else marked.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7806 of 24,000 · GreatLakes w7584 of 24,000
Endpoint 24001: EastIndies 79 off, 0 unlinked · GreatLakes 78 off, 2 unlinked

**Opener: commander on Opus (DECISIONS 34), spawning — never working. In
flight: 276 (East Indies' 7806 — grep run88's 7816 block before booking a
screen) and the lane on 272 (Great Lakes' word window, the headline). 280
is landing. Then 283, the allocator. Order set with the user after the
stopping rule fired at two: score first, infrastructure behind it.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable.

276. **East Indies' word has a shutdown dump above it already** — 7806 is
    past run88's `[7474, 7800)` and this crate spends `Guy::set_anim+0x97a
    < Guy::inc_time+0x271` at index 2 of 8 where the original spends
    `< Unit::move_step+0x823`. **Grep before booking a screen**: run88's
    `!quit` left a block at **7816**, ten frames above the word, carrying
    `1/6` and `1/7` parting and never read as its residue. Falsified by
    that block explaining the two; if it does not, the window is
    `[7740, 7900)` at run88's detail, ~160 blocks, ~90 MB.

277. **An age gained through a cascade is unmodelled** — `Sim::gain_tech`
    reads the age gate off the type the call was made with, as the
    original does, but the original **recurses into `gain_tech`** per
    cascaded grant where this crate flattens the cascade into events, so a
    cascaded age would take 271's snap arm there and not here. No capture
    on this disk gains an age other than directly. Now a grep rather than
    a reading: `BuildDump::max_age` is parsed, so the falsifier is a block
    whose `max_age` moves on research that is not one of the seven ages.

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

280. **The mmap comes back out** — decided with the user 2026-09-07, in
    flight. 260's two halves are separable: lazy frame indexing is safe
    Rust and is most of the win (run58's arena 2,221 → 17 MiB), while the
    mapping needs `Mmap::map` and cost `crates/rondata` the workspace's
    `forbid(unsafe_code)`. `Mmap::map` is unsafe because a writer can
    truncate under a live `&[u8]`, and this repo has one: `runqueue.sh`
    renames over an archive name while tests run. Acceptance is a grep —
    no `allow(unsafe_code)` in `crates`. (266) closed by 260.

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

272. **Great Lakes' word has no dump under it** — 7584 is outside run87's
    `[7244, 7520)`, so the headline map's next value diff is a capture:
    **`[7530, 7760)` at run87's detail**, `cover=0`, six blocks under the
    tail, ~230 blocks, ~100 MB. At 7584 the polarity reverses (48 draws
    against 49, the original spending `Guy::set_anim+0x97a <
    Guy::move+0x19f` at index 24). (268) `AnimalData::ox`/`whom`/`aid` are
    `GUYS ≥ 3`, unparsed: `samegame.py --drop ox --drop whom --drop aid`,
    and assert the count. (274) run87's `1/26`, the window's last residue.

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
