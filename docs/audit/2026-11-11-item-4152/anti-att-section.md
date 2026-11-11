## 6. Anti-attrition (`anti_att`)

### 6.1 Leaves

- `anti_att`: +0x7f4, `float` (IEEE single). Array form `0xe3ab84 + i·0x6eec`.

### 6.2 The rule

**A pure recompute, no step and no decay.** It is a float, and it is held as
a `u32` of single-precision bits per player, emitted raw at +0x7f4. Every
run starts again from 256.0, so nothing carries over between runs.

**Writers.**

- `Leader::calc_anti_attrition (006cdcc0)` is the only computing writer. It
  has eight stores, and it is the only store of `0x43800000` to +0x7f4 in
  `.text` (no inlined copy). It has three direct callers and no indirect
  one: `Leader::calc_unit_stats (006cf970)`, `Leader::gain_tech (006dcb60)`
  and `Leader::init (006e3930)`.
- `Leader::init (006e3930)` zeroes it twice: by its head memset, and by a
  store of 0 at 6e4955. It then zeroes `take_att_disabled` and calls the
  writer (6e49c3). So init's recompute never takes the disabled exit, and
  it runs after the tribe is known.
- The bulk writers zero it or restore it as the header says.
  `Leaders::clear (006ed5d0)` runs between games only (`Game::close`,
  `Game::run_oos_recovery`). `Leader::close (006b7f80)` holds the same
  memset but is dead: nothing calls it. A load restores the saved bits raw
  through `LeaderData::walk_data (006d6750)`, only when `leader_flags & 1`.

**The recompute**, for leader `w`. The predicates read `leaders.list[w]`
and the stores go to the same record.

1. `x = 0x43800000` (256.0). It is stored **before** the next test.
2. If `take_att_disabled` (+0x7fc) is non-zero: `x = 0`, stop.
3. The Foraging tier is the **highest** held of three, not cumulative:
   - 3 if the leader holds Logistics;
   - else 2 if it holds Supply;
   - else 1 if it holds Forage;
   - else none.

   These are `has_preq(FORAGING_3..1)` on bonus types `0x300..0x2fe`. For
   these three types `has_preq` reduces to `has_tech(preq0)`, and the
   shipped `TECHBONUSES` rows 84, 83, 82 give `preq0` = Logistics, Supply,
   Forage. With a tier, `p = attrition_upgrade[tier − 1]`.
4. The sources apply in this order, each one held applying its own `p`:
   1. the tier;
   2. the Statue of Liberty: `has_wonder(0x219)`, with `liberty_attrition`
      (`Constants` +0x524);
   3. the Mongol bonus: `has_tribe_bonus(17)`, with `mongol_attrition`
      (+0x7f0);
   4. titanium: `has_rare(36)` over `rare | rare_conquest` (bit 30 of
      +0x6da4 or of +0x6dcc), with `titanium_attrition` (+0x95c).

   For each held source:
   - if `p ≥ 100` (a **signed** compare), then `x = 0` and stop;
   - otherwise
     `x = Single(x).mulss(Single::from_bits(0x42c80000)).divss(Single::from_i32(100 − p))`.
     Here `0x42c80000` is 100.0, read from `.rdata` at `0xb696ac`. All of
     it is scalar SSE single, round to nearest even: the executable has no
     `ldmxcsr`, and its one `_controlfp_s` call sets only the x87
     precision control.

The order shows in the bits, not the rational: Mongol before Forage 1
gives `0x442aaaab`, the writer's order `0x442aaaac`.

**What the predicates read:**

- `has_wonder`: an entry of this leader's wonder list that is live (+0xc
  & 1), of the type, and (any type but `0x216`) whose object's city short
  +0x72 is ≥ 0; or a CtW `conquest_wonders` bit at Civic ≥
  `ctw_wonder_min_civics`.
- `has_tribe_bonus`, first match: 0 under `info.flags & 4`, or no starting
  town and `city_num == 0`, or `tribe < 0`; 1 on a CtW
  `conquest_racial_powers` bit; 0 if `leader_flags2 & 0x40`; else the
  tribe's bonus index (`tribe·0x5f0 + 0x54`) is 17.

**When it runs.**

- At `Leader::init`.
- At the head of `calc_unit_stats`, after `calc_attrition`. It is reached
  by the per-frame fold in `Leaders::process_all (006ed2a0)` when
  `leader_flags & 0x4000000`, and the fold clears that bit. Seven sites
  set it:
  - `gain_tech`, twice: its tail and its unit-type arm;
  - `lose_tech` (site 6d2afd);
  - `gather` (site 6ce3d0);
  - `calc_gather` (site 6cf5ea);
  - `Build::activate`, **for a wonder only** (types `0x20e..0x21e`; every
    other building jumps past the set);
  - `Build::close`.
- Eagerly from `gain_tech` (6dd3cf), **only** in one arm. The gained type
  must answer `TypeData::is_epoch_type`, its techtype `+0x14` must be 1,
  and the decrypted Civic level must equal `ctw_wonder_min_civics`, which
  ships as 4. That arm also runs `calc_pop_cap`, `calc_attrition` and
  `Region::fix_borders`. Its value equals the next fold's, so it moves the
  leaf early only when a source arrived after that frame's fold.
- From `set_age (006d25a0)` / `set_epoch (006d26f0)`, and from the
  `ScenarioEditor::*` / `ScenarioFuncSet::*` functions, through direct
  `calc_unit_stats` calls.

**Timing.** Research finishes in `Objects::process_all`, after
`Leaders::process_all`, so a tech gained on frame F shows from F+1.

**Readers that change a decision.**

`UnitData::get_attrition (00608fd0)` reads the unit's **owner's** leaf. Its
non-militia arm is:

```
trunc(bits_as_f32 · k / 256)
k = 256, or 512 for siege   (25600 / (100 − siege_attrition))
```

The two multiplies are exact (k and 2^-8 are powers of two). Militia
bypass it.

The idle immunity that follows is `has_preq(0x2fe)`, which means "holds
Forage". It holds at tiers 2 and 3 as well, not at tier exactly 1.

`LeaderData::get_anti_attrition (006e0b70)` has no caller. `log_data`
prints the leaf.

**Float class (`docs/DECISIONS.md` entry 120 §3)** is class 1 or class 2,
never class 3:

- **Class 1** on a frame where no walked leader holds Forage (or above),
  the Statue, the Mongol bonus or titanium. The value is the constant
  `0x43800000`, from the store at 6cdcce.
- **Class 2** in general. Every input is an integer the simulation holds:
  four predicates and four `Tuning` percentages. The chain needs only
  `Single::from_bits`, `from_i32`, `mulss` and `divss`, all already in
  `num::single::Single` and correctly rounded. Nothing joins
  `no_float.rs`'s allowances.

The chain is the port (it serves a modded `rules.xml`); the table is its
test.

**The nine bit patterns over the shipped data** (`attrition_upgrade` 25,
50, 75; Liberty 100; Mongol 50; titanium 50). M and T are the Mongol bonus
and titanium, and "one" means either of the two. Read the cells across, in
the order "neither / one / both".

| tier | M, T: neither / one / both | reader's integer, k = 256 / 512 |
|---|---|---|
| none | `0x43800000` / `0x44000000` / `0x44800000` | 256/512, 512/1024, 1024/2048 |
| 1 (Forage) | `0x43aaaaab` / **`0x442aaaac`** / **`0x44aaaaac`** | 341/682, 682/1365, 1365/2730 |
| 2 (Supply) | `0x44000000` / `0x44800000` / `0x45000000` | exact |
| 3 (Logistics) | `0x44800000` / `0x45000000` / `0x45800000` | exact |
| Liberty held, or disabled | `0x00000000` | 0 |

On the two bold cells the bits are one ulp above the nearest single to the
exact rational, which is `0x442aaaab` and `0x44aaaaab`. In all 32 states
and for both values of k, the reader's integer equals the floor of the
exact rational. So the reader may stay on the rational.

### 6.3 In the crate

- **The rule exists unwired** as an exact rational:
  `territory::attrition::resistance` with `ResistanceMods`
  (`crates/sim/src/territory/attrition.rs:102, 132`), called only from its
  tests; `PlayerState.resistance` defaults to `Resistance::BASE` (`:268`).
- **The home is `Sim::calc_unit_stats`** (`crates/sim/src/economy/rares.rs:679`).
  It is run by the fold on `roster.unit_stats_dirty`
  (`core/frame.rs:186-189`). Its module doc (`rares.rs:47-48`) names
  `calc_attrition` / `calc_anti_attrition` as calls it does not make.
  Place the recompute after `calc_attrition`, and make sure a wonder's
  activate or close raises the dirty flag.
- **The trap: do not recompute in `Sim::apply_gained`** (`tech/gain.rs:601/608`),
  except the Civic-4 epoch arm above. An eager call there is one frame
  early on every frame a tech finishes. The same holds for the crate's
  existing eager `calc_attrition` at `gain.rs:608`, which is §7's to
  settle.
- **Init.** The `Leader::init` analogue recomputes after the tribe and the
  starting techs are set.
- **Stale docs:** `attrition.rs:76-79` ("the product … is never an
  integer": 256 is); `attrition.rs:100-101` (order unobservable holds for
  the rational, not the bits); `calc_attrition`'s "holds no wonder a player
  owns" (`PlayerTech::has_wonder`, `tech.rs:814`, and `Sim::wonders_held`,
  `economy/holdings.rs:572`, exist and `player_borders` reads them).
- **`TechTree::has_tribe_bonus`** (`crates/sim/src/tech.rs:1102`) holds the
  first arms and the index compare. It has no `leader_flags2 & 0x40` arm.
- **`PlayerState.foraging_1`** (`attrition.rs:261`) must come from "holds
  Forage", not from the tier. The tier itself is
  `Sim::bonus_level` (`economy/rares.rs:84`) over `bonus_at(82..84)`, the
  same shape as `ATTRITION1..4` (`crates/rondata/src/load.rs:1932`).
- **The checksum.** The leaf is `("anti_att", 0x7f4, 4, 1)` at
  `crates/sim/src/checksum/leaders.rs:151`, with no value arm. `Leaf::bytes`
  emits 0 for it (`:401-409`), which is wrong for every walked leader not
  under Liberty or the disabled flag. A port gives it a value arm and
  removes `("anti_att (unheld)", 2)` from `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1756`).

### 6.4 Reached by a plain game

**Yes.** In a skirmish against the computer, every leader is computed at
init:

- 256.0;
- or 512.0 for the Mongols, unless "No Nation Powers" is set or the start
  is nomadic. A skirmish with starting towns passes the nomad arm while
  `city_num` is 0.

After init, the plain-game sources are Forage / Supply / Logistics, the
Statue, titanium, and the Civic-4 epoch arm.

Out of reach: `take_att_disabled` (triggers only:
`ScenarioFuncSet::enable_take_attrition (00a02880)` /
`disable_take_attrition (00a028c0)`, no recompute); `rare_conquest`,
`conquest_wonders`, `conquest_racial_powers` (CtW); the editor and
`ScenarioFuncSet` callers; `Leaders::clear` (between games); the load walk
(save track).

### 6.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/anti-att-*.md`). **No diff backs the
class 2 rows yet.** The class 1 value `0x43800000` was read off run 1562
frame 91 and run1100 frame 5408 (entry 120 §3.4). That reading is a
witness, not a pinned diff.

Verdicts: 33 confirmed, 6 corrected, 3 open (§6.6 lists four). The
corrections that changed a rule: the tier is "holds Logistics / Supply /
Forage", not a tech named FORAGING_n; `Build::activate` dirties for
wonders only; no eager recompute on gain but the Civic-4 arm; nine
patterns, not eleven; class 1 or 2, not 3; the crate does hold wonders.

Confidence:

- **High** on the writer set, exits, order and arithmetic (listing read
  whole, both constants from `.rdata`, bits replayed as f64 and as exact
  rationals rounded per op, agreeing on all 32 states); on the bonus-type
  reading (decompile plus shipped rows); on the timing (complete callers).
- **Medium** on the Mongol init value: the caller's store of `tribe` was
  not traced.

### 6.6 Not established

- **Whether a nomad Mongol's first city raises `0x4000000`.** If it does
  not, `has_tribe_bonus` flips with no recompute. To settle it, grep the
  city founder (`City::init`, `Build::activate` for a city centre) for a
  leader-flags write.
- **Whether `PlayerTech.wonders` carries `has_wonder`'s `+0x72 ≥ 0`
  (captured-city) predicate** for a captured Statue. To settle it, read
  the crate's wonder bookkeeping on a city capture.
- **Whether a lobby's later starting age reaches `set_age`** (and so
  `calc_unit_stats`): read the lobby's age setup.
- **What sets `leader_flags2 & 0x40`**: grep the stores to that flag.
- **The Mongol init value of 512.0** (medium). Check it against a capture
  whose walked leader plays the Mongols.

None changes a plain game's first frames. A capture whose walked leader
researches Forage then Supply or Logistics, or plays the Mongols, would
back every row of the pattern table at once.
