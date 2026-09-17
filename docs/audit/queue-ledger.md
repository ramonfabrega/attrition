# The queue ledger — where the deleted numbers went

`docs/QUEUE.md` deletes a finished item rather than striking it, and the
deletion is a **claim**: the work landed, and its story is in
`docs/JOURNAL.md` under that number. `tools/queueledger.py` checks the
claim on every run of `tools/guard.sh`. This file is the escape hatch for
the numbers the check cannot see — an item retired in a phrasing the
journal never used, one dropped on purpose, or one that turns out to have
been lost.

**A line here is not a pardon.** It says what happened to the number, with
enough evidence that the next reader does not have to re-derive it. The
default disposition for a number nobody can account for is not `dropped`;
it is re-booking the item in the queue.

## Why this exists

On 2026-09-06 the post-crash rewrite that landed item 227 — commit
`8b37e5f`, whose own message reads *"Recovered from the worktree the crash
left behind"* — shed 196 lines of queue, and five consecutive items went
with it. That run of five is the only one of its kind in the file's
history; every other retirement in 218 numbers was one or two at a time.
Two of the five had in fact landed. **Three had not**, and nobody noticed
for a day, because every check anyone ran was a *branch* check and the loss
was one level below it: `loop-234`'s branch tip was a docs-only handoff
commit that merged clean while the item's actual product — 406 lines of
`crates/sim/src/anim.rs` — sat uncommitted in the worktree, scheduled for
reaping. `git worktree remove` refusing a dirty tree is the only reason it
survived (2026-09-07; the recovered work is on branch `rescue-234`, commit
`0e14e48`).

## The crash cluster, audited 2026-09-07

- **230** landed — `unit_masks & 8` is compared in
  `crates/rondata/src/diff/unit.rs`, and its note says the disagreement it
  was booked for ("335 of these 450 unit-frames") is now zero:
  *"At zero it is an ordinary row of the labelled comparison, and that is
  what it is now."* The cause was `clear_partial_path`, not the refused
  step the row named (`docs/MOVEMENT.md`, "The verified line's lifecycle").
- **231** landed — `Sim::same_group_soft` in `crates/sim/src/collide.rs`
  carries the three-rule split whole, including the correction that
  `(iVar7 == 0 || iVar7 == 0xc)` short-circuits the action test for the
  other seven kinds, and the two seams (`+0x104`, `CHANGE_FORM`) are named
  in place.
- **232** re-booked — `Levels::for_player` in
  `crates/sim/src/holdings.rs` still returns `Levels::BASE` and ignores its
  player. Re-booked in the queue under its own number.
- **233** re-booked — `Sim::process_building` in
  `crates/sim/src/city.rs` still keys the 32-frame phase on
  `frame + b as i64`, the `Vec` index, where `Build::process@0061edf0:728`
  keys it on the object number. Wrong the first time a slot is reused.
  Re-booked.
- **234** re-booked — recovered from the reaped worktree onto branch
  `rescue-234`. Re-booked.

## Retired before this ledger existed — unverified

These nine left the queue between 2026-08 and 2026-09-06 without a journal
entry naming them, and their disposition has **not** been audited. They are
listed so the guard has a truthful baseline, not because they are believed
finished. Auditing them is part of the re-booked item that introduced this
file; each one is a ten-minute read of the code the row names.

- **96** unverified — retired 2026-08-27 era.
- **108** unverified.
- **118** unverified — booked with 119 in the same commit.
- **119** unverified — booked with 118 in the same commit.
- **162** unverified — the journal's 2026-09-02 entry touches the same
  ground ("the fifty was a goody box") without naming the number.
- **192** unverified — the merchant's unpack window may have closed it
  (journal, 2026-09-06) without naming it.
- **199** unverified — `1/19` turns wrong on the same frame as 198: right
  speed, wrong heading, +14,−20 against +25,0.
- **200** unverified — `1/2015`'s `y_internal` four cells south from 4577,
  15936 against 15744, four hundred frames downstream of the parting.
- **214** unverified.

## Item 7, and the parser that hid it, 2026-09-07 / 2026-09-17

- **7** landed — Orders, `docs/ORDERS.md` and `crates/sim/src/orders.rs`,
  struck through in the queue as done on 2026-08-21 (`57546c07`) and never
  named `item 7` in the journal because the entry predates the phrasing.
- The 09-07 row here said 7 "was never booked", and that was wrong:
  `queueledger.py` then read item references with
  `re.findall(r"\(([\d/,\s]+)\)", …)` and pulled every `\d{1,3}` out of the
  group, so item 253's **coordinate** `(11,7)` registered 7 and 11 as live
  numbers and the real item 7 was hidden behind the false one. Item 279
  booked the fix; the third Fable pass (2026-09-17) landed it — `(N)` and
  `(N/M)` only, and `item N` case-insensitive — and the guard's first run
  with the fix named the real 7 above. Both false alarms were made to
  fail on purpose first (`docs/audit/2026-09-17-fable-pass-3.md`).

## Referenced and never booked — the phantom check, 2026-09-17

- **304** and **224** were each written into an item as a dependency
  ("wire when 304 closes"; "takes 224", since 2026-09-04) and never booked
  in any form. The ledger now fails on a number the queue *refers to* that
  no revision ever booked; both were booked the day the check landed.
