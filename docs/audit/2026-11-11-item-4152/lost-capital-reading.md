# Item 4152, family 11 `lost-capital` — first reading

- Family: 11, slug `lost-capital`.
- Leaves: `LeaderData::lost_capital_stamp` (+0x414, int),
  `lost_capital_timer` (+0x418, int), `lost_capital_modifier` (+0x41c, int).
- Reader: Opus 5.5 (`claude-opus-5-5`), wave 1, first reading, 2026-11-11.
- Layout check: `types.txt` gives `Leader` size 0x6eec with no members listed
  and `LeaderData` 0x6ee4; the leaders array `Leaders::list` is `Leader[10]`
  at 0xe3a390 and `docs/CHECKSUM.md` §8 takes slot i's `LeaderData` at
  `0xe3a390 + i·0x6eec`, so `LeaderData` is at offset 0 of `Leader` (the 8
  extra bytes trail) and a `Leader` method's `field_0x414` is
  `lost_capital_stamp` with no prefix to subtract. Confirmed in the listing
  below (absolute stores at 0xe3a7a4/0xe3a7a8/0xe3a7ac + i·0x6eec).

## The two nets, once for all three leaves

- **Export.** `grep -rn -- '->lost_capital'` finds only `LeaderData::log_data (006e5110)` (the
  gamelog printer, a reader). `grep -rn 'field_0x41[48c]\b'` over `funcs/` finds the
  leader hits in `Leader::lost_capital (006dc870)`, `Leader::recapture_capital (006d1ba0)`,
  `Build::activate (00623e20)`, `Leader::process (006b88b0)`, `Leaders::process_all
  (006ed2a0)`, `Leader::init (006e3930)`, `Leader::process_elimination (006b8a20)`,
  `Game::retake_capital (00594530)`, `Player::leave_game (006ee010)`; the other hits
  (`GameReportWin::close`, `ConquestInvadeWin::close`) are window classes, not leaders.
- **Listing.** Absolute `0xe3a7a4/0xe3a7a8/0xe3a7ac + i·0x6eec` for i = 0..9: nine sites,
  all in `Game::retake_capital` (594587, read), `Build::activate` (625f64..625fc8),
  `Leader::process_elimination` (6b8a4f, read), `Player::leave_game` (6ee0a8, read).
  Register-relative stores whose destination is `0x414/0x418/0x41c(%reg)` (any
  register but `esp`; 70 sites): the leader ones are exactly `Leader::process`
  (6b8994), `Leader::recapture_capital` (6d1bb8, 6d1bd7), `Leader::lost_capital`
  (6dc88a, 6dc8fb, 6dc952, 6dc968, 6dc976), `Leader::init` (6e3f89, 6e3f8f),
  `Leaders::process_all` (6ed394). The rest were attributed to non-leader bases and
  excluded: `ConquestMapLoader`/`ConquestLeader` (CtW objects with a byte/word at
  0x414), `TerrainBorder`, `RiverSectionOut`, `RTreeData`, `GraphicWinXML`, the `ebp`
  negative-offset locals of `ImageIO`/`Steam`/`File::copy`/`*::log_data`, and
  `ScenarioRead::load_leaders_chunk (009acde0)` 9ad0a4/9ad118/9ad11e, whose `edx` is the
  `Game` (`info.player[]` stride 0x8c; it writes a *word* at 0x414, which a
  `LeaderData` int cannot be). `LeaderData::get_team_wonders (006da2d0)` reads
  `0x41c(%edi)` with `edi = leader + 8`, i.e. `wonder_mark` (+0x424), not ours.
- **Bulk writers.** `memset(leader, 0, 0x6929)` covers all three leaves: in
  `Leaders::clear (006ed5d0)` (each of the ten leaders; called from
  `Game::run_oos_recovery` and `Game::close`) and in `Leader::init`'s `param_2 < 0`
  arm (listing 6e398c: `push 0x6929; push 0; push ebx; call memset`), which
  `Leaders::init (006ed850)` takes for every slot with `init(i, -1, i)`;
  `Leaders::init` is called once, from `Game::init_rules_and_teams`. The save/load
  walker (`Leaders::walk_data (006e38e0)`, `WalkDataGame::walk_data`) was not read; it
  is a generic serialiser and not reached by a plain game without a load.
- `Leader::process (006b88b0)` has **no caller**: no `call 0x6b88b0` in `.text`, and
  its address appears nowhere in the file as a 32-bit word (no vtable). The map names
  it (the decorated name of `Leader::process`, `QAEXXZ`, from leaders.obj). Its body is inlined, word for word,
  into `Leaders::process_all`, which `Game::do_frame (00591ef0)` calls once a frame
  (59224f). So the live per-frame writer is `Leaders::process_all`; `Leader::process`
  is a dead copy. **This contradicts the README row only in emphasis**: the row names
  `Leader::process`, and the store that runs is `process_all`'s.

## Leaf 1 — `lost_capital_stamp` (+0x414, int, frames)

**Meaning, as the writers use it.** Two regimes, switched by `lost_capital_timer`:
with the timer 0 the stamp is a **debt** in frames, `≤ 0`, that is paid back
toward 0; with the timer 1 it is an **absolute frame**, the origin the capital
countdown is measured from (`frame − stamp` is the time charged so far).

### Writers

1. `Leader::lost_capital (006dc870)`, stores 6dc968 and 6dc976, only on the arm
   where the timer was 0 (6dc87e `cmp 0, 0x418(edi); jne` past the block). After
   the modifier `m` is set (leaf 3): `stamp = (stamp << 8) / m` (signed `idiv`,
   truncating toward zero; 6dc95e–6dc962), stored, then `stamp = frame + that`
   (`Game+0x550`, `frame`; 6dc96e–6dc976). So a debt `d ≤ 0` is converted to frames
   at the new rate and the origin is pulled back by it: `stamp = frame + trunc(d·256 / m)`.
   With `d = 0` (the first loss of a game) the stamp is just the frame.
   - Callers (listing `call 0x6dc870`: two):
     - `Cities::capture_city (00733380)` at 735092. Arguments from the listing:
       `capture_city(this, taker = [ebp+8], city = [ebp+0xc], owner = [ebp+0x10])`.
       Predicate: the captured city (owner's list `0xc09970 + owner·0x1c`, index
       `city`) has flag `0x10` (capital) at `+4`; **and** `LeaderData::has_wonder(owner,
       0x213)` (the Forbidden City) returns 0 **or** 2 (2 is the CtW conquest-wonder
       bit alone, `has_wonder (006ebc10)`; a live owned one sets bit 1, so 1 and 3
       exempt); **and** `find_capital(owner, &c, &h, skip_city = city, skip_owner =
       owner)` answers `c < 0` **or** `h != owner`. Then `lost_capital(owner, taker)`.
       `find_capital (006eb930)` first scans the owner's own list for a live
       (`flags & 1`) capital (`flags & 0x10`) other than the skipped one (holder =
       owner); else, only when the owner's `leader_flags & 3 == 3`, scans the other
       seven leaders' lists for a live city whose was-capital mask (`City+0x68`) has
       the owner's bit (holder = that leader); else `c = −1`, holder = owner. So the
       loss fires unless the owner still holds another live capital-flagged city of
       its own.
     - `Build::close (00628980)` at 62912e, the wonder arm: the closing building's
       `flags & 1` (active; listing 6289ce `and esi, 1` of byte `+8`), it `is(0x213)`
       (the Forbidden City), `Game` semaphore byte 1 bit 3 (`& 8`) clear (not read
       what that bit is), and `elimination != 3`. Then `find_capital(owner, -1, -1)`;
       **only if the holder `h != owner`** (6290cd) — i.e. another leader holds a
       city with the owner's was-capital bit; a leader with no capital at all gets
       `h = owner` and is *not* charged here — `lost_capital(owner, h')` with
       `h' = h`, except that under `has_tribe_bonus(owner, 0x17)` (the bonus the
       crate's `persian_second_capital` keys on) a second `find_capital(owner, skip
       that city, skip h)` that finds a city whose flags word is negative (bit 15)
       takes that city's holder instead (629102–629126). It is followed by
       `Leader::lost_a_city(owner, 1, 1, h')`.
2. `Leader::recapture_capital (006d1ba0)`, store 6d1bd7, only when the timer is
   nonzero: `stamp = trunc(((stamp − frame) · m) / 256)` (the `cdq; and 0xff; add; sar 8`
   bias, so toward zero). `stamp − frame` is `≤ 0` (minus the charged time), so the
   stamp returns to a debt: the old debt minus the elapsed time scaled by `m/256`.
   - Caller: `Cities::capture_city` at 7350e2, for the **taker**: the new city in the
     taker's list (index `[ebp−0x10]`) has flag `0x10`, and `find_capital(taker, &c,
     &h, skip = that new city, skip_owner = taker)` answers `c < 0` or `h != taker`.
     No other caller in `.text`.
3. `Build::activate (00623e20)`, stores 625f90 (`sub frame`) and 625fc8: the same
   arithmetic as `recapture_capital` inlined — `stamp −= frame; stamp = trunc(stamp ·
   m / 256)` — and the timer cleared. Predicate: the building is a wonder (type
   `0x20e..0x21e`, or `is_wonder_type`), `Wonders::init_wonder` returned a slot
   `≥ 0`, it `is(0x213)` (the Forbidden City), and the owner's (`Build+9`) timer is
   nonzero. Not gated by `param_1` (captured) nor by `elimination`; it posts a
   message (string `+0x834`) unconditionally, without the sounds.
4. `Leaders::process_all (006ed2a0)`, store 6ed394 (the live copy of `Leader::process
   (006b88b0)` 6b8994): each frame, for each of leaders 0..7 (loop bound `0xe71af0`)
   with `leader_flags & 2`, when `constants.timer_refresh_ratio != 0` and
   `frame % timer_refresh_ratio == 0`: if `stamp < 0` **and** the timer is 0,
   `stamp += 1`. The crate's `TIMER_REFRESH_RATIO` is 5 (`crates/sim/src/core/tuning.rs:1923`),
   so the debt is repaid one frame per five frames. Runs after
   `process_elimination` in the same per-leader body.
5. `Leader::init (006e3930)`, store 6e3f89: `stamp = 0`, unconditionally (both the
   `param_2 < 0` and the setup arms reach it); so does the `memset` above.

**Step summary.** loss (timer 0→1): `stamp = frame + trunc(stamp·256/m)`;
recapture / Forbidden City built (timer 1→0): `stamp = trunc((stamp − frame)·m/256)`;
refresh: `+1` per `timer_refresh_ratio` frames while `stamp < 0` and timer 0.

### Resets
`Leader::init` (to 0, every init), `Leaders::clear` and `Leaders::init` (memset).
No per-frame fold other than the refresh.

### Readers
- `Leader::process_elimination (006b8a20)` (called by `process_all` at 6ed35a and the
  dead `Leader::process`): under `elimination == 1` and the timer nonzero, `elapsed =
  frame − stamp` (6b8a4f), `r = Game::retake_capital(who)`; if `r − elapsed ≤ 0`
  (6b8a5a–6b8a5e, signed) the leader is `defeat(1, …)`ed — by the holder of its
  capital per `find_capital(−1, −1)` when `gamec.elimination == 1`, else by itself.
  **This is the decision the stamp drives**: elimination by capital countdown.
- `LeaderData::log_data (006e5110)`: the gamelog's leader record.

### Crate
- The per-frame refresh has a home: `Sim::refresh_victory_timers`
  (`crates/sim/src/territory/victory.rs:88`), called per leader from
  `crates/sim/src/core/frame.rs:194`, already pays back `popwin_stamp` (+0x440) on the
  same predicate; the stamp would be its third clause (+0x448 is the other).
- The loss and recapture: `Sim`'s capture in `crates/sim/src/cities/city.rs` (around
  3213) already sets `city_tally[o].lost_capital_frame = Some(frame)` when the
  captured city was a capital and none is set, and clears the taker's on taking a
  capital back. That is a partial copy: it has no Forbidden City exemption, no
  `find_capital`, no debt (it clears instead of converting), and it is **written and
  never read** (`grep lost_capital_frame crates` finds only the field and the two
  stores).
- Forbidden City arms: `Sim::activate` (`crates/sim/src/cities/city.rs:2420`, its
  wonder branch near 2485) and `Sim::close_building` (`city.rs:2782`, its wonder close
  near 2881) are where `Build::activate`'s and `Build::close`'s arms would go; neither
  has one.
- `process_elimination` has no port; `ai.lobby.elimination` (`crates/sim/src/ai.rs:1088`,
  default 1) is read only by the AI's `capital_lost`/`holds_capital` predicates.

### Plain game?
Yes. `lost_capital` from `capture_city` needs only a capital captured in a skirmish;
its stores are not gated by the elimination mode (only the messages and
`process_elimination` are). The refresh runs every game but is a no-op until a debt
exists. `recapture_capital` needs the capital (or a city carrying `0x10` for the taker)
retaken. The Forbidden City arms need that wonder (a plain-game wonder).
A captured game already shows the value: `crates/rondata/src/diff/korean_ancient_pair.rs:326`
records the human's `lost_capital_stamp 5639` on its defeat frame 5639 — exactly
`frame + 0` for a first loss.

### Confidence
High for the arithmetic and the store sites (listing read at each store). Medium for
the exact predicates of `capture_city`'s two calls (read in the listing; the
decompile's locals are tangled) and of `Build::close`'s tribe-bonus-0x17 substitution.
Not read: `Game` semaphore byte 1 bit 3; which code sets city flag `0x10` on the
taker's new city (the recapture's precondition); the save/load walker; `capture_city`'s
own callers (it is reached from many places, including `defeat_by`).

## Leaf 2 — `lost_capital_timer` (+0x418, int, a 0/1 flag)

Despite the name it holds no time: only the constants 0 and 1 are ever stored. 1
means "capital lost, countdown running".

### Writers
1. `Leader::lost_capital (006dc870)`, 6dc88a: `timer = 1`, on the arm where it was 0
   (the same arm that sets the modifier and converts the stamp). A second loss while
   it is 1 changes nothing but the messages. Callers and predicates as leaf 1
   (`Cities::capture_city` 735092; `Build::close` 62912e).
2. `Leader::recapture_capital (006d1ba0)`, 6d1bb8: `timer = 0`, if it was nonzero.
   Caller `Cities::capture_city` 7350e2 (predicate as leaf 1).
3. `Build::activate (00623e20)`, 625f71: `timer = 0`, if nonzero, on a Forbidden City's
   activation (predicate as leaf 1).
4. `Leader::init (006e3930)`, 6e3f8f: `timer = 0`, unconditionally.
5. Memset: `Leaders::clear`, `Leaders::init` → `Leader::init(i, −1, i)`.

### Readers
- `Leader::process_elimination (006b8a20)` 6b8a36: gates the countdown check (with
  `elimination == 1`).
- `Leaders::process_all` (6ed2a0, its line 51) and the dead `Leader::process`: the
  refresh only while it is 0.
- `Player::leave_game (006ee010)` 6ee0a8: a leaving leader with `leader_flags & 3 ==
  3` and the timer set is `defeat(1, holder of its capital)` under `elimination == 1`,
  else `defeat(1, itself)`; otherwise `defeat(param_1, −1)`. Changes who is credited.
- `Leader::lost_capital`, `recapture_capital`, `Build::activate`: their own once-only
  guards.
- `LeaderData::log_data`.

### Crate
No field. The truth it encodes is approximated by `lost_capital_frame.is_some()`
(`crates/sim/src/cities/city.rs:122`), set/cleared at the same two capture moments
(city.rs 3213–3224) but without the Forbidden City arms. The AI's
`capital_lost` (`crates/sim/src/ai/ai_build.rs:296`) recomputes "no own capital and
someone holds my old one" from the cities instead of reading a flag; a builder would
keep it beside `lost_capital_frame` in `Tally` or on the leader.

### Plain game?
Yes, as leaf 1: a captured capital sets it. `Player::leave_game` is the multiplayer /
resign path and was not followed to its callers.

### Confidence
High: every store is a constant, each site read in the listing. Not read:
`Player::leave_game`'s callers (whether a single-player resign reaches it).

## Leaf 3 — `lost_capital_modifier` (+0x41c, int, 8.8 ratio: 0x100 = 1)

### Writers
1. `Leader::lost_capital (006dc870)`, 6dc8fb / 6dc903 / 6dc952, on the timer-0 arm only
   (so computed afresh at each new loss, never on a repeated one). `this` = the loser
   O, `param_1` = the taker A (`capture_city`'s taker, or `Build::close`'s holder).
   - `To = LeaderData::get_team_terr(O)`, `Ta = get_team_terr(A)` (`get_team_terr
     (006d62e0)`: the leader's `territory` (+0x9d8) plus each live ally's, with a
     team-style-7 special case not read here).
   - `w = world.land_size (WorldData+0x78) / game.num_nations (Game+0x6a0)`, signed
     truncating; `w = 0` when `num_nations == 0`.
   - If `To + w == 0` or `Ta + w == 0`: `m = 0x100` (6dc903).
   - Else `m = trunc((To + w)·256 / (Ta + w))`, then `m < 0x80 → 0x80`, `m > 0x200 → 0x200`
     (6dc8db–6dc8fb).
   - Then, if `O.city_num ≠ 0` and `A.city_num ≠ 0` and `A.city_num < O.city_num`
     (strict; `city_num` +0x3f8): `m = trunc(m·(O.city_num + 2) / (A.city_num + 2))`,
     clamped to `[0x80, 0x300]` (6dc90d–6dc952). Note the second clamp's ceiling is
     0x300, not 0x200, and the floor 0x80 again.
   - So `m ∈ [0x80, 0x300]` always, never zero: the later `idiv` by `m` cannot fault.
   - When in `capture_city` the call runs, the captured city's `City::close(old, A)`
     has not yet run (the close is after both capital calls, 7350e7 onward), so
     whether `O.city_num` still counts the captured city at that moment was **not
     established** (it depends on where `city_num` is decremented).
2. No other store. `Leader::init`'s setup arm (`param_2 ≥ 0`) does **not** write it
   (the 6e3f89/6e3f8f pair is the stamp and the timer only); only the `memset`s clear
   it — `Leaders::init` (once, in `Game::init_rules_and_teams`), `Leaders::clear`
   (`Game::close`, `Game::run_oos_recovery`), and `Leader::init(…, param_2 < 0, …)`.
   `recapture_capital` and `Build::activate` read it and leave it, so after a
   recapture it keeps the last loss's value until the next loss overwrites it.

### Readers
- `Game::retake_capital (00594530)`: the countdown length. `base = 0` if
  `constants.retake_capital (+0xd04) < 1`, else `max(1, (S/2 + world.xs ·
  retake_capital) / S)` with `S` the int at `+0x144` of the record the pointer at
  `0xe7fca8` names (listing 594548–59454d; the decompile prints it `map_sizes.list`;
  the field is unnamed here), `S/2` signed toward zero, and `world.xs` (`*(int*)world`,
  `WorldData+0`, taking `WorldData` at offset 0 of `World` as `docs/ECONOMY.md`'s
  `land_size` at +0x78 row does — the same assumption names `w` above);
  then, for `who ≥ 0`, `r = trunc(base · m / 256)` (toward zero, 594587–59459f). Under
  `Game` semaphore byte 2 bit 1 (`& 2`; not read what it is), the Forbidden City
  doubles it (`has_wonder(who, 0x213) == 2` → `2r`; 2 is the CtW conquest-wonder bit
  alone, so this is a Conquer the World arm, and the semaphore bit is likely CtW's —
  not checked) or, when the owner has none and
  leader slot (0 if `who ≠ 0`, else 1) is live, at peace (`get_diplo == 0`) with
  `who` and owns one, halves it — this branch was read from the decompile only.
  So a larger `m` gives the loser more time. Callers: `process_elimination` (6b8a55)
  and `IFaceMainBase::draw_timers` (80c40e, the on-screen countdown, `max(0, r −
  (frame − stamp))`).
- `Leader::lost_capital` (the division of the stamp), `recapture_capital` and
  `Build::activate` (the multiplication back).
- `LeaderData::log_data`.

### Crate
No field and no `RETAKE_CAPITAL` constant (`grep -rn retake crates/sim/src` finds
nothing of it; `docs/CITIES.md` table at its line ~2100 names the rule). The inputs
exist: `team_terr` (`crates/sim/src/ai/ai_census.rs:155`), `land_size`
(`crates/sim/src/territory/victory.rs`), `city_num`. The computation belongs beside the
loss in the capture path of `crates/sim/src/cities/city.rs` (around 3213), and
`retake_capital` with a port of `process_elimination` in the per-leader frame body
(`crates/sim/src/core/frame.rs` near 194).

### Plain game?
Yes: the first capital loss of a skirmish writes it.

### Confidence
High for the formula and clamps (listing read end to end, 6dc870–6dc976). Medium for
`get_team_terr`'s ally predicate (not my leaf; read only its head). Not read:
`Game+0x6a0` beyond its name `num_nations` (whether it counts gaia or the dead), and
the `city_num` timing above.

## Against the README row

The row's writers — `Leader::lost_capital`, `recapture_capital`, `Build::activate`,
`Leader::process` — are all confirmed, with three amendments:

- `Leader::process (006b88b0)` has no caller; the store that runs each frame is its
  inlined copy in `Leaders::process_all (006ed2a0)` (6ed394), called from
  `Game::do_frame`.
- Missing from the row: `Leader::init (006e3930)` (stamp and timer to 0, every init;
  **not** the modifier), and the `memset(…, 0x6929)` of `Leaders::clear (006ed5d0)` and
  of `Leader::init`'s `param_2 < 0` arm (via `Leaders::init (006ed850)`), which are the
  modifier's only resets.
- `Leader::lost_capital` has two callers, `Cities::capture_city (00733380)` and
  `Build::close (00628980)` (an active Forbidden City closing while another leader
  holds the owner's old capital); `recapture_capital` has one, `capture_city`.
- The row's "the tally's `lost_capital_frame` is a partial copy of the stamp": true in
  shape, and in the crate it has no reader at all.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `lost_capital_stamp` +0x414 | Leader::lost_capital (006dc870) ← Cities::capture_city (00733380), Build::close (00628980); Leader::recapture_capital (006d1ba0) ← capture_city; Build::activate (00623e20) (Forbidden City); Leaders::process_all (006ed2a0) (live copy of Leader::process (006b88b0), no caller); Leader::init (006e3930) | loss: `frame + trunc(stamp·256/m)`; recapture / FC built: `trunc((stamp − frame)·m/256)`; `+1` per `TIMER_REFRESH_RATIO` (5) frames while `< 0` and timer 0 | `Leader::init` → 0; memset in `Leaders::clear`, `Leaders::init` | refresh: `refresh_victory_timers` (`territory/victory.rs:88`, from `core/frame.rs:194`); loss/recapture: capture in `cities/city.rs` ~3213 (`lost_capital_frame`, write-only); FC arms: `Sim::activate` (city.rs:2420), `Sim::close_building` (city.rs:2782) | yes (any captured capital) | high (arith), medium (capture predicates) |
| `lost_capital_timer` +0x418 | Leader::lost_capital (006dc870) → 1; Leader::recapture_capital (006d1ba0) → 0; Build::activate (00623e20) → 0; Leader::init (006e3930) → 0 | 0/1 flag, set at loss, cleared at recapture or FC activation | `Leader::init`; memsets | approximated by `lost_capital_frame.is_some()` (`cities/city.rs:122`); no field | yes | high |
| `lost_capital_modifier` +0x41c | Leader::lost_capital (006dc870) only | at each new loss: `(To+w)·256/(Ta+w)` clamped [0x80,0x200], then × `(O.cities+2)/(A.cities+2)` if `0 < A.cities < O.cities`, clamped [0x80,0x300]; 0x100 if a sum is 0 | memset only (`Leaders::clear`, `Leaders::init`); **not** `Leader::init`'s setup arm | none; inputs `team_terr` (`ai/ai_census.rs:155`), `land_size` (`territory/victory.rs:110`); belongs in the capture path, read by a `retake_capital` port | yes | high (formula), medium (`city_num` timing) |
