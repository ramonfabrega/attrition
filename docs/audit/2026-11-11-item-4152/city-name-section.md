## 1. The city-name counter

### 1.1 Leaves

- `city_name`, +0x40c, `int`. A plain count with no scale. Slot `i` sits at
  `0xe3a79c + i·0x6eec`.

### 1.2 The rule

**Writers.** Two value stores, both in `City::generate_name (00735c90)`: the
step `+= 1` on the winner slot (listing 73614a) and the `= 1` on the owner's
own slot (7362d2). One explicit clear to 0 in `Leader::init (006e3930)`
(6e3b14). The three bulk writers of the header: `Leader::init`'s memset when
its tribe argument is negative (redundant with the explicit clear),
`Leaders::clear (006ed5d0)` (called from `Game::close (00594b80)` and
`Game::run_oos_recovery (005869e0)`, so never a reset inside play), and
`LeaderData::walk_data (006d6750)` when a save game loads. There is no
whole-record copy, and no other writer.

**The event.** `generate_name`'s only caller is `City::init`, and only when
its fourth argument is 0. That argument comes unchanged through
`Cities::init_city` from the first argument of `Build::activate`, in its city
branch (`Build+8 & 0x20`). So the counter steps on a city's activation with
first argument 0, and never with 1. The callers of `activate` that reach a
city:

- `Setup::build_cities (005ab910)` activates the start city with `(0,0,0)`
  when the game's `starting_town` option is set and `leader_flags2 & 0x80` is
  clear. Steps.
- A city completed by construction. Steps.
- `Cities::capture_city (00733380)` pushes `(0,1,1)`: capture and
  `switch_teams`. No step.
- `Wall::do_construct (006434d0)` pushes `(0,1,1)`. No step.
- `check_capture`, `defeat_by`, `find_buildings` and `cast_bribe` reach
  `activate` only on non-city buildings; `cast_bribe`'s city arm either calls
  `capture_city` or refunds and returns.
- `Leader::produce_building` reaches `activate` only at frame 0, and its
  Setup callers push no city type (none of VILLAGE 0x19e, TOWN 0x19f,
  METROPOLIS 0x1a0, FORBIDDENCITY 0x213).

**Exemptions (no step), in the order tested:**

1. The city's building `is(FORBIDDENCITY 0x213)`.
2. The game is Conquer the World (`semaphore[2] & 2`) **and** the owner's
   tribe is not barbarian (`Tribe +0x58`). Barbarians under CtW do step.
3. The tribe's cities element is missing, or its list is empty.

**The step.** Let `own` be the city's owner and `T = tribe[own]`.

- Over slots `k` in `0..8` (never 8 or 9, unrolled), with `tribe[k] == T`,
  find the largest `city_name[k]`; the comparison is strict `>`, starting
  from -1, so a tie keeps the lowest `k`. No liveness test: the scan reads
  `tribe` (+0xc) and `city_name` alone.
- `winner = k` if that largest value is `> 0`, else `winner = own`. An unused
  slot's `city_name` stays 0, so it can never be the winner.
- `prev = city_name[own]`, read before the step.
- `city_name[winner] += 1`; `n` = the winner's new value.

**The second store.** With `L` the length of the tribe's city list:

    if n < L and (n <= 1 or prev == 0) and city_name[own] == 0:
        city_name[own] = 1

The tests are signed. When the winner is `own`, `city_name[own]` is at least
1 after the step, so the store fires only when a tribe-mate took the step and
`own` had never named a city. When `n >= L`, the name is the tribe name with
a numbered suffix and no second store runs.

**Not needed for the counter:** the LFSR (taps 0x170, signed `idivl`), the
name index loop and the numbered suffix. They choose the name and write no
`city_name`. The LFSR is the maximal 8-bit polynomial; its loop ends whenever
`M >= 2` and never at `M = 1`. That matters to a name port, not to this leaf.

**Reset.** `Leader::init` alone, at game setup. `Setup::build_game` runs it
for each live slot (`leader_flags & 1`) before it reaches `build_cities`
through `build_empire`, so the start city's step lands on a zeroed count.
Nothing decrements it, and nothing folds it per frame. `ScenarioRead::
load_leader_chunk` calls `Leader::init` on every branch and drops a saved
value; a save game restores it raw through `walk_data`.

**Readers.** `generate_name` (the step itself and the name choice);
`log_data` (key `city_name`); `save_leaders_chunk` (9a791e); and the
sync checksum and the save game through `walk_data`
(`CheckSums::check_one_leader` walks `leaders.list + slot`). No reader
outside naming changes a decision.

### 1.3 In the crate

- **No counter today.** The event is `Sim::activate`
  (`crates/sim/src/cities/city.rs:2420`) calling `Sim::init_city` (`:451`) at
  `:2509`, inside the `building_is_city` branch; `activate`'s `captured`
  argument is the original's first argument. `init_city`'s only caller is
  `:2509`. The start city enters through `crates/sim/src/setup.rs:94`. Every
  `activate(.., true, ..)` outside tests is
  `crates/sim/src/cities/capture_view.rs:236`, which must not step.
- **Trap: the FORBIDDENCITY exemption is load-bearing.** The crate's
  `building_is_city` is true for FORBIDDENCITY (`city.rs:333-336`, where
  `flags & 0x20` is `is(VILLAGE)`; `crates/sim/src/vision.rs:977-980`), so a
  completed Forbidden City runs the city branch. The port must exempt it by
  type, not trust the branch.
- **No city-name list in the crate.** Exemption 3 and the `n < L` guard need
  `L` per tribe. Until the data layer carries it, a port takes every shipped
  list as non-empty and long enough that `n < L`, and records that as an
  assumption.
- **Unrelated:** `crates/sim/src/ai/ai_host.rs:445 city_name` is a synthetic
  AI label.
- **Checksum.** The leaf is `crates/sim/src/checksum/leaders.rs:98`. The held
  arm goes beside `city_mark` (`leaders.rs:1020-1033`). The port drops
  `("city_name (unheld)", 2)` at
  `crates/rondata/src/diff/checksum_lines.rs:1772` from 2 to 0 on the run-1562
  record. The verdicts give no `LEADERS_HELD_OUT` figure.

### 1.4 Reached by a plain game

A skirmish against the computer reaches the start city (when `starting_town`
is on), every city completed by construction, and a completed Forbidden City
(the exempt path). Capture reaches `activate` but never steps. The tribe-shared
winner and the `= 1` store need two leaders of one nation. Exemption 2 sits
behind Conquer the World. Exemption 3 needs a tribe with a missing or empty
city list, and is taken as unreached under the assumption in 1.3. `Leaders::clear` runs at game close
and in multiplayer out-of-sync recovery.

### 1.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/city-name-*.md`); no diff backs it yet.

Verdicts: 41 confirmed, 3 corrected, 1 open, plus one new finding (the
crate's FORBIDDENCITY exemption is load-bearing). None of the corrections
changed a rule: the count of absolute `0xe3a79c` sites (8, the same two
stores), the count of `Leader::init` call sites in `ScenarioRead::
load_leader_chunk` (6), and "unused slots have tribe -1" (the dump shows
slots 8 and 9 at tribe 0; they sit outside the scan and cannot win).

Confidence is high on the writers (both nets swept, the bulk writers read)
and high on the step and its predicate (listing 735ff3-7362d2, both readings
agreeing). Gate (a)'s reach in a plain game is medium: it rests on the
crate's ratified `is(VILLAGE)` reading for FORBIDDENCITY.

**Witness, not a pinned diff.** Run 1562's frame-91 `@dumpall`
(`gamelog-run1562-greatlakes-playback-of-minted-packet-91.txt`) logs slot 0
(tribe 4) and slot 1 (tribe 11) at `city_name 1` (lines 817145, 829402), and
slots 8 and 9 at 0. Two distinct tribes, each with `starting_town` on, each
stepped once for its start city: the rule's distinct-tribe path, with no
frame-0 AI placement and no double step. The crate emits 0 for both. The
tribe-shared winner and the `= 1` store rest on the listing alone.

### 1.6 Not established

- **A computer player placing a city at frame 0** through `Leader::make_this
  (006c94f0)` (called only from `make_stuff` and `plan_strategy`) and
  `produce_building`. Open, bounded by run 1562, which shows none by frame
  91. A capture with a computer player whose `city_name` exceeds its city
  count at frame 1 would settle it.
- **The tribe-shared path** (winner a tribe-mate, the `= 1` store). A
  capture with two leaders of one nation, diffed on this leaf, would make it
  a diff.
- **Whether `starting_town` is on by default in the lobby.** A capture of a
  default lobby, or a reading of the option's default, settles it.
- **The type predicate that sets the city flag on FORBIDDENCITY**
  (`SubObject::init (00662300)`) was not re-derived; it is taken from the
  crate's ratified reading. A diff of a completed Forbidden City on this leaf
  would back the exemption directly.
- **`L` per tribe** is not in the crate; the `n < L` guard and exemption 3 are
  ported under the assumption above.
