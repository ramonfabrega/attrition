# Item 4152, family 11 `lost-capital` — adjudication

- Family 11, slug `lost-capital`. Leaves: `LeaderData::lost_capital_stamp`
  (+0x414, int), `lost_capital_timer` (+0x418, int), `lost_capital_modifier`
  (+0x41c, int).
- Inputs: `lost-capital-reading.md` (first), `lost-capital-second.md` (blind
  second), the README row 11, and `docs/CITIES.md` §8.5.
- Adjudicator: Opus 5.5 (`claude-opus-5-5`), wave 3, 2026-11-11.
- Sources: the export `~/ghidra-projects/decomp/funcs/`, the `.text` listing
  `r4152/aa/exe.s` (llvm-objdump, AT&T syntax), `crates/sim/src`.
- Written leaf by leaf; a leaf without its closing rule was not finished.

## The nets, re-run once for all three leaves

- **Export.** `grep -rlE 'field_0x41[48c]\b|lost_capital_(stamp|timer|modifier)'`
  over `funcs/` gives twelve files: `Build::activate (00623e20)`,
  `Game::retake_capital (00594530)`, `Leader::init (006e3930)`,
  `Leader::lost_capital (006dc870)`, `Leader::process_elimination (006b8a20)`,
  `Leader::process (006b88b0)`, `Leader::recapture_capital (006d1ba0)`,
  `LeaderData::log_data (006e5110)`, `Leaders::process_all (006ed2a0)`,
  `Player::leave_game (006ee010)`, and two windows
  (`GameReportWin::close (007b91b0)`, `ConquestInvadeWin::close (0074f3d0)`).
  Both readers listed exactly these. **agree.**
- **Listing, absolute.** `0xe3a7a4/a8/ac` appear at 594587, 625f64, 625f71,
  625f90, 625f95..625fc8, 6b8a4f, 6ee0a8 only; the constant slot forms
  `0xe3a7a4 + k·0x6eec` (k = 1..9) appear nowhere. **agree** with both.
- **Listing, register-relative destinations** (`, 0x41[48c](%reg[,%reg])`
  with or without a trailing `# imm` comment, `esp` excluded): 34 lines.
  Leader stores: 6b8994, 6d1bb8, 6d1bd7, 6dc88a, 6dc8fb, 6dc903, 6dc952,
  6dc968, 6dc976, 6e3f89, 6e3f8f, 6ed394 (6dc962 is the `idivl` read). The
  rest are the non-leader bases both readers rejected (`ConquestMapLoader`
  48ff1c/26/2f/cd and 77e9a5, `GraphicWinXML` 517343/519a9c, `ConquestLeader`
  7a3afe/7a3b14, `RiverSectionOut` 88c198/285/28f/299, `TerrainBorder`
  8afadf..8afbf9, `RTreeData` 922dd1/9235c2, `ScenarioRead::load_leaders_chunk
  (009acde0)` 9ad0a4/118/11e). **No writer neither reader found.** A biased
  base (`0x40c..0x414` off `Leader+8`) cannot be excluded by grep; neither
  reader nor this pass saw one, and the export net (which normalises the
  base) is clean.
- **Dead twin.** `0x6b88b0` occurs in no `call`/`jmp` operand of `exe.s`
  and nowhere in `riseofnations.exe` as a little-endian dword (python count
  0). Both readers: `Leader::process (006b88b0)` is a dead, inlined copy;
  the live per-frame store is `Leaders::process_all (006ed2a0)` 6ed394.
  **confirmed.**
- **Bulk.** `memset(this, 0, 0x6929)` in `Leader::init`'s `param_2 < 0` arm
  (decompile line 70-77) and in `Leaders::clear (006ed5d0)` line 13 (callers
  586dd6 `Game::run_oos_recovery (005869e0)`, 594faa `Game::close
  (00594b80)`); `Leaders::init (006ed850)` has one caller, 589d04
  `Game::init_rules_and_teams (00589bb0)`. `LeaderData::walk_data
  (006d6750)` (second reading only) walks `[&who, +0x692a)` raw (line 17);
  its callers are `LoadGame::verify_load (005a39f0)`, `SaveGame::verify_save
  (005a76b0)`, `Leaders::walk_data (006e38e0)` and the three `CheckSums::*`
  walkers, so it writes only on a load. **confirmed** (one-sided, second);
  not a plain-game writer.

## Leaf 1 — `lost_capital_stamp` (+0x414, int, frames)

| claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|
| W1 `Leader::lost_capital`, timer-0 arm only: `stamp = frame + trunc((stamp<<8)/m)`, signed | yes | yes | agree | **confirmed**: 6dc87e `cmp 0,0x418; jne 6dc97c`; 6dc958-6dc976 `shl 8; cltd; idivl 0x41c; store; add 0x550; store` |
| W1's two callers, `Cities::capture_city` 735092 and `Build::close` 62912e | yes | yes | agree | **confirmed**: `call 0x6dc870` occurs at exactly those two addresses |
| W1 E1 predicate: city flag `0x10`; `has_wonder(O,0x213)` ∈ {0,2}; `find_capital(O, skip (c,O))` gives `idx<0` or holder ≠ O | yes | yes | agree | **confirmed**: `capture_city` decompile 1137-1144. Spot-check: the foreign scan in `find_capital` (006eb930 lines 26-52) also honours the skip pair and is gated on **the asking leader's** `leader_flags & 3 == 3`; with the old city still live and `City::capture` having set O's bit in the new record's `+0x68`, both branches of the predicate reduce to "O holds no other live `0x10` city of its own" |
| W1 E2 predicate: the closing building **active** | "`flags & 1`, 6289ce" | "the decompiler's `unaff_ESI != 0`" | disagree | **corrected**: the branch at 628fce tests the stack slot `0x8(%esp)`, which 6289e2-6289e5 fill with byte `+8 & 4` on the Build-vftable path and 6293bb fills with `WallData::is_active (00472350)` (`return flags & 4`) on the other. So the gate is **`is_active` (Build byte `+8` bit `0x4`)**, not bit `0x1`; the `and $1` at 6289d0 feeds a different slot (`0x10(%esp)`). Else-arm (not active) is `UnbuiltWonders::remove_unbuilt_wonder` |
| W1 E2: `is(0x213)`, `semaphore` byte 1 bit 8 clear, `elimination != 3`, then `find_capital(O,-1,-1)`, only if holder ≠ O | yes | yes | agree | **confirmed**: `Build::close` decompile 404-436; listing 6290c8 call, 6290cd `cmp %esi,0xc(%esp)` |
| W1 E2: tribe bonus `0x17` re-pick: a second `find_capital(O, skip (idx, holder))` whose hit's flag word is negative takes that hit's holder | yes | yes | agree | **confirmed**: decompile 438-447; the skip is honoured in the foreign scan (006eb930 line 39), so it finds a *second* former-capital city; followed by `lost_a_city(O,1,1,h')` line 449 |
| W2 `Leader::recapture_capital`, timer≠0 arm: `stamp = trunc((stamp − frame)·m/256)` toward zero | yes | yes | agree | **confirmed**: 6d1bb2-6d1bd7 (`sub 0x550; imul 0x41c; cltd; and 0xff; add; sar 8`) |
| W2 caller and E3 predicate: `capture_city` 7350e2, taker's new city has `0x10`, `find_capital(A, skip (new, A))` `idx<0` or holder ≠ A | yes | yes | agree | **confirmed**: decompile 1146-1153; sole `call 0x6d1ba0` at 7350e2 |
| W3 `Build::activate` FC arm: `stamp −= frame` then `trunc(stamp·m/256)`; timer cleared | yes | yes | agree | **confirmed**: 625f64-625fc8 |
| W3 predicate: wonder type `0x20e..0x21e` or the virtual test; `init_wonder` ≥ 0 as a **signed short**; `is(0x213)`; owner's timer ≠ 0; not gated by `captured` | yes | yes | agree | **confirmed**: decompile 1250-1270 (`if (-1 < (short)iVar18)`), 1409-1417; the `param_1 == 0` block opened at 1271 closes before 1296 |
| W4 refresh in `process_all`: each of slots **0..7** with `leader_flags & 2`, `ratio != 0 && frame % ratio == 0`, `stamp < 0 && timer == 0` → `+1`; after `process_elimination` | yes (0..7) | yes ("each slot") | agree | **confirmed**: decompile 49-55; loop bound `0xe71af0 = 0xe3a390 + 8·0x6eec`; listing 6ed381-6ed394 (`jns` skip, `cmp %edx,0x418` with `edx` the zero remainder, `incl; store`) |
| W5 `Leader::init` stores 0, top level, every call | yes | yes | agree | **confirmed**: decompile 310 (two-space indent, after the `do…while` closing at 305); 6e3f89 |
| W6 `walk_data` on a load | — | yes | one-sided | **confirmed**, load-only (see the nets) |
| reset | `Leader::init`, memsets | same + load | agree | **confirmed** |
| reader `process_elimination`: `elimination == 1`, timer ≠ 0, `r − (frame − stamp) ≤ 0` signed → `defeat(1, by)` | yes | yes | agree | **confirmed**: 6b8a30-6b8a5e (`sub; test; jg` skip). `by` = holder from `find_capital(-1,-1)` when `gamec` (0xc061e8) elimination is 1, else the leader itself (6b8a60-6b8a75); the outer gate read `game` (0xc061ec) |
| reader `IFaceMainBase::draw_timers` | via `retake_capital`'s caller list | yes, 80c407 | agree | **confirmed**: `call 0x594530` at 80c40e; UI only |
| reader `log_data` | yes | yes | agree | confirmed |
| crate: `refresh_victory_timers` (`territory/victory.rs:88`), called at `core/frame.rs:194`, pays only `popwin_stamp` | yes | yes | agree | **confirmed** (`victory.rs:88-97`). The original's `process_elimination` runs just before this refresh in the same per-leader body, so its port goes immediately before `frame.rs:194` |
| crate: `Tally::lost_capital_frame` (`cities/city.rs:122`) set/cleared at `city.rs:3213-3224`, written and never read | yes | yes | agree | **confirmed**: `grep -rn lost_capital crates/sim/src` finds the field, the two stores and the checksum rows `checksum/leaders.rs:100-102` only |
| crate: E2 home `Sim::close_building` (`city.rs:2782`); E4 home `Sim::activate` (`city.rs:2420`) — second names `kremlin_activate` (`cities/nations.rs:467`, called `city.rs:2475`) as the neighbour | yes | yes | agree | **confirmed**: all four functions exist at those lines |
| plain game | yes | yes | agree | **confirmed**: E1's stores are ungated by `elimination` (only 6dc97c's messages are); W3 is also reached by a **capture** of a Forbidden City (activate's FC arm is not gated by `captured`), which stops the *taker's* running clock — a plain-game path neither reader drew out |
| the E2-during-capture ordering (second: open) | — | open | one-sided | **open**: whether `Build::close` runs on a Forbidden City inside `capture_city` (it is a civilian member, which is `swap_team`ed and `activate(1,1,0)`ed, not closed, per `docs/CITIES.md` §7.3 step 4) — settle by reading `Build::swap_team`'s body for a `close` call |
| `korean_ancient_pair.rs:326` `lost_capital_stamp 5639` on frame 5639 | yes | yes | agree | **confirmed** as a data point (the file holds it); a first loss from stamp 0 gives `frame` exactly |

**CITIES.md §8.5 against the readings.** The spawn asked for the first
reading's repayment rate to be settled against `docs/CITIES.md` §8.5 line
1951-1952, which says the debt "decays one frame per frame while the capital
is held". **Verdict: §8.5 is corrected; the first reading (and the second)
stand.** The debt gains `+1` only on frames with `frame % TIMER_REFRESH_RATIO
== 0`, so **one frame per five** at the shipped value: `Leaders::process_all
(006ed2a0)` decompile 49-55, listing 6ed381-6ed394; `rules.xml` line 727
spells the intent itself (`<TIMER_REFRESH_RATIO value="5 seconds with timer
off removes 1 second from clock"/>`); the crate pins
`timer_refresh_ratio: 5` (`core/tuning.rs:1923`). Three smaller
corrections to the same paragraph, each by the deciding line:

- `stamp = ((stamp − frame) × m) >> 8` (recapture) and `× modifier >> 8`
  (the countdown) are **truncation toward zero**, not an arithmetic shift:
  both add `0xff` to a negative product first (6d1bcb-6d1bd4; 594590-59459f).
  The recapture product is `≤ 0`, so a floor would differ by one frame
  whenever it is not a multiple of 256.
- "How the base countdown is first seeded is not established": **settled**.
  `Leader::init (006e3930)` line 310 zeroes the stamp (and 311 the timer) on
  every init, so the first loss stores `frame + 0`; the countdown is
  `Game::retake_capital (00594530)` = `max(1, (xs·RETAKE_CAPITAL + S/2) / S)`
  with `S` the Standard map size `+0x144` off `*(0xe7fca8)` (594548-594572;
  70 per `docs/MAPGEN.md` line 4273), then `trunc(· m / 256)`. `rules.xml`
  line 728 gives `RETAKE_CAPITAL` 3600. This also settles `docs/CITIES.md`'s
  open item 6, "the seeding of `lost_capital_stamp`" and "the map-size field
  `retake_capital` divides by".
- `lost_capital` "once" is right in effect but the guard is the timer, so a
  second loss while the clock runs is a no-op for all three leaves, and a
  loss after a recapture runs again and folds the remaining debt in.

### The rule, as a builder should port it (stamp)

- **Loss** (`timer == 0` only; E1 from the capture of a `0x10` city whose
  owner holds no live Forbidden City and no other live own `0x10` city; E2
  from an **active** (`flags & 4`) Forbidden City closing outside
  `elimination == 3` while another leader holds a city carrying the owner's
  was-capital bit): after the modifier is set, `stamp = frame + (stamp·256)
  / m`, C truncating division (`stamp ≤ 0` here).
- **Stop** (`timer != 0` only; E3 the taker's new city has `0x10` and it holds
  no other; E4 a Forbidden City's activation for its owner, captured or
  built): `stamp = (stamp − frame)·m / 256`, truncating toward zero.
- **Refresh**: per frame, slots 0..7 with `leader_flags & 2`, after the
  elimination check: if `frame % 5 == 0` and `stamp < 0` and `timer == 0`,
  `stamp += 1`.
- **Reset**: 0 at every `Leader::init`; whole-record zero at game init/close.
- **Crate**: field beside `popwin_stamp` (`ai.rs:705`); E1/E3 replace the
  `lost_capital_frame` stores at `cities/city.rs:3213-3224`; E2 in
  `Sim::close_building` (`city.rs:2782`); E4 beside `kremlin_activate` from
  `Sim::activate` (`city.rs:2475`); refresh as a second clause of
  `refresh_victory_timers` (`territory/victory.rs:88`); the countdown as a
  port of `process_elimination` just before `core/frame.rs:194`.
- **Plain game**: yes, on any capital capture.
- **Confidence**: high (every store read in the listing; two readers agree
  on all arithmetic). **Open**: whether E2 can run inside a capture
  (`Build::swap_team`'s body); `semaphore` byte 1 bit 8's meaning.

## Leaf 2 — `lost_capital_timer` (+0x418, int, a 0/1 flag)

| claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|
| only constants 0 and 1 are stored | yes | yes | agree | **confirmed**: 6dc88a `movl $1`; 6d1bb8 `movl %edi` with `edi` zeroed at 6d1ba8; 625f71 `movl $0`; 6e3f8f `movl %eax` with `eax` zeroed at 6e3f6d |
| W1 `lost_capital`: 0 → 1, only if 0; re-entry skips modifier and stamp | yes | yes | agree | **confirmed**: 6dc87e-6dc88a; the `jne` lands at 6dc97c, past every store |
| W2 `recapture_capital`: 1 → 0 only if set | yes | yes | agree | **confirmed**: 6d1baa-6d1bb8 |
| W3 `Build::activate` FC arm: → 0 only if set | yes | yes | agree | **confirmed**: 625f64-625f71 (predicate as leaf 1, W3) |
| W4 `Leader::init`: 0, every call | yes | yes | agree | **confirmed**: decompile 311; 6e3f8f |
| bulk: memsets; load | memsets | memsets + load | agree / one-sided | **confirmed** |
| no defeat-time clear | — | yes | one-sided | **confirmed**: `Leader::defeat`/`defeat_by` are not in the export net and no listing store outside the four functions exists |
| E1/E3 writes ungated by `elimination` | yes | yes | agree | **confirmed**: the `cmpb $1, 0x37(game)` tests at 6dc97c and 6d1bdd come after the stores |
| reader `process_elimination` (gate) | yes | yes | agree | **confirmed**: 6b8a36 |
| reader the refresh (only while 0) | yes | yes | agree | **confirmed**: 6ed38b |
| reader `IFaceMainBase::draw_timers (0080b060)` | — | yes, 80c3ab | one-sided | **confirmed**: `cmpl $0, 0x418(%ecx,%esi)` at 80c3ab; UI only |
| reader `Player::leave_game (006ee010)`: `leader_flags & 3 == 3` and timer set → `defeat(1, holder)` under `elimination == 1`, else `defeat(1, self)`; otherwise `defeat(param_1, −1)` | yes | yes | agree | **confirmed**: decompile 37-49; 6ee0a8. Reach (first left open, second said departure only): its callers are `Player::resign (006edcb0)` (from `Player::quit (006edc00)`, `CommandManager::issue_resign (00941170)`, `CommandPackage::process_resign (009438c0)`) and `Player::drop (006ede80)`, so a resign **command** reaches it in single player too. It only chooses who is credited with the defeat; it writes none of the three leaves |
| crate: no field; `lost_capital_frame.is_some()` (`city.rs:122`) the analogue; the AI's `capital_lost` (`ai/ai_build.rs:296`) recomputes from the cities | yes | yes (analogue only) | agree / one-sided | **confirmed** for the field; `capital_lost` exists at `crates/sim/src/ai/ai_build.rs` (not re-read here; the claim is a pointer, not a rule) |
| plain game | yes | yes | agree | **confirmed** |

### The rule, as a builder should port it (timer)

- A 0/1 flag. **Set to 1** by the loss (E1/E2) only when 0, which also
  gates the modifier and stamp; **set to 0** by the stop (E3/E4) only when 1,
  which also gates the stamp conversion; **0** at every `Leader::init`.
- Readers that matter to the rules: the elimination countdown (runs while
  1), the refresh (runs while 0), and the resign/drop path's choice of
  `by`.
- **Crate**: a field beside `lost_capital_stamp`, replacing
  `lost_capital_frame`'s `Option`; same sites as the stamp.
- **Plain game**: yes. **Confidence**: high, nothing open beyond the leaf 1
  items.

## Leaf 3 — `lost_capital_modifier` (+0x41c, int, 8.8: 0x100 = ×1)

| claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|
| sole value writer `Leader::lost_capital`, timer-0 arm, stores 6dc8fb / 6dc903 / 6dc952 | yes | yes | agree | **confirmed** (nets above) |
| `To = get_team_terr(O)`, `Ta = get_team_terr(A)` (`A` = the call's `param_1`: E1 the captor, E2 the holder, possibly re-picked) | yes | yes | agree | **confirmed**: 6dc894 (`ecx` = `this`), 6dc8a7-6dc8ad (`0xe3a390 + A·0x6eec`) |
| `w = land_size / num_nations` signed, 0 when `num_nations == 0` | yes | yes | agree | **confirmed**: 6dc8b2-6dc8cf (`Game+0x6a0`, `World+0x78`, `cltd; idivl`) |
| either sum 0 → `m = 0x100` | yes | yes | agree | **confirmed**: 6dc8d1-6dc8d9 `je 6dc903` on each sum |
| else `m = trunc((To+w)·256 / (Ta+w))` clamped `[0x80, 0x200]`, signed | yes | yes | agree | **confirmed**: 6dc8db-6dc8fb (`shl 8; cltd; idivl`; `jl` → 0x80; `cmovg` 0x200) |
| the city rescale follows **either** branch: both `city_num` ≠ 0 and `A.city_num < O.city_num` (strict, signed) → `m = trunc(m·(O+2)/(A+2))` clamped `[0x80, 0x300]` | yes | yes ("after either branch") | agree | **confirmed**: 6dc901 `jmp 6dc90d` and 6dc903 falls through to 6dc90d; 6dc925 `cmp; jle` skip; 6dc93d `idivl`; 6dc940-6dc94b. Note: since `(O+2)/(A+2) > 1` here, the result is `≥ m ≥ 0x80`, so the second floor is unreachable; only the 0x300 ceiling binds |
| `m ∈ [0x80, 0x300]`, so the stamp's `idivl` cannot fault | yes | yes | agree | **confirmed** |
| no reset but the memsets (and a load); `Leader::init`'s setup arm leaves it | yes | yes | agree | **confirmed**: `Leader::init` 310-311 store only +0x414/+0x418; 6e3f89/6e3f8f |
| stale between losses, visible only to a whole-record dump: every rule reader is behind `timer != 0` | implied | yes | one-sided | **confirmed**: `retake_capital` is called only from `process_elimination` (behind 6b8a36) and `draw_timers` (behind 80c3ab); `recapture_capital` and the FC arm read it behind their timer tests; `lost_capital` writes it before its own read. A checksum/diff of the field **does** see the stale value, so a port must keep it, not zero it at the stop |
| reader `Game::retake_capital`: `base = 0` if `RETAKE_CAPITAL < 1`, else `max(1, (xs·RC + S/2)/S)`; `limit = trunc(base·m/256)` for `who ≥ 0`; `semaphore` byte 2 bit 2 arms (FC doubles at `has_wonder == 2`, halves via the other of slots 0/1) | yes | yes | agree | **confirmed**: 59453a-594572 (`S/2` by `cltd; sub; sar`, toward zero), 594587-59459f, 5945a2 `testb $2, 0x822(game)`; decompile 24-45 for the arms. Both arms are behind a `semaphore` bit whose meaning neither reader nor this pass read; the FC `== 2` value is `has_wonder`'s CtW bit alone |
| `S` = `+0x144` of `*(0xe7fca8)` | "unnamed" | "the Standard map size's width" | disagree (one named it) | **confirmed** (second): the same record and field `crates/sim/src/world/analyze.rs:17` and `docs/MAPGEN.md` line 4273 name as `map_sizes` record 3, the Standard size, value 70; `retake_capital`'s base is the scale-by-axis expression |
| `city_num` at E1 (both: not established) | open | open | agree (open) | **settled**: both counts **include the captured city**. `capture_city`'s step 3 calls `Build::activate(1,1,0)` on the swapped city building (decompile ~205, vtable `Build::vftable +0x1a8 Build::activate` in `vtables.txt` line 28415), whose city arm (`(flags & 0x20) != 0`, the `else` at decompile 446) increments `A.city_num` at line 503 regardless of `captured`; `O.city_num` falls only in `City::close` (00737550) line 36, which `capture_city` calls at decompile 1157, after both capital calls (1142, 1152). So the rescale is `(O_left + 3)/(A_before + 3)` in terms of the cities each held outside the one changing hands. Confidence medium-high (the decompile's order; `Build::swap_team` not read for a `city_num` store of its own) |
| `territory` at E1 (second: not established) | — | open | one-sided | **settled, medium**: in play the only writer of `+0x9d8` is `GameDaemon::check_borders (00732060)` (decompile 51-60; the other writers are setup and editor paths: `World::compute_all_territory (006b5700)`'s callers are `Setup::*`, `ConquestGame::*`, `ScenarioEditor::*`, `ScenarioRead::import`, `ScenarioFuncSet::*`), called only from `GameDaemon::process_all (00732700)` line 86. So `To`/`Ta` are the last daemon pass's sums, **before** the capture's border change: the loser's still counts the captured city's land |
| `num_nations`' writer | open | open | agree (open) | **open**: read the writers of `Game+0x6a0` (whether it counts gaia or the dead) |
| `get_team_terr`'s ally rule and team-style-7 arm | head only | frame-0 lobby team, then `is_ally`; style 7 not read | agree (partial) | **open** here; not this family's leaf (the crate's `team_terr` is `ai/ai_census.rs:155`) |
| crate: no field, no `retake_capital`; inputs exist (`team_terr` `ai_census.rs:155`, `land_size` `victory.rs:110`); checksum row `checksum/leaders.rs:102`; diff key unheld `rondata/src/diff/checksum_lines.rs:1809` | yes | yes | agree | **confirmed** for the sim crate (`grep -rn retake crates/sim/src` empty; `land_size` at `victory.rs:110`); no `scale_by_axis` exists in `crates/sim/src` (only `mapgen/map_lakes.rs:314 scale_by_area`), so the port adds the axis expression |
| plain game | yes | yes | agree | **confirmed** |

### The rule, as a builder should port it (modifier)

- **Written** only at a loss (timer 0 → 1), before the stamp uses it, with
  `O` the loser and `A` the captor (E1) or holder (E2):
  `w = land_size / num_nations` (0 if none);
  if `To + w == 0` or `Ta + w == 0` then `m = 256`, else
  `m = clamp((To + w)·256 / (Ta + w), 128, 512)`; then if `O.city_num != 0`,
  `A.city_num != 0` and `A.city_num < O.city_num`,
  `m = min((m·(O.city_num + 2)) / (A.city_num + 2), 768)`. All divisions C
  truncating; `To`/`Ta` the team territory sums as of the last
  `check_borders`; both `city_num`s counting the city being captured.
- **Read** by the countdown `limit = trunc(max(1, (xs·3600 + 35) / 70) · m /
  256)` frames (at `S = 70`), by the loss's stamp division and by the stop's
  multiplication.
- **Reset** only by the whole-record zero at game init/close (and a load);
  kept stale after a stop.
- **Crate**: a field beside the stamp; computed in the loss step added to
  `Sim::capture_city` (`cities/city.rs:3169`) and `Sim::close_building`
  (`city.rs:2782`); `retake_capital` beside the `process_elimination` port.
- **Plain game**: yes, every capital loss.
- **Confidence**: high for the formula (listing end to end, two readers
  agree); medium-high for the two input timings above. **Open**:
  `num_nations`' writer; `get_team_terr`'s team-style-7 arm; the
  `semaphore` bits.

## Against the README row 11

The row's writers (`Leader::lost_capital`, `recapture_capital`,
`Build::activate`, `Leader::process`) hold with the amendments both readers
made: the per-frame store that runs is `Leaders::process_all (006ed2a0)`'s
inlined copy, `Leader::process (006b88b0)` is dead; `Leader::init` resets the
stamp and timer (not the modifier); `Build::close (00628980)` is a second
caller of `lost_capital`. The row's "`lost_capital_frame` is a partial copy of
the stamp" is right in shape and the copy has no reader.

## Verdict table

| leaf | claims confirmed / corrected / open | the rule in one line |
|---|---|---|
| `lost_capital_stamp` +0x414 | 24 / 1 / 1 (corrected: `Build::close`'s Forbidden City arm gates on `is_active`, flags bit `0x4`, not bit `0x1`; open: whether that arm can run inside a capture) — plus `docs/CITIES.md` §8.5 **corrected** (one frame per **five**, truncation toward zero, seeding settled) | loss (timer 0): `frame + stamp·256/m`; stop (timer 1, recapture or Forbidden City activation): `(stamp − frame)·m/256` toward zero; `+1` when `frame % 5 == 0`, `stamp < 0`, timer 0; 0 at `Leader::init` |
| `lost_capital_timer` +0x418 | 14 / 0 / 0 | 0/1: set at a loss if clear, cleared at a stop if set, 0 at `Leader::init`; gates the countdown, the refresh and the resign/drop's `by` |
| `lost_capital_modifier` +0x41c | 14 / 0 / 3 (two "not established" input timings settled: both `city_num`s include the captured city; territory is the last `check_borders` sum; open: `num_nations`, `get_team_terr` style 7, the `semaphore` bits) | at a loss: `clamp((To+w)·256/(Ta+w), 128, 512)` (256 if a sum is 0), then `×(O.cities+2)/(A.cities+2)` capped 768 when `0 < A.cities < O.cities`; never reset but by the whole-record zero |
