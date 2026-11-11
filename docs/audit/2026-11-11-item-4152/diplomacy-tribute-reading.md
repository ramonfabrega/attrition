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
when `i == p`, or under team style 7 neither side is the team-8 neutral and
(at frame 0) `i` and `p` share a team 0..3, or (after frame 0)
`is_ally(leaders[i], p)`. **At the first qualifying slot it returns 0 if the
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

