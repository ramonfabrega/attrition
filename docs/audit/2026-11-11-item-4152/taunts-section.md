## 9. The taunts

A human's taunt-order chat line reaches an allied computer through an
eight-slot ring per receiving leader, fires on a due frame, and leaves a
stamp that one later ping reads. Every leaf is indexed or written in that one
chain.

### 9.1 Leaves

- `last_taunt`, +0x354, `TauntRequest[8]`, indexed by the sender's `who`.
- `taunt_frame`, +0x374, `int[8]`, indexed by the sender's `who`; an absolute
  sim frame.
- `incoming_taunt`, +0x394, `int[8]`, the ring's request word.
- `incoming_taunt_who`, +0x3b4, `int[8]`, the ring's sender word (a leader
  `who`, not a play index).
- `incoming_taunt_frame`, +0x3d4, `int[8]`, the ring's due frame.
- `incoming`, +0x3f4, `int`, the ring's write cursor, 0..7.

`TauntRequest` runs 0..17; `HELP` is 16. Every leaf is zero at game start
and no rule here resets one inside a match.

### 9.2 The rule

**Writers.** Two functions, and the header's bulk writers. `Leader::receive_taunt
(006b8bf0)` writes the ring and the cursor; `Leader::process_taunt (006b8cc0)`
writes `last_taunt` and `taunt_frame`. `Leader::init (006e3930)` on its
negative-tribe branch also clears the three ring arrays and `incoming` in an
explicit loop after its memset. There is no other writer: both the absolute
net over every slot's family range and the register-relative net over
displacements 0x354..0x3f7 were re-run, and every other hit belongs to
another record. In particular `Leader::end_process (006b8690)` and
`Leaders::end_process_all (006ed070)` store at 0x3bc off the 0x8c-stride
`GameInfo` player record, never a leader. `Leader::process (006b88b0)`,
`Leader::process_taunts (006b8ba0)` and `Leader::close (006b7f80)` are dead.

**The producer.** No computer issues a chat command; every producer is a
window of the local human. `CommandPackage::process_chat (009454f0)` handles
a `Chat` command from player `play` (sender `S = player[play].who`, the byte
at +0x77 of the 0x8c record). If `S`'s `net_player` is reported blocked by
crossplay, nothing is delivered. Otherwise, for each valid slot `k` in 0..7,
and only when the command's `taunt >= 0`, it calls `receive_taunt` on `k`
when the delivery gate passes (`sem` is the game semaphore word):

- **Broadcast** (`bits == -1`): (`leaders[S].chat_status[k] == 0` or
  `sem & 0x40`) and (`leaders[k].chat_status[S] != 2` or `sem & 0x40`).
- **Masked**: (`bits` has bit `k`, or `sem & 0x40`). Then, outside
  multiplayer (`sem & 4` clear), a line starting with the cheat string goes
  to the console and nothing is delivered. Else (`leaders[k].chat_status[S]
  != 2` or `sem & 0x40`). The masked arm has no sender-side test.

The taunt word comes from `Taunts::check_for_order (009809d0)`: −1 when no
taunt has the id or its order string is empty, 0 when the string matches no
request name, else 1..16 (two names map to `HELP`). The cheat path pushes 0.
Both 0 and −1 deliver nothing to the ring.

**`receive_taunt(R, req, play)`.** Exemptions, all must pass, else nothing:

1. `req > 0`, signed (0 and negatives refused; no upper bound).
2. `S` active (`leaders[S].flags & 2`).
3. `R` not human (flag bit 4 clear) and active (bit 2).
4. `is_ally(R, S)` (`LeaderData::is_ally (006edb50)`: `S == R`, or
   `diplos == 2` both ways).
5. `R != S`.

Then, at slot `i = incoming`:

    incoming_taunt[i]       = req
    incoming_taunt_who[i]   = S
    incoming_taunt_frame[i] = frame        if req == 16
                              frame + 60   otherwise
    incoming                = (incoming + 1) & 7

The cursor step is C's signed `% 8`, which is `& 7` for a cursor that only
ever holds 0..7. It advances on every accepted request, `HELP` included.
There is no occupancy test: a ninth request inside one delay overwrites an
unfired slot. `frame` is the frame on which the command is applied, which is
before that frame's leader pass.

**The drain.** In `Leaders::process_all (006ed2a0)`, at the tail of each
slot's step with `flags & 2`, and only when `frame != 0`: for `k` in 0..7 in
slot order, if `incoming_taunt_frame[k] == frame`, call
`process_taunt(incoming_taunt[k], incoming_taunt_who[k])`. Nothing is
cleared. Consequences: a `HELP` fires on its receiving frame; a `HELP`
received on frame 0 never fires; the zeroed ring never fires; several slots
due on one frame fire in slot order, which after a wrap is not arrival order.

**`process_taunt(R, req, S)`.** If `leaders[R.who]` is active and not human,
`S != R`, and `leaders[R].diplos[S] == 2` and `leaders[S].diplos[R] == 2`
(re-tested now, not at receipt):

    last_taunt[S]  = req
    taunt_frame[S] = frame

The store precedes the request's range test, so any accepted `req` is
stored; 1..16 then act, above 16 does nothing. The actions themselves
(goods offers for 1..5, the production-AI weights for 7..15, nothing for
`HELP`) write other families' leaves.

**The reader that changes a decision.** `Leader::action_ping (006d18a0)`,
when a human `P` pings for itself: for each other of the eight slots `k`
that is valid, not human and `is_ally(P, k)` (a failure continues the loop),
if `leaders[k].last_taunt[P] == 16` and `frame <= leaders[k].taunt_frame[P]
+ 225`, it sends `Armies::find_useful_army` for `k` to the spot with
`send_here(MOVE_TO)` and sets that army's +0x28 to 2,700. Nothing else reads
`last_taunt` or `taunt_frame`; `incoming` is read only by `receive_taunt`;
the ring words only by the drain. `LeaderData::walk_data (006d6750)` reads
all six whole for the checksum.

### 9.3 In the crate

Nothing of the chain exists yet.

- `Command::Chat` is decoded with `taunt` and `taunt_num`
  (`crates/rondata/src/commands.rs:921`) and dropped at
  `crates/rondata/src/input.rs:1611` inside `Stream::one` (:1503). The port
  replaces that skip with a chat arm that applies the delivery gate and calls
  a new `receive_taunt` on the sim.
- The ring and stamps belong on the leaders record
  (`crates/sim/src/leaders.rs`).
- The drain belongs in `Sim::tick`'s per-player loop
  (`crates/sim/src/core/frame.rs:94`, the `Leaders::process_all` analogue),
  calling a new `process_taunt`.
- `Command::Ping` is not mapped, so there is no `action_ping`;
  `Armies::find_useful_army` exists (`crates/sim/src/ai/army.rs:2207`) with
  no caller.
- Checksum: the six leaves are listed at
  `crates/sim/src/checksum/leaders.rs:87`–92, and `leader_long` (:949)
  answers each through its catch-all `_ => None` (:1305). Each is a
  `(unheld)` entry of `LEADERS_HELD_OUT`
  (`crates/rondata/src/diff/checksum_lines.rs`) with count 2 (two walked
  leaders on run 1562's frame 91; read from the code, not from the
  verdicts). Six `leader_long` arms holding the leaves move those six
  entries, off the list where run 1562's values are the zero a game with no
  taunt line predicts.

### 9.4 Reached by a plain game

Only a game in which a human has a mutually allied computer (team play) and
sends a chat line whose taunt carries an order string reaches any writer. A
one-versus-one against the computer, or any game where no human sends such a
line, leaves all six leaves at zero for its whole length. No trigger, Conquer
the World or multiplayer path is needed; run1077 and run1086 are such
games.

### 9.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/taunts-*.md`); no diff backs it yet.
Counts per claim row: `last_taunt` 11/0/0, `taunt_frame` 7/0/0,
`incoming_taunt` 12/1/0, `incoming_taunt_who` 8/0/1, `incoming_taunt_frame`
11/0/0, `incoming` 8/0/0, the shared pipeline 8/2/2 (confirmed / corrected /
open).

Corrections that changed a rule:

- The delivery gate (9.2, the producer). The masked arm has no sender-side
  `chat_status` test; a masked cheat line outside multiplayer delivers
  nothing; a crossplay-blocked sender delivers nothing.
- `check_for_order` returns 0, not −1, for an order string that names no
  request. This changes no leaf.

Confidence is **high** on every store, gate, constant, cursor step and the
drain: each was read in the listing by both readers and the adjudicator.
It is **medium** on the delivery gate's `sem & 0x40` and crossplay arms,
which rest on the decompile alone; neither bit's meaning is established.

Witnesses, dump readings in `docs/RUNS.md` and **not pinned diffs**:
run1077's `HELP`, after which leader 1 holds `last_taunt[0]` 16 and
`taunt_frame[0]` 5411 on its 5412 dump. That is the zero delay, and it shows
that commands are applied before the drain on the same frame. run1086's food
and timber requests read `last_taunt[0]` 1 with `taunt_frame[0]` 661, and 2
with 681. Receipt one block before 661 matches the 60-frame delay.

### 9.6 Not established

- **The sender's range.** `S` is a byte; the ring and `last_taunt` have
  eight entries and `leaders.list` has ten. Every capture's live `who` is
  0..7, and nothing read bounds the byte. This does not matter to any plain
  game. A bound check at the producer, or a capture with a ninth or tenth
  seat sending a taunt, would settle it.
- **`Setup::build_leader` mid-match.** Seven of `Leader::init`'s ten callers
  pass a tribe word; any of them re-zeroes the family only if that word is
  negative. All are believed to run at setup, when the family is already
  zero. Reading `Setup::build_leader`'s callers would settle whether it can
  run inside a match.
- **`LeaderData::walk_data` in load mode** was not re-read; its range
  [+0x8, +0x692a) is confirmed only through `docs/CHECKSUM.md` §8.
- **The meaning of `sem & 0x40` and of crossplay's block**: a reading of
  their writers, or a capture with chat between allied seats under each.
- **The two witnesses become assertions**: a `rondata::diff` check on
  run1077 and run1086 that the leaves hold 16/5411 and 1/661, 2/681 on the
  cited frames, once the chain is ported.
