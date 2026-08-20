# Adjudication: attrition and territory — Reading A vs Reading B

A = `docs/ATTRITION.md` + `crates/sim/src/{attrition,territory,world}.rs` (+ `lib.rs` where it applies the result).
B = `~/.claude/jobs/3c382923/tmp/rederive-attrition.md`.

Every verdict below was settled by re-reading the decompilation under
`~/ghidra-projects/decomp/` (`funcs/<Class>/<method>@<addr>.c`, `types.txt`).
Only behaviour-changing content is listed; wording, scope and emphasis are
ignored. Struct-offset names come from `types.txt` (PDB-derived).

---

## Part 1 — Disagreements

### D1. Which unit kinds are exempt (eligibility check 8)

- **A:** "Workers and merchants are subject. Heroes, supply units, spies and
  'special' units are exempt, as are caravans." `UnitKind::exempt_kind` doc
  repeats it (attrition.rs:152-155).
- **B:** workers (unless gathering non-flat), merchants and heroes are *not*
  exempt; supply units pass this check (exempt only later, and only outside
  the peace/assassin flag); otherwise exempt if type `attack == 0`, or
  `is_special`, or Spy (`is(0x3a)`), or caravan.
- **Evidence:** `Unit/process_attrition@005e11a0.c`. The block is
  `if (!is_worker) { if (!is_merchant) { h = is_hero; if (h == 0) { s = is_supply; if (s == 0) { if (type+0x1e8 == 0) return 0; if (is_special) return 0; if (is(0x3a)) return 0; if (is_caravan) return 0; } } } } else { if (is_gathering && order->non_flat_gather) return 0; }`.
  A hero (`uVar13 != 0`) or a supply unit skips the inner block and falls
  through — it is **not** exempt. `+0x1e8` is the type's attack value
  (`ObjectData/attack@006469f0.c:12` reads it as the base attack).
- **Verdict: B right.** Three corrections to A: heroes are subject; supply
  units are not exempt here (see D2); a type with zero attack is exempt
  (the gate A omits entirely). `UnitKind::exempt_kind` is caller-supplied so
  the function is not wrong, but the documented rule the caller must follow
  is.

### D2. Supply units under peace / assassin attrition

- **A:** supply units are exempt outright (check 8), so they never get the
  peace period. `attrition.rs:447` returns `Exempt(UnitKind)` before the
  peace branch.
- **B:** supply units are exempt only in the land tail, and only when
  `unit_masks & 0x400000` is clear — "the supply-unit exemption is lifted"
  under peace.
- **Evidence:** `process_attrition@005e11a0.c`, label `LAB_005e18f1`:
  `if (type+0x218 == 0) { if ((masks & 0x400000) == 0) { if (is_supply) return 0; } m = get_attrition(owner); ... }`.
  The peace branch sets `masks |= 0x400080` before reaching this, so a
  supply unit on a peaceful border gets `peace_attrition` **and** the
  computed period (min of the two). At war the flag is clear, so the supply
  unit returns 0 with `attrition` still 0 from entry — exempt.
- **Verdict: B right.** A's implementation over-exempts supply units in the
  peace and assassin cases.

### D3. Does the assassin path defeat supply?

- **A:** "the peace and assassin paths set a flag that makes the supply check
  give up immediately" (`ignores_supply = assigned.is_some()`).
- **B:** only names `0x400000` as the "peace-violation" flag (§2.5 describes
  the assassin path without it; §5 item 12 says "peace-violation attrition").
- **Evidence:** `process_attrition@005e11a0.c`, enemy branch, assassin case:
  `*(uint *)&this->field_0x68 |= 0x400080; sVar1 = assassin_attrition;` —
  the same mask the peace branch sets. `process_supply@005e0560.c` returns 0
  immediately when `masks & 0x400000`.
- **Verdict: A right; B incomplete.** Supply shelters neither peace nor
  assassin bleeds, and D2's lifted supply-unit exemption applies to both.

### D4. What the damage number is

- **A:** "Damage 16 / 8 / 6 / 4 by squad size"; `lib.rs:789-790` does
  `unit.health -= attrition::damage(squad_size)` against a `health` of 100,
  and a test comment calls one 48-frame tick "a sixth of a full-health
  squaddie's life".
- **B:** those are **sixteenths of a hit point** accumulated in
  `ObjectData::damage_frac`; a lone figure loses 1 HP per tick; in a squad
  every member object takes `16/n` sixteenths itself.
- **Evidence:** `Unit/suffer_attrition@005e1a10.c` calls vslot 0x16c with
  `(whole=0, frac=16/n or 6, ...)`. `Object/take_damage@00652020.c` (label
  `LAB_006522f2`): `acc = (char)(damage_frac + param_2); whole = param_1 + acc/16; damage_frac = acc mod 16; damage += whole` —
  `types.txt` names `+0x24 damage`, `+0x3b damage_frac`. Each squad member
  is its own `UnitData` in the owner's unit list
  (`UnitData/curr_uber_size@0060a760.c` walks `o_up`/`o_down`), so each runs
  its own `Unit::process` and its own `suffer_attrition`.
- **Verdict: B right.** `damage()`'s values are correct but are sixteenths;
  A's `lib.rs` applies them as whole points, sixteen times too strong per
  figure. Per-squad total per tick is 1 HP (18/16 for size 3), spread as
  one whole point every `n` ticks on each figure.

### D5. The `+0x308` path in `suffer_attrition`

- **A:** "a type flagged at +0x308 takes a different path … that first
  argument is a damage *kind* rather than an amount, and what the kind does
  is unread."
- **B:** `+0x308` is `uber_size`; `uber_size == 1` → `take_damage(1, 0)` =
  one whole point.
- **Evidence:** `types.txt` UnitTypeData `+0x308 int uber_size`;
  `take_damage` as in D4: param_1 is whole points, param_2 sixteenths.
- **Verdict: B right.** No damage "kind"; a type whose uber size is 1 is
  dealt 1 whole HP directly, which is the same as 16 sixteenths. A's open
  question closes.

### D6. Eligibility check 5 — "the owner is the victim's team"

- **A:** check 5 is a team test; `PlayerState::team` with
  `owner_id == victim.team` (attrition.rs:438), default `team: 0`.
- **B:** `owner == leaders[who].who`.
- **Evidence:** `types.txt` LeaderData `+0x8 int who`. `Leader/init@006e3930.c:134`
  `field_0x8 = param_1` (the slot index); `LeaderData/is_enemy@006ebaa0.c`
  uses `this->who` as the leader's own index (`param_1 != this->who`);
  `calc_anti_attrition@006cdcc0.c` indexes `leaders.list + this->field_0x8`.
  The mutual-alliance test that follows also uses it:
  `diplos[who][owner] == 2 && leaders[owner].diplos[leaders[who].who] == 2`.
- **Verdict: B right.** There is no team concept here; the check is a
  redundant `owner == who`. A's `team` field is an invention, and because
  `Sim::new`/`add_player` (`lib.rs:298, 322`) leave it at the default 0,
  any unit of player ≥1 standing in player 0's territory is wrongly
  exempted as `SameTeam` unless the caller sets `team` by hand
  (`harness_tests.rs:44-49` does exactly that to compensate). Latent bug.

### D7. What `UnitTypeData + 0x218` is

- **A:** a three-way "attrition mode": 0 normal, 1 outright immune, 2
  "special" (halved special periods, no computed period).
- **B:** the type's domain: 0 land, 1 sea, 2 air.
- **Evidence:** `ObjectData/num_aircraft_here@00645330.c:24` counts units
  with `type+0x218 == 2`; `ObjectData/in_a_ship@006440c0.c:23` tests
  `== 1`; `Unit/add_to_army@005f7740.c:22` tests `== 1`. The PDB leaves the
  field unnamed (`_padding_`), so this is inference, but it is consistent
  everywhere.
- **Verdict: B right.** The code paths A implements are correct (`Mode::Immune`
  ≡ sea, `Mode::Special` ≡ air), so behaviour is unchanged if callers map
  correctly — but the documented meaning is wrong and matters for callers:
  ships never take attrition; aircraft take only the halved peace/assassin
  period and never the territorial one; only land units get the computed
  period.

### D8. When territory is recomputed

- **A:** "recomputed wholesale rather than incrementally.
  `World::compute_all_territory` drives it"; `lib.rs:680` recomputes every
  region every time.
- **B:** `GameDaemon::check_borders` runs every frame with a shared budget
  of 256 cells per frame, region by region, resuming at `Region::borders`;
  `compute_all_territory` is only game setup.
- **Evidence:** `GameDaemon/check_borders@00732060.c`: `this->borders = 0`,
  then for each of the 64 land regions with `size != 0 && borders < size`
  call `compute_reg_territory(world, r, 0)`. `World/compute_reg_territory@006b0bb0.c`:
  `if (0xff < game_daemon->borders) return 1;` at the top and per cell
  `if (borders < 0 || (borders++, borders < 0x100))`, resume index
  `Region +0x2c borders` (`types.txt`). `compute_all_territory@006b5700.c:57`
  sets `borders = -1` (unlimited) and calls with `param_2 = 1`.
- **Verdict: B right.** Steady-state ownership is the same, but in the
  original a cell can carry stale ownership for up to `land cells / 256`
  frames after an invalidation, and `process_attrition` reads whatever is
  there. A's wholesale recompute is a deliberate simplification, not what
  the original does; it should be stated as such.

### D9. The grace window (check 12) and the constants behind it

- **A:** check 12 "inside the `ally_to_war_grace` window after an ally became
  an enemy"; implementation tests `in_war_grace` inside the not-at-war
  branch (attrition.rs:465-468).
- **B:** the grace test is in the *non-enemy* branch; and
  `ALLY_TO_WAR_DELAY`/`GRACE` are absent from `rules.xml`, so both load as
  −1 and the "recently broke alliance" / grace logic never fires.
- **Evidence:** `process_attrition@005e11a0.c`: the `broke_alliance`/grace
  computation and `if (bVar2) return 0;` are inside
  `if (is_enemy == 0 || (CtW && !war_allowed))`, i.e. not at war.
  `Constants/init@00569a90.c:2397-2400` loads the two with `get_item`
  between `paradrop_range` and `timer_refresh_ratio`;
  `Constants/get_item@0057fa60.c` calls `get_attrib_num(..., -1)`, and
  `XMLElement/get_attrib_num@00a27580.c` returns that default when the
  attribute is empty. The user's `game/data/rules.xml:726-727` has
  `PARADROP_RANGE` immediately followed by `TIMER_REFRESH_RATIO`; no
  `ALLY_TO_WAR_*` element exists anywhere in the file. With −1,
  `ally_to_war_delay <= frame - stamp` is always true for any set stamp, so
  `bVar4` is always false and `bVar2` (within grace) is never true.
- **Verdict: implementation agrees with the code (A's placement is right;
  A's prose "after an ally became an enemy" is misleading — it is "alliance
  broken, not yet at war"). B extends, verified:** in shipped RoN:EE the
  grace window is dead code; `in_war_grace` can only ever be true for a
  mod that adds the constants.

### D10. Runner-up bookkeeping when a fort becomes the first winner

- **A (`territory.rs`):** on a new best, `if best.is_some() { second = best }`
  for cities and forts alike.
- **B:** "second = best, owner2 = owner" unconditionally, for both.
- **Evidence:** `compute_reg_territory@006b0bb0.c`. City path:
  `if (local_14 != 0xffffffff) { local_48 = local_54; local_24 = local_14; }`
  (conditional). Fort path: `local_48 = local_54; local_54 = iVar14; local_24 = local_14;`
  (unconditional). So when a fort is the first in-cap claim after an
  over-cap city was recorded as runner-up, the original overwrites the
  runner-up with the `999999999 / -1` sentinel.
- **Verdict: both wrong**, in opposite halves. Affects `who2` only (never
  `who`), so no ownership or attrition consequence; worth a one-line fix or
  a note.

### D11. What `XOR 0x63187` protects

- **A:** "a city's stored age".
- **B:** the leader's civic level, `LeaderDataEncrypt::epoch[1]`.
- **Evidence:** `compute_reg_territory@006b0bb0.c`:
  `uVar19 = *(uint *)(data_encrypted + 0xec) ^ 0x63187; civic_upgrade_terr[uVar19]`;
  `types.txt` LeaderDataEncrypt `+0xe8 int[4] epoch` → `+0xec` is
  `epoch[1]`. `+0xdc ages ^ 0x62766` is the age used in `get_attrition`.
- **Verdict: B right.** Doc-only; the implementation takes civic level as an
  input.

### D12. The gathering-worker exemption

- **A:** "a worker actively gathering at a site flagged exempt" /
  `gathering_at_exempt_site` "a site the scenario flagged exempt".
- **B:** `GatherOrder::non_flat_gather` (+0x25), set in `add_gather_order`
  when the target build type's vslot 0x94 returns 0 and the build is not
  University (0x1a4).
- **Evidence:** `Unit/add_gather_order@0061a5c0.c:70-84`;
  `types.txt:19172 +0x25 uchar non_flat_gather`.
- **Verdict: B right.** Not a scenario flag — a property of the building
  type being gathered from. Caller-supplied in A, so doc-only, but the
  field name and doc mislead.

---

## Part 2 — B extends or closes (verified by me unless marked)

- **Decoy path = the "second entry into `suffer_attrition`"** (A open
  question). `Unit/process@00610bc0.c:73-74,258-285`: the `else` of
  `(unit_flags2 & 2) == 0`; counter at +0x96 against
  `decoy_time * (general_upgrade + 2) / 2`; while on map and the half-cell
  visibility byte has bits outside the owner's ally mask,
  `suffer_attrition(this, frame % 7 == 0)` every frame. Closed.
- **`+0x308` is `uber_size`** (D5). Closed.
- **`attrition_stamp2`** is the once-per-game "your units are suffering
  attrition" message throttle (`suffer_attrition@005e1a10.c`, `field_0x1f8 == 0`
  guard, then set to frame). Presentation only. `stamp3` unseen. Half closed.
- **Second limit triple** = `colonized_territory_limit{,_civic,_city}` at
  WorldData +0x44/48/4c (`types.txt:18239+`); selector is `Region::flags & 4`
  (→ player limits, else colonized) — `compute_reg_territory@006b0bb0.c`
  after the budget check. Closes A's open question on *what*; *which
  regions carry flag 4* remains open.
- **Incremental recompute with 256-cell budget** (D8), and an invalidation
  list (`Region::fix_borders` / `Regions::fix_all_borders` call sites) that
  I did not re-verify call by call.
- **Tikal temple bonus also ×(100+`ctw_missionaries_bonus`)/100 in CtW**
  with leader byte +0x6916 — `compute_reg_territory@006b0bb0.c`. Verified.
- **`give_att_disabled` also zeroes `att` inside `calc_attrition`**
  (`calc_attrition@006cdea0.c`: whole body under `if (field_0x7f8 == 0)`),
  in addition to the `process_attrition` check. Verified; same outcome.
- **Tribe bonus 4 counts one attrition step free**: `calc_attrition` treats
  the step whose `TypeIndex` equals Ghidra's `BUY_SELL` as held when
  `has_tribe_bonus(4)`. The code is there; which step it is stays unresolved
  (the `TypeIndex` enum is not in `types.txt`). Neither reading settles it.
- **Foraging: only the highest tier counts** (`calc_anti_attrition`:
  FORAGING_3 → idx 2, else FORAGING_2 → 1, else FORAGING_1 → 0). A's
  ×4/3, ×2, ×4 table already assumes this; now explicit.
- **`ALLY_TO_WAR_DELAY/GRACE` absent → −1 → grace dead** (D9).
- **City `bordering` bits and `city_flags |= 0x1000`** when winner and
  runner-up are distinct real players not at war both ways, plus rare-goods
  reveal to the owner — present in `compute_reg_territory`, not attrition
  relevant.
- **`is_enemy` is "either side at war"** (`diplos[a][b]==0 || diplos[b][a]==0`),
  mutual alliance is both `== 2`. Verified (`is_enemy@006ebaa0.c`,
  `is_ally@006edb50.c`).
- **`(frame+o) % period` can fire the frame after assignment; garrisoned
  units neither refresh nor tick** — consistent with `Unit/process@00610bc0.c:435-535`
  (all under `inside_up < 0`). A's `on_map` already does this.

---

## Part 3 — Doubly confirmed arithmetic (A and B agree; re-checked in the decompilation)

- Coordinates: position unit / tile (`div_3[c>>6]`) / cell (`div_3[c>>8]`);
  cell centre tile `4w+2`; `div_3_table = floor(i/3)` (A has the
  `init_coord_lookup_array` evidence; B assumed it).
- Strength: `ATTRITION_IMPROVED[n-1]`, then Colosseum → Russian → CtW →
  Kremlin, each `(100+pct)*x/100` with floor 1 applied only when that step
  produced 0; max 72 with shipped data.
- Resistance: `256` × `100/(100-pct)` for highest Foraging tier, Liberty,
  Mongol, titanium; any `pct > 99` → 0 (immune); float in the original,
  exact rational in A; every reachable value agrees.
- `get_attrition`: base 256, siege `25600/(100-SIEGE)` = 512; militia
  `base*100/(MILITIA+100)` = 64 ignoring `anti_att`; others
  `trunc(base*anti_att/256)`; Foraging-1 holder idle → 0; types
  0x3d/0x3e/400 halve; `+0x218 == 2` halves (unreachable from the tick);
  age-ahead ceiling `((AGED_UP*d+100)*att+99)/100` for `d >= 0`; return
  `val/att`; zero sentinel; strength > 256 would divide to 0.
- Period: `max(1, (48*m + bias) >> 8)`, min'd with an already-assigned
  peace/assassin period; `PEACE_ATTRITION = ASSASSIN_ATTRITION = 8`, halved
  for `+0x218 == 2`; computed period only for `+0x218 == 0`.
- Eligibility order 1–7, 9–11 as A lists them (free radius; unowned with
  `neutral_attrition` — the OOB `leaders[-1]` quirk; own cell; owner flags
  `&1`, `&2`; redundant `who`; mutual 2/2; take/give disabled; `+0x218==1`;
  `neutral_attrition != 0`; CtW bonus 9 outside team style 2). Assassin =
  team style 2 and `get_target() != owner`.
- Cadence: `(frame+o)&15 == 0` → cloak; `&31 == 0` → clear supplied flag,
  `process_attrition` (which zeroes `attrition` and `masks & 0x400080` on
  entry); every frame `attrition != 0 && (frame+o) % attrition == 0` →
  `process_supply() ? supplied flag : suffer_attrition`. Phase-locked, stale
  up to 31 frames in both directions.
- `process_supply`: `!(masks & 0x400000)`, not supply unit, not militia,
  `find_supply >= 0` or one of three hero types (0x16b/0x176/0x16e) via
  `has_general`.
- Damage table 16/8/6/4 by `curr_uber_size` (units differ — D4).
- Territory: per-leader inputs (temple 1–4 → `temple_upgrade_terr`, Tikal
  ×1.5; fort 1–4, Colosseum add, Roman add + `(roman+1)/2` range steps;
  `civic_upgrade_terr[civic]`; gems/Colosseum/Eiffel flat bonuses halved for
  Russians with `(b+1)/2` (or 1 for Russians) limit steps, gems one step;
  Russian `per_age*civic + borders`, `(borders!=0)+civic` steps; handicap
  `(h+15)/25` with no step); `cityRange = base + civic*limitCivic + steps`;
  `fortRange = fortLvlIdx*limitCity + cityRange`; city gate
  `d <= (level-1)*limitCity + templeLvl*limitCity(if temple) + cityRange`;
  `vector_dist` = `hi + lo²/(2hi)` (`lo<60000` guard); contraction
  `<13: *2/3; <9: *2/3; <5: /2` in that order on the running value after
  the gate; cost `den*d*256 / ((bonuses)*mult + num)`, cap `base<<8`;
  strict `<` tie-break with player order `(k + wx) & 7`, cities before
  forts; `-2` when `CityData::race != p`; sea regions `who=who2=0xff`;
  `reg_terr` per region; bonuses from `race`, limit from list owner.

---

## Part 4 — What must change

**`docs/ATTRITION.md`**
1. Check 8: heroes and supply units are **not** exempt here; add "type attack
   is zero" to the exempt list; the worker case is `GatherOrder::non_flat_gather`.
2. Supply units: exempt from the *computed* period only when the
   peace/assassin flag is clear; under peace/assassin they bleed.
3. `+0x218` is domain (land/sea/air), not an "attrition mode"; rephrase the
   three cases accordingly (ships immune; aircraft half peace/assassin only).
4. Check 5 is `owner == leaders[who].who` — redundant, not a team test.
5. Damage section: 16/8/6/4 are sixteenths of a HP per figure, accumulated
   in `damage_frac`; `+0x308` is `uber_size` (== 1 → one whole HP). Strike
   the "damage kind" open question.
6. Territory: the original recomputes incrementally at 256 cells/frame via
   `GameDaemon::check_borders`; `compute_all_territory` is setup only. Name
   the colonized triple and the `Region::flags & 4` selector.
7. Grace window: lives in the not-at-war branch ("alliance broken, not yet
   at war"); the constants are absent from shipped `rules.xml`, so it never
   fires in RoN:EE.
8. `0x63187` masks the leader's civic level (`epoch[1]`), not a city age.
9. Close the "second `suffer_attrition` entry" question: decoys.
10. Note the fort/city runner-up asymmetry (who2 only).

**`crates/sim/src/attrition.rs`**
- Remove `PlayerState::team` (or default it to the player's own index) and
  the `SameTeam` check, or make it `owner_id == victim_id` again; as shipped
  it exempts everyone from player 0's borders by default.
- Supply units: stop exempting them via `exempt_kind` before the peace
  branch; exempt only in the computed-period path when `!ignores_supply`.
  Add `attack == 0` to the documented `exempt_kind` rule; drop heroes/supply
  from it.
- Rename `Mode::{Normal,Immune,Special}` → domain `{Land,Sea,Air}` (or fix
  the docs).
- `damage()` doc: sixteenths per figure. `Resistance`/`foraging_1` doc: any
  tier ≥1 holds FORAGING_1.

**`crates/sim/src/lib.rs`** (downstream of attrition.rs)
- Apply `damage()` as sixteenths against a per-unit fractional accumulator
  (or keep `health` in sixteenths explicitly); today a lone figure loses 16
  HP per tick where the original loses 1.

**`crates/sim/src/territory.rs`**
- Optional: match the fort-path runner-up overwrite, or document the
  divergence (who2 only).
- Document that wholesale recompute is a simplification of the original's
  budgeted incremental pass.

**`crates/sim/src/world.rs`** — no change required; `vector_dist` and the
unit conversions are doubly confirmed.
