# Item 4152, family 8 (`diplomacy-tribute`) — first reading

- **Family**: 8, `diplomacy-tribute`.
- **Leaves** (`LeaderData` offsets, `types.txt`): `tribute_stamp` (+0x174
  int[8]), `gift_stamp` (+0x194 int[8]), `counteroffer` (+0x314 int[8]),
  `tribute_demanded` (+0x334 int[8]), `tributes` (+0x498 int[6]),
  `tribute_sent` (+0x860 int), `tribute_received` (+0x864 int).
- **Reader**: Claude Opus 5.5 (`claude-opus-5-5`), wave 1 first reading,
  2026-11-11.
- **Base**: `Leader` (0x6eec) and `LeaderData` (0x6ee4) share offset 0 — a
  `Leader` method prints `who` as `this->field_0x8` (for example
  `Leader::action_clear_tributes (006d1690)`), so a `Leader*` displacement is
  the `LeaderData` offset unchanged. The slot-`i` record is at
  `0xe3a390 + i*0x6eec`; the `Diplomacy dip[8]` array begins at +0x692c
  (`0xe40cbc - 0xe3a390`), 0x5c each.

## The machinery the seven leaves share

Read once here; each leaf section names the part it lives in.

- **Offers are a pair of mirrored ledgers.** `Leader::action_offer (006d1780)`
  adds `amount` to `leaders[me].dip[them].offers[res]` and subtracts it from
  `leaders[them].dip[me].offers[res]`. A positive slot in `A.dip[B].offers`
  means "A gives B". `Leader::action_tribute (006d1770)` is a bare `jmp` to
  `action_offer` (listing `6d1770`; the export prints the folded body), and
  `Leader::action_demand_tribute (006d1750)` calls it with `-amount`. **None
  of the three writes a leaf of this family**; they write the `Diplomacy`
  record (`dip[8]` at +0x692c: `agree` +0, `any_offer` +4, `treaty` +8,
  `offers[6]` +0xc, `dows[6]` +0x24, `attacks[8]` +0x3c, `types.txt`).
- **`Leader::action_respond (006d03c0)`, `this` = T responding to `p1` = P,
  `param_2` = 0 withdraw my agreement, 1 accept, 2 reject** (`action_reject
  (006d0390)` and `CommandPackage::process_reject (00946e90)` pick 0 when
  `T.dip[P].agree == 1`, else 2; `process_accept (00946fb0)` and
  `action_agree (006d03b0)` pass 1). On accept (listing from `6d05a9`):
  1. If T has not yet agreed (`T.dip[P].agree != 1`): for each resource the
     two leaders both have (`type_avail(r,1)` on both), T must hold
     `bucket[r] >= max(T.dip[P].offers[r],0) + attack_price[r]`, else the
     accept is refused (`6d06ec` → `6d0920`). Then **escrow**: for each such
     `r` with `T.dip[P].offers[r] > 0`, T's `bucket[r]` and `escrow[r]`
     (+0x468) each lose the offer (each clamped at 0) and **T's
     `tributes[r]` gains it** (`6d07b5`). A second pass does the same with
     `attack_price[r]` (the price of an attack request in
     `T.dip[P].attacks[]`, from a type's virtual at slot 0x78, doubled when
     the third party is T's ally, **zero unless T is human**) and also
     stores it in `T.dip[P].dows[r]` (`6d0857`, `6d0863`; not type_avail
     gated). Then `T.dip[P].agree = 1` and `P.agendas[T] |= 4`.
  2. A third-party check (an attack target allied to either side cancels the
     deal with `clear_agree`/`Diplomacy::clear_all` on both sides).
  3. **Whether to execute now** (`6d08f5`–`6d0d00`): if P had already agreed
     (`P.dip[T].agree == 1`) go straight to execution (`6d090e`/`6d091b`; the
     decompile's `local_1c = NUM_ITEMTYPES` there is misleading, the jump
     skips the return). Otherwise execute only when the deal asks nothing of
     P — `T.dip[P].treaty == -1`, `Diplomacy::num_attacks(T.dip[P]) == 0`,
     and no `P.dip[T].offers[r] > 0` — else return and wait for P
     (`6d0cfc`).
  4. Execution (`6d0d06`): `T.dip[P].treaty == 2` takes an alliance arm
     (`6d0d1e`) not read here; anything else goes to **the exchange at
     `6d0df9`**. Per resource `r` 0..5 (refund each side's `dows[r]` to its
     bucket and zero it first; then only if both have `r`):
     `g_T = max(T.dip[P].offers[r],0)` (what T gives), `g_P =
     max(P.dip[T].offers[r],0)` (what P gives).
     - T's bucket gains `scale_tribute_P(g_P)`; **P.`tributes[r]` −= g_P;
       P.`tribute_sent` += g_P; T.`tribute_received` +=
       `scale_tribute_P(g_P)`**; `consider_tribute(T, P, g_P, r)`.
     - P's bucket gains `scale_tribute_T(g_T)`; **T.`tributes[r]` −= g_T;
       T.`tribute_sent` += g_T; P.`tribute_received` +=
       `scale_tribute_T(g_T)`**; `consider_tribute(P, T, g_T, r)`.
     - Clamp T.`tributes[r]` and P.`tributes[r]` at 0.
     `scale_tribute_X` is `LeaderData::scale_tribute (006d5240)` **on the
     giver**: percent `pct = epoch[2]·commerce_tribute + base_tribute`
     (`LeaderDataEncrypt::epoch[2]` xor 0x63187, the Commerce line; the two
     from `Constants`), floored at 1; `pct > 100` returns the amount
     unchanged; `pct == 100` likewise; else `amount·pct/100` with a rounding
     add of 0 below 20, 50 below 100, 99 from 100 (integer division,
     truncating). The bucket stores are `LeaderData::bucket_add (0043ed10)`,
     the xor-0x8221 `LeaderDataEncrypt::bucket[r]`.
- **Every `action_respond`, whatever `param_2`, first zeroes
  `T.tribute_demanded[P]` and `T.counteroffer[P]`** (`6d03d8`, `6d03e0`;
  `ebx` = `this`, `edi` = `param_1`), before any branch.
- **`Leader::clear_agree (006d1af0)`**, called from every `action_*` that
  changes an offer and from `action_respond`: when `this.dip[p].agree == 1`,
  for each `r`, first `min(dows[r], tributes[r])` and then (if `offers[r] >
  0`) `min(offers[r], tributes[r])` move from `tributes[r]` back to the
  bucket; then `Diplomacy::clear_dows`, `agree = 0`, `any_offer = 0`.

`docs/journal/2026-10-22-item-3470.md` and `docs/PARKED.md` 3470a (run1086)
already hold a dump of this path from a taunt gift: the ally's stock fell by a
third, the human gained about half of it, `gift_stamp[0]` of the ally read the
frame, and **every `tributes` and `escrow` slot read 0 at end of frame**. The
reading above accounts for all three: escrow and exchange run inside one
`action_respond`, so `tributes[r]` goes +x then −x (net 0, and clamped), and
`escrow` was 0 and is clamped at 0. That is a reading; 3470a's packet check
would make it a diff.

Two oddities of the exchange that touch `tributes` and are not settled here:
the attack-price escrow (step 1's second pass) is added to `tributes[r]` but
the exchange subtracts only `g_T`/`g_P`, and the trailing
`Diplomacy::clear_all` on both records (decompile `:549`–`:550`) refunds
nothing, so a human-paid attack request would leave its price standing in
`tributes[r]` while the exchange's `dows` refund returns it to the bucket.
And the exchange's second `dows` zeroing goes to `0xe40ce0 + p1·0x6f48` =
`leaders[P].dip[P].dows[r]` (`6d0e23`–`6d0e8e`), not `leaders[P].dip[T]`; on
the face of the listing a source slip. Both are readings only.

## Leaf 1 — `tribute_stamp` (+0x174, int[8])

**Writers.**
- Export: `grep -rn 'field_0x174\b'` → `Leader::consider_tribute (006d1360)`
  `:18` only (plus `ConquestStyle::walk_data`'s `lea 0x174`, another class).
  By name, only the reader `LeaderData::is_tribute_period (006d5b90)`.
- Listing (stores to +0x174..+0x190 and abs `0xe3a504`+): `consider_tribute`
  `6d13d8` (`edx` = `this`, `esi` = `param_1`), and **`Leader::init
  (006e3930)` `6e3c21`** (`movl %eax,0xa0(%esi)` with `esi = this+0xd4+4·j`,
  `eax = 0`). Register-relative hits in `ScenarioRead::load_leaders_chunk`
  (`9acf19`–`9acf2b`) are `GameInfo` player slots (byte/short stores beside
  them; the decompile names them `info.player[n]`), `Game::run_oos_recovery`/
  `Game::close` stores are off the pointer at `0xc06214`, not a `Leader`.

**Event and predicate.** `consider_tribute(R = this, G = param_1, amount,
res)` is called only from `action_respond`'s exchange (`6d0f4f`, `6d0fbb`),
once per resource per direction, with R the receiver and G the giver of
`amount` (pre-scale). It returns at once unless `amount > 0`. Then (after the
`gift_stamp` store, leaf 2) it sets `R.tribute_stamp[G] = frame` when **R is
human** (`R.leader_flags & 4`), **or** when all of: `amount >= G.blacken·100`
(`blacken` +0x20c of the giver; integer, units of the bucket), R's
`bucket[res]` (after this resource's `bucket_add`, which ran first, `6d0f0b`
before `6d0f4f`) is **below 3000**, and `res == 2` (WEALTH) or
`R.econ[res] & 4` (+0x450). Value stored: `Game::frame` (a sim frame).

**Reset.** `Leader::init (006e3930)` zeroes all eight, unconditionally (the
per-slot loop `6e3c0d`–`6e3d93` runs on both `param_2` paths).
`Leaders::clear (006ed5d0)` memsets each slot's `[+0, +0x6929)`. No
`reset_score`, no per-frame fold.

**Readers.** `LeaderData::is_tribute_period (006d5b90)`, `(this, p)`: walks
slots `i` 0..7, skipping `this.who` and inactive slots; a slot *qualifies*
when `i == p`; otherwise (and unless team style 7 makes either `i` or `p`
the team-8 neutral, which disqualifies it) when, at frame 0, `i` and `p`
share a lobby team 0..3, or, after frame 0, `is_ally(leaders[i], p)`. **At the first qualifying slot it returns 0 if the
stamp is 0** (`6d5c8e` → `6d5cd0`, no further scan), 1 if `frame − stamp <
3000`, else it moves on. So "R received a qualifying tribute from p or p's
ally within 3000 frames", but a lower-indexed qualifying slot with a zero
stamp hides a later fresh one. Callers: `Leader::diplomacy (006bc950)` `:1340`
(gates the computer's demand-or-war arm off), `:2683` and `:2790` (price
evaluation of a proposal, asked of the other leader), `Army::do_marching
(006f3df0)` `:85`, `Army::find_target (006f69b0)` `:904` (army target
choice).

**Crate.** No writer: the crate has no `action_respond`, no exchange, no
`consider_tribute`. The reader is a seam: `crates/sim/src/ai/army.rs:3262`
(`fn tribute` returns `false`, documented at `:3104`), and
`crates/sim/src/leaders/diplomacy.rs` stops before the `:1340` gate (the
demand arm is past the team-style gate it does not port). The leaf is walked
for the checksum at `crates/sim/src/checksum/leaders.rs:65`. A builder adds
the stamp in a new `consider_tribute` beside an `action_respond` port (e.g. a
`leaders/tribute.rs`), and replaces `army.rs:3262`'s `false` with
`is_tribute_period`.

**Plain game?** Yes. Any accepted deal that moves goods reaches it: a human's
tribute through the diplomacy window (`CommandPackage::process_tribute
(00946d70)` → `action_offer`, then `process_accept` → `action_respond`), a
computer's taunt gift (`process_taunt`, run1086 entered `consider_tribute` on
661), or a computer's own offers in `Leader::diplomacy`. With a human
receiver the stamp is unconditional. Not behind triggers or CtW (the
`ScenarioFuncSet::offer_tribute`/`accept_offer` route is a second, scenario
route to the same code).

**Confidence.** High on the writer set and the predicate (listing-checked,
`6d1360`–`6d13e1`). The run1086 dump has `tribute_stamp[scan]` in the
coverage list (`crates/rondata/src/diff/coverage.rs:295`); the reading
predicts the human's `tribute_stamp[1]` = 661 on that frame — not checked
here. Not followed: `ConquestStyle::walk_data`, `LoadGame` (a saved game's
restore), `ScenarioRead::load_leader_chunk (009ac080)` (no store to the
leaf by either net).

## Leaf 2 — `gift_stamp` (+0x194, int[8])

**Writers.**
- Export: `field_0x194` in `Leader::consider_tribute (006d1360)` `:12`
  (store), `Leader::process_taunt (006b8cc0)` `:90` (store) and `:82`–`:83`
  (read), `Leader::diplomacy (006bc950)` `:1826`–`:1827` (read). The
  `Border`/`ConquestWin` hits are other classes.
- Listing: `consider_tribute` `6d1384` (`movl %eax,0xe3a524(,%ecx,4)`, `ecx =
  param_1·0x1bbb + this.who`, i.e. `leaders[param_1].gift_stamp[this.who]`),
  `process_taunt` `6b8ecd` (`0x194(%esi,%ebx,4)`, `esi` = `this`), **`Leader::init
  (006e3930)` `6e3c27`** (`0xc0(%esi)`, zero). `ScenarioRead::load_leaders_chunk`
  `9aced0` is a byte store into a `GameInfo` player slot, not this leaf.

**Event and predicate.**
1. `consider_tribute(R, G, amount, res)`: when `amount > 0`,
   **`G.gift_stamp[R] = frame`** — the giver's slot, indexed by the receiver,
   unconditionally (before the `tribute_stamp` test). So every accepted
   exchange stamps each giving side's slot for the other, per resource with
   a positive give (both nets agree; `6d1367` is the only branch before it).
2. `process_taunt(this, code, from)`: `this` must be a computer
   (`leader_flags & 4` clear) and alive (`& 2`), `from != this.who`, and
   (unless `from` is `this`) the two must be mutual allies (`diplos` 2 both
   ways). It records `last_taunt[from] = code`, `taunt_frame[from] = frame`
   (family of taunts). Codes 1..5 ask for FOOD, TIMBER, WEALTH, METAL, OIL.
   Both leaders must answer `type_avail(res,1) == 4`. If (`teams_locked ==
   0` or `this` is neutral) **and** `gift_stamp[from]` is nonzero and under
   4500 frames old (`0x1194`), it refuses with a chat line. Otherwise, if
   `this.bucket[res] > 149` (`0x95 <`): **only when `teams_locked == 0` or
   `this` is neutral, `this.gift_stamp[from] = frame`**; then
   `action_clear_all`, `action_offer(from, res, bucket/3)`, and
   `action_respond(this, from, 1)` — a one-way gift that executes at once
   (machinery step 3), which re-stamps the same slot through
   `consider_tribute` (G = `this`, R = `from`) whatever `teams_locked` says.
   Else a "cannot spare" chat line.

**Reset.** `Leader::init` zeroes all eight (unconditional loop);
`Leaders::clear` memset. Nothing per frame.

**Readers.** `process_taunt` (the 4500-frame refusal above, only when
`teams_locked == 0` or neutral). `Leader::diplomacy` `:1826` (`6bf61e`), in
the per-resource walk of a proposal from `k` (decompile `:1800`–`:1840`): a
`gift_stamp[k]` under 4500 frames sends that resource to `LAB_006bf63d`; the
branch's effect was not followed.

**Crate.** No writer and no reader: the crate has no taunt handling (`grep -i
taunt crates/sim/src` finds only the checksum's leaf list), no
`consider_tribute`. Checksum walk at `crates/sim/src/checksum/leaders.rs:66`.
A builder adds both stores with a `process_taunt` port and the
`consider_tribute` above.

**Plain game?** Yes. The taunt path needs a human allied with a computer (a
team game, team style 1 is enough — run1086's lobby was teams-locked and
the stamp came from `consider_tribute`, `docs/journal/2026-10-22-item-3470.md`
"What the run says"). Any accepted gift between two leaders reaches
`consider_tribute`.

**Confidence.** High; and **diff-adjacent**: run1086's dump shows who 1's
`gift_stamp[0]` = 661 then 681 under `teams_locked ≠ 0`, which is what this
reading predicts (`process_taunt`'s own store skipped, `consider_tribute`'s
made). That closes the attribution question parked as 3470a's second half
on a reading; the packet check named there would make it a diff.

## Leaf 3 — `counteroffer` (+0x314, int[8])

**Writers.**
- Export: `field_0x314` in `Leader::action_respond (006d03c0)` `:61` and
  `Leader::diplomacy (006bc950)` `:2003`, `:2785`, `:2821`, `:3180` (stores),
  `:166`, `:2780` (reads). The `Form`, `GameReportWin`, `RiverSection`,
  `ObjectsOut`, `UnitType`, `GameSpyBack` hits are other classes.
- Listing (stores to +0x314..+0x330): `action_respond` `6d03e0` (`and $0`,
  `ebx` = `this`, `edi` = `param_1`); `Leader::diplomacy` `6bf3ea` (=0),
  `6c005e` (=1), `6c0840` (=0), `6c086b` (=0), all `0x314(%edi,%reg,4)` with
  `edi` = `this` and the index the target `k`; **`Leader::init (006e3930)`
  `6e3d75`** (`0x240(%esi)`, zero). `Game::init_type_backup`'s +0x314..+0x32c
  stores are a type record.

**Event and predicate.** The only setter is the computer's proposal
evaluation in `Leader::diplomacy`, reached at `LAB_006bfa57` when
`LeaderData::any_proposals(this, k)` says `k` has a proposal on the table.
It prices the deal (a quantity built from `aggression[k]`, epochs, difficulty
and the rusher/boomer flags — family 7's and AI.md's ground, not re-read
here), then **subtracts 99 from that price when `counteroffer[k] != 0`**
(`6bfe1c`–`6bfe2a`: `lea -0x63(%ecx)` taken unless the slot is 0). With
`local_54` (the "clear everything" flag: `this.dip[k].agree == 1` or a
`Diplomacy` word of `k` toward `this` equal to 2, decompile `:1218`–`:1224`)
and `local_48` (not followed):
- price ≤ 0 and not the reject case → **`counteroffer[k] = 0`** (`6c086b`);
  if `any_proposals` still holds, accept (`action_respond(this,k,1)`) and
  `good_deeds[k] −= price` (+50 when `weak[k]` and not in a tribute period).
- price > 0, `local_54 == 0`, `local_48 == 0` → **`counteroffer[k] = 1`**
  (`6c005e`): it picks the resource (not KNOWLEDGE) the two both have with
  the largest `resource_value` (at least 64), clears both agreements
  (`clear_agree` both ways) and tells the console player the ask, rounded up
  to a hundred (decompile `:2930`–`:2936`).
- `local_54 != 0` and not (price > 0 and `local_48`) → the rejection arm:
  chat, `counteroffer[k] = 0` (`6c0840`), and if `this.dip[k].agree == 1`
  `action_respond(this,k,0)`.
- Separately (`6bf3ea`, decompile `:2003`): no allies, no victory timer,
  both leaders computers and `aggression[k] == 0` → accept and store 0
  (already 0: `action_respond` cleared it on entry).

**Reset.** Every `action_respond(this, k, *)` zeroes `this.counteroffer[k]`
on entry (`6d03e0`) — accept, reject or withdraw, from either side's
command, `process_taunt`, or `Leader::diplomacy`'s many calls (31 call
sites, listing). `Leader::init` zeroes all eight; `Leaders::clear` memset.

**Readers.** `Leader::diplomacy` `:166` (`6bcadf`): the per-pair cadence —
when `this.dip[k].agree != 1`, a standing `counteroffer[k]` doubles the
period (0x800, or 0x2000 when `tribute_demanded[k]` is set, then ×2; then
÷4 unless `agendas[k] & 8`). And the −99 above.

**Crate.** `crates/sim/src/leaders/diplomacy.rs:177`–`:179` names it: "the
Diplomacy records are not carried — and `tribute_demanded`/`counteroffer`
(`+0x334`/`+0x314`) are never set", and the cadence code at `:180`–`:190`
uses the base 0x800 alone. The offers arm (`LAB_006bf0e0`, `:490`) and the
proposal evaluation are SEAMs. A builder adds the stores to a port of the
proposal-evaluation arm in `Sim::diplomacy_with` and the entry clear to an
`action_respond` port; the cadence term then reads it.

**Plain game?** Yes, in a lobby where a human can propose to a computer
(diplomacy on: team style 0/8/11, where leaders start at peace; the
evaluation also runs in team style 1 against an ally's proposal, not
traced). Not trigger- or CtW-only. No capture is known to have set it
(run1086 read 0 throughout).

**Confidence.** Medium-high: the five stores and the −99 are listing-checked;
the price formula and `local_48`/`local_54`'s meanings are not.

## Leaf 4 — `tribute_demanded` (+0x334, int[8])

**Writers.**
- Export: `field_0x334` in `action_respond` `:60` (store), `Leader::diplomacy`
  `:1567` (store), `:162`, `:1372` (reads). `RiverSection`, `ConquestInvadeWin`,
  `GameReportWin` are other classes.
- Listing (+0x334..+0x350): `action_respond` `6d03d8` (`and $0`);
  `Leader::diplomacy` `6be85d` (`movl $1,0x334(%edi,%esi,4)`, `edi` =
  `this`, `esi` = `k`, confirmed from `6be81d`–`6be848`); **`Leader::init`
  `6e3d7b`** (`0x260(%esi)`, zero). `Game::init_teams` `58b9c4` is a byte
  store (a `GameInfo` slot); `Game::run_oos_recovery`/`Game::close`'s
  `0x344(%eax)` is off `0xc06214`.

**Event and predicate.** One setter: `Leader::diplomacy`'s aggression arm
(the computer `this`, target `k`, `aggression[k] != 0`, not a scenario with
`diplomacy_setting`), at peace with `k` both ways (`diplos` nonzero), past
the team-style gate (team styles 0, 8, 11; team style 7 only under its
team-8 test; everything else jumps to `LAB_006bed3f` first), with
`is_tribute_period(this, k) == 0` and `frame >= local_40` (a seed- and
personality-derived floor, `(((seed + who) & 7) + span·3)·300` frames with
`span` the rusher/boomer draw, decompile `:202`). If any of the war tests holds — `local_54`, a counter
above 3, `local_34`, `hire_stamp[k]` set and under 13501 frames (`0x34bd`),
or `k`'s WEALTH bucket below `(epoch[0]+1)·(count+1)·100` — it goes to war
instead (and its chat line differs when `tribute_demanded[k]` is already
set, `:1372`). Otherwise it **demands**: `action_clear_all(this,k)`, a
random walk of the four non-WEALTH, non-KNOWLEDGE goods placing
`action_offer(k, res, −min(k.bucket[res], cap))` — `cap` the same
`(epoch[0]+1)·(count+1)·100` the WEALTH test computed and stored into
`local_40` — for goods `this` is short of (`econ & 0xc == 4`, own bucket
under 1000, `k` holding over 49; a coin can skip one and add its value to
the WEALTH ask instead), a WEALTH demand `−min(k.bucket[WEALTH], ask)`, then
`action_respond(this, k, 1)` (which zeroes the slot on entry), `agendas[k]
|= 8`, and **`tribute_demanded[k] = 1`** (`6be85d`, after the call).

**Reset.** `action_respond(this, k, *)`'s entry (`6d03d8`) — but note which
side: the human's accept is `action_respond(human, computer, 1)`, which zeroes
`human.tribute_demanded[computer]`, not the computer's slot. The computer's
own slot falls only when the computer itself calls `action_respond(this, k,
…)` (its next demand re-sets it after the call; the war arm's
`action_respond(this,k, agree==1 ? 0 : 2)` at `:1484`–`:1491` when a
`Diplomacy` word of `k` equals 1; the other arms' calls). `Leader::init`
zeroes all eight; `Leaders::clear` memset. So a paid demand leaves
`tribute_demanded[k] = 1` standing; what stops a re-demand is the
`is_tribute_period` gate (leaf 1: the computer is the receiver, and its
`tribute_stamp[k]` is stamped only when the amount clears `k.blacken·100`,
its bucket stays under 3000, and the good is WEALTH or `econ & 4`).

**Readers.** `Leader::diplomacy` `:162` (`6bcacb`): the cadence period is
0x2000 instead of 0x800 while the slot is set and `this.dip[k].agree != 1`.
`:1372` (`6be9f5`): picks the "you ignored my demand" message set on the
war arm (text only; the war decision itself does not read the slot).

**Crate.** As leaf 3: named at `crates/sim/src/leaders/diplomacy.rs:177`–
`:179` as never set; the demand arm lies past the team-style gate the crate
models as "team style 1 goes to `LAB_006bed3f`" (`diplomacy.rs:497`–`:499`,
`docs/AI.md` §220). A builder adds the store with a port of the demand arm
in `Sim::diplomacy_with`, and the period term at `diplomacy.rs:180`.

**Plain game?** Yes, but only in a lobby with diplomacy on — team style 0, 8
or 11 (`docs/AI.md` §220; `docs/PARKED.md` notes every capture but the
coverage pair is team style 1). A skirmish against the computer at team
style 0 reaches it once a computer turns aggressive toward a leader it is at
peace with. Not trigger/CtW-only (`ScenarioData::diplomacy_setting` is a
scenario override that *suppresses* the arm).

**Confidence.** High on the store, its position after `action_respond`, and
the entry clear; medium on the demand composition (the 1000/49 thresholds
and `local_40` read from the decompile, not re-listed); the war tests'
`local_54`/`local_34`/counter meanings not followed.

