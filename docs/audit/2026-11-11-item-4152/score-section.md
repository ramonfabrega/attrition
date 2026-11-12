## 18. The score and its eleven parts

### 18.1 Leaves

- `score` (+0x18), i32: the sum.
- `score_explored` (+0x1c), i32: always 0.
- `score_territory` (+0x20), i32.
- `score_units` (+0x24), i32.
- `score_units_2` (+0x28), i32: armed units; not summed, no reader.
- `score_buildings` (+0x2c), i32.
- `score_economy` (+0x30), i32.
- `score_pop` (+0x34), i32: always 0.
- `score_unit_upgrades` (+0x38), i32.
- `score_research` (+0x3c), i32.
- `score_wonders` (+0x40), i32.
- `score_combat` (+0x44), i32: an accumulator, never recounted.

### 18.2 The rule

**Reset**: `reset_score (006e37f0)` zeroes all twelve, called only from
`Leader::init (006e3930)` and `Setup::build_game (005ac190)` (default 0).

**When.** `Leaders::strategy_all` runs, for each of the 8 slots with
`leader_flags & 3 == 3`, **humans included**: `check_explore`,
`plan_strategy`, `compute_score(who, forced=0)`, `diplomacy`. `do_frame`
skips it when `Game+0x821 & 8`. `compute_score` recomputes when

    frame == 0 || forced || (Game+0x820 & 0x40) || (who + frame) % 10 == 0

(`frame` before `do_frame` increments it, signed remainder), else changes
nothing. Bit 6 of `Game+0x820` has four setters: `check_victory`'s game-over
latch, `RecordGame::read_package`, `TurnControl::toggle_pause`,
`MPDropWin::on_button_clicked`. `defeat_all`'s forced `compute_score(1)` runs
only after Armageddon (`process_victory` calls it only when
`get_armageddon() <= game->armageddon`), so it only zeroes.

**Dirty bits** of `leader_flags`, cleared only by `compute_score`: bit 23
(`0x800000`) set by `Unit::init (00612100)` (owner `< 8`, signed) and
`Unit::close (0060ee50)` (owner byte `< 8`); bit 24 (`0x1000000`) set by
`Leader::gain_tech (006dcb60)` at entry. The whole-word stores in
`Game::init_rules_and_teams (00589bb0)` and `Game::init_tribes (0058a600)`
are read-modify-write and keep both.

**A recompute, in order**: `score_explored = 0`; units if bit 23 or 24, then
clear 23; buildings and wonders; economy; `score_pop = 0`; upgrades then
research if bit 24, then clear 24; territory. Then, if
`game->armageddon (G+0x6e0) >= get_armageddon()`, zero every leaf **except
`score_units_2`** and return; else

    score = combat + wonders + research + upgrades + economy + buildings
          + units + territory + pop + explored        (i32 wrapping)

Each part also zeroes and returns early after Armageddon. `get_armageddon = ARMAGEDDON (C+0xd14) + ARMAGEDDON_PER_TEAM
(C+0xd1c)·num_sides (G+0x6a8) + ARMAGEDDON_PER_NATION (C+0xd18)·num_nations
(G+0x6a0)`; ×3 if `starting_resources (G+0x2d)` is 7, ×2 if 5, 6, 11 or 12,
floor 100 if 8.

**Values.** Divides truncate toward zero; multiplies wrap at i32.
`value(t, p)` is `get_score_value` (vslot `+0x7c`, one body `006673b0`),
`S = Σcosts[0..6]`:
- `p == 0`: `factor × S` (UCF for units, TCF for techs). The militia family
  is priced from type 0x32, still with UCF; range tests read the type's own
  index (`TypeData+0x4`).
- `p == 1`: `((RP × (UCF × S)) / 256 × research_premium_cost(t)) / 256`, in
  that order; no militia substitution.

`kill(t, o)` (`get_kill_value`: unit `0061d3d0`, building `00639790`, other
`00663a00`) is `value(t,0) + (sc0+sc1)·n`, times `BUILD_SUPPORT_FACTOR`
(constants `+0x37c`) on the support term for a building. `sc0`, `sc1` are the
type's `support_cost`s; `n` is owner `o`'s count of `t` **plus its
`num_queued[t]`** (the kill is priced as the owner's next one); `o < 0` gives
`value(t,0)`.

- **Territory**, every recompute:
  `territory (+0x9d8) × 1000 / land_size (WorldData+0x78)` (faults on 0).
- **Units** (`Leader::compute_unit_score (006bc500)`), when bit 23 or 24: both
  to 0; for `t` in `0x32..0x192` with `n = num_units[t−0x32] + num_queued[t] > 0`,

      u = (value(t,0)·n + ((n−1)·(sc0+sc1)·n) / 2) / 5

  to `score_units`, and to `score_units_2` too when `attack (+0x1e8) != 0`.
  No BSF.
- **Buildings, wonders** (`Leader::compute_build_score (006bc3f0)`), every
  recompute: both to 0; for `t` in `0x19e..0x21f` with
  `n = num_buildings[t−0x19e] + num_queued[t] > 0` (queued = inactive sites),

      b = value(t,0)·n + ((n−1)·((sc0+sc1)·BSF)·n) / 2

  `b / 3` to `score_wonders` when the own index is in `0x20e..0x21f`, else
  `b / 5` to `score_buildings`.
- **Economy**, every recompute: to 0; for `g in 0..6` with
  `type_avail(g, 1) != 0`, add
  `clamp((bucket[g] − starting[g]) / 20, 0, 100) + min(income[g] / 80, 200)`
  (decrypted `^ 0x8221`, `^ 0x90236`; `starting` is `Game+0x600`; income has
  **no floor**). Then `+ 25 × popcount(rare)` (`+0x6d98`). Side effect:
  `BitMask<44>::count (0047cd40)` writes `rare.flags` (`+0x6da0`) to 1 when
  the count is 0, else 0, and returns 0 at once when it is already 1.
- **Unit upgrades**, when bit 24: to 0; for `t` in `0x32..0x192` add
  `value(t,1) / 5` when (`has_tech(t)` or `researching(t, −1, 0, 1)`, an exact
  match in a queue of my active building), `tech_at_start[t]` (`+0x6c8c`) is
  clear, and `unit_flags (+0x2b4) & 0x80` is clear.
- **Research**, when bit 24: to 0; for `t` in `0x220..0x275` (ages included)
  held or queued (`researching(t, −1, 0, 0)`, exact for techs) and not
  `tech_at_start[t]`, add `(TCF × S) / 5`.
- **Explored, pop**: 0. `compute_explore_score (006bc5e0)`,
  `compute_pop_score (006bc190)` and `compute_combat_score (006bc250)` are
  dead (no call, no pointer).

**`score_combat`**: each event moves it by `v = kill(t, o) / 5`, `o` the
victim's owner. Its only stores: `Object::do_damage (0064a480)`,
`Object::take_damage (00652020)`, `SpellType::cast_sabotage (006720e0)`,
`SpellType::cast_counterintel (00673740)`, the Armageddon zero.
- **Unit killed** (`take_damage`). A building victim takes `+0x81c += 1` and
  leaves. Else it must be a captain (vslot `+0xe8`) on its last figure
  (`curr_uber_size(0) == 1`), not a decoy (`unit_masks & 1`): victim owner
  `+0x810 += 1`. Then **only if the killer id `param_7 >= 0`**: `+0xa58`,
  `+0xa5a`, `+0x80c` move; `unit_flags & 0x10` redirects the victim to
  `(+0x3e, +0x28)`; a negative id or a false vslot `+0x18` scores nothing;
  `sc[K] = is_ally(K, o) ? max(sc[K] − v, 0) : sc[K] + v`. No killer-kind or
  splash test. It runs before the victim's `die`: `n` counts the victim.
- **Building razed** (`do_damage`): type captured before `take_damage`; when
  that returns non-zero and the target is a building, `+0x818 += 1` and the
  same ally rule, credited to the attacker's owner. `die` has already run
  `Build::close (00628980)` → `Wall::decrement_stats (00642da0)`: `n`
  **excludes** the razed building (or site).
- **Sabotage**, unit arm (captain, last figure, not decoy) or building arm
  (`take_damage` non-zero, `param_7 = −1`, so no double count):
  `sc[caster] += v`, no ally test, no floor.
- **Counterintelligence**, on a target climbed to its captain whose owner is
  neither the caster nor allied both ways, single figure, not decoy:
  `sc[caster] += v`.

**Readers.** `Army::find_target (006f69b0)` takes an arm when
`((score[c] << 2) / 3) < score[army owner]` (shift wraps). `process_victory`
with `victory == 5` defeats the live leader with the strictly lowest score; a
tie at the minimum defeats nobody. `Leader::diplomacy` counts live leaders
scoring above me and finds the top scorer.

### 18.3 In the crate

- **Step site**: `Sim::strategy_all` (`crates/sim/src/ai/ai_drive.rs:73`,
  comment `:87`). **Trap**: the human arm `continue`s at `:82`–`:85`, ahead
  of the site; the score must run before it.
- **Inputs**: `Muster::by_type`/`queued_by_type` (`units/unit.rs:838`,
  `:840`); `Holdings::territory`/`land_size` (`economy.rs:554`–`555`);
  `Ledger::bucket`/`income`, `Holdings::starting`/`rare`
  (`economy.rs:679`–`699`). Building `num_queued` as `num_sites_of` is the
  crate's own claim.
- **Dirty-bit hooks**: `add_unit` (`units/store.rs:48`), `close_unit`
  (`combat/fight/delivery.rs:1245`), `tech/gain.rs:189`.
- **Kill value**: `economy::cost::kill_value_by` (`economy/cost.rs:1258`)
  ports the building form, swept in
  `crates/sim/tests/sweep/build_type_data_get_kill_value.rs`, uncalled;
  `factor = 1` with the unit arrays gives the unit form.
- **Combat**: unit arm in `take_damage_typed` (`delivery.rs:537`–`543`, by
  `unit_plunder`); building arm in `do_damage`'s `killed` arm
  (`delivery.rs:215`), which has the attacker, not `damage_building` (`:957`,
  only an optional `by`). The predicate is wider than plunder's: any
  attacker, splash included, an ally subtracts. Sabotage SEAMs
  `delivery.rs:804`, `:882`; counterintelligence `Sim::cast_counterintel`
  (`spells/cast.rs:1031`), SEAM `:1028`.
- **Readers seamed**: `leaders/diplomacy.rs:155`, `:385`;
  `ai/army/target_score.rs:457`.
- **Trap, `is_ally` past the table.** `LeaderData::is_ally (006edb50)`
  indexes `diplos int[8]`; gaia units are owner 8 (and 9; `gaia.rs:93`). A
  gaia victim with a killer object calls `is_ally(K, 8)`, reading
  `diplos[8]` = `treaties[0]` (`+0x94`) and `leaders[8].diplos[K]`. The
  crate's `Sim::is_ally` (`leaders/relations.rs:99`) assumes no such caller;
  do not answer "not allied" from its guard (18.6).
- **Checksum**: slots `checksum/leaders.rs:41`–`52`; all twelve are
  `(unheld)` in `LEADERS_HELD_OUT`, count 2 each
  (`crates/rondata/src/diff/checksum_lines.rs:1823`–`1834`). Each port removes
  its entry; explored and pop need only the hold.

### 18.4 Reached by a plain game

A skirmish against the computer reaches `compute_score` for every live slot,
every part, both combat arms, and the `victory == 5`, `find_target` and
`diplomacy` readers. Bit 6 comes from the game-over latch and from pause;
`read_package` needs a recorded-game playback, `MPDropWin` multiplayer. The
Armageddon arm and `defeat_all` need the Armageddon count reached. The spells
need a spy or commando cast; whether the computer casts them is open.

### 18.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/score-*.md`); no diff backs it yet. Every
integer formula, both `get_kill_value`s and `get_score_value` were checked
against the listing instruction by instruction; none was wrong. Confidence
high.

Confirmed / corrected / open: shared 12/2/1; `score` 6/2/0; explored 3/0/0;
territory 4/0/1; units pair 9/0/1; buildings pair 6/0/0; economy 6/0/2; pop
2/0/0; upgrades 4/0/1; research 2/0/1; combat 10/1/1.

Corrections that changed a rule: `defeat_all` follows Armageddon only (no
end-of-game sweep); `find_target`'s clause is `4·score[c]/3 < score[mine]`,
not the reverse; the razed building is out of `n` when priced (settled). The
crate has `kill_value_by` and `cast_counterintel`. New: the gaia `is_ally`
read and the `rare.flags` write. No kept dump was consulted;
`LeaderData::log_data (006e5110)` prints `score` for a later value diff.

### 18.6 Not established

- **Gaia ally read**: whether `treaties[0]` and `leaders[8].diplos[K]` are
  ever both 2. If not, every gaia kill with a killer object adds
  `kill(t, 8)/5` on slot 8's counts. A `LEADERDATA` dump across a hunt, or a
  frame-snapshot read of the two words, settles it.
- **`rare.flags`**: whether the leaders checksum walks `BitMask::flags`
  (`+0x6da0`); read `checksum/leaders.rs`'s ranges.
- **Economy**: whether the crate's `income` (documented as the interface's
  net rate; `rate` is sixteenths) is the original's word and moment;
  `type_avail` for goods (`has_preq`, `type_eligible` unread).
- **Territory**: whether the crate's `territory` is current at
  `strategy_all`.
- **Units**: whether every removal goes through `Unit::close`; the crate's
  `queued_by_type` for units.
- **Upgrades, research**: `has_tech` past its head; `researching`'s vslot
  `+0x4c` "active" test.
- **Combat**: the `unit_flags & 0x10` redirect's meaning; whether the
  computer casts sabotage or counterintelligence.
- **Rest**: `get_team_score` past its head; the three non-latch bit-6
  setters' contexts; `LeaderData::walk_data (006d6750)` on load (off the
  frame path).
