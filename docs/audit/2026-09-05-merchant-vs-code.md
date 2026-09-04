# MERCHANT.md versus `crates/sim` — 2026-09-05

Reader: Opus, docs-versus-code pass (not a blind reading: this reader saw the
document and the implementation, and nothing else).

Checked: 68 rules across §1–§5. Rows below are the ones where the code and the
document disagree.

Note on the "reached" column: `/Users/rf-studio/ron-audit-scratch/blind-all-68traces.txt`
contains **no MERCHANT.md row at all** — none of the 109 never-entered
functions is cited by this document. (The brief said `Unit::add_cast_order@005e4a60`
was in that list; it is not — `grep 5e4a60` on the file is empty.) So every
row below is **reached**, and the residual uncertainty in each is about
whether a capture *exercises the branch*, not whether the function ran.

## Rows

### R1 — `SEARCH_FRIENDLY` is the asker's own leader, not "mine and my allies'"

| | |
|---|---|
| document | MERCHANT.md §2.2, "`if find_unit(at, SEARCH_FRIENDLY, who, 0x300, FILTER_NOT_ME(o, who), FILTER_TYPE(my type)) >= 0: continue`" and §2.2.1, "Both fold over the eight leaders' unit arrays, apply the same two `Search::valid_filter` gates" |
| code | `crates/sim/src/merchant.rs:193`–`201` (`merchant_refused`'s `sibling` closure), used by both searches at `crates/sim/src/merchant.rs:202` and `:205` |
| document says | the first two searches are `SEARCH_FRIENDLY` — and the document never says what that enumerator resolves to |
| code does | `s.is_ally(who, x.owner)` — the candidate may belong to **any mutual ally**. The code's own comment at `merchant.rs:189`–`191` states the gloss outright: "mine and my allies', never me". The original's `SEARCH_FRIENDLY` is `Search::valid_search`'s **case 1**, `if ((Search *)param_2 != this) return 0` — the iterated leader must *be* the asker. Allies are `SEARCH_ALLIED` (case 4, `is_ally`), which this call site does not pass. |
| difference shows | `UnitsSync` `orders_x`/`orders_y` on a merchant's first think, in a game where a leader has a mutual ally holding a merchant of the same `TypeIndex` near, or ordered at, one of the asker's `new_rares`. **No capture on disk reaches it** — run54/66/67/68 are East Indies 1-v-1 and run74 is Great Lakes 1-v-1, and neither AI has an ally with merchants. |
| reached | reached (`ObjectsData::find_unit@0065ca80`, `Search::valid_search@0067daa0`) |

Evidence. `Search::valid_search@0067daa0` switches on the search index with
`this` = the leader being iterated and `param_2` = the asking leader:
case 1 `param_2 != this → 0`; case 3 `!is_enemy → 0`; case 4 `!is_ally → 0`;
case 5 `is_ally → 0`; case 6 `param_2 == this → 0`; case 7 `is_enemy → 0`.
The seven enumerator names that appear at call sites across the export —
`SEARCH_FRIENDLY`, `SEARCH_ENEMY`, `SEARCH_ALLIED`, `SEARCH_NON_FRIENDLY`,
`SEARCH_NON_ALLIED`, `SEARCH_NON_ENEMY` — map onto those cases one-for-one,
and in exact complementary pairs: 1↔6 (`== this` / `!= this`),
4↔5 (`is_ally` / `!is_ally`), 3↔7 (`is_enemy` / `!is_enemy`). Only case 1 is
the complement of `SEARCH_NON_FRIENDLY`, so `SEARCH_FRIENDLY` is case 1.
Corroboration from a second direction: `ObjectsData::find_unit_ordered@0065bc40`
**ignores its own `param_3`** and passes the literal `1` to `valid_search`
(`funcs/ObjectsData/find_unit_ordered@0065bc40.c:40`), where
`find_unit@0065ca80` passes `param_3` (`:58`, `:146`) — the hardcoded 1 is
the same restriction the 57 `SEARCH_FRIENDLY` call sites want, and
`think_merchant` passes garbage (`(SearchIndexBH)this_00`, i.e. `who`) for
that argument precisely because it is unread. And `Ammo::check_hit@00678d90:22`
picks `SEARCH_NON_FRIENDLY` for what a projectile may hit, which is
"anything not the shooter's own", not "anything not allied".

The consequence is the same for **both** friendly searches: search 1 should
be `x.owner == who`, and search 2 likewise. Note that **search 2 does not
depend on the enumerator argument at all** — `find_unit_ordered` hardcodes
case 1 — so `merchant.rs:205`–`211` is wrong whatever `SEARCH_FRIENDLY`
turns out to be, and that half of the row cannot be argued away. Search 1
rests on the enumerator identification above. The third search,
`SEARCH_ENEMY`, is case 3 = `is_enemy`, and `merchant.rs:217` has it right.

### R2 — the good's `ever_seen` byte is substituted by a fog read, and the fog read is ally-wide

| | |
|---|---|
| document | MERCHANT.md §2.2, "`if not (g.ever_seen & (1 << who)): continue`" |
| code | `crates/sim/src/merchant.rs:94`, `crates/sim/src/merchant.rs:228`–`231` (`good_ever_seen`), `crates/sim/src/scout.rs:285`–`294` (`was_really_seen_fog`) |
| document says | the good object's own accumulated byte, tested against **one** bit — the asker's |
| code does | reads `World::seen2` at the fog half-cell containing the good, and `was_really_seen_fog` masks with `ally_mask \| (1 << who)` (`scout.rs:292`) — so a rare **an ally saw and this leader never did** passes here and is refused there. It also returns `true` unconditionally when `who >= 8` or `reveal_map == 3` or the fog cell is off-grid (`scout.rs:286`–`291`), where the original would read a byte that is still zero. |
| difference shows | `UnitsSync` `orders_x`/`orders_y` on the first think of a merchant belonging to a leader with an ally, or on any map started with `reveal_map == 3`. No capture on disk separates them: East Indies and Great Lakes are both 1-v-1 with fog, and every good the AI scores was seen by the AI itself. |
| reached | reached (`Unit::think_merchant@005f4740`, run54 frame 6354) |

Evidence. The decompile is unambiguous about the shape:
`(*(byte *)(iVar5 + 0x20) & (byte)(1 << (this->field_0x9 & 0x1f)))` —
`SubObjectData +0x20`, one bit, the asker's. `crate::goody` faced the same
substitution for a goody box and documented it (`goody.rs:178`–`191`), but
that call site's original *does* use an ally mask (`ever_seen & ally_mask`);
this one does not, so the borrowing widens the predicate. MERCHANT.md §6
does not list this among the reading-only claims and §7 does not list it
among what is not established — it is invisible in the paperwork.

UNSURE on one half: whether `check_ever_seen`'s accumulation is close enough
to `seen2` that only the ally mask differs. What would settle it is a
`GoodsSync`-style dump of a good's `+0x20` byte across a game with an ally,
which no ini category on disk currently writes.

### R3 — `Unit::think`'s human rare-collector arm (§4 row 2) is not implemented

| | |
|---|---|
| document | MERCHANT.md §4, the middle table row: "`is_rare_collector` **and `leader_flags & 4`** and packed, on `idle == 1` or every 32: `do_gather(search)` and then `unpack_merchant(this, 4)`" |
| code | `crates/sim/src/orders.rs:1376`–`1470` (`Sim::think`) — no such arm; `Sim::unpack_merchant` has exactly **one** caller, `crates/sim/src/merchant.rs:71`, with ring 3 |
| document says | a human player's merchant or fishing boat, standing idle and packed, runs `do_gather` and then a ring-4 `unpack_merchant`, with a message and `S_INVALID_ORDER` when the spot search refuses |
| code does | nothing — the arm is absent, and `unpack_merchant` is never asked with ring 4. `merchant.rs:241`–`243` alludes to it ("asks with ring 4 and no capture reaches it") without saying it is unbuilt, and MERCHANT.md §7 lists the *Dutch* arm as unmodelled but not this one. |
| difference shows | no capture — every capture is AI-side, and `leader_flags & 4` gates the arm to a human. It becomes observable the first time a traced human walks a packed merchant or fishing boat onto a deposit and leaves it idle. |
| reached | reached (`Unit::think@005f6e40`, `Unit::unpack_merchant@006038e0` — entered on run54 frame 6354, via the *other* arm) |

Evidence, and a document gap alongside the code gap: the original's arm at
`funcs/Unit/think@005f6e40.c:166`–`190` carries a gate MERCHANT.md §4 does
not mention — `if (((unit_masks & 0x100) == 0) || is_merchant(this))` at
`:177`, between the cadence and the `do_gather`. So a **fishing boat** that
was ordered recently (`0x100` set) skips the arm where a merchant does not.
And `unpack_merchant(this, 4)` runs only when `do_gather` answered non-zero
(`:181`–`:182`), which §4's "and then" leaves ambiguous.

### R4 — `find_merchant_spot`'s third test, `detect_unit_collision`, is not asked

| | |
|---|---|
| document | MERCHANT.md §3, "`if detect_unit_collision(this, t * 0xc0, 1, 1, 0, 0, 0): continue`" |
| code | `crates/sim/src/merchant.rs:288`–`293` — only `good_merchant_spot` and `invalid_loc` |
| document says | three tests per ring entry, in order; a tile another unit occupies is skipped |
| code does | two. A merchant may deploy on an occupied tile. |
| difference shows | `UnitsSync` `orders_x`/`orders_y` and the order list on the frame a merchant arrives at a rare with a unit already standing on the ring's first acceptable tile. run68's block 6714 is the only capture that reaches the ring at all, and there the winner is `MOVE_49` entry 3 with nothing standing on it — `detect_unit_collision` refuses nothing in any capture on disk. |
| reached | reached (`Unit::find_merchant_spot@00603ab0`, run68 frame 6714) |

Known and stated: `merchant.rs:272`–`276` carries it as a SEAM and
MERCHANT.md §7 names it. Recorded here only so the row set is complete;
the reason is real (this crate's `detect_unit_collision` is `&mut` and
writes the `collide_o`/`collide_who` bookkeeping).

### R5 — `unpack_merchant`'s `unit_masks &= ~0x100` is not cleared

| | |
|---|---|
| document | MERCHANT.md §3, "`unit_masks &= ~0x100`" — the line between the spot search and the cast |
| code | `crates/sim/src/merchant.rs:244`–`261` — the clear is absent; the function goes straight from `find_merchant_spot` to `add_cast_order_at` |
| document says | the "ordered recently" bit is cleared before the cast order is built |
| code does | nothing with it |
| difference shows | R3's arm is where it bites: `think@005f6e40:177` reads exactly this bit to let a packed **fishing boat** that was ordered recently skip the human deploy arm. With neither the bit nor that arm modelled, there is no observable in this crate — and no capture reaches the human arm anyway. |
| reached | reached (`Unit::unpack_merchant@006038e0`, run54 frame 6354) |

Known and stated: MERCHANT.md §7 names it. `Unit::unpack_merchant@006038e0`
line 21 of the decompile is `*(uint *)&this->field_0x68 & 0xfffffeff`,
immediately after the spot search succeeds.

### R6 — `think_merchant`'s tail does not clear `unit_masks & 0x4000000`

| | |
|---|---|
| document | MERCHANT.md §2.3, "The tail is `Unit::add_move_order@00616ed0` inlined: `unit_masks &= ~0x4000000`, `path.length = 0`, `close_orders`, `clear_partial_path`, `update_action`, then the order" |
| code | `crates/sim/src/merchant.rs:155` → `crates/sim/src/orders.rs:827`–`838` → `crates/sim/src/orders.rs:699`–`715` (`enqueue`, `QueuePos::New`): `path.clear()`, `close_orders`, `clear_partial_path`, `update_action`, push, `update_action`. Four of the five; the mask clear has no counterpart anywhere in the crate. |
| document says | five things happen before the order is built |
| code does | four. `Unit.cant_reach` is `SubObjectData::flags & 0x10` (`crates/sim/src/lib.rs:234`), a different bit; grep finds no model of `unit_masks & 0x4000000`. |
| difference shows | nothing in this crate, because neither of the bit's two readers is modelled either: `Unit::think@005f6e40:233` (a text bubble and a `0xfbffffff` clear of its own, console-side) and `Unit::work@0060d180:124`, which re-issues the tail move order at `QUEUE_FIRST` when the bit is set. The second one **is** sim-visible, so this row is really "the bit and its setter are unmodelled", not "the merchant forgot a clear". |
| reached | reached (`Unit::think_merchant@005f4740`, `Unit::work@0060d180`) |

UNSURE: whether the crate models the bit's *setter* under another name, in
which case this becomes a live divergence rather than a consistent absence.
What would settle it is finding `unit_masks |= 0x4000000`'s writers in the
export and checking each against ORDERS.md — out of scope for a
MERCHANT.md pass, and worth an item of its own.

### R7 — the Dutch merchant's auto-attack arm (§4 row 1) is not implemented

| | |
|---|---|
| document | MERCHANT.md §4, the first table row: "the auto-attack block, `think:146` — `is(MERCHANTDUTCH 0x3e, 1)` and packed — the Dutch merchant only" |
| code | `crates/sim/src/orders.rs:1405`–`1415` (the auto-attack arm of `Sim::think`) — no merchant branch |
| document says | a packed `MERCHANTDUTCH` reaches `think_merchant` from inside the auto-attack block, **ahead of everything else**, and so does not wait for the mod-32 tail gate or the mod-128 merchant cadence |
| code does | a Dutch merchant takes only the step-5 arm at `orders.rs:1462`, so its first think is up to 127 frames late relative to the original's |
| difference shows | `UnitsSync` `orders_x`/`orders_y` and the frame of the first `MOVE_TO`, for a Dutch player's merchant. No capture has one — neither East Indies nor Great Lakes is the Dutch. |
| reached | reached (`Unit::think@005f6e40`) |

Known and stated: MERCHANT.md §7's last-but-two bullet names it. Recorded
for completeness.

## What was checked and found faithful

The 61 rules not rowed above. In outline, so a later pass need not re-derive
them:

- **§1** — `is_merchant` is the three exact `TypeIndex`es `0x3d`, `0x3e`,
  `0x190` and never a lineage (`calc_gather.rs:39`, `:93`); `is_rare_collector`
  is those three or the `FISHERMEN` lineage (`calc_gather.rs:103`); a type
  with `unit_flags2 & 4` is born packed (`lib.rs:1289`–`1298`,
  `combat.rs:174`).
- **§2.1** — the head's `return 1` for a deployed merchant, and
  `unpack_merchant(3)` ending the think (`merchant.rs:67`–`73`). Both arms
  return `true`, matching `if ((mask & 0x80000) == 0 || unpack_merchant(this,3) != 0) return 1;`.
- **§2.2** — `local_10` from `tregion_alt` of the merchant's own **tile**
  against the good's cell's plain `region_of` (`merchant.rs:80`, `:124`);
  the ocean bit as `mask & 0x30 == 0x20` on both tiles, compared for
  equality (`merchant.rs:81`, `:110`; `world.rs:548`–`550`);
  `base = 200 − 10·i` falling at the loop tail whatever the slot did
  (`merchant.rs:89`, and the `enumerate` index makes the "whether or not
  accepted" part structural); `alive` = `SubObjectData.flags & 1`
  (`world.rs:527`); the good's cell as `pos / 0x300` (`UNITS_PER_CELL` is
  768, `world.rs:56`–`60`); the ally test passing on a negative owner —
  `Owner::None` and `Owner::Ambiguous` both give `player() == None`
  (`world.rs:181`–`188`, `merchant.rs:101`–`104`), which is the original's
  `iVar6 < 0`; the cell centre `c·0x300 + 0x180` (`merchant.rs:116`–`119`);
  the three searches in order, each asked only when the one before found
  nothing (`merchant.rs:202`, `:205`, `:212`); `+100` for the own region
  (`merchant.rs:124`–`126`); `− danger[who][c >> 1]` (`merchant.rs:127`,
  `world.rs:1134`–`1139`); `max(score, 1)` (`merchant.rs:128`); the strict
  `best < score` with `best` starting at −1, matching `local_24 = -1`
  (`merchant.rs:83`, `:131`); the exact `TypeData.type` filter
  (`merchant.rs:200`).
- **§2.2.1** — `find_unit` measures the object's own position and
  `find_unit_ordered` the candidate's `orders_x`/`orders_y`
  (`merchant.rs:179`–`188`); the move-family gate `{1, 2, 3, 4, 0x12, 0x13,
  0x15}` (`scout.rs:74`, used at `merchant.rs:207`) — the decompile's
  `iVar5 == 1 || … || iVar5 == 0x15` chain at
  `find_unit_ordered@0065bc40.c:61`–`63` is exactly that set; the
  `param_6 & 0x200` region test never running, because the call site passes
  0 and the code has no equivalent.
- **§2.2.2** — `Sim::seed_new_rares_from_fog` (`rares.rs:252`–`272`) walks
  the installed grid row-major and offers each set bit once, and
  `rondata::diff::build_sim` calls it (`crates/rondata/src/diff.rs:944`)
  after the goods and the `human` bits are in. The row-major-versus-sweep
  order is correctly carried as an open question, not a claim.
- **§2.3** — `SimpleArray<int>::remove` by **value** then append
  (`merchant.rs:142`–`146`) matches `remove(&leaders[who].field_0x6e6c, local_34)`
  followed by the manual append at `field_0x6e70`; the list is never
  shortened; `MOVE_TO`, `QUEUE_NEW`, `action = 0` (`merchant.rs:155`);
  the destination 48-snapped as `(p/48)·48 + 24` (`orders.rs:423`–`428`,
  matching `div_3_table[x >> 4] * 0x30 + 0x18`) with the bearing computed
  to the **unsnapped** point before the snap (`orders.rs:836`–`837`).
- **§3** — `find_merchant_spot` or `return 0`; `add_cast_order` with the
  generic `0x28c` at `QUEUE_NEW` and the caster rewrite to the merchant
  family's `0x290`, `FISHERMEN`'s `0x292` (`orders.rs:1105`–`1114`,
  `merchant.rs:257`); §3.1's `[MOVE_TO, CAST]` execution order, built here
  as cast-at-New then move-at-First (`merchant.rs:257`–`259`) — equivalent
  to the original's `add` + `head = head->next`
  (`unpack_merchant@006038e0.c:52`–`56`); the move's destination
  `snap(t·0xc0)` and its angle to the unsnapped `t·0xc0`
  (`merchant.rs:258`–`259` through `orders.rs:827`–`838`);
  `clear_partial_path` and `update_action` (`orders.rs:709`–`714`).
- **§3's spot search** — the `calc_gather` gate at the unit's own position
  and nowhere else (`merchant.rs:278`, `calc_gather.rs:131`–`132`);
  `radius[] = 1, 9, 25, 49, 81` with ring 3 = 49 = the `MOVE_49` prefix of
  `MOVE_289` (`merchant.rs:53`, `:282`–`283`, `world.rs:473`); the
  off-the-map `continue` (`merchant.rs:285`); `good_merchant_spot` then
  `invalid_loc(0,0,0,0,0)` in that order, first hit wins
  (`merchant.rs:288`–`293`).
- **§3's `good_merchant_spot`** — the two-by-two `(x,y)`, `(x−1,y)`,
  `(x,y−1)`, `(x−1,y−1)`, each in bounds, not `BLOCKED` `0x4000`, not
  `mask & 3 == 3`, not `PLACED` `0x80`, then `calc_gather` at `t·0xc0`
  (`merchant.rs:304`–`319`; constants at `world.rs:542`–`574`).
- **§4** — the step-5 arm's gate is `is_merchant` alone and its cadence is
  `idle == 1 || ((o + frame) & 127) == 0` (`orders.rs:1462`–`1463`),
  matching `think@005f6e40:283`–`292`; it sits below the tail's mod-32 gate
  (`orders.rs:1447`–`1449`) and below `think_fish` (`orders.rs:1454`), as
  there; `leader_flags & 4` is *human* and is read that way throughout
  (`rares.rs:292`–`295`, `scout` per the document).
- **§5** — `Sim::init_guys` ends with `Sim::seat_guys` (`anim.rs:686`),
  which is where `Unit::init@00612100:548`–`549` puts it; and a tracked
  crew figure is skipped by the unit-level turn override while a trackless
  one is not (`anim.rs:786`–`793`), which is the effect §5 names.


## Adjudication — 2026-09-05, Opus

Every row re-checked against the source and, where it turned on the
original, against the export. **Seven confirmed, none struck.** All seven
are stated, unimplemented and unreached: the corpus is 1-v-1 with fog on
both maps, every capture is AI-side, and no leader in any of them has an
ally holding merchants.

### R1 splits, and only half of it is settled

**The half that is settled needs no enum at all.** The reader's second line
of evidence checks out from the export directly:
`ObjectsData::find_unit_ordered@0065bc40` passes the **literal `1`** to
`Search::valid_search` (`:40`), where `ObjectsData::find_unit@0065ca80`
passes its own `param_3` (`:58`). Case 1 of `valid_search@0067daa0` is
`if ((Search *)param_2 != this) return 0` — the iterated leader must *be*
the asker. So the merchant's **second** friendly search is own-leader-only
whatever `SEARCH_FRIENDLY` resolves to, and `merchant.rs:205`–`211`'s
`is_ally` is wrong there outright.

**FABLE: the first search's half is not settled.** It rests on identifying
`SEARCH_FRIENDLY` as case 1, and the reader's argument for that is a
complementary-pairs read of the call sites (1↔6, 4↔5, 3↔7) — good
reasoning, and exactly the kind `CLAUDE.md` says is not enough: *a name is
settled by the type record, never by the surrounding code.* The export's
`enums/` holds only `TypeIndex.txt` and `types.txt` has no `SearchIndexBH`
record, so the mapping is not available here. What would settle it: the
enumerator's own record via the PDB's field-list type index (the
`pe-global-table-from-pdb` recipe), or a single call site whose argument is
a literal beside a comment. Until then search 1 is *probably* wrong in the
same way and is not asserted to be.

`merchant.rs:217`'s third search — `SEARCH_ENEMY`, case 3, `is_enemy` — is
right on any reading.

### The rest

R2 (the good's `ever_seen` byte substituted by an ally-masked fog read), R3
(`Unit::think`'s human rare-collector arm absent; `unpack_merchant` has
exactly one caller, `merchant.rs:71`, with ring 3), R4 (two ring tests where
the document has three), R5 (the `unit_masks &= ~0x100` clear absent), R6
(`think_merchant`'s tail does not clear `unit_masks & 0x4000000`; the
`0x4000000` that *is* modelled in `crate::lib` and `crate::rares` is a
`LeaderData` flag, a different bit, so the reader's UNSURE about the setter
is well placed) and R7 (the Dutch merchant's auto-attack arm — the string
"dutch" does not appear in `merchant.rs`) are all confirmed as written.

R4, R5 and R7 are already carried as seams in `docs/MERCHANT.md` §7 or at
the code site. R2, R3 and R6 are not, and are the ones §8 now records.

### A correction to the brief this reader was given

The brief told this reader that `Unit::add_cast_order@005e4a60` was on the
blind list. It is not — that came from a single-trace run made before the
68-trace union was built, and the lane carried the stale fact into the
brief. The reader checked rather than believing it and said so. **No
MERCHANT.md row is blind**, and the file marks every row reached
accordingly. The union list itself is correct; only the brief was wrong.
