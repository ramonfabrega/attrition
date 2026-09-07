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

*2026-09-07, the commander's first unattended pair (Opus) — the headline
moved 129 frames, and the two maps swapped places.*

- **Great Lakes' word is 7584** (was 7455). `Group::compute_form`'s tail has
  a **second** reverse test, distinct from the `facing` toggle above it: given
  the formation angle — every army call — it compares that against the bearing
  to the destination and negates every member's `off_x`/`off_y` on both the
  `Form` and the `GroupData`, so a flipped formation **stands on the wrong
  side of its leader**. GROUPS §6.3 had it written down and never built.
- **East Indies is the lower map now, and its word is one field.** run88
  `[7474, 7800)` landed — 327 blocks, 187 MB, three checks that could
  each fail, and the corpus's first total overlap needing no flag. 271 is
  the whole of 7529 and needs no screen: 425 rows are already on disk.
- **The negation costs East Indies three endpoint counters, measured not
  reasoned** — 267b disabled it in place and got the old pin back exactly.
  DECISIONS 36 governs (telemetry past the word, no trade owed), but the
  cost is named here rather than absorbed silently.
- **260's falsifier is met, so the lane is held.** The merged tree gated at
  **16,732 MiB of 20 GiB**, third over 16 in two days and the highest yet.
  272 waits on 260 — and Great Lakes stopped being the headline when the
  maps swapped, so that capture lost its urgency the day it was written.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7806 of 24,000 · GreatLakes w7584 of 24,000
Endpoint 24001: EastIndies 79 off, 0 unlinked · GreatLakes 78 off, 2 unlinked

**Opener: commander on Opus (DECISIONS 34), spawning — never working. In
flight: 271 (East Indies' gather countdown, the headline) and 260 (the lazy
parser, spawned ahead of 272 on the 16,732 MiB gate). They share the parser,
so merge 260 first and expect to re-run 271. Then 272 to the lane.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable.

271. **The gather countdown is a tick long** — East Indies' word whole,
    and the map is now the lower one. `1/13`'s `GATHERORDER wait` reads
    `theirs + 1` on all 55 blocks of the cycle ending at 7529, so the
    original fires (`wait −1`) where this crate still holds 1 and spends
    `Guy::set_anim+0x97a < Unit::do_non_flat_gather+0xb99` a frame late;
    the step is 24 units on 7531. Unbroken from run86's 6985 through
    7409 — **425 rows on disk** — and into run88's 7474. A diff, no
    screen. Falsified by a run86 block under 6985 where the waits agree.

260. **The parser goes lazy and the gate sub-GB** — `Log::parse` builds
    all 76.6 M fields of an 800 MB dump (235) where a test reads hundreds.
    Frame offsets in one pass, blocks on demand over a memory-mapped file,
    whole-file scans (252) stream; the 230 tests are the oracle. (266)
    Batching captures ahead of the word waits on this. **Now measured**:
    the gate peaked 15,479 then 16,169 MiB of the 20 GiB cap with run87's
    121 MB and run88's 187 MB in, a 700 MiB swing at two threads — a third
    run over 16 GiB makes the next window of this size unbookable.

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

248. **ATTRITION's Territory section is 15,625 bytes of 16,000** and wants
    a retelling pass before anything is added. Its remaining terms are
    closed — handicap, temple and fort are unreachable in any game that
    runs here; read it before re-booking any. (AI §2.1's `check_explore`.)

161. **The make-list block is 2,500 frames behind East Indies' word**:
    `create_buildings` first runs on 9982 (AI §25), so `building_value`,
    `gather_value` and §24.4's arm wait on it. (195) `find_repair_spot`.

253. **`1/13` parts at 6938**, a citizen at `myspeed 25` — run86's only
    other divergence, and run82's 6946 dump shows it off by (11,7). A
    diff, not a capture. **Not 271's countdown**: the position parts at
    6938 and the first `wait` row is 6985, 47 blocks apart, so folding
    the two would lose one. (254) `come_out`'s ring has one sample (7284)
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
