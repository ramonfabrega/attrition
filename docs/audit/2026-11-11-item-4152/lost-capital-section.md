## 11. The lost capital

### Leaves

- `lost_capital_stamp`, +0x414, int, frames.
- `lost_capital_timer`, +0x418, int, a 0/1 flag (only 0 and 1 are stored).
- `lost_capital_modifier`, +0x41c, int, 8.8 (`0x100` = ×1).

The three move together. A **loss** sets the timer, writes the modifier and
turns the stamp into a start time. A **stop** clears the timer and turns the
stamp back into a debt. A per-frame **refresh** pays the debt down while no
clock runs. The **countdown** defeats a loser whose clock runs out.

### The rule

**Loss: `Leader::lost_capital (006dc870)`**, called with `O` the loser and
`A` the other leader. It does anything only when `timer == 0`. If the timer
is already 1, the call stores nothing, so a second loss while the clock runs
is a no-op for all three leaves. Live callers:

- **E1**, `Cities::capture_city`, call site 735092, with `A` the captor.
  The captured city carries flag `0x10`, `has_wonder(O, 0x213)` (the
  Forbidden City) is 0 or 2, and `find_capital(O, skipping this city)`
  finds nothing or a holder other than `O`. Together these mean `O` holds no
  live Forbidden City and no other live `0x10` city of its own.
- **E2**, `Build::close (00628980)`, call site 62912e, with `A` the holder.
  The closing building is the Forbidden City (`0x213`) and is **active**,
  meaning Build byte `+8` bit `0x4`, the test `WallData::is_active
  (00472350)` makes. The `Game` semaphore byte 1 bit `0x8` is clear,
  `elimination != 3`, and `find_capital(O, -1, -1)` returns a holder other
  than `O`. Under tribe bonus `0x17`, a second `find_capital(O, skipping
  (idx, holder))` runs, and if its hit's flag word is negative, that hit's
  holder becomes `A`. `lost_a_city(O, 1, 1, A)` follows. When the closing
  Forbidden City is not active, the arm calls
  `UnbuiltWonders::remove_unbuilt_wonder` instead.

Within the loss, in order:

1. `timer = 1`.
2. The modifier. `To = get_team_terr(O)` and `Ta = get_team_terr(A)`, where
   `get_team_terr` sums `territory` (+0x9d8) over a team. Then:
   ```
   w = num_nations == 0 ? 0 : land_size / num_nations    // World+0x78, Game+0x6a0
   m = (To + w == 0 || Ta + w == 0) ? 256
                                    : clamp((To + w)·256 / (Ta + w), 128, 512)
   if O.city_num != 0 && A.city_num != 0 && A.city_num < O.city_num:
       m = min(m·(O.city_num + 2) / (A.city_num + 2), 768)
   ```
   The city rescale follows either branch. Its result is at least `m`, so
   only the 768 ceiling binds. Both `city_num`s **count the city changing
   hands**: `Build::activate` has already added it to `A`, and `City::close
   (00737550)`, which takes it from `O`, runs after both capital calls. In
   terms of the cities each side held outside that one, the ratio is
   `(O_left + 3)/(A_before + 3)`. `To` and `Ta` are the sums from the last
   `GameDaemon::check_borders (00732060)` pass, the only in-play writer of
   +0x9d8, so they come from before the capture's border change. All
   divisions are signed and truncating.
3. `stamp = frame + (stamp·256) / m`, C truncating division, with
   `stamp ≤ 0` on entry. A first loss starts from 0 and stores `frame`
   exactly. A loss after a recapture folds the remaining debt in, rescaled
   by the new `m`.

The messages after the stores are gated on `elimination == 1`. The stores
themselves are not gated.

**Stop: two writers.** Each does anything only when `timer == 1`. Each sets
`timer = 0` and `stamp = (stamp − frame)·m / 256`, truncating toward zero:
the listing adds `0xff` to a negative product before the shift. The product
is `≤ 0`, so a floor would differ by one frame.

- **E3**, `Leader::recapture_capital (006d1ba0)`, on `A`, the taker. Its
  sole caller is `capture_city`, call site 7350e2, when the taker's new city
  has `0x10` and `find_capital(A, skipping (new city, A))` finds nothing or
  a holder other than `A`.
- **E4**, `Build::activate (00623e20)`, Forbidden City arm (625f64-625fc8),
  on the owner. It fires when the wonder type is in `0x20e..0x21e` (or
  passes the virtual test), `init_wonder` is `≥ 0` as a signed short, the
  building `is(0x213)`, and the owner's timer is set. It is **not** gated by
  `captured`, so a captured Forbidden City stops the taker's running clock.

**Refresh, inlined in `Leaders::process_all (006ed2a0)`** (store 6ed394).
`Leader::process (006b88b0)` is its dead twin. Every frame, for slots
**0..7** with `leader_flags & 2`, the refresh runs after that slot's
`process_elimination`:

```
if ratio != 0 && frame % ratio == 0 && stamp < 0 && timer == 0: stamp += 1
```

Here `ratio` is `TIMER_REFRESH_RATIO`, 5 shipped (`rules.xml` line 727,
`core/tuning.rs:1923`). The debt is paid at **one frame per five**.

**Countdown: `Leader::process_elimination (006b8a20)`.** It runs under the
outer `game` (0xc061ec) gate when `elimination == 1` and `timer != 0`. If
`limit − (frame − stamp) ≤ 0` (signed), the leader is `defeat(1, by)`. Here
`by` is the holder `find_capital(-1, -1)` returns when `gamec` (0xc061e8)
elimination is 1, and the leader itself otherwise. `limit` comes from
`Game::retake_capital (00594530)`:

```
base  = RETAKE_CAPITAL < 1 ? 0 : max(1, (xs·RETAKE_CAPITAL + S/2) / S)
limit = base·m / 256                         // truncating toward zero, who ≥ 0
```

`xs` is `World+0x0`, the map width. `S` is `+0x144` of the record at
`*(0xe7fca8)`, the Standard map size, 70 (`docs/MAPGEN.md` line 4273).
`RETAKE_CAPITAL` is 3600 (`rules.xml` line 728). `S/2` truncates toward
zero. With `Game` semaphore byte 2 bit `0x2` set, two further arms apply:
`has_wonder(who, FC) == 2` doubles the limit, and a slot-0/1 peace arm
halves it. Both are Conquer the World-shaped (`== 2` is the CtW bit alone).
A larger `m` gives the loser more time.

**Resets.** `Leader::init (006e3930)` stores 0 to the stamp and the timer at
top level on every call (6e3f89, 6e3f8f), not to the modifier. The bulk
writers (header) zero all three. `Leader::defeat` and `defeat_by` clear
nothing. **The modifier goes stale after a stop and must be kept, not
zeroed.** Every rule reader of it sits behind `timer != 0`, but the checksum
and a whole-record dump see the stale value.

**Readers that change a decision.** The countdown (timer, stamp,
modifier). The refresh (stamp, timer). The stop (modifier). `Player::leave_game
(006ee010)` (timer): on a resign or drop, a leader with `leader_flags & 3 ==
3` and the timer set is `defeat(1, holder)` under `elimination == 1`, and
`defeat(1, self)` otherwise. It chooses who is credited with the defeat and
writes none of the leaves. Its callers are `Player::resign (006edcb0)`
(from `Player::quit (006edc00)`, `CommandManager::issue_resign
(00941170)`, `CommandPackage::process_resign (009438c0)`) and `Player::drop (006ede80)`.
`IFaceMainBase::draw_timers (0080b060)` and `LeaderData::log_data (006e5110)` read the
leaves for display only.

### Where this corrects docs/CITIES.md §8.5

§8.5 says the debt "decays one frame per frame while the capital is held".
It decays **one frame per `TIMER_REFRESH_RATIO` frames, five shipped**, and
only while the timer is 0. Three further corrections to the same paragraph:

- Both `× m >> 8` steps, in the stop and in `retake_capital`, **truncate
  toward zero**. They are not arithmetic shifts.
- "How the base countdown is first seeded is not established" is
  **settled**. `Leader::init` zeroes the stamp, so a first loss stores
  `frame`. The countdown is `retake_capital` above, with `S` the Standard
  map size. This also settles `docs/CITIES.md`'s open item 6: the stamp's
  seeding, and the map-size field `retake_capital` divides by.
- The loss runs "once" only in effect. Its guard is the timer, so a loss
  after a recapture runs again.

### In the crate

- **Stamp and timer.** There is no field for either. `Tally::lost_capital_frame`
  (`cities/city.rs:122`), an `Option`, is set and cleared at
  `city.rs:3213-3224` and **never read**: it is a partial copy of the stamp
  plus timer. E1 and E3 replace those stores in `Sim::capture_city`
  (`city.rs:3169`). The new fields sit beside `popwin_stamp` (`ai.rs:705`).
- **E2** goes in `Sim::close_building` (`city.rs:2782`). **E4** goes in
  `Sim::activate` (`city.rs:2420`), beside `kremlin_activate`
  (`cities/nations.rs:467`, called at `city.rs:2475`).
- **Refresh** becomes a second clause of `refresh_victory_timers`
  (`territory/victory.rs:88`, called at `core/frame.rs:194`), which today
  pays only `popwin_stamp`. **Countdown**: a port of `process_elimination`
  goes immediately before `frame.rs:194`, with `retake_capital` beside it.
  There is no `retake_capital` and no scale-by-axis helper today (only
  `mapgen/map_lakes.rs:314 scale_by_area`). The inputs exist: `team_terr`
  (`ai/ai_census.rs:155`) and `land_size` (`victory.rs:110`).
- The AI's `capital_lost` (`ai/ai_build.rs:296`) recomputes from the cities.
  The verdicts treat this only as a pointer.
- **Checksum.** The rows are `checksum/leaders.rs:100-102`. Today
  `LEADERS_HELD_OUT` lists `lost_capital_modifier`, `_stamp` and `_timer`,
  each `(unheld)` with count 2 (`rondata/src/diff/checksum_lines.rs`
  1809-1811). Holding all three removes those three entries.

### Reached by a plain game

Yes, on any capital capture. E1 and E3 are skirmish events, and their stores
are ungated by `elimination`. E4 is reached by building a Forbidden City or
by capturing one. E2 needs an active Forbidden City to close while another
leader holds the owner's former capital. The refresh runs every frame. The
countdown needs `elimination == 1`. `leave_game` is reached by a resign
command, in single player too. Two paths are probably not plain-game:
`retake_capital`'s semaphore-gated arms (CtW-shaped), and E2's semaphore
byte 1 bit `0x8` (meaning unread).

### How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/lost-capital-*.md`); no diff backs it yet.

The verdicts: stamp 24 confirmed, 1 corrected, 1 open; timer 14 confirmed,
0 corrected, 0 open; modifier 14 confirmed, 0 corrected, 3 open. Every
store was read in the listing, and both readers agree on all arithmetic.
Confidence is high for the formulas. It is medium-high for E1's
`city_num` timing (taken from the decompile's order; `Build::swap_team` was
not read for a `city_num` store) and medium for the territory timing.

Corrections that changed a rule:

- E2's gate is `is_active` (bit `0x4`), not bit `0x1`.
- The §8.5 corrections above.
- Two inputs the readings left open are settled: both `city_num`s include
  the captured city, and `To`/`Ta` come from the last `check_borders`.
- `leave_game` is reached by a resign command, not only by a departure.

Witness, not a pinned diff: `crates/rondata/src/diff/korean_ancient_pair.rs:326`
records `lost_capital_stamp 5639` on the human's defeat on frame 5639,
consistent with a first loss storing `frame`.

### Not established

- **Whether E2 can run inside a capture.** A captured Forbidden City is
  `swap_team`ed and `activate`d, not closed (`docs/CITIES.md` §7.3). Read
  `Build::swap_team`'s body for a `close` call.
- **The `Game` semaphore bits**: byte 1 bit `0x8` (E2's gate) and byte 2
  bit `0x2` (`retake_capital`'s arms). Read their writers.
- **`num_nations`' writer** (`Game+0x6a0`): whether it counts gaia or the
  dead. Read its writers.
- **`get_team_terr`'s team-style-7 arm and its ally rule** (not this
  family's leaf). Read `LeaderData::get_team_terr (006d62e0)` whole.
