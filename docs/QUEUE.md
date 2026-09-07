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

*2026-09-07, the eighth steer (Fable, with the user) — no word moved; the
ratchet is reverted, 266 is resolved, and the loop resumes at width two.*

- **The endpoint line is telemetry again** (DECISIONS 36). 258's
  may-only-fall assertion was falsified by the first landing after it and
  re-pinned upward three times in two days; the 09-01 rule stands —
  assert up to the word, print past it. `endpoint.rs` now pins the counts
  **exactly** and the handoff line must equal them: a move in either
  direction is a one-line re-pin naming the item, and no trade is owed.
- **266 was half stale, and the half that stands is entry 29's own rule.**
  run87 covers Great Lakes 7244–7520, so the word's cause (267's second
  row, the 7418 tick) is a diff. East Indies' coverage ends at run85's
  7480 and the word is 7529, so the map is owed a capture: **`[7474,
  7800)` at run85's detail**, six blocks over run85's tail, about ten
  minutes and 200 MB. Batching ahead of the word waits on 260.
- **The stopping rule fired correctly** at two after 264's reset and the
  commander asked; it stands unchanged. What it caught was not drift but
  a queue with nothing headline-nearest on disk — which the lane fixes.
- **The commander spawns; it does not work items.** 267 was done in the
  main thread and spent the commander's context (a clear owed). The
  marked-row batch is empty; everything is merged, pushed and main level.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7529 of 24,000 · GreatLakes w7455 of 24,000
Endpoint 24001: EastIndies 79 off, 0 unlinked · GreatLakes 81 off, 0 unlinked

**Opener: commander on Opus (DECISIONS 34), spawning — never working —
two at once: a worker for 267's second row (Great Lakes' 7418 tick, ARMY
§5, run87 on disk, a diff) and the East Indies capture lane above, booked
here and needing no further ask. Then the backlog at width two.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable.

260. **The parser goes lazy and the gate sub-GB** — `Log::parse` builds
    all 76.6 M fields of an 800 MB dump (235) where a test reads hundreds.
    Frame offsets in one pass, blocks on demand over a memory-mapped file,
    whole-file scans (252) stream; the 230 tests are the oracle. (266)
    Batching captures ahead of the word waits on this: a capture is
    sized to what the parser can hold, so a batch costs what it saves.

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
    (256) `come_out`'s three `action_move_to` sites stay unreached.

247. **An upgrade is an in-place guy-type change on the standing unit** —
    run76's **6737**, three Archers going guy **170 → 177** keeping
    `(who, o)` and `group 64`; East Indies' `1/32` does 340 → 341, the
    danger row moving by `(110 − 100) / 2`. (181) CARAVAN §7.2–§7.3.

234. **Four rules of the turn/idle animation neither side has** (ANIM §9),
    written and never landed — branch `rescue-234`. **The two static
    assertions are the cheap half**; the limbs move no score (§4.7's
    turning types are in neither scored game). Takes the ledger's nine
    `unverified` rows on the way.

**The widening ledger** (87, DATALAYER §4, §4.1): **19** fields the
harness names nowhere, **44** one capture names; blind spot `avg_speed`
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

248. **ATTRITION's Territory section is 15,625 bytes of 16,000** and wants
    a retelling pass before anything is added. Its remaining terms are
    closed — handicap, temple and fort are unreachable in any game that
    runs here; read it before re-booking any. (AI §2.1's `check_explore`.)

161. **The make-list block is 2,500 frames behind East Indies' word**:
    `create_buildings` first runs on 9982 (AI §25), so `building_value`,
    `gather_value` and §24.4's arm wait on it. (195) `find_repair_spot`.

253. **`1/13` parts at 6938**, a citizen at `myspeed 25` — run86's only
    other divergence, and run82's 6946 dump shows it off by (11,7). A
    diff, not a capture. (254) `come_out`'s ring has one sample (7284)
    and wants a second disembark. (167) run61's two (SYNC §3.9).

211. **`get_speed`'s three remaining arms** (MOVEMENT): `unit_masks &
    0x10`, set and cleared inside a frame; `has_general(0, 0x162)`'s siege
    doubling; the group cap, gated on `action_type == 0` (219) — it wants
    a grouped unit holding no action. (229) a figure in melee does not
    step its clock — `unit_masks2 & 0x10` freezes `Guy::inc_time` (ANIM
    §5), and that arm waits on a melee frame.

251. **`memcap.sh` has no fixture and three doors**: its `ps | awk` reads
    `0 0` when `ps` answers nothing, so a refused sample reads as no memory
    used and the ceiling never fires (lore); it was **mode 644**, so
    `tools/memcap.sh …` exited 126 having run no test, until 09-07; and it
    takes a **GiB cap as its first argument**, which the line below telling
    you to use it never said. Wants a fixture: past the cap, dead in N
    seconds, exit 137 — and a gate that cannot be run wrong.

267. **The six-slot assignment at Great Lakes' 7418 tick** — what the
    word is made of, now that its run-up is clean (264, run87, ARMY §5).
    On sim-frame 7418 — `7162 + 256`, `frame ≡ 250 (mod 256)`, the tick
    261 fixed at 7162 — all six AI soldiers take a fresh `orders_x/y` and
    **four come out on a different path waypoint**, `1/32` and `1/33` on
    exactly each other's (`dest_x` 39838 against 39863, swapped). `1/36`
    is 94 units adrift by 7455 and that is the stand. A diff, not a
    capture. (268) `AnimalData::ox`/`whom`/`aid` are `GUYS ≥ 3`, unparsed.

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
