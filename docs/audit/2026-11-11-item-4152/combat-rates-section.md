## 3. The combat rates

### 3.1 Leaves

- `frame_battle`, +0xa4c, `int`. A frame number; 0 means never.
- `average_death_rate` +0xa50, `average_kill_rate` +0xa52,
  `average_damage_rate` +0xa54, `average_hit_rate` +0xa56, `ushort`.
- `deaths_current_frame` +0xa58, `kills_current_frame` +0xa5a,
  `hits_current_frame` +0xa5c, `damage_current_frame` +0xa5e, `ushort`.
- `deaths_fifteen_seconds` +0xa60, `kills_fifteen_seconds` +0xa62,
  `hits_fifteen_seconds` +0xa64, `damage_fifteen_seconds` +0xa66, `ushort`.

Every add below is a 16-bit wrapping add. Slot `i`'s block sits at
`0xe3addc + i·0x6eec`.

### 3.2 The rule

**Writers, complete.** Four event writers feed the `_current_frame` words
and one fold writes everything else. The four `Leader::inc_*_event` have no
caller; their inlined copies are the live sites. Besides the header's bulk
writers, `Leader::init (006e3930)` zeroes all thirteen by explicit stores.
No other store exists (both offset nets and an absolute sweep).

**W1, the blow: `Object::do_damage (0064a480)`.** Exemptions: return if the
blow count argument is `< 1`, or if the attacker is a unit and a decoy
(`unit_masks & 1`). Nothing else returns before the store, and there is no
owner, ally or self test.

    s = 5 if is_siege(attacker type) and target is a building or wall else 1
    leaders[attacker.who].hits_current_frame   += s
    leaders[owner(target)].damage_current_frame += s

It counts blows, never hit points. A blow on one's own object adds to the
same leader's hits and damage.

**W2, the structure kill: `do_damage`, after the take.** Before the take,
record `active = target alive (flags & 1) and is_build (a Build, not a Wall)
and active (flags & 4)`. If the take (vslot `+0x16c`) returns non-zero and
the target is a building or wall:

    k = 5 if active else 2
    leaders[attacker.who].kills_current_frame  += k
    leaders[owner(target)].deaths_current_frame += k

No unit, splash, air or ally gate; razing one's own wall counts a kill and a
death to one leader. The take also returns 1 on a **not-dead** path, and W2
counts it: a building or wall that is not active, whose leader lacks
`leader_flags & 4`, with `hits <= 2·damage`, and whose type has flag
`0x2000` at `+0x60`, is disbanded, giving `k = 2`.

**W3, a unit's death: `Object::take_damage (00652020)`.** A dying building or
wall skips these leaves. A dying unit counts only if it is its squad's
captain, the squad's size is 1, and it is not a decoy. Then `+0x810 += 1`
unconditionally, and only if the attacker index argument `param_7 >= 0`:

    leaders[victim.who].deaths_current_frame += 1
    leaders[param_8].kills_current_frame     += 1

`do_damage` passes the attacker's own index and `who`. The other nine
object callers of the take (attrition, `Wall::process`, scenario functions)
pass `-1`, so attrition and scripts count nothing here. The take's tail
re-calls itself only when the dying unit is a **decoy** with positive
overflow `damage - hits` and a captain index `>= 0` other than itself; the
re-call passes the captain's index and the victim's own `who`. If the
overflow kills the captain, the owner gets one death and one kill of its
own.

**W4, the sniper: `SpellType::cast_sabotage (006720e0)`, unit arm.** A dead
target takes the refund and returns. For a target that is a unit, not a
building or wall, a captain, size 1 and not a decoy:

    leaders[param_4 (target owner)].deaths_current_frame += 1
    leaders[param_2 (caster)].kills_current_frame        += 1

(with `+0x80c` and `+0x44 += cost/5` on the caster). The arm then kills the
target by `Unit::die` (vslot `+0x158`), not by a take, so the credit rests
on the predicate alone, and the unit dies whether or not it qualified. The
**building arm writes no leaf of this family**: its stores are `+0x818` and
`+0x44`, and its take passes `-1`.

**W5, the fold: `Leader::process_event_frame (006ec180)`.** `Game::do_frame
(00591ef0)` calls it for slots 0..7 with `leader_flags & 1`, after
`Achieve::capture_data` and before `frame += 1`. It returns unless
`frame % 50 == 0`. Then, in order, for each of the four kinds X (deaths,
kills, hits, damage):

1. `X_fifteen_seconds += X_current_frame` (the raw count).
2. `c = (X_current_frame · 100) mod 2^16`, stored back.
3. `avg = c == 0 ? (avg · 7) >> 3 : (c + avg) >> 1`, unsigned 32-bit on
   zero-extended words, stored as `u16`; the zero test is on the 16-bit
   product. deaths→+0xa50, kills→+0xa52, hits→+0xa56, damage→+0xa54.
4. The console leader's music mood, from `average_hit_rate +
   average_damage_rate` (thresholds 300, 600, and 2000 only when
   `0xecba20 == 2`) and the rivals' non-zero rates. Writes no leaf.
5. The battle test, below.
6. All four `_current_frame` words to 0.

**The battle test.** With `age = leaders[this.who]`'s age (the encrypted
word `data_encrypted->+0xdc ^ 0x62766`, 0-based), `D =
average_death_rate`, `K = average_kill_rate`, zero-extended, all compares
signed `i32`:

    if D + K >= (age + 1)·125 and (frame_battle == 0 or frame - frame_battle >= 1800):
        if D >= K + 10·age + 20:   Achieve::add_event(1 defeat, who, "")
        elif K >= D + 10·age + 20: Achieve::add_event(0 victory, who, "")
        else: nothing (averages untouched)
        on either event: D = K = 64536 (0xfc18); frame_battle = frame

`frame_battle` is the pre-increment frame, a multiple of 50. The sentinel
reads unsigned (every read zero-extends). Over the 36 empty folds of the
lock-out it decays to 525 (`v·7 >> 3`, 36 times). A consequence of the
arithmetic only: past the lock-out, a window with deaths and no kills halves
`D` toward `c` while `K` keeps 7/8, which can record a **victory** (from 525,
three deaths give `D = 412`, `K = 459`, a lead of 47, enough through age 2).

**Resets.** `_current_frame`: every fold frame (step 6), `Leader::init`.
`_fifteen_seconds`: `Leader::init` only, so despite the name each is a
game-long total mod 2^16. Averages: the fold and the battle event.
`frame_battle`: `Leader::init`. A save-game load restores all thirteen raw.

**Readers that change a decision.** `average_death_rate` and
`average_kill_rate` (the battle test); `frame_battle` (the lock-out);
`average_hit_rate` and `average_damage_rate` (music only). `draw_timers`
displays the averages and `deaths`/`kills_current_frame` (×100), never
`hits_current_frame`. `log_data` and the checksum walk read all thirteen.
`Achieve::add_event (007af660)` writes no leaf.

### 3.3 In the crate

No field or function of the family exists in `crates/sim` beyond the
checksum table (`crates/sim/src/checksum/leaders.rs:264-276`). The fields
belong in `crate::ai::Leader` (`crates/sim/src/ai.rs:637`), beside
`frame_attacked`/`attacked_by` (`:697-698`).

- **W1**: `Sim::do_damage_round` (`crates/sim/src/combat/fight/delivery.rs:70`),
  between the decoy return (`:94-98`) and `treaty_on` (`:107-109`), where
  the original's store also sits; `hit_who` is already `owner_of(target)`
  (`:107`). **Trap**: the crate's `!self.struck(target)` return
  (`:81-83`; `combat/fight/stats.rs:92-100`) precedes the decoy test and
  refuses a non-active unit or a dead or combat-less building, which the
  original's store does not see (open, 3.6).
- **W2**: the `if let Obj::Building(b) = target { if killed` block
  (`delivery.rs:212-222`, `killed` at `:193`), outside its unit/splash/
  air/ally gate, with `active` read before the take. `Obj` has only `Unit`
  and `Building` (`combat.rs:381-384`), so the wall's home is wherever the
  crate keeps walls; unsettled. `killed` must be true on a disbanded
  inactive structure; whether `damage_building` models the disband arm was
  not checked.
- **W3**: the unit `Taken::Died` arm of `Sim::take_damage_typed`
  (`delivery.rs:469`, arm `:537`), gated on captain, squad size 1, not
  decoy, real attacker. `Sim::take_damage` (`:452`) is `#[cfg(test)]`;
  attrition is `Sim::suffer_attrition` (`territory/upkeep.rs:286`) and never
  reaches this arm, so no attrition guard is needed. The decoy tail already
  exists (`:560-596`) and passes the captain as its own attacker, so a
  counter in the `Died` arm credits the owner's own kill there with no
  extra code.
- **W4**: `Sim::cast_sniper_unit` (`delivery.rs:806`), on the
  captain/size/decoy predicate, before `die_with`. **Trap**: its SEAM
  (`:803`) lists `+0xa58` among "the caster's leader counters";
  `deaths_current_frame` goes to the target's owner. **Trap**:
  `cast_sabotage_building`'s SEAM (`:881`) names `+0xa58`/`+0xa5a`/`+0x80c`;
  the building arm writes none of them, and the SEAM should name `+0x818`.
- **W5**: a new fold in `Sim::tick` (`crates/sim/src/core/frame.rs:73`),
  before `self.frame += 1` (`:475`), for slots 0..7 with `leader_flags & 1`.
  `Achieve` is unported.
- **Checksum.** The held arms go beside `"frame_attacked"`
  (`checksum/leaders.rs:1157`). Each of the thirteen is
  `("<leaf> (unheld)", 2)` in `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs:1762-1765, 1776-1779, 1789,
  1797-1798, 1803-1804`), counted on run 1562's frame 91; a port holding a
  leaf moves its row. The verdicts give no figure for how many would agree.

### 3.4 Reached by a plain game

A skirmish against the computer reaches W1 on every blow, W2 on every
structure razed, W3 on every squad's last captain killed in combat, the fold
every 50 frames, and the battle event. W4 needs a spy's sniping, rare but
reachable. At age 0, three unit deaths in one window (`D = 150`) pass the
battle size test. The disband arm's reach is open (3.6). Scenario functions reach the take
with `-1` and count nothing (triggers, cut from v1). `Leaders::clear` runs
at game close and in multiplayer out-of-sync recovery. Gaia slots 8 and 9
never fold.

### 3.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/combat-rates-*.md`); no diff backs it yet.

Verdicts, as the adjudication counts them: the shared writers 52 confirmed,
5 corrected, 1 open; per leaf, `hits_current_frame` 6/1/1,
`damage_current_frame` 5/0/0, `kills_current_frame` 9/1/1,
`deaths_current_frame` 5/1/0, each `_fifteen_seconds` 6/0/0, each average
6/1/0 (one shared correction), `frame_battle` 9/0/0, the gaia rows 2/0/1.
Three questions are open (3.6).

Corrections that changed a rule: the take's tail re-call is the decoy
overflow only; the sniper's unit arm kills by `Unit::die`, and the
building arm counts nothing; the crate's `take_damage` wrapper is test-only;
`draw_timers` does not read `hits_current_frame`; the decay is 525; the
take has nine object call sites. One addition: the disband arm counts in W2.

Confidence is high throughout: every step and predicate is confirmed on the
listing. The verdicts cite no kept-dump value for these leaves.

### 3.6 Not established

- **The crate's `struck` return ahead of W1.** Whether the original ever
  runs `do_damage` on a target `struck` refuses (non-active unit, dead or
  combat-less building). If it does, the original counts the blow and the
  crate as placed cannot. A per-frame `LeadersSync` diff of
  `hits_current_frame` over a fight settles it.
- **The disband arm's reach.** Which types carry flag `0x2000` and which
  leaders hold `leader_flags & 4`; and whether the crate's `damage_building`
  disbands at all. A reading of the flag's loader, then a diff of
  `kills_current_frame` on an inactive structure under attack.
- **Gaia slots 8 and 9.** The fold stops at slot 8, so any store there would
  never fold or clear; `ObjectData::valid_target_const (006472c0)` refuses
  `who > 7` and the crate's searches stop at 8 (`crates/sim/src/world.rs:40-55`),
  so no ordinary blow reaches them. A `LeadersSync` line for slot 8 or 9
  with non-zero `deaths_current_frame` would settle it.
- **The wall's home in the crate** for W2.
- **No diff yet** of "`_fifteen_seconds` is never reset" (a long-fight
  dump) or of the battle event (any capture that logs one).
