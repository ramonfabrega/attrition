# The nations and the AI's difficulty tiers, audited whole (item 2976)

DECISIONS 77 (iii): every nation's powers and every difficulty tier's
constants, each row marked **modelled at a module or not**, and held by a
guard that reads the table (`sim::nations::tests::the_audit_table_*`). The
product is the census, and the list of what it found unmodelled, ranked by
the held-out lobbies that would reach it (§7). No run was taken: every
claim below is a reading of the executable's export or of this crate, and
**no row is a diff**. A row says where the original reads a constant and
where this crate does, not that the arithmetic agrees.

**How it was established.** (1) The roster is `rules.xml`'s `TRIBES` block,
24 nations; the index is the `n` of `has_tribe_bonus(n)`. (2) Every
function in the decompile export that calls `has_tribe_bonus` was listed
with the indices it passes (`tools/nations_census.py` parses the
argument by balanced parentheses; a call whose argument is a variable is
marked `+var`) — 120 functions. (3) Every constant `rules.xml` names with
a nation's prefix (202) was looked up in `tuning.rs` for a carried field
and in the rest of `crates/sim/src` for a reader, and in the export for the
functions that read it. (4) The held-out nations' arms (Egyptians,
Russians, Chinese) and `train_time`'s national block were read arm by arm
against the sim (§3). (5) The difficulty reads were listed from every
function the export holds that calls `LeaderData::get_diff@006ec000` or
reads `info.difficulty` (§6). Regenerate the tables with
`python3 -I tools/nations_census.py <export> <rules.xml> --write`;
a hand judgement is an entry in that script's `HAND` or `OVERRIDES`.

**The vocabulary.** The last two cells of a guarded row are a status and a
witness, and the guard checks the pair:

| status | the witness is | the guard fails when |
|---|---|---|
| `modelled at F.rs` | a field name, or an address, the module holds | `F.rs` lacks it (comments do not count for a field) |
| `carried at F.rs` | the function's address, cited in `F.rs` | `F.rs` lacks it — **the function is carried; this audit did not read each nation arm of it** |
| `loaded at F.rs` | a `Tuning` field | `crates/rondata/src/F.rs` does not read it (the loader builds the rule the sim's `TechTree` runs) |
| `carried, unread` | a `Tuning` field | the field is not in `tuning.rs`, or any other module reads it |
| `not carried` / `not modelled` | the field or address | the sim now holds it — the row is stale and the candidate may have landed |
| `not sim` | a reason | never — a judgement about the class, not the code |

**Confidence.** The tables of §2, §4 and §5 are exact about *presence*: a
field is carried or it is not, an address is cited or it is not, and the
guard keeps that true. They say nothing of correctness, and `modelled` on
a constant means a module other than `tuning.rs` reads the field, not that
the reader matches the original's arm. The §3 rows are a reading of both
sides and carry the usual reading-only caveat; no capture holds a Russian,
Egyptian or Chinese **human** or a trace past the battery's first parting,
and the battery is never debugged against (`docs/AI.md` §125).

## 2. The roster

<!-- GENERATED:roster -->
| index | nation | rules key | `city::Nation` flag | census functions | lobbies that hold it |
|---:|---|---|---|---:|---|
| 0 | Aztecs | aztecs | `aztecs` | 7 | `GreatSaharaAztecTier2` (AI word, tier 2); `EastIndiesAztecTier1` (map 18, tier 1) |
| 1 | Maya | maya | `maya` | 2 | `GreatSaharaMayaTier4` (AI word, tier 4) |
| 2 | Inca | inca | `inca` | 3 | `EastIndiesIncaTier3` (AI word, tier 3); `GreatSaharaIncaTier3` (AI word, tier 3) |
| 3 | Bantu | bantu | `bantu` | 5 | `GreatSaharaBantuTier4` (map 7, tier 4) |
| 4 | Nubians | nubians | `nubians` | 37 | the human in every capture |
| 5 | Greeks | greeks | none (read off `has_tribe_bonus`) | 7 | `GreatSaharaGreekTier1` (AI word, tier 1) |
| 6 | Romans | romans | `romans` | 10 | `EastIndiesRomanTier2` (AI word, tier 2), `GreatLakesRomanTier2` (map 14, tier 2) |
| 7 | Egyptians | egyptians | `egyptians` | 8 | `GreatSaharaEgyptianTier4` (AI word, tier 4), `EastIndiesEgyptianTier3` (map 18, tier 3) |
| 8 | Turks | turks | `turks` | 7 | `GreatLakesTurkishTier4` (AI word, tier 4); `GreatSaharaTurkishTier3` (map 7, tier 3) |
| 9 | Spanish | spanish | none (read off `has_tribe_bonus`) | 9 | `GreatSaharaSpanishTier3` (AI word, tier 3), `EastIndiesSpanishTier3` (AI word, tier 3); `GreatLakesSpanishTier4` (AI word, tier 4) |
| 10 | French | french | `french` | 7 | `GreatSaharaFrenchTier4` (AI word, tier 4) |
| 11 | British | british | `british` | 6 | scored pairs |
| 12 | Germans | germans | `germans` | 8 | `EastIndiesGermanTier3` (AI word, tier 3); `GreatSaharaGermanTier3` (AI word, tier 3) |
| 13 | Russians | russians | `russians` | 10 | `GreatSaharaRussianTier4` (map 7, tier 4); `GreatLakesRussianTier2` (AI word, tier 2) |
| 14 | Chinese | chinese | `chinese` | 6 | `GreatLakesChineseTier1` (map 14, tier 1) |
| 15 | Japanese | japanese | none (read off `has_tribe_bonus`) | 4 | `GreatSaharaJapaneseTier1` (map 7, tier 1) |
| 16 | Koreans | koreans | `koreans` | 6 | `GreatSaharaKoreanTier3` (AI word, tier 3); scored pairs |
| 17 | Mongols | mongols | `mongols` | 3 | none |
| 18 | Iroquois | iroquois | none (read off `has_tribe_bonus`) | 8 | `GreatLakesIroquoisTier3` (AI word, tier 3); `GreatSaharaIroquoisTier3` (AI word, tier 3) |
| 19 | Lakota | lakota | `lakota` | 17 | `GreatLakesLakotaTier2` (AI word, tier 2); `GreatSaharaLakotaTier2` (map 7, tier 2) |
| 20 | Americans | americans | none (read off `has_tribe_bonus`) | 14 | `EastIndiesAmericanTier1` (AI word, tier 1); `GreatLakesAmericanTier4` (AI word, tier 4) |
| 21 | Indians | indians | `indians` | 4 | `EastIndiesIndianTier2` (AI word, tier 2); `GreatSaharaIndianTier3` (AI word, tier 3) |
| 22 | Dutch | dutch | `dutch` | 9 | `EastIndiesDutchTier4` (AI word, tier 4); `GreatSaharaDutchTier3` (AI word, tier 3); `GreatLakesDutchTier4` (map 14, tier 4) |
| 23 | Persians | persian | none (read off `has_tribe_bonus`) | 11 | `GreatLakesPersianTier2` (AI word, tier 2) |

## 3. The held-out nations, arm by arm

`GreatSaharaEgyptianTier4`, `GreatLakesRussianTier2` and
`GreatLakesChineseTier1` (`docs/AI.md` §125) hold the three nations no
scored word reaches. Each row is one `has_tribe_bonus` arm (or, for the
`train_time` block, one nation's arm of it) read in the export and looked
for in the sim. The block under the three nations is the rest of
`ObjectData::train_time@006508c0`'s national tail: `lib.rs`'s `train_tail`
says in its doc comment that only the British and Mongol arms are built,
"the nine other arms inert in every capture on disk".

<!-- GENERATED:hand -->
| nation | arm | original | status | witness |
|---|---|---|---|---|
| Egyptians | wonders cost 25% less | `TypeData::get_cost@00664090` | modelled at cost.rs | `egyptian_wonders` |
| Egyptians | scholars swap food and wealth (gate is 0 in the rules) | `TypeData::get_cost@00664090` | not modelled | `egyptian_scholars_food` |
| Egyptians | granary available early | `LeaderData::has_preq@006db810` | modelled at tech.rs | `egyptian_granary_early` |
| Egyptians | wonders one age early | `LeaderData::special_preq@006d7030` | modelled at tech.rs | `egyptian_wonders_early` |
| Egyptians | one more wonder per city (placement) | `BuildTypeData::blocked_location@006375b0` | modelled at location.rs | `n.egyptians` |
| Egyptians | the same limit enforced at construction | `Wall::do_construct@006434d0` | modelled at city.rs | `006434d0` |
| Egyptians | farm limit per city | `LeaderData::get_farm_limit@006db270` | modelled at city.rs | `egyptian_farms_per_city_base` |
| Egyptians | food commerce bonus | `Leader::calc_resource_caps@006ce900` | modelled at economy.rs | `egyptian_food_commerce` |
| Egyptians | granary upgrades applied on tech gain | `Leader::gain_tech@006dcb60` | loaded at load.rs | `egyptian_granary_upgrades` |
| Russians | attrition doubled | `Leader::calc_attrition@006cdea0` | modelled at attrition.rs | `russian_attrition` |
| Russians | oil +20% in the rate pass | `LeaderData::calc_resource_bonuses@006db030` | modelled at economy.rs | `russian_oil` |
| Russians | spy cost 50% less | `TypeData::get_cost@00664090` | modelled at cost.rs | `russian_spy_cost` |
| Russians | borders grow per age | `World::compute_reg_territory@006b0bb0` | modelled at territory.rs | `russian_borders_per_age` |
| Russians | gems' territory bonus halved | `World::compute_reg_territory@006b0bb0` | modelled at territory.rs | `n.russian` |
| Russians | plunder steals from the plundered | `Build::plunder@00623660` | modelled at city.rs | `russian_plunder_steal` |
| Russians | a free civic at start | `Leader::init@006e3930` | modelled at tech.rs | `russian_free_civic` |
| Russians | oil well counts as held (has_preq) | `LeaderData::has_preq@006db810` | modelled at tech.rs | `has_tribe_bonus(setup, p, 13)` |
| Russians | attrition upgrades applied on tech gain | `Leader::gain_tech@006dcb60` | loaded at load.rs | `russian_attrition_upgrades` |
| Russians | market prices pinned at 100 after the fifth age (gate is 0) | `LeaderData::calc_market_prices@006dc2a0` | carried, unread | `russian_communism` |
| Chinese | science techs 20% cheaper | `TypeData::get_cost@00664090` | modelled at lib.rs | `chinese_science_discount` |
| Chinese | citizens train twice as fast | `ObjectData::train_time@006508c0` | modelled at lib.rs | `chinese_citizen_creation` |
| Chinese | citizens train instantly (the rules' CHINESE_CITIZEN_INSTANT is 1, the arm returns 1) | `ObjectData::train_time@006508c0` | modelled at lib.rs | `chinese_citizen_instant` |
| Chinese | a large-city building on a new city | `Build::activate@00623e20` | modelled at city.rs | `chinese_large_cities` |
| Chinese | a large-city building on assimilation | `City::assimilate@00738e90` | modelled at city.rs | `chinese_large_cities` |
| Chinese | herbal lore applied on tech gain | `Leader::gain_tech@006dcb60` | loaded at load.rs | `chinese_herbal_lore` |
| Chinese | knowledge upgrades applied on tech gain (0 in the rules) | `Leader::gain_tech@006dcb60` | carried, unread | `chinese_knowledge_upgrades` |
| Greeks | an age or an epoch researches faster (train_time's research tail) | `ObjectData::train_time@006508c0` | modelled at lib.rs | `greek_research_speed` |
| Greeks | an age or an epoch costs 10% less in every good but knowledge | `TypeData::get_cost@00664090` | modelled at lib.rs | `greek_research_cost` |
| Greeks | a Library or a University costs 66% less | `TypeData::get_cost@00664090` | modelled at city.rs | `greek_library_cost` |
| Romans | legions train faster | `ObjectData::train_time@006508c0` | modelled at lib.rs | `roman_legion_fast` |
| French | siege units train faster | `ObjectData::train_time@006508c0` | modelled at lib.rs | `french_siege_speed` |
| French | special (general) units train faster, 0 in the rules | `ObjectData::train_time@006508c0` | not modelled | `french_special_speed` |
| Germans | aircraft train faster | `ObjectData::train_time@006508c0` | modelled at lib.rs | `german_air_speed` |
| Germans | submarines train faster | `ObjectData::train_time@006508c0` | modelled at lib.rs | `german_submarine_speed` |
| Japanese | barracks units train faster (negative in the rules) | `ObjectData::train_time@006508c0` | not modelled | `japanese_barracks_speed` |
| Japanese | carriers train faster | `ObjectData::train_time@006508c0` | not modelled | `japanese_carrier_speed` |
| Japanese | a Farm costs 50% less | `TypeData::get_cost@00664090` | modelled at city.rs | `japanese_farms_cheaper` |
| Japanese | a farmer gathers 25% more food (calc_gather's flat branch) | `BuildTypeData::calc_gather@00639e40` | modelled at holdings.rs | `japanese_fishing_boats` |
| Indians | a building's support ramp zeroed, halved rounding up for a wonder or a defence | `TypeData::get_cost@00664090` | modelled at cost.rs | `indian_ramp` |
| Indians | an elephant-line unit costs 15% less, the last step | `TypeData::get_cost@00664090` | modelled at cost.rs | `indian_elephant` |
| Lakota | raze speed (train_time's tail) | `ObjectData::train_time@006508c0` | not modelled | `lakota_raze_speed` |
| Mongols | stable units train faster | `ObjectData::train_time@006508c0` | modelled at lib.rs | `mongol_stable_speed` |
| British | ships, archers and anti-air train faster | `ObjectData::train_time@006508c0` | modelled at lib.rs | `british_ship_speed` |

## 4.1 Constants: Aztecs to Romans

A row per constant `rules.xml` names with the nation's prefix. **Rules** is
the number the install holds (a ratio is `a/b`; `?` is a value that is not a
number). **The export reads it in** is up to three of the functions whose
body dereferences a field of that name — the loaders (`Tribe::parse`) are
left out; a constant with no reader in the export is dead there too.

<!-- GENERATED:consts1 -->
| constant | rules | the export reads it in | status | witness |
|---|---:|---|---|---|
| `AZTEC_SLAY_BONUS` | 15 | Unit::plunder | not carried | `aztec_slay_bonus` |
| `AZTEC_SLAY_CAP` | 60 | Unit::plunder | not carried | `aztec_slay_cap` |
| `AZTEC_BARRACKS_LIGHT` | 1 | Build::activate | modelled at nations.rs | `aztec_barracks_light` |
| `AZTEC_MAX_LIGHT` | 3 | Build::activate | modelled at nations.rs | `aztec_max_light` |
| `AZTEC_PLUNDER` | 100 | Build::plunder, Unit::plunder | modelled at plunder.rs | `aztec_plunder` |
| `AZTEC_FREE_MILITARY` | 1 | ConquestGame::setup_scenario_bonuses, Leader::init | modelled at tech.rs | `aztec_free_military` |
| `MAYA_GARRISON_ARROWS` | 1 | BuildData::get_base_garrison_arrows | not carried | `maya_garrison_arrows` |
| `MAYA_BUILDING_TIMBER` | 25 | TypeData::get_cost | modelled at city.rs | `maya_building_timber` |
| `MAYA_BUILDING_HP` | 25 | Wall::update_hits | modelled at build.rs | `maya_building_hp` |
| `MAYA_BUILDING_SPEED` | 20 | Wall::update_construct_time | modelled at build.rs | `maya_building_speed` |
| `INCA_WEALTH_CAP` | 33 | Leader::calc_resource_caps | modelled at economy.rs | `inca_wealth_cap` |
| `INCA_WEALTH_PER_MINER` | 10 | BuildTypeData::calc_gather, Leader::calc_gather | modelled at holdings.rs | `inca_wealth_per_miner` |
| `INCA_REFUND` | 25 | Object::take_damage, Unit::plunder | not carried | `inca_refund` |
| `BANTU_CITY_LIMIT` | 1 | Leader::found_cities, LeaderData::get_city_limit | modelled at place.rs | `bantu_city_limit` |
| `BANTU_POP_CAP` | 100 | Leader::calc_pop_cap | modelled at cost.rs | `bantu_pop_cap` |
| `BANTU_FINAL_POP_CAP` | 25 | Leader::calc_pop_cap | modelled at cost.rs | `bantu_final_pop_cap` |
| `BANTU_UNIT_UPGRADES` | 1 | no reader in the export | not carried | `bantu_unit_upgrades` |
| `BANTU_UNITS_MOVE` | 25 | Unit::update_speed | modelled at rares.rs | `bantu_units_move` |
| `BANTU_CITY_COST` | 75 | TypeData::get_cost | modelled at city.rs | `bantu_city_cost` |
| `NUBIAN_RARE` | 50 | LeaderData::calc_rare | modelled at economy.rs | `nubian_rare` |
| `NUBIAN_CARAVAN_LIMIT` | 1 | LeaderData::get_caravan_limit | modelled at ai_units.rs | `nubian_caravan_limit` |
| `NUBIAN_FREE_CARAVAN` | 0 | Build::activate | modelled at nations.rs | `nubian_free_caravan` |
| `NUBIAN_MERCHANTS` | 10 | TypeData::get_cost | not carried | `nubian_merchants` |
| `NUBIAN_HIT_POINTS` | 50 | Unit::update_hits, Wall::update_hits | modelled at cast.rs | `nubian_hit_points` |
| `NUBIAN_MARKET_PRICES` | 20 | LeaderData::calc_market_prices | modelled at ai_make.rs | `nubian_market_prices` |
| `GREEK_RESEARCH_SPEED` | 100 | ObjectData::train_time | modelled at lib.rs | `greek_research_speed` |
| `GREEK_RESEARCH_COST` | 10 | TypeData::get_cost | modelled at lib.rs | `greek_research_cost` |
| `GREEK_KNOWLEDGE_EARLY` | 1 | LeaderData::has_preq | modelled at tech.rs | `greek_knowledge_early` |
| `GREEK_UNIVERSITY_EARLY` | 2 | LeaderData::has_preq, Setup::build_civ_specific | modelled at tech.rs | `greek_university_early` |
| `GREEK_START_SCHOLARS` | 0 | Setup::build_units | not carried | `greek_start_scholars` |
| `GREEK_AGE_KNOWLEDGE` | 0 | TypeData::get_cost | not carried | `greek_age_knowledge` |
| `GREEK_CLASSICAL` | 0 | TypeData::get_cost | not carried | `greek_classical` |
| `GREEK_LIBRARY_COST` | 66 | TypeData::get_cost | modelled at city.rs | `greek_library_cost` |
| `GREEK_SCHOLARS_FAST` | 0 | no reader in the export | not carried | `greek_scholars_fast` |
| `GREEK_DELAY_KNOWLEDGE` | 1 | Leader::gain_tech, Leader::init | modelled at lib.rs | `greek_delay_knowledge` |

## 4.2 Constants: Egyptians to Germans

<!-- GENERATED:consts2 -->
| constant | rules | the export reads it in | status | witness |
|---|---:|---|---|---|
| `ROMAN_FORT_HP` | 0 | Wall::update_hits | modelled at build.rs | `roman_fort_hp` |
| `ROMAN_FORT_RANGE` | 0 | BuildTypeData::get_building_range | not carried | `roman_fort_range` |
| `ROMAN_FORT_BORDERS` | 3 | World::compute_reg_territory | modelled at territory.rs | `roman_fort_borders` |
| `ROMAN_FORT_COST` | 25 | TypeData::get_cost | modelled at cost.rs | `roman_fort_cost` |
| `ROMAN_FORT_EARLY` | 1 | LeaderData::has_preq, LeaderData::special_preq | modelled at tech.rs | `roman_fort_early` |
| `ROMAN_FORT_SPEED` | 50 | Wall::update_construct_time | modelled at build.rs | `roman_fort_speed` |
| `ROMAN_FORT_UPGRADES` | 1 | Leader::gain_tech | loaded at load.rs | `roman_fort_upgrades` |
| `ROMAN_BARRACKS_LEGION` | 1 | Build::activate | modelled at nations.rs | `roman_barracks_legion` |
| `ROMAN_AGE_FOR_1_LEGION` | 1 | Build::activate | modelled at nations.rs | `roman_age_for_1_legion` |
| `ROMAN_AGE_FOR_2_LEGIONS` | 3 | Build::activate | modelled at nations.rs | `roman_age_for_2_legions` |
| `ROMAN_AGE_FOR_3_LEGIONS` | 5 | Build::activate | modelled at nations.rs | `roman_age_for_3_legions` |
| `ROMAN_MAX_LEGION` | 3 | Build::activate | modelled at nations.rs | `roman_max_legion` |
| `ROMAN_CITY_GATHER` | 10 | LeaderData::calc_city_resources | modelled at economy.rs | `roman_city_gather` |
| `ROMAN_LEGION_CHEAP` | 10 | TypeData::get_cost | modelled at cost.rs | `roman_legion_cheap` |
| `ROMAN_LEGION_FAST` | 10 | ObjectData::train_time | modelled at lib.rs | `roman_legion_fast` |
| `ROMAN_FREE_MILITARY` | 1 | ConquestGame::setup_scenario_bonuses, Leader::init | modelled at tech.rs | `roman_free_military` |
| `EGYPTIAN_WONDERS` | 25 | TypeData::get_cost | modelled at cost.rs | `egyptian_wonders` |
| `EGYPTIAN_GRANARY_EARLY` | 2 | LeaderData::has_preq, Setup::build_civ_specific | modelled at tech.rs | `egyptian_granary_early` |
| `EGYPTIAN_GRANARY_UPGRADES` | 1 | Leader::gain_tech | loaded at load.rs | `egyptian_granary_upgrades` |
| `EGYPTIAN_SCHOLARS_FOOD` | 0 | TypeData::get_cost | not carried | `egyptian_scholars_food` |
| `EGYPTIAN_WONDERS_EARLY` | 1 | LeaderData::special_preq | modelled at tech.rs | `egyptian_wonders_early` |
| `EGYPTIAN_FARMS_PER_CITY_BASE` | 7 | LeaderData::get_farm_limit | modelled at city.rs | `egyptian_farms_per_city_base` |
| `EGYPTIAN_FOOD_COMMERCE` | 10 | Leader::calc_resource_caps | modelled at economy.rs | `egyptian_food_commerce` |
| `EGYPTIAN_FARM_WEALTH` | 2 | BuildTypeData::calc_gather | modelled at holdings.rs | `egyptian_farm_wealth` |
| `TURK_SIEGE_RANGE` | 3 | UnitData::max_range | modelled at fight.rs | `turk_siege_range` |
| `TURK_SIEGE_LOS` | 3 | Unit::update_los | modelled at vision.rs | `turk_siege_los` |
| `TURK_ASSIMILATE` | 200 | Build::process, IFaceSelected::update_progress | modelled at city.rs | `turk_assimilate` |
| `TURK_CITIZEN` | 33 | TypeData::get_cost | modelled at lib.rs | `turk_citizen` |
| `TURK_FREE_SIEGE` | 2 | Build::activate | modelled at nations.rs | `turk_free_siege` |
| `TURK_FREE_SIEGE_UPGRADES` | 1 | Leader::gain_tech | loaded at load.rs | `turk_free_siege_upgrades` |
| `TURK_PACK` | 100 | SpellTypeData::get_job_time | not carried | `turk_pack` |
| `TURK_MILITARY_CHEAP` | 33 | TypeData::get_cost | modelled at lib.rs | `turk_military_cheap` |
| `SPANISH_LOS` | 0 | Unit::update_los | not carried | `spanish_los` |
| `SPANISH_EXTRA_SCOUT` | 1 | Setup::build_units | not carried | `spanish_extra_scout` |
| `SPANISH_FREE_TRIREME` | 1 | Build::activate | modelled at nations.rs | `spanish_free_trireme` |
| `SPANISH_RUINS_BASE` | 30 | Unit::explore_goody | modelled at goody.rs | `spanish_ruins_base` |
| `SPANISH_RUINS` | 26 | Unit::explore_goody | modelled at goody.rs | `spanish_ruins` |
| `SPANISH_COMMANDOS` | 1 | no reader in the export | not carried | `spanish_commandos` |
| `SPANISH_SCOUT_UPGRADES` | 1 | Leader::gain_tech | loaded at load.rs | `spanish_scout_upgrades` |
| `FRENCH_SUPPLY_HEAL_RATE` | 20 | Build::process, Unit::process_healing | modelled at supply.rs | `french_supply_heal_rate` |
| `FRENCH_SPECIAL_CRAFT` | 0 | UnitData::mana | modelled at cast.rs | `french_special_craft` |
| `FRENCH_SPECIAL_COST` | 0 | TypeData::get_cost | not carried | `french_special_cost` |
| `FRENCH_SPECIAL_SPEED` | 0 | ObjectData::train_time | not carried | `french_special_speed` |
| `FRENCH_SIEGE_COST` | 15 | TypeData::get_cost | modelled at lib.rs | `french_siege_cost` |
| `FRENCH_SIEGE_SPEED` | 25 | ObjectData::train_time | modelled at lib.rs | `french_siege_speed` |
| `FRENCH_SIEGE_MOVE` | 20 | Unit::update_speed | modelled at rares.rs | `french_siege_move` |
| `FRENCH_FREE_GENERAL` | 1 | Build::activate | modelled at nations.rs | `french_free_general` |
| `FRENCH_FREE_SUPPLY` | 1 | Build::activate | modelled at nations.rs | `french_free_supply` |
| `FRENCH_ENTRENCH` | 100 | SpellTypeData::get_job_time | not carried | `french_entrench` |
| `FRENCH_LUMBERMILL_EARLY` | 2 | LeaderData::has_preq, Setup::build_civ_specific | modelled at tech.rs | `french_lumbermill_early` |
| `FRENCH_LUMBERMILL_UPGRADES` | 1 | Leader::gain_tech | loaded at load.rs | `french_lumbermill_upgrades` |
| `FRENCH_WOODIES` | 1 | BuildTypeData::calc_gather | modelled at gather.rs | `french_woodies` |
| `FRENCH_TIMBER_COMMERCE` | 10 | Leader::calc_resource_caps | modelled at economy.rs | `french_timber_commerce` |
| `BRITISH_COMMERCE` | 25 | Leader::calc_resource_caps | modelled at economy.rs | `british_commerce` |
| `BRITISH_SHIP_SPEED` | 33 | ObjectData::train_time | modelled at lib.rs | `british_ship_speed` |
| `BRITISH_ARCHER_SPEED` | 0 | ObjectData::train_time | modelled at lib.rs | `british_archer_speed` |
| `BRITISH_ARCHER_UPGRADES` | 1 | Leader::gain_tech | loaded at load.rs | `british_archer_upgrades` |
| `BRITISH_AA_COST` | 25 | TypeData::get_cost | not carried | `british_aa_cost` |
| `BRITISH_AA_SPEED` | 33 | ObjectData::train_time, Wall::update_construct_time | modelled at build.rs | `british_aa_speed` |
| `BRITISH_FREE_SHIP` | 0 | no reader in the export | not carried | `british_free_ship` |
| `BRITISH_TAXATION` | 100 | Leader::calc_gather, TypeData::get_desc_parsed | modelled at lib.rs | `british_taxation` |
| `BRITISH_TOWER_RANGE` | 2 | BuildTypeData::get_building_range | modelled at fight.rs | `british_tower_range` |
| `BRITISH_AGE_FOR_1_ARCHER` | 0 | Build::activate | modelled at nations.rs | `british_age_for_1_archer` |
| `BRITISH_AGE_FOR_2_ARCHERS` | 2 | Build::activate | modelled at nations.rs | `british_age_for_2_archers` |
| `BRITISH_AGE_FOR_3_ARCHERS` | 3 | Build::activate | modelled at nations.rs | `british_age_for_3_archers` |
| `BRITISH_TAXATION_DISCOUNT` | 50 | TypeData::get_cost | modelled at lib.rs | `british_taxation_discount` |
| `BRITISH_FREE_FISHERMEN` | 0 | Build::activate | modelled at nations.rs | `british_free_fishermen` |

## 4.3 Constants: Russians to Mongols

<!-- GENERATED:consts3 -->
| constant | rules | the export reads it in | status | witness |
|---|---:|---|---|---|
| `GERMAN_SUBMARINE_COST` | 25 | TypeData::get_cost | modelled at lib.rs | `german_submarine_cost` |
| `GERMAN_SUBMARINE_SPEED` | 33 | ObjectData::train_time | modelled at lib.rs | `german_submarine_speed` |
| `GERMAN_AIR_SPEED` | 33 | ObjectData::train_time | modelled at lib.rs | `german_air_speed` |
| `GERMAN_FREE_FIGHTER` | 2 | Build::activate | modelled at nations.rs | `german_free_fighter` |
| `GERMAN_FREE_BOMBER` | 0 | no reader in the export | not carried | `german_free_bomber` |
| `GERMAN_METAL_EARLY` | 0 | LeaderData::has_preq | modelled at tech.rs | `german_metal_early` |
| `GERMAN_METAL_UPGRADES` | 0 | Leader::gain_tech | carried, unread | `german_metal_upgrades` |
| `GERMAN_MINE_COST` | 0 | Build::plunder, TypeData::get_cost | modelled at city.rs | `german_mine_cost` |
| `GERMAN_MINERS` | 0 | BuildTypeData::calc_gather | not carried | `german_miners` |
| `GERMAN_HEAVY_INFANTRY` | 0 | Leader::gain_tech | loaded at load.rs | `german_heavy_infantry` |
| `GERMAN_INDUSTRY_UPGRADES` | 0 | Leader::gain_tech | carried, unread | `german_industry_upgrades` |
| `GERMAN_INDUSTRY_DISCOUNT` | 50 | TypeData::get_cost | modelled at cost.rs | `german_industry_discount` |
| `GERMAN_COMPLETION_BONUS` | 50 | Build::do_bonus | modelled at city.rs | `german_completion_bonus` |
| `GERMAN_BUILDINGS_EARLY` | 1 | LeaderData::has_preq | modelled at tech.rs | `german_buildings_early` |
| `GERMAN_INDUSTRY_EARLY` | 1 | LeaderData::special_preq | modelled at tech.rs | `german_industry_early` |
| `GERMAN_CITY_GATHER` | 5 | BuildTypeOut::set_parse_data, LeaderData::calc_city_resources, Scene::set_help_type | modelled at economy.rs | `german_city_gather` |
| `RUSSIAN_ATTRITION` | 100 | Leader::calc_attrition | modelled at attrition.rs | `russian_attrition` |
| `RUSSIAN_ATTRITION_UPGRADES` | 1 | Leader::gain_tech | loaded at load.rs | `russian_attrition_upgrades` |
| `RUSSIAN_OIL` | 20 | LeaderData::calc_resource_bonuses | modelled at economy.rs | `russian_oil` |
| `RUSSIAN_BORDERS` | 0 | World::compute_reg_territory | modelled at territory.rs | `russian_borders` |
| `RUSSIAN_BORDERS_PER_AGE` | 1 | World::compute_reg_territory | modelled at territory.rs | `russian_borders_per_age` |
| `RUSSIAN_COMMUNISM` | 0 | LeaderData::calc_market_prices | carried, unread | `russian_communism` |
| `RUSSIAN_SPY_COST` | 50 | TypeData::get_cost | modelled at cost.rs | `russian_spy_cost` |
| `RUSSIAN_COSSACK_DAMAGE` | 25 | ObjectData::get_damage | modelled at damage.rs | `russian_cossack_damage` |
| `RUSSIAN_PLUNDER_STEAL` | 1 | Build::plunder, Cities::capture_city | modelled at city.rs | `russian_plunder_steal` |
| `RUSSIAN_FREE_CIVIC` | 1 | ConquestGame::setup_scenario_bonuses, Leader::init | modelled at tech.rs | `russian_free_civic` |
| `CHINESE_KNOWLEDGE_UPGRADES` | 0 | Leader::gain_tech | loaded at load.rs | `chinese_knowledge_upgrades` |
| `CHINESE_HERBAL_LORE` | 1 | Leader::gain_tech | loaded at load.rs | `chinese_herbal_lore` |
| `CHINESE_SCIENCE_DISCOUNT` | 20 | TypeData::get_cost | modelled at lib.rs | `chinese_science_discount` |
| `CHINESE_CITIZEN_INSTANT` | 1 | ObjectData::train_time | modelled at lib.rs | `chinese_citizen_instant` |
| `CHINESE_CITIZEN_CREATION` | 100 | ObjectData::train_time | modelled at lib.rs | `chinese_citizen_creation` |
| `CHINESE_LARGE_CITY` | 1 | ConquestGame::build_cities, Setup::build_cities | not carried | `chinese_large_city` |
| `CHINESE_LARGE_CITIES` | 1 | Build::activate, City::assimilate, ConquestGame::build_cities +1 | modelled at activate.rs | `chinese_large_cities` |
| `JAPANESE_DAMAGE` | -5 | ObjectData::get_damage | modelled at damage.rs | `japanese_damage` |
| `JAPANESE_BARRACKS_COST` | -7 | TypeData::get_cost | not carried | `japanese_barracks_cost` |
| `JAPANESE_BARRACKS_SPEED` | -10 | ObjectData::train_time | not carried | `japanese_barracks_speed` |
| `JAPANESE_SHIP_COST` | 10 | TypeData::get_cost | not carried | `japanese_ship_cost` |
| `JAPANESE_AIRCRAFT_CARRIERS_SPEED` | 33 | no reader in the export | not carried | `japanese_aircraft_carriers_speed` |
| `JAPANESE_FISHING_BOATS` | 25 | BuildTypeData::calc_gather, LeaderData::calc_rare | modelled at holdings.rs | `japanese_fishing_boats` |
| `JAPANESE_CARRIER_SPEED` | 33 | ObjectData::train_time | not carried | `japanese_carrier_speed` |
| `JAPANESE_FARMS_CHEAPER` | 50 | TypeData::get_cost | modelled at cost.rs | `japanese_farms_cheaper` |
| `KOREAN_START_CITIZEN` | 3 | no reader in the export | not carried | `korean_start_citizen` |
| `KOREAN_FREE_CITIZEN` | 5 | no reader in the export | not carried | `korean_free_citizen` |
| `KOREAN_CITIZENS` | [1, ...] | Build::activate, Setup::build_units | modelled at nations.rs | `korean_citizens` |
| `KOREAN_TEMPLE_UPGRADES` | 2 | Leader::gain_tech, LeaderData::has_preq, Setup::build_civ_specific | modelled at tech.rs | `korean_temple_upgrades` |
| `KOREAN_TEMPLE_TAX_UPGRADES` | 0 | Leader::gain_tech | loaded at load.rs | `korean_temple_tax_upgrades` |
| `KOREAN_MILITIA_UPGRADES` | 1 | Leader::gain_tech | loaded at load.rs | `korean_militia_upgrades` |
| `KOREAN_BUILD_UNDER_FIRE` | 1 | Object::do_damage, Unit::do_build, Unit::do_repair | modelled at orders.rs | `korean_build_under_fire` |
| `KOREAN_REPAIR` | 50 | Unit::do_repair | modelled at orders.rs | `korean_repair` |
| `KOREAN_TOWER_COST` | 33 | TypeData::get_cost | modelled at cost.rs | `korean_tower_cost` |
| `MONGOL_STABLE_COST` | 10 | TypeData::get_cost | not carried | `mongol_stable_cost` |
| `MONGOL_STABLE_SPEED` | 20 | ObjectData::train_time | modelled at lib.rs | `mongol_stable_speed` |
| `MONGOL_ATTRITION` | 50 | Leader::calc_anti_attrition | modelled at attrition.rs | `mongol_attrition` |
| `MONGOL_THREE_MIL_CAVALRY` | 3 | Build::activate, CheckSums::check_constants, Game::walk_rules_data +3 | modelled at nations.rs | `mongol_three_mil_cavalry` |
| `MONGOL_FREE_CAVALRY` | 2 | Build::activate | modelled at nations.rs | `mongol_free_cavalry` |
| `MONGOL_START_CAVALRY` | 1 | Build::activate | modelled at nations.rs | `mongol_start_cavalry` |
| `MONGOL_FREE_FORAGE` | 1 | Leader::gain_tech | loaded at load.rs | `mongol_free_forage` |
| `MONGOL_NOMADIC_FOOD` | 1 | Leader::calc_gather | modelled at economy.rs | `mongol_nomadic_food` |

## 4.4 Constants: Iroquois to Persians

<!-- GENERATED:consts4 -->
| constant | rules | the export reads it in | status | witness |
|---|---:|---|---|---|
| `IROQUOIS_LOS` | 0 | CheckSums::check_constants, Game::walk_rules_data, GameLog::say_checksum +3 | not carried | `iroquois_los` |
| `IROQUOIS_FOOD` | 2 | BuildTypeData::calc_gather, BuildTypeOut::draw_template | modelled at economy.rs | `iroquois_food` |
| `IROQUOIS_SENATE` | 0 | Setup::build_civ_specific | not carried | `iroquois_senate` |
| `IROQUOIS_HEAL_RATE` | [20 frames, ...] | Unit::process_healing | not carried | `iroquois_heal_rate` |
| `IROQUOIS_FREE_SCOUT` | 1 | Build::activate | modelled at nations.rs | `iroquois_free_scout` |
| `IROQUOIS_SCOUT_UPGRADES` | 1 | Leader::gain_tech | loaded at load.rs | `iroquois_scout_upgrades` |
| `IROQUOIS_GOVS_EARLY` | 0 | TypeData::get_preq | modelled at tech.rs | `iroquois_govs_early` |
| `IROQUOIS_EXTRA_HITS` | 5 | Unit::update_hits | modelled at cast.rs | `iroquois_extra_hits` |
| `IROQUOIS_QUICK_SENATE` | 1 | BuildData::construct_time, TypeData::get_cost | modelled at build.rs | `iroquois_quick_senate` |
| `IROQUOIS_SPEAR_SPEED` | [1, ...] | UnitData::speed | modelled at cast.rs | `iroquois_spear_speed` |
| `LAKOTA_FOOD` | 4 | Leader::calc_gather, Object::insert_inside, Object::remove_from_inside +2 | modelled at economy.rs | `lakota_food` |
| `LAKOTA_HERBAL_LORE` | 1 | no reader in the export | not carried | `lakota_herbal_lore` |
| `LAKOTA_RAZE_SPEED` | 100 | ObjectData::train_time | not carried | `lakota_raze_speed` |
| `LAKOTA_RAZE_PRICE` | 0 | Build::plunder, Object::disband | modelled at disband.rs | `lakota_raze_price` |
| `LAKOTA_CAV_UPGRADES` | 1 | Leader::gain_tech | loaded at load.rs | `lakota_cav_upgrades` |
| `LAKOTA_CAV_DMG_BONUS` | 1/3 | Object::do_damage | not carried | `lakota_cav_dmg_bonus` |
| `AMERICANS_FREE_SCHOLAR` | 1 | Build::activate | modelled at nations.rs | `americans_free_scholar` |
| `AMERICANS_FREE_BOMBER` | 1 | Build::activate, Leader::gain_tech, LeaderData::get_support_count | modelled at nations.rs | `americans_free_bomber` |
| `AMERICANS_AIRCRAFT` | 20 | TypeData::get_cost | not carried | `americans_aircraft` |
| `AMERICANS_MARINE_SPEED_BONUS` | 2 | Unit::update_speed | not carried | `americans_marine_speed_bonus` |
| `AMERICANS_MARINE_HP_BONUS` | 70 | Unit::update_hits | not carried | `americans_marine_hp_bonus` |
| `AMERICANS_MARINE_ENTRENCH` | 5 | Unit::check_idle | not carried | `americans_marine_entrench` |
| `AMERICANS_STARTING_FARMS` | 3 | Setup::build_units, Setup::small_city_buildings | not carried | `americans_starting_farms` |
| `AMERICANS_FREE_SCIENCE` | 0 | ConquestGame::setup_scenario_bonuses, Leader::init | modelled at tech.rs | `americans_free_science` |
| `AMERICANS_SCIENCE_DISCOUNT` | 0 | TypeData::get_cost | modelled at lib.rs | `americans_science_discount` |
| `AMERICANS_UPGRADE_DISCOUNT` | 25 | TypeData::get_cost | modelled at cost.rs | `americans_upgrade_discount` |
| `AMERICANS_BARRACKS_GATHER` | 2 | Leader::calc_gather, Object::insert_inside, Object::remove_from_inside +2 | modelled at holdings.rs | `americans_barracks_gather` |
| `INDIANS_CITY_RADIUS` | 4 | Leader::plan_strategy, LeaderData::get_radius | modelled at city.rs | `indians_city_radius` |
| `INDIANS_CARAVAN` | 15 | Caravan::trade_value | modelled at caravan.rs | `indians_caravan` |
| `INDIANS_ELEPHANT_COST` | 15 | TypeData::get_cost | modelled at cost.rs | `indians_elephant_cost` |
| `INDIANS_ELEPHANT_UPGRADES` | 1 | Leader::gain_tech | loaded at load.rs | `indians_elephant_upgrades` |
| `DUTCH_FREE_COMMERCE` | 1 | ConquestGame::setup_scenario_bonuses, Leader::init | modelled at tech.rs | `dutch_free_commerce` |
| `DUTCH_COMMERCE_DISCOUNT` | 10 | TypeData::get_cost | modelled at lib.rs | `dutch_commerce_discount` |
| `DUTCH_FORT_PLACEMENT` | 0 | BuildTypeData::blocked_tcoord | modelled at tcoord.rs | `dutch_fort_placement` |
| `DUTCH_INTEREST` | 5 | Leader::do_gather, Options::describe | modelled at economy.rs | `dutch_interest` |
| `DUTCH_INTEREST_CAP` | 50 | Leader::do_gather, Options::describe | modelled at economy.rs | `dutch_interest_cap` |
| `DUTCH_FREE_LIGHT_SHIP` | 1 | Build::activate | modelled at nations.rs | `dutch_free_light_ship` |
| `DUTCH_FORT_SPEED` | 0 | Wall::update_construct_time | modelled at build.rs | `dutch_fort_speed` |
| `DUTCH_ATTACK_BONUS` | 1 | ObjectData::armor, ObjectData::attack | modelled at fight.rs | `dutch_attack_bonus` |
| `DUTCH_HP_BONUS` | 10 | Unit::update_hits | modelled at cast.rs | `dutch_hp_bonus` |
| `DUTCH_SHIP_DISCOUNT` | 10 | TypeData::get_cost | modelled at cost.rs | `dutch_ship_discount` |
| `PERSIANS_DESPOTISM` | 0 | ConquestGame::setup_scenario_bonuses, Leader::init | modelled at tech.rs | `persians_despotism` |
| `PERSIANS_BONUS_FOOD` | 50 | ConquestGame::setup_scenario_bonuses, Leader::init | modelled at lib.rs | `persians_bonus_food` |
| `PERSIANS_CIVIC_DISCOUNT` | 30 | TypeData::get_cost | modelled at lib.rs | `persians_civic_discount` |
| `PERSIANS_TAXATION` | 1 | Leader::gain_tech | loaded at load.rs | `persians_taxation` |

## 5.1 The census of `has_tribe_bonus` callers, by address (first half)

Every function in the export that calls `LeaderData::has_tribe_bonus`,
sorted by address, with the indices it passes. **`carried at` is
function-level**: the address is cited in that module, so the function is
carried there; whether each nation's arm of it is, is §3 (for three
nations) and the constants of §4 (for the rest). `not sim` rows are the
interface, the Conquer-the-World and scenario classes, and the lobby's
start builder (`Setup::`), whose state the harness takes from the dump.

<!-- GENERATED:census1 -->
| function | nations | status | witness |
|---|---|---|---|
| `LeaderData::has_bonus@0047df90` | 4 | not modelled | `0047df90` |
| `Setup::large_city_buildings@005aac80` | 19 | not sim | `the lobby's start builder; the harness takes the start state from the dump` |
| `Setup::small_city_buildings@005aae10` | 19 20 | not sim | `the lobby's start builder; the harness takes the start state from the dump` |
| `Setup::build_units@005aafc0` | 5 9 16 19 20 22 | not sim | `the lobby's start builder; the harness takes the start state from the dump` |
| `Setup::build_civ_specific@005ab760` | 4 5 7 10 16 18 22 | not sim | `the lobby's start builder; the harness takes the start state from the dump` |
| `Setup::build_cities@005ab910` | 14 | not sim | `the lobby's start builder; the harness takes the start state from the dump` |
| `Setup::build_empire@005abb80` | 9 | modelled at ai_sites.rs | `sees_every_cell` |
| `AnimalData::is_seen@005d8160` | 4 9 | carried at fight.rs | `005d8160` |
| `Unit::process_healing@005e0670` | 10 18 | carried at heal.rs | `005e0670` |
| `Unit::process_cloak@005e1c20` | 18 | carried at lib.rs | `005e1c20` |
| `Unit::do_build@005eebf0` | 16 | carried at orders.rs | `005eebf0` |
| `Unit::set_new_location@005f8d20` | 18 | carried at collide.rs | `005f8d20` |
| `Unit::explore_goody@005f9780` | 9 | carried at goody.rs | `005f9780` |
| `Unit::update_speed@006055c0` | 0 3 10 | carried at rares.rs | `006055c0` |
| `Unit::plunder@00605b00` | 0 | not modelled | `aztec_slay_bonus` |
| `Unit::update_los@0060e4d0` | 8 9 18 | carried at vision.rs | `0060e4d0` |
| `Unit::update_hits@0060e930` | 4 18 22 | carried at cast.rs | `0060e930` |
| `Unit::close@0060ee50` | 19 20 | carried at fight.rs | `0060ee50` |
| `UnitData::max_range@0060fe90` | 8 | carried at fight.rs | `0060fe90` |
| `Unit::init@00612100` | 18 19 20 | carried at lib.rs | `00612100` |
| `Build::process@0061edf0` | 8 23 | carried at city.rs | `0061edf0` |
| `Build::plunder@00623660` | 0 12 13 19 | modelled at city.rs | `russian_plunder_steal` |
| `Build::activate@00623e20` | 4 8 9 10 11 14 16 20 23 | carried at city.rs | `00623e20` |
| `Build::do_bonus@00627140` | 12 | not modelled | `00627140` |
| `Build::close@00628980` | 23 | carried at city.rs | `00628980` |
| `BuildOut::make_options@00629ed0` | 4 | not sim | `interface or render side` |
| `BuildData::construct_time@0062d5c0` | 20 | modelled at build.rs | `iroquois_quick_senate` |
| `BuildTypeData::get_building_range@00639990` | 6 | carried at fight.rs | `00639990` |
| `BuildTypeData::calc_gather@00639e40` |  +var | carried at economy.rs | `00639e40` |
| `Wall::update_construct_time@0063d560` | 1 6 11 22 | carried at economy.rs | `0063d560` |
| `Wall::update_hits@0063f0d0` | 1 4 6 | carried at build.rs | `0063f0d0` |
| `Wall::do_construct@006434d0` | 7 | carried at construct.rs | `006434d0` |
| `ObjectData::get_damage@00644130` | 15 | carried at damage.rs | `00644130` |
| `Object::disband@006455c0` | 0 19 | carried at object_disband.rs | `006455c0` |
| `Object::insert_inside@00647e90` | 19 20 | carried at holdings.rs | `00647e90` |
| `Object::remove_from_inside@006480f0` | 19 20 | carried at object_remove_from_inside.rs | `006480f0` |
| `ObjectData::train_time@006508c0` | 5 15 | carried at lib.rs | `006508c0` |
| `Object::take_damage@00652020` | 0 2 | carried at fight.rs | `00652020` |
| `TypeData::get_desc_parsed@006626e0` | 11 | not sim | `interface or render side` |
| `TypeData::get_cost@00664090` | 5 7 8 12 14 15 20 22 23 | carried at lib.rs | `00664090` |
| `GoodData::is_seen@0066e4c0` | 4 9 | carried at good_data_is_seen.rs | `0066e4c0` |
| `SpellTypeData::get_job_time@00675800` | 9 | carried at lib.rs | `00675800` |
| `World::compute_reg_territory@006b0bb0` | 6 13 | carried at ai.rs | `006b0bb0` |
| `Leader::plan_strategy@006b9620` | 21 | carried at ai_census.rs | `006b9620` |
| `Leader::create_buildings@006c1be0` | 19 | carried at ai_build.rs | `006c1be0` |
| `Leader::create_units@006c40a0` | 19 | carried at ai_units.rs | `006c40a0` |
| `Leader::found_cities@006c7a60` | 3 | carried at ai_sites.rs | `006c7a60` |
| `Leader::market_speculation@006c8110` | 4 | carried at ai_make.rs | `006c8110` |
| `Leader::production_ai_setup@006c83e0` | 19 | carried at ai.rs | `006c83e0` |
| `Leader::use_market@006c91c0` | 4 | carried at ai_make.rs | `006c91c0` |
| `Leader::calc_anti_attrition@006cdcc0` | 17 | modelled at attrition.rs | `mongol_attrition` |
| `Leader::calc_attrition@006cdea0` | 4 13 | carried at lib.rs | `006cdea0` |

## 5.2 The census (second half)

<!-- GENERATED:census2 -->
| function | nations | status | witness |
|---|---|---|---|
| `Leader::do_gather@006ce450` | 22 | carried at economy.rs | `006ce450` |
| `Leader::calc_resource_caps@006ce900` | 2 7 10 11 | carried at economy.rs | `006ce900` |
| `Leader::calc_gather@006ceee0` | 2 11 17 19 20 | carried at economy.rs | `006ceee0` |
| `LeaderOut::warn_resources@006d3650` | 19 | not sim | `interface or render side` |
| `LeaderData::calc_city_resources@006d5530` | 6 12 | carried at economy.rs | `006d5530` |
| `LeaderData::can_see_borders@006d5f60` | 19 | not modelled | `006d5f60` |
| `LeaderData::get_city_limit@006d6130` | 3 | carried at place.rs | `006d6130` |
| `LeaderData::special_preq@006d7030` | 3 6 7 12 | modelled at tech.rs | `egyptian_wonders_early` |
| `LeaderData::get_support_count@006da110` | 20 | carried at lib.rs | `006da110` |
| `LeaderData::get_ships_speed_upgrade@006da800` | 4 | carried at lib.rs | `006da800` |
| `LeaderData::get_troops_speed_upgrade@006da850` | 4 | carried at tech.rs | `006da850` |
| `LeaderData::get_vehicle_speed_upgrade@006da8a0` | 4 | carried at tech.rs | `006da8a0` |
| `LeaderData::get_building_speed_upgrade@006dae90` | 4 | carried at tech.rs | `006dae90` |
| `LeaderData::get_building_hp_upgrade@006daee0` | 4 | carried at tech.rs | `006daee0` |
| `LeaderData::calc_resource_bonuses@006db030` | 13 | carried at economy.rs | `006db030` |
| `LeaderData::get_farm_limit@006db270` | 7 | modelled at city.rs | `egyptian_farms_per_city_base` |
| `LeaderData::get_radius@006db790` | 21 | carried at build_type_data_blocked_location.rs | `006db790` |
| `LeaderData::has_preq@006db810` | 4 5 6 7 10 12 13 16 | carried at ammo_do_damage.rs | `006db810` |
| `LeaderData::calc_market_prices@006dc2a0` | 4 13 | carried at ai_make.rs | `006dc2a0` |
| `Leader::calc_pop_cap@006dc490` | 3 | carried at build_finished_628490.rs | `006dc490` |
| `Leader::lost_a_city@006dc6d0` | 23 | carried at city.rs | `fn lost_a_city` |
| `LeaderData::get_caravan_limit@006dca50` | 4 | carried at ai_units.rs | `006dca50` |
| `Leader::gain_tech@006dcb60` | 5 6 7 8 9 10 11 12 13 14 16 17 18 19 20 21 23 | carried at lib.rs | `006dcb60` |
| `LeaderData::get_general_upgrade@006e0830` | 4 | carried at cast_decoy.rs | `006e0830` |
| `LeaderData::get_supply_upgrade@006e0880` | 4 | carried at supply.rs | `006e0880` |
| `LeaderData::calc_rare@006e08d0` | 4 15 | carried at economy.rs | `006e08d0` |
| `LeaderData::has_bonus@006e0b80` | 4 | not modelled | `006e0b80` |
| `LeaderData::get_spy_upgrade@006e1090` | 4 | carried at supply.rs | `006e1090` |
| `LeaderData::get_troops_los_upgrade@006e1110` | 4 | carried at tech.rs | `006e1110` |
| `LeaderData::get_heal_level@006e12e0` | 4 | not modelled | `006e12e0` |
| `LeaderData::has_tribe_bonus@006e1370` |  +var | carried at borders_is_border_hidden.rs | `006e1370` |
| `Leader::init@006e3930` | 0 5 6 13 19 20 22 23 | carried at lib.rs | `006e3930` |
| `Options::describe@00724460` | 4 22 | not sim | `interface or render side` |
| `Cities::capture_city@00733380` | 13 23 | carried at cities_capture_city.rs | `00733380` |
| `City::capture@00736c40` | 23 | carried at city.rs | `00736c40` |
| `City::assimilate@00738e90` | 14 | carried at city.rs | `00738e90` |
| `Caravan::trade_value@0073d9d0` | 21 | carried at caravan.rs | `0073d9d0` |
| `ConquestGame::setup_scenario_bonuses@0078be90` | 0 6 13 20 22 23 | not sim | `Conquer the World, cut from v1` |
| `ConquestGame::build_cities@00796140` | 14 | not sim | `Conquer the World, cut from v1` |
| `StatWin::setup_entries@007ed7b0` | 4 | not sim | `interface or render side` |
| `IFaceMarket::on_redraw@00812d20` | 4 | not sim | `interface or render side` |
| `IFaceMarket::setup_selected@008134d0` | 4 | not sim | `interface or render side` |
| `IFaceOptions::on_redraw@00815a50` | 8 | not sim | `interface or render side` |
| `IFaceResources::update_visibility@0081ba60` | 4 | not sim | `interface or render side` |
| `Scene::set_help_type@008be140` | 12 | not sim | `interface or render side` |
| `CommandPackage::process_sell@009468a0` | 4 | carried at command_package_process_sell.rs | `009468a0` |
| `ScenarioFuncSet::buy_good@009fb790` | 4 | not sim | `the scenario trigger set, cut from v1` |
| `ScenarioFuncSet::sell_good@009fb880` | 4 | not sim | `the scenario trigger set, cut from v1` |
| `ScenarioFuncSet::can_buy_sell@009fb970` | 4 | not sim | `the scenario trigger set, cut from v1` |
| `ScenarioFuncSet::find_sell_price@009fb9e0` | 4 | not sim | `the scenario trigger set, cut from v1` |
| `ScenarioFuncSet::find_buy_price@009fbab0` | 4 | not sim | `the scenario trigger set, cut from v1` |
| `ScenarioFuncSet::set_capital@00a00220` | 23 | not sim | `the scenario trigger set, cut from v1` |

## 6. The four tiers

The lobby's difficulty is a byte 0–5 (`Easiest` … `Toughest`). Every AI
read goes through `LeaderData::get_diff@006ec000` (the multiplayer
semaphore's `multi_diff`, else the lobby byte; this crate reads the lobby
byte, `Sim::ai_difficulty`, the semaphore arm is a SEAM at
`diplomacy.rs`). The comparisons the export makes against it cut the six
settings into **four bands**: `0–1` (the `< 2` and `== 0`/`== 1` reads),
`2` (the `== 2` and `< 3` reads), `3` (the `> 2` and `== 3` reads) and
`4–5` (the `> 3` and `≥ 4` reads). `LeaderData::get_gather_handicap@006d66a0`
is the one table that tells all six apart. The captures on disk hold
difficulty 0 and 5 (runs 53, 54, 346, 347) and the battery's four minted
lobbies at 1–4 (tier 1 Chinese, 2 Russians, 3 Germans, 4 Egyptians).

| reader | what it reads | status | witness |
|---|---|---|---|
| `LeaderData::get_gather_handicap@006d66a0` | AI income percent by setting: −35, −15, −7, 0, +25, +50; zero for a human | modelled at economy.rs | `fn gather_handicap` |
| `LeaderData::get_mod_resource_cap@006d65b0` | the production AI's cap ×½ at 0, ×¾ at 1, ×1 above | modelled at ai_types.rs | `006d65b0` |
| `LeaderData::get_econ_mod@006d6660` | ~~an early return at 0~~ ×½ at 0, ×¾ at 1, ×1 above (§9.1); the export holds no caller | not modelled | `006d6660` |
| `LeaderData::get_diff@006ec000` | the read itself | modelled at ai_types.rs | `fn ai_difficulty` |
| `Leader::production_ai_setup@006c83e0` | the stockpile clamp below 3, the `== 2` ×5 term | carried at ai.rs | `006c83e0` |
| `Leader::research_techs@006c6ba0` | age pacing and epoch quota below 2 (0/1 and 2 differ), the `≥ 4` gather weights | carried at ai_research.rs | `006c6ba0` |
| `Leader::create_units@006c40a0` | twelve reads: the army-size gates, the `0`/`1`/`< 2` arms, the matchup coin on every setting but 2 | carried at ai_units.rs | `006c40a0` |
| `Leader::upgrade_units@006c6430` | the `> 2` enemy-age gate, `!= 2` coin, `unit_prod_value` ×3/2 at 0 and above 2, negated below 2 | carried at ai_units.rs | `006c6430` |
| `Leader::unit_prod_value@006cc580` | the `> 1` ×200 weight | carried at ai_sites.rs | `006cc580` |
| `Leader::produce_unit@006cb9e0` | the batch above 1 | carried at ai_units.rs | `006cb9e0` |
| `Leader::create_buildings@006c1be0` | five reads: the `< 3` ×500 escrow, the `0`/`1` fort arm, the `> 2` wonder arm, `< 2`, `== 0` ÷100 | carried at ai_build.rs | `006c1be0` |
| `Leader::found_cities@006c7a60` | the `0`/`1` two-city cap, `== 2` plus one | carried at ai_sites.rs | `006c7a60` |
| `Leader::plan_strategy@006b9620` | the forced strategy bit below 2 | carried at ai_census.rs | `006b9620` |
| `Leader::diplomacy@006bc950` | twelve reads over bands 1, 2, 3, 4 (aggression, tribute, the `> 2` counts, the timer 45×900 vs 30×900) | carried at diplomacy.rs | `006bc950` |
| `Army::do_mustering@006f4260` | `DEFENDING` below 3 | modelled at army.rs | `self.ai_difficulty() < 3` |
| `Army::release_mustering@006f87c0` | release thresholds 6, 10, 12 at settings 0, 1, 2 | carried at army.rs | `006f87c0` |
| `Army::find_target@006f69b0` | the difficulty gate and its `> 2` arms | carried at army.rs | `006f69b0` |
| `Object::find_nearby_target@00648da0` | the AI flag at 0 | carried at fight.rs | `00648da0` |
| `Object::take_damage@00652020` | the `frame_attacked` stamp below 2; the `> 1` peasant arms | carried at fight.rs | `00652020` |
| `Object::do_launch@0064f3b0` | a launch against the leader winning the wonder race, above 2 | carried at airbase.rs | `0064f3b0` |
| `Unit::do_gather@005ef2a0` | the AI repair arm above 1 | modelled at orders.rs | `find_repair_spot@00604320` |
| `Build::process@0061edf0` | the AI repair-order arm above 1, building below half hits | carried at city.rs | `0061edf0` |
| `ConquestGame::*` (`demand_tribute`, `attacked_player`, `setup_*`; `ConquestLeader::plan_strategy@0079ea80`, §9.1) | Conquer the World's own difficulty | not sim | `cut from v1` |
| `SetupWin::*`, `Options`, `do_save`, `do_load`; `ScenarioFuncSet::get_difficulty@009e5320` | the lobby's combo and the save format; ~~`get_difficulty`'s too~~ a conquest script's getter (§9.1) | not sim | `interface and save format` |

The tier arms are literals in the readers' bodies (`economy.rs` says of the
six handicap percentages that no constant stands behind them; `rules.xml`'s
`handicaps` list is the multiplayer per-player table, read only under
`semaphore[0] & 4`, off in a solo lobby). So "every tier's constants" is
this table: the six handicap percentages, the three cap factors, and the
thresholds. **Function-level again**: `carried at` does not say each band's
arm of the function is built. The arms the tree built are listed in
`docs/AI.md` §80.2 (H1–H14) and ride on the diffs the captures at 0 and 5
give; the battery's tiers 1–4 are the only runs that reach the others, and
none has been read below its first parting.

## 7. Candidates, ranked by the held-out lobbies

None is a mechanism the diff has named. Each is a **supposition booked
with the frame its lobby parts on** (`docs/AI.md` §125); the battery is
never debugged against, so a candidate is a row for the successor item that
a pass books against a word, not a fix for the lobby that exposed it.

1. ~~**Chinese citizens (`ObjectData::train_time@006508c0`, arm 0xe).**~~
   **Built, item 3092** (`train_tail`, `lib.rs`). The arm sits after the
   Japanese arms and before the British, and applies when the unit's type
   index is `BASE_UNITTYPES`/`PEASANTSKOREAN` (0x32/0x33, unit types 0 and
   1), or `TypeData::is_merchant` (the three exact ids), or the type has
   `unit_flags2 & 8` (the caravan bit, `+0x2b8` — read off the listing at
   `00650d1a`: `ecx` is still the type record across `is_merchant`).
   Where it applies it divides by `CREATION + 100`, and under `INSTANT`
   returns 1: nothing after it — the British, French, German arms, the
   speed-upgrade step, the spy and wool arms — runs, and `train_tail`
   returns a lone scale to zero that `MIN_TIME` clamps to 1. The earlier
   steps (ramp, handicap, President, Mongol, Japanese) are discarded by the
   return too. Not diff-backed: no capture on disk has a Chinese leader
   with a queue; the reading and the listing carry it, and the sim test
   `a_chinese_citizen_trains_at_once` fails without the arm. Lobby
   `GreatLakesChineseTier1` (176): see the journal for the battery measure.
2. ~~**Egyptian wonders cost (`TypeData::get_cost@00664090:906`).**~~
   **Built, item 3096** (`city.rs` `building_price` → `cost::Modifiers::egyptian_wonders`,
   applied once after the city tail as `(100 - 25) * cost / 100`). The
   decompile's wonder tail (`LAB_00665ced`, `0x20d < type < 0x21f`) takes
   the marble term first (`marble_wonder_cost`, **not carried**: no marble
   holder has been priced) and then this one; the Korean tower term follows.
   Not diff-backed; the cost test `an_egyptian_wonder_costs_a_quarter_less`
   carries it. Lobby `GreatSaharaEgyptianTier4`: see the journal.
   `Wall::do_construct@006434d0`'s second enforcement of the two-wonders
   limit was built already (`city.rs`, `<= 1 + egyptians`); item 3221 found
   the row stale and added `a_second_wonder_site_survives_its_first_builder_only_for_the_egyptians`.
3. ~~**Russian oil +20% (`calc_resource_bonuses@006db030`) and spy −50%
   (`get_cost:299`).**~~ **Built, item 3108.** The oil term is the
   listing's first: `has_tribe_bonus(0xd)` then `(RUSSIAN_OIL + 100) *
   oil / 100`, before the Pyramids (`economy.rs` `resource_bonuses`, now
   taking a `russians` flag from `Holdings`). The spy term is
   `cost::Modifiers::russian_spy`, its own truncating step ahead of the
   Spy/General arm (`Sim::russian_spy_discount`, loose `is(0x3a, 0)`, set
   on the train and research arms alike: the block `get_cost:224`–`425`
   precedes the research fork). Not diff-backed; the tests
   `a_russian_leader_pays_a_fifth_more_oil` and `a_russian_spy_costs_half`
   carry them. Still open: `calc_market_prices`' communism arm, gated by
   `RUSSIAN_COMMUNISM` = 0 (inert as shipped).
4. **`Leader::gain_tech@006dcb60`'s nation arms** — the `*_UPGRADES`
   fields. **The effect was already carried in another form**: step 13 of
   `TechTree::gain` (`tech.rs`, `free_rules`) is the original's tail of
   free-tech blocks, and the `*_UPGRADES` constants are its `enabled`
   gates; the "carried and read by no module" in the table meant the
   tuning field, not the arm. What was missing was the **range blocks'
   loading** (`rondata::load`, after the French Carpentry block). **Built,
   item 3113**: the Russians (`has_tribe_bonus(0xd)`,
   `russian_attrition_upgrades`, `ALLEGIANCE` `0x254`..`<600`), the Chinese
   herbal lore (`0xe`, `chinese_herbal_lore`, `0x25b`..`0x25d`) and the
   Chinese knowledge block (`0xe`, `chinese_knowledge_upgrades` = 0 as
   shipped, inert, `LITERACY` `0x267`..`0x26b`), each the `PreqMatch`
   shape read off the decompile. Test
   `the_russian_and_chinese_free_tech_blocks_are_loaded`, made to fail by
   narrowing the Russian range. **Moved the battery**: the Chinese lobby
   5663 → 12712 (floor raised); the median stays 2207. Not diff-backed by a
   dump of a Chinese or Russian leader; the lobby's later parting is the
   evidence. **Still unloaded**: the Roman (`+0x40 == 0x1bb`, `!has_tech`
   — a different predicate), Egyptian granary, Mongol forage, Persian
   taxation, Korean temple/militia, Indian elephant, Lakota and Iroquois
   blocks, the Red Fort pair, Colosseum, Tikal, Hanging Gardens, German
   metal/industry — each its own reading of the same listing.
   **Built, item 3124**: the Egyptian granary (power 7, `AGRICULTURE`
   `0x264`..`0x266`), Korean temple (16, `0x24a`..`0x24c`) and temple tax
   (16, `0x24d`..`0x250`, 0 as shipped), Persian taxation (23, same run),
   Mongol forage (17, `0x25e`..`0x260`), the Roman fort block (6; techs whose
   `+0x40` is `0x1bb` the Fort, unowned: `Shape::PreqMatchUnowned`) and the
   unit-line blocks as `TwoPreq` (Lakota 19: `0xbb`..`0xd0`, `0xd8`..`0xdf`;
   Iroquois 18: `0x45`..`0x51`; Indian 21: `0x105`..`0x108`; Korean militia
   16: `0x42`..`0x44`). Test `the_other_nation_free_tech_blocks_are_loaded`,
   made to fail by narrowing the Persian run. **Battery unmoved** (every
   lobby at its pin, median 2270). **Still unloaded**: the wonder-gated blocks
   (`PlayerTech::wonders` is never fed; parked) and the inert German pair.
   **Built, item 3194**: the wonder-gated blocks, `PreqMatch` behind
   `Gate::Wonder`; `Sim::gain_tech` feeds `PlayerTech::wonders` off
   `wonders_held` before the call. Red Fort `0x216`: `red_fort_fortification`
   (1) `FORTIFICATION` `0x251`..`0x253` and `red_fort_tactics` (1)
   `END_FINALTYPES` `0x247`..`0x249`; Hanging Gardens `0x210`
   (`hanging_gardens_upgrades`, 0) `CARPENTRY` `0x261`..`0x263`,
   `METAL_ALLOYS` `0x26c`..`0x26e`, `AGRICULTURE` `0x264`..`0x266`; Colosseum
   `0x212` (`colosseum_fort_upgrades`, 0) `FORTIFICATION`; Tikal `0x214`
   (`tikal_temple_upgrades`, 0) `RELIGION` `0x24a`..`0x24c`. Tests
   `gain_tech_feeds_the_players_held_wonders` (fails with the feed off),
   `a_wonder_gated_free_rule_waits_for_the_wonder`,
   `the_wonder_gated_free_tech_blocks_are_loaded` (fails with Tikal's range
   narrowed). Only Red Fort's blocks are live as shipped, and no battery lobby
   is known to hold one before its parting: **battery unmoved** (median 4770).
   Not diff-backed. **Still unloaded**: the inert German pair.
5. **The remaining national `get_cost` and `train_time` arms** (§3,
   §4): about forty percentage terms, of which `cost.rs` carries the
   French siege and the Spy/General arms and `lib.rs` the Chinese, Dutch,
   American, Persian and Turkish tail arms of the tech lines. The
   uncarried ones that an AI reaches first in its lobby ~~are the German
   air and submarine speeds and costs~~ — built by item 2974 at `lib.rs`
   (`docs/AI.md` §242; the submarine pair diff-backed by the battery
   word, the air speed by the sim test alone).
   **Built, item 3137 (§7.5): the Roman legion pair.** `get_cost:306`–`311`:
   a Roman leader (`has_tribe_bonus(6)`) and a unit that `is(0x84, 0)` (the
   Legion, by lineage) take `ROMAN_LEGION_CHEAP` (10%) off in a truncating step
   after the Russian spy's and before the Terra Cotta Army's
   (`Sim::roman_legion_discount` → `cost::Modifiers::roman_legion`).
   `train_time@006508c0:208`–`213`: the same predicate divides by
   `ROMAN_LEGION_FAST + 100`, after the German arms and before `TROOPS_FASTER`
   (`train_tail`). Test `a_roman_legion_line_costs_a_tenth_less_and_trains_faster`,
   made to fail by disabling both arms. Not diff-backed: no capture holds a
   Roman leader. **Not established**: the crate applies the Russian-spy step
   before the French discount where the listing has it after; the two nations
   are disjoint, so no leader sees the difference.
   **Built, item 3145: the Dutch ship-upgrade discount.** `get_cost:530`–`533`,
   inside the research arm's Military-level block (`role & 0x10000`, ~~`0 < ahead`~~ —
   **amended by item 3329**: the arm follows the `0 < ahead` block's close, so a ship
   research pays it whether or not the line is ahead; run963b's `DROMON` at 54 against
   60, `docs/AI.md` §267):
   `has_tribe_bonus(0x16)` and `UnitTypeData +0x218 == 1` (a sea unit) take
   `DUTCH_SHIP_DISCOUNT` (10) in a truncating step after the Military discount's clamp
   (`cost::Research::dutch_ship`, set in `Sim::research_modifiers`). Test
   `a_dutch_ship_upgrade_costs_a_tenth_less`. Not diff-backed. **Battery unmoved**: the Dutch lobby
   (`EastIndiesDutchTier4`) stays at 1567 and all eight lobbies at their pins, median 2270.
   ~~The American twin (`+0x218 == 0`) is unbuilt (parked).~~ **Built, item 3374** (`docs/AI.md` §287):
   `get_cost:528`–`531`, `has_tribe_bonus(0x14)` and `+0x218 == 0` take `AMERICANS_UPGRADE_DISCOUNT` (25) in the
   step before the Dutch arm's (`cost::Research::american_land`), and `get_cost:1048`–`1053` makes a government
   (`is_gov_type`, `0x26f..0x274`) free for an American leader (`Sim::tech_price`). Both diff-backed by the
   American battery word (run999 block 9376, run999b block 10983). Tests
   `an_american_land_upgrade_costs_a_quarter_less`, `an_american_government_is_free`.
   **Built, item 3160: the Citizen cost arms.** `get_cost`'s tail, after the Spy/General arm and
   the stable rares and before the Statue of Liberty, on the train and research arms alike: for
   `TypeIndex` `0x32`/`0x33`, `WOOL_CITIZENS_COST` (15) for a player holding Wool (`rare` or
   `rare_conquest`), then `TURK_CITIZEN` (33) for `has_tribe_bonus(8)`, each its own truncating step
   (`cost::Modifiers::citizens`, `Sim::citizen_discounts`). Tests `a_turkish_citizen_costs_a_third_less`
   (cost and sim). **Moved the battery**: `GreatLakesTurkishTier4` 1440 → 6982 (count and sequence), floor
   raised; median 2270 → 4770. Audit of the other lobbies' leaders (Russian, Egyptian, Aztec, Spanish,
   Chinese, Roman, Dutch): their `get_cost` arms (Russian spy, Egyptian wonders, Roman legion, Dutch ship)
   are built; the Aztec and Spanish have none in `get_cost`/`train_time`. Not diff-backed by a dump of a
   Turkish leader's citizen price; the lobby's parting later is the evidence.
   **Built, item 3172: the Sulphur siege arm, and the `train_time` audit.** `get_cost:332`–`335`,
   right after Rubber: a unit whose trainer (`+0x40`) is `0x1ae`/`0x1af` takes `SULPHUR_SIEGE_COST`
   (20) for a player holding Sulphur (rare 16, `& 4`; `rare` or `rare_conquest`), a third truncating
   step (`cost::Modifiers::stable_rares` now `[i32; 3]`, `Sim::stable_rare_discounts`; the rule is
   the player's rare, **not** the Turks' — 3171 mis-framed it). Test in `cost::tests`
   (`horses_and_rubber_each_take_their_own_percentage`, the `[15, 0, 20]` row). `train_time@006508c0`'s
   national arms by `has_tribe_bonus` index: 5 Greek research, 6 Roman legion (built), 0xa French,
   0xb British, 0xc German, 0xe Chinese (built), 0xf Japanese, 0x11 Mongol, 0x13 Lakota raze: **no
   battery leader (Aztec 0, Egyptian 7, Turk 8, Spanish 9, Russian 0xd, Dutch 0x16) reaches an
   unbuilt arm.** **Battery unmoved**: all eight lobbies at their pins, median 4770; the Turkish lobby
   parts on 6982 at `use_market+0x1ed` against `make_stuff+0x221`, a market decision, not a price.
   **Diff-backed after all, by the keys that part**: the Persian and Lakota pairs hold Sulphur, and seven
   widenings' parting-key counts fell — run716 149 → 146, run721 175 → 159, run754 172 → 163, run762 192 → 191,
   run763 186 → 185, run855 and run861 122 → 120 — the siege units' queue prices now agree (re-pinned, each
   comment carrying the delta).
6. **The tiers' unbuilt arms** (§6): `Unit::do_gather`'s and
   `Build::process`'s AI repair arms (H12), `get_econ_mod` (no caller
   in the export, so it is dead there too). Tier 1–4 parts have not been
   read; candidates wait for a tier word.
   **Built, item 3187: `Unit::find_repair_spot@00604320`** (`orders.rs`), the AI repair arm of
   `do_gather`/`think_peasant`/`build_done`/`do_repair`: a computer below difficulty 2 answers 0; from 2 up it
   swarms the nearest damaged, active, not-under-attack friendly building in range (`MOVE_TO` approach, repair
   order). Reached by tiers 2–4. Test `find_repair_spot_is_a_tier_arm_and_takes_the_nearest_damaged_building`.
   **Battery unmoved** (all eight at their pins, median 4770). Not diff-backed by a battery dump.

## 8. What is not established

- **Arm-level fidelity of every `carried at` function.** Only the
  constants (§4) and the three held-out nations and `train_time` (§3) are
  read arm by arm; the other nations' arms in the census are function-level
  presence.
- **That a `modelled` constant's reader matches the original.** The guard
  checks a field is read outside `tuning.rs`, not that it is read at the
  right place with the right truncation.
- **That a `not modelled` arm changes any trace.** Several are inert at
  the shipped value (`EGYPTIAN_SCHOLARS_FOOD`, `RUSSIAN_COMMUNISM`,
  `CHINESE_KNOWLEDGE_UPGRADES`, `GERMAN_*` at 0); the table says so in the
  rules column. The ones that bite are §7's.
- **The `Setup::` rows.** They are marked `not sim` because the harness
  takes the start state from the dump; a lobby the harness starts without
  a dump (a battery lobby's `--dump-all-start` sibling is a dump) would
  need them. Unread.
- **The multiplayer handicap and `HANDICAPS` rows.** `semaphore[0] & 4`
  is off in every capture.
- **`Setup::`, `ConquestGame::` and the interface classes' reads of
  the difficulty,** which are not sim. The never-entered ones
  are read in §9 (item 3412), each with what would enter it.

## 9. The never-entered functions — what would enter each, and what nothing can (item 3412, reading)

**How established.** A `reading` item (DECISIONS 84, 86): every function this document cites that `blind::NEVER` lists,
read off the export and, where a local could not be right, the listing. On the base `cd37c74b` that is **21**, none in
`RESIDUE`, **18** cited by this document alone (the queue counted 21/19). `AnimalData::is_seen@005d8160` is swept by item
3411 and not re-read. **Every claim below rests on a reading alone**: no capture, packet or emulator run backs it, and each
row names what kills it. The census rows of §5 are generated by `tools/nations_census.py`, so a claim that amends one is
written here and the row is left as generated.

### 9.1 Claims

| function | claim | read at | kills it |
|---|---|---|---|
| `ConquestGame::setup_scenario_bonuses@0078be90` | Conquer the World only: its one caller is `ConquestGame::build_conquest_game@00797c40:164`. Its six national arms (:399–436: Dutch 22 commerce, Russians 13 civic, Aztecs 0 and Romans 6 military, Americans 20 science, Persians 23 despotism) are a twin of `Leader::init@006e3930:539–576`, which runs them only when the CtW flag `semaphore[2] & 2` is clear (:538) — the skirmish path, which `tech.rs` carries | the export | a skirmish trace entering it |
| `ConquestGame::build_cities@00796140` | Conquer the World only: `Setup::build_empire@005abb80:23` calls it on `semaphore[2] & 2`, else `Setup::build_cities`. Its Chinese (14) arm (:133–140, the capital to `0x19f`) is `Setup::build_cities:48–54`'s twin, carried at `activate.rs` | :23–31 | a skirmish trace entering it |
| `ConquestLeader::plan_strategy@0079ea80` | The class is `ConquestLeader` (§6 wrote it under `ConquestGame::*`). Reached from `ConquestWin::post_turn_processing`, the campaign map; it reads the CtW game's difficulty, never `LeaderData::get_diff` (:491–499, :698, :790) | `INDEX.tsv`; the export | a skirmish trace entering it |
| `ScenarioFuncSet::get_difficulty@009e5320` | A script getter (function 83, `init_funcs:1619`), not the lobby's combo or the save format as §6 says: the CtW game's difficulty while the CtW window is up, else the lobby byte + 1. No native caller; no `game/ai/scripts/*.bhs` calls it, only `game/conquest/*` and `ctw_lib.bhs` | the export; the install's scripts | an AI script calling it |
| `ScenarioFuncSet::set_capital@00a00220` | A script setter (`init_funcs:3868`) no AI script calls, only 17 `game/conquest/*` scripts. Its Persian (23) arm (:71–81) strips capital and founding bits (`0x4010`) only from a city holding both; every other nation's old capital loses `0x10` alone. It also zeroes `+0x2c` of the 64 `Regions` entries (:97–106), unestablished | the export; the install's scripts | an AI script calling it |
| `CommandPackage::process_sell@009468a0` | The Nubian (4) arm is the market gate `has_tribe_bonus(4) \|\| has_preq(BUY_SELL)` (:51); it lifts the tech, never the Market. Only the human's Sell buttons issue the command: `Options::exec@007188c0:1157–1165` (flags 0, 1, 2 for one lot, 500, all) → `LeaderOut::issue_sell@006d3eb0` → `CommandManager::issue_sell@00942a80` (`'/'`). The computer sells through `do_sell`/`action_sell` directly; no capture has clicked Sell, and the tracer's `@sell` (item 3426, run1031) calls `LeaderOut::issue_sell` as the button does. **The census's `carried at` is the emulator sweep alone**: `crates/sim` has no `process_sell`, and the harness drops `Command::Sell` as "the market is not modelled" (`rondata/src/input.rs:1189`), stale beside `ai_make.rs`'s `do_sell`, `do_buy` and `calc_market_prices` | the export; the tree | a human Sell whose `LEADERS` buckets move other than `do_sell`'s |
| `IFaceMarket::on_redraw@00812d20`, `IFaceMarket::setup_selected@008134d0` | Interface only, through vslots `+0xb8` and `+0x4f8` while a Market is selected: the needs-tech or embargo text, and the Buy/Sell buttons per good (Sell at stock ≥ 100, Buy at price ≤ wealth), behind the same Nubian gate. Neither writes sim state | the export | a sim field written by either |
| `ScenarioFuncSet::buy_good@009fb790`, `sell_good@009fb880`, `can_buy_sell@009fb970`, `find_sell_price@009fb9e0`, `find_buy_price@009fbab0` | Script functions on the one global interface (`init_funcs@009c7570:3466–3477`), so "the scenario trigger set" is the class, not the reason: **no shipped script calls them** but `game/conquest/Alexander/sogdiana.bhs:499`'s `find_sell_price`. Each is one `do_buy`/`do_sell` or one `calc_market_prices@006dc2a0` read (ported at `ai_make.rs:715–767`) behind the Nubian gate, with a 1-based player; `can_buy_sell` omits the embargo test | the export; the install's 365 `.bhs` | a shipped skirmish script calling one |
| `LeaderData::has_bonus@0047df90`, `LeaderData::has_bonus@006e0b80` | Two byte-identical copies of the Nubian (4) market gate, told apart only by the translation unit a call binds to (`llvm-objdump` call targets). `0047df90`'s one caller is `Options::describe@00724460` (four sites, tooltip text): interface only. `006e0b80`'s is `LeaderData::can_buy_sell@006d53e0`, whose one sim path is the human's Buy (`Options::exec` → `issue_buy` → `process_buy` → `action_buy`); `gain_tech`'s `action_sell` is dead (every good's `<OBS>` is `disable`), and the computer's `use_market` writes the gate inline, carried at `ai_make.rs:521–528` | the listing | a computer buy entering `006e0b80` |
| `LeaderData::get_econ_mod@006d6660` | **Dead, and not "an early return at 0" as §6 had it**: the listing (`6d6660–6d669c`) returns `0.5` at difficulty 0, `0.75` at 1 and `1.0` above, and `1.0` for a human (`leader_flags & 4`) or under `semaphore[0] & 4` (the floats at `0xb694c0`, `0xb694dc`, `0xb69514`, read off the PE) — `get_mod_resource_cap`'s tier shape. No call and no four-byte pointer to it in the image | the listing; the PE | a trace entering it |
| `LeaderData::get_vehicle_speed_upgrade@006da8a0` | Counts each of `VEHICLES_FASTER_1..3` `has_preq` answers; its inlined Nubian/`BUY_SELL` arm cannot fire, since `BUY_SELL` is outside the range. Its one caller is `ObjectData::train_time@006508c0:262`, for a type with mask `0x200000` and not `0x1000`, applied as `(10 − n)/10`. The port (`tech.rs:483`, `lib.rs`) agrees, but asks one stored prerequisite per bonus where the original asks the bonus's whole PREQ list. **Why no pinned trace trains such a type is unestablished** | :262; the export | a `0x200000` unit queued with one upgrade held whose queue time is not ×9/10 |
| `LeaderOut::warn_resources@006d3650` | Called when a press cannot be paid: `Options::exec@007188c0:1677` (a build or train button), `Options::do_declare_war@007216b0:121` and `Group::action_spell:685`. Returns unless the leader is the console player (:65), then writes only the flash, a message line and a sound; the Lakota (19) arm is the food-advice string (:370). No sim state; the hook at `action_spell_view.rs:135` and the sweep's recording stub are right to drop it | the export | a sim field written by it |
| `TypeData::get_desc_parsed@006626e0` | Tooltip text, nine interface and CtW callers; the British (11) arm is the `TAX_1..4` number (:128–146) | the export | a sim caller |
| `Object::disband@006455c0` (its national arms; the rest is `docs/AI.md`'s) | Lakota (19, :97–108): with `LAKOTA_RAZE_PRICE ≠ 0` a Lakota building disbanded for any reason refunds its whole price — the constant is a gate, not an amount — carried by `disband.rs`'s `SimDisband` for buildings, without the console message. Aztec (0, :144–153): a disbanded unit with `unit_masks & 0x4000` pays `Unit::plunder` to its last damager (`+0xa9`) when that damager is an Aztec; the sweep follows it, the sim disbands no unit, and no writer of `0x4000` was found (unestablished). `SimDisband::price` returns one price for the full arm (last argument −1) and the partial (`orig_type`), a possible disagreement. Every caller is an object's removal — a delete, a captured city, a duplicate wonder, a stranded site, a raze — and no pinned trace takes one | the export; the tree | a Lakota building deleted under its price whose `LEADERS=1` buckets gain less than its whole price |

### 9.2 Captures owed (the forty-eighth pass books from this table)

All at `cover=1`; the frame is the action's. `CM` is `[Misc Logging] COMMANDMANAGER=1`'s `process_*` line; `L1` is
`[End Frame] LEADERS=1`'s buckets.

| function | ini category · field | lobby | action · frame |
|---|---|---|---|
| ~~`CommandPackage::process_sell@009468a0`~~ | `CM` `process_sell`; `L1` `bucket.timber`, `bucket.wealth`; `Game::log_data`'s `market` | a Nubian human (any pair's lobby with the human Nubian) | **entered** on 620 of run1031 (item 3426): the golden start's Nubian Market, the tracer's `@sell 0 1 0`; `process_sell 1 621`, and block 622's timber bucket 1241 → 1141, wealth 1113 → 1167, run1031b's to the value |
| ~~`LeaderData::has_bonus@006e0b80`~~ | `CM` `process_buy`; `L1` buckets | the same lobby | **entered** on 640 of run1031 (item 3426), `@buy 0 1 0` before Coinage: `process_buy 1 641`, block 642's timber 1142 → 1242, wealth 1168 → 1107 |
| ~~`LeaderData::get_vehicle_speed_upgrade@006da8a0`~~ | `[End Frame] BUILDS=6` queue time | any lobby | a `0x200000`-masked land unit queued with one `VEHICLES_FASTER` held — **entered** on frame 582 of run1016, the peace pair's lobby (item 3417, `docs/AI.md` §297), unasked; the queue-time field is not read |
| `LeaderOut::warn_resources@006d3650` | the message line (no sim field) | any lobby | the human presses a train button it cannot afford |
| `Options::do_declare_war@007216b0` | `L1` the diplomacy state; `CM` | any lobby with the computer allied or at peace | the human's declaration of war (`afford_dow`; :119 warns when it cannot be paid) |
| ~~`LeaderOut::issue_sell@006d3eb0`, `CommandManager::issue_sell@00942a80`~~ | as `process_sell` | as `process_sell` | **entered** on 620 of run1031 (item 3426): the DLL calls the leader issuer, the button's own call |
| ~~`LeaderData::can_buy_sell@006d53e0`~~ | as `has_bonus@006e0b80` | as `has_bonus@006e0b80` | **entered** on 640 of run1031 (item 3426) |
| `Object::disband@006455c0` | `CM` `process_disband`; `L1` buckets | a Lakota human | delete a completed Lakota building; the Aztec arm needs an Aztec damaging a `0x4000` unit that is disbanded |

### 9.3 Residue proposals

**Sixteen**, each with its reason; none is added to `RESIDUE` by this item.

| function | why no skirmish capture reaches it |
|---|---|
| `ConquestGame::setup_scenario_bonuses@0078be90`, `ConquestGame::build_cities@00796140`, `ConquestLeader::plan_strategy@0079ea80`, `ConquestGame::build_conquest_game@00797c40` | Conquer the World only (`semaphore[2] & 2`), cut from v1 |
| `ScenarioFuncSet::get_difficulty@009e5320`, `set_capital@00a00220`, `buy_good@009fb790`, `sell_good@009fb880`, `can_buy_sell@009fb970`, `find_sell_price@009fb9e0`, `find_buy_price@009fbab0` | script functions no shipped skirmish script calls; only `game/conquest/*` does |
| `IFaceMarket::on_redraw@00812d20`, `IFaceMarket::setup_selected@008134d0`, `LeaderData::has_bonus@0047df90`, ~~`TypeData::get_desc_parsed@006626e0`~~ (entered: run1029 on 511, `MouseOver::set_good`'s resource tooltip under the resting cursor; off `RESIDUE` with item 3425) | interface text and buttons; no sim state |
| `LeaderData::get_econ_mod@006d6660` | no caller and no pointer in the image |

`LeaderOut::warn_resources@006d3650` is not proposed: an unaffordable press reaches it in a skirmish, so §9.2 owes it a
capture, though it writes no sim state.
