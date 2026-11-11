# Item 4152, family 15 (`odd-counters`): adjudication

- Slug: `odd-counters`. Leaves (all `int`, `LeaderData`): `repair_stamp`
  (+0x208), `missiles_used` (+0x7c0), `units_bribed` (+0x8f4),
  `nukes_launched` (+0xa38).
- Readings: `odd-counters-reading.md` (first), `odd-counters-second.md`
  (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep. Nothing run of
  the original. Leaves are appended as each is settled.

## Ground shared by the four leaves

Settled once here, cited per leaf.

- **Absolute net (rerun).** Every absolute operand `A` in the full listing
  with `A` in `[0xe3a390 − 2·0x6eec, 0xe3a390 + 10·0x6eec)` and
  `(A − 0xe3a390) mod 0x6eec ∈ {0x208, 0x7c0, 0x8f4, 0xa38}`: exactly
  `5eea82` (read) and `5eea98` (store) in `Unit::do_repair (005ee420)`,
  `6188fa` in `Unit::come_out (00617c10)`, `67306e` in
  `SpellType::cast_bribe (00672d00)`, `67860e` in `Ammo::do_damage
  (00678060)`, plus three `0xe33c64` hits in `PublishWorkshopWin` (a UI
  object below the array, not a leader). Both readers' absolute lists are
  this list.
- **Register-relative net (rerun).** Every `mov/inc/dec/add/sub/and/or/xor/
  neg/not/lea` whose memory destination is `0x208(`, `0x7c0(`, `0x8f4(` or
  `0xa38(` (with or without an index register), mapped through
  `INDEX.tsv`: the only `Leader`/`LeaderData` methods are `Leader::init
  (006e3930)` (`6e3e07` +0x208, `6e3bbc` +0x7c0, `6e4a9c` +0xa38) and
  `Leader::reset_score (006e37f0)` (`6e38bf` +0x8f4). The others are
  `Objects`, `ObjectType`/`UnitType`/`BuildType`, `Scene`, `MouseOver`,
  `TileSetData`, `PlayerProfile`, `SteamLobbyMgr`, `MemTracker`,
  `Constants::init`, `GameSpyGroup`, `ConquestMap`, `ConquestGame`,
  `IFaceMainBase`, `IFaceRenderManager`, `AchieveChartPane`,
  `IFaceResources`, `GraphicPieces`, `ChatSpace`, `TerrainMap`, plus
  `Unit::come_out`'s `617c53` (`0xa38(%esp)`, a stack slot) and
  `ScenarioRead::load_leaders_chunk (009acde0)`'s `9acf92`, whose `%edx` is
  `*0xc061ec` loaded at `9acde6` (the game object's lobby records, not a
  leader). No writer either reader missed.
- **Whole-record clears.** `pushl $0x6929` occurs at `6b7f82`
  (`Leader::close (006b7f80)`), `6e398c` (`Leader::init`) and `6ed5e0`
  (`Leaders::clear (006ed5d0)`), and nowhere else as a size (the fourth hit,
  `875783` in `TerrainOut::caravan_step`, is an `addl`). `0x6929` covers all
  four leaves. In `Leader::init` the memset is **conditional**: `6e3948`
  `testl %esi,%esi; jns 6e3a36` skips it when the second argument is `≥ 0`;
  the `< 0` arm falls through to `6e3a36` and both arms then run the
  explicit per-field zeroes. `Leaders::clear` is called from `Game::close`
  (`594faa`) and `Game::run_oos_recovery` (`586dd6`); `Leader::close` has no
  direct `calll` in the listing. Neither is a per-match reset in a plain
  game beyond teardown.
- **Restore.** `LeaderData::walk_data (006d6750)` streams all four on a
  load (both readers). It restores, it does not step.

## Leaf `repair_stamp` (+0x208)

### 1. Claim by claim

| # | claim | first | second | mark |
|---|---|---|---|---|
| 1 | event writer `Unit::do_repair (005ee420)`, store `5eea98`, value `game->frame` | yes | yes | agree |
| 2 | `Leader::init` explicit `= 0` at `6e3e07` | yes | yes | agree |
| 3 | `Leader::init`'s memset: when `tribe < 0` (first) / "the opening memset" (second, no condition) | conditional | unconditional by wording | disagree |
| 4 | `Leader::close (006b7f80)` and `Leaders::clear (006ed5d0)` memsets also clear it | — | yes | one-sided |
| 5 | `walk_data` restores it | yes | yes | agree |
| 6 | predicate: first good `g` in `0..5` that is `type_avail(g,1)`, has nonzero price, and whose stock `< due[g]` (0 or 1) ends the call: `kill_current_order(0)`; if `frame − stamp ≤ 150` return; else stamp, then message only for the console | yes | yes | agree |
| 7 | the comparisons are signed (`jl`, `jle`) | implied | yes | agree |
| 8 | stamp written for every player, before the console test | yes | yes | agree |
| 9 | no goods are charged on the refusing call | — | yes | one-sided |
| 10 | index is the repairer's owner `(byte)unit+0x9` | yes | yes | agree |
| 11 | reset: `Leader::init` only among events; not `reset_score` | yes | yes (+ #4) | agree |
| 12 | readers: `do_repair` (throttle) and `log_data`; nothing else | yes | yes | agree |
| 13 | crate: `do_repair_steps` (`orders/jobs.rs:1802`), throttle at `:1915`, stubs in `SimDoRepair` at `:2105`–`2110`; `checksum/leaders.rs:74` | yes | yes | agree |
| 14 | the `i32::MIN` stub makes the throttle **always return** (not "always due" as the SEAM says) | yes | — | one-sided |
| 15 | `rondata` prints it `repair_stamp (unheld)` (`checksum_lines.rs:1821`) | — | yes | one-sided |
| 16 | a plain game reaches it | yes | yes | agree |

### 2. Settlements

- **#3 corrected (wording of the second).** The memset in `Leader::init`
  runs only when the second argument is negative: `6e3948 testl %esi,%esi`,
  `6e394a jns 6e3a36`. The explicit `movl %eax, 0x208(%ebx)` at `6e3e07`
  (`eax` zeroed at `6e3dce`, untouched through `6e3e02`) runs on both arms,
  so the leaf is 0 after every `Leader::init` either way. The first reading
  had it right. No consequence for a builder.
- **#4 confirmed.** `6b7f82` and `6ed5e0` push `0x6929`. They are teardown
  and out-of-sync recovery (callers in the ground section), not resets a
  plain match runs between frames.
- **#6, #7 confirmed against the listing.** `5ee91a calll *0x78(%eax)` is
  the price; `5ee922 je 5ee9a9` skips the good (and its stock test) when the
  price is 0. The due is `5ee968`..`5ee97f` (truncated single-precision
  quotients, `due = 1` when `before < after`). Stock `5ee991`..`5ee99a`
  (`*(leader+0x6eb8)[g] ^ 0x8221`), `5ee99f cmpl -0x34(%ebp,%esi,4), %eax;
  5ee9a3 jl 5eea62`: signed `stock < due`. Then `5eea66` kill order,
  `5eea82 subl 0xe3a598(%ecx)`, `5eea88 cmpl $0x96; 5eea8d jle` (return
  when `frame − stamp ≤ 150`, signed, wrapping), `5eea98` store, `5eeaa4
  cmpl` against `console->who` (`0x298(*0xc06210)`) after the store.
  **One edge neither reader stated:** the test is `stock < due` with `due`
  possibly 0, so a good with nonzero price and a **negative** stock trips
  the refusal even when this step charges nothing of it. Whether a stock
  can go negative is the economy's question; the charge loop below clamps
  only the second ledger (`cmovs` at `5eea21`), not the bucket. A builder
  keeps the comparison signed and against `due`, which the crate already
  does (`jobs.rs:1912`).
- **#9 confirmed.** The charge loop (`5ee9b3`..`5eea2f`) is reached only by
  falling out of the six-good test loop at `5ee9ad`; the refusal branches
  out at `5ee9a3` before it.
- **#13 confirmed.** `jobs.rs:1915` `frame.wrapping_sub(v.feedback_stamp())
  < 0x97` is the same test as `≤ 0x96`; the view stubs are at `:2105`–`2110`;
  the bucket and the index are the repairer's owner (`:2102`), matching
  `movzbl 0x9(%ebx)` at `5eea6b`.
- **#14 confirmed.** With `feedback_stamp() == i32::MIN`,
  `frame.wrapping_sub(i32::MIN) = frame + 2³¹` (wrapping) is negative for
  every `frame ≥ 0`, so `< 0x97` holds and the stub returns before
  `set_feedback_stamp`. Harmless while both arms are empty; the SEAM text is
  wrong about which way it falls.
- **#15 confirmed** by grep (`crates/rondata/src/diff/checksum_lines.rs:1821`).
- **Writer completeness.** The ground nets find nothing more; the export's
  `field_0x208` hits outside `do_repair`/`Leader::init` are all
  `GameAccess::objects->field_0x208` (`Unit::fight`, `detect_boat_collision`,
  `cast_entrench`, `Nuke::do_damage`, `Objects::*`).

### 3. The rule as a builder should port it

- Storage: per player `repair_stamp: i32`, initial 0.
- Writer: in `do_repair_steps`, at the refusal (first available good with
  nonzero price whose stock `< due[g]`, signed): kill the repairer's order;
  if `frame.wrapping_sub(repair_stamp[owner]) ≤ 150` return; else
  `repair_stamp[owner] = frame`, then the console-only message. Every
  player, AI or human. Assignment, not a step. No charge on that call.
- Reset: 0 at `Leader::init`; nothing mid-game. Restored by a load.
- Crate site: `SimDoRepair::feedback_stamp`/`set_feedback_stamp`
  (`crates/sim/src/orders/jobs.rs:2105`–`2110`) read and write it; wire the
  checksum row `repair_stamp` (`checksum/leaders.rs:74`).
- Plain game: yes (any repairer short of a charged good, after frame 150).
- Confidence: high (listing read end to end over the branch). Open: none
  for the leaf; the float `due` is the repair mechanic's, ported under item
  3346 and not re-derived here.

## Leaf `missiles_used` (+0x7c0)

### 1. Claim by claim

| # | claim | first | second | mark |
|---|---|---|---|---|
| 1 | event writer `Ammo::do_damage (00678060)`, `67860e incl 0xe3ab50(%eax)`, `eax = AmmoData::who (+0x3c) · 0x6eec` | yes | yes | agree |
| 2 | `Leader::init` explicit `= 0` at `6e3bbc`; memset (and `close`/`clear` memsets, second) | yes (init) | yes (+ close, clear) | agree / one-sided as for `repair_stamp` |
| 3 | outer guards: `o ≥ 0`, `who ≥ 0`, graphic-piece type `≠ 9` | yes | yes | agree |
| 4 | arm: `splash_area (+0x30) ≠ 0` | yes | yes | agree |
| 5 | shooter type `obj_masks (+0x1e4) & 0x8000000`, else straight to `hit_target` at `678614` | yes | yes | agree |
| 6 | `is(0x13b, 0)` evaluated and kept at `-0x1c(%ebp)` before the shield | yes | yes | agree |
| 7 | shield: landing owner `L ≥ 0`, `has_preq(L, MISSILE_DEFENSE_BONUS)`, `L ≠ who` → close, no count; `≠`, not "enemy" | yes | yes | agree |
| 8 | ocean landing: owner from a 120-entry ring at `0xadcaf4`/`0xadc404` | yes (decompile only) | yes | agree |
| 9 | nuke line (`-0x1c ≠ 0`) → nuke arm, returns, no count | yes | yes | agree |
| 10 | otherwise `S_NUKE_HIT (0x50)` then `+1`, then `hit_target`/splash | yes | yes | agree |
| 11 | reset: `Leader::init` only; never decremented | yes | yes | agree |
| 12 | reader: `get_support_count (006da110)` adds it for type index `0x139` or `0x13a` (exact), feeding `TypeData::get_cost (00664090)` | yes | yes | agree |
| 13 | the term is on `get_support_count`'s fall-through only: a type with `progression & 1` and `+0x1e8 ≠ 0` returns its group count first | — | yes | one-sided |
| 14 | crate: counter in `Sim::land` (`combat/fight/projectiles.rs`) after the nuke arm's `return`, under `mask::MISSILE`, keyed `p.owner`; storage beside `Nukes::used` (`combat/nuke.rs`); reader `nuke_ramp_extra` (`economy/pricing.rs:287`) | yes | yes | agree |
| 15 | aside: the crate runs `hit_target`/`check_hit` before the shield and nuke tests | yes | yes | agree |
| 16 | a plain game reaches it (Industrial/Modern, V2 Rocket / Cruise Missile) | yes | yes | agree |

### 2. Settlements and spot-checks

- **#3–#6 confirmed against the listing.** `678186`..`67819f`: `o
  (+0x40) js 67845d`, `who (+0x3c) js 67845d`, `cmpl $0x9 ... je 67845d`;
  `6781a5 cmpl $0x0, 0x30(%ebx); je 678ba9` (the zero-splash arm leaves);
  `678200 testl $0x8000000, 0x1e4(%eax); je 678614` (no count); `678212
  pushl $0; 678214 pushl $0x13b`, result stored `67823e -0x1c(%ebp)`.
  The shooter object is `objects[who][o]` (`6781e7`..`6781fd`), i.e. the
  missile unit itself.
- **#7 confirmed.** `678335 js 67846d` (owner < 0 skips the shield),
  `678343 pushl $0x2b5` (693 = `MISSILE_DEFENSE_BONUS`) → `has_preq
  (006db810)`, `678355 je 67846d`, `67835b`..`678363 cmpl %eax,%edx; je
  67846d` (owner equal to `who` skips). Otherwise messages and
  `Ammo::close (006791a0)` at `67845f`, no count. The test is `L ≠ who`:
  an ally's shield stops it too.
- **#8 confirmed from the listing**, which the first reading had not
  done: `678267 calll WorldData::is_ocean (006b4830)`; zero goes to the
  direct tile at `678311`; nonzero walks the ring `678281`..`678303`
  (`0xadcaf4(%ecx)` dx, `0xadc404(%ecx)` dy, `ecx += 4` to `cmpl $0x1e0`:
  120 entries), skipping off-map and ocean cells, taking the first cell's
  owner (`movsbl 0xf(tile)`, **signed byte**) that is `≥ 0`
  (`6782e8 jns 67833d`). No owner found leaves `−1` and skips the shield.
- **#9, #10 confirmed.** `67846d cmpl $0x0, -0x1c(%ebp); je 6785f0`; the
  nuke arm ends `6785e7`..`6785ef` (`Ammo::close`, `xorl %eax,%eax`,
  `ret`). `6785fe pushl $0x50; 678602 calll` (the sound), `678607 imull
  $0x6eec, 0x3c(%ebx)`, `67860e incl 0xe3ab50(%eax)`, then `678614`.
- **#12 confirmed.** `get_support_count` decompile lines 36–41: the
  `nukes_used` term for `0x13b`/`0x13c`, then `missiles_used` for `0x139`
  (`V2ROCKET`, `enums/TypeIndex.txt:336`) or `0x13a` (`CRUISEMISSILE`),
  exact equality.
- **#13 confirmed, and settled for the shipped data.** Decompile lines
  15–33: `(type+0x2f4 & 1) && type+0x1e8 ≠ 0` returns a group count
  (barracks `0x1ab`, stable `0x1ac`, factory `0x1ae`, dock `0x1b0`, or
  domain `+0x218 == 2` air) before line 36. `+0x2f4` is `progression`
  (`types.txt`). In the install's `data/unitrules.xml`, V2 Rocket and
  Cruise Missile both have `<PROGRESSION>0</PROGRESSION>` (and `DOMAIN
  Air`, `ATTACK` 150 / 300), so they skip the group arm and **do** read the
  term; were the progression bit set, the `DOMAIN Air` arm would return
  `air_queued + air_units` and the term would be dead. A builder keeps the
  term on the by-type count only, which is where `nuke_ramp_extra` already
  feeds (`of_type`, `pricing.rs:851`–`855`). Also after the term: the
  Americans' bomber discount (`has_tribe_bonus(0x14)` and `is(0x130)`,
  lines 42–55) does not touch the missiles.
- **#14 confirmed.** `projectiles.rs:560` `land`; shield
  `:608`–`614`, nuke arm `:617`–`620`; `combat/nuke.rs:76` `pub used`,
  `:89` `used_of`, `:134` `nuke_land`; `pricing.rs:281`–`300`
  `nuke_ramp_extra`, whose doc says the term is not modelled.
- **#15 (aside, not this leaf).** Confirmed as stated by both: the
  original reaches `hit_target` at `678614` only after the missile tests,
  the crate calls it at `projectiles.rs:577`. Observable only if
  `hit_target`/`check_hit` have side effects on a round that then dies;
  not adjudicated here.
- **Side finding (crate, not this leaf's evidence).** `crates/sim/src/ids.rs`
  names `0x139` `CRUISE`; the enum names `0x139` `V2ROCKET` and `0x13a`
  `CRUISEMISSILE` (`crates/sim/src/air/airbase.rs:42`/`48` have them
  right). A builder wiring this term should key on the numbers, not on
  `ids::CRUISE`.
- **Writer completeness.** Ground nets: no other writer. The export's
  `field_0x7c0` on a leader base appears only in `Ammo::do_damage` and
  `Leader::init`.

### 3. The rule as a builder should port it

- Storage: per player `missiles_used: i32`, initial 0.
- Writer: in `Sim::land`, on the splash arm (`splash_area ≠ 0`), when the
  shooter's profile has `mask::MISSILE`, the shield did not close the round,
  and the shooter is not of the Nuclear Missile line: `missiles_used
  [p.owner] += 1` (the round's `who` at launch). Put it right after the
  `is_nuke` return and before the splash walk.
- Reset: 0 at `Leader::init`; never decremented. Restored by a load.
- Reader: `get_support_count`'s by-type fall-through adds it for exact type
  `0x139` or `0x13a`; port as a third term in `nuke_ramp_extra` (`if t ==
  0x139 || t == 0x13a { extra += missiles_used[who] }`), so every V2 or
  Cruise Missile that lands makes that player's next one dearer.
- Checksum: `checksum/leaders.rs:141` row.
- Plain game: yes, from the Industrial age with missiles built.
- Confidence: high for writer, predicate and reader (listing read across
  `678186`..`67861b`). Open: which types carry `obj_masks` bit 27 outside
  the nuke line is data (the crate's `mask::MISSILE`), and whether a
  missile ever produces more than one round per detonation (both readers
  assume one); a capture with a V2 landing and a later V2 priced settles
  both.

