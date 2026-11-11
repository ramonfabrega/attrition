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

