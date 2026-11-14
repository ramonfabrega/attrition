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
decisions this lane would put in front of him come first.

Every byte count was measured on `0430a0321` with the scripts in the
appendix, which port `docs_guard`'s own `sections()` (live bytes: struck
spans are not counted). "B" is bytes. Token figures assume 3.3 bytes a
token and are estimates. Session figures are `lore`'s, as of its last
index, at its own dated list prices.

## The finding

**What compaction can buy is small and it is all at the top.** 108 lane
sessions since 2026-11-10 made 11,244 requests carrying a mean of 194,292
cache-read tokens each, 800.94 USD at list (`lore usage --by session`). A
lane's read-first set is `CLAUDE.md` (25,606 B), the memory index (7,324 B)
and the head of the queue (18,475 B; below): about 15,600 tokens, 8 % of
what a request carries. At entry 131 §6's rates (lore, Opus 5.5,
2026-11-13: 8.00 USD a million tokens read again, 0.20 a million carried a
request) the 18,475 B of queue a lane reads cost about 0.16 USD a lane
session, 17 USD over those 108, 2 % of their bill; every trim in this file
together is about 3 %. The 14.5 MB below is read a section at a time and
its size costs a session nothing measurable.

**What compaction should buy instead is a true first read.** The read-first
set is where the stale and the contradicted are (Part 2: a queue opener 21
commits behind its own file, a tracks table seven facts behind the queue, a
sentence cut off mid-clause, two counts of lanes in one file), and it is
where a byte ceiling has become a target: the handoff has stood within 100 B
of 16,000 on 91 of its last 120 commits. A wrong read costs a wrong act, and
a fitting loop costs the edits it forces. Those are the returns.

**The giants have no cheap cut.** Their sections are an API the code calls
(2,374 citations of `AI.md`, 402 of its 438 sections) and they do not
repeat the journals (median 6 % shared text, measured; the hypothesis is
parked with its kill in Part 3). Any cut there is a retelling, which is the
edit the guard's own header says drops findings. This lane recommends
leaving them, with three small structural exceptions.

## For Ramon: decisions worth a review

None is treated as overturned. Each is a standing rule whose cost is now
measurable; the question is his.

| # | the rule | what it costs today (measured) | the case for keeping it | the question |
|---|---|---|---|---|
| D1 | A `## ` section is held under 16,000 B (`SECTION_CEILING`, 2026-08-30) | It works as a target. The handoff's median over its last 120 commits is 15,958 B: 91 within 100 B of the ceiling, 46 within 20. The DECISIONS index has stood within 100 B on 36 of its last 60 commits and its newest rows are squeezed to four words. 30 sections of 17 files sit at 15,900 to 16,000 (`AI.md` 6, `GOLDEN.md` 5, `COMBAT.md` 3). Of the 114 commits that changed the handoff, 47 added 150 B or more, cut about as much and landed at 15,900 or over; in all they added 48,303 B and cut 48,244 from a 16,000 B section. Entry 35 removed the queue's line count for this ("43 fitting episodes ... 117 USD"; "the argument that decided it is correctness, not tokens": a global bound "is satisfied by compressing *any* item") and named its own failure measure: "If a session is seen compressing an item it did not write, it did not [work]." A booking that cuts another booking's clause to seat its number is that. | A section is what a session reads whole; without a bound the handoff was 70 lines in two days. | Keep the number, and take the pressure off the two sections rewritten every session (R1 for the handoff, S1 for the index)? Or give those two their own rule? This lane recommends the first. |
| D2 | The handoff is at most 32 lines (`HANDOFF_LINES`) | Met by lines of 2,779, 3,285 and 3,686 B. The line bound has bound nothing since the lines stopped wrapping. | The parsers need each scoreboard line unwrapped. | Retire the line bound for the prose, or have the prose wrap and count? |
| D3 | `docs/DECISIONS.md`: append, amend in place, never delete | 843,219 B. 54 entries that score a tranche or a pass hold 478,745 B (58 %); 70 entries (427,951 B) are cited by no code and no tool. Nobody reads it whole, so the bytes cost little. What costs is the walk: a rule's standing form is at the end of a chain (entry 128 is "amended by 129, 130, 131, 132"), and the index that should shorten the walk is by number and squeezed (S1). | It is where a considered decision is told from an accident, and a retelling drops findings. | G3: three tiers. This lane recommends the index split now and an index **by topic** (additive, no rule change); it recommends against freezing or rewriting entries unless he wants a file he can read end to end, which is a different product. |
| D4 | A mechanic document's sections are appended per item and their numbers are an API | `AI.md` is 438 sections in landing order, 2.8 MB, 349 titled by an item. It is a notebook with stable addresses, not a text a session can read to learn the AI. | The addresses work: 2,374 code citations resolve and a guard checks each. | Is a digest by mechanism wanted on top (a new file citing the numbered sections)? An editorial job of days. Not before the milestone, in this lane's view. |
| D5 | "Neither headline slot is ever empty" (entry 41; QUEUE "How to maintain this file"; METHOD "Booking an item") | It contradicts the queue's own "A residue or chapter item books only when another track's frame needs one" forty lines above it, and TRACKS's "no item stands". | Entry 41's reason holds for the `Golden:` line, which a guard keeps. | Confirm that the milestone (entry 116) retired the two-slot rule, so both clauses can be struck (S3). |
| D6 | A track's stub pool holds two unassigned sections (`docs_guard`; parked 1718, 1741) | The guard holds `AI.md` and `GOLDEN.md` only: the two tracks the handoff calls whole. Eight stubs stand there (`AI.md` §412, §417 to §419; `GOLDEN.md` §195 to §198). The documents four workers do append to (`RENDERER.md`, `CHECKSUM.md`, `LEADERS.md`, `CLIENT_INPUT.md`) have no pool. | Seven of twenty-four merges backed out before the pool. | Move the pool to the live tracks' documents, or retire it? Not measured: how many merges have conflicted on `RENDERER.md` or `CHECKSUM.md` since 2026-11-09. |
| D7 | The brief's frame: "Read `docs/QUEUE.md`'s opener, then what the item cites" | Lanes read it as the head of the file. Of 98 lane worktrees with a session since 2026-11-10, 93 touched the queue and at least 61 ran `head -N docs/QUEUE.md` (41 at 60 lines, 18,475 B; 14 at 40 lines, 15,259 B; `lore search --lane tool`, a floor). In `CLAUDE.md`'s words the opener is the handoff's last line, 717 B, written to the driver. No lane read `docs/TRACKS.md` whole (0 of 98; 6 read a part), though `CLAUDE.md` says to. | A lane with the headline in view parks better. | What does a briefed lane need from the handoff: all of it, the first bullet, or nothing its brief does not carry? R0. And `CLAUDE.md`'s "then `docs/TRACKS.md`" is not what lanes do; scope it to a session with no brief? His file. |
| D8 | A finding that names no score parks in its item's journal (entry 125) | 187 parked rows in the 73 journals of 2026-11-12 to 11-14; the queue cites 21 parked ids. Nothing lists the rest: `brief.py` and `queueledger.py` read the rows only to check a cite. `docs/PARKED.md` held 1,832 rows when it was archived. | It ended a second, uncapped queue no session could read. | Is a parked row meant to be read again? If yes, a pass-time listing (a tool, not a file) is the missing half; if no, the rule is working as designed and this is only its size. |

## Summary

| # | file, section | bytes now | action | saved | needs |
|---|---|---|---|---|---|
| R0 | `tools/brief/frame.md`, "opening"; `brief.py` line 458 | 18,475 read by a lane | say what a lane reads of the queue | up to 18,475 a lane session | the pass (its file); D7 |
| R1 | QUEUE, "Where things stand": ten lines whose every row is whole | 5,738 | a closed form, as `Golden:` has | ~5,300 | `rondata::diff::floors` |
| R2 | QUEUE, handoff prose: rules and pinned numbers restated | ~2,600 of 6,967 | strike | ~2,600 | the driver |
| R3 | QUEUE, "How to maintain this file" | 3,454 | one pointer, two superseded bullets, two moved | ~1,300 | lower `QUEUE_SECTIONS` |
| R4 | TRACKS, "The tracks": the state column | ~6,300 of 7,777 | one state a cell, or name the queue's line | ~4,500 | the pass |
| R5 | LOOP, "The chain": the waiter block | 4,931 | point at `tools/lanewait.py`'s header | ~4,000 | closes Loop 4051, 4055 |
| R6 | LOOP, "The brief": kinds no item is booked under | 1,823 | to the retired-clauses record | ~1,600 | none |
| R7 | LOOP, "The driver": the section drivers open with | 13,382 | four gate paragraphs to "The chain"; fold the changelog; move the history | ~1,700, and 3,453 off the opening read | none |
| R8 | LOOP, "Ratification, and the passes": first acts | 2,062 | a list; figures to the records | ~1,000 | the pass |
| R9 | METHOD, "Conventions as they stood", "Tooling as it stood" | 8,563 | delete ten bullets that are `CLAUDE.md`'s word for word | 2,139 | none |
| R10 | METHOD, "Phase 3's scoring and succession" | 1,911 | to the retired-clauses record | ~1,800 | none |
| R11 | `CLAUDE.md` | 25,606 of 26,000 | candidates only | up to ~4,500 | Ramon |
| R12 | the memory index | 7,324 | hooks to one line; one stale pointer | ~2,500 | outside git |
| S1 | DECISIONS, "Index" | 15,985 of 16,000 | split at entry 100; retitle; reorder; fix 121's row | 0 (frees ~8,000) | none |
| S2–S16 | fifteen syncs, Part 2 | | | | |
| G1 | the eight sections pinned in `OVER`, and 45 of the 77 at 15,000 B or more | 284,070; 831,211 | promote their numbered `###` parts to `##` | 0 (largest read 70,859 → 16,594; 45 sections off the ceiling) | delete seven `OVER` rows; three tools to check |
| G2 | `docs/JOURNAL.md`, the chronicle to 2026-09-17 | 1,185,526 | leave, or archive at a revision | 0 or 1.19 MB | `queueledger.py`; Ramon (`CLAUDE.md`'s table) |
| G3 | `docs/DECISIONS.md` | 843,219 | an index by topic; no entry moved | 0 | the pass |
| G4–G7 | `AI.md`, `GOLDEN.md`, `COMBAT.md`, `ORDERS.md`, `RUNS.md`, struck text | 8.1 MB | leave | 0 | |

Read-first total: about 26,000 B of the 101,371 in QUEUE, TRACKS, LOOP,
METHOD and `CLAUDE.md`, and R0 on top for lanes.

## The three to take first

1. **The syncs S9, S2, S5, S6 and S4.** They cost minutes and each is a
   place where a fresh session is told something false today: an opener
   that reaps a row long gone and cuts builders on landed steps; a start
   track "158 of 161" beside a queue that reads 161; a rule that stops
   mid-sentence; one lane in one paragraph and three in another.
2. **R1 with S1: take the two rewritten sections off the ceiling.** One
   parser change and the handoff stands near 10,700 B, not 15,983; one
   heading and the index has 8,000 B of room. Both end a fitting loop
   (D1) that has run on most commits for weeks, and R1 is self-correcting:
   the closed form is accepted only while every row is at its length.
3. **R0, then R7 and R5.** R0 is one sentence of the frame and decides
   whether 93 of 98 lanes read 18 KB or 1 KB. R7 is the section of LOOP
   drivers open with (31 of 43 sessions): a quarter of it is the gate's
   and the tree's rules, which belong with the merge. R5 removes 4,931 B the
   file itself has called unverified since 2026-11-09, by pointing at the
   tool header that already holds every exit.

## Part 1. The read-first set

### What a fresh session reads

| file | bytes | who reads it (measured where it says so) | a guard on its size |
|---|---|---|---|
| `CLAUDE.md` | 25,606 | every session and subagent, whole | ceiling 26,000 (394 B of room) |
| memory index | 7,324 | every session and subagent | none (outside git) |
| `docs/QUEUE.md` | 24,235 | lanes read its head: 93 of 98 lane worktrees since 2026-11-10 touched it, 61 or more by `head -N`; 26 of 43 driver sessions read it whole | handoff 32 lines and the 16,000 B section ceiling; 18 items of 8 lines; two line pins |
| `docs/TRACKS.md` | 11,817 | the driver and the pass: 10 of 43 driver sessions read it whole, 15 a range; 0 of 98 lane worktrees read it whole | section ceiling only |
| `docs/LOOP.md` | 37,586 | the driver, by section: 36 of 43 driver sessions touched it, 31 by line ranges (most often "The driver", lines 11 to about 166), 4 whole; 16 lane worktrees grepped it | section ceiling only ("The driver" 13,382, "The chain" 10,990) |
| `docs/METHOD.md` | 22,127 | residue, chapter, coverage, reading and sweep items, of which few are booked now: one driver session read a range of it and 9 lane worktrees touched it, by range or grep. R9 and R10 are hygiene, not tokens | section ceiling only |
| `tools/brief/frame.md` | 33,632 | a lane, by kind: `brief.py` selects sections, so a render lane does not carry the sweep's 5,759 B | a test holds it to the checklist |
| DECISIONS entries the five files cite | 364,270 over 51 entries | on a pointer | 16,000 a section |

The queue's opener sends the next driver to entry 132 (10,895 B) and five
sections of the pass's record (10,142 B).

The named things in these files resolve. Checked mechanically, each check
made to fail first on a planted name: 59 constants, 28 module paths and 60
tool flags in backticks in `CLAUDE.md`, QUEUE, TRACKS, LOOP, METHOD and the
frame are all found in `crates/` or `tools/`; no backticked repo path in
them names a missing file. The staleness is in what the prose says, not in
what it points at.

### R0. What a lane reads of the queue

- **Where:** `tools/brief/frame.md`, "opening": "Read `docs/QUEUE.md`'s
  opener, then what the item cites."; `tools/brief.py` line 458 prints
  "`docs/QUEUE.md`: the opener, and item N".
- **Measured:** D7's row. The typical call is `head -60 docs/QUEUE.md;
  grep -n "<item>" docs/QUEUE.md`: 18,475 B, of which the lane's own item is
  about 600 and is already in its brief "in the queue's own words".
- **Action:** the frame says what it means. Three candidates for the pass:
  nothing (the brief carries the item and the other lanes' items); the
  headline bullet alone; or the handoff as now, once R1 and R2 have made it
  10 KB. `CLAUDE.md`'s "Read `docs/QUEUE.md` first" then needs scoping to a
  session with no brief (R11).
- **Saved:** up to 18,475 B a lane session (about 5,600 tokens, 2.9 % of a
  lane request's mean context). With R1 and R2 alone, about 7,900 B.
- **What cites it:** `tools/explore/test_brief.py` holds the frame's
  sections to the checklist in `docs/audit/README.md`; the opening line has
  no row number.
- **Confidence:** high on the measurement; the choice is a judgment about
  what a lane should know, and is the pass's.

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
  turns the short line red and the row is written out again.
- **What it would read** (a sketch of the form, 380 B where 5,738 stand;
  the wording is the pass's):

      Long captures: every word whole · 2
      Second pair: every word whole · 2
      Third pair: every word whole · 2
      Coverage pair: every word whole · 1
      Successor pair: every word whole · 1
      Peace pair: every word whole · 1
      Late ages: every word whole · 82
      Battery word: every word whole · 28
      Melee: every word whole · 6, 0 off
      Third map: every word whole · 4

- **Saved:** about 5,300 B (5,700 with `Mapgen:`), a third of the section.
- **What cites it:** `crates/rondata/src/diff/floors.rs` reads each line by
  prefix (`the_handoff_carries`, `ai_line_verdict`, `ai_line_closes`, and
  the per-line tests for the late ages, the battery word, the second,
  third, coverage, successor and peace pairs, the melee and the third map,
  and `every_closing_line_row_has_its_close`); `testkit.rs` names
  `NEWEST_PAIR = "Late ages"`, whose reader this lane did not read;
  `bantu_word.rs` cites the line in a comment. Nothing under `tools/`
  parses these ten.
- **Guard:** those tests, and `docs_guard::the_handoff_is_short`. Made to
  fail first by lowering one row's word under the closed form.
- **Rule touched:** the queue's "The floors and these lines move together
  ... read literally" bullet gains the closed form. No decision forbids it.
- **Confidence:** high on the saving and the precedent; medium that the
  parser change is one function and `NEWEST_PAIR`'s reader.

### R2. The handoff's prose

- **Where:** `docs/QUEUE.md`, "Where things stand", the dated line and the
  two bullets: 6,967 B on three lines.
- **What repeats a rule from another read-first file** (1,109 B measured):
  "The pilot ... gated once at the merge, scoped ... One kill rule ... a
  soft stop at 99 %" (292 B; LOOP, "The driver"); "Workers: none out; four
  stand ... a fifth or sixth is Ramon's" (180 B; LOOP's roster; TRACKS,
  "The workers"); "The passes ...: the gate takes every merged landing and
  is scoped ... a lane clears at 400 k; a handover is `tools/handover.py`"
  (637 B; LOOP, and TRACKS for parity's definition).
- **What repeats a pinned line fifteen lines below it** (about 1,500 B, by
  reading): the `INPUT_*` counts and "24,000 of 24,000" (`Recording:`);
  "Eight of twelve lines", three times (`Checksum:`); "107 of 109 packets
  ... 314 of 1,053 lines" (`Checksum:`); "The territory 733 · 332 · 574"
  (`Rubric:`); "The blind list: open 0 (never 336 · residue 336)"
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
  needs no ledger line; a booked item that leaves does. No guard parses the
  prose.
- **Confidence:** medium. Editorial, and the driver's to write; the list is
  the evidence, not a draft.

### R3. The queue's maintenance notes

- **Where:** `docs/QUEUE.md`, "How to maintain this file", 3,454 B, 38 lines.
- **Stale or superseded:** S4 (the chain is LOOP's, not `CLAUDE.md`'s); S3
  (the never-empty slots; D5); and two bullets that are method for a
  residue item ("A capture is a draw-stream trace first ...", "Grep for the
  derived quantity ..."), which METHOD holds and is where those kinds are
  sent.
- **Action:** keep the four bullets about this file (end of session; a
  worker never books; the floors and these lines; the gate line). Fix the
  pointer, strike the slot rule on D5, move the two to METHOD.
- **Saved:** about 1,300 B.
- **Guard:** `a_queue_section_may_only_shrink` pins this section at 38
  non-item lines and **fails when it comes in four or more under the pin**,
  so `QUEUE_SECTIONS` is lowered in the same commit, in
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
  header already says where the earlier ones are). Or no state: the
  "number" column names the queue line that carries it (`Start:`,
  `Renderer:`, `Checksum:`, `Recording:`, `Lobby:`, `Rubric:`), which a
  guard keeps true on every commit and this column nobody does.
- **Saved:** about 4,500 B, and S2's seven stale facts.
- **What cites it:** nothing in `crates/` or `tools/` parses TRACKS and
  `docs_guard` names it nowhere. LOOP and QUEUE cite it for the milestone
  and the workers, never for a state.
- **Rule touched:** none ("The steering pass maintains this page").
- **Confidence:** high.

### R5. The waiter block of the chain

- **Where:** `docs/LOOP.md`, "The chain", from "And a spawn is followed by a
  wait on the lanes" to the `UNCLEARED` clause: 4,931 B of the section's
  10,990, in one paragraph.
- **Why:** LOOP says of it, twice, that it is unverified on the harness of
  2026-11-09 and owed a shortening ("Its waiter clauses are to be
  re-measured on the new harness (Loop 4051) and then shortened (Loop
  4055)"). `grep -rl 'Loop 4051\|Loop 4055' docs tools crates` finds LOOP,
  the seventy-sixth pass's record and the parked archive: no pass since has
  ruled either. `tools/lanewait.py`'s header lists every exit (`LANDED`,
  `ENDED`, `HELD`, `QUIET`, `DEEP`, `STALLED`, `UNCLEARED`) and every line
  that is not one (`WAITING`, `PUSHED`, `DRAFT`, `ARMED`, `ARM-REFUSED`,
  `GATING`) with its answer and the case that taught it. The block is that
  header retold as prose.
- **Action:** a pointer to the header, and the three rules the tool cannot
  hold: a done line that arrives while a wait runs stops and re-arms the
  wait without that lane; a message is never sent to a lane beside an armed
  clear; an exit is answered in its turn. A case in neither the header nor
  a pass record goes to `docs/audit/2026-11-09-loop-retired-clauses.md`,
  which entry 116 made for this.
- **Saved:** about 4,000 B of what a driver opens at a merge (drivers
  read LOOP by range; R7 is the part they open with).
- **Guard:** none reads LOOP.
- **Confidence:** medium-high. Not established: Loop 4051 itself, whether
  `DRAFT`, `HELD` and `UNCLEARED` still fire. A pointer is true either way.

### R6. The brief kinds no item is booked under

- **Where:** `docs/LOOP.md`, "The brief", from "The fourth kind is
  `reading`" to "the pass's `--record` row stays the pass's": 1,823 B of
  the section's 2,580.
- **Why:** it specifies the `reading` and `coverage` kinds, the late-age
  pair's shape and the rubric's re-pin. The handoff reads `Blind: ... open
  0`, every `Late ages:` row whole and "ages held 8 of 8", and the queue
  books a residue or chapter item "only when another track's frame needs
  one". `tools/brief.py`'s header lists all nine kinds with their gates.
- **Action:** to the retired-clauses record; LOOP keeps "the kinds and
  their gates are `tools/brief.py`'s header" and the start and input
  sentence.
- **Saved:** about 1,600 B.
- **Confidence:** medium. A kind can be booked again, and then the tool's
  header is what a driver reads.

### R7. The driver section's history

- **Where:** `docs/LOOP.md`, "The driver", 13,382 B. This is the part of
  LOOP a driver does read (31 of 43 sessions, by range), so a byte here is
  worth more than one in "The chain".
- **Its fourteen paragraphs, in bytes:** one session drives 1,594; Ramon
  steers 317; banks and clears 1,735; supply calls the pass 547; who builds
  1,503; the roster 1,713; the kill rule and the stop 947; three things
  reach Ramon 134; the trunk 501; the booking gate takes every landing
  865; the booking gate is scoped 751; a tree's corpus 645; the gate clears
  the disk 1,192; the harness moved 909. Four of them (the booking gate
  takes every landing, the booking gate is scoped, a tree's corpus, the
  gate clears the disk: 3,453 B) are rules of the gate and the tree a
  driver needs at a merge, not at its start, and "The chain" is where the
  merge is; moving them there costs no bytes and takes a quarter off the
  section drivers open with.
- **History inside the rules:** in "one session drives", the two failures
  the handover mechanism answers ("312,347 tokens written", "the pass
  armed at 01:30 on 2026-11-13 never started") are entry 131 §8's; in
  "banks and clears", the 230 k figure's three measurements are entry 131
  §6's and entry 116's; in "the gate clears the disk", the 0 GB reading
  and its counts are entry 128's and 129's.
- **Items:** "The harness moved under this file" (909 B) is a changelog of
  2026-11-09: two of its five bullets are rules (a message to a running
  lane wakes it mid-gate, so it is sent only when the lane must act first;
  a clear no longer kills a background command), one is said again forty
  lines up ("A builder on another model takes that model's own default
  effort"), and the rest is history. The roster's measurements (953 B: the
  trial tranche's false reds, "27 tranches of that roster read 7.25 USD")
  are in the passes' records. The stop's history ("95 % from 2026-11-11, 85
  % before") and the clear threshold's ("7.77 USD against 4.27") are in
  entries 121 and 131. "Three things reach Ramon at once" stands here (134
  B) and again in "Ratification, and the passes" (279 B), in other words.
- **Saved:** about 1,700 B.
- **Confidence:** medium-high.

### R8. The pass's first acts

- **Where:** `docs/LOOP.md`, "Ratification, and the passes", from "Its
  first act is the archives' compression" to the blind list's scan: 2,062 B.
- **Why:** three passes' disk readings ("the sixty-fifth read 114 ... the
  sixty-seventh 138 ... 151 after") and a scan of the blind list's open
  rows, which read 0. Since entry 128 §1 the gate itself clears the disk.
- **Action:** the acts as a list with their commands; the figures to the
  records; the blind scan to the retired clauses with R6.
- **Saved:** about 1,000 B.
- **Confidence:** medium. Whether the compression is still a first act is
  the pass's to say.

### R9. METHOD's copies of `CLAUDE.md`

- **Where:** `docs/METHOD.md`, "Conventions as they stood" (4,284 B) and
  "Tooling as it stood" (4,280 B).
- **Measured:** of their seventeen bullets, **ten are `CLAUDE.md`'s word
  for word** (2,139 B; compared with the line wraps joined): earn every dependency; a probe shape graduates;
  `tools/guard.sh` is the reflex; the python-heredoc patch; the pinned
  toolchain; where format notes and decisions live; one document a
  mechanic; `llvm-pdbutil`; constants and their representation; the
  `rondata` survey. The other seven are the long forms `CLAUDE.md`
  shortened.
- **Action:** delete the ten. Every reader of METHOD has inherited
  `CLAUDE.md`, and METHOD's header says `CLAUDE.md` is the rule where they
  differ, so a copy can only drift.
- **Saved:** 2,139 B, nothing lost.
- **Guard:** none reads METHOD; `tools/brief.py` names it for five kinds.
- **Confidence:** high.

### R10. METHOD's phase 3

- **Where:** `docs/METHOD.md`, "Phase 3's scoring and succession", 1,911 B.
  Its own first lines: "As `CLAUDE.md` stated phase 3 while it was open.
  Every word it names is closed and stands as a floor."
- **Action:** to the retired-clauses record; the succession rules are in
  entries 53 to 56 and 61.
- **Saved:** about 1,800 B.
- **Confidence:** high that it is history.

### R11. `CLAUDE.md`: candidates, not a draft

Its edits are Ramon's word. It stands 394 B under its ceiling, so the next
clause that wants in must cut one. What a render, start or input lane does
not use, by measured size:

| lines | block | bytes | note |
|---|---|---|---|
| 270–295 | the blind second reading | 1,886 | the blind list is open 0, and a reading item reads METHOD. Its last sentence (this file names nothing a reader re-derives) is a rule about this file and stays. |
| 77–121 | the phases, each with its state | 2,829 | a phase's state is TRACKS's; the arc is five lines. |
| 197–215 | the permission-shaped failure | 1,223 | the rule is three sentences; the rest is its cases. |
| 413–424 | prior art | 651 | not operational. |
| 147–150 | "Read `docs/QUEUE.md` first ... then `docs/TRACKS.md`" | 280 | not what lanes do (D7); scope it to a session with no brief. |
| 408 | "(`docs/ORACLE.md`, last section)" | | S10: that is no longer the last section. |

### R12. The memory index

Outside git, and a session's own to keep; proposed, not edited. 7,324 B in
32 lines, inherited by every session and subagent. Three hooks run past
300 B (598, 422, 329), where the index's own rule is one line a memory.
One hook points at a section that is gone: "Subagent model split —
graduated to git 2026-08-25: CLAUDE.md "Fan-out rules"": `CLAUDE.md` has no
such heading; the fan-out rules are the last paragraph of LOOP's
"Ratification, and the passes" (entry 83 moved them). Saved: about 2,500 B
if every hook is held to 150.

## Part 2. Doc syncs

Each row is a claim a later decision, the code or the tree contradicts,
with the evidence. "Fix" is the smallest edit that makes it true.

| # | where | the claim | the evidence against it | fix |
|---|---|---|---|---|
| S1 | DECISIONS, "Index — what stands, as of 2026-10-06" | the date; "one line per entry" | 132 rows, the newest of 2026-11-13; 15,985 B of 16,000. Out of order: 82 follows 85; 91 and 93 to 132 precede 92, 90, 89, 88, 87, which close the list. Rows 111 to 126 are a few words ("113 extended by 114 — rows booked") where 76's is 197 B. Row 121 reads "standing — no kill rule; soft stop 95 %"; the entry was amended in place twice and says 99 % ("Amended 2026-11-13, evening"). | split into "Index, 1 to 99" and "Index, 100 on" (`every_decision_has_an_index_row` reads every heading that starts `Index`); restore the order; 121 "amended in place"; drop the date. |
| S2 | TRACKS, "The tracks", state column | start: "158 of 161 ... the three left on two styles with no mask port (item 4211)" | QUEUE: "`Start: 161 of 161 lobbies`", "The start track: closed" | R4 |
| | TRACKS, "The milestone", step 1 | "(`Start: 1 of 161`)" | the same | R4 |
| | | renderer: "K 59 of M 59" | QUEUE `Renderer: K 59 of M 60` | R4 |
| | | checksum: "150 of 1,058 lines" | QUEUE `Checksum: ... lines 314 of 1,053` | R4 |
| | | orders: "next ... the attack played (items 4194, 4195)" | QUEUE: `INPUT_ATTACKED` 10 of 10 (4195), `INPUT_FOLLOWED` 12 of 12 (4202) | R4 |
| | | lobby: "next ... a plain HUD (items 4197, 4198)" | QUEUE: both landed; the `Hud:` line | R4 |
| | | structure: "**done**" | QUEUE books 4224, a `structure` item marked whole | R4 |
| S3 | QUEUE "How to maintain this file"; METHOD "Booking an item" | "Neither headline slot is ever empty"; "the headline slot is never empty" | QUEUE "The queue": "A residue or chapter item books only when another track's frame needs one"; TRACKS, rules: "no item stands" | strike, on D5 |
| S4 | QUEUE "How to maintain this file" | "The chain — merge, book, spawn, gate, push, reap — is `CLAUDE.md`'s" | `CLAUDE.md`: "The loop's machinery is `docs/LOOP.md`" | the pointer |
| S5 | LOOP, "The chain", its last line | "The turn does not end with a lane empty and the queue not, and it never asks whether to" | the sentence stops there. `git show 75ffba6d1:docs/LOOP.md` has its end: "continue — the queue's opener is the answer." Lost in `15aae746e` (2026-11-09). | restore seven words |
| S6 | LOOP, "The lanes", first paragraph | "one ccc lane on the render track and the driver's builders on the chain" | LOOP's own roster: "three ccc lanes ... four workers with the chain's builder, confirmed at the pass by load (DECISIONS 129 §1)"; QUEUE "four stand" | strike the count; the roster holds it |
| S7 | LOOP, "The chain" | "the whole gate ... (DECISIONS 132 §5; every third landing whose merge widened, before it)", with entries 72 and 79's history | entry 132 §5: "This replaces ... entry 79's every third landing" | strike the parenthesis (405 B) |
| S8 | METHOD, "The definition of done" | "the lane gate runs those, the fast tests and the guards"; "the commander's booking gate takes no flag" | entry 132 §6: a lane's gate "leaves every timed test to the booking gate but its filters and the paperwork guards, and runs whatever the timings file has never timed"; §5: the booking gate takes `--scope`. `CLAUDE.md` carries the new form. | amend in place |
| S9 | QUEUE, the dated line and the opener | "handing back to `commander`"; "cut a builder on **VII-b1** and spawn three lanes, 4208 ... 4212 ... 4214 ... and a fifth on 4216" | all landed: `docs/journal/2026-11-13-item-4208.md`, `-4212`, `-4214`, `-4216` exist, and the queue's item 4159 says VII-b1 to VII-d landed. 21 commits edited the queue since the pass's gate (`git log 8a0b2a0e7..HEAD -- docs/QUEUE.md`); none rewrote these lines. A row that died now would hand a fresh session an opener that reaps a row long gone and cuts builders on landed steps. | the booking commit keeps the opener true, or the mid-tranche opener is "continue the tranche" with the lanes that are out |
| S10 | `CLAUDE.md` line 408; METHOD line 112 | the behavioural check is "(`docs/ORACLE.md`, last section)" | ORACLE's last section is "The click-free lanes are a pool (2026-10-07 ...)". The recipe is "Running a check: the recipe in one place", the fourth section. | name the section (in `CLAUDE.md`, Ramon's word) |
| S11 | LOOP, "The chain" | the chain ends "`ccc rm <ref>`, and `git push origin --delete <the lane's branch>`" | LOOP, "The driver": "a remote branch delete is refused from every row". 37 of 39 remote branches are merged into `origin/main` (`git branch -r --merged origin/main`). | say whose the delete is (Ramon's), or what reaps a merged remote branch |
| S12 | LOOP, "The lanes" | "The handoff's `For the pass` counts are the tables'" | the handoff has no "For the pass" part (`grep -c 'For the pass' docs/QUEUE.md` is 0) | say when the part exists (at a hand-over to the pass), or strike |
| S13 | twelve doc-to-doc section cites | a section that does not exist: `AI` §4.4 (CITIES line 531) and §9.3 (ORACLE 1841); `ANIM` §4.4 (ORDERS 2059); `ARMY` §15.8 (COMBAT 2687, four times) and §16.3 (INPUT 454); `COMBAT` §14.1 (MOVEMENT 107); `NATIONS` §5 (DECISIONS 6619); `ORDERS` §714 (INPUT 203, a line number); `PATHFINDER` §18.6 (COMBAT 12899) and §4.6 (TRANSPORT 17, twice); `TECH` §13 (AI 14501) and §7 (ANIM 2214) | `a_section_the_code_cites_is_one_the_document_holds` reads `crates/` only; the same parser over `docs/*.md` finds these. Six of the twelve are sections its `NO_SUCH_SECTION` already pins for the code. | mend each; or extend the guard to the documents with these pinned |
| S14 | four paths | `crates/rondata/src/mapped.rs` (DATALAYER line 131); `tools/gamelog/handdriver.sh` (CENSUS 619; RUNS twice); `tools/ghidra/decomp` (VISION 9, RECGAME 12, COMBAT 4444) | none exists in the tree (`tools/ghidra` holds `analyze.sh`, `export.sh`, `run.sh`, `scripts`); the export is outside the repo by rule | name the file that holds it now; say "the export" without a repo path |
| S15 | `docs_guard.rs`, `OVER` | four pins: `ORDERS.md` §4 37,583; `AI.md` §2 70,933, §15 26,923, §99 51,530 | the guard's own stderr asks for each to be lowered: "shrank to 37575 ... 70859 ... 26922 ... 51527; lower its pin" (86 B of slack; a passing test's stderr is captured, so nobody sees it) | lower the four; or make the slack a failure, as the queue's pins are |
| S16 | DECISIONS, "Index": statuses | 116 "amended by 121"; 100 "standing"; 59 "extended by 60" | entry 126 §1: "Entry 116's single row and entry 122's roster line are superseded here"; entry 103: "amends entry 100 §3's "next cell""; entry 61: "amends ... entry 59's clause 1". Three read; a mechanical pass (a later entry's sentence that names an earlier one beside amend, replace, retire, withdraw or supersede) lists 7 rows the index calls standing (57, 64, 71, 72, 99, 100, 121) and 16 whose list of successors is short; 16 entries carry an "Amended <date>" note while their row says something other than "amended in place" (4, 9, 24, 29, 34, 55, 63, 64, 68, 71, 72, 77, 78, 87, 88, 121). Candidates, not verdicts: the heuristic is noisy, and a successor does not always name what it changes (entry 126 returns the two rows without a sentence that says "amends entry 116" in those words). | the pass that splits the index (S1) checks these rows; G3's index by topic is the durable fix |

Also seen, and not a contradiction: `CLAUDE.md` gives the gate as
`--test-threads 4` and QUEUE and LOOP give the booking gate as
`--test-threads 8`. The four is the worker's line; the eight is the
driver's. One sentence in `CLAUDE.md` would say so.

## Part 3. The giants

| file | bytes | `## ` sections | sections no `crates/` code cites | struck bytes | over 16,000 | 15,000 to 16,000 |
|---|---|---|---|---|---|---|
| AI.md | 2,824,624 | 438 | 36 (90,695 B) | 45,261 | 3, pinned | 15 |
| RUNS.md | 2,522,798 | 1,391 | cited by heading, 572 mentions | 1,090 | 0 | 0 |
| GOLDEN.md | 1,463,800 | 240 | 26 (94,115 B) | 16,296 | 0 | 12 |
| JOURNAL.md | 1,303,935 | 257 | not measured by the guard | 3,320 | 3 | 0 |
| DECISIONS.md | 843,219 | 134 | by entry: 70 of 132 cited by no code or tool | 1,540 | 0 | 2 |
| COMBAT.md | 768,266 | 88 | 8 (38,254 B) | 25,513 | 0 | 9 |
| ORDERS.md | 494,693 | 58 | 10 (33,386 B) | 10,890 | 2, pinned | 5 |

Beside them: `docs/journal/` 10,220,695 B in 1,529 files (337 of
September, 856 of October, 336 of November so far); `docs/audit/` 8.3 MB in
225 files; `docs/lab/` 2.2 MB.

### A hypothesis, killed

"The item-titled sections repeat their journals, so the history can move
out." Measured by six-word shingles between each item-titled section and
its item's journal file: `AI.md` 349 sections, median overlap 0.06, none
over 0.31; `GOLDEN.md` 181, median 0.06; `COMBAT.md` 70, median 0.06;
`ORDERS.md` 29, median 0.03; `MAPGEN.md`, `RENDERER.md`, `CHECKSUM.md` 0.02
to 0.03. Between each DECISIONS entry from 57 on and its nearest pass
record: median 0.09; with `docs/JOURNAL.md`, 0.01. The section and the
journal tell one landing in different words and neither is a copy. So
nothing in the giants can be deleted as a duplicate.

**A second, also empty.** "The specifications still speak of landed items
in the future tense." A scan for unstruck sentences that pair an item
number with "owed", "until item", "is booked", "not yet" and the like,
where the item has a journal file, lists 80 sentences in 24 documents. The
eight read on the live tracks (`CHECKSUM.md`, `START.md`) are all dated
history ("until item 4113"), none a stale promise. Not pursued.

### G1. The eight sections over the ceiling split where they are already numbered

- **Where:** the rows of `OVER` in `docs_guard.rs`: `AI.md` §2 (70,859 B
  live), §99 (51,527), §15 (26,922); `GROUPS.md` §6 (44,660); `ORDERS.md`
  §4 (37,575), §5 (16,056); `DATALAYER.md` §2 (20,130); `CITIES.md` §3
  (16,280). 284,070 B.
- **Measured:** every one is built of `### ` parts, and in seven the parts
  are numbered (`AI.md` §2 has twenty, the largest 12,100 B). The only part
  over 16,000 is one of `GROUPS.md` §6's, at 16,594. `DATALAYER.md` §2's
  seven parts are not numbered.
- **Action:** promote each numbered `### N.M` to `## N.M`. It is the split
  the guard's comment asks for ("a `## ` heading is the split, and it costs
  nothing a reader needs") and `docs/RENDERER.md` already numbers its
  sections this way (`## 3.6`, `## 3.12`). No number changes:
  `sections_of` reads a number at any heading level, so every code
  citation resolves as before.
- **Saved:** no bytes. The largest section a pointer can send a session to
  falls from 70,859 B to 16,594, and seven rows leave `OVER` (the guard
  demands the deletion once a section is under the ceiling). `GROUPS.md`
  §6's one part stays pinned, or splits again.
- **The same move, where lanes are landing now.** Of the 77 sections at
  15,000 B or more outside QUEUE, DECISIONS, RUNS and the journal, **45
  (831,211 B) are made wholly of numbered `###` parts that are each under
  16,000**: promotion alone takes them off the ceiling (`AI.md` 12,
  `COMBAT.md` 9, `ORDERS.md` 7, `LEADERS.md` 4). 27 have no `###` part and
  need an authored split. The ones on live tracks, by what changed since
  2026-11-07: `RENDERER.md` §5 "The counter" grew from 4,449 B to 15,975 in
  the week and stopped there (four numbered parts; 124 commits touched the
  file); `LEADERS.md` §7 (15,901), §8, §10 and §18, which the chain's item
  4159 amends at every step; `ORDERS.md` §8 (13,625 to 15,485).
  `CHECKSUM.md` §20, the chain's own section, is 15,995 B in one block
  with no part to promote: its next amendment is a fitting episode unless
  it is split first. Sections of `AI.md` that no item is booked on moved
  by 3 to 12 B in the same week and stayed within 20 B of the ceiling
  (§331 15,984 to 15,996), which is what an amendment under a full
  section looks like.
- **What cites it** (citations of the section or a part of it, in
  `crates/` and in other documents): `AI.md` §2, 71 and 33; §99, 46 and
  12; §15, 8 and 8; `GROUPS.md` §6, 53 and 34; `ORDERS.md` §4, 49 and 53;
  §5, 11 and 16; `CITIES.md` §3, 22 and 17; `DATALAYER.md` §2, 0 and 1.
  None cites a heading level.
- **Guard:** `a_section_over_the_ceiling_may_not_grow`. Three tools split
  a document on `## ` and were not read by this lane: `tools/retire.py`,
  `tools/runwiring.py`, `tools/seams.py`; `tools/brief.py --section` only
  passes its string through. Check each before promoting in a document it
  reads.
- **Confidence:** medium-high.

### G2. `docs/JOURNAL.md`

- **Measured:** 1,185,526 B of its 1,303,935 (91 %) are the 234 entries to
  2026-09-17, the day the per-item files began; `CLAUDE.md` calls the file
  "the chronicle to 2026-09-17 and the passes' entries since". Since then
  it has 22 entries, 117,340 B; there are 61 pass records under
  `docs/audit/` since 2026-10-01 and four journal entries, so a pass's
  story is its record now.
- **What reads it:** `tools/queueledger.py` reads it on every run as half
  of "one journal" (a number that left the queue must be named there);
  `docs_guard` names it nine times, mostly to exclude it; `seams.py`,
  `rubric.py` and `release_gate.py` name it.
- **Options:** leave it (it costs no session anything; nobody is sent to
  read it). Or archive the chronicle at a revision with an index, as
  `docs/PARKED.md` was (entry 125: `ARCHIVE_REV` in `queueledger.py`, a
  frozen index under `docs/audit/`), which takes 1.19 MB out of the tree
  and needs the ledger to read the item numbers from the revision once.
- **Rule touched by the second:** "`docs/JOURNAL.md` is the main thread's"
  stays true; `CLAUDE.md`'s table row changes, which is Ramon's.
- **Recommendation:** leave. The only gain is a smaller `grep docs/*.md`.

### G3. `docs/DECISIONS.md`

- **Measured:** 132 entries, 827,065 B, and the index. By era: 1 to 33,
  79,230 B; 34 to 56, 142,544; 57 to 99, 347,665; 100 to 113, 155,027; 114
  to 132, 102,599. The 54 entries whose title scores a tranche, a pass or a
  steer hold 478,745 B. In the 63 entries with a "Decided" marker, 25,768 B
  stand before it and 346,493 after: the measurements are woven into the
  numbered decisions, so no cut keeps "the decisions" and drops "the
  story" without rewriting sentences. Code cites entries 676 times in
  `crates/` (entry 36, 172 times; 72, 166; 90, 85) and 123 in `tools/`.
- **Tier 0, no rule change: S1.** Split the index, restore its order, fix
  121's row.
- **Tier 0, additive: an index by topic.** The question a session has is
  "what stands about the gate", and the answer is a chain. From the
  index's own rows and titles (not from reading the entries): the booking
  gate, 72 → 79 → 131 §4 → 132 §5; a lane's own gate, 53 → 72 → 80 → 96 →
  132 §6; the roster and the count, 59 → 61 → 114 → 116 → 126 → 128 §4 →
  129 §1; the rows and the handover, 61 → 116 → 122 → 126 → 130 → 131 §8;
  the stop, 118 → 121 → 124 → 121 again, amended in place (99 %); the parked
  file, 39 → 67 → 125; the finish line, 24 → 26 → 29 → 41 → 53 → 114 → 116
  → 132 §10. About twenty such lines, each ending on the entry that
  stands, is the page Ramon can read to find what to review. It can live
  in the index's second half; the rows by number can then be bare.
- **Tier 1, a rule change: freeze the tranche-scoring entries at a
  revision**, each keeping its heading, its index row and a paragraph of
  what still stands, the text at `git show <rev>:docs/DECISIONS.md`.
  Precedent: entry 125. It takes about 400 KB out and breaks nothing a
  guard checks. It also puts 166 code citations of entry 72 one `git show`
  away from their text, and the paragraph of "what still stands" is 54
  retellings with no mechanical check. The rule it changes: "append; amend
  in place; never delete" becomes "an entry may stand by pointer".
- **Recommendation:** tier 0, both parts. Not tier 1: nobody reads the
  file whole, so its bytes are not a cost, and the retelling is a risk.

### G4. `AI.md`, `GOLDEN.md`, `COMBAT.md`, `ORDERS.md`: leave

A section of these is read on a pointer and cited by code: of `AI.md`'s 438
sections, 19 (37,146 B) are cited by nothing but journals and audits; of
`GOLDEN.md`'s 240, 16 (41,024 B); `COMBAT.md` 4 of 88; `ORDERS.md` 7 of 58.
That is 1.3 to 4.4 % of each file, not worth an edit to an addressed
document. What each does have is sections fitted to the ceiling (the last
column above): the next amendment to any of the 30 at 15,900 to 16,000 is a
fitting episode or a split. A split is free (G1's method), so it is the
amendment's first move and no proposal is needed.

### G5. `docs/RUNS.md`: leave

A well-shaped ledger: 1,390 sections, median 1,632 B, largest 10,199, none
near the ceiling. 1,344 sections (2,421,416 B) are of runs below 1546,
which `rondata::corpus` does not admit by heading; 41 (81,017 B) are from
1546 on and are parsed. Code and tools name the file 572 times and cite it
by heading. It could be split by run range exactly as it was split out of
ORACLE ("nothing was rewritten in the move"), and nothing would be gained.

### G6. Struck text: leave

184,609 B across the specifications (`AI.md` 45,261, `COMBAT.md` 25,513,
`GOLDEN.md` 16,296): 1.3 % of the tree's top level, already uncounted by
the ceiling, and each span is a wrong claim kept from being read as live.

### G7. `docs/audit/README.md`: one section

137,908 B in 45 sections; `CLAUDE.md` points an adjudicator at it for "its
record and its lessons". "The record" is 30,524 B, the largest section in
the tree outside `OVER` and `JOURNAL.md`, and `docs/audit/` is not under
the ceiling. 37 of the 45 sections are "The brief checklist", one a pass
from the eighteenth to the sixty-second, held to the frame by
`tools/explore/test_brief.py`; `brief.py` points a lane there for "the
story behind any row", on demand. The frame is the digest and this is its
history, read a row at a time. Leave it; a pointer into it could name the
section.

## Not established

- Bytes per token for this prose; every token figure scales with it.
- Whether a lane that reads less of the handoff parks worse (R0, D7).
- `lore`'s index was not refreshed by this lane; the session counts are as
  of its last run, and a tool-lane search returns snippets, so "at least
  61" is a floor.
- `NEWEST_PAIR`'s reader, and whether any tool reads a `## N.` section as a
  unit (R1, G1).
- The statuses in the DECISIONS index were read for entries 59, 100, 116
  and 121; the rest of S16 is a heuristic's list.
- The topic chains under G3 are read off the index, not off the entries.
- Whether merges conflict on the live tracks' documents (D6).
- What `tools/retire.py`, `tools/runwiring.py` and `tools/seams.py` do with
  a `## ` heading (G1).

## Appendix. The measuring scripts

Scratch, in the lane's job directory, and gone with it; the logic is small
enough to restate. A probe reached for a third time would graduate to
`tools/`; none has been yet.

- **Sections** (a port of `docs_guard::sections`): split on lines that
  start `## `; a section's bytes are its lines plus a newline each; then
  for every `~~` that is not part of `~~~`, pair it with the next one in
  the same section and subtract the bytes between. Checked against the
  guard: run with `--nocapture`, `a_section_over_the_ceiling_may_not_grow`
  prints 37,575, 70,859, 26,922 and 51,527 for the four sections under
  their pins, the port's figures to the byte, and the other four equal
  their pins.
- **History of a section:** for the last N commits of `git log
  --first-parent -- <file>`, `git show <rev>:<file>` through the port.
- **Citations:** `docs_guard`'s `section_number`, `sections_of` and
  `section_cites`, run over `crates/**/*.rs`, `tools/**`, `docs/*.md`,
  `docs/journal/` and `docs/audit/`, each citation credited to the
  longest numbered `## ` heading that prefixes it.
- **Overlap:** lower-cased words of each text as a set of six-word
  shingles; the share of the section's set found in the other text.
- **Pointers:** every backticked `tools/…`, `docs/…` or `crates/…` path
  tested with `os.path.exists`; every backticked constant, module path's
  last segment and `--flag` grepped under `crates/` and `tools/` with
  markdown excluded. Made to fail first on a planted name of each kind.
- **Lanes:** `lore search '<file>' --lane tool --well attrition --since
  2026-11-10 -n 3000 --format jsonl`, grouped by worktree; `lore usage --by
  session --well worktrees-att- --since 2026-11-10 --limit 500`.
