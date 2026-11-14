# A compaction proposal for `docs/` (2026-11-14)

A lane on `worktree-att-docs`, Fable 5.2 (`claude-fable-5-2`), asked for by
Ramon on the commander's row: "an openbounded fable suggest doc trims/doc
syncs? e.g. kind of a compactor of our docs?". **This file proposes; it
edits nothing.** No other document, guard or tool was touched. The next
`steer` pass decides each row.

Ramon widened it twice while the lane ran: "you may consider even condensing
decisions/append only logs ... we are considering any sort of compaction",
and "if there is a decision/something you think could be overturned, you can
ask me to decide ... any contentious or the like decision/etc can be
reviewed". So a proposal may change a rule, and says which; and the
decisions this lane would put in front of him are listed first.

Every number here was measured on `0430a0321` with the scripts in the
appendix, which port `docs_guard`'s own `sections()` (live bytes: struck
spans are not counted). "B" is bytes. Token figures are estimates at an
assumed 3.3 bytes a token for this prose, not a measurement.

**State of this file: slice 1 of several.** Part 1 (the read-first set) is
whole. Parts 2 and 3 are stubs with what is measured so far; later commits
fill them.

## The finding in one paragraph

The bytes that cost money are the ones every session reads, and there are
about 70,000 of them for a worker (`CLAUDE.md` 25,606, the memory index
7,324, `docs/QUEUE.md` 24,235, `docs/TRACKS.md` 11,817) and about 120,000
for a driver (those, `docs/LOOP.md` 37,586, and the entry and record the
opener names, about 20,000). The 14.5 MB under them is read a section at a
time and its size costs a session almost nothing. So the proposals with a
return are in the read-first set, where roughly a third of the bytes are
rows that say "whole", rules restated from another read-first file, or
history. The giants have no cheap cut: their sections are cited by code
(2,374 citations of `AI.md` alone, 402 of its 438 sections) and are not
repeated in the journals (median 6 % shared text, measured). What the
giants do have is a ceiling that works as a target: 30 sections sit within
100 bytes under 16,000, and the queue's handoff has been within 100 bytes
of it on 91 of its last 120 commits.

## For Ramon: decisions worth a review

None of these is treated as overturned. Each is a standing rule whose cost
is now measurable; the question is his.

| # | the rule | what it costs today (measured) | the case for keeping it | the question |
|---|---|---|---|---|
| D1 | A `## ` section is held under 16,000 B (`SECTION_CEILING`, `docs_guard.rs`; 2026-08-30) | It is a target, not a bound. The handoff's median over its last 120 commits is 15,958 B; 91 within 100 B of the ceiling, 46 within 20. The DECISIONS index has stood within 100 B on 36 of its last 60 commits and its newest rows are squeezed to four words ("113 extended by 114 — rows booked"). 30 sections of the specifications sit at 15,900 to 16,000. Item 262 removed the queue's line count for exactly this ("43 fitting episodes and 117 USD", the guard's own comment); the byte ceiling rebuilt it one level down. | A section is what a session reads whole; without a bound the handoff was 70 lines within two days (the guard's header). | Keep a ceiling but stop it binding the two sections that are rewritten every session (the handoff, the index)? R1 and S1 below remove the pressure without moving the number; the alternative is a lower ceiling for the handoff with a count of *facts* rather than bytes. |
| D2 | The handoff is at most 32 lines (`HANDOFF_LINES`) | Met by lines of 2,779, 3,285 and 3,686 B. The line bound has not bound anything since the lines stopped wrapping; the byte ceiling is the real one (D1). | "Never wrapped" is required by the parsers that read the scoreboard lines. | Drop the line bound for the prose (it is fiction), or make the prose wrap and count? |
| D3 | `docs/DECISIONS.md`: append, amend in place, never delete | 843,219 B; 54 entries that score a tranche or a pass hold 478,745 B (58 %); 70 entries (427,951 B) are cited by no code and no tool. A rule's standing form is reached by walking a chain (entry 128 is "amended by 129, 130, 131, 132"). Nobody reads the file whole, so the bytes cost little; the chain-walk and the index's squeeze are the cost. | It is the one place a considered decision can be told from an accident, and a retelling drops findings (the guard's comment on `8b37e5f`). | Three tiers under G-D below: split the index (no rule change); freeze the tranche-scoring entries at a revision with an index, as `docs/PARKED.md` was (entry 125's precedent); or a rewritten digest. This lane recommends the first now and the second on his word. |
| D4 | A mechanic document's section numbers are an API, and a section is appended per item | `AI.md` is 438 sections in landing order, 2.8 MB; 349 are titled by an item. It is a notebook with stable addresses, not a specification a session can read to learn the AI. | The addresses work: 2,374 code citations resolve, and a guard checks every one. | Is a *digest by mechanism* wanted on top (new file, cites the numbered sections, no renumbering)? It is an editorial job of days, not a compaction; this lane does not recommend it before the milestone. |
| D5 | "Neither headline slot is ever empty" (entry 41; `docs/QUEUE.md`, "How to maintain this file"; `docs/METHOD.md`, "Booking an item") | It contradicts the queue's own "a residue or chapter item books only when another track's frame needs one" and the tracks' "no item stands". A fresh session reads both. | Entry 41's reason (the second map's capture fell out of the queue) still holds for the `Golden:` line, which a guard keeps. | Confirm the milestone (entry 116) retired the two-slot rule, so the clause can be struck in both files (S3). |
| D6 | A track's stub pool holds two unassigned sections (`docs_guard`, parked 1718, 1741) | The queue carries "Pools: AI §417–§419; GOLDEN §195–§198; MAPGEN §37" for tracks the same handoff calls whole. | Seven of twenty-four merges backed out when stubs were appended beside a live lane. | Does the pool still apply to a track with no lane? (Not yet measured which documents the guard holds; Part 3.) |
| D7 | `CLAUDE.md` says "Read `docs/QUEUE.md` first ... then `docs/TRACKS.md`"; the brief's frame says "Read `docs/QUEUE.md`'s opener, then what the item cites" | Two instructions to the same lane. If a briefed lane follows `CLAUDE.md` it reads 36,052 B (about 11 k tokens) its brief already carries in the item's words. Not measured: which one lanes follow (a `lore` query over lane transcripts would say). | A session with no brief needs the full read. | Scope `CLAUDE.md`'s sentence to a session with no brief? His word, his file. |

## Summary

| # | file, section | bytes now | action | saved | needs |
|---|---|---|---|---|---|
| R1 | QUEUE, "Where things stand": the ten lines whose every row is whole | 5,738 | closed form, as `Golden:` has | ~5,300 | `rondata::diff::floors` parser; the queue's maintenance bullet |
| R2 | QUEUE, handoff prose: rules restated from LOOP and numbers restated from the pinned lines | ~2,600 of 6,967 | strike; the prose keeps what no pinned line carries | ~2,600 | nothing (driver's to write) |
| R3 | QUEUE, "How to maintain this file" | 3,454 | fix one pointer, strike two superseded bullets, move two to METHOD | ~1,300 | lower `QUEUE_SECTIONS` pin (38) |
| R4 | TRACKS, "The tracks": the state column | ~6,300 of 7,777 | one state a cell, or name the queue's line | ~4,500 | nothing; the pass maintains it |
| R5 | LOOP, "The chain": the waiter block | 4,931 | point at `tools/lanewait.py`'s header; keep three rules | ~4,000 | nothing; closes Loop 4051, 4055 |
| R6 | LOOP, "The brief": kinds no item is booked under | 1,823 | move to the retired-clauses record | ~1,600 | nothing |
| R7 | LOOP, "The driver": the harness changelog, roster history, a doubled clause | ~2,400 | fold two rules in, move the history | ~1,700 | nothing |
| R8 | LOOP, "Ratification, and the passes": first acts | 2,062 | list the acts; figures to the pass records | ~1,000 | nothing |
| R9 | METHOD, "Conventions as they stood", "Tooling as it stood" | 8,563 | delete the ten bullets byte-identical to `CLAUDE.md` | 2,139 | nothing |
| R10 | METHOD, "Phase 3's scoring and succession" | 1,911 | move to the retired-clauses record | ~1,800 | nothing |
| R11 | `CLAUDE.md` | 25,606 of 26,000 | candidates only; Ramon's word | up to ~4,500 | Ramon |
| S1 | DECISIONS, "Index" | 15,985 of 16,000 | split at entry 100; retitle; restore order | 0 (room: ~8,000) | nothing: the guard reads any `## Index…` |
| S2–S9 | syncs, Part 2 | | | | |
| G… | the giants, Part 3 | | | | |

**Read-first total: about 26,000 B of 101,371** (QUEUE, TRACKS, LOOP,
METHOD, `CLAUDE.md`), of which about 13,700 B come out of the three files
every worker reads (QUEUE and TRACKS: 13,700 of 36,052, 38 %).

## The three to take first

1. **R1, the whole lines of the handoff.** One parser change and the
   handoff loses a third of its bytes for good, and with them the fitting
   loop of D1: the section would stand near 10,700 B, not 15,983. It is
   self-correcting: the closed form is accepted only while every row is at
   its length, so a floor that falls brings its row back.
2. **R4 with S2, the tracks' state column.** It is the read-first set's
   largest stale surface (seven facts behind the queue, listed under S2) and
   the cheapest to fix: no guard, one owner, about 4,500 B.
3. **R5, the waiter block.** 4,931 B the file itself has called unverified
   since 2026-11-09 (Loop 4051) and owed a shortening (Loop 4055), neither
   ruled in five days and three passes; the tool's own header already holds
   every exit, its answer and its case.

## Part 1. The read-first set

### What a fresh session reads

| file | bytes | who reads it | a guard on its size |
|---|---|---|---|
| `CLAUDE.md` | 25,606 | every session and subagent, whole | ceiling 26,000 (394 B of room) |
| memory index | 7,324 | every session and subagent | none (outside git) |
| `docs/QUEUE.md` | 24,235 | every session `CLAUDE.md` reaches | handoff 32 lines and the 16,000 B section ceiling; 18 items of 8 lines; two line pins |
| `docs/TRACKS.md` | 11,817 | the same | section ceiling only |
| `docs/LOOP.md` | 37,586 | the driver | section ceiling only ("The driver" 13,382, "The chain" 10,990) |
| `docs/METHOD.md` | 22,127 | residue, chapter, coverage, reading, sweep items | section ceiling only |
| `tools/brief/frame.md` | 33,632 | a lane, **by kind**: `brief.py` selects sections, so a render lane does not pay for the sweep's 5,759 B | a test holds it to the checklist |
| DECISIONS entries the five files cite | 364,270 over 51 entries | on a pointer | 16,000 a section |

The queue's opener sends the next driver to entry 132 (10,895 B) and five
sections of the pass's record (10,142 B).

### R1. The handoff's whole lines

- **Where:** `docs/QUEUE.md`, "Where things stand", the lines `Long
  captures:`, `Second pair:`, `Third pair:`, `Coverage pair:`, `Successor
  pair:`, `Peace pair:`, `Late ages:`, `Battery word:`, `Melee:`, `Third
  map:`.
- **Bytes now:** 5,738 (`Late ages:` 3,687 for 82 rows; `Battery word:`
  1,199 for 28). Every one of their 129 rows reads `w<N> of <N>`: the word
  is the length. `Mapgen:` is 471 more, fifteen styles each "8 of 8 phases".
- **Action:** a closed form for a line whose every row is whole, as the
  `Golden:` line already has (`every chapter closed · 130 closed`, parked
  638): `Late ages: every word whole · 82`. The rows live in `AI_WORDS`,
  where they are pinned. The per-row form returns by itself: the parser
  accepts the closed form only when every pinned row has word equal to
  length (and `0 off` where it carries a close), so a named regression
  makes the short line a red and the row is written out again.
- **Saved:** about 5,300 B now (5,700 with `Mapgen:`), a third of the
  section and a fifth of the file, on every session's first read.
- **What cites it:** `crates/rondata/src/diff/floors.rs` reads each line by
  prefix (`the_handoff_carries`, `ai_line_verdict`, `ai_line_closes`, and
  the per-line tests `the_handoff_s_late_ages_are_the_pinned_words`,
  `…battery_word…`, `…second_pair…`, `…third_pair…`, `…coverage_pair…`,
  `…successor_pair…`, `…peace_pair…`, `…melee…`, `…third_map…`,
  `every_closing_line_row_has_its_close`); `testkit.rs` names `NEWEST_PAIR
  = "Late ages"` (its reader was not read by this lane);
  `bantu_word.rs` cites the line in a comment. `tools/` does not parse
  these ten.
- **Guard:** those tests, and `docs_guard::the_handoff_is_short`. The
  change is made to fail first by lowering one row's word under the closed
  form.
- **Rule touched:** the queue's own "The floors and these lines move
  together ... read literally" bullet gains the closed form. No decision
  entry forbids it.
- **Confidence:** high on the saving and the precedent; medium that the
  parser change is small (one function and `NEWEST_PAIR`'s reader).

### R2. The handoff's prose

- **Where:** `docs/QUEUE.md`, "Where things stand", the dated line and the
  two bullets: 6,967 B on three lines.
- **What repeats a rule from another read-first file** (1,109 B measured):
  "The pilot ... gated once at the merge, scoped ... One kill rule ... a
  soft stop at 99 %" (292 B; LOOP, "The driver"); "Workers: none out; four
  stand ... a fifth or sixth is Ramon's" (180 B; LOOP's roster and TRACKS,
  "The workers"); "The passes ...: the gate takes every merged landing and
  is scoped ... a lane clears at 400 k; a handover is `tools/handover.py`"
  (637 B; LOOP, and TRACKS for parity's definition).
- **What repeats a pinned line fifteen lines below it** (about 1,500 B,
  by reading): the `INPUT_*` counts and "24,000 of 24,000" (`Recording:`);
  "Eight of twelve lines", said three times (`Checksum:`); "107 of 109
  packets ... 314 of 1,053 lines" (`Checksum:`); "The territory 733 · 332 ·
  574" (`Rubric:`); "The blind list: open 0 (never 336 · residue 336)"
  (`Blind:`); "The rules track: whole, 130 closed" (`Golden:`); "`Start:
  161 of 161 lobbies`" (`Start:`); "K 59 of M 60 ... frames 35 of 35,
  pieces 1,465 of 1,478" (`Renderer:`); the HUD's and the lobby's counts
  (`Hud:`, `Lobby:`).
- **Action:** the prose says what no pinned line can: what moved and what
  did not, the held-out leaves and their parked ids, the open readings,
  who is out. A rule is cited by its file, not restated.
- **Saved:** about 2,600 B.
- **What cites it:** `tools/queueledger.py` reads the numbers the queue
  refers to (`item N`, `(N)`, `parked 4118a`): a cite that leaves the prose
  needs no ledger line, a booked item that leaves does. No guard parses the
  prose itself.
- **Confidence:** medium. It is editorial and the driver's to write; the
  list above is the evidence, not a draft.

### R3. The queue's maintenance notes

- **Where:** `docs/QUEUE.md`, "How to maintain this file", 3,454 B, 38 lines.
- **Stale or superseded, each with its evidence:**
  - "The chain — merge, book, spawn, gate, push, reap — is `CLAUDE.md`'s."
    It is `docs/LOOP.md`'s since entry 116: `CLAUDE.md` says "The loop's
    machinery is `docs/LOOP.md`". Fix the pointer.
  - "**Neither headline slot is ever empty** ... no rules item means the
    next chapter without a capture (GOLDEN §13, §14 ...)". Contradicted
    forty lines up ("A residue or chapter item books only when another
    track's frame needs one") and by TRACKS's rules row. D5.
  - "A capture is a draw-stream trace first ..." and "Grep for the derived
    quantity ..." are method for a residue item; METHOD holds the same
    rules ("grep the disk before booking a capture") and is where the kinds
    that use them are sent.
- **Action:** keep the four bullets about this file (end of session; a
  worker never books; the floors and these lines; the gate line). Fix the
  pointer, strike the slot rule on Ramon's confirmation, move the two
  method bullets to METHOD.
- **Saved:** about 1,300 B.
- **Guard:** `a_queue_section_may_only_shrink` pins this section at 38
  non-item lines and **fails when it comes in four or more under the pin**,
  so `QUEUE_SECTIONS` is lowered in the same commit. That is a code edit in
  `crates/sim/src/docs_guard.rs`.
- **Confidence:** high on the pointer and the contradiction; medium on the
  move.

### R4. The tracks' state column

- **Where:** `docs/TRACKS.md`, "The tracks", the fifth column: about 6,300 B
  of the table's 7,777. Its header says "state at the seventy-sixth pass
  (2026-11-09; earlier states in the passes' records under `docs/audit/`)",
  and each cell then appends every later pass's state in bold: the
  renderer's cell is 1,613 B and holds five states, the checksum's 1,263 B
  and four.
- **Action:** one state a cell, the newest, rewritten by the pass (the
  header already says where the earlier ones are). Or no state at all: the
  "number" column names the queue line that carries it (`Start:`,
  `Renderer:`, `Checksum:`, `Recording:`, `Lobby:`, `Rubric:`), which a
  guard keeps true on every commit and this column nobody does.
- **Saved:** about 4,500 B, and the seven stale facts of S2.
- **What cites it:** nothing in `crates/` or `tools/` parses TRACKS;
  `docs_guard` names it nowhere. LOOP and QUEUE cite it by name for the
  milestone and the workers, neither for a state.
- **Rule touched:** none. "The steering pass maintains this page".
- **Confidence:** high.

### R5. The waiter block of the chain

- **Where:** `docs/LOOP.md`, "The chain", from "And a spawn is followed by a
  wait on the lanes" to the `UNCLEARED` clause: 4,931 B of the section's
  10,990, one paragraph.
- **Why:** LOOP says of it, twice, that it is unverified on the harness of
  2026-11-09 and owed a shortening ("Its waiter clauses are to be
  re-measured on the new harness (Loop 4051) and then shortened (Loop
  4055)"). `grep -rl 'Loop 4051\|Loop 4055' docs tools crates` finds LOOP
  itself, the seventy-sixth pass's record and the parked archive: neither
  was ruled by the five passes since. Meanwhile `tools/lanewait.py`'s
  header lists every exit (`LANDED`, `ENDED`, `HELD`, `QUIET`, `DEEP`,
  `STALLED`, `UNCLEARED`) and every non-exit line (`WAITING`, `PUSHED`,
  `DRAFT`, `ARMED`, `ARM-REFUSED`, `GATING`) with its answer and the case
  that taught it. The block is that header retold as prose with the cases
  inline.
- **Action:** replace it with a pointer to the header and the three rules
  the tool cannot hold: a done line that arrives while a wait runs stops
  and re-arms the wait without that lane; a message is never sent to a lane
  beside an armed clear; an exit is answered in its turn. The case
  histories are already in the tool's header and the passes' records; what
  is in neither goes to `docs/audit/2026-11-09-loop-retired-clauses.md`,
  which entry 116 made for this.
- **Saved:** about 4,000 B of the driver's first read.
- **Guard:** none reads LOOP.
- **Confidence:** medium-high. What is not established is Loop 4051
  itself: whether `DRAFT`, `HELD` and `UNCLEARED` still fire on this
  harness. The proposal does not need the answer; a pointer is true either
  way.

### R6. The brief kinds no item is booked under

- **Where:** `docs/LOOP.md`, "The brief", from "The fourth kind is
  `reading`" to "the pass's `--record` row stays the pass's": 1,823 B of
  the section's 2,580.
- **Why:** it specifies the `reading` and `coverage` kinds, the late-age
  pair's shape and the rubric's re-pin. The handoff reads `Blind: ... open
  0`, every `Late ages:` row whole, "ages held 8 of 8", and the queue books
  a residue or chapter item "only when another track's frame needs one".
  `tools/brief.py`'s header lists all nine kinds with their gates.
- **Action:** move to the retired-clauses record; LOOP keeps "the kinds and
  their gates are `tools/brief.py`'s header" and the start and input
  sentence.
- **Saved:** about 1,600 B.
- **Confidence:** medium: a kind can be booked again, and then the tool's
  header is what a driver reads anyway.

### R7. The driver section's history

- **Where:** `docs/LOOP.md`, "The driver", 13,382 B.
- **Items:**
  - "The harness moved under this file" (909 B): a changelog of 2026-11-09.
    Two of its five bullets are rules (a message to a running lane wakes it
    mid-gate, so it is sent only when the lane must act first; a clear no
    longer kills a background command). One is said again forty lines up
    ("A builder on another model takes that model's own default effort").
    Fold the two into their clauses; the rest is history.
  - The roster's measurements (953 B: the trial tranche's false reds, "27
    tranches of that roster read 7.25 USD"): the passes' records hold them.
  - The stop's history ("95 % from 2026-11-11, 85 % before") and the clear
    threshold's ("230 before DECISIONS 131 §6 ... 7.77 USD against 4.27"):
    entry 121 and 131 hold them.
  - "Three things reach Ramon at once" stands here (134 B) and again in
    "Ratification, and the passes" (279 B), in different words.
- **Saved:** about 1,700 B.
- **Confidence:** medium-high.

### R8. The pass's first acts

- **Where:** `docs/LOOP.md`, "Ratification, and the passes", from "Its
  first act is the archives' compression" to the blind list's scan: 2,062 B.
- **Why:** it carries three passes' disk readings ("the sixty-fifth read
  114 at its start ... the sixty-seventh 138 ... 151 after"), and a scan of
  the blind list's open rows, which read 0. Since entry 128 §1 the gate
  itself clears the disk (`tools/disk.py`).
- **Action:** the acts as a list with their commands; the figures to the
  records; the blind scan to the retired clauses with R6.
- **Saved:** about 1,000 B.
- **Confidence:** medium. Whether the compression is still a pass's first
  act after `disk.py` is the pass's to say.

### R9. METHOD's copies of `CLAUDE.md`

- **Where:** `docs/METHOD.md`, "Conventions as they stood" (4,284 B) and
  "Tooling as it stood" (4,280 B).
- **Measured:** of their seventeen bullets, **ten are byte-identical** to
  `CLAUDE.md`'s (2,139 B): earn every dependency; a probe shape graduates;
  `tools/guard.sh` is the reflex; the python-heredoc patch; the pinned
  toolchain; where format notes and decisions live; one document a
  mechanic; `llvm-pdbutil`; constants and their representation; the
  `rondata` survey. The other seven are the long forms `CLAUDE.md`
  shortened (host admission, the wait, Ghidra, the original under Wine,
  the emulator, the packet, the lab).
- **Action:** delete the ten. Every reader of METHOD has inherited
  `CLAUDE.md`, and METHOD's own header says `CLAUDE.md` is the rule where
  they differ, so a copy can only drift.
- **Saved:** 2,139 B, nothing lost.
- **Guard:** none reads METHOD; `tools/brief.py` names it for five kinds.
- **Confidence:** high.

### R10. METHOD's phase 3

- **Where:** `docs/METHOD.md`, "Phase 3's scoring and succession", 1,911 B.
  Its own first lines: "As `CLAUDE.md` stated phase 3 while it was open.
  Every word it names is closed and stands as a floor."
- **Action:** move to the retired-clauses record. The succession rules it
  cites live in entries 53 to 56 and 61.
- **Saved:** about 1,800 B.
- **Confidence:** high that it is history; medium that nothing still reads
  it for the held-out battery's rule (entry 61 has it).

### R11. `CLAUDE.md`: candidates, not a draft

Its edits are Ramon's word. It stands 394 B under its ceiling, so the next
clause that wants in must already cut one. What a render, start or input
lane does not use, by measured size:

| lines | block | bytes | note |
|---|---|---|---|
| 270–295 | the blind second reading | 1,886 | the blind list is open 0; a reading item reads METHOD. The last sentence (this file names nothing a reader re-derives) is a rule about this file and stays. |
| 77–121 | the phases, with each one's state | 2,829 | the state of each phase is TRACKS's; the arc is five lines. |
| 197–215 | the permission-shaped failure | 1,223 | the rule is three sentences; the rest is its cases (a parked id, "163 minutes"). |
| 413–424 | prior art | 651 | not operational. |
| 129–150 | which file holds what | 1,724 | stays; but its `docs/JOURNAL.md` row would change under G-J. |

And one sentence that says two things with the frame (D7).

## Part 2. Doc syncs

*Slice 1: the rows found while measuring Part 1. More to come.*

| # | where | the claim | the evidence against it | fix |
|---|---|---|---|---|
| S1 | DECISIONS, "Index — what stands, as of 2026-10-06" | the date; and one row an entry | 132 rows, the newest of 2026-11-13. 15,985 B of 16,000. Rows are out of order (82 after 85; 91, 93 to 100 before 92, 90, 89, 88, 87, which close the list). Rows for 111 to 126 are cut to a few words, where 76's is 197 B. | split into "Index, 1 to 99" and "Index, 100 on" (`every_decision_has_an_index_row` reads every `## Index…` heading); drop the date or make it the newest entry's. |
| S2 | TRACKS, "The tracks", state column | start "158 of 161 ... the three left on two styles with no mask port (item 4211)" | QUEUE: "`Start: 161 of 161 lobbies`", "The start track: closed" | R4 |
| | | milestone step 1: "(`Start: 1 of 161`)" | the same | R4 |
| | | renderer "K 59 of M 59" | QUEUE `Renderer: K 59 of M 60` | R4 |
| | | checksum "150 of 1,058 lines" | QUEUE `Checksum: ... lines 314 of 1,053` | R4 |
| | | orders "next ... the attack played (items 4194, 4195)" | QUEUE: `INPUT_ATTACKED` 10 of 10 (4195), `INPUT_FOLLOWED` 12 of 12 (4202) | R4 |
| | | lobby "next ... a plain HUD (items 4197, 4198)" | QUEUE: both landed, `Hud:` line | R4 |
| | | structure "**done**" | QUEUE books 4224, a `structure` item marked whole | R4 |
| S3 | QUEUE "How to maintain"; METHOD "Booking an item" | "Neither headline slot is ever empty"; "the headline slot is never empty" | QUEUE "The queue": "A residue or chapter item books only when another track's frame needs one"; TRACKS rules row "no item stands" | strike, on D5 |
| S4 | QUEUE "How to maintain" | "The chain ... is `CLAUDE.md`'s" | `CLAUDE.md`: "The loop's machinery is `docs/LOOP.md`" | pointer |
| S5 | LOOP, "The chain", last line | "The turn does not end with a lane empty and the queue not, and it never asks whether to" | the sentence stops. `git show 75ffba6d1:docs/LOOP.md` has its end: "continue — the queue's opener is the answer." Lost in `15aae746e` (2026-11-09). | restore seven words |
| S6 | LOOP, "The lanes", first paragraph | "one ccc lane on the render track and the driver's builders on the chain" | LOOP, "The driver", the roster: "three ccc lanes ... four workers with the chain's builder, confirmed at the pass by load (DECISIONS 129 §1)"; QUEUE "four stand" | strike the count; the roster holds it |
| S7 | LOOP, "The chain" | "the whole gate ... (DECISIONS 132 §5; every third landing whose merge widened, before it)" with entries 72 and 79's history | entry 132 §5: "This replaces ... entry 79's every third landing" | strike the parenthesis (405 B) |
| S8 | METHOD, "The definition of done" | "the lane gate runs those, the fast tests and the guards"; "the commander's booking gate takes no flag" | entry 132 §6: a lane's gate "leaves every timed test to the booking gate but its filters and the paperwork guards, and runs whatever the timings file has never timed"; §5: the booking gate takes `--scope`. `CLAUDE.md` has the new form. | amend in place |
| S9 | QUEUE, the dated line and the opener | "handing back to `commander`"; "cut a builder on **VII-b1** and spawn three lanes, 4208 ... 4212 ... 4214 ... and a fifth on 4216" | all landed: `docs/journal/2026-11-13-item-4208.md`, `-4212`, `-4214`, `-4216` exist; the queue's own item 4159 says VII-b1 to VII-d landed. 21 commits have edited the queue since the pass's gate (`git log 8a0b2a0e7..HEAD -- docs/QUEUE.md`) and none rewrote these two lines. A row that died now would hand a fresh session an opener that reaps a row long gone and cuts builders on landed steps. | the booking commit keeps the opener true, or the opener is "continue the tranche" with the lanes out |

## Part 3. The giants

*Slice 1: measured, proposals to come.*

| file | bytes | `## ` sections | sections no `crates/` code cites | struck bytes | sections over 16,000 | at 15,000 to 16,000 |
|---|---|---|---|---|---|---|
| AI.md | 2,824,624 | 438 | 36 (90,695 B) | 45,261 | 3 (pinned) | 15 |
| RUNS.md | 2,522,798 | 1,391 | not cited by section | 1,090 | 0 | 0 |
| GOLDEN.md | 1,463,800 | 240 | 26 (94,115 B) | 16,296 | 0 | 12 |
| JOURNAL.md | 1,303,935 | 257 | not measured by the guard | 3,320 | 3 | 0 |
| DECISIONS.md | 843,219 | 134 | cited by entry: 70 of 132 by no code or tool | 1,540 | 0 | 2 |
| COMBAT.md | 768,266 | 88 | 8 (38,254 B) | 25,513 | 0 | 9 |
| ORDERS.md | 494,693 | 58 | 10 (33,386 B) | 10,890 | 2 (pinned) | 5 |

**A hypothesis killed.** "The item-titled sections repeat their journals,
so the history can move." Measured by six-word shingles between each
item-titled section and its item's journal file: `AI.md` 349 sections,
median overlap 0.06, none over 0.31; `GOLDEN.md` 181, median 0.06;
`COMBAT.md` 70, median 0.06; `ORDERS.md` 29, median 0.03. And between each
DECISIONS entry from 57 on and its nearest pass record: median 0.09. The
two tell the same landing in different words; neither is a copy. So
nothing in the giants can be deleted as a duplicate, and any cut is a
retelling, which is the edit the guard's header warns drops findings.

## Appendix. The measuring scripts

*To come with the last slice: the section port, the citation counter, the
shingle overlap, the history of a section's size.*
