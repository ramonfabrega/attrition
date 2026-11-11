# odd-counters: blind second reading (item 4152)

Slug `odd-counters`. Leaves (all `int`, `LeaderData`): `repair_stamp` (+0x208), `missiles_used` (+0x7c0), `units_bribed` (+0x8f4), `nukes_launched` (+0xa38).
Reader: Claude Opus 5.5 (claude-opus-5-5), blind (has not opened the first reading or the files the brief bars).
Sources: the decompile export at `~/ghidra-projects/decomp`, the listing of `riseofnations.exe` (a full `.text` objdump), `crates/sim`.

## Ground the four leaves share

- `Leader` (size `0x6eec`) is `LeaderData` (size `0x6ee4`) at offset 0 plus a
  vbtable pointer at `+0x6ee4` (`Leader::Leader (006d2bf0)` writes
  `field_0x6ee4 = &_vbtable_`). So a `Leader` method's `this->field_0xNN` is
  the `LeaderData` offset unshifted: `Leader::reset_score (006e37f0)` fills
  seven `0xffffffff` from `field_0x8f8`, which is `age_stamp int[7]` at
  `LeaderData +0x8f8`, and `Setup::build_game (005ac190)` tests
  `*(byte*)leaders.list[i] & 1`, which is `leader_flags` at `+0x0`.
- Global array: `leaders` at `0xe3a390`, stride `0x6eec`, ten slots (loop
  bound `0xe71af0` in `build_game`). Absolute forms of the leaves are
  `0xe3a598` (`+0x208`), `0xe3ab50` (`+0x7c0`), `0xe3ac84` (`+0x8f4`),
  `0xe3adc8` (`+0xa38`).
- **Both nets, run once for all four.**
  - Export: `->repair_stamp` / `->missiles_used` / `->nukes_launched` hit only
    readers (`LeaderData::log_data (006e5110)`, `LeaderData::get_support_count
    (006da110)`); `->units_bribed` hits nothing. `field_0x208`, `field_0x7c0`,
    `field_0x8f4`, `field_0xa38` on a `leaders.list[...]` or `Leader*` base:
    the writers below and nothing else (the many `field_0x208` hits are
    `Objects`/`ObjectType`/`UnitType`/`BuildType`, other records).
    The `Leader`-shifted forms are the same offsets (prefix 0).
  - Listing (the full `.text` objdump): every absolute address
    `A` in `[0xe3a390 − 2·0x6eec, 0xe3a390 + 10·0x6eec)` with
    `(A − 0xe3a390) mod 0x6eec ∈ {0x208, 0x7c0, 0x8f4, 0xa38}` — a scan that
    also catches index-biased bases — gives exactly five instructions on the
    leaders array (`5eea82` read and `5eea98` store in `Unit::do_repair`,
    `6188fa` in `Unit::come_out`, `67306e` in `SpellType::cast_bribe`,
    `67860e` in `Ammo::do_damage`) plus two hits at `0xe33c64` inside
    `PublishWorkshopWin` (a UI string object, not a leader). Register-relative
    stores ending in `0x208(%r)`, `0x7c0(%r)`, `0x8f4(%r)`, `0xa38(%r)` (and
    the `(%r,%r[,s])` and `leal` forms) map to `Leader::init (006e3930)` and
    `Leader::reset_score (006e37f0)` among `Leader`/`LeaderData` methods; the
    rest are `Constants::init`, `GameSpyGroup`, `IFace*`, `TerrainMap`,
    `GraphicPieces`, `ChatSpace`, `AchieveChartPane`, `TileSetData`,
    `ConquestSetupWin`, `ScenarioRead::load_leaders_chunk` (its `0x208(%edx)`
    has `edx = *0xc061ec = GameAccess::game`, the lobby's player records).
  - Whole-record clears: `memset(this, 0, 0x6929)` in `Leader::close
    (006b7f80)`, `Leader::init (006e3930)` (`6e398c`) and `Leaders::clear
    (006ed5d0)` — `0x6929` covers all four leaves. The scenario loaders
    (`ScenarioRead::load_leader_chunk (009ac080)`, `load_leaders_chunk`) write
    none of them; a scan for `$0x6929`, `$0x1bbb`, `$0x6eec`, `$0x4554c`
    found no other bulk writer.

## `repair_stamp` (`LeaderData +0x208`, int)

**1. Writers.**
- `Unit::do_repair (005ee420)`, store `5eea98` (`movl %edx, 0xe3a598(%ecx)`,
  `ecx = owner·0x6eec`, `edx = game->frame` from `0x550(GameAccess::game)`).
  Decompile line 262.
- `Leader::init (006e3930)`: the opening `memset(this,0,0x6929)` and an
  explicit `field_0x208 = 0` at `6e3e07` (decompile line 260).
- `Leader::close (006b7f80)`, `Leaders::clear (006ed5d0)`: the `0x6929` memset.

**2. Event, step, predicate.** It is a frame stamp, not a counter: the
throttle on the "cannot afford the repair" complaint. In `do_repair`, after
the repair amount for this frame is known (`amount`, hit points ×1, nonzero),
the unit's owner `p = unit+0x9` walks the six goods `g = 0..5` it has
(`LeaderData::type_avail(g,1)`); for each with a nonzero per-type cost
(vslot `0x78` of the site's type), it computes a due of 0 or 1 (the
float32 bucket test on hit points against `max_hits / cost`, not this leaf's
business). The first good whose stock (`*(leader+0x6eb8)[g] ^ 0x8221`, the
encrypted holdings, in the holdings' own units) is **signed-less than** its
due (`0` against `1`) ends the call:
1. `Unit::kill_current_order(0)` (`5eea66`);
2. `d = frame − repair_stamp[p]` (`5eea82`); if `d <= 150` (`cmpl $0x96; jle`
   at `5eea88`, signed 32-bit) return with nothing more;
3. else `repair_stamp[p] = frame` (`5eea98`), and only if `p ==
   console->who` (`0x298(*0xc06210)`) the red feedback and `S_INVALID_ORDER`.

So the stamp is written for every player, AI or human, console or not; only
the message is console-gated. Units: sim frames. The constant is `0x96` =
150 frames inclusive, i.e. a new stamp needs `frame − stamp >= 151`. With the
cleared value 0, no stamp is written before frame 151 even on a refusal.
No goods are charged on the refusing call (the charge loop runs only after
all six pass).

**3. Reset.** `Leader::init` (memset + explicit 0), `Leader::close`,
`Leaders::clear`. Nothing else; `Leader::reset_score` does not touch it.

**4. Readers.** `Unit::do_repair` itself (`5eea82`, the throttle) and
`LeaderData::log_data (006e5110)` line 608 (the leaders dump). The crate's
checksum walk lists it (`crates/sim/src/checksum/leaders.rs:74`). No gameplay
reader beyond the throttle.

**5. Crate.** The whole branch exists: `do_repair_steps` in
`crates/sim/src/orders/jobs.rs:1802` (the `0x97` test at ~line 1916) over
`DoRepairView`, whose `feedback_stamp`/`set_feedback_stamp` are no-ops in
`SimDoRepair` (`crates/sim/src/orders/jobs.rs:2105`, "SEAM: the complaint's
throttle stamp is not carried", returns `i32::MIN`). A builder adds a
per-player `repair_stamp` (beside the other leader stamps, e.g. on
`PlayerTech`) and wires those two methods; `rondata`'s checksum lines carry
it as `repair_stamp (unheld)` (`crates/rondata/src/diff/checksum_lines.rs:1821`).

**6. Plain game.** Yes. `do_repair` runs for any citizen on a REPAIR order
against its own (or a mutually allied) damaged building; a skirmish player,
AI included, with an empty stock of a good the repair charges reaches it.

**7. Confidence.** High for the writer set (both nets, plus the absolute
scan) and for the 150-frame throttle and the reset set (listing read). The
float bucket that decides the due was not re-derived (it is the repair
mechanic, not this leaf). Not read: whether any savegame path restores it
(none found by the bulk-writer scan; the scenario chunks do not carry it).

