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

*2026-09-17, Fable steering (Fable 5.1) — the lab merged whole, the
gate is one command, no score moved.*

- **Great Lakes 7585** leads and is unmoved; **East Indies 7812**. 295
  is headline-nearest, 294 beside it, one per map.
- **The lab landed whole** — `codex/methodology-exploration`, 80
  commits, DECISIONS 38; ledger `docs/lab/LEDGER.md`. Defaults now:
  readers that own one frame, a soak digest carrying the RNG seed and
  an activity signal apart from the clock, a trace reader refusing torn
  frames, `memcap.sh` refusing an unsampled run, `focus.sh` fail-closed.
- **The gate**: `python3 tools/release_gate.py <install> --test-threads
  4` — survey, offline tests, release suite under the cap with a fixture
  audit, clippy, fmt, guards. This merge's run: 132 rondata s at
  12,646 MiB, against 288 s / 10,869 at two threads before. DECISIONS
  34 amended; bare `cargo test` stays at two.
- **Opt-in until a landing uses it**: the diff viewer and checkpoint
  replay (`docs/DEBUG_VIEWER.md`), piloted on 295's first window; the
  click-free capture lane, 6 of 10 pairs, parked as 298 with 299 (a
  Gaia reseat heading witness) and 300 (the unused command API).
- **Loop**: landed single-threaded so the gate held still; it resumes
  as a two-lane wave, and the next steer reads USD per word-moving
  landing against 102.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7812 of 24,000 · GreatLakes w7585 of 24,000
Endpoint 24001: EastIndies 76 off, 5 unlinked · GreatLakes 70 off, 9 unlinked

**Opener: 295 (Great Lakes' word, `create_units`' value chain, three
oracles) and 294 (the 7809 collision miss) as a two-lane commander wave
on Opus; gate `tools/release_gate.py`; briefs carry `CLAUDE.md`'s worker paragraph.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable.

294. **Why does this crate not collide on 7809 at all** — 289 left East
    Indies' word as one block. `1/7`'s entire collision-field divergence
    over run90's 111 blocks is 7809, where the original has `collide 1 /
    collide_o 6 / collide_who 1` and this crate has it clear. So the first
    question is not which arm of §6 step 5 fires but why the detection
    misses, and no capture is needed: every field is on disk. Takes §6's
    wait-versus-pause predicate, which is now the whole of the word.

295. **`create_units` values three Slingers at 9999999** — the `val < 0`
    overflow guard firing where the original prints 360000, num 1, which
    puts a Temple at the make-list head on Great Lakes 7585 and ours three
    Slingers. A Temple costs no food, and `make_stuff:172` only tries the
    goods the head costs, so food never goes on trial and the Citizen is
    bought. Headline-nearest. Three oracles for the value chain: 360000,
    `num 1`, and slot 7's exact factor of two. AI §33, §34.

278. **`Unit::work@0060d180:440` is a second `set_new_location(…, 1, 1)`**
    and nothing models it — two units of one type within `0x180` are
    pushed apart by half their separation and **both snapped**, gated on
    `field_0x82 < 0` and the order's `+0x30` vcall (read, never run; the
    gate's meaning is the unread half). Ruled out for 271's 6937 — nothing
    within 700 units of `1/13` — but `Guy::last_pos` now makes the
    signature searchable on every dump: a unit that moved whose figure has
    `last_x == x`. run79's squad first. Falsified by no pair closing.

279. **`queueledger.py` has two regex defects and both cried wolf** — its
    group regex pulls every `\d{1,3}` from a paren, so item 253's "off by
    (11,7)" booked 7 and 11 and deleting 253 reported 7 as gone in silence;
    and it matches `item N` **case-sensitively**, so a journal heading
    reading `Item 289` failed that deletion at the 289/290 merge. No item
    has ever written two numbers in one group. Make each fail on purpose
    first, both ways — a real deletion must still be caught. The guard's
    loudness is correct; the false alarms are the regexes'.

220. **TECH §13's twenty range blocks.** (209) **the
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

251. **`memcap.sh` has two doors left and half a fixture**: the refused
    sample that read as zero and let the ceiling sleep is closed — it exits
    125 and refuses to launch (lab L25, `tools/explore/test_memcap.py`) —
    and the mode-644 door closed 09-07. Left: it takes a **GiB cap as its
    first argument**; and (281) it read `ps rss`, which **over-counted the
    mapping 280 removed** — ~1.3 GB of 260's 8,816 was run58's clean text
    — and a 2 s poll under-reports a sawtooth. Still wants the fixture
    with teeth: past the cap, dead in N seconds, exit 137.

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
- **Run the diff suite with `--release`** under `tools/memcap.sh 20` — or
  `tools/release_gate.py <install> --test-threads 4`, which wraps it — to a
  file, never piped (a pipe launders the 137), keeping the `peak` line. **Before a blind fan-out**, grep
  `CLAUDE.md` and the memory index: a subagent inherits both, and neither
  they nor a brief may quote this file or the journal.
