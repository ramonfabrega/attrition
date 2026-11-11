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

