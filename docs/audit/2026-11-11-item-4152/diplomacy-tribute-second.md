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

