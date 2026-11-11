# diplomacy-tribute — blind second reading (item 4152, wave 2)

- **Slug:** `diplomacy-tribute`
- **Leaves (`LeaderData`, size `0x6ee4`):** `tribute_stamp` (+0x174, int[8]),
  `gift_stamp` (+0x194, int[8]), `counteroffer` (+0x314, int[8]),
  `tribute_demanded` (+0x334, int[8]), `tributes` (+0x498, int[6]),
  `tribute_sent` (+0x860, int), `tribute_received` (+0x864, int).
- **Reader model:** Claude Opus 5.5 (`claude-opus-5-5`).
- **Sources:** the decompile export `~/ghidra-projects/decomp/` and the
  `.text` listing (`llvm-objdump`, pre-dumped). Written blind: no reading of
  the first write-up, CHECKSUM §20, PARKED, LEADERS or the item's journals.

## Ground the family stands on

- **Offsets.** `Leader` (0x6eec) is `LeaderData` (0x6ee4) at offset 0 with
  8 bytes after it: `consider_tribute` (006d1360) reads `who` as
  `this+0x8` and `leader_flags` as `*(byte*)this`, the same offsets the
  `LeaderData` layout gives. So a `Leader*` method's `field_0xNNN` is the
  `LeaderData` offset unshifted, and the global record of slot `i` is at
  `0xe3a390 + i*0x6eec` (a store appears as `0xe3a390+off(,%reg,4)` with
  the slot folded in as `i*0x1bbb` words, or as `off(%reg)` off a `Leader*`).
- **Nets.** (a) A scan of the whole `.text` listing for every write-shaped
  instruction (`mov/add/sub/and/or/inc/…` with the memory operand last, and
  `lea` with it first) whose displacement lands in any leaf's byte range,
  register-relative and absolute; (b) the same scan for reads; (c) the
  export's `->leaf` (one hit, `is_tribute_period`) and `field_0x<off>`
  greps. Register-relative hits below `+0x200` were kept only where the
  base register is shown to be a `LeaderData`/`Leader`; the rest were
  checked and discarded (listed per leaf).
- **Whole-record writers, shared by all seven leaves.**
  - `Leaders::clear` (006ed5d0) `memset`s bytes `[0, 0x6929)` of every
    slot's record (it walks from `+0x6c14` and subtracts `0x1b05` words);
    callers `Game::close` (00594b80) and `Game::run_oos_recovery`
    (005869e0).
  - `Leader::init` (006e3930), when its second argument is negative,
    `memset(this, 0, 0x6929)` (listing `6e398c`–`6e3995`).
    `Leaders::init` (006ed850) calls it as `init(slot, -1, slot)` for every
    slot; `Setup::build_game`/`build_leader` pass `LeaderData+0xc`
    instead and skip the memset.
  - `LeaderData::walk_data` (006d6750) walks `[0, 8)` and, for a seated
    record (`leader_flags & 1`), `[8, 0x692a)` as raw bytes: the save
    game reads and writes all seven leaves verbatim.
  - The two score counters are also zeroed by `Leader::reset_score`
    (006e37f0), which `Leader::init` calls unconditionally (decompile line
    140) and `Setup::build_game` (005ac190) calls directly.
- **Goods** (`enums/TypeIndex.txt`): 0 food, 1 timber, 2 wealth,
  3 knowledge, 4 metal, 5 oil. A stockpile is
  `data_encrypted->bucket[g] ^ 0x8221` (`LeaderData::bucket_add`,
  0043ed10), in whole resource units.
- **`leader_flags & 4`** is the human bit (`docs/ARMY.md:103`,
  `LeaderData::is_human`).
- **The deal machinery these leaves sit in** (read to state predicates;
  not leaves). `Diplomacy dip[8]` at `+0x692c`, stride 0x5c: `agree +0`,
  `any_offer +4`, `treaty +8`, `offers[6] +0xc`, `dows[6] +0x24`,
  `attacks[8] +0x3c`. `Leader::action_offer` (006d1780) mirrors an offer:
  `me.dip[k].offers[g] += x` and `k.dip[me].offers[g] -= x`, so a demand
  is a negative offer on the demander's side and a positive one on the
  target's. `Leader::action_respond(me, P, r)` (006d03c0) is the only
  executor: `r == 1` commits my side (below, `tributes`) and sets
  `me.dip[P].agree = 1`; the trade runs at once when P has already agreed
  (`P.dip[me].agree == 1`), or when it is a pure gift (my `dip[P].treaty ==
  -1`, no `attacks`, and none of P's `offers` to me positive); otherwise it
  waits. A treaty-2 deal is refused if either side already has a team mate
  or ally beyond itself (`num_team_members`/`num_allies`).

## `tribute_stamp` (+0x174, int[8])

**1. Writers.**
- `Leader::consider_tribute` (006d1360), `6d13d8 movl %eax, 0x174(%edx,%esi,4)`
  (`edx` = `this`, a `Leader*`; `esi` = first argument). The only
  per-slot store in `.text`.
- Whole-record: the three memsets/walk above.
- Discarded register-relative hits in the range: `Leader::end_process`
  (006b8690) `movw 0x18c(%edx)`, `Leaders::end_process_all` (006ed070)
  `movw 0x18c(%eax)`, `Leader::plan_strategy` (006b9620) and
  `Leader::init` (006e3930) `movw … 0x17e(%reg)` — 16-bit stores at
  odd-word offsets, a different record (the `Leader::init` loop's base is
  `this+0xd4`, listing `6e3b74`, so its `0x17c(%esi)` is `weak`, `+0x250`);
  `Leaders::clear` `0x17c(%esi)`/`lea 0x180(%esi)` (base `+0x6c14`, so
  `+0x6d90`); `Leader::produce_building` (006e1400) `lea 0x180(%eax)`;
  `ScenarioRead::load_leaders_chunk` (009acde0) (base is `GameInfo.player[]`,
  stride 0x8c); `Game::close`/`run_oos_recovery` `0x184(%eax)` (base is the
  global at `0xc06214`, an interface object); the `IFaceDiplo*` vtable
  stores at `0x180(%esi)`; `ConquestLeader::demand_tribute` (007a2460)
  and `ConquestPieces::drop_piece` (007ad6c0), CtW classes with their own
  fields.

**2. Event, step, predicate.** `consider_tribute(this = receiver R,
G = giver, amount, g)` is called twice per good by `action_respond`'s
trade loop, once per direction (listing `6d0f4f`, `6d0fbb`), after the
receiver's stockpile has been credited. With `amount > 0` it sets
`R.tribute_stamp[G] = Game::frame` (sim frames) when **either**
- R is human (`R.leader_flags & 4`), **or**
- all three hold: `amount >= 100 * G.blacken` (`G.blacken` is `+0x20c` of
  the **giver**'s record, listing `6d1396`); R's stockpile of `g`, after
  the credit, is below 3000 (`cmpl $0xbb8`); and `g == 2` (wealth) or
  `R.econ[g] & 4` (`+0x450`, bit 2).

`amount` is the gross amount the giver committed, not the scaled amount
the receiver got. An amount of 0 (the side of a trade that gave nothing in
that good) stamps nothing. Good-by-good: a multi-good gift stamps once per
good, the same frame each time.

**3. Reset.** Only the whole-record memsets (`Leaders::clear`,
`Leader::init(…, -1, …)`) and a save-game load. Nothing ages it out; the
readers compare against a 3000-frame window.

**4. Readers.** `LeaderData::is_tribute_period(me, t)` (006d5b90), its
only reader (listing `6d5b9a lea 0x174(%ecx)`): walk slots `i = 0..7`,
skipping `i == me` and unseated slots; a slot *qualifies* when `i == t`,
or (outside team style 7's team-8 players) at frame 0 `i` is on the same
lobby team 0..3 as `t`, or after frame 0 `LeaderData::is_ally(L[i], t)`.
At the **first** qualifying slot: `me.tribute_stamp[i] == 0` returns 0 at
once (the walk does not go on to later slots — listing `6d5c8e je
6d5cd0`); `frame - stamp < 3000` returns 1; otherwise the walk
continues. Callers: `Leader::diplomacy` (006bc950) three times (listing
`6be639` gates the tribute demand, `6bfd29`, `6c0893` prices a proposal),
`Army::do_marching` (006f3df0, `6f3fa9`) and `Army::find_target`
(006f69b0, `6f6cbe`) — a computer army does not march on or target a
leader who, or whose ally, paid its owner tribute in the last 3000 frames.

**5. Crate.** No writer. The readers are stubbed: `crates/sim/src/ai/army.rs:3262`
and `crates/sim/src/ai/army/target_gate.rs:523` return `false` for
`is_tribute_period`. The checksum leaf is listed at
`crates/sim/src/checksum/leaders.rs:65`. A builder adds the stamp in a
port of `consider_tribute`, called from a port of `action_respond`'s
trade loop; neither exists (no tribute order in `crates/sim/src/command.rs`).
Until then the two stubs equal the original exactly when every stamp is 0.

**6. Plain game?** Only through a trade executing. In a skirmish against
the computer: a human's diplomacy-window tribute or deal
(`CommandPackage::process_tribute` (00946d70) → `action_offer`,
`CommandPackage::process_accept` (00946fb0) → `action_respond`); a
human's resource taunt to an allied computer (`process_taunt`, below);
or a computer's own demand, which needs team style 0, 8 or 11 (or 7's
special case) — the default style 1 skips it (`006bc950`, team-style test
before `LAB_006bed3f`). `ScenarioFuncSet::accept_offer`/`reject_offer`
are the trigger layer. An all-computer game at team style 1 reaches no
writer that I found.

**7. Confidence.** High on the writer set, the predicate and the reader's
early-return (all from the listing). Not read: what `econ[g] & 4` means
(the writers of `econ`), `blacken`'s writers, and the exact
team-style-7 arm of `is_tribute_period`.

## `gift_stamp` (+0x194, int[8])

**1. Writers.**
- `Leader::consider_tribute` (006d1360), `6d1384 movl %eax,
  0xe3a524(,%ecx,4)` with `ecx = G*0x1bbb + R.who` — the **giver's**
  record, indexed by the **receiver**: `G.gift_stamp[R] = frame`.
- `Leader::process_taunt` (006b8cc0), `6b8ecd movl %eax,
  0x194(%esi,%ebx,4)` (`esi` = `this`, `ebx` = the taunter's slot).
- Whole-record: the memsets/walk above.
- Discarded: `Leader::init` `0x19c(%esi)` (base `this+0xd4`, so `dow`
  `+0x270`); `ScenarioRead::load_leaders_chunk` `movb 0x194(%edx)`
  (`GameInfo.player[]`); `Game::close`/`run_oos_recovery` `0x1a0(%eax)`
  (the `0xc06214` object); `PlayerProfile::load`/`default_profile`,
  `Player::leave_game` (`Player`, 0x8c bytes), `Objects::remove`,
  `GraphicPieces::*` — other types.

**2. Event, step, predicate.**
- *Trade:* in `consider_tribute`, whenever `amount > 0`, before and
  independent of the `tribute_stamp` test: `G.gift_stamp[R] = frame`. So
  every executed trade stamps each side that gave a positive amount of
  any good, gifts and two-sided deals alike.
- *Taunt grant:* `process_taunt(this, taunt, k)` runs only for a
  computer (`!(leader_flags & 4)`), active (`leader_flags & 2`) leader,
  `k != me`, and (for `k != who`) `diplos` 2 both ways (mutual allies,
  `+0x74`). Taunts 1–5 ask for food, timber, wealth, metal, oil; both
  sides must have `type_avail(g, 1) == 4`. If (teams unlocked
  (`Game::teams_locked` (00594880) 0) **or** I am neutral) and
  `gift_stamp[k]` is non-zero and `frame - gift_stamp[k] < 4500`
  (`0x1194`), I refuse with a chat line. Otherwise, if my stock of `g` is
  at least 150 (`> 0x95`): when (teams unlocked or neutral)
  `gift_stamp[k] = frame` (`6b8eb1`–`6b8ecd`); then `action_clear_all`,
  `action_offer(k, g, stock/3)` and `action_respond(me, k, 1)` — a pure
  gift, so the trade executes at once and `consider_tribute` stamps the
  same word again on the same frame. With teams locked and not neutral,
  the grant still happens and `consider_tribute` still stamps; only the
  refusal test is skipped.

**3. Reset.** Whole-record memsets and save load only.

**4. Readers.**
- `process_taunt` (above): the 4500-frame refusal.
- `Leader::diplomacy` (006bc950), listing `6bf61e`: pricing k's proposal
  good by good, an amount I am asked for that is positive, at most a
  third of my stock, and asked by an ally is free — unless (team style
  not 0, 8 or 11) `gift_stamp[k]` is non-zero and younger than 4500
  frames, in which case it is charged at `resource_value(g) * amount /
  256` (`6bf63d`–`6bf657`) like any other ask.

**5. Crate.** None: no `consider_tribute`, no `process_taunt`, no
proposal pricing (`crates/sim/src/leaders/diplomacy.rs` stops at
`SEAM: the offers arm`). Checksum leaf
`crates/sim/src/checksum/leaders.rs:66`. A builder adds it to a port of
`consider_tribute` and of `Leader::process_taunt`.

**6. Plain game?** The taunt grant needs a human who sends a resource
taunt to a computer ally — reachable in a default team-style-1 skirmish
with a human allied to a computer, but only by a human's chat taunt. The
trade writer: as for `tribute_stamp`. Not reached by an all-computer
game at team style 1.

**7. Confidence.** High on writers and both predicates. Not read: what
`type_avail(…) == 4` means beyond "available", the callers of
`process_taunt` (`Leader::process` (006b88b0), `Leader::process_taunts`
(006b8ba0), `Leaders::process_all` (006ed2a0)) and how a chat taunt
reaches them.

## `counteroffer` (+0x314, int[8])

A 0/1 flag on a computer leader, per other leader `k`: "I answered `k`'s
proposal with a counter-offer and am waiting".

**1. Writers.** All register-relative off `this` (`edi`/`ebx` = the
`Leader*`, index = `k`):
- `Leader::diplomacy` (006bc950): `6c005e` `= 1`; `6bf3ea`, `6c0840`,
  `6c086b` `= 0`.
- `Leader::action_respond` (006d03c0): `6d03e0 andl $0x0,
  0x314(%ebx,%edi,4)` — `this.counteroffer[P] = 0` on entry, for every
  response code.
- Whole-record: memsets/walk.
- Discarded: `Leader::end_process`/`Leaders::end_process_all` `movw
  0x330(…)` (16-bit, other record); `ScenarioRead::load_leaders_chunk`
  (`GameInfo.player[]`); `Game::close`/`run_oos_recovery` `0x328(%eax)`
  (`0xc06214` object); `IFaceDiplo::*` (`this` is the window);
  `PlayerProfile::*`.

**2. Event, step, predicate.** All inside `Leader::diplomacy`'s
evaluation of a proposal `k` has made to me (a computer: the function
returns at once for `leader_flags & 4`, `semaphore[1] & 2` or `ai_off`).
The evaluation builds a shortfall `s` (what the deal is worth to me
minus what I get, in the function's value units); with
`counteroffer[k] != 0` it forgives 99: `s' = s - 99` (`6bfe1c lea
-0x63(%ecx)`, `cmovel`).
- `= 1` (`6c005e`): `s' > 0` and the "already asked" local (`-0x44`) is
  0. I pick the good with the highest `LeaderData::resource_value` (at
  least 0x40; default wealth; knowledge never), set `counteroffer[k] = 1`,
  then `clear_agree(k, me)` and `clear_agree(me, k)` (which refund any
  committed `tributes`, below), and go on to build the counter-offer of
  `ceil(s' * 256 / value)` rounded up to a multiple of 100 (`6c004a`–
  `6c0080`).
- `= 0` (`6c086b`): `s' <= 0` — I accept: clear, then if
  `LeaderData::any_proposals(k)` call `action_respond(me, k, 1)` (which
  clears it again) and charge the worth to `+0xd4[k]`.
- `= 0` (`6c0840`): reject — stamp `k.got_diplo_message = frame`, clear,
  then `action_respond(me, k, 0)` if `dip[k].agree == 1` else
  `action_respond(me, k, 2)`.
- `= 0` (`6bf3ea`): neither side human and the local at `-0x28` is 0 —
  `action_respond(me, k, 1)` then clear.
- `action_respond` (006d03c0) clears `this.counteroffer[P]` before
  anything else, on accept, reject and cancel alike.

**3. Reset.** The clears above, and the whole-record memsets/load.

**4. Readers.** `Leader::diplomacy` only: the per-pair cadence
(`6bcadf`, below under `tribute_demanded`: doubles the period while
`dip[k].agree != 1`) and the 99 forgiveness (`6bfe1f`).

**5. Crate.** `crates/sim/src/leaders/diplomacy.rs:177–179` names it in
a SEAM: "`tribute_demanded`/`counteroffer` … are never set", and the
cadence in `diplomacy_with` (same file, from line 182) leaves out both
terms; the proposal evaluation is not ported (`SEAM: the offers arm`,
`LAB_006bf0e0`). No `action_respond`. Checksum leaf
`crates/sim/src/checksum/leaders.rs:85`. A builder adds the writers to a
port of `diplomacy_with`'s offers arm and of `action_respond`.

**6. Plain game?** Needs a proposal to a computer. A human can make one
from the diplomacy window in any skirmish where the window allows it; a
computer's proposals come from its own diplomacy arms, which at the
default team style 1 do not reach the demand (see `tribute_demanded`).
Not reached by an all-computer team-style-1 game that I found.

**7. Confidence.** High that these are all the writers and on each
store's predicate as stated; medium on the meaning of the locals
(`-0x44`, `-0x28`, `-0x50`) I name loosely — the 3,282-line evaluation
was read around each site, not end to end.

## `tribute_demanded` (+0x334, int[8])

A 0/1 flag on a computer leader: "I have demanded tribute of `k`".

**1. Writers.**
- `Leader::diplomacy` (006bc950), `6be85d movl $0x1,
  0x334(%edi,%esi,4)` (`edi` = me, `esi` = k).
- `Leader::action_respond` (006d03c0), `6d03d8 andl $0x0,
  0x334(%ebx,%edi,4)` — `this.tribute_demanded[P] = 0` on entry.
- Whole-record: memsets/walk.
- Discarded: `ScenarioRead::load_leaders_chunk` (`GameInfo.player[]`);
  `Game::close`/`run_oos_recovery` `0x344(%eax)`; `PlayerProfile::*`.

**2. Event, step, predicate.** The demand is the computer's alternative
to declaring war. In `Leader::diplomacy`, for a target `k` that has
passed the per-pair cadence, that I want war with and am not at war
with (`diplos` non-zero both ways, or `k == me`), past the rush-rules
gate, and **only when the team style is 0, 8 or 11** (or team style 7's
special case; style 1 goes to `LAB_006bed3f`), the arm at `6be639`:
- skip if `is_tribute_period(me, k)` (above) or `frame` is below my
  start threshold `(3*d + ((seed + who) & 7)) * 300` frames, where
  `d = 4 + seed%3` for a rusher, `8 + seed%6` for a boomer,
  `6 + seed%5` otherwise (`6bcb93`–`6bcbcc`; `seed` is `GameInfo.seed`);
- go to the declare-war branch instead if a pending agreement with `k`
  was just cleared (local `-0x50`, set at `6be4a8` when my
  `dip[k].agree == 1` or `k.dip[me].agree == 2`, after
  `action_clear_all`), `k.blacken >= 4`, the local at `-0x30` is
  set, `hire_stamp[k]` is younger than 13501 frames (`0x34bc`), or `k`'s
  wealth is below `D = (my epoch word +0xe8 ^ 0x63187, plus 1) *
  (k.blacken + 1) * 100`;
- otherwise demand: `action_clear_all(me, k)`; from a random start
  (`game_random % 6`) offer `-min(k's stock, D)` of each good that is
  not wealth or knowledge, available to both, flagged `econ[g] & 0xc ==
  4` for me, below 1000 in my stock and above 49 in `k`'s (one at a
  time, by reservoir draw), else grow the wealth ask; then offer
  `-min(ask, k's wealth)` of wealth; `action_respond(me, k, 1)` (which
  clears `tribute_demanded[k]` and commits nothing of mine, my offers
  all being negative); `agendas[k] |= 8`; then `tribute_demanded[k] =
  1` (`6be85d`, after the call, so the net is 1).
`action_respond(this, P, ·)` clears `this.tribute_demanded[P]`. `k`'s
own answer runs `action_respond(k, me, ·)` and so clears `k`'s word, not
mine: mine stays 1 until I call `action_respond(me, k, ·)` myself
(another arm of my `Leader::diplomacy`, a trigger) or a memset.

**3. Reset.** As above.

**4. Readers.** `Leader::diplomacy`:
- the per-pair cadence (`6bcacb`): period `P = 0x800` frames; when
  `dip[k].agree != 1`, `P = 0x2000` (`local_a8`) if `tribute_demanded[k]`,
  then `P *= 2` if `counteroffer[k]`; `P /= 4` unless `agendas[k] & 8`;
  the pair runs when `(P - 1) & ((k + 8*me) * (P >> 6) + frame) == 0`, or
  at once when `agendas[k] & 4`;
- the war message (`6be9f5`): with `tribute_demanded[k]` set, the chat
  line comes from a different table of seven.

**5. Crate.** As `counteroffer`: the SEAM at
`crates/sim/src/leaders/diplomacy.rs:177–179`, and the cadence in
`diplomacy_with` omits the `0x2000` term. A builder adds the store to
`diplomacy_with` past its `SEAM: the elimination and capital arms of
team styles 0, 8 and 11` (the crate returns there for those styles).
Checksum leaf `crates/sim/src/checksum/leaders.rs:86`.

**6. Plain game?** No at the default team style 1: the arm needs team
style 0, 8 or 11 (`docs/ORACLE.md:2911`, `--profile TEAM_STYLE=0`, is the
lobby key that reaches it). A skirmish with diplomacy on reaches it with
no human.

**7. Confidence.** High on both writers and the team-style gate (listing
and decompile agree); medium on the side conditions named by local
(`-0x30`) and on the demand amounts, which are not this leaf's.

## `tributes` (+0x498, int[6])

Per good, what this leader has taken out of its stockpile and holds
committed to deals not yet executed — an escrow ledger in resource
units.

**1. Writers.**
- `Leader::action_respond` (006d03c0):
  - `6d07b5 addl %esi, 0x498(%ebx,%edx,4)` and `6d0857 addl %edx,
    0x498(%ebx,%esi,4)` (`ebx` = `this`, index = good) — the commit;
  - `6d0e19 lea 0xe3a828(,%eax,4)` (pointer to `P.tributes[0]`, walked
    by 4 per good), `6d0f17 subl %edi, (%eax)` through it, `6d0f7e subl
    %ebx, 0xe3a828(,%eax,4)` (`me.tributes[g]`) — the trade;
  - `6d0fd4`/`6d0fe2` — both clamped to 0 after the trade.
- `Leader::clear_agree` (006d1af0), `6d1b10 lea 0x498(%edx), %esi` then
  stores through `esi` — the refund.
- Whole-record: memsets/walk.
- Discarded: `IFaceDiplo*`/`IFaceDiploNeg*` `0x4a8`/`0x4ac(%esi)`
  (window fields), `ScenarioRead::load_leaders_chunk`
  (`GameInfo.player[]`), `IFaceAirGarrison::update_visibility`.

**2. Event, step, predicate.**
- *Commit* — `action_respond(me, P, 1)` when `me.dip[P].agree != 1`,
  after an affordability pass: for each good `g` available to both
  (`type_avail(g, 1)`), if `stock[g] < offer_g + price_g` the whole
  response is abandoned (a message to the console player, no state
  change). Then, per good available to both with `me.dip[P].offers[g] >
  0`: `stock[g] = max(stock[g] - o, 0)`, `escrow[g] (+0x468) =
  max(escrow[g] - o, 0)`, `tributes[g] += o` — the full `o`, not the
  floored difference. Then per good with a non-zero `price_g`: the same
  three steps with `price_g`, and `me.dip[P].dows[g] = price_g`.
  `price_g` is the cost of the `attacks` I asked `P` to make: for each
  seated `j` with `me.dip[P].attacks[j] != 0` that I am not at war with,
  `UnitType vtable +0x78 (g, me, …)` times 2 if I am allied to `j`, and
  times 0 unless I am human (`leader_flags & 4`) — so only a human pays
  it. Then `me.dip[P].agree = 1` and `P.agendas[me] |= 4`.
- *Trade* — the loop at `LAB_006d0df9` over `g = 0..5` (listing
  `6d0df9`–`6d0ffa`). First, each side's `dip[other].dows[g]` is
  credited back to its own stockpile (`bucket_add`) and zeroed — **with
  no matching decrement of `tributes`**. (The zeroing of P's side uses
  `0xe40ce0 + P*0x6f48`, i.e. `P.dip[P].dows[g]`, not `P.dip[me].dows[g]`
  — a slot that looks wrong but is what the listing does, `6d0e23`.)
  Then, if `g` is available to both: `a = max(me.dip[P].offers[g], 0)`,
  `b = max(P.dip[me].offers[g], 0)`; `me` receives
  `P.scale_tribute(b)`, `P.tributes[g] -= b`; `P` receives
  `me.scale_tribute(a)`, `me.tributes[g] -= a`; then both
  `tributes[g]` are floored at 0. (A refunded `dows` amount therefore
  stays in `tributes` until that floor or a later `clear_agree` takes
  it; I did not find what charges the war price after the refund —
  plausibly `action_declare`, which the trade's tail calls for each
  requested attack when `Game::war_allowed`.)
- *Refund* — `clear_agree(me, P)`, only when `me.dip[P].agree == 1`:
  per good, if `dows[g] != 0` move `r = min(tributes[g], dows[g])` from
  `tributes[g]` back to the stockpile; then if `offers[g] > 0` move
  `r = min(tributes[g], offers[g])` likewise; then `clear_dows`,
  `agree = 0`, `any_offer = 0`. Callers: `action_offer`,
  `action_clear_all`, `action_clear_tributes`, `action_attack`,
  `action_treaty`, `action_respond` (decline and cancel paths),
  `Leader::diplomacy` (the counter-offer, both sides),
  `ScenarioFuncSet::make_counter_offer`.

**3. Reset.** Memsets/load; otherwise only the trade's floor and the
refund lower it.

**4. Readers.** `clear_agree` (the `min`) and `action_respond` (the
floor). No interface or dump reader found by the read scan; the
`log_data` and `EndGameWin` hits at these offsets are other bases.

**5. Crate.** None: the crate has no `Diplomacy` records (the SEAM at
`crates/sim/src/leaders/diplomacy.rs:177`), no `action_offer`,
`action_respond` or `clear_agree`, and no tribute order. Checksum leaf
`crates/sim/src/checksum/leaders.rs:118`. A builder needs the
`Diplomacy` record first; the counter belongs in a port of
`action_respond` and `clear_agree`, beside the stockpile code in
`crates/sim/src/economy.rs`.

**6. Plain game?** As `tribute_stamp`: a human's tribute or deal, a
human's taunt to a computer ally, or a computer's demand under team
style 0/8/11. The human attack price needs a human. An all-computer
game at team style 1: not reached that I found.

**7. Confidence.** High on all store sites and arithmetic (listing).
Medium on the `dows` refund reading and its consequence for this
counter; not read: `UnitType` vtable `+0x78` (named only by slot), and
`action_declare`'s own charge.

## `tribute_sent` (+0x860, int) and `tribute_received` (+0x864, int)

Lifetime score counters: gross resources given, and resources received
after the giver's scaling.

**1. Writers.**
- `Leader::action_respond` (006d03c0), the trade loop:
  `6d0f1c addl %edi, 0xe3abf0(%eax)` (`eax = P*0x6eec`: `P.tribute_sent
  += b`), `6d0f31 addl %eax, 0xe3abf4(%esi)` (`esi = me*0x6eec`:
  `me.tribute_received += P.scale_tribute(b)`), `6d0f8c addl %ebx,
  0xe3abf0(%eax)` (`me.tribute_sent += a`), `6d0faf addl %eax,
  0xe3abf4(%ecx)` (`P.tribute_received += me.scale_tribute(a)`).
- `Leader::reset_score` (006e37f0), `6e3895`/`6e389b` — both to 0.
- Whole-record: memsets/walk.
- The only other `0x860`/`0x864` references in `.text` are other
  records (`Constants::init`/`log_data`, `Build::activate`,
  `Tribe::parse`, `TileSet::init`, `GraphicPieces::~GraphicPieces`, and
  `LeaderData::get_support_count`/`Leader::gain_tech`, which read
  `0x864` off the globals at `0xc061e4`/`0xc061f0`, not a leader).

**2. Event, step, predicate.** Per good `g = 0..5` of an executed trade,
when `g` is available to both: the giver's `tribute_sent` adds the gross
`max(offer, 0)` and the receiver's `tribute_received` adds
`giver.scale_tribute(amount)`. A good unavailable to either side adds
nothing; an amount of 0 adds 0. `LeaderData::scale_tribute` (006d5240):
`pct = (giver's encrypted epoch word [2] ^ 0x63187) * commerce_tribute +
base_tribute` (two `Constants` fields; their load scale I did not read);
`pct < 1` → 1; `pct > 100` → the amount unscaled; `pct == 100` →
unscaled; otherwise `x*pct/100` for `x < 20`, `(x*pct + 50)/100` for
`20 <= x < 100`, `(x*pct + 99)/100` for `x >= 100` (signed division,
non-negative operands).

**3. Reset.** `Leader::reset_score` — called by `Leader::init` (every
path) and `Setup::build_game`; the memsets; save load.

**4. Readers.** `AchieveChartPane::set_chart_data` (007f2c40), listing
`7f2e84`/`7f2eb0`, off `GameAccessConst::leadersc` (`0xc061e0`): the
end-of-game chart. Nothing in the sim reads either.

**5. Crate.** No writer; the score counters beside it are ported one by
one (`crates/sim/src/economy.rs:700–704`, `goody_box_resources`, cites
`reset_score`). Checksum leaves `crates/sim/src/checksum/leaders.rs:178–179`.
A builder adds both to the port of `action_respond`'s trade loop and to
whatever stands for `reset_score` at leader setup.

**6. Plain game?** As `tributes`' trade: a human's tribute or deal, a
human's taunt grant from a computer ally, or a computer demand at team
style 0/8/11. Not by an all-computer team-style-1 game that I found.

**7. Confidence.** High (listing, four stores, one reset). Not read:
`Constants::init`'s load of `commerce_tribute`/`base_tribute`, and which
epoch index 2 is.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `tribute_stamp` [8] | `Leader::consider_tribute` (006d1360) | `R.tribute_stamp[G] = frame` on a trade with amount > 0, when R human, or amount ≥ 100·`G.blacken` and R's post-credit stock of g < 3000 and (g wealth or `R.econ[g] & 4`) | `Leaders::clear` (006ed5d0) / `Leader::init` (006e3930, arg2 < 0) memset; save load | none; readers stubbed false at `ai/army.rs:3262`, `ai/army/target_gate.rs:523` | only via a human's tribute/deal or taunt, or a computer demand at team style 0/8/11 | high |
| `gift_stamp` [8] | `Leader::consider_tribute` (006d1360); `Leader::process_taunt` (006b8cc0) | `G.gift_stamp[R] = frame` on every positive trade amount; and on a computer's grant of an ally's resource taunt (teams unlocked or neutral) | memsets; load | none | taunt: a human with a computer ally, any team style; trade: as above | high |
| `counteroffer` [8] | `Leader::diplomacy` (006bc950) ×4; `Leader::action_respond` (006d03c0) | 1 when a computer counters k's proposal (shortfall − 99 > 0); 0 on its accept/reject and on any `action_respond(this, k, ·)` | the clears; memsets; load | SEAM `leaders/diplomacy.rs:177–179`; cadence omits it | needs a proposal to a computer; not an all-computer style-1 game | high (writers), medium (locals) |
| `tribute_demanded` [8] | `Leader::diplomacy` (006bc950); `Leader::action_respond` (006d03c0) | 1 when a computer demands tribute instead of war (team style 0/8/11); 0 on `action_respond(this, k, ·)` | the clear; memsets; load | SEAM `leaders/diplomacy.rs:177–179`; cadence omits the `0x2000` term | no at team style 1; yes, with no human, at style 0 | high |
| `tributes` [6] | `Leader::action_respond` (006d03c0) (+ commit ×2, − trade ×2, floor ×2); `Leader::clear_agree` (006d1af0) (refund) | + committed offer (and a human's attack price) at agree; − the offer at trade, floor 0; − `min(tributes, dows/offer)` refunded on cancel | memsets; load | none (no `Diplomacy` record) | as `tribute_stamp` | high (sites), medium (`dows` refund effect) |
| `tribute_sent` | `Leader::action_respond` (006d03c0) ×2; `Leader::reset_score` (006e37f0) | + gross amount given per good per trade | `reset_score` (every `Leader::init`, `Setup::build_game`); memsets | none | as `tributes` | high |
| `tribute_received` | `Leader::action_respond` (006d03c0) ×2; `Leader::reset_score` (006e37f0) | + `giver.scale_tribute(amount)` per good per trade | as `tribute_sent` | none | as `tributes` | high |

**Disclosure.** Two greps for unrelated vocabulary (`leader_flags` bit 4,
`TEAM_STYLE`) printed one line each of `docs/CHECKSUM.md` §20
(`:1706`, the `leader_flags` bit table) and three lines of
`docs/PARKED.md` (`:5911`, `:9755`, `:9758`, about team style 0's
lobby). None named a leaf of this family or any writer above; nothing
here rests on them.
