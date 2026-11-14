# A compaction proposal for `docs/` (2026-11-14)

A lane on `worktree-att-docs`, Fable 5.2 (`claude-fable-5-2`), asked for by
Ramon on the commander's row: "an openbounded fable suggest doc trims/doc
syncs? e.g. kind of a compactor of our docs?". **This file proposes; it
edits nothing.** No other document, guard or tool was touched. The next
`steer` pass decides each row. **A reader with ten minutes reads to "The
three to take first"; everything after is the evidence.**

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

**Two sessions wrote it.** The first ran to 02:10 −05:00 and stopped on
its context. The second (02:12 to 05:00, the same model, a fresh context)
worked through the first's "Left to do": it corrected three of the first's
figures where a measurement replaced a reading (R2, G1's largest part, two
of G3's chains), ran G1 dry against the guards, and added S19 to S25 and
G9 to G12. Where the two disagree the text says which session measured
what.

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
sentence cut off mid-clause, two counts of lanes in one file, three pin
counts in the handoff that the code left two days ago), and it is
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
| D6 | A track's stub pool holds two unassigned sections (`docs_guard`; parked 1718, 1741) | The guard holds `AI.md` and `GOLDEN.md` only: the two tracks the handoff calls whole. Eight stubs stand there (`AI.md` §412, §417 to §419; `GOLDEN.md` §195 to §198). The documents four workers do append to (`RENDERER.md`, `CHECKSUM.md`, `LEADERS.md`, `CLIENT_INPUT.md`) have no pool. Since 2026-11-09, 9 of the 18 merges that touched `RENDERER.md` carry a hunk that is neither parent's (`git show --cc`, non-empty), 1 of 17 on `CHECKSUM.md`, 1 of 1 on `CLIENT_INPUT.md`; none on `AI.md` (no merge touched it) or `GOLDEN.md` (one did). `f1c41c606`'s subject says what they are: "both sides' ground pins and RENDERER §3.18 and §3.19 kept". **The journals, read in the second session, say which were conflicts:** 12 of the 132 item journals of 2026-11-10 to 11-14 record a `ccc update --keep-conflicts`; six name `docs/RENDERER.md` (4160, 4165, 4168, 4169, 4174, 4199), one `docs/CHECKSUM.md` (4209), one the parked file (4089), none `AI.md` or `GOLDEN.md`; the other four are code (`gpu.rs`, `ground.rs`, `world.rs`, `main.rs`, `observe.rs`). 4169's "merged them cleanly", so a `--cc` hunk over-counts; three say "both sides kept". The gate already has a word for it: `release_gate.py`'s `row_list_overlap` (Loop 3694) calls any `docs/*.md` both sides touched "row lists on the base's side" and lets the landing stand on its verdict. Before the pool, the same scan over 2026-11-03 to 11-09 reads `docs/PARKED.md` 15 times (archived since, entry 125), `MAPGEN.md` 7, `AI.md` 4. | Seven of twenty-four merges backed out before the pool. | Move the pool to the documents lanes append to (`RENDERER.md` first), or retire it where no lane lands? |
| D7 | The brief's frame: "Read `docs/QUEUE.md`'s opener, then what the item cites" | Lanes read it as the head of the file. Of 98 lane worktrees with a session since 2026-11-10, 93 touched the queue and at least 61 ran `head -N docs/QUEUE.md` (41 at 60 lines, 18,475 B; 14 at 40 lines, 15,259 B; `lore search --lane tool`, a floor). In `CLAUDE.md`'s words the opener is the handoff's last line, 717 B, written to the driver. No lane read `docs/TRACKS.md` whole (0 of 98; 6 read a part), though `CLAUDE.md` says to. | A lane with the headline in view parks better. | What does a briefed lane need from the handoff: all of it, the first bullet, or nothing its brief does not carry? R0. And `CLAUDE.md`'s "then `docs/TRACKS.md`" is not what lanes do; scope it to a session with no brief? His file. |
| D8 | A finding that names no score parks in its item's journal (entry 125) | 187 parked rows in the 73 journals of 2026-11-12 to 11-14; the queue cites 21 parked ids. Nothing lists the rest: `brief.py` and `queueledger.py` read the rows only to check a cite. `docs/PARKED.md` held 1,832 rows when it was archived. **Measured in the second session:** of the 174 rows that open with an id (`4212a`) under `## Parked` in the journals of 2026-11-10 to 11-14 (63 journals), 128 are named by another file: 97 by a specification, 53 by another journal, 40 by the code, 28 by the queue, 24 by a pass record. Six are struck where they stand, answered. **46 are named by nothing but their own journal**, and those are the rows no session will meet again unless it greps for them. | It ended a second, uncapped queue no session could read. | Is a parked row meant to be read again? Three in four already are, from the document or the code that carries the gap, so the rule works. The missing half is small: a pass-time listing of the rows nothing else names (a tool, not a file; 46 in five days) is what would close it, if he wants it closed. The journals' "for the Loop" lines are the same question (S27): 22 with content in three days, four taken up by a pass. |
| D9 | `CLAUDE.md`, the phases: "3. **Sim skeleton** ... **Done**: every captured game replays in lockstep for its full length ... entry 29 names the three counters that define the sim complete" | Entry 132 §10 (Ramon, 2026-11-13) made parity "zero mismatched frames on the twelve state checksums over a declared set of games", "the rules' finish line from the day step H prints its first number"; §7 calls the draw stream the words stand on "blind to state". Step H prints (item 4214) and the queue reads "0 of 1 games". TRACKS carries the new definition; `CLAUDE.md`, which every session and subagent reads first, says done on the old one and names neither parity nor step H. | Phase 3's claim is true as written: the lockstep floors hold, and 132 §10 keeps them as the regression gate. He "left the day-one rules as they are" and gave his word for one sentence only. | Does the phase list say what "done" was counted in, and that parity by the checksum is the open count (0 of 1)? One clause in phase 3, his word. This lane would add it: a session that reads "Done" plans as though the rules were finished. |

## Summary

| # | file, section | bytes now | action | saved | needs |
|---|---|---|---|---|---|
| R0 | `tools/brief/frame.md`, "opening"; `brief.py` line 458 | 18,475 read by a lane | say what a lane reads of the queue | up to 18,475 a lane session | the pass (its file); D7 |
| R1 | QUEUE, "Where things stand": ten lines whose every row is whole | 5,738 | a closed form, as `Golden:` has | ~5,300 | `rondata::diff::floors` and `::audit` |
| R2 | QUEUE, handoff prose: rules and pinned numbers restated | 2,056 of 6,967 | strike | ~2,050 | the driver |
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
| R13 | `tools/brief/frame.md`, "booked", "lanes", "landing": a word's clauses in every live kind's brief | 3,280 | `booked` to the kinds with a booking; split "landing" | ~1,200 a lane brief | the pass (its file); S26, S27 |
| S1 | DECISIONS, "Index" | 15,985 of 16,000 | split at entry 100; retitle; reorder; fix 121's row | 0 (frees ~8,000) | none |
| S2–S30 | twenty-nine syncs, Part 2 | | | | |
| G1 | the eight sections pinned in `OVER`, and 45 of the 77 at 15,000 B or more | 284,070; 831,211 | promote their numbered `###` parts to `##` (run dry: 28 of 29 guards green, the red is the `OVER` rows) | 0 (largest read 70,859 → 16,448; 45 sections off the ceiling) | delete seven `OVER` rows, add one |
| G2 | `docs/JOURNAL.md`, the chronicle to 2026-09-17 | 1,185,526 | leave, or archive at a revision | 0 or 1.19 MB | `queueledger.py`; Ramon (`CLAUDE.md`'s table) |
| G3 | `docs/DECISIONS.md` | 843,219 | an index by topic; no entry moved | 0 | the pass |
| G4–G7 | `AI.md`, `GOLDEN.md`, `COMBAT.md`, `ORDERS.md`, `RUNS.md`, struck text, the audit README | 8.2 MB | leave; number the unnumbered sections | 0 | |
| G8, G11 | `ORACLE.md` and `TEST_PERFORMANCE.md`: each sends its reader to the first design, not the standing one | 179,422; 68,519 | a head that says what stands; S22's strikes | 0 | the pass; S10 (Ramon's word) |
| G13 | `docs/LEADERS.md`: 80 of the tree's 81 file:line cites into `crates/`; eight held-out bullets (S29) | 270,550 | cite by name; a pin that may only fall | 0 | `docs_guard`; the chain's builders |
| G9 | `docs/audit/`: the gate's live timings under a dated name | 528,058 in three files | an undated path | 0 | `release_gate.py`'s `TIMINGS`; `suite_timings.py` |

Read-first total: about 25,400 B of the 101,371 in QUEUE, TRACKS, LOOP,
METHOD and `CLAUDE.md`, and R0 on top for lanes.

## The three to take first

1. **The syncs S9, S24, S26, S27, S2, S19, S5, S6 and S4.** They cost
   minutes and each is a place where a fresh session is told something
   false today: an opener that reaps a row long gone and cuts builders on
   landed steps; three render counts the code has moved past; a brief
   that stops every lane at 300 k where the decision measured 400, and
   promises to file a line a decision said is filed nowhere; a start
   track "158 of 161" beside a queue that reads 161; a stop of 95 % one
   line above a stop of 99 %; a rule that stops mid-sentence; one lane in
   one paragraph and three in another. The appendix has the exact text
   for seven of them.
2. **R1 with S1: take the two rewritten sections off the ceiling.** One
   branch in the parser, two lines in the audit, and the handoff stands
   near 10,700 B, not 15,983; one
   heading and the index has 8,000 B of room. Both end a fitting loop
   (D1) that has run on most commits for weeks, and R1 is self-correcting:
   the closed form is accepted only while every row is at its length.
   With them, **G1**: it is one character a heading, it was run dry and
   28 of 29 guards stayed green, and it takes 45 more sections off the
   ceiling before their next amendment has to fit.
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

By a crude count about a fifth of LOOP is the case behind a rule: 5,061 B
in parentheses that cite a Loop item, a parked id, an entry or a date, and
about 3,062 B in sentences that tell one (METHOD 5 %, `CLAUDE.md` under 1
%). R5 to R8 are where it sits.

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
  lane request's mean context). With R1 and R2 alone, about 7,400 B.
- **What cites it:** `tools/explore/test_brief.py` holds the frame's
  sections to the checklist in `docs/audit/README.md`; the opening line has
  no row number.
- **Confidence:** high on the measurement; the choice is a judgment about
  what a lane should know, and is the pass's.

### R13. The frame's common sections speak to a residue lane

- **Where:** `tools/brief/frame.md`, "booked" (474 B), "lanes" (342 B) and
  "landing" (2,464 B). `brief.py`'s `KINDS` gives "lanes" and "landing"
  to every kind and "booked" to all but `sweep` and `coverage`. The four
  kinds booked today (`structure`, `render`, `start`, `input`) each carry
  the three and one section of their own (`input` the capture rows too).
- **Read in the second session:** "booked" is whole a residue item's
  rule ("The frame and the draw delta are the booking ... Name each unit
  a booking names by `TypeIndex` and domain before reading its chain").
  "lanes" ends "Your lane gate carries every lobby's `first_parting`",
  which the render section, forty lines on, calls moot. In "landing", the
  first bullet (the word's value diff, "its block in its widening, its
  window in the coverage driver's slice"), the trace bullet
  (`runwiring.py`, a `Pair`'s word, `WANT_CH*`) and "read your RUNS row;
  say if you moved a word" are a word's landing; S26 and S27 are in the
  same section.
- **Action:** `brief.py` gives "booked" to the kinds that have a booking
  in that sense (`residue`, `chapter`, `reading`), and "landing" is split
  into what every lane does (the journal, commit then gate, the gate
  line, the update, the done line) and what a word's lane does.
- **Saved:** about 1,200 B a lane brief, by reading. Small; the reason to
  do it is that the part a lane is told last is the part it is likeliest
  to act on, and a third of it is for another kind of item.
- **Guard:** `tools/explore/test_brief.py` holds each checklist row to a
  frame section; a row that moves sections moves there too.
- **Confidence:** medium. The pass writes this file and knows which
  clauses a render lane has needed.

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
  and `every_closing_line_row_has_its_close`).
  `crates/rondata/src/diff/audit.rs` reads the same lines through
  `ai_line_parts`: `every_whole_scoreboard_word_is_audited` takes the
  whole words *from the handoff's lines* (`audit_gaps`) and treats any
  line the parser reads as a scoreboard line, so it must take them from
  `AI_WORDS` under the closed form. `testkit.rs`'s `NEWEST_PAIR = "Late
  ages"` filters `AI_WORDS`, not the queue's text, and is untouched;
  `bantu_word.rs` and four melee modules cite the line in comments.
  Nothing under `tools/` parses these ten.
- **Guard:** those tests, and `docs_guard::the_handoff_is_short`. Made to
  fail first by lowering one row's word under the closed form.
- **Rule touched:** the queue's "The floors and these lines move together
  ... read literally" bullet gains the closed form. No decision forbids it.
- **Confidence:** high on the saving and the precedent. The code change is
  two readers and one filter, each of which already holds `AI_WORDS`.
- **The change, read off the two functions in the second session** (a
  description, not a patch). `floors.rs`'s `ai_line_verdict(line, prefix,
  rows)` parses the line with `ai_line_closes` and compares the parts
  with the pinned rows. One branch ahead of the parse: a body that reads
  `every word whole · <N>` (and `, 0 off` on a closing line) is accepted
  when `rows` holds N rows, each with word equal to length and, where it
  carries a close, a close of 0; any other row state is the error the
  per-row form gets today, with "write the rows out" added. In
  `audit.rs`, `whole_words` already takes every closed word from the
  pinned rows as well as from the line (Loop 3998), so the audit loses
  nothing; it only has to skip the parse on a closed-form line, where
  `ai_line_parts` would otherwise panic on "unreadable part".
  `an_ai_line_is_read_against_its_rows` gains the closed form's cases: N
  wrong, one row a frame short, a close of 1. **A third place, which the
  first session missed:** `every_whole_scoreboard_word_is_audited` finds
  a scoreboard line nobody listed by whether `ai_line_parts` reads it
  ("a new line cannot slip past the audit"). A new line written in the
  closed form would not parse and would slip past, so that filter takes
  the closed form's recogniser beside the parser.

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
- **What repeats a pinned line fifteen lines below it** (947 B, measured
  span by span in the second session; the first session's "about 1,500"
  was by reading and too high): "24,000 of 24,000", three times, 160 B
  (`Recording:`); "8 of 12 lines" and "Eight of twelve lines are the
  original's ...", 118 B, and "107 of 109 packets ... 314 of 1,053 lines
  agree", 109 B (`Checksum:`); "The territory 733 · 332 · 574", 45 B
  (`Rubric:`); "The blind list: open 0 (never 336 · residue 336)", 82 B
  (`Blind:`); "The rules track: whole, 130 closed", 55 B (`Golden:`);
  "`Start: 161 of 161 lobbies`", 56 B (`Start:`); "K 59 of M 60 ... frames
  35 of 35, pieces 1,465 of 1,478", 92 B (`Renderer:`); the HUD's and the
  lobby's sentences, 230 B (`Hud:`, `Lobby:`). **Not a restatement, and
  the first session called it one:** the `INPUT_*` counts are on no
  pinned line. They are 6 of the prose's 25 backticked constants that
  carry a count (799 B in all: `RUN1559B_3279_GROUND` 1,097 of 1,109 and
  the like), each pinned by a test in `crates/` and by no line of the
  handoff. **Nothing checks a count there against its constant, and three
  have drifted (S24):** each was written by the booking that
  set the pin and not rewritten by the later booking that moved it. The
  handoff is the only place a reader sees them together; it is also the
  one place they can be wrong.
- **Action:** the prose says what no pinned line can: what moved and what
  did not, the held-out leaves and their parked ids, the open readings,
  who is out. A rule is cited by its file, not restated. A code pin is
  named without its count (the `Renderer:` line carries the score, K of
  M), or it keeps its count and a test reads it: `floors.rs` already
  parses the handoff by prefix, and "a backticked constant followed by
  `N of M` equals the constant" is the same kind of reader. The first
  costs nothing and saves the 799 B; the second keeps the page a person
  can read the renderer's state from.
- **Saved:** about 2,050 B (1,109 of rules, 947 of pinned numbers).
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
- **Why it drifts:** the pass does maintain it. Eleven commits touched
  the page from 2026-11-08 to 11-13, six of them a pass's, and each
  added its state to a cell and left the states before it.
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
| 336 | "the gate: before a commit" | | S17: line 304 says the other order. |
| 108 | phase 4: "**Open**, with the start, checksum, orders, lobby and save tracks beside it" | | the queue reads "The start track: closed", `Lobby: ... 161 of 161` and step 4 closed; the list is three tracks long now. |
| 140 | the table's row for `docs/JOURNAL.md`: "the passes' entries since" | | G2: four entries against 61 pass records since 2026-10-01. |

### R12. The memory index

Outside git, and a session's own to keep; proposed, not edited. 7,324 B in
32 lines, inherited by every session and subagent. Three hooks run past
300 B (598, 422, 329), where the index's own rule is one line a memory.
One hook points at a section that is gone: "Subagent model split —
graduated to git 2026-08-25: CLAUDE.md "Fan-out rules"": `CLAUDE.md` has no
such heading; the fan-out rules are the last paragraph of LOOP's
"Ratification, and the passes" (entry 83 moved them). Saved: about 2,500 B
if every hook is held to 150.

Behind the index are 32 files, 113,145 B, read on a pointer. The one a
driver opens most is `ccc-orchestrated-loop.md` (16,438 B): its
frontmatter description reads "One driver since 2026-11-09 (the steer
row; the commander row is retired, DECISIONS 116)", its first paragraph
says the same, and its last five paragraphs reverse it (two rows since
2026-11-12, a handover by `tools/handover.py` since 11-13); it names
"`CLAUDE.md`'s fan-out rules" three times. The index's hook for it is
current, so a session is sent to a file whose opening contradicts the
hook that sent it. A memory is a session's own to rewrite: the machine
facts about `ccc` verbs, newest first, and the history out.

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
| | | save and replay: "the writer is the milestone's step 3" | QUEUE: "**Step 3 stands** (4058, run1560; RECGAME.md §8): our order-free file played for 24,000 of 24,000, every recording on disk round-trips" | R4 |
| | TRACKS, "The milestone", last paragraph (found in the second session) | "So this is reached before the start block is taken across the other 160 lobbies, before M is counted on a second lobby, and before the census filler" | all three happened first: `Start: 161 of 161 lobbies`; `Renderer: ... over 110 lobbies`; the census filler's items 4039 and 4170 (`RUBRIC.md` §0); and the milestone's step 5 stands open at 8 of 12 lines. The sentence was the plan of 2026-11-09 and reads as the state. | say it in the past: what the order was, and that breadth was taken as filler while the chain ran (entry 116's own allowance) |
| | TRACKS, "The order of play" | "the renderer beside it on one lane, taking textures and team colours next; then the lobby screen and the client's input" | QUEUE: three lanes; the lobby (4184), the HUD (4197) and five `INPUT_*` rows landed | the order that stands today, or its date in the sentence, not only in the bracket |
| S3 | QUEUE "How to maintain this file"; METHOD "Booking an item" | "Neither headline slot is ever empty"; "the headline slot is never empty" | QUEUE "The queue": "A residue or chapter item books only when another track's frame needs one"; TRACKS, rules: "no item stands" | strike, on D5 |
| S4 | QUEUE "How to maintain this file" | "The chain — merge, book, spawn, gate, push, reap — is `CLAUDE.md`'s" | `CLAUDE.md`: "The loop's machinery is `docs/LOOP.md`" | the pointer |
| S5 | LOOP, "The chain", its last line | "The turn does not end with a lane empty and the queue not, and it never asks whether to" | the sentence stops there. `git show 75ffba6d1:docs/LOOP.md` has its end: "continue — the queue's opener is the answer." Lost in `15aae746e` (2026-11-09). | restore seven words |
| S6 | LOOP, "The lanes", first paragraph | "one ccc lane on the render track and the driver's builders on the chain" | LOOP's own roster: "three ccc lanes ... four workers with the chain's builder, confirmed at the pass by load (DECISIONS 129 §1)"; QUEUE "four stand" | strike the count; the roster holds it |
| S7 | LOOP, "The chain" | "the whole gate ... (DECISIONS 132 §5; every third landing whose merge widened, before it)", with entries 72 and 79's history | entry 132 §5: "This replaces ... entry 79's every third landing" | strike the parenthesis (405 B) |
| S8 | METHOD, "The definition of done" | "the lane gate runs those, the fast tests and the guards"; "the commander's booking gate takes no flag" | entry 132 §6: a lane's gate "leaves every timed test to the booking gate but its filters and the paperwork guards, and runs whatever the timings file has never timed"; §5: the booking gate takes `--scope`. `CLAUDE.md` carries the new form. | amend in place |
| S9 | QUEUE, the dated line and the opener | "handing back to `commander`"; "cut a builder on **VII-b1** and spawn three lanes, 4208 ... 4212 ... 4214 ... and a fifth on 4216" | all landed: `docs/journal/2026-11-13-item-4208.md`, `-4212`, `-4214`, `-4216` exist, and the queue's item 4159 says VII-b1 to VII-d landed. 21 commits edited the queue since the pass's gate (`git log 8a0b2a0e7..HEAD -- docs/QUEUE.md`); none rewrote these lines. A row that died now would hand a fresh session an opener that reaps a row long gone and cuts builders on landed steps. | the booking commit keeps the opener true, or the mid-tranche opener is "continue the tranche" with the lanes that are out |
| S10 | `CLAUDE.md` line 408; METHOD line 112 | the behavioural check is "(`docs/ORACLE.md`, last section)" | ORACLE's last section is "The click-free lanes are a pool (2026-10-07 ...)". The recipe is "Running a check: the recipe in one place", the fourth section. | name the section (in `CLAUDE.md`, Ramon's word) |
| S11 | LOOP, "The chain" | the chain ends "`ccc rm <ref>`, and `git push origin --delete <the lane's branch>`" | LOOP, "The driver": "a remote branch delete is refused from every row"; and the chain itself, eight lines on: "the classifier may refuse that delete ... a refusal is reported at the drain with the count, never retried" (Loop 3848). So the step is one its own text expects to fail. Entry 129 §9 ruled it: "The merged branches on origin are Ramon's to delete, or his to allow: 65 stood, and the classifier refuses a session the delete"; he ran the line during that pass. Two days on, 36 of 41 remote branches are merged into `origin/main` again (`git branch -r --merged origin/main`, 2026-11-14 02:15 −05:00). | the chain's last step cites entry 129 §9 and drops the delete; the pass's record carries the line for Ramon, as it did |
| S12 | LOOP, "The lanes" | "The handoff's `For the pass` counts are the tables'" | the handoff has no "For the pass" part (`grep -c 'For the pass' docs/QUEUE.md` is 0). **Settled in the second session:** `git log -S'For the pass' -- docs/QUEUE.md` shows the part written when a tranche is banked for a pass (`05e9edc97`, `d64f135cc`, `ef794025c`) and removed by the pass's own commit (`f35f57351`, `471755ade`). The clause is true of a banked handoff and silent on the rest. | "a handoff banked for a pass carries a `For the pass` part, and its counts are the tables'" |
| S13 | twelve doc-to-doc section cites | a section that does not exist: `AI` §4.4 (CITIES line 531) and §9.3 (ORACLE 1841); `ANIM` §4.4 (ORDERS 2059); `ARMY` §15.8 (COMBAT 2687, four times) and §16.3 (INPUT 454); `COMBAT` §14.1 (MOVEMENT 107); `NATIONS` §5 (DECISIONS 6619); `ORDERS` §714 (INPUT 203, a line number); `PATHFINDER` §18.6 (COMBAT 12899) and §4.6 (TRANSPORT 17, twice); `TECH` §13 (AI 14501) and §7 (ANIM 2214) | `a_section_the_code_cites_is_one_the_document_holds` reads `crates/` only; the same parser over `docs/*.md` finds these. Six of the twelve are sections its `NO_SUCH_SECTION` already pins for the code. | mend each; or extend the guard to the documents with these pinned |
| S14 | four paths | `crates/rondata/src/mapped.rs` (DATALAYER line 131); `tools/gamelog/handdriver.sh` (CENSUS 619; RUNS twice); `tools/ghidra/decomp` (VISION 9, RECGAME 12, COMBAT 4444) | none exists in the tree (`tools/ghidra` holds `analyze.sh`, `export.sh`, `run.sh`, `scripts`); the export is outside the repo by rule | name the file that holds it now; say "the export" without a repo path |
| S15 | `docs_guard.rs`, `OVER` | four pins: `ORDERS.md` §4 37,583; `AI.md` §2 70,933, §15 26,923, §99 51,530 | the guard's own stderr asks for each to be lowered: "shrank to 37575 ... 70859 ... 26922 ... 51527; lower its pin" (86 B of slack; a passing test's stderr is captured, so nobody sees it) | lower the four; or make the slack a failure, as the queue's pins are |
| S16 | DECISIONS, "Index": statuses | 116 "amended by 121"; 100 "standing"; 59 "extended by 60" | entry 126 §1: "Entry 116's single row and entry 122's roster line are superseded here"; entry 103: "amends entry 100 §3's "next cell""; entry 61: "amends ... entry 59's clause 1". Three read; a mechanical pass (a later entry's sentence that names an earlier one beside amend, replace, retire, withdraw or supersede) lists 7 rows the index calls standing (57, 64, 71, 72, 99, 100, 121) and 16 whose list of successors is short; 16 entries carry an "Amended <date>" note while their row says something other than "amended in place" (4, 9, 24, 29, 34, 55, 63, 64, 68, 71, 72, 77, 78, 87, 88, 121). Two more read in the second session: entry 71 carries "Amended 2026-10-15, Ramon's word mid-tranche" and entry 72 "Amended by the forty-fifth pass (entry 84, Loop 3344)", and the index calls both standing. Candidates, not verdicts: the heuristic is noisy, and a successor does not always name what it changes (entry 126 returns the two rows without a sentence that says "amends entry 116" in those words). | the pass that splits the index (S1) checks these rows; G3's index by topic is the durable fix |
| S17 | `CLAUDE.md` lines 304 and 336 (and METHOD line 244) | "it commits before it gates"; "The full `--release` diff suite remains the gate: before a commit and after a document rewrite" | the two orders stand thirty lines apart. LOOP ("A worker commits before it gates") and entry 43's title ("a landing is committed before it is gated") hold the first. | "before a landing" for "before a commit" (Ramon's word) |
| S18 | QUEUE, "The queue", first line | "The shapes are DECISIONS 90–94 and 100 §3" | written 2026-11-07. Entries 90 to 94 score the fifty-first to fifty-fifth tranches and shape a late-age pair; 100 §3 a territory row. Six entries, 49,820 B, behind a pointer no open item uses: the five items are a chain step, a structure item, a render item and two rules items, each of which names its own document. | keep the four shapes the line goes on to name; drop the six numbers or say which kind they are for |
| S19 | QUEUE, "Where things stand", the dated line | "The week 36 % at 19:25Z ... the stop 95 %" | the bullet under it: "a soft stop at 99 % of the window, Ramon's to move (DECISIONS 121)"; entry 121: "Amended 2026-11-13, evening: the stop is 99 %"; LOOP: "99 % of the weekly window since 2026-11-13". The dated line is the pass's reading at 15:15 and was true then; the amendment rewrote the bullet and not the line. Still so on `origin/worktree-replan-pdb` at `78b2b9140`. | a dated reading says its hour ("the stop then 95 %"), or the line drops the stop and leaves it to the bullet |
| S20 | `docs/journal/README.md`, first rule | "The first line is the heading the single file used: `## <date> — item <N>: <title> (<model>)`" | of the 132 item journals of 2026-11-10 to 11-14, 118 open `# Item <N>: <title>` with the model on the italic line under it; 13 use the README's form. `queueledger.py`'s `item N` regex is case-insensitive since item 279 and reads both; all 132 name a model in their first six lines. | the README states the form in use |
| S21 | `docs/journal/README.md`, last rule; `CLAUDE.md`'s table row for `docs/JOURNAL.md` | "A steering pass writes to `docs/JOURNAL.md`, not here ... the chronicle is its record" | G2: four entries in `docs/JOURNAL.md` since 2026-10-01 against 61 pass records under `docs/audit/`; the last entry (2026-11-13) is the first since 2026-10-24. LOOP names "the pass's own record under `docs/audit/`" and never asks a pass for a journal entry. | "a pass's story is its record under `docs/audit/`; `docs/JOURNAL.md` takes an entry when the pass chooses" (the `CLAUDE.md` row is Ramon's word) |
| S22 | `docs/TEST_PERFORMANCE.md`, "Shared isolated runner" (the thirteenth of 29 sections; `CLAUDE.md` sends a reader to this file for the host budget's "design and its limits") | "Every cooperating worktree shares `~/Library/Caches/attrition/resources-v1`: sixteen CPU slots and 32 GiB reserved RSS"; "guards reserve eight slots/32 GiB"; "Each nonignored rondata library test runs in a fresh process" | `tools/resource_budget.py`: `CPU = 20`, `MEMORY_MIB = 98304`, `resources-v3`; `tools/guard.sh`: `--cpu 2 --memory-mib 4096`; the same file's last section ("Trial terms") says so, 700 lines on; and "The group unit" (item 4216): "The isolated runner's unit was one process a test ... A group is the tests of one game". None of the three earlier sentences is struck. | strike the three in place and point at the section that stands; G11 for the order |
| S23 | `docs/TEST_PERFORMANCE.md`, eleven sections of 2026-10-15 to 10-17 | "Independent review remains owed", eleven times (lines 65, 107, 192, 298, 667, 707, 890, 912, 933, 949, 965) | no item, decision or pass record names the review: `grep -i 'independent review'` over QUEUE, DECISIONS, LOOP, TRACKS and the pass records of 2026-10-15 to 10-29 is empty. Entries 75 and 76 put the branch on trial and kept it on the tranche's waiting and admission figures. A promise with no item is one nobody holds. | the pass rules once: the trial's verdict (entry 76) and a month of gates stand for the review, said in one line at the head; or it books the review as an item with a score |
| S24 | QUEUE, "Where things stand", the renderer's bullet | "`RUN1559B_3279_TRACED` 33 of 33, `_COLOUR` 17"; "`_GROUND_LUMA` 1,029 of 1,109"; "`RUN1559B_3279_SHADOW` 41 → 89 of 105" | `crates/render/src/silhouette.rs`: `RUN1559B_3279_COLOUR = (21, 33)`, "21 of 33 since item 4221 (from 17)". `crates/client/src/ground.rs`: `GROUND_LUMA_STILL`'s run1559b row `(1036, 1109)`, "Item 4221: run1559b 1,029 → 1,036". `crates/client/src/shadows.rs`: `RUN1559B_3279_SHADOW = [(80, 104), (79, 104)]`, "Item 4193 ... 105 → 104 scored", "Item 4212: hue 93 → 80, luma 89 → 79". All three still so on `origin/worktree-replan-pdb` at `78b2b9140`. A fourth reads two ways: "`_COLOUR_STILL` 14 of 33 (18)" where the pin is `(18, 33)`. The other 22 constants with a count agree with the code (checked one by one: the `INPUT_*` six, `_TRACED`, `_TRACED_STILL`, `_DEVICE_STATE`, `_GROUND_BASE`, `_BLEND`, `_PACKET_PATCH`, `_GROUND`, `RUN1570_DEAD`, `RUN1564_3279_GROUND`, the two `_GROUND_STILL`, `_LUMA`, `GRASS_HUE`, the two `_RING`, the two `_PLAZA`); `RUN1564_3294_COAST_PATCH` is a table of four rows whose first two sum to the handoff's 439 and was not checked further. | the three counts; then R2's action, so the next one cannot drift |
| S25 | LOOP, "The driver", the roster's first bullet | "**The pass**: the `steer` row, Fable 5.2, armed at a boundary (DECISIONS 126 ...)" | the same section, eighty lines up: "a handover is a spawn and a reap (DECISIONS 130, 131 §8)", "No prompt is typed into a pane"; entry 131 §8 withdrew the arm of the other row. The only "armed" left in the read-first files that means a handover. | "spawned at a boundary (DECISIONS 126, 131 §8)" |
| S26 | `tools/brief/frame.md`, "landing" (every lane carries it); `docs/audit/README.md`, rows 1637 and 1638 | "**at 300 k, one line to me** (1637, 1638): draft, commit, end the turn **done**" | entry 131 §6: "A lane is cleared at 400 k, not 230", measured (a clear at 230 k lost 3.49 USD a lane); LOOP: "A waiter on a lane is armed with `--deep 400`"; the pass's record: "The clear at 230 k is withdrawn for 400". The frame's line has read 300 since 2026-10-17 (`a65a2a596`) and neither move of the driver's threshold (230 at entry 116, 400 at 131) reached it. A lane that follows its brief stops and writes to the driver 100 k before the figure the driver clears at, and the line it sends wakes the driver on its whole context. | the frame says 400, or says nothing and leaves the trigger to the waiter's `DEEP` exit; the checklist rows with it (`test_brief.py` holds the frame to them) |
| S27 | `tools/brief/frame.md`, "landing" (every lane carries it) | "a "for the Loop" line: `none`, or a finding I file"; "**A red on the journal-quote or Loop-line guard alone is forgiven** (3277, 3328)" | entry 125 §3: "There is no Loop backlog. A journal's "for the Loop" line stays in the journal and is filed nowhere"; §4: `docs_guard.rs` loses "the Loop line's filing", and "Entry 84's forgiveness of the Loop-line guard on a lane goes with the guard". No test or tool names a Loop-line guard now (`grep -i` over `docs_guard.rs` and `release_gate.py`). The line is still written: 68 of the 73 journals since 2026-11-12 carry it, 22 with something to say (items 4159T, 4162, 4165, 4167, 4168, 4170, 4176, 4177, 4185, 4186, 4188, 4189, 4190, 4198, 4199, 4205, 4206, 4207, 4214, 4222, 4223, 4225). Two pass records took up four of them (4167, 4176, 4177, 4185); nothing lists the other eighteen. | "or a finding for the pass", and drop the guard's name; the eighteen are D8's question again |
| S28 | LOOP, "Ratification, and the passes" and "The brief" | "An unattended pass ... ratifies the marked rows, rules the Loop items, builds the loop's tools and guards"; "a worker's "for the Loop" line that names a brief's omission is a candidate row"; and three Loop numbers LOOP still carries as owed (4051 and 4055 on the waiter, 4053 on a builder's wait) | entry 125 §3: "There is no Loop backlog ... the handoff counts nothing; what the driver would raise at a pass goes in that pass's record"; LOOP's own "The lanes": the line "is filed nowhere". With no backlog a pass has no Loop items to rule, and the three numbers are a backlog of three kept in the rules file, unruled since 2026-11-09 (R5). | "rules what the tranche's journals and its own record raise"; the three numbers ruled once by the pass, or struck with R5 |
| S29 | `docs/LEADERS.md` §13, §16, §19, §20 ("In the crate", the Checksum bullet), and §5, §12, §18 in part | "Each sits in `LEADERS_HELD_OUT` at count 2 (`crates/rondata/src/diff/checksum_lines.rs:1787, 1788, 1817, 1835, 1837, 1850, 1851`)"; "Each is a `(unheld)` entry of `LEADERS_HELD_OUT` with count 2: `cannon_time` at ...`:1771`, `handicap` at `:1793`, `multi_diff` at `:1815`"; "all twelve are `(unheld)` in `LEADERS_HELD_OUT`, count 2 each" | `LEADERS_HELD_OUT` holds 30 leaves, at lines 2027 to 2058: none of §13's seven, §16's three, §19's two or §20's three is among them; of §18's twelve, `score`, `score_combat`, `score_units` and `score_units_2` are; of §12's two, `pop_issues`. The queue: "`LEADERS_HELD_OUT` 30 ... from 140". G13 has the count of line cites. | each bullet struck or dated at the step that released its leaves, as §7's was; the line numbers dropped |
| S30 | `docs/RENDERER.md` §3.12 (line 1912), §7.1 (line 3593) and §3.22 (line 3038) | "The coral layer (draws 130–131, `coralblend.tga`), the surf (163–169) and the frame's mean luminance are not drawn"; "The shore's blend, `sand.tga`'s own layer and its patches, is not drawn (§3.12, 4178b)" | the same section, thirty lines on, struck and answered: "~~... and the coral are not drawn.~~ ... the coral is drawn"; §3.16 "The shore's sand (item 4192)"; §3.18 "The coast's patches, and what lies under the lake (item 4200)", `RUN1564_3294_COAST_PATCH` 439 of 439; §3.21 the frame's white (item 4212). Each claim stood in two places and was struck in one. A scan of the nine unstruck "is not drawn" sentences in the file found these two; the surf (parked 4225a) and the other six read current or were not checked. A second scan, for a parked id struck in its own journal and still cited open in a document, finds one more of the same shape: §3.22, "the fall is in one of the two draws' stage colour (parked 4217a)", sixteen lines above "~~... (parked 4217a).~~ Answered by". Of the six rows struck in their journals (4199b, 4200c, 4212a, 4214d, 4214e, 4217a) it is the only one a document still cites open. | strike the three with their answers; the habit that prevents the next is a grep of the document for the parked id (`4178b`) when the item that answers it lands |

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
September, 856 of October, 336 of November so far); `docs/audit/` 8.0 MB in
319 files, 226 entries at its top level (G9); `docs/lab/` 2.2 MB.

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

**Three more, empty, from the second session.** "The read-first files
repeat each other": by sentences whose six-word shingles are 60 % or more
in the other file, only METHOD and `CLAUDE.md` share anything (28 of
METHOD's sentences, 4,724 B, are `CLAUDE.md`'s or its long forms: R9,
which counted the 2,139 B that are word for word). No other pair among
`CLAUDE.md`, QUEUE, TRACKS, LOOP, METHOD and the frame shares 300 B; the
frame shares 1,427 B with the audit README's checklist, which it is the
digest of. So a lane that inherits `CLAUDE.md` and is handed the frame
reads nothing twice. "The journals have grown a boilerplate": the 132
journals since 2026-11-10 have a median of 9,151 B (5,928 for the 204
before them in November, 5,362 in October); lines quoting the gate are
1.5 % of their bytes and the `## Parked` part 5.3 % (0.4 % before entry
125). The growth is prose, one landing's story each. "The pass records
repeat each other": G9, median 0.07.

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
  over 16,000 is one of `GROUPS.md` §6's, §6.4, at 16,448 by the guard's
  own count in the dry run (the first session's port read 16,594).
  `DATALAYER.md` §2's seven parts are not numbered.
- **Action:** promote each numbered `### N.M` to `## N.M`. It is the split
  the guard's comment asks for ("a `## ` heading is the split, and it costs
  nothing a reader needs") and `docs/RENDERER.md` already numbers its
  sections this way (`## 3.6`, `## 3.12`). No number changes:
  `sections_of` reads a number at any heading level, so every code
  citation resolves as before.
- **Saved:** no bytes. The largest section a pointer can send a session to
  falls from 70,859 B to 16,448, and seven rows leave `OVER` (the guard
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
- **Guard:** `a_section_over_the_ceiling_may_not_grow`. Three tools read
  `## ` headings and none blocks this: `tools/runwiring.py` matches the
  run heads of RUNS alone; `tools/seams.py` reads the queue's "The queue"
  and RUNS's run heads; `tools/retire.py` lists top-level sections, so a
  promoted part becomes a row of its own, cited by its own number.
  `tools/brief.py --section` passes its string through.
- **Run dry in the second session, and it holds.** In this lane's own
  tree, uncommitted and restored afterwards (`git status` clean): every
  section at 15,000 live bytes or more outside QUEUE, DECISIONS, RUNS and
  the journal whose `### ` parts are all numbered had those parts
  promoted. 46 sections, 271 headings, 16 files (`AI.md` 72, `COMBAT.md`
  56, `ORDERS.md` 44, `LEADERS.md` 24, `CITIES.md` 14, and eleven more),
  one character a heading. Then the reflex's steps, each run to its end:
  - `docs_guard`: **28 of 29 pass**, the section-citation guard among
    them, so every `§N.M` the code cites still resolves. The one red is
    `a_section_over_the_ceiling_may_not_grow`, and it says what the
    commit owes: delete seven rows of `OVER` (`CITIES.md` §3, `ORDERS.md`
    §4 and §5, `GROUPS.md` §6, `AI.md` §2, §15 and §99, each now 20 to
    2,229 B of head), and "`docs/GROUPS.md` `## 6.4 The slot table`:
    16448 bytes, bound 16000", which takes a row of its own or a split.
    `DATALAYER.md` §2 keeps its row; its parts carry no number.
  - `test_brief`, `test_queueledger`, `shape.py --check`, the writers'
    and ledger's guards, `queueledger.py`, `nations_census.py --check`:
    green, unchanged.
  - `tools/retire.py --no-ledgers`: 185 of 1,875 sections unnamed, where
    it read 185 of 1,604; each promoted part is a row and every one is
    named.
  - Nothing under `crates/` or `tools/` matches a `### ` heading
    (`grep`), outside the explore tests' fixtures.
  After it the largest section in those sixteen files is `GROUPS.md`
  §6.4's 16,448 B, and no other is over 15,995. **Not run:** the release
  suite; no test in it was found that reads a heading's level, and the
  reflex's guards are the ones that read these files.
- **Confidence:** high, on the dry run.

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
  index's own rows and titles, then checked in the second session against
  each entry's own sentences of succession ("amends", "replaces", "is
  superseded", "is withdrawn"; not a whole read): the booking gate, 72 →
  79 → 131 §4 → 132 §5 (132: "This replaces ... entry 79's every third
  landing"); a lane's own gate, 53 → 72 → 80 → 132 §6; the roster and the
  count, 59 → 61 → 94 §1 → 96 → 114 → 116 → 126 → 128 §4 → 129 §1 → 131
  §5; the rows and the handover, 61 → 116 → 122 → 126 → 130 → 131 §8
  (131: "Entry 130 §3's interim rule ... is withdrawn"); the stop, 118 →
  121 → 124 (98 %) → 126 §2 (95 % after the reset) → 121 again, amended in
  place (99 %); the parked file, 39 → 67 → 125; the finish line, 24 → 26 →
  29 → 41 → 53 → 114 → 116 → 132 §10. **Two corrections from the check:**
  the first session put entry 96 in the lane's-gate chain, and its clause
  is the lane *count* ("The lane rule holds two lanes", on entry 94 §1);
  and the stop's chain passed over entry 126 §2. Both are what a chain
  read off titles gets wrong, and the reason the page is the pass's to
  write from the entries. About twenty such lines, each ending on the entry that
  stands, is the page Ramon can read to find what to review. It can live
  in the index's second half; the rows by number can then be bare.
  **One constraint, read off the guard in the second session:**
  `every_decision_has_an_index_row` treats every `- ` line under a
  heading that starts with `Index` as a row, and panics on one that does
  not open with an entry number and one of its five statuses. So the
  page by topic sits under a heading of another name ("What stands, by
  topic"), or its lines are not `- ` items. The same reading confirms S1:
  the rows are gathered across every heading that starts with `Index`,
  so the split needs no change to the guard.
- **A first draft of that page, for the rules of the loop as they stand**
  (entries 128 to 132 read whole in the second session, 126 in the
  first; the older links are the chains above). Each line ends on what
  stands:
  - the rows: 116 → 122 → 126 §1 (`commander` drives, `steer` passes);
    no row rests on its context, 130 §1; a handover is a spawn and a
    reap, 131 §8 (130 §3 withdrawn; 128 §2's one tree stays).
  - the workers: four, 128 §4 → 129 §1 → 131 §5; one lane at most on a
    whole item, 129 §1; a fifth is Ramon's (TRACKS).
  - the booking gate: every landing merged when it starts, 131 §4;
    scoped, whole at a drain and at a pass, 132 §5; the merge pins, 131
    §4 and 132 §4.
  - a lane's gate: its brief's filters, the guards and the untimed, 132
    §6; no lane gates whole but a `structure` lane (128 §4's "a start
    item gates whole" superseded there).
  - the stop: 121, amended in place, 99 %.
  - the clear: a lane at 400 k, 131 §6 (116's 230 k superseded).
  - the disk: the gate clears it, 128 §1, keeping a generation in use,
    129 §2; a capture never leaves this way, 84 (iv).
  - a timed test: two sessions, 128 §8; it waits for the host, 129 §3.
  - a held leaf's release: 126 §3 → 128 §3 (on transfer) → 129 §5 (on
    the packets) → 131 §1 (only where it moves, or frozen by
    construction) → 132 §1 (never on a staged decision).
  - a tree's corpus: its own ledger's, 132 §3.
  - parity: zero mismatched frames on the twelve state checksums over a
    declared set, 132 §10; the hook, 132 §7.
  - the suite's form: one walk a game, 132 §11; no verdict cached, 132
    §8.
  - a parked row: its item's journal, 125.
  - the interface: a plain page and a plain HUD, 128 §6 and 129 §7;
    porting the original's windows stays cut.
  - merged branches on origin: Ramon's to delete, 129 §9.
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

**The tree already has the instrument for this, and this lane found it
late.** `tools/retire.py` (Loop 3486, the fifty-first pass) lists every
top-level section no other tracked file names, by heading or by number;
none of `CLAUDE.md`, LOOP, METHOD or the audit README names the tool. On
this tree: `retire: 0 of 814 scripts and 185 of 1604 sections unnamed`
with the ledgers left out, 688,165 B. Its unnamed mass is not in the
numbered giants (`AI.md` 12 sections, `GOLDEN.md` 7) but in the documents
whose sections carry no number: `ORACLE.md` 33 sections (170,948 B),
`PRODUCTION.md` 15, `FORMATS.md` 15, `TEST_PERFORMANCE.md` 24, `CENSUS.md`
7, `ECONOMY.md` 7, `TECH.md` 11. It over-counts where a cite shortens the
heading, as its header says: `CLAUDE.md` cites "Off CrossOver" and the
heading runs on. So an unnumbered section is one nothing can check a
pointer to (S10 is that failure); numbering them is the fix, not removing
them.

### G8. `docs/ORACLE.md`: the recipe section is the first path of three

179,422 B in 35 sections, in the order things were found; its preamble
says so ("written as the method was discovered, each correcting the one
before, and they are kept that way") and calls the file "the runbook
only" since 2026-09-18. `CLAUDE.md` points at it five times. The first
session judged it by title. The second read the preamble, the recipe
section and the heads of the lane sections.

- **The designated reading order is of 2026-08-20.** "Running a check:
  the recipe in one place" (the fourth section, 4,352 B) calls itself "the
  reading order for someone about to *run* a check". Its five steps are
  the hand-driven path: the lobby by `-config` and then "Solo Game → Quick
  Battle → Start", cheats typed into the chat box by `cheat.sh` and read
  back, orders by `rclick.sh`, keys by `osascript`, the quit through the
  in-game menu. Every script it names still exists (`setlog.py`,
  `cheat.sh`, `rclick.sh`, `lastframe.py`, `objs.py`, `one.py`,
  `track.py`, `aim.py`), so nothing in it is false.
- **What it does not name is how a capture is taken now.** Each has a
  later section and none is in the reading order: a scenario from a file
  (`rontrace.cmd`, "The cheat channel", 2026-08-24); the stanza and
  `longtrace.sh` ("The capture lane", 08-31); free Wine and
  `winelaunch.sh` ("Off CrossOver", 09-04); the click-free lane
  (`unattended_capture.py`, 09-18); the pool, where "no caller names a
  lane" (10-07, the last section). A briefed lane is told by the frame's
  `capture` section: `viadriver.sh golden_capture.sh`, `waitrun.sh`,
  `filerun.py`. A session with no brief is told by `CLAUDE.md`'s Tooling
  bullet, which describes the first path ("play a minute, quit through
  the in-game menu") and points at "last section", which is the third.
- **By title, six sections are the story of a fault since closed** ("The
  fourth permission, and the probe that did not test it", "The window
  nobody can see — CrossOver's expired bottle", "226: the fault is the
  bop, not the handler", "The falsifier for 226, costed", "Coverage is
  back", "`restart` from the channel wedges the game": 32,085 B). Not
  read; a runbook can keep a fault's story as the reason for a trap.
- **Proposal:** the recipe section becomes today's reading order, about a
  kilobyte, nothing moved and nothing struck below it: which of the three
  paths a question takes (a lobby's capture on the pool; a staged
  scenario by the channel's file; the hand-driven path for what needs a
  mouse), each a pointer to its section, with the 2026-08-20 steps kept
  under the third. Then S10's pointer names that section, and the bullet
  in `CLAUDE.md` says which path it means (Ramon's word).
- **Confidence:** high that the reading order predates the lanes; the
  three-path split is this lane's reading of the headings and the
  frame, and the pass that knows the lanes writes it.

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
Two hygiene probes in the second session, both empty: no `## ` section of
`docs/*.md` or `CLAUDE.md` holds an unpaired `~~`, and no struck span
crosses a blank line (where a renderer would print the markers and show
the text as live).

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

### G9. `docs/audit/` as a set: leave the records; one file is not a record

- **Measured (second session):** 319 files, 8,042,196 B. At the top level
  176 `.md` (4,302,646 B), 27 `.txt` (1,151,294), 19 `.json` (446,513), a
  patch and `rubric.tsv`; two directories, the orders' first readings (8
  files) and item 4152's readings and verdicts for `LEADERS.md` (87 files,
  1,805,660 B). By kind among the `.md`: 81 pass records (1,786,588 B,
  median 21,548), 40 territory readings of 2026-11-06 (1,237,653), 19
  audits of 2026-08-25 (403,433), 12 item records, 24 others.
- **The pass records do not repeat each other.** Six-word shingles of each
  against the one before it: median 0.07, largest 0.185 (the
  fifty-seventh). Their headings are a form (`Rulings` in 78 of 81, `What
  was measured` 59, `For Ramon` 58, `The roster, lane by lane` 46), and a
  form is not a copy. Nothing to cut.
- **One file there is live, under a date.** `docs/audit/2026-10-15-suite-timings.txt`
  is `release_gate.py`'s `TIMINGS` (line 281): the table a lane's gate
  reads to decide which tests it leaves to the booking gate. 32 commits
  have rewritten it, the last on 2026-11-13, and `-suite-memory.txt`
  beside it 28 times. `CLAUDE.md` calls it "the timings file" and names no
  path. Every other file in the directory is a record of its date; this
  one is an input the gate reads, and its name says it is a month old.
  **Proposal:** an undated path for the pair and the `-suite-cpu.txt`
  beside them (528,058 B), one constant in `release_gate.py`, one line of
  `suite_timings.py`'s header and `TEST_PERFORMANCE.md` line 500. The
  twelve pass records that cite the dated name cite what was true of
  their day and stay.
- **Snapshots nothing names.** 26 data files (588,203 B) are named by no
  tracked file outside `docs/journal/`: among them the suite's timings of
  2026-10-16 (three files, 183,318 B), three of the four `astra-reader`
  suite dumps of 2026-10-15 (236,968 B the four), fourteen `astra-*`
  notes of about 1 KB, and small JSON. The timings of 2026-10-06, 10-07
  and 10-13 (162,072 B) are each named once or twice, by the pass that
  took them. All are evidence a dated
  document rests on (`docs/journal/2026-10-15-astra-reader.md`,
  `TEST_PERFORMANCE.md`'s experiments). Leave; they cost a `grep -r
  docs/` and nothing else.
- **Confidence:** high on the measurement. The rename is small and the
  pass's.

### G10. `docs/lab/` and `docs/journal/README.md`

`docs/lab/` is 90 files, 2.2 MB, of which three observation dumps are 1.5
MB (`2026-09-19-jev-*.json` and two `.json.gz`), read by
`tools/explore/jev_lab_analyze.py` and `jev_evidence_analyze.py`. The
guards do not scan it and `CLAUDE.md` says so (entry 38). Not a compaction
target. `docs/journal/README.md` (1,673 B) carries two rules practice has
left: S20 and S21.

### G11. `docs/TEST_PERFORMANCE.md`: the standing form is the fourteenth section

- **Measured (second session):** 68,519 B in 29 sections, in the order
  things were tried. Thirteen sections of 2026-10-15 to 10-17 come first
  (the scoped replay, the coverage recording, the allocator, the parsed
  windows, a rejected lookup, the process isolation *experiment*); "The
  group unit" (item 4216, 13,048 B: what the suite's unit is today), "The
  rondata library's length" and "The release step's lanes" stand at lines
  301 to 628; twelve more of October follow, and "Trial terms", which
  holds the budget's live numbers, is last. 56 commits have touched it,
  13 of them since 2026-11-07.
- **Who is sent there:** `CLAUDE.md` and METHOD, for the host budget's
  "design and its limits". That reader meets the first design (S22:
  sixteen slots, `resources-v1`) 700 lines before the one that stands
  (twenty slots, `resources-v3`).
- **Action:** no cut. Three moves, none of which rewrites a sentence: a
  five-line head that says what stands and where ("the unit is the group,
  §The group unit; the budget is twenty slots and 96 GiB, §Trial terms;
  the sections between are the measurements each rests on"); S22's three
  strikes; S23's ruling. Or the order: the three standing sections first,
  the October experiments after, as G8 proposes for `ORACLE.md`.
  `tools/retire.py` calls 24 of its sections unnamed because nothing
  cites one by heading, so nothing breaks on a move.
- **Confidence:** high on the order and on S22; the choice between a head
  and a reorder is the pass's.

### G13. `docs/LEADERS.md`: the one document that cites the code by line

The chain's document (270,550 B; item 4159 amends it at every step). Not
read whole; two mechanical probes in the second session.

- **Line cites.** `docs/*.md` hold 81 unstruck cites of the form
  `crates/<path>.rs:<line>`, and 80 are in `LEADERS.md` (one in
  `GOLDEN.md`); with the short forms beside them (`:949`, `:105–108`) it
  carries about 295 line numbers. They are item 4152's readers' of
  2026-11-11. Every other specification cites the code by name and the
  original by `name@address`, both of which a guard checks.
- **Three days on:** of the 43 cites written as a name beside its line
  (`` `Sim::leader_long` (`…:949`) ``), **17 still have the name within
  three lines of the line cited and 26 do not** (a crude test: a cite of
  a call site and not a definition can fail it honestly). All 30 cites
  into `checksum_lines.rs` point at lines 1755 to 1851, where
  `LEADERS_HELD_OUT` stood on 2026-11-11; the list is at lines 2027 to
  2058 today, after 37 commits to that file. No cite names a missing
  file or a line past its file's end, so nothing fails loudly.
- **Present-tense status** (S29): eight "Checksum" bullets say each leaf
  of their section "is `(unheld)` in `LEADERS_HELD_OUT`, count 2". The
  list held 140 leaves when they were written and holds 30. Four are
  wholly stale (§13's seven build counters, §16's `cannon_time`,
  `handicap` and `multi_diff`, §19's two and §20's three: none of the
  fifteen is in the list), and at least three partly (§12: `misery` has
  left and `pop_issues` has not; §18: "all twelve" where four `score*`
  leaves remain; §5: `attrition_stamp2` remains). §7 was amended
  by the steps that released its leaves ("off `LEADERS_HELD_OUT` since
  VII-b1 (47 → 44)"), and §3 says "the map as it stood before"; the
  others say neither.
- **Action:** the bullets cite the constant and the leaf by name and say
  whose reading the status is ("as item 4152 read it, 2026-11-11"), or a
  step that releases a leaf strikes its section's bullet as §7's did
  (the definition of done already asks it: "Any open question this
  closes in another document is struck through there"). The line
  numbers go at the next amendment of each bullet, never in a sweep. A
  guard is cheap and in the tree's manner: `docs_guard` counts
  `crates/…rs:<line>` cites per document against a pin that may only
  fall, as `UNBUILT` does for constants, so the form does not spread.
- **Saved:** nothing in bytes. A builder reading §13 or §16 before the
  residue step is told seven and three leaves are held that left days
  ago.
- **Confidence:** high on the counts; the crude test's 26 is a ceiling
  on the cites that moved and the 30 into the list are certain.

### G12. The track documents against their queue lines: they agree

`docs/HUD.md` and `docs/LOBBY.md` state the numbers their queue lines
carry (`Hud:` 2 of 2, 628 of 628, 57,420 of 57,420, 3 of 3, 840 of 840;
`Lobby:` 161 of 161 twice, 600 frames), each beside the constant that
pins it. `docs/CENSUS.md` and `docs/RUBRIC.md` are ledgers of readings by
item, each dated or headed by its item, and the newest is the queue's
less the landings since (`RUBRIC.md` §0's last reading is item 4170's 731
· 332 · 576; the queue's 733 · 332 · 574 is 4186's). One undated line:
`RUBRIC.md` §0 opens its readings "On this tree: **ported 529 · replaced
by design 318 · open 1305 of 3611**", the first reading of three, with
nothing to say which tree. A date or an item number on that line is the
whole fix.

A mechanical check was tried on these and on `RENDERER.md`,
`CHECKSUM.md`, `LEADERS.md`, `CLIENT_INPUT.md`, `START.md` and
`RECGAME.md`: each document's last statement of a backticked constant's
count against the constant in `crates/`. It is too noisy to keep as it
stands (a specification states a pin's history, and "the last mention"
is often a "from"); on the queue's handoff, where every count is meant
to be current, it found S24.

## Left to do

The first session stopped at 02:10 −05:00 on its context and left a list.
The second took that list from 02:12 on a fresh context. What it reached
is in the text where it belongs, each marked "second session": the
half-checks R2, S12 and D6; `docs/audit/` as a set (G9), `docs/lab/` and
the journal's README (G10); `TEST_PERFORMANCE.md` (G11) and `ORACLE.md`'s
reading order (G8); the track documents against their queue lines (G12);
entries 128 to 132 read whole, LOOP and the frame read against them (S25
to S28, R13) and a first draft of G3's page; every constant the handoff
quotes against the code (S24); the parked rows' afterlife (D8); R1's
change read off its three readers; G1 run dry against the guards;
`LEADERS.md`'s line cites and `RENDERER.md`'s twice-stated claims (G13,
S29, S30). What follows is what neither session reached.

**Not reached.**

- The mid-size specifications read as documents: `MAPGEN.md`,
  `COLLISION.md`, `PATHFINDER.md` and `CHECKSUM.md` were measured by
  section and citation and never read; `LEADERS.md` and `RENDERER.md`
  were probed mechanically (G13, S29, S30), which finds a claim of a
  known shape and nothing else.
- `tools/brief/frame.md`'s sections for kinds no item is booked under
  (`sweep`, `coverage-item`, `reading-item`, `chapter`, `capture`,
  `reading`, `before-reading`: about 17,500 B). Selected by kind, so no
  lane carries them today.
- `ORACLE.md`'s other thirty sections, past the preamble, the recipe and
  the lane sections' heads.
- The release suite on G1's promoted tree: the dry run was the reflex's
  guards.
- DECISIONS before entry 125, read whole: entry 35 alone.

**Half-checked.**

- S16: six statuses read (59, 71, 72, 100, 116, 121), the other rows are
  a heuristic's list.
- G3's seven topic chains are checked against each entry's sentences of
  succession, not against the entries read whole; the draft page's lines
  for entries 128 to 132 are.
- R11's candidates are measured by line range; none was weighed against
  what each kind of lane uses.
- S24: `RUN1564_3294_COAST_PATCH` was not checked past its first two
  rows.
- S29: §4's bullet (four leaves) was not checked leaf by leaf.
- G13: the 26 cites that fail the three-line test were not each read;
  the 30 into the held-out list were.
- S30: of nine unstruck "is not drawn" sentences in `RENDERER.md`, three
  were read against the later sections.

**For Ramon, unanswered when this closed:** D5 (whether the milestone
retired the never-empty headline slots), D7 (what a briefed lane should
read of the handoff), D3 (whether he wants more of DECISIONS than an
index by topic), and, new in the second session, D9 (whether
`CLAUDE.md`'s phase 3 says what "done" was counted in).

## Not established

- Bytes per token for this prose; every token figure scales with it.
- Whether a lane that reads less of the handoff parks worse (R0, D7).
- `lore`'s index was not refreshed by this lane; the session counts are as
  of its last run, and a tool-lane search returns snippets, so "at least
  61" is a floor.
- The statuses in the DECISIONS index were read for entries 59, 71, 72,
  100, 116 and 121; the rest of S16 is a heuristic's list.
- The topic chains under G3 rest on the index and on each entry's
  sentences of succession; no chain's entries were read whole.
- Which of the nine `RENDERER.md` merges with a `--cc` hunk were
  conflicts: the journals name six updates on that file and say one
  merged cleanly (D6); the two counts were not joined merge by merge.

## Appendix. Ready text for the cheap syncs

The exact strings, so the pass that takes them spends minutes. Each "now"
is copied from the tree at `0430a0321` and checked to occur once; each
"then" is this lane's wording and the pass's to change. Nothing here has
been applied.

- **S5**, `docs/LOOP.md`, the last line of "The chain". Now: "The turn
  does not end with a lane empty and the queue not, and it never asks
  whether to". Then, appended: "continue — the queue's opener is the
  answer." (the words `75ffba6d1` had).
- **S4**, `docs/QUEUE.md` line 102. Now: "The chain — merge, book, spawn,
  gate, push, reap — is `CLAUDE.md`'s." Then: "... is `docs/LOOP.md`'s."
- **S6**, `docs/LOOP.md`, "The lanes". Now: "(DECISIONS 114 and 116): one
  ccc lane on the render track and the driver's builders on the chain."
  Then: "(DECISIONS 114, 116 and 129 §1): the roster above holds the
  count."
- **S7**, `docs/LOOP.md`, "The chain". Now: "(DECISIONS 132 §5; every
  third landing whose merge widened, before it) (the thirty-fifth steer,
  DECISIONS 72: a scoped gate runs the timed tests only when a merge
  since the last whole one can move a pin; the forty-first pass,
  DECISIONS 79: a sweep or document landing in third position drew the
  whole gate by position, twice in one tranche)". Then: "(DECISIONS 132
  §5)". The two cases are in entries 72 and 79, which 132 §5 names.
- **S12**, `docs/LOOP.md`, "The lanes". Now: "**The handoff's `For the
  pass` counts are the tables'**". Then: "**A handoff banked for a pass
  carries a `For the pass` part, and its counts are the tables'**".
- **S19**, `docs/QUEUE.md`, the dated line. Now: "the stop 95 %". Then:
  "the stop then 95 %".
- **S24**, `docs/QUEUE.md`, the renderer's bullet. Now "`_COLOUR` 17";
  then "`_COLOUR` 21" (`RUN1559B_3279_COLOUR`, item 4221). Now
  "`_GROUND_LUMA` 1,029 of 1,109"; the painter's pin of that name is
  1,075 and the still's (`GROUND_LUMA_STILL`, which 4212 and 4221 moved)
  is 1,036, so the driver names the one it means. Now
  "`RUN1559B_3279_SHADOW` 41 → 89 of 105"; the pin is hue 80, luma 79 of
  104 since item 4212. Now "`_COLOUR_STILL` 14 of 33 (18)"; then
  "`_COLOUR_STILL` 18 of 33".
- **S26** and **S27**, `tools/brief/frame.md`, "landing". Now: "a "for
  the Loop" line: `none`, or a finding I file; **at 300 k, one line to
  me** (1637, 1638)". Then: "a "for the Loop" line: `none`, or a finding
  for the pass; **at 400 k, one line to me** (1637, 1638; DECISIONS 131
  §6)". Now: "**A red on the journal-quote or Loop-line guard alone is
  forgiven** (3277, 3328)". Then: "**A red on the journal-quote guard
  alone is forgiven** (3277)". Rows 1637, 1638 and 3328 of the checklist
  in `docs/audit/README.md` move with them, or `test_brief.py` is red.
- **S22**, `docs/TEST_PERFORMANCE.md`, "Shared isolated runner": strike
  "`resources-v1`: sixteen CPU slots and 32 GiB reserved RSS" and "guards
  reserve eight slots/32 GiB", each followed by "(twenty slots and 96
  GiB now, a guard 2 slots and 4 GiB: Trial terms, below)";
  after "Each nonignored rondata library test runs in a fresh process",
  "(a group's tests share one since item 4216: The group unit, below)".

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
- **Unnamed sections:** `python3 tools/retire.py --no-ledgers`, the tree's
  own tool; about a minute.
- **Merges:** for each merge of `git log --merges --since=2026-11-09 --
  <file>`, whether `git show --cc <merge> -- <file>` prints a hunk.
- **Lanes:** `lore search '<file>' --lane tool --well attrition --since
  2026-11-10 -n 3000 --format jsonl`, grouped by worktree; `lore usage --by
  session --well worktrees-att- --since 2026-11-10 --limit 500`.
