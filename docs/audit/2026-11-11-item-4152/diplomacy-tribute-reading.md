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

