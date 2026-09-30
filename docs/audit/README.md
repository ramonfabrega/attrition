# Audits: the second reading

Each mechanic document in `docs/` was written by one reader from the
decompiled original. The files here are the record of a **blind second
reading**: an independent reader re-derives the mechanic from the same
decompile export without seeing the document or the implementation, and a
third reader adjudicates every disagreement back to the decompiled function
and records a verdict — A (the document) right, B (the second reading) right,
both wrong, or genuinely ambiguous — with the function and the expression that
settles it.

They are reading notes in the same sense as the documents: short expressions
are quoted where a verdict turns on one, nothing is transcribed, and nothing
here is a source to implement from. What they are for:

- **A list of corrections the document and implementation owe.** Each file ends
  with "must change". Until a correction lands, the document is the one that is
  wrong, and it should say so in its own "second reading" note.
- **A record of what is now doubly confirmed**, so a later reader knows which
  claims two people reached independently.
- **Closed open questions**, where the second reader settled something the
  first had listed as not established.

## The brief checklist

What a commander's brief carries, one line each, and the landing that
paid for it (the fifteenth Fable pass, 2026-09-25, from the tranche's
"for the Loop" lines; `CLAUDE.md`, "Fan-out rules"). A row is added by
a pass, never by a worker; a worker's Loop line names the omission.

- **The frame and the draw delta, and the previous item's hypothesis
  written as one** (DECISIONS 42) — and **an event the brief names is
  cited with the record that shows it**: 742's journal read a fall on
  17085 as "a purchase", 757's brief carried it as the cause's frame, and
  it was a building's activation, one grep of `BUILDDATA` away (778).
- **A value the booking names is grepped in the dump first**, the gate's
  terms in order, before any instrument is booked: 722 booked a packet
  for `frame_attacked`, which `LeaderData::log_data` prints every block
  (737); eight packets of one tranche were reserved and replaced by a
  grep.
- **The window is sized to the word: the six blocks before it for
  `samegame.py`, 250 blocks after it** (DECISIONS 50 §7); a brief that
  asks a contiguous overlap for a +3,000 jump has said the wrong thing
  (799). **A word that is a round carries `AMMO` in the capture line**
  (783). The stanza states the window in blocks.
- **The first parting's field list, not its unit**: 736's widening showed
  the give-up's stores agreeing on 16459, one field from the mechanism
  (758).
- **The compared pin is grepped for the word's record**: chapter
  seventeen's word was a building's first damage, and `BuildDump.damage`
  stood "parked for a widening" until 763 looked (772). **A standing
  floor row whose field has a live reader is named**: `RON_STANDING`
  printed 280 rows on run227's block and the cause was two of them (769).
- **Arithmetic reads the listing first**: five functions in a row hit the
  decompiler's dropped-register trap; 759 lost nothing because every
  reading went to the listing (766). **A spec line that states a count's
  domain against its own bound is a cheap catch**: CITIES §2.6.4 stood on
  one for a tranche (730).
- **A killer tests a claim the readings disagree on**: 776's killer for
  the planner reading was predicted false by both, and only the world diff
  split them (789). On a golden-lane word that is a plan, the trace
  proxies `calc_cost` by default: the priced steps name the refusing cell
  before any reading.
- **An issuer chapter's staging is walked before its capture**, on an
  existing capture's start with a prototype of the command's entry (794),
  or a **staging-only packet** where the class is decided at process time
  (808; 790's 59 s of lane answered every predicate before a word was
  written).
- ~~**The module is reserved when two items sit in one, and the fence
  lifts at the other lane's merge** (726); a module the brief neither
  gave nor fenced is named in the landing ("say if it should move").~~
  **Code is not fenced** (the nineteenth pass, DECISIONS 55 §5; the row
  is in that pass's section below).
- **The restore in a mutation run comes from version control, or is
  `touch`ed**: a `shutil.move` from a copy puts back the older mtime and
  cargo does not rebuild (784). **Commit, then mutate, then restore**
  (907): a checkout restore took 890's uncommitted fix with it.

The sixteenth pass (2026-09-26) added the rows below from the tranche's
twenty "for the Loop" lines and one measurement of its own.

- ~~**A brief names the fenced modules and nothing else**: a module no
  other lane holds is the worker's to touch and to name in the landing,
  and a grant is asked only for a fenced one — seven of twenty landings
  stopped for a mid-item grant and eight more said "not named in the
  brief" of a module nobody else held (the sixteenth pass).~~
- ~~**A grant, like a title, names a module and a frame, never a
  function** (856): 842's brief granted `do_strafe`'s `fire_ammo` from
  836's reading, and the round's path was the release events.~~ Both
  rows: DECISIONS 55 §5, no grant is asked or given.
- **The writers of the counter that parted are counted** (823): one
  scratch print of every group action on the word's tick, each record's
  `order_num` beside it, named 811's cause in one run. **And a field's
  writers are grepped by its offset in every spelling** (`field_0x60`,
  `(ushort*)(x+0x60)`), never by its name alone (869): `build_masks` by
  name found three readers and no writer.
- **A standing row set aside as "not on this chain" names its reader**
  (834): 813's widening listed the idle gathers' `group` among rows off
  the chain, and the gather approach's collision test reads it.
- **A `SEAM:` on the draw's call chain is grepped before any reading**
  (860): 850's word sat on one that said no capture had been measured on
  that frame. The widening prints the first parting block's standing
  rows by default now (830).
- **A first killer that is a group move names the slot's `GROUPDATA` on
  both sides of the gap** (875): slot 69's `facing` and `order_num`
  answered 865 without a capture.
- **A floor row's brief asks for a killer per reading, run as a mutation
  on the built tree** (864): chapter twenty-two's damage row was three
  causes and each reading was half of one. **A landing that introduces a
  divergence it can see builds it or names it as a regression**, never
  as a park.
- **Staging is walked off an existing dump when every predicate is a
  printed field** (821; run250 was 92 s and no packet), a packet only
  otherwise; **a chapter whose precondition is staged by combat walks
  its staging first** (828; three of four takes died on something other
  than the chapter); **a staging walk goes through the command's own
  entry** before any document attributes a difference to the crate
  (885; a scratch walk was eight blocks off and two documents said "this
  crate's clock").
- **A premise that names an arm asks which event each arm needs, and
  where a lever between two staged events buys both in one capture**
  (879).
- **A measurement the brief asks for names its instrument** (881):
  "measure the other map on every step" walked nothing until a run list
  existed, and a Great Lakes list is one line.

The seventeenth pass (2026-09-27) added the rows below from the
tranche's "for the Loop" lines and two measurements of its own.

- **The gate a brief names is the lane's**: `python3
  tools/release_gate.py <install> --test-threads 4 --lane`, to a file.
  Red only on the commander's lines, it reaches `guard` and exits 0;
  the landing quotes its `Lane verdict:` and `Gate steps:` lines.
  Twenty of twenty gates of one tranche exited 1 by rule and four hid
  a red of the worker's own (the seventeenth pass, 969).
- **A mechanism the brief calls built has a unit test that fails
  without it, and the booking says which** (918): eight of twenty
  landings met a mutation that failed nothing or a test that passed
  without the fix — a sort, a pass order, `Coll::All`, `returning`. One
  mutation at booking finds it before a capture is the only guard.
- **Rows booked under one mechanism have their writers counted first**
  (889): 25 of chapter twenty-four's 30 rows belonged to `Unit::init`,
  ten minutes of counting away.
- **A premise that names a callee's tail cites the caller's branch**
  (933), **and a `SEAM:` the listing can decide cites the listing's
  branch before it is written as an assumption** (964, 910): `come_out`
  is never reached at a hangar; `FILTER_ALL` skips the one function
  that reads the seeker; a seam named the carrier arm "the queue arm".
  Read the loop's boundaries in the listing end to end.
- **A staging booked on a function's name greps the function's writers
  and gates first** (968, 903): "resources cheated to the cap" entered
  nothing its name promised, and a falsifier's reading of `num` was
  foreclosed by `researching` on the same block. Per alternative
  reading, the brief lists the gates between it and the block.
- **A parting that is a draw count asks the trace for its seeds
  first** (922): a draw record's seed makes every roll on the frame
  readable, so a candidate one side skips is named by arithmetic, with
  no packet. **A completion's draws ask for the two consecutive
  blocks' `BUILDDATA` beside them** (926).
- **A route parting's brief cites every proxied step the disk holds**
  (914): run240's whole-game `calc_cost` proxies had been read for one
  tick since item 776, and walked whole they named 899's cause.
- **A residue whose unit is a captain reads its members' rows on the
  same block first** (958): `0/18` and `0/19` agreed where their
  captain did not, which the booked premise could not survive.
- **A row that agrees on a state another writer reached is quiet, not
  proven** (898): chapter twenty-four's pool agreed only because
  `Groups::process` had re-speeded the squad four blocks earlier.
- **An issuer chapter's brief names `blind.rs` as a pin it will move**
  (950), **and a document that cites an address only for context
  chooses between the address and the pin** (943): every `name@addr`
  joins the blind list.
- **A chapter whose table row will not fit says where the row goes and
  "move the closing paragraph into it"** (949): a pointer row alone put
  GOLDEN §14 over its ceiling. §14a is the continuation (932).
- **A capture stanza with `@` lines is run on a DLL built from the
  brief's own tree**, and the brief says so: `longtrace.sh` refuses
  otherwise (936), and the rebuild is one line.

## The brief checklist, from the eighteenth pass

The section above reached its size; the rows continue here, and
everything its opening paragraph says applies. The eighteenth pass
(2026-09-28) added these from the tranche's "for the Loop" lines and
three measurements of its own.

- **A mutation is scored against the walk, and it is checked to have
  taken** (918 amended; 1018): the booking's mutation runs against the
  chapter's or the word's own walk — `cargo test --release -p rondata
  <the pin>` — and the landing names the pin that failed. Four chapters'
  booking mutations passed the whole `sim` suite and said nothing of
  the walk that holds them; 1012's squad-head mutation scored eleven
  frames *better* than the reading. A mutation the walk does not fail
  is the finding. **`git diff --stat` is non-empty before the run**: the
  pass's own first mutation of a new guard matched nothing, changed
  nothing and passed.
- **A brief that quotes a parked row greps its nouns first** (the
  pass's own, from five journals): a function against `INDEX.tsv`
  (976's `issue_launch_flight`; a guard holds the queue's names now,
  1010), a `NEVER` sentence against `blind.rs` (1033), a rule against
  its document's struck text (1028's packet frame), a lobby path
  against `docs/ORACLE.md` (972's `-config`). A parked row is the
  previous item's wording, never a citation.
- **A capture names its lane with its cover** (1047): `cover=1` with
  the golden lane's call proxies never starts on the click-free lane,
  which `docs/RUNS.md` had recorded at run185, and the stall guard's
  relaunch sits at the menu — two hung takes, 81 minutes. An issuer
  chapter that wants both splits as 1011 did: the chapter at `cover=0`
  click-free, the blind list at `cover=1` on the queue lane. **And a
  `cover=1` batch is ranked by the `NEVER` rows each staging names**
  (1033): three of 1011's four captures entered nothing new.
- **A staged gate's brief asks for every entry to the block that holds
  the call**, not only the gates inside it (995): the listing's one
  `jge` answered what 959 asked of three captures.
- **Before any reading, every instance of the parting event on the
  disk is listed** (1058): every strike on the target, every launch of
  the piece. Two of 1040's words stood on rows already on disk, and the
  listing alone argued the wrong way twice.
- **"Nothing parts on the block" is said of the object, walked back to
  its first parted field** (1046): 1028 called 4691 quiet while
  `0/2000`'s damage had stood parted since 4661. **And a parking that
  shares the word's frames is a candidate for the word's mechanism**
  (1060): 1048's parting sat in the unit whose drift was parked as
  not-mine.
- **A floor a mechanism passes is quiet, not proven, while a value it
  reads is wrong elsewhere** (1039; 898 one level up): 1012's hand-off
  passed on an inflated city value that 1028's packet showed.
- **A reader who calls an inlined idiom "the same as" a named function
  quotes that function's listing beside it** (1065): three second
  readers called a half turn the sine fold's first step and two
  sections stood on it for a month. Six of the executable's sixty-eight
  sites carry one.
- **A listing read in slices overlaps them by a line and trims headers
  by pattern** (1032): a `sed` by line count dropped the instruction on
  the join.
- **A new pair's brief names the `GAME INFO` line that carries its
  setting** (992): a sibling's frame words are borrowed only on an equal
  block (979), and a setting the block does not print passes that gate.
- **A packet's brief names the function it enters** (1026, 685):
  `Game::do_frame` is not enterable on a packet, and the click-free
  lane takes a lobby field as `--profile KEY=N`, which every packet of
  a pair past the first needs.

## The brief checklist, from the nineteenth pass

The rows continue, and everything the first section's opening paragraph
says applies. The nineteenth pass (2026-09-29) added these from the
tranche's "for the Loop" lines and five measurements of its own. **The
rows are regrouped by the kind of brief that reads them in
`tools/brief/frame.md`** (parked 1137, built the same day), which is
what a brief is composed from; this list keeps each row's story, and
`tools/explore/test_brief.py` fails on a row here that no section there
carries.

- **Code is not fenced** (the pass's own, DECISIONS 55 §5): a brief
  reserves the run numbers and the section numbers where another lane
  is in the same document, and it says where the other lanes' words sit
  as information. A worker touches
  what its word needs, takes `ccc update` before its gate, and measures
  on the merged tree; **it never hands its word over for where the code
  sits**. Nine grants and five handovers in one tranche, and the three
  merges that refused were two appended sections and a pin row.
- **The first probe prints both sides' event on the word's frame,
  before any reading** (1076, 1085, 1088, 1097): both sides' order
  stacks (`RON_STACKS`), the arguments of the call the value parts
  through (`do_damage`'s said the arithmetic was right and the `splash`
  flag wrong), ours' rounds where the first row is a wound
  (`RON_DEBUG_AMMO`), the caller of the `kill_current_order` that ended
  a chase. **A booking's direction is a hypothesis as its mechanism
  is**: three of ten said which side had acted and had it backwards.
- **A value diff is on the word's frame, or on the frame the state
  first parted, walked back from it** (DECISIONS 55 §8): where nothing
  dumped parts on the word's own frame the landing says so and gives the
  first parted field with its frame.
- **A brief that reserves a packet for a round asks first whether the
  capture printed `AMMO`** (1079; 783 one step on): run371 had, and two
  reserved runs went unused.
- **The tests a re-pin touches run green before a mutation is scored**
  (1083): a pin behind an earlier one's failure made a killer look as if
  it had fired. A floor or widening test reports every pin that moved in
  one run (`diff::testkit::pins`), and a new one takes `Pins::hold()`.
  **"Stands from N" says whether N is the window's first block or the
  gap's first frame**: 1072's gap opened in blocks no dump prints.
- **A mutation is checked to mean what it is for**, past having taken
  (the pass's own): a line appended on frame 700 to a chapter that ends
  on 644 changed the file and tested nothing.
- **A vtable slot the export names by a folded function is read off the
  PE beside the name** (1090): `vtables.txt` called `Unit::vftable + 8`
  by a COMDAT twin and a section read it as "is it a unit"; the bytes
  are `flags & 1`.
- **A gate's own test uses a key the measured table lacks** (1126): a
  release gate nested inside `pivot::release`'s lookup was silent for
  every piece but the one the table measures, and its test used that
  one.
- **A `SEAM` that says no capture holds its arm carries `scan:` and the
  command** (1132), so the next capture can run it again; a guard pins
  the forty-nine that do not (parked 1135).
- **A fixture for a sea or air unit sets the domain in all three
  fields** — the type's `kind`, the type's `combat`, the unit's `kind`
  (1130, parked 1134): vision read one and movement another, and the
  test passed without its fix.
- **A stanza `check:` that pipes `samegame.py` into a count wraps the
  call in `(… || true)`** (1080): the runner's `pipefail` carries the
  verdict the line meant not to assert.
- **A worker's own test run is `--test-threads 2`, or under
  `tools/memcap.sh`** (1081, 1086, 1089): three read the thread-width
  guard's red as their mutation's.

## The brief checklist, from the twentieth pass

The rows continue. The twentieth pass (2026-09-29) added these from the
three-lane tranche's "for the Loop" lines and one failure of its own;
two of them name a tool the pass built, because a rule that was prose
had been broken by the landings that filed it.

- **What the tree says it left out is read by name before the original
  is** (1146, 1159, 1193): `python3 tools/seams.py --item N` prints every
  live `SEAM` and every "not modelled" paragraph of a specification that
  names the draw chain's functions. `soft_collision`'s `TRADE_ROUTE` seam
  held East Indies at 6151, ORDERS §4.4's region check at 6321, and
  `World::set_blocked_at`'s road clearing — a seam whose door item 695
  had built four days before — at 7512.
- **A standing key whose field this crate reads is named, read or not**
  (1162, 1187): the widening's `RON_FIRSTS=1` print through `python3
  tools/standing.py`. who=1's `pop_cap` held Great Sahara at 12783; its
  `pop` and `escrow`, then its wealth and `trade_val`, stood on two
  widenings' first blocks each, read as noise. The row the checklist
  already had (769, 834) asked for this in prose.
- **A row three widenings pin is a rule, not a residue** (1176): run414,
  run419 and run420 each pinned a passenger's `avg_speed` apart after a
  landing, and it was one unbuilt exit rule.
- **A first row is no cause until the same key is read on every other
  unit of the window** (1155): `0/7`'s hit on 784 was chapter
  thirty-eight's first row, and `0/6` parted the same way on 854 with no
  hit behind it.
- **An actor that is gone parts no key** (1199): `1/56`'s arrival stand
  was found by asking which figures vanish between two blocks, and a
  close that holds its number thirty frames turned every unit above the
  slot into a standing key. The units on one block and not on the next
  are listed beside the firsts.
- **A function whose export prints `unaff_` arguments is read at each
  call site's listing** (1166): `find_unit_ordered`'s distance was read
  right for the merchant and wrong for the scout for four weeks.
- **The lane says its own state** (1180): `ron_lane_state` prints
  `free`, `stale` or `held by`. A lock whose pids were dead read as a
  busy lane on eight landings, and one whose pid had been handed to
  another process refused a capture with no game running; the lock
  carries each pid's start time now. A stale lock is never removed by
  hand — one removal was refused by the permission classifier, rightly.
- **A first take that dies before frame 0 is taken again once** before
  anything is read into it (1150): run416's and run420's first takes
  faulted inside Wine within four seconds of the launch.
- **A release node is read with the `get_position` sweep on a packet**
  (1208): reached three times from one scratch file; the item that
  reaches it next commits it under `tools/recomp/`.
- **A script that stages nothing still exits `success: true`** (1170),
  and **a command to another player's units holds `be` across the pump**
  (1205): two of run422's three takes and run436's first were lost to
  the script's own lines.
- **Two sections appended at one anchor, or a count two lanes re-pinned,
  merge by hand with both kept** (1161, 1151), and the count is measured
  again on the merged tree. Six landings met a refused update and none
  was on code; the capture stanzas merge by union now, and a
  specification does not, because it is amended in place.
- **A word that reaches its trace's end owes five things** (1217): the
  `ENDPOINTS` row and its `Endpoint` part, the `AI_WORDS` row closed,
  the compared pin's window dropped with its fields re-pinned, the
  coverage driver's word block a constant, a capture of the last blocks.
- **A type or an object a booking names is a supposition** (1213): "a
  Gate across a path" booked chapter forty-two, no type in the rules
  carries a gate, and one `cmovne` of `invalid_loc` made it any armed
  walker and another player's footprint. **A chapter is booked on at
  most three arms of one staging** (the pass's own, DECISIONS 56 §5):
  the chapter booked on seven functions cost 59.74 USD and reached 852 k
  of context; the one booked on one cost 21.82.
- **A mutation's scorer is made to fail once** (the pass's own): the
  pass scored four mutations of its own tool by a pattern on `unittest`'s
  output, which is coloured, and all four "failed nothing". A mutation
  is scored by the run's exit code and the failed tests' names.

## How a second reading is run

The rules are `CLAUDE.md`'s ("Every mechanic gets a blind second reading",
"Fan-out rules") and the rationale is `docs/DECISIONS.md` entries 22 and
23; this is the operating checklist, every line of it paid for once in the
record below.

1. `grep -n <mechanic> CLAUDE.md`, and read the memory index. A reader
   inherits both. If either names a finding the readers are meant to
   re-derive, move it to `docs/QUEUE.md` or `docs/JOURNAL.md` first — the
   brief cannot undo what the system prompt already delivered.
2. Brief with entry points and the export's structural traps only — facts
   about the export, never claims about the mechanic. Where a run has
   already confirmed a claim, do not spend a reader on it: brief the
   mechanic's never-executed functions (`tools/trace/report.py … blind
   docs/`).
3. Ask each reader to disclose any repository content it was handed, and to
   name, for each claim, the capture that would falsify it — the ini
   category, the frame, the field. **Hand the readers the captures** —
   the trace logs with `tools/trace/report.py`, and the dumps under
   `Logs/` — as evidence, with the frames the mechanic's functions were
   first entered on. They are the original's output, not the document,
   so a blind reader may read them; a reader who can run its own
   falsifier catches the map fact that two readings of the code agree on
   and both get wrong (transports, B.9: a dock registers in the *sea*).
4. Readers write to `~/ghidra-projects/reading/<mechanic>-<date>/` from the
   first finding, section by section. Launch in waves of two or three and
   let each finish; a wall then costs a wave, not a run.
5. Adjudicate in the main thread, or on Opus under the marker discipline:
   append each table row as it is settled; mark anything unsettled
   `FABLE:` rather than guess. A verdict is a function and an expression,
   never a paraphrase.
6. **If the mechanic's product is arithmetic, implement it now** — before
   the ratification, or in parallel with step 4. A formula cannot be
   checked by reading it again: every citation in a row can be right and
   the result still wrong, and the pass that catches it is the one that
   writes the code and runs the diff. Predicates and call graphs do not
   need this; formulas do. `docs/DECISIONS.md` entry 23, and the group
   orders' "Fourth pass" is where it was paid for.
7. Markers and code-changing verdicts go on the **ratification ledger**;
   the next mechanic does not wait on a pass. Fable ratifies in batches
   over what is marked, in the main thread, each row re-read from its own
   citation, recorded as a numbered pass in the audit file. The batch size
   and cadence are not fixed.
8. Every finding that can become an assertion becomes one before the audit
   is closed; the widening of a dumped record is the cheapest and has
   out-produced the reading. **Grep the dump before booking a reading** —
   more than one open question has been answered by a field the original
   was already printing.

## The ratification ledger

What is owed a Fable pass, as of 2026-08-26. Ratification runs in batches
over this list rather than gating each mechanic on a pass of its own
(`CLAUDE.md`, "Fan-out rules"; `docs/DECISIONS.md` entry 22 as amended),
so this section is the thing that keeps a batch from losing a row. Update
it when a pass lands, not when one is planned.

**Ratified.** `2026-08-21-orders.md`, `2026-08-25-ai.md` and
`2026-08-25-groups.md` each carry a Fable pass recorded in the file. Every
`FABLE:` marker in the groups file is struck through with its answer.

**Two markers are still open, deliberately**, both in the orders audit and
both kept as the pointer to a check rather than as an unsettled verdict:
R2 O1, `do_move`'s attack-retarget block — nothing in `crates/sim` depends
on it, and the `objdump` recipe is in the R2 adjudication — and R4's
`find_gather_tcoords@0063bdc0`, which matters only once the harness builds
a camp itself. Neither blocks a batch; both go into one when the thing
that needs them is being built.

~~**Owed, oldest first.** Adjudicated on Opus and never ratified:~~
**Struck 2026-08-30, Fable — the list was owed against an empty set.**
None of the fifteen files below carries a `FABLE:` marker (the nine of
08-20 predate the marker discipline), so "marked rows only" over them was a
batch of nothing. The surface a pass ratifies is now claim-level: a
function `docs/` cites that no traced run has entered *and* `crates/sim`
implements — `tools/trace/report.py … blind docs/`, 101 of 617 on this
date — with markers on it. A run that enters the function retires the
claim; `docs/DECISIONS.md` entry 25; `docs/QUEUE.md` item 88 builds the
intersection. The files stay as the record of their readings.

- ~~the nine of 2026-08-20 — attrition, cities, combat, costs, economy,
  movement, production, supply, tech;~~
- ~~`2026-08-23-pathfinder.md`, `2026-08-24-anim.md`,
  `2026-08-24-commands.md`, `2026-08-24-recgame.md`;~~
- ~~`2026-08-25-transport.md`, `2026-08-25-army.md`;~~
- ~~**`2026-08-25-groups.md`'s "Fourth pass"** — the row that matters most,
  because it *overturns* three verdicts an earlier Fable pass confirmed.
  A ratifier should start there and should be told that a previous
  ratification agreed with the rows now being retracted.~~ **Off the list
  2026-08-28, Fable — by diff, not by reading.** Rows 35 and 36 are the
  slot table and `compute_dests`, which `docs/GROUPS.md` §12 (its table's
  §6.4 row, and §12.2's 36-member reproduction from run31) now backs
  against run29's and run31's `GROUPDATA`; a diff is the stronger oracle
  and needs no reader. Row 37 — Square is dead code — models nothing and
  is checked by the export alone. The earlier pass's three rows stand
  retracted.
- ~~**`docs/ARMY.md` §11's two corrections and §18's new `FABLE:` marker**
  (2026-08-26, Opus, from the listing at `6f5160`): that `is_engaged` and
  `engagement` test `get_action()` rather than the front order — which a
  run29 diff then confirmed, so it needs no ratification — and that
  `is_map_unit` gates only the loop's **break**, so an army with no
  map-unit target adopts the **last** qualifying unit's target. The
  second is the marked one: it rests on a register spill
  (`6f5324`/`6f5342`) and a fall-through (`6f536b`) and **no run has
  exercised it**. It is implemented in `Sim::army_engagement_seed`, so a
  ratifier is checking live code.~~ **Ratified 2026-08-28, Fable, from
  the listing.** The reading holds — `6f5345`/`6f5349` leave the loop
  iteration without restoring the spills, `6f536d` is the only arm that
  does, `6f5369` the only exit — with one refinement that changed a line
  of `Sim::army_engagement_seed`: a last qualifying unit whose target
  order is negative yields nothing. `docs/ARMY.md` §11, §18. The test
  that "confirmed" the old reading passed for an unrelated reason (an
  unarmed building is not `active`, so `group_action_attack` returned
  before ordering anyone); it is rewritten.
- ~~**`docs/GROUPS.md` §4.4's `find_leader` key** (2026-08-26, Opus,
  listing `0070ccb0`). Implemented and unit-tested; unobserved, because
  every group in every dump so far has one `type_cat` category.~~
  **Observed 2026-08-26 by run31** and off this list:
  `GroupOrder::oxx` names the leader in the record, and it is not
  `list[0]`. §4.4, §12.1.
- ~~**`FABLE:` — `sin_table@00a46a00`'s second-quadrant branch**~~
  **Closed 2026-08-27, Fable, from the listing.** The decompiler is right
  about `sin_table`; the mirror is in `sinx`/`cosx` and inlined at 63 of the
  65 call sites; the branch is reached only from
  `MapGrass::make_continents`. `docs/MOVEMENT.md`, "The mirror is the
  original's". No code change.

**2026-09-04, Fable — the batch was empty, and the ledger has a feeder
now.** Every `FABLE:` marker in the tree is either struck with its answer
or one of the two deliberate pointers above; the run57/run58 markers in
`docs/ORACLE.md` proposed a booking clause that `CLAUDE.md` has since
adopted ("grep the disk before booking a capture"). What produces the next
markers is the audit lane (`docs/DECISIONS.md` entry 33): docs-versus-code
readers over the documents amended this week, then blind second readings
for the nine documents that have never had one — caravan, collision,
danger, goody, input, merchant, roads, scout, vision — then item 72. Its
verdicts land here as audit files; its `FABLE:` rows are the next batch.

**2026-09-28, Fable, the eighteenth pass — two rows, the first batch
since 2026-09-04.** `FABLE:` 1038: the commander's one-clause fix to
`CLAUDE.md` — a packet is taken at the word's own frame, read as a
logger frame — is **ratified**; it is `docs/EMULATOR.md` §8's amended
sentence (item 597, parked 605) and the clause it replaced was the one
the twelfth pass struck. `FABLE:` 1065: item 1052's overturn of
`2026-08-25-army.md`'s A.36, A.62 and A.67 is **ratified from the
listing**. The `sub $0x80000000` at `6f4db1` is an operand, not a flag
test: its result is stored, masked and looked up, and the cosine's
`add $0x40000000` is taken from the same stored angle, where
`sinx@0092d100` opens `test %esi,%esi; jns`. Swept whole, 68 sites
reach `sin_table@00a46a00` and six carry a half turn before the fold:
`Army::do_forming` `6f45ec`, `Army::march_to_target` `6f4db1` and
`6f50a4`, `Army::send_here` `6f99a9` — the four the verdict covered,
and `docs/ARMY.md` §8.5, §9, §14 and §23 say "back" at all four now —
and `Guy::set_angle` `5d915e` and `Guy::set_new_location` `5d8998`,
which no audit row names (parked 1068). The audit file's rows are
struck in place. The verdict's premise, "opens every `sin_table`
call", was false of sixty-two.

**Batch 19, 2026-09-29, the nineteenth pass
(`docs/audit/2026-09-29-fable-pass-19.md`): empty.** No `FABLE:` row
stood and no commander's clause was landed in `CLAUDE.md` in the
tranche; `docs/ARMY.md` §8's step 5 still read "`FABLE:` ratify", the
marker batch 18 had ratified, and says so now.

**The cheapest way to shorten this list is not a pass.** Most of what is
owed is arithmetic and predicates a capture can settle outright, so
`docs/QUEUE.md` item 13's differential fuzzing retires more of it per hour
than a reading does — and per entry 23, anything on it whose product is a
formula wants its implementation before its ratification anyway.

## The record

The first pass (2026-08-20) covered the seven mechanics then implemented. Its
headline: arithmetic doubly confirmed almost everywhere; predicates, scopes and
wiring wrong in several places — which unit kinds are exempt, which step a
multiplier belongs to, which building a rule applies to. These are exactly the
errors that tests written from the same reading cannot catch, which is why the
second reading is now part of a mechanic's definition of done (`CLAUDE.md`,
working agreement).

How it was run: six `lean` subagents on Fable 5, one per mechanic, given the
entry points and the traps but not the documents, writing to a scratch
directory; then one adjudicator per report with both readings and the export.
About an hour of wall clock for all seven, because `tools/ghidra/export.sh`
had already turned the decompile into files.

The tech tree (`2026-08-20-tech.md`, the same day, the eighth mechanic) was
read by two blind readers at once — one over the predicates, one over
`gain_tech` and the state — because the mechanic is twice the size of the
others. Its headline was different in kind: the predicates were doubly
confirmed branch by branch, and what the first reading had missed was in the
**loaders** — a derived prerequisite (`UnitType::init` giving every combat
unit its age's Military epoch) and derived back-links that no amount of
reading the consumers reveals. Read the `init` of every type the mechanic
touches, not only the functions that ask about it.

Combat (`2026-08-20-combat.md`, the ninth mechanic, later the same day) was
read by two blind readers split by function rather than by half — the damage
pipeline and the firing path — because the mechanic is the largest yet and the
two halves share almost nothing but `do_damage`. Its headline: the formula was
doubly confirmed step by step and the **edges of the reading** were where the
first draft was wrong — a magic-number division read as `/96` instead of
`/192`, a `goto` inverted, an enum value of 1 read as a tag, a branch not read
to its end, a block read and not written down. Five corrections, four
additions, all landed the same day.

Cities and buildings (`2026-08-20-cities.md`, the tenth mechanic, later the
same day) was read by **five** readers in parallel for the first reading —
one per sub-area: placement, construction, city levels, garrisons, capture —
and then by five blind readers split the same way, all on Fable 5. Its
headline differs again: the predicates were doubly confirmed almost
everywhere, and what the second reading found was at the **edges of the first
reading's scope** — a caller nobody grepped for (`Leader::calc_wall_stats`
re-bakes every unfinished building's clock, so "frozen at placement" was
wrong), a table taken on trust (`even_circle_init` builds the city radius
mask with a rounded `sqrtf`, not the octagonal metric), a gate read with its
sense inverted (the building's own attrition is every 32 frames, 16 only
under rush rules before war), an early `return` ten lines into a long
function. Seven corrections, all landed the same day. Two disagreements were
settled neither way but in the listing or the PE: the `do_construct`
argument, `capture_strength`, and the type-vtable slot `+0xfc`. The lesson:
grep the writers of every field you call frozen, and the callers of every
function you call once-only.

Orders (`2026-08-21-orders.md`, the eleventh mechanic) was run twice, and the
record keeps both attempts.
Seven readers took the blind side, one per sub-area, split as the first
reading was — and, for the first time, on a **different model from the first
reading**: Opus 5 blind against Fable 5's document, the trial the model-split
note had left pending. That half worked: all seven finished, 355 numbered
claims, 64–83 KB apiece. The adjudication did not. Six of the seven
adjudicators, on Fable per the split, hit the account's Fable limit within
about ten minutes of each other and died; five had read both readings and
written no verdict. Only R5, the start of a game, survives — 37 verdicts,
seven corrections, all landed.

The second attempt ran the adjudication on Opus instead, in waves, and it
completed: **A 32 · B 89 · both 165 · neither 18 · open 8** across seven
sub-areas, ten corrections to `crates/sim`, all three queued disagreements
closed without a behavioural check. Its own lesson is in the file, and it is
about hedges: every place the first reading wrote "medium" or "the
decompiler's local is stale", the second found something — twice, the hedge
was pointing straight at the mechanism it had missed.

Three lessons from the failure, all cheap next time:

- **A fan-out of adjudicators is a quota commitment, not just a token
  cost.** Seven long Fable agents in flight is enough to exhaust a day's
  limit, and when the wall arrives it takes every one of them at once. Land
  them in two or three waves, and let each wave finish before the next
  starts, so a wall costs one wave and not a run.
- **A failed agent's notification carries no usage figures**, so the waste
  can only be bounded by wall clock. The instruction to readers to write
  their file *section by section* is what saved the blind side; the
  adjudicators had the same instruction and five of them still died before
  the first verdict, because a verdict costs a lot of reading before it
  costs a line of output. Tell an adjudicator to append **each table row as
  it is settled**, not each section.
- **Keep the raw reports somewhere durable from the start.** They were in a
  job scratch directory that is deleted with the job; 2.2 M subagent tokens
  of reading survived only because they were copied out. They live at
  `~/ghidra-projects/reading/<mechanic>-<date>/` now, which is where the
  next mechanic's should be written directly.

The reports, briefs and adjudications are all at
`~/ghidra-projects/reading/orders-2026-08-21/`. One process change from the
second attempt is worth keeping whatever the model: adjudicators were told to
mark anything they could not settle **`FABLE:`** rather than produce a
verdict they did not believe. Five rows across seven sub-areas carry it —
five honest gaps instead of five plausible errors.

**The pattern's third step, proven on orders (2026-08-23): a ratification
pass.** When an adjudication has run on Opus, the loop is closed by Fable
before the next mechanic builds on it: every `FABLE:` row taken back to the
binary, and every verdict that changed Rust re-verified from its own
citation — the listing re-read where the verdict cites the listing, the
decompile where it cites the decompile. On orders the pass cost about an
hour, retracted nothing, and settled four of the six flagged rows (two of
them from the PDB's own type records, the strongest evidence in the
project). That result is what makes the arrangement safe to repeat: Opus
blind readers, Opus adjudication under the marker-and-append-per-row
discipline, Fable ratification of the markers and the code-changing
verdicts, recorded as a "third pass" section in the audit file.

**The pathfinder (`2026-08-23-pathfinder.md`, the same day as the
mechanic).** Two blind readers on Opus 5, split by function — `astar_path`
and its machinery, `calc_cost` and the three wrappers — with the PE listing
allowed for garbled locals; an Opus adjudicator under the marker discipline,
32 verdicts; then the main thread, on Fable, ratified every verdict and
re-read the five behavioural ones from the decompile itself. The structure
was doubly confirmed — the LIFO tie-break, the expansion wheel, the budgets,
the arrival radius, the reconstruction roots — and the corrections were at
the edges again: which branch each cost term is reachable from, a flag both
readings had left uninterpreted (`leaders.flags & 4`), a give-up exit nobody
had transcribed, and the 24 corner-cutting probe offsets, which the
decompiler had mangled and the first reading had refused to trust, settled
byte by byte in the listing. Five behavioural corrections landed in
`path.rs` the same day, and one of them found a bug outside the mechanic:
the gamelog parser had been handing every leader its successor's flags. The
new lesson: **check the project's own established facts before leaving a
flag uninterpreted** — `is_human` had been in `docs/ORDERS.md` §8 the whole
time, and both readings re-derived around it. The working papers are at
`~/ghidra-projects/reports/pathfinder/`, one directory over from where the
later mechanics' live.

**The recorded-game container (`2026-08-24-recgame.md`).** The first audit
of a file format rather than a mechanic, so the blind reader had a second
oracle — the heavengames sample as well as the export. One Opus 5 reader,
adjudicated by the first reader on Fable the same day. The 943-byte header
map came back identical field for field, and the strongest agreement in
the record so far is the package-stream start: the first reading's landmark
and the second's backward dynamic program over all 1.68 MB — two algorithms
with no shared code — arrived at the same byte. The corrections were in the
**writers**, once more: the first reading had the recording path writing
gzip, and the mode bits in `File::open` show it writes raw and gzips only in
`finalize`, so an unfinalized recording is a raw file and the reader now
sniffs for both. Two more: a marker field settled from its use sites, and a
Types stretch the second reader walked to exactly 806 records and then
proved unparseable without the loader's own class order. The blind report
was not kept — it lived in the job's tmp, against the orders audit's third
lesson — and survives only as absorbed into the audit file and the
document.

**The command payload encoding (`2026-08-24-commands.md`, the same day).**
One Opus 5 blind reader in an isolated worktree branched from `main`, which
carried neither the document nor the queue entry — the isolation structural
rather than an instruction — 886 lines, every claim cited and graded;
adjudicated by the first reader on Fable. The 82-entry dispatch table and
every size were derived four times over, each reading from the decode side
and the encode side, and all four coincide entry for entry; the obfuscation
layer — the seed-keyed XOR, the seeded gap bytes, the gate on the network
bit — was proven from the call ordering by each reader independently.
Three of the four corrections were of one kind, **what can actually be
emitted**: the console command's wire size exceeds the package cap, so the
dispatcher handles a command nothing can send; five more commands have no
issuer anywhere in the export; and the walker the first reading had cited
to prove `group` is never serialised was the recording head, while the
save-game walker does write it. Each was settled by a sweep of every
`add_command` caller — the cities lesson with a different verb: grep the
callers of every function you call once-only, and the **emitters of every
command you call sendable**. Its report was also lost with the job's tmp;
from the AI audit on, readers write to `~/ghidra-projects/reading/` from
the first finding.

**The animation clock (`2026-08-24-anim.md`, the same day as the mechanic).**
One Opus blind reader, ~20 minutes, adjudicated by the first reader on
Fable the same hour, while the context that wrote the mechanic was still
loaded — the cheapest adjudication yet, and the argument for running the
second reading before `/clear` rather than in a later session. The
structure was doubly confirmed to the line, and the oracle matched 52/52
both ways. Three verdicts went to the second reading, two of them changing
Rust: a rescale the decompiler prints as a formula and the listing shows to
be an identity (both calls take the old slot), and a flag bit the first
reading had named a boat's crew that the `TypeIndex` enum names a scholar.
The third closed an open question from the dump's own state — a figure's
non-zero `end_time` proves its `set_anim` ran, and `set_anim` there draws —
which the first reading had deferred to a debugger run. One verdict went
against it: a field named from its offset rather than from `types.txt`
(`o_up` for `inside_up`), refuted by the PDB and by the dump in one line
each. The lesson is the cities audit's, again: the readings agree on the
arithmetic and disagree on names, and a name is settled by the type
record, never by the surrounding code.

**The AI (`2026-08-25-ai.md`, the twelfth mechanic, and the largest).** Seven
blind readers on Opus 5, one per sub-area, run in **two waves** — the orders
audit's quota lesson applied deliberately for the first time, and it cost
nothing. Adjudication ran in the **main thread** rather than as a fan-out:
one standard across seven reports, and no repeat of the wall that killed six
of seven adjudicators on orders. 590 numbered claims, 42 rows doubly
confirmed, 31 corrections, two changes to `crates/sim`.

The readers were spent **unevenly on purpose**, which is new. §2.18–§2.20
rested on a single reading (the implementation's) and the trace's coverage
report had all three functions on the never-executed list, so they went in the
first wave; the census, driver and make list had been confirmed behaviourally
by run18 and run19, so they went in the second. That triage is worth repeating
wherever a document's sections have visibly different provenance.

Its headline is a methodological one, and it is uncomfortable: **the blind
protocol leaked, and the project's own habits caused it.** A subagent inherits
the repository's `CLAUDE.md`, and ours narrates each mechanic in the working
agreement — so every reader had this mechanic's headline findings in context
before it read the brief. B1 disclosed it unprompted and precisely; the
affected rows are marked rather than counted. The fix is not to the brief,
which cannot reach the system prompt, but to where the handoff prose lives.
Until that moves, a blind reading of any mechanic `CLAUDE.md` describes is
weaker than it looks.

Three findings show what the pattern is now good for. The one verdict that
went against **both** readings — `compute_site_stats`' coastal ring is centred
on the original cell, not the slid one — was not something the second reader
found; it was something the second reader stated *confidently and wrongly in
passing*, which made it worth checking, and the check found the document and
the implementation both wrong. The one **refutation** went the other way: B7
flagged that `create_units` might feed a type index where a count belongs,
marked it medium, and handed it to whoever owned that function rather than
asserting it — and it was wrong, settled in twenty lines of listing. And
`TypeData::can_pay_cost` reducing with `max` instead of `min` was found by
**two readers independently, in different scopes**, which is the strongest
evidence this method produces.

The recurring lesson recurred three times in one audit: **grep the writers
outside the class you are reading** (B3-b, B5-f, B7-b). In B3-b it had led the
first reading to a confident negative — "nor anywhere" — that was false. The
new lesson is about **transcription**: three of the sharpest findings are
places where the decompiler's C is a faithful-looking lie (an unsigned compare
printed signed, a `% 63` aliasing, a reloaded base register), and in two of
them the implementation was right *because* it had been written from
understanding rather than transcribed. The single place it was wrong is the
place it followed the document's own coordinate bookkeeping.

Still owed, and named here so it is not lost: a guard for the coastal-ring
fix. The existing suite passes either way, so nothing covered it. *Landed
2026-08-25 with run20 (`docs/AI.md` §15.8).*

**Transports and docks (`2026-08-25-transport.md`, the same day as the
mechanic).** The first reading in the main thread on Fable from the
decompile and run21's trace; two blind readers on Opus 5 in one wave,
briefed while the mechanic's own capture (run22, a `DUMP_ALL` window at the
frame the trace gave for the first dock) was running; adjudicated on Fable
against both the same hour. 114 claims, six verdicts against the document,
two of them changing Rust — a stale `gull_o` and a `num_coasts` that answers
1 for a sea region — and the rest names and scope: `num_captains` for
`num_units`, an air unit that skips a gate rather than failing it, a caller
list that turned out to be the census alone. The day's two largest facts
came from the capture and from neither reading: a dock registers in the
*sea* region, so the `reg_docks` counter both readings described is never
incremented for it, and owner 9's gull is not in the dump at all. The
lesson is `CLAUDE.md`'s "prefer a diff to a reading" with an edge: which
side of a shore a building's centre falls on is a map fact, and a reading
cannot settle a map fact. The sequence — read, stage the capture from the
trace's frame numbers, spawn the blind readers while it runs, adjudicate
against both — cost under two hours end to end and is the shape to repeat.

**Armies (`2026-08-25-army.md`, the same day as the mechanic).** The first
reading in the main thread on Fable from 5,500 lines of decompile and the
listing; two blind readers on Opus 5 in one wave, launched while three
`DUMP_ALL` windows of a raid on the AI's capital were being captured and
briefed with them; adjudicated on Fable against both the same hour. 142
claims, eleven verdicts against the document, six of them changing Rust —
a direction, a base register, a field twice over, a predicate's shape, a
switch's polarity, and a fold read as a reversal. The day's two lessons:
the readers went to `rise.pdb` itself for the enums the export does not
carry (`ArmyStatus`, `CITY_*`, `OBJECT_*`, the leader flags) and that
settled five inferred names at once — the export should ship them; and
the largest correction was to a claim the first reading had marked
"settled in the listing", read without its base register. The blind
reader re-derives every claim, the settled ones included. The captures
did what the readings could not: two of the three windows end an army
through the one search the sim stands in for, and only the dump could
say so.


**The group orders (`2026-08-25-groups.md`, the same day as the mechanic;
applied 2026-08-26).** The first reading in the main thread on Opus 5; two
blind readers on Opus 5 in one wave; adjudicated in the main thread on
**Opus 5** under the marker discipline, appending each verdict as it was
settled and marking `FABLE:` what could not be. 124 verdict rows, nine
verdicts changing Rust, five markers. The first reading was given **low**
standing by the brief and deserved it, for a reason that has now cost two
sessions: it never opened `rise_z.map`, and it guessed at five vtable slots
the PE names outright — three of the guesses were wrong, and two sections
had built rules on them.

Applying it a day later produced three lessons of its own, and all three
are about **where the evidence already was**.

- **The seam that was never a seam.** The document declared
  `Form::compute`'s slot table unimplementable partly because "no capture
  pins its output". `GroupData::log_data` had been dumping the whole of that
  output — five arrays per member — every frame the category was on, and a
  run with it on was already on disk. `CLAUDE.md`'s rule is "before
  declaring a seam, grep `log_data` for the fields it covers", and it was
  written *because of* this mechanic; the correction is that the rule also
  applies to a seam you are only declaring provisionally.
- **Four of the five `FABLE:` markers were settled in minutes, by the checks
  the audit itself had named.** A semaphore bit that the document called the
  network flag turned out to be "the scenario editor is open"
  (`ConsoleWin::run_cmd` sets it around `ScenarioEditor::init`); a claim
  about an asymmetric restore was confirmed by twelve instructions of
  `llvm-objdump`; a flag the audit thought might be vacuous was
  `unitrules.xml`'s own "flies like a helicopter", and is not; and a role
  bit was named by the type record. **A marker is a question with a costed
  answer, and the cost is usually smaller than the estimate written beside
  it.** Settling one on Opus narrows the Fable pass rather than discharging
  it.
- **The widening found something all three readings had missed, and it was
  outside the class.** `GroupData::facing` reads 1 on live groups in the
  capture, which `compute_form` cannot produce with that semaphore bit
  clear. The third writer is `Unit::kill_current_order@005e2cb0` — a
  different subsystem, which is exactly why a brief scoped to the `Group`
  family could not reach it. And the first `GROUPDATA` assertion written
  from the audit **failed on its first run**: `priority` is 0 on an emptied
  hotkey slot, because `Group::kill`'s `num == 0 → clear(−1)` writes over
  the bit. Two for two on `CLAUDE.md`'s "the first widening failed on its
  first run".

**The group orders' third pass (2026-08-26, the same file).** The first
ratification run **as the session** rather than as a subagent — bank,
`/clear`, Fable in the main thread — and briefed by a **charter** rather
than the list of nine, which is what let it find what it found. The floor
held eight of nine; the ninth verdict's two gates were right and its
*consequence* wrong, because nobody had followed what the first loop did to
the state the second loop reads (`order_type` after `clear_orders`). Its
sharpest finding was outside the floor and inside a function no pass had
opened: `UnitTypeData::get_stance_type`, the predicate every stance
decision resolves to, with its tests in an order the sim had wrong. Two
methods worth keeping: **the PDB's `LF_ONEMETHOD` records name a vtable
slot when `rise_z.map` cannot** (a COMDAT-folded slot has one name per
address in the map and its own in the type stream — `llvm-pdbutil dump
--types`, ten seconds), which closed the last marker and confirmed nine
inferred slots at once; and **delegate the mechanical sweeps, not the
judgment** — two Opus `lean` scanners over the whole export (writers and
readers of every field, callers of every function) cost a quarter hour and
surfaced a fourth `facing` writer, with every hit re-read in the main
thread before it became a claim. The one control the widening dropped is
recorded with its reason: a capture where the leader's heading equals the
move's bearing cannot separate the two, and only the listing can.
**A third instance of the `LF_ONEMETHOD` method, on a predicate this time
(2026-08-26, no audit file — it was a mechanic's own session).** Two
vtable slots in `PathFinder::astar_path`'s prologue had been read as "the
current order's target is a transport-relevant unit" and "fetch that
target", because Ghidra's `vtables.txt` prints them as
`StrafeOrder::is_air` and `MoveOrder::get_move_order` — both COMDAT folds,
the second naming a *different* slot with the same body. The PDB's own
`UnitOrder` field list names slot `+0x14` `is_move` and slot `+0x40`
`update_move_order`, and three `llvm-objdump` lines confirm the bodies:
`mov eax,1` on the move family, `xor eax,eax` on the base,
`lea eax,[ecx-0x54]` for the down-cast. So the offset is the current
*move order's own*, and no target is involved anywhere — a predicate
error, of exactly the class the first pass's headline named, found and
fixed by the same ten-second command. `docs/PATHFINDER.md` §4.1, §3.

And its sibling lesson, which is not about vtables: **the answer to a
residue is often already written in another mechanic's document.** The
`0x18` on run20's goal was queued as a pathfinder question for a session;
`docs/GROUPS.md` §6.7 had the sentence that answers it, and a `grep` for
"slot" would have found it in a minute. Before booking a reading for a
residue, grep the documents of every mechanic the value passes through —
not only the one it was measured in.

**A plausible bug is more dangerous than an implausible one (2026-08-26,
no audit file — item 31's own session).** The rule above is "when the
decompiler prints a local that cannot be right, the listing settles it in
a minute". Building `docs/GROUPS.md` §6.7 found the other half of it.
Ghidra renders `action_move_near`'s coastline snap as

```
y = fy + (wy / 0x300 - fy / 0x300) * 0x300;
x = fx + (wx / 0x300 - fx / 0x300) * 0xc0;      // <- a tile stride
```

which *can* be right, and if it were would be exactly the kind of
asymmetric original defect this project reproduces on purpose. It is not
one: `70672d`–`706764` is `leal (%eax,%eax,2)` then `shll $0x8` — × 3 ×
256 — on both axes, with `imull $0x2aaaaaab` / `sarl $0x7` either side,
a signed divide by `0x300`. A faithful transcription would have shipped a
defect the original does not have, and no capture on disk crosses a
coastline, so no test would have caught it. **The trigger is not "this
cannot be right"; it is "this is arithmetic, and a constant is doing the
work" — check the fold.**

Two more from the same hour, both of them the decompiler's rendering
rather than its logic. `grouppath` and `cols` print as
`grouppath.length` and `cols._padding_`, which read like fields of
something; both are **function-local statics** of `action_move_near`
(`0xee1538`, `0xee155c`), and `cols` is written by no instruction in the
export — so Column's non-final waypoints index uninitialised heap, and
what looked like a mechanic to port is a seam by necessity. And
`pathfinder +0x70`, glossed in §6.7's first draft as "an AI hint the
pathfinder reads", is the *same word* `find_wpath` sets for an AI's own
units: a mode forced on from outside, not a private channel. All three
were one `llvm-objdump` or one `grep` for a writer away, which is the
point.

And from the tests rather than the listing: **a guard whose subject is
one of two short-circuits has to be placed against the other one.** The
first `a_short_group_move_plans_no_route_at_all` put its goal two cells
away and stayed green when the `0x900` gate was removed, because
`find_wpath`'s own near test produced the same one-entry stack. Breaking
it on purpose is what found that; a fixture chosen to clear both is what
fixed it.

**A byte constant inside an array index is not always a field offset
(2026-08-30, no audit file — queue item 76's own session).** The rule
above is about a fold doing arithmetic; this is the same trap one level
down. Ghidra prints `Unit::think_scout`'s surface probe as

```
*(byte *)(world->tdata + 4 + ((wy*4 + 2) * world->tile_xs + wx*4) * 2) & 0x30
```

and two lines above it, over a **different** array, the identical `+ 4` is
genuinely a field offset: `wdata` has stride `0x1c` and `WData::region` sits
at `+0x4`. `docs/SCOUT.md` §7 read the second one the way it had just read
the first, wrote "note: `4*wx`, not `4*wx + 2`", and carried the asymmetry
as a quirk of the original for four days. But `TData` is **`size 0x2` with
`mask` at `+0x0`** — there is no field at `+4` to reach — so four bytes is
two elements and the tile is the cell centre. Two things settle it in a
minute each: the type record's **size**, and the array's canonical
accessors, which carry no constant at all (`WorldData::is_cliff_at`,
`is_tocean_slow`).

So: **before reading a constant byte offset in an index as a field, check
that the element is wide enough to hold one, and compare against another
reader of the same array.** The cost here was a scout sent to the wrong
cell for four days; the headline moved 362 → 436 when it was fixed. And the
reason no test caught it is worth its own line — every capture that had
exercised the mechanic was a **frame-0** one, where no candidate cell
straddles a shoreline, so the two tiles agree. A mechanic checked only at
the opening frame is checked on its easiest input.

**A right document does not make a right implementation, and a monotone
grid hides its own errors (2026-08-30, no audit file — queue item 79's own
session).** `docs/VISION.md` §3 has said since it was written that
`Object::update_seen` projects a unit's vision disc along `unit->angle`,
and cited the instruction that proves it: `mov 0x50(%ecx), %ecx` at
`00651cf1`, `UnitData +0x50`. `vision.rs` projected along
`Movement::facing` instead — the *guy's* eased angle, `GuyData +0x18` —
because that is the field whose name reads like "which way it is facing".
Nothing in the second-reading process looks at this: a blind reader
re-derives the *document*, and the document was right.

Two lessons, and the second is the larger.

- **Audit the citation, not only the prose.** Where a document pins a
  field by offset, the check that has teeth is `grep` for that offset in
  the module that claims to implement it. Queue item 72 was booked for
  two *documents* citing one address; this is the same tool pointed at a
  document and its code, and it is cheaper than either reading.
- **A monotone accumulator is the worst place for a bug.** `seen2` only
  ever grows, so a wrong reveal is silent until some later mechanic reads
  the cell — here three hundred frames later, inside a scout's ring walk,
  which is what the item was booked as. The general form: **when a
  mechanic writes state that nothing reads for hundreds of frames, the
  diff has to be on the state, not on its consumers.** The `WORLD` dump
  had been printing all 14,400 bytes of the grid since the first capture
  and nothing compared them; ten frames of it are compared now
  (`run13_s_fog_grid_is_the_original_s_on_every_cell_of_ten_frames`), and
  it is exact — but *it does not catch this bug*, because the projection
  is wrong only while a unit turns and nothing turned far enough inside
  that window. So the widening is right and insufficient at once, which
  is the honest thing to record about it.
- **And monotone hides a missing write, not only a wrong one** (item 99,
  2026-08-31 — the same grid five days later). `vision.rs` skipped
  buildings in the hundred-frame resync with the reason written down:
  "because `seen2` is monotone — the pass cannot *remove* a bit from it".
  True, and beside the point: nothing else in the crate wrote a
  building's disc at all, so a farm finished mid-game revealed nothing
  ever. The general form: **a justification for skipping work must name
  the write that covers it, not a property of the store.** The same
  ten-frame diff was exact throughout, because no building finishes
  inside its window.

## The docs-versus-code wave, 2026-09-05 — a different reading, and what it caught

Eight documents, one Opus reader each, in two waves of four; the audit lane
adjudicated every row against the source itself. Files:
`2026-09-05-<doc>-vs-code.md` for roads, cities, orders, economy, collision,
anim, movement and merchant.

**It is not a second reading and it is not blind.** Each reader saw the
document *and* the implementation, and nothing else: for every stated rule,
formula, predicate and order of steps, find the code that implements it and
write down the places they disagree. It exists because four times in one
week a document stated a rule the code did not have and the code lagged its
own document by 70 to 312 frames until somebody noticed by hand. A blind
second reading cannot catch that — it never sees the code — and a test
written from the document cannot either, because it encodes the same
sentence.

**The numbers.** About 1,140 stated rules traced to their implementing code.
**101 rows, all 101 confirmed, none struck.** Every code citation the lane
re-checked was accurate, across eight independent readers.

**What the shape of the yield turned out to be**, which was not what the
wave was designed for:

- **A document can teach the code its error, and then nothing can find it.**
  COLLISION §4.3 is the case. `Unit::detect_unit_collision@00617060`
  short-circuits two order kinds *before* an action test, so the gated set
  is seven kinds and not four; the document names "the last four", and
  `Sim::same_group_soft` implements exactly that — plus a `GroupMove` gate
  the original never consults. Document, code and original are three
  different rules. This is the wave's argument in one row.
- **Rules that neither side carries.** Five of ANIM's nine rows are rules of
  the original that are in neither the document nor the code — visible only
  because one reader held both against the same function. A document-only
  or code-only pass shows nothing there.
- **A row is worth more as a widening than as prose.** Three of the wave's
  rows became assertions the same day. MOVEMENT's verified-line bit was
  booked as "the refusal does not clear `unit_masks & 8`"; comparing the
  bit against `Unit::line_ok` over run65's window — a field the dump had
  printed all along and nobody compared — gave **335 disagreements in 450
  unit-frames**, and not on the refusal path at all. ANIM's frozen clock
  was booked as unreachable; `GuyData::log_data` prints `last_time` beside
  `cur_time`, so the *step the original took* was already on disk, and it
  is 0 on all 26 frames with the bit and 1 on all 89,522 without. ROADS'
  reconstruction order could not be separated by any capture and was
  settled by reading instead — and the widening written to test it found a
  different finding, thirteen tiles of `World::set_behind`'s `0x4`.
- **Two documents reached the same missing call from opposite ends.**
  `Build::remove_from_city` not regenerating its city's roads is ROADS' R4
  and CITIES' R15, from two readers who never saw each other's work.
- **A stale comment at the code site is worse than a stale document.** Four
  doc-comments in `economy.rs` carry claims ECONOMY.md has retracted, one
  contradicting four pinned diff tests; MOVEMENT's SEAM says a bit "has no
  reader this crate models" when the crate has one on the hot path. Both
  are what the next session reads *while editing*.
- **Grep the dump before booking anything, again.** ANIM's row and
  MOVEMENT's both turned on a field the original had been printing since
  August. Neither needed a capture; both had been written up as needing one.

**What it cost, and the one brief error.** Eight readers, no drops, no
quota wall. One reader was briefed that a function was on the blind list
when it was not — the lane had built that list from a single trace before
building the 68-trace union and carried the stale fact forward. The reader
checked rather than believing it and said so in its report, which is the
behaviour the brief asks for and the reason the error cost nothing.


### One row's afterword, 2026-09-06 — a placement claim is not a value claim

The 2026-09-05 economy pass lists `trade_val` first into wealth` among the
claims it agreed with. It was right, and it was checking **where the field
goes**. What the field *is* — `Caravan::trade_value` over
`CityData::get_trade_value`, both of which this crate had wrong by a factor
apiece — was not a row in that reading, because the document did not make a
claim about it that a reader could disagree with.

The defect paid the AI fourteen wealth a minute on every map, from the first
trade route, and nothing failed for a fortnight: a rate in sixteenths reaches
nothing that spends a draw. It surfaced only when a third mechanic — the make
list — bought a building with the surplus (item 238, `docs/AI.md` §30).

**The lesson for the next pass's brief.** When a document says a field is
added somewhere, that is two claims, and the second one is usually the
unwritten one. Ask which of them the reading is checking, and prefer a
**diff over the field itself** to either: the assertion that closed this is a
`CITY`-record census over 610 frames and three archives, which no reading
would have produced and which now checks on every commit.
