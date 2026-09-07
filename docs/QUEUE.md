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

*2026-09-07, Opus commanding (DECISIONS 34) — 258 landed, 261 is still
out, and the reap found three items the crash had eaten.*

- **Great Lakes 7176** (leads; item **261** in flight): 97 draws differing
  at 93 — the original in `Unit::do_idle+0x7d`, this crate in
  `Guy::inc_time+0x271`; a unit goes idle there and not here, and run79's
  `[6910,7250]` covers it. **East Indies 7529**: a `Guy::set_anim` under
  `Unit::do_non_flat_gather+0xb99` the original spends and this does not.
- **258 landed**: the endpoint at 24001 is pinned on both maps and may only
  fall (`diff/endpoint.rs`, 260's split first cut). The disk holds **four
  games, not two**, and `borrow_from_siblings` had read East Indies against
  Great Lakes' herds; `check_setup` asserts each map's fingerprint now.
- **Three items the crash ate are back.** `loop-234`'s tree held 406
  uncommitted lines no branch carried; `8b37e5f` deleted 230–234 and only
  two of the five had landed. 232/233/234 re-booked, the work on
  `rescue-234`, audited in `docs/audit/queue-ledger.md`. **A branch check
  cannot see a dirty tree** — ccc holds that half as their item 32.
- **The queue is bounded by items** (262, 263): 18 at 8 lines, sections
  pinned, backlog parked. **A new item costs no compression** — shorten
  what you are touching, never another author's item.
- **The gate**: 238 tests, 208 s, 16,027 MiB at the pinned two threads.
  **No pipe between `memcap.sh` and the exit code.**

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7529 of 24,000 · GreatLakes w7176 of 24,000
Endpoint 24001: EastIndies 80 off, 0 unlinked · GreatLakes 73 off, 7 unlinked

**Opener: commander on Opus (DECISIONS 34). Merge 261 and 258, gate, file;
then the headline is whatever Great Lakes' word reads after 261. 232, 233
and 234 are re-booked and cheap — 234's two static assertions first.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable.

260. **The parser goes lazy and the gate sub-GB** — `Log::parse` builds
    all 76.6 M fields of an 800 MB dump (235) where a test reads hundreds.
    Frame offsets in one pass, blocks on demand over a memory-mapped file,
    whole-file scans (252) stream; the 230 tests are the oracle.

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

233. **A building's periodic phase is its object number**, not its slot:
    `Build::process@0061edf0:728` keys the 32-frame phase on `o`;
    `process_building` uses `frame + b`, right until a slot is reused
    (CITIES §5). (232) `Levels::for_player` ignores its player, answering
    a constant whose `taxation` is 0 (ECONOMY, audit R8).

**The widening ledger** (87, DATALAYER §4, §4.1): **19** fields the
harness names nowhere, **44** one capture names; blind spot `avg_speed`
(210) — it counts *fields* and cannot see a record the parser never
visits, which is what hid 252's dumps. Uncounted: (88) the blind list, 101
of 617; (72) every `+0xNN` a document pins vs its module; (89) the guard;
(35) VISION §7.

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

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items to the journal. **A new
  item costs nothing** — shorten only what you are touching, never another
  author's item, and never widen a line to beat a count.
- **A floor that moves** moves `FLOORS`, the assert reading it, and the
  `Scoreboard:` line together. **Start of session:** merge the lanes, then
  "Where things stand", the item, its document. Lanes never write this
  file, and a brief names its `docs/audit/…-vs-code.md` where one exists.
- **Overlap a capture's neighbours on purpose.** The run-up is free, so an
  overlap costs seconds and turns "same seed, therefore same game" into a
  state check. Six blocks each end is the floor (run83); a neighbour that
  **contains** yours is free and total (run84) with `--exclude <the
  category you raised>`. **The trap**: `samegame.py` exits 0 when nothing
  is in common, so assert the count, not the verdict.
- **Grep for the derived quantity, the mechanic's SHAPE and the GATE — not
  only for coverage of a frame.** `BUILDQUEUE` prints
  `queue[scan].cost[0..2]`, so a price the original *paid* is on disk
  wherever `BUILDS` is on (238); 178's two passes write different
  footprints, so a half-cell with no building in its 3×3 isolates the unit
  pass in archives already held. And a term zero in every dump is either
  zero-valued or switched off: 117 and 178 were booked on the first.
- **A check on a field never cleared must assert a CHANGE, not a value** —
  `collide_frame` is a permanent stamp, so "some unit has one in the band"
  is true of any window, and run85's first teeth check passed on a band
  where nothing happened. Test both directions on real data first.
- **A commander's clear is free when**: every landed branch merged, gated
  and pushed; nothing in flight owing its result elsewhere; this handoff
  current with the headline *measured*; nothing unfiled. `ccc clear <ref>
  --then "continue"` arms it — 0.1.29 fixed the gate, and the next cut
  resolves a session **name** everywhere rather than only a daemon id.
- **Run the diff suite with `--release`** under `tools/memcap.sh 20` — the
  cap is its first argument — redirected to a file, never piped (a pipe
  launders its 137), keeping the `peak` line; a long wait is the capture,
  not a hang. **Before a blind fan-out**, `grep -n <mechanic> CLAUDE.md`
  and the memory index — a subagent inherits both, and neither they nor a
  brief may quote this file or the journal.
