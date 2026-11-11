# score: second (blind) reading

Slug `score`. Leaves: `score` (+0x18), `score_explored` (+0x1c), `score_territory` (+0x20), `score_units` (+0x24), `score_units_2` (+0x28), `score_buildings` (+0x2c), `score_economy` (+0x30), `score_pop` (+0x34), `score_unit_upgrades` (+0x38), `score_research` (+0x3c), `score_wonders` (+0x40), `score_combat` (+0x44), all `int` in `LeaderData`.

Reader: Claude Opus 5.5, blind (has not opened the first reading). Sources: decompile export `~/ghidra-projects/decomp`, the PE listing `exe.s`.

## 0. The family at a glance

Eleven of the twelve leaves are written only by `Leader::compute_score (006ec560)` and the
part functions it calls, all `__thiscall` on a `Leader*` in `esi`/`ebx`/`edi`
(`mov %ecx,%esi` etc. at each prologue). `Leader` (size 0x6eec) lists no fields of its own in
`types.txt`; it is `LeaderData` (0x6ee4) at offset 0 plus 8 trailing bytes. The proof that the
base is `LeaderData+0` and not shifted: the same `this` is read as `leader_flags` at `(%esi)`
(`testl $0x1800000,(%esi)`, 006ec59a) and as `who` at `0x8(%esi)` (`imull $0x6eec,0x8(%esi)`,
006ec5e7, used as the slot index into the global array at `0xe3a390`). So `0x18(%esi)` ..
`0x44(%esi)` are `score` .. `score_combat`. `score_combat` is the exception: the part function
`Leader::compute_combat_score (006bc250)` is an empty body and is never called; the leaf is
stepped event-by-event at kills, always through the absolute form `0xe3a3d4(%reg)` =
`0xe3a390 + 0x44 + slot·0x6eec`.

### 0.1 Cadence: `Leader::compute_score (006ec560)`

Callers (listing `call 0x6ec560`, two sites only, and the export agrees):

- `Leaders::strategy_all (006ed430)` at 006ed45b, `compute_score(this, 0)`, for every slot of
  the 8 (`0xe3a390`..`0xe71af0`, stride 0x6eec) with `leader_flags & 3 == 3`, between
  `plan_strategy` and `diplomacy`. `strategy_all` is called once per sim frame from
  `Game::do_frame (00591ef0)` (call 00592448) unless `semaphore` bit 11 (`semaphore.ptr[1] & 8`,
  Game+0x814 bits at +0x820) is set.
- `Game::defeat_all (00592aa0)` at 00592adb, `compute_score(this, 1)` for every slot with
  `leader_flags & 1`; its one caller is `GameDaemon::process_victory (00730ef0)` (call 00730f22),
  the end-of-game sweep.

The gate at the head (006ec563..006ec594): the body runs when **any** of
`game->frame == 0` (Game+0x550), `param_1 != 0`, `semaphore` bit 6 (`byte [Game+0x820] & 0x40`),
or `(who + frame) % 10 == 0` (signed `idiv` by 10, remainder tested; `who` = LeaderData+0x8).
Otherwise it returns with no store. So in a plain game each live leader recomputes on one frame
in ten, phased by slot: slot `w` on frames with `frame ≡ −w (mod 10)`, plus frame 0.

### 0.2 Body order

1. `score_explored = 0` (store `0x1c(%esi)` at 006ec5a0, unconditional once the gate passes).
2. If `leader_flags & 0x1800000` (bit 23 or bit 24): `compute_unit_score`, then clear bit 23
   only (`andl $0xff7fffff,(%esi)`).
3. `compute_build_score` (always), writes `score_buildings`, `score_wonders`.
4. `compute_economy_score` (always), writes `score_economy`.
5. `score_pop = 0` (006ec5ca, unconditional).
6. If `leader_flags & 0x1000000` (bit 24): `compute_unit_upgrades_score`, `compute_research_score`,
   then clear bit 24 (`andl $0xfeffffff,(%esi)`).
7. `score_territory = leaders[who].territory * 1000 / world->land_size` (signed `imul $0x3e8`,
   `cltd; idivl 0x78(%ecx)`, truncation toward zero). `territory` is LeaderData+0x9d8, read
   absolute at `0xe3ad68(%eax)` (`0xe3a390 + 0x9d8`); `land_size` is World+0x78 through the
   pointer at `GameAccess::world` (0x00c06188).
8. If `game->armageddon (Game+0x6e0) >= Game::get_armageddon (00594020)`: zero `score_economy`,
   `score_buildings`, `score_territory`, `score_units`, `score_explored`, `score_combat`,
   `score_wonders`, `score_research`, `score_unit_upgrades`, `score_pop`, `score` and return.
   **`score_units_2` (+0x28) is not zeroed here.**
9. Else `score = score_combat + score_wonders + score_research + score_unit_upgrades +
   score_economy + score_buildings + score_units + score_territory + score_pop + score_explored`
   (plain `add`s, wrapping i32; 006ec66d..006ec68a). **`score_units_2` is not a term.**

`Game::get_armageddon (00594020)`: `armageddon + armageddon_per_nation·num_nations +
armageddon_per_team·num_sides` (all `get_item`, plain integers, `Constants::init (00569a90)`
:2413-2417), ×3 when `info.starting_resources == 7`, ×2 when it is 5, 6, 11 or 12, floored at 100
when it is 8. Every part function repeats this test at its own head and leaves its part at 0
when armageddon has been reached, so the zeroing in step 8 is belt and braces except for
`score_territory`, `score_explored`, `score_pop`, `score_combat` and `score`.

### 0.3 The dirty bits (who sets `leader_flags` bits 23 and 24)

Listing net: every `orl $0x800000|$0x1000000` whose destination is a leader record (`(%reg)`
where the reg was built as `0xe3a390 + slot·0x6eec`, or `0xe3a390(%reg)`):

- bit 23 (`0x800000`): `Unit::close (0060ee50)` at 0060eedb (unit removed, owner byte
  `+0x9 < 8`) and `Unit::init (00612100)` at 00612d83 (unit created, `param_1 < 8`). The many
  other `orl $0x800000, 0x68(%reg)` hits are `UnitData::unit_masks`, not this field.
- bit 24 (`0x1000000`): `Leader::gain_tech (006dcb60)` at 006dcb99, unconditionally at entry.
  (The `orl $0x1000000, 0x4(%eax)` hits at 0051a33f, 00525128, 00891040.. are another
  record's `+0x4`; not a `Leader*`.)
- The only clears are the two `andl` in `compute_score` (listing grep for `andl
  $0xff7fffff|$0xfeffffff` on a leader record finds those two and nothing else).

Consequence a builder must keep: `score_units`/`score_units_2` are recomputed only on a gated
frame following a unit creation, a unit removal or a tech gain; `score_unit_upgrades` and
`score_research` only on a gated frame following a tech gain. Both include *queued/researching*
items in their inputs, but queuing alone does not dirty them, so the parts can sit stale.
A forced call (`param_1 = 1`) bypasses the cadence gate but **not** the dirty bits.

### 0.4 `TypeData::get_score_value (006673b0)` — the common value

Vtable slot `+0x7c` of every `Type` vtable. PE bytes: `UnitType`'s `Type` vtable 0x00b41fd4,
`BuildType`'s 0x00b42b94, `ObjectType`'s 0x00b43aac and `TypeData`'s 0x00b43e48 all hold
0x006673b0 at `+0x7c`; the map has one `get_score_value` symbol. With `S = Σ costs[0..6]`
(TypeData+0x18, six ints) of type `t'`:

- `t' = t`, except when `param_1 == 0` and `t ∈ {MILITIA 0x42, MINUTEMAN 0x43, PARTISAN 0x44}`,
  where `t' = BASE_UNITTYPES 0x32` (the costs of type 0x32 stand in).
- unit type (`0x31 < t < 0x19e`, or a non-inlined `is_unit_type` override true):
  `param_1 == 0`: `unit_cost_factor · S`. `param_1 != 0`:
  `x = research_premium · (unit_cost_factor · S)`; `x = trunc(x / 256) ·
  UnitTypeData::research_premium_cost(t')`; return `trunc(x / 256)` (the `(v + (v>>31 & 0xff)) >> 8`
  idiom is division toward zero). `research_premium` is 8.8 (`get_fraction(…,0x100)`,
  `Constants::init`:1161); `research_premium_cost` (UnitTypeData+0x2e0) is 8.8
  (`String::fraction(…,0x100)`, `UnitType::init (0061ab50)`:612).
- build type (`0x19d < t < 0x21f`): `build_cost_factor · S`.
- tech type (`0x21f < t < 0x275`) and the fallthrough: `tech_cost_factor · S`.
- spell type (`0x274 < t < 0x2ac`): `spell_cost_factor · S`.
- `unit_cost_factor`, `build_cost_factor`, `tech_cost_factor`, `spell_cost_factor` are plain
  `get_item` integers (`Constants::init`:1121-1128).

