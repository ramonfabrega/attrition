# ECONOMY.md versus `crates/sim` — 2026-09-05

Reader: Opus, docs-versus-code pass (not a blind reading: this reader saw the
document and the implementation, and nothing else).

Checked: 112 rules across §"The six resources" – §"Support was removed". The
three census sections (run59, run42, run60) and §"Second reading" were read as
evidence, not as rules. Rows below are the ones where the code and the document
disagree; everything not listed here was found faithful.

Blind marks come from `/Users/rf-studio/ron-audit-scratch/blind-all-68traces.txt`.
Only three functions ECONOMY.md cites by address are on it:
`ObjectData::is_dock@004711e0`, `Leader::process@006b88b0`,
`LeaderData::get_fishermen@006d6e80`.

## Rows

### R1 — Dutch interest is specified in full and implemented nowhere

| | |
|---|---|
| document | ECONOMY.md §"Dutch interest", "Between the cap and the `income` capture sits the one term that reads the *stockpile* rather than the rate" |
| code | `crates/sim/src/economy.rs:713` (`fn pay`), lines 734–749 |
| document says | for a Dutch player and any resource but knowledge, `excess = bucket[t] - start`; if positive, `rate += (DUTCH_INTEREST * excess / 100) * 16`, then `rate = min(rate, DUTCH_INTEREST_CAP * 16 + resource_cap[t])`, then `rate = min(rate, 16000)` — all between the cap clamp and the `income` capture |
| code does | goes straight from the cap clamp to `ledger.income[i] = rate`. There is no Dutch arm and no 16000 clamp; `Holdings` has no `dutch` field and `Tuning` has no `dutch_interest` or `dutch_interest_cap` slot at all |
| difference shows | `LEADERDATA`'s `income` and `bucket` for a Dutch player, from the first frame their stockpile exceeds `STARTING_GOODS`. No capture on disk has a Dutch player (run40's AI is `tribe 11` British, the human `tribe 4` Nubian), so nothing on disk reaches it |
| reached | reached (`Leader::do_gather` runs every frame in every trace; the Dutch arm itself is unreached) |

`grep -rn "dutch_interest" crates --include='*.rs'` answers only
`crates/sim/src/orders.rs:123`, a comment citing the constant by name from
another mechanic's document. `Tuning` (`crates/sim/src/tuning.rs`) has
`dutch_fort_speed`, `dutch_fort_placement` and `dutch_free_light_ship` and
neither of the two interest constants, so this is not a wiring gap — the
constants were never loaded. The 16000 ceiling is the only clamp in the
pipeline that exists inside one nation's branch, so a reader adding this later
must not hoist it.

### R2 — the knowledge tech-cost penalty is not in `pay`

| | |
|---|---|
| document | ECONOMY.md §"Paying it out", "`if t == KNOWLEDGE and tech_cost > 4: rate = rate * 3/4 if tech_cost < 7 else rate / 2`" |
| code | `crates/sim/src/economy.rs:751` (`fn pay`) |
| document says | after the handicap, a knowledge rate is cut to three quarters at tech-cost setting 5–6 and halved from 7, truncating toward zero |
| code does | applies `h.handicap` and nothing else; there is no tech-cost term, and `Holdings` carries no tech-cost field. `crates/sim/src/ai.rs:968` has a `tech_cost` (default 3) but it is the AI's lobby knob, read by `ai_research`, and `economy` never sees it |
| difference shows | `LEADERDATA`'s `bucket[3]` and `leftover[3]` divergence beginning on the first frame a knowledge rate is non-zero, in any capture whose `rise.ini` sets tech cost above 4. Every capture on disk runs the default (setting 3), so no capture reaches it |
| reached | reached |

Two steps, not one — the document is explicit that a reader who writes it as a
single threshold gets setting 5 and 6 wrong. Nothing in `crates/sim` implements
either step, so there is no half-implementation to correct; this is an absence.

### R3 — the fast-economy option and `ai_speed` are missing from the income multiplier chain

| | |
|---|---|
| document | ECONOMY.md §"Paying it out", "`if <fast-economy game option>: rate = rate * 3/2` / `if ai_speed > 1: rate = rate * ai_speed`" |
| code | `crates/sim/src/economy.rs:751`–`753` (`fn pay`) |
| document says | both sit between the handicap and the accumulator, in that order, and `ai_speed` is a plain multiplier on income |
| code does | neither term exists in `pay`. `Sim::ai_speed` (`crates/sim/src/lib.rs:870`) is live and is read by `production::accel` (`production.rs:270`), `air.rs:160` and the AI cadences (`ai.rs:919`), so the global is in the simulation — income is the one documented consumer that does not read it |
| difference shows | `LEADERDATA`'s `bucket`/`leftover` on every good from frame 1 of any run started with the `ai speed increase` cheat or the fast-economy lobby setting. No capture on disk uses either |
| reached | reached |

The document flags this itself — "`ai_speed` appears here as a plain multiplier
on income, exactly as it appears in `Guy::move` … Two subsystems now read the
same global". Only one of the two does in this crate.

### R4 — the infinite-resources short-circuit is not implemented

| | |
|---|---|
| document | ECONOMY.md §"Paying it out", "`if <infinite-resources option>: bucket[t] = 99999; escrow[t] = 0; continue`" |
| code | `crates/sim/src/economy.rs:715`–`732` (`fn pay`, the head of the per-resource loop) |
| document says | with `starting_resources == 8` the stockpile is **assigned** 99999 and the escrow zeroed every frame per resource, ahead of everything else in the loop; the literal is `0x104be` as stored, 99999 once the `bucket` XOR mask comes off |
| code does | the loop's first test is `!h.available[i]`, then the rate; there is no option arm, and `Holdings` has no `starting_resources` |
| difference shows | `LEADERDATA`'s `bucket` pinned at 99999 on every good and frame in a capture whose lobby sets infinite resources. No capture on disk does — dead in every run so far |
| reached | reached |

### R5 — `base_rate` has no representation at all

| | |
|---|---|
| document | ECONOMY.md §"Paying it out", "`rate = resources[t] - support[t] + base_rate[t]`"; §"The state it touches", "`base_rate` @ +0x4b0 … Added to the rate before the cap … stored `<< 4`" |
| code | `crates/sim/src/economy.rs:724` (`let mut rate = ledger.rate[i];`) |
| document says | `do_gather`'s rate is the assembled rate minus support plus `base_rate`, and `base_rate` is the only way a *negative* rate can arise — which is what the negative-rate guard three lines below exists for |
| code does | takes `ledger.rate[i]` alone. Neither `Ledger` nor `Holdings` has a `base_rate` field, so the negative-rate guard (`economy.rs:728`) is unreachable from within the simulation and is exercised only by the hand-built ledger in `a_negative_rate_is_shown_and_never_charged` (`economy.rs:1120`) |
| difference shows | no observable — `ScenarioFuncSet::set_base_rate` is the field's only writer, no capture is a scenario, and the document says so |
| reached | blind (`ScenarioFuncSet::set_base_rate@009fbb80` is on the never-entered list, cited by `audit/2026-08-20-economy.md`) |

Recorded because the *comment* at `economy.rs:722`–`723` explains away `support`
and says nothing about `base_rate`, so the omission reads as complete when it is
not. The document's own §"Paying it out" note — "the only way to reach one is a
negative `base_rate`" — is the sentence this code cannot honour.

### R6 — the `over_cap == 2` threshold is 15968 in the code and 15983 in the document

| | |
|---|---|
| document | ECONOMY.md §"Paying it out", "`over_cap[t] = 1 + (resource_cap[t] > 15983)`" |
| code | `crates/sim/src/economy.rs:736`, `if cap > CAP_CEILING * RATE_SCALE - RATE_SCALE` |
| document says | the uncapped marker is `resource_cap > 15983`, i.e. `999 × 16 − 1` |
| code does | tests `cap > 999 × 16 − 16 = 15968` |
| difference shows | no observable — `commerce_cap` only ever returns a multiple of 16 (`economy.rs:647`, `cap.clamp(0, CAP_CEILING) * RATE_SCALE`), and there is no multiple of 16 strictly between 15968 and 15984, so the two predicates agree on every value the pipeline can produce |
| reached | reached |

Cheap to state and cheap to fix; left as a row because a later change that makes
a cap something other than a multiple of sixteen (a `bonus_cap` added after the
`<< 4`, say) silently parts the two.

### R7 — the territory tax carries none of its three modifiers

| | |
|---|---|
| document | ECONOMY.md §"The territory tax", "Three modifiers sit on it, all read: the British scale `TERRITORY_TAXES` by `(BRITISH_TAXATION + 100) / 100` … a Conquer-the-World conquest bonus … and the Mongols additionally take **food**" |
| code | `crates/sim/src/economy.rs:605` (`fn territory_tax`) |
| document says | the British term as shipped **doubles** the territory tax; the Mongol term is `num_nations * territory * 800 / land_size / MONGOL_NOMADIC_FOOD` into food, and is the one term in the economy that reads the player count |
| code does | `h.territory * t.territory_taxes[level] * RATE_SCALE / h.land_size` into wealth and nothing else. `Holdings::british` is live and is read by `commerce_cap` (`economy.rs:636`) but not here; there is no `british_taxation`, `mongol_nomadic_food` or `ctw_missionaries_bonus` slot in `Tuning` |
| difference shows | `LEADERDATA`'s `resources[2]`/`income[2]` for the British AI in run40/run59/run60 — except that R8 makes the whole line zero on **this** side and the original's own territory tax is zero too through all 5,400 frames of run60, so nothing parts until a taxation level is reached on either side |
| reached | reached |

The British half is the one that matters: `crates/sim/src/nations.rs` now sets
`Nation::british` from the dump's own `LeaderData::tribe` (11 on run40's AI), so
the flag the doubling would key off is already true in three captures on disk.

### R8 — the five bonus levels the economy indexes by are hard-wired to their floor, so the territory tax is permanently zero

| | |
|---|---|
| document | ECONOMY.md §"The territory tax", "`TERRITORY_TAXES[taxation_level]` … ships as `0, 50, 100, 200, 300` percent by taxation level"; §"The enhancers", "`GRANARY_BONUS[granary_level - 1]` … Shipped, `GRANARY_BONUS` is `20 50 100 200 250` percent by level"; §"What a city gives", "`(SCHOLAR_RATE[level - 1] * 16) >> 8`" |
| code | `crates/sim/src/holdings.rs:145` (`fn Levels::for_player`), returning `Levels::BASE` (`holdings.rs:133`) |
| document says | the levels move with the player's bonus types — taxation 0..=4 (`TAX_1..4`), granary 1..=5, lumber mill 1..=4, smelter 1..=4, university 1..=6 |
| code does | `Levels::for_player` ignores its `who` (it only `debug_assert!`s the index) and answers the constant `BASE`: granary 1, lumber 1, smelter 1, university 1, **taxation 0**. `assemble_holdings_with` (`holdings.rs:353`) never writes `Holdings::taxation` at all, so it stays at its `Default` zero, and `territory_tax` (`economy.rs:610`) therefore multiplies by `TERRITORY_TAXES[0] = 0` for the life of every game |
| difference shows | `LEADERDATA`'s `resources[2]` and `income[2]` on the frame after either player takes Taxation, and `resources[3]` on the frame after a University upgrade. **Not live in any capture on disk**: run60 compares `resources` on six goods for both players over 5,400 frames and leaves no territory-tax row (ECONOMY.md §"What run60 leaves" lists only the merchant seam from 4992, the accumulator pair, `resource_cap` at 2958 and item 156), so neither leader reaches a taxation level inside the window — but run60's own `resource_cap` step at 2958 says the commerce line *is* being researched there, and the taxation ladder sits beside it |
| reached | reached |

`crates/sim/src/holdings.rs:102`–`113` names this a seam and says the natural
home is `city::Nation`; it is recorded here because ECONOMY.md states the five
indexed rules as live arithmetic and does not say the index never moves. The
taxation one is the sharpest: it is not "a level too low" but a whole documented
income line (§"The territory tax", the economic mirror of `docs/ATTRITION.md`)
that can never be non-zero.

### R9 — step 5's filter admits any gather building without a city, not just the camp and the mine

| | |
|---|---|
| document | ECONOMY.md §"Assembling the rate" step 5, "Buildings outside a city — the Woodcutter's Camp (`0x1a2`) and the Mine (`0x1a3`) … The filter is: the object is active, its `city` link is negative, it is not neutralized, and its type is one of those two" |
| code | `crates/sim/src/holdings.rs:303` (`fn gather_sites_outside_cities`), lines 310–317 |
| document says | four tests, and the fourth is an identity test against exactly two build types |
| code does | takes `self.gather_good(b)` — which answers for the farm, camp, mine, university, oil well and platform alike (`holdings.rs:167`) — exempts the two oil types from the city-link test, and admits everything else whose `city` is `None`. A farm or a university with no city link would be paid here where the original pays it nothing. "Not neutralized" is not tested at all: `counts_for_economy` (`holdings.rs:191`) is `alive && active` |
| difference shows | `LEADERDATA`'s `resources[0]`/`resources[3]` on the frame a cityless farm or university becomes active, and `resources[*]` for a neutralized camp on run40/run59/run60. No capture on disk is known to have either, so this is a latent widening rather than a live one |
| reached | reached |

The oil exemption is right and is the document's step 4; the widening is
everything else. `Sim::gather_good`'s own doc-comment says it stands in for
`BuildTypeData::get_good`, which is a six-way table — but step 5's predicate in
the original is not `get_good`, it is two identity tests, and the code has
substituted the wider one.

### R10 — the rate path applies no slot bound, so a mine, camp or university can be paid for more gatherers than it has room for

| | |
|---|---|
| document | ECONOMY.md §"What a city gives", "The flat branch is also the only one that does not bound gatherers by slots: it adds `per × n` for whatever `n` it was handed, where the mine, woodcutter and university branches all take `min(slots, n)` — and the woodcutter additionally caps at twice its `total_gather_access`" |
| code | `crates/sim/src/economy.rs:441`–`449` (`fn city_rates`) and `crates/sim/src/holdings.rs:277` (`fn site_of`) |
| document says | three of the four branches clamp the gatherer count against the building's surveyed slot count before multiplying |
| code does | `Site` (`economy.rs:152`) carries `gatherers` and no slot count; `site_of` fills it from `num_gatherers(b, true, true)` unclamped, and `city_rates` multiplies `rate * site.gatherers` with no `min`. `BuildData::gather_max` is stored (`lib.rs:1022`) and surveyed (`gather.rs:446`), so the number exists — the rate path just does not read it |
| difference shows | `LEADERDATA`'s `resources[1]`/`resources[3]`/`resources[4]` on a frame where a camp, mine or university holds more workers than slots. Reachable only past the order layer's own gate: `orders.rs:3923` refuses a gather order when the chain is already at `gather_max`, and a university's scholars come through the garrison instead (`orders.rs:4012`), which that gate does not cover |
| reached | reached |

Marked `UNSURE:` on reachability, not on the disagreement. What would settle it:
whether `Build::garrison_max` for the university is 7 in the shipped data — if
it is, the garrison arm cannot overfill and the whole row is dead; if it is
higher, a university over seven scholars overpays here and does not there.

### R11 — the flat branch's per-gatherer rate is a constant where the document says it is a survey

| | |
|---|---|
| document | ECONOMY.md §"What a city gives", "Its per-gatherer rate is not a constant: it walks the building's whole footprint … summing each tcoord's `num_make` richness for the land under it, doubling a river tcoord by `RIVER_RESOURCE_VALUE`, and **skipping any tcoord owned by somebody who is neither the owner nor an ally**" |
| code | `crates/sim/src/economy.rs:414` (`fn per_gatherer`), lines 420–421 |
| document says | 160 (a farm) and 560 (an oil well) are *one case* of the flat branch, not the rule; a farm straddling an enemy border earns strictly less and an oil well on a rich site earns more |
| code does | returns `(peasant_rate >> 8) * 16` for every non-knowledge, non-oil resource and `(oil_rate >> 8) * 16` for oil — the single case, unconditionally. There is no `num_make` read and no `river_resource_value` in `Tuning` |
| difference shows | `LEADERDATA`'s `resources[0]` for a player whose farm footprint overlaps enemy-owned tcoords, and `resources[5]` for any oil well at all once oil is available. run60 would show it on the frame the border sweep first puts a hostile tcoord under a farm |
| reached | reached |

`economy.rs:404`–`413` states this deferral in its own words, so the code and its
comment agree; the row exists because the document's §"What a city gives" table
prints `(PEASANT_RATE >> 8) << 4` = 160 in the **Mine** row and the footprint sum
in the **Flat** row, and only the mine row is what the code implements — for both.

### R12 — the Forbidden City and the CEO hero, and the reach both of them have

| | |
|---|---|
| document | ECONOMY.md §"What a city gives", "The `FORBIDDEN_CITY_GATHER` percentage — 25%, applied as `(pct + 100) * out[t] / 100` to all six — runs **immediately after the building walk and before everything below it**"; "a CEO hero (`TypeIndex 0x165`) standing in the city scales all six by `THECEO_PRODUCTION_BONUS`, with the same reach" |
| code | `crates/sim/src/economy.rs:433` (`fn city_rates`) |
| document says | two multipliers sit between the building walk and the flat `CITY_GATHER`, and the `FORBIDDEN_CITY_BASE_GATHER` substitution replaces `CITY_GATHER` only in the slots that already pay (food and timber as shipped) |
| code does | neither exists. `economy::City` has no Forbidden City or CEO field, and `Tuning` has no `forbidden_city_*` or `theceo_*` slot |
| difference shows | `LEADERDATA`'s six `resources` for the owner of a Forbidden City from the frame it activates. No capture reaches the Medieval wonder line |
| reached | reached |

Recorded chiefly for the **reach**, which is the part a later implementer will
get wrong: the document twice says both multiply the building walk *only* — not
the flat city gather, not the taxes, not the literacy — and `city_rates`'s
current shape (one `out` array accumulated straight through) has no seam at that
point to hang them on. Adding them later means splitting the accumulator, not
adding a line.

### R13 — the nation terms inside the city and branch arithmetic

| | |
|---|---|
| document | ECONOMY.md §"What a city gives", "the Japanese scale food by `(JAPANESE_FISHING_BOATS + 100) / 100`, and the Egyptians add `EGYPTIAN_FARM_WEALTH * 16` of wealth per farm … the Inca earn `gatherers * INCA_WEALTH_PER_MINER * 16` … Then the Romans add wealth per city and the Germans add food, timber and metal per city"; §"What a city gives" table, "160, or `IROQUOIS_FOOD * 16` for the Iroquois' food" |
| code | `crates/sim/src/economy.rs:433` (`fn city_rates`), `economy.rs:414` (`fn per_gatherer`) |
| document says | six nation terms live inside the city walk, one of them (the Inca's, when `INCA_WEALTH_PER_MINER` is negative) redirecting the player's whole metal rate into wealth back in `Leader::calc_gather` |
| code does | none of the six. `Holdings` carries `egyptians`, `french` and `inca` as booleans but reads them only in `commerce_cap` (`economy.rs:639`–`644`); `Nation::germans` is read only by the completion bonus (`city.rs:1494`) |
| difference shows | `LEADERDATA`'s `resources[2]` for an Egyptian or Inca player, `resources[0]` for the Japanese or the Iroquois, `resources[0]/[1]/[4]` for the Germans, `resources[2]` for the Romans — on the first recompute frame. No capture on disk has any of those six nations (run40: British AI, Nubian human) |
| reached | reached |

The Inca's negative-constant arm is the one worth carrying forward verbatim: the
document records it as a whole-rate metal→wealth redirect in `Leader::calc_gather`
rather than a per-mine term, and there is no place in `assemble` today where a
term that reads a *finished* rate could go.

### R14 — five of the thirteen assembly steps are absent

| | |
|---|---|
| document | ECONOMY.md §"Assembling the rate", steps 2, 8, 9, 10 and 12 |
| code | `crates/sim/src/economy.rs:502` (`fn assemble`) |
| document says | step 2 nation flat bonuses (Lakota food per unit, American barracks gather); step 8 `CAPITALISM_OIL_PROD * 16` into oil; step 9 one `calc_rare` per bit of `rare_conquest`; step 10 Coffee scaling all six; step 12 `calc_resource_bonuses` — Russians/oil, Pyramids/food, Colossus/wealth, Hanging Gardens flat knowledge, Angkor/metal, Taj/wealth, Eiffel/oil, Tikal/timber, then `GLOBAL_PROSPERITY` over the five capped goods, then the CtW per-good bonus |
| code does | implements steps 1, 3, 4, 5, 6, 7 and 11 in the document's order and stops. The whole of step 12 is absent, and with it the one ordering fact the document goes out of its way to state — that Hanging Gardens is a flat **addition** where its eight neighbours are percentages, and that `GLOBAL_PROSPERITY` skips knowledge |
| difference shows | `LEADERDATA`'s `resources` for a wonder holder on the recompute frame after the wonder activates. Nothing on disk: no capture reaches a wonder |
| reached | reached |

Steps 9 and 10 are dead in a skirmish by the document's own reading (step 9 "is
reached only when no other initialised in-game leader exists"); steps 2, 8 and 12
are not.

### R15 — the obsolete redirect is absent

| | |
|---|---|
| document | ECONOMY.md §"The obsolete redirect", "`if obs_prod_good >= 0: out[obs_prod_good] += out[t] * obs_prod_rate >> 8` / `out[t] = 0`" |
| code | `crates/sim/src/economy.rs:534`–`535` — `assemble` ends at the territory tax |
| document says | the last thing `calc_gather` does is walk the six goods for one that is no longer available but whose prerequisite the player still holds, move its rate at `OBS_PROD_RATE` (8.8) and **zero it either way** — the zeroing is not conditional on a redirect target existing, and the shift is the sign-corrected `(x + (x >> 31 & 0xff)) >> 8` |
| code does | nothing; there is no `obs_prod_good`/`obs_prod_rate` in `Tuning` and no post-tax pass |
| difference shows | no observable — dead path. The document says so ("Nothing in a standard game makes a basic resource obsolete"), and §"Availability" adds the structural reason: `has_preq`'s `obs` is never set, so `available` and `discovered` are the same test for a good and `Redirects::of` never reaches the obsolete table |
| reached | reached |

### R16 — the commerce cap stops after the nation percentages

| | |
|---|---|
| document | ECONOMY.md §"The commerce cap", the seven-line block: `DIAMONDS_COMMERCE`, the flat wonder additions, `REPUBLIC_COMMERCE_BONUS{,2,3}` "by republic level", `bonus_cap[t]`, then the clamp and the `<< 4`; plus the two overrides that skip the whole body |
| code | `crates/sim/src/economy.rs:630` (`fn commerce_cap`) |
| document says | after the British and per-nation percentages come the Diamonds percentage (rare bit 22, from **either** `rare` or `rare_conquest`), then the flat wonder additions (Pyramids food+wealth, Colossus timber+wealth, Taj wealth, Eiffel oil, Kremlin food/timber/metal/oil but *not* wealth, Tikal timber, Angkor metal via `resource_cap_add`), then the republic term — **the highest tier held, not the sum** — then `bonus_cap`; and the Virtual Reality bonus overrides the whole body to 999 |
| code does | British, one per-resource nation term, `+ h.bonus_cap[r]`, clamp, `× 16`. No Diamonds, no wonders, no republic, no Virtual Reality override. Knowledge's 999 override is implemented (`economy.rs:631`) |
| difference shows | `LEADERDATA`'s `resource_cap` — the field run40, run42, run59 and run60 all print and all four diff tests already compare (`crates/rondata/src/diff.rs:9906`, `:10177`, `:10287`, `:14576`) — on the frame a player holds Diamonds or researches a republic |
| reached | reached, except `LeaderData::resource_cap_add@0047da40`, which is **blind** |

`commerce_cap`'s own doc-comment (`economy.rs:622`–`625`) says the wonder, rare
and republic terms "are not here"; the document's §"The commerce cap" states
them as the pipeline without saying which end is built. The republic term's
"highest tier rather than summing" is the predicate most likely to be got wrong
later, and it is stated only in the document.

### R17 — the Porcelain Tower's market-tax modifier is missing from `get_taxes`

| | |
|---|---|
| document | ECONOMY.md §"What a city gives", "The market term has one modifier, which lives inside `get_taxes` rather than in the wonder layer: a player holding the **Porcelain Tower** takes `MARKET_TAXES * (PORCELAIN_MARKET + 100) / 100` instead. It is per city with a market, so it scales with how many the player has" |
| code | `crates/sim/src/economy.rs:468` (`fn taxes`), lines 470–472 |
| document says | the modifier is inside the taxes function, applied to the `MARKET_TAXES` term alone, once per city holding a market |
| code does | `n += t.market_taxes` flat; `taxes` takes only `(&Tuning, &City)` and has no way to see a player-level wonder |
| difference shows | `LEADERDATA`'s `resources[2]` for a Porcelain Tower holder, scaling with market count. No capture reaches it |
| reached | reached |

Called out separately from R12/R14 because the document is explicit that this one
is **not** in the wonder layer — it is inside `CityData::get_taxes`, so the
signature `taxes(&Tuning, &City)` is the thing that would have to change, and a
later implementer working through `calc_resource_bonuses` alone would miss it.

### R18 — a change in the rare mask recomputes nothing but unit stats

| | |
|---|---|
| document | ECONOMY.md §"Rare resources pay through merchants, not through ownership", "when it differs the player's population cap is recomputed, the borders are marked for a redraw if the rare that changed is one of the territory-affecting ones, and two more dirty bits go up" |
| code | `crates/sim/src/lib.rs:2691`–`2695` |
| document says | three consequences follow a `rare_owned \| rare_conquest` change: pop-cap recompute, a conditional border redraw, and two further dirty bits — beside the `0x4000000` unit-stats flag |
| code does | sets `ledgers[who].rare = rare` and raises `unit_stats_dirty[who]`, and nothing else. `Sim::recompute_pop_caps` (`lib.rs:1420`) has three callers — `apply_gained` (`lib.rs:2345`), `city.rs:804` and tests — none of them the rare mask |
| difference shows | `LEADERDATA`'s pop cap on the frame a merchant settles on a pop-cap rare, and the border cells around a territory rare. Today it is **no observable**: `cost::pop_cap` reads `m.bonuses` (`lib.rs:1422`) and nothing wires a rare into that array, so the recompute would be a no-op even if it were called |
| reached | reached (`Leader::gather`); `Leader::process@006b88b0`, the function the document cites for the same-frame `0x4000000` handling, is **blind** |

**Half closed, 2026-09-06 (item 117).** The border redraw is landed: the
arm is one bit wide — Gems, `rare.ptr[2] >> 7` — and `Sim::tick` now calls
`sync_territory` when it moves, which `run80_s_gem_widens_the_ai_s_border_
by_forty_three_cells` pins against the original's own `LeaderData::
territory` at Great Lakes 23,999 (`docs/ATTRITION.md`, "Territory"). The
pop-cap recompute and the two further dirty bits are still open, and the
observable is still dead for the same reason.

The row is worth keeping despite the dead observable: the moment any rare is
wired into `Muster::bonuses` the missing call becomes a live one-frame-late bug,
and the code has no comment marking the gap.

### R19 — a player starts holding all six goods where the original grants three of them with the age

| | |
|---|---|
| document | ECONOMY.md §"Availability, and where a price lands instead", "A leader still *starts* with a hundred of each of the three, where the original starts with none: `Leader::gain_tech@006dcb60` walks the six goods on every gain and, for one whose bucket is zero and whose `goodtypes[g] + 0x30` prerequisite is the tech just gained, calls `bucket_add(g, game->starting[g])` — the starting grant arrives **with the age**, not at `Leader::init`, which zeroes all six" |
| code | `crates/sim/src/economy.rs:384` (`fn Ledger::starting`), reached from `lib.rs:1117` and `lib.rs:1186` |
| document says | `Leader::init` zeroes all six; the grant for a good arrives on the tech gain that unlocks it, and only when that good's bucket is still zero |
| code does | `Ledger::starting` assigns `t.starting_goods` — `[200, 200, 100, 100, 100, 100]` (`tuning.rs:791`) — to all six buckets at construction, and no path in `Sim::gain_tech`/`apply_gained` (`lib.rs:2338`) adds a starting grant on a gain |
| difference shows | `LEADERDATA`'s `bucket` on goods 3, 4 and 5, 100 here against 0 there, on both players from frame 1 — already asserted, as item 156, by `diff::tests::run40_s_census_…` (240 rows) and visible on all 5,400 frames of run60 |
| reached | reached |

The document books this itself under "What this does not close", and calls it
inert (an unavailable good is never charged and never accrues). It is listed
here because it is a stated rule the code does not implement, and because the
inertness argument stops holding the moment the *available* half changes: the
grant is conditional on `bucket[g] == 0`, so a player who earns knowledge before
the Classical Age lands would take no grant in the original and would already
hold 100 here.

### R20 — the Porcelain Tower's `rare_owned` pass, the one thing that makes a rare pay with nobody on it

| | |
|---|---|
| document | ECONOMY.md §"Rare resources pay through merchants…", "The one exception is the Porcelain Tower, whose pass at the top of `calc_gather` sets `rare_owned` bits directly for every rare inside the player's own territory" |
| code | `crates/sim/src/rares.rs:86` (`fn gather_rares`) — `owned` is built from the unit walk alone (`rares.rs:127`–`129`) |
| document says | the pass runs at the **top** of `calc_gather`, before the unit walk, and sets bits for every rare inside the player's own territory |
| code does | no territory pass; `rare_owned` is exactly the set of goods an idle fisherman or merchant was standing on |
| difference shows | `LEADERDATA`'s `rare` mask, and every downstream consumer of it, for a Porcelain Tower holder. No capture reaches it |
| reached | reached |

`crates/sim/src/rares.rs:32`–`35` names this omission, so code and comment agree;
the row records that the *position* (top of `calc_gather`, before the walk that
clears the mask) is stated only in the document.

### R21 — four doc-comments in `economy.rs` still carry claims ECONOMY.md has retracted

| | |
|---|---|
| document | ECONOMY.md §"The six resources", "(An earlier draft said 'no rate, no cap, no accrual'; only the last is true.)"; §"The commerce cap", "**And the British term now fires.**"; §"Confidence", "The count is read and verified as of 2026-08-24"; §"Assembling the rate" step 6 and `crates/sim/src/rares.rs` |
| code | `crates/sim/src/economy.rs:288`–`291`, `:276`–`278`, `:156`–`160`, `:496`–`501` |
| document says | (a) an unavailable resource **does** have a live rate and a live cap, and only the accrual is skipped; (b) `Holdings::british` is fed from the dump's `LeaderData::tribe` and the British cap term fires; (c) the gatherer count is read and implemented in `crates/sim/src/gather.rs`; (d) step 6, the idle fisherman and merchant walk, is landed |
| code does | the behaviour is right in all four cases and only the prose is stale: `Holdings::available`'s comment says "an unavailable resource takes no part at all: no rate, no cap, no accrual" — the exact sentence the document retracted; the nation-flag comment says "nothing reads the dump's `tribe` yet, so a traced game leaves all four false", where `holdings.rs:391`–`394` reads all four from `Nation`; `Site::gatherers`'s comment says "that survey is unread", where `gather.rs` is the survey; and `assemble`'s comment lists "the idle-unit loop" among the terms it leaves out, twenty-four lines above the loop at `economy.rs:526`–`528` that adds it |
| difference shows | no observable — prose only |
| reached | reached |

Included because this pass exists to catch the document and the code drifting
apart, and a comment that contradicts the specification is the cheapest way for
the next reader to re-introduce a bug the document already closed. (a) is the
sharpest: it states a rule — no cap for an unavailable good — that
`economy::caps` deliberately does not implement, so a reader trusting the
comment would "fix" `caps` and break `resource_cap`, which four diff tests pin.

## What was checked and found faithful

Recorded so a later pass need not re-derive it: the frame order (income before
objects, `crates/sim/src/lib.rs:2669`); the three-step `gather` and the zeroed
`support`; the 8/512-frame cadence and the `who × 8` stagger
(`economy.rs:103`); `GATHER_RATE` 450 and sixteenths; the ×256 trio and their
`>> 8`; the assembly's steps 1, 3, 4, 6, 7 and 11 and their order; `calc_rare`'s
two `(BONUS_TYPE, BONUS_NUM)` pairs, the merchants bonus **replacing** the 100,
the fishermen bonus **added** to food alone, both gated on FISH/WHALES, and the
per-unit `crowd + 1` truncation (`rares.rs:115`–`126`); step 6's packed test,
`order_type == NONE` and the crowd's own packed rule (`calc_gather.rs:172`–`182`);
the two idle-latch dirty-flag writers (`orders.rs:1182`–`1187`, `:1366`–`1371`);
the whales speed arm and the same-frame `calc_unit_stats`; `trade_val` first into
wealth; `get_taxes` and `get_literacy`; the enhancer truncating per gatherer;
`CITY_GATHER` once per city; the commerce cap's British-then-nation order with
its per-percentage truncation before the `<< 4`, and knowledge's 999;
`do_gather`'s cap clamp, the `income` capture before the handicap, the handicap
table and its human zero, and the accumulator written the original's way;
`Build::activate`'s two slot blocks — the wealth slot at line 590 and the gather
slots at 1151, both with the `frame != 0 / captured / counted` gate, the German
multiplier, the flat-versus-per-slot split, `Farms::add_animals` on every exit,
and `Build::close`'s two mirrors (`city.rs:1479`, `:1522`, `:1651`, `:1660`);
`Build::init`'s branch and `find_gather_tiles`' four steps including the `4 × n`
shuffle and its `rnd` early return (`city.rs:1035`, `gather.rs:422`); the timber
`find_gather_tcoords` walk and the metal branch's stated absence;
`sync_goods_available` and the `available`/`discovered` pair (`lib.rs:2374`);
`new_rare`'s gates and the FISH/WHALES exclusion, `reveal_fog`'s two tests,
`seed_new_rares_from_fog`, and `num_rare_resources_seen` as the list's length
(`rares.rs:203`, `:291`, `ai_host.rs:306`).


## Adjudication — 2026-09-05, Opus

**Twenty-one confirmed, none struck.** Every row's code citation was
re-checked against the source; R6, R8, R9, R10, R11 and the `pay` rows
(R1–R4) were read at the site rather than only checked for existence.

The pass divides three ways.

**One row is a whole documented income line that can never fire.** R8: the
five ladders the economy indexes by are hard-wired to their floor.
`Levels::for_player` (`holdings.rs:145`) ignores its `who` — it only
`debug_assert!`s the index — and answers the constant `BASE`, whose
`taxation` is **0**; `territory_tax` (`economy.rs:605`) then multiplies by
`t.territory_taxes[0]`, which the shipped table gives as 0 per cent. So the
territory tax is identically zero for the life of every game, and the
granary, lumber mill, smelter and university ladders are all pinned at
level 1. `holdings.rs:102`–`113` names the seam; ECONOMY.md states the five
indexed rules as live arithmetic and does not say the index never moves.

**Two rows are the reader's own careful negatives, and worth keeping as
such.** R6 — the `over_cap == 2` threshold is `999 × 16 − 16` in the code
and `999 × 16 − 1` in the document — has **no observable**, because
`commerce_cap` only ever returns a multiple of sixteen and there is no
multiple of sixteen strictly between the two. It stays a row because a
later `bonus_cap` added after the shift would part them silently. R15's
obsolete redirect is a dead path. Neither is a bug today, and neither
should be written up as one.

**The rest are stated, unimplemented and unreached**, and the largest of
them are whole clauses rather than constants: Dutch interest; the knowledge
tech-cost penalty; the fast-economy option and `ai_speed` in the income
multiplier chain (`pay` scales by `handicap` and nothing else); the
infinite-resources short-circuit; `base_rate` with no representation at all
(and blind — `ScenarioFuncSet::set_base_rate@009fbb80`); the territory
tax's three modifiers; the slot bound on the rate path; the flat branch's
per-gatherer rate, a constant where the document has a footprint survey
(`economy.rs:420`, `(t.peasant_rate >> 8) * RATE_SCALE`); the Forbidden City
and the CEO hero; six nation terms inside the city and branch arithmetic;
five of the thirteen assembly steps; four of the commerce cap's terms; the
Porcelain Tower's market-tax modifier and its `rare_owned` pass; a rare-mask
change recomputing nothing but unit stats; and a player starting with all
six goods where the original grants three with the age.

**R21 is a different kind and should be taken first**, because it is cheap
and it actively misleads: four doc-comments in `economy.rs` still carry
claims ECONOMY.md has retracted, and one of them contradicts four pinned
diff tests. A stale comment at the site is worse than a stale document,
because it is what the next session reads while editing.
