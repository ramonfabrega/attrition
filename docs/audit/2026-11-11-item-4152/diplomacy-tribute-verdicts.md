# Item 4152, family 8 (`diplomacy-tribute`): adjudication

- Slug: `diplomacy-tribute`. Leaves: `tribute_stamp` (+0x174 int[8]),
  `gift_stamp` (+0x194 int[8]), `counteroffer` (+0x314 int[8]),
  `tribute_demanded` (+0x334 int[8]), `tributes` (+0x498 int[6]),
  `tribute_sent` (+0x860), `tribute_received` (+0x864).
- Readings: `diplomacy-tribute-reading.md` (first), `diplomacy-tribute-second.md` (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text` listing
  (`exe.s` in the job scratch), `crates/sim` grep. Nothing run of the original.
- Spawn questions: (A) does an all-computer skirmish at the default team
  style reach any writer; (B) `tributes` not reduced for refunded `dows`;
  (C) the zeroed `dows` slot being `P.dip[P]`. Settled in the shared section
  below, then leaf by leaf (appended as settled).

## Shared: the three spawn questions

### (A) Does an all-computer skirmish at team style 1 reach any writer?

The second reading says no (each leaf, section 6); the first answers a
different question (any plain game, a human included) and says yes. Settled
by walking every route into the writers.

1. **Command routes are human-only.** Every `CommandPackage::process_*`
   that reaches `action_offer (006d1780)` or `action_respond (006d03c0)` is
   fed by a `CommandManager::issue_*` (`issue_treaty (00942100)`,
   `issue_accept (00942500)`, `issue_reject (00942600)`, `issue_tribute
   (00942700)`, `issue_demand_tribute (00942810)`, `issue_propose_attack
   (00942920)`), and the only callers of those are `IFaceDiploNeg::*`, the
   `LeaderOut::issue_*` wrappers and `Options::exec (007188c0)` (hotkeys):
   interface code (call-site scan of the listing).
2. **The taunt route is human-only.** `process_taunt (006b8cc0)` acts on
   `incoming_taunt*`, which only `Leader::receive_taunt (006b8bf0)` fills,
   whose only caller is `CommandPackage::process_chat (009454f0)`; the chat
   command is issued by `CommandManager::issue_chat (00942b10)`, called from
   `ChatBox::on_modal_end (0093d560)` and `LeaderOut::issue_chat (006d2c50)`,
   itself called from `IFaceDiploBase::on_button_clicked (008014e0)`,
   `IFaceDiploBase::on_key_click (008017f0)` and `ChatBox::on_modal_end`
   (listing `801722`, `80197c`, `93d6f6`). No sim code sends a chat.
3. **`ScenarioFuncSet::*`** (`offer_tribute (009fc030)`, `accept_offer
   (009fc410)`, ...) is the trigger layer, absent from a random-map skirmish.
4. **`Leader::diplomacy (006bc950)` never originates a deal at team style 1.**
   Its 22 `action_respond` and 4 `action_offer` call sites split into
   (a) answers, every one under `k.dip[me].agree == 1`, `me.dip[k].agree ==
   1` or `any_proposals(me, k)` (`LeaderData::any_proposals (006d5ad0)` reads
   `me.dip[k]`: `treaty != -1` or a nonzero offer or attack, all written only
   by someone's `action_*`; `Leader::init` sets every record to
   `Diplomacy::clear_all (0047e030)`'s `treaty = -1`), and (b) four
   originators, each gated off at style 1:
   - the demand (`6be85d` arm): `6be54c`–`6be556`, team style not 0/8/11
     (and not 7's team-8 case) jumps to `LAB_006bed3f`;
   - the alliance-with-attack offer (decompile `:2310`–`:2314`): `6c09b6`–
     `6c09c8`, team style not 0/8/11 jumps to `6c0a2e`;
   - the peace offer (`LAB_006c0f50`, `:2320`–`:2452`) and everything past
     `6c0a2e`: **`6c0a2e cmpl $0, 0x210(%edi,%esi,4)` / `6c0a39 jne
     6be98d`** — `aggression[k] != 0` leaves the target. At style 1
     `LAB_006bdf25` stores `aggression[k] = 1` for every target that is not
     me or a mutual ally (`6bdf4d`–`6bdf56`), so an enemy never passes; a
     mutual ally (stored 0, `6bdf78`) passes but then fails the
     alliance arm's "at peace, not allied" test and `LAB_006c13ad`'s
     "not allied" test (`:2494`–`:2504`), neither of which offers anything;
   - the gang-up alliance offer to a top-scoring human (`:2600`–`:2635`):
     reached only when the `:2128`–`:2150` disjunction is false, and its
     last disjunct (team style not 0/8/11) is true at style 1 (`6c09c8`).
   So at style 1 with no human, no `Diplomacy` record ever leaves
   `clear_all`, `k.dip[me].agree` stays 0, and **no `action_respond`,
   `consider_tribute` or `clear_agree` call is ever made**.

**Verdict: confirmed (the second reading), with one refinement.** An
all-computer skirmish at team style 1 reaches no *event* writer of any of the
seven leaves. It does reach their *setup* writers: `Leader::init
(006e3930)`'s unconditional zeroing loops (`6e3c21`, `6e3c27`, `6e3d75`,
`6e3d7b`, `6e3e8b`; below) and `Leader::reset_score (006e37f0)` (`6e3895`,
`6e389b`), which store 0 over the 0 that `Leaders::clear` or `Leader::init`'s
`memset` already left. Every leaf therefore reads 0 for the whole of such a
game, and the crate's stubs (`ai/army.rs:3262`, `ai/army/target_gate.rs:523`
returning `false` for `is_tribute_period`) equal the original exactly there.
The deciding lines are `6bdf56` with `6c0a39` (the peace arm), `6c09c8` (the
two alliance arms) and `6be556` (the demand).

### (B) `tributes` not reduced for the refunded `dows`

Both readings raise it; the first calls it a reading only. **Confirmed.**
At accept, the attack-price pass (decompile `:185`–`:204`) lowers T's
bucket and `escrow` by `price[r]` (each clamped at 0), raises
`T.tributes[r]` by `price[r]` (`6d0857`) and stores `T.dip[P].dows[r] =
price[r]` (`6d0863`). At execution the exchange credits each side's `dows`
back to its own bucket (`6d0e57` for T, `6d0e6f` for P) and zeroes the slot,
then subtracts only `max(offers[r], 0)` from `tributes` (`6d0f17`,
`6d0f7e`); the clamps (`6d0fd4`, `6d0fe2`) only lift a negative value. The
tail's two `Diplomacy::clear_all` (`6d12c1` on `T.dip[P]`, `6d12d5` on
`P.dip[T]`) zero the records and touch no ledger, and they reset `agree`, so
no later `clear_agree` for that pair can take the residue back. No other
code charges the price: the tail's attack loop (decompile `:520`–`:541`)
stamps `hire_stamp`/`hire_who` and calls `action_declare`, with no
`bucket_add`. **The rule**: an executed deal in which a *human* side had an
attack request priced (`price` is multiplied by `leader_flags & 4 != 0`,
decompile `:110`) refunds that price in full to the human's bucket and leaves
`tributes[r]` permanently higher by it, until `Leader::init`. Behaviourally
inert (the residue only loosens `clear_agree`'s `min` cap, which an
un-inflated ledger never reaches either), but `tributes` is in the `LEADERS`
dump (below, leaf 5) and the checksum walk (`checksum/leaders.rs:118`), so a
port must keep the residue. A human is required, so no all-computer game
shows it.

### (C) The zeroed `dows` slot is P's own

Both readings raise it. **Confirmed as a listing fact, and inert.** The
exchange's second zeroing is `6d0e8e andl $0x0,(%eax)` through `-0x30(%ebp)`
= `0xe40ce0 + P·0x6f48 + 4r` (`6d0e23`–`6d0e2e`, advanced at `6d0fe9`);
`0x6f48 = 0x6eec + 0x5c`, so the word is `leaders[P].dip[P].dows[r]`. The
credit two instructions before (`6d0e5c`–`6d0e6f`) reads the right word,
`P.dip[T].dows[r]` (`0xe40ce0 + 4·(T·0x17 + P·0x1bbb + r)`), so P is paid
once, and the stale `P.dip[T].dows[r]` is zeroed by the tail's
`clear_all(P.dip[T])` at `6d12d5` before anything reads it (nothing in the
loop or the tail does). The self record `P.dip[P]` is never written nonzero
by any route (no `action_*` targets its own slot), so the stray store
writes 0 over 0. **A port writes `P.dip[T].dows[r] = 0` or nothing; both
match the original.**

