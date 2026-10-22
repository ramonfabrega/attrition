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
~~- **A first killer that is a group move names the slot's `GROUPDATA` on
  both sides of the gap** (875): slot 69's `facing` and `order_num`
  answered 865 without a capture.~~
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
~~- **A route parting's brief cites every proxied step the disk holds**
  (914): run240's whole-game `calc_cost` proxies had been read for one
  tick since item 776, and walked whole they named 899's cause.~~
~~- **A residue whose unit is a captain reads its members' rows on the
  same block first** (958): `0/18` and `0/19` agreed where their
  captain did not, which the booked premise could not survive.~~
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
- ~~**A capture names its lane with its cover** (1047): `cover=1` with
  the golden lane's call proxies never starts on the click-free lane,
  which `docs/RUNS.md` had recorded at run185, and the stall guard's
  relaunch sits at the menu — two hung takes, 81 minutes. An issuer
  chapter that wants both splits as 1011 did: the chapter at `cover=0`
  click-free, the blind list at `cover=1` on the queue lane.~~ Retired
  by item 3429: the hang was the autostart declining `cover=1` (`INFO
  174 1` in every dead take), so the menu waited for a click; the
  driver now runs beside the stubs, and the pool takes `cover=1`. **And a
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
  `0/2000`'s damage had stood parted since 4661. ~~**And a parking that
  shares the word's frames is a candidate for the word's mechanism**
  (1060): 1048's parting sat in the unit whose drift was parked as
  not-mine.~~
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
~~- **A gate's own test uses a key the measured table lacks** (1126): a
  release gate nested inside `pivot::release`'s lookup was silent for
  every piece but the one the table measures, and its test used that
  one.~~
- **A `SEAM` that says no capture holds its arm carries `scan:` and the
  command** (1132), so the next capture can run it again; a guard pins
  the forty-nine that do not (parked 1135).
~~- **A fixture for a sea or air unit sets the domain in all three
  fields** — the type's `kind`, the type's `combat`, the unit's `kind`
  (1130, parked 1134): vision read one and movement another, and the
  test passed without its fix.~~
- **A stanza `check:` that pipes `samegame.py` into a count wraps the
  call in `(… || true)`** (1080): the runner's `pipefail` carries the
  verdict the line meant not to assert.
- ~~**A worker's own test run is `--test-threads 2`, or under
  `tools/memcap.sh`** (1081, 1086, 1089): three read the thread-width
  guard's red as their mutation's.~~


## The brief checklist, from the thirty-second pass

The rows continue; the first section's opening paragraph applies.

- ~~**`tools/memcap.sh` is written with its ceiling** (2082, 2049's Loop
  line): `memcap.sh <gib> <command…>`, `12` for one filter and `20` for
  the suite — the frame once said "under `tools/memcap.sh`" and one run
  came of the bare form, refused by the usage line.~~
## The brief checklist, from the fiftieth pass

The fiftieth tranche (DECISIONS 89): twenty landings on two Opus lanes
in 13 h 06 from the base to the drain, every one a `coverage` take on
the click-free pool; 7.40 USD and 52.0 minutes waiting a landing; one
red booking gate (3442's gate quote), three `DEEP`, no stall; `NEVER`
321 → 273, and the pass's ratification took the open list 192 → 91. The
pass added these rows, each from a lane's own "for the Loop" line.

- **A button pressed right after a `select` can hit an empty cell**
  (3450; 3443's run1058): the second take pressed on 620 while the
  card still held the group's options.
- **A menu or popup stanza names `@after`, and pairs `@key` with
  `@char`** (3452; 3445): the rename popup and the in-game menu held
  the solo game's frame.
- **A unit-type button's craft is the unit type** (3457; 3446): the
  chapter's row and the item both wrote `@spell … 661`, which queued
  nothing.
- **A site on the golden start names its `library` line** (3459;
  3448): a disallowed `@build` is dropped silently.
- **A booked field names its writer** (3466; 3456): `process_ping`
  never reaches the gamelog.
- **An army is killed by `die` on the twin's ids** (3467; 3455):
  the booked Battleships lost to the Persians' navy.
- **A taunt's effect lands at +0x3c** (3475; 3470): the first take
  ended twenty frames short.
- **A draw-side booking names the list's filler** (3476; 3472): the
  gates checked were the walker's, not `sort_all`'s.

## The brief checklist, from the forty-ninth pass

The forty-ninth tranche (DECISIONS 88): sixteen landings on two Opus
lanes in 8 h 57 from the base to the sixteenth booking, drained early
on Ramon's pit stop; 5.59 USD and 47.2 minutes waiting a landing; one
red booking gate (the drain's first, on a gate quote's name@address
cite), one `DEEP`, no stall; `NEVER` 380 → 321. The pass added these
rows, each from the tranche's own receipts.

- **A click is an `@click`/`@rclick` line on the pool, and the queue lane
  is only for what no `@` line sends** (3437; 3427's and 3436's eleven
  driven takes): the pool's `@click` retook 3427's three driven games
  draw for draw and block for block, and the queue lane's exclusion
  with the pool cost 3437 111 minutes waiting and 3436 two takes to a
  dead wait (Loop 3440).
- **Before a coverage take: the owed field's print level, the state's
  expiry frame and a same-named twin** (3430; 3417, 3419 and 3423's retakes): three
  lanes each spent a retake on one of these, and the brief's row is
  cheaper than the capture.
- **A take with an unshared input is checked with `samegame.py --from F`**
  (3435; 3427's "for the Loop"): a driven or `@click` take parts its twin
  at its first click by construction, and "0 differing" was being
  argued by hand in three journals.

## The brief checklist, from the forty-eighth pass

The forty-eighth tranche (DECISIONS 87): seven landings on four lanes in
2 h 40 — the queue's whole booking, five readings and two sweeps; 5.07
USD and 38.5 minutes waiting a landing; no red gate, no `DEEP`, no
stall. The pass added the fifth kind, `coverage`, and these rows.

- **A reading's booking counts its functions with `tools/census.py
  --pin --never --cites docs/<DOC>.md`, never from prose** (3415; 3409's
  "for the Loop"): 3409's booking named four functions its document
  does not cite, one dead in the image, and 3408 and 3412 each counted
  their set differently from the queue; the line's three numbers are
  the booking's.
- **A coverage capture is one pinned lobby at `cover=1` on the queue
  lane, checked against its `cover=0` twin and pinned into
  `blind::TRACES`** (3416): `never` moves only when a trace enters a
  function, and a reading tranche climbed 337 → 345 open because it
  cited what it read and entered nothing; the click-free pool takes
  `cover=1` as well as the queue lane (3429).

## The brief checklist, from the forty-seventh pass

The forty-seventh tranche (DECISIONS 86): twenty landings on four lanes
in 13 h 46 from the pass's last commit to the twentieth booking, no
stall; 7.67 USD and 117.2 minutes waiting a landing (11.10 / 127.2), six
battery games closed and the held-out battery whole eight of eight; no
red booking gate; seven `DEEP` answered, one `STALLED`. The pass added
this row, from the tranche's own receipt.

- **A wait is on what you started, by its pid or its own log, never a
  `pgrep -f` of a tool name every lane runs** (3406; 3403's "for the
  Loop"): a subagent's `until ! pgrep -f "mutate.py --batch"` matched a
  sibling lane's batch and the waiting shell's own command line, and
  stood 161 minutes on a batch that had finished — the commander's
  `STALLED` exit found it, no rule did.

## The brief checklist, from the forty-sixth pass

The forty-sixth tranche (DECISIONS 85): twenty landings on four lanes
in 16 h 00 to the handoff from the pass's last commit, about three and
a half of them a weekly-limit stall; 11.10 USD and 127.2 minutes waiting
a landing (14.79 / 96.4), ten battery games closed; four red booking
gates, one a test, three the sim lib's memory cap; twelve `DEEP`
answered, one `LANDED` under a clear, one blocked ending of a minute.
The pass added these rows, each from the tranche's own receipts.

- **Never delete under `~/ron-data` or `~/ron-golden`; a retried
  attempt writes a new file name** (3375's block): the classifier
  refused a lane's `rm` of its own scratch files and the lane stood on
  a person's prompt until the refusal fired — one minute, a push
  notification — on files no later step needed deleted.
- **A lane gate on a moved base refuses before its offline step** (3366;
  3360's line): the `Base moved:` line is the first after `Gate pid:`,
  nothing ran, and the remedy is `ccc update` and a second gate; the
  offline cases no longer read the real tree.
- **The paperwork guards run in every lane gate whatever their timing**
  (3373; 3362's booking gate red on the writers guard after a green lane
  gate): `writers::`, `ledger::`, `docs_guard` and the handoff's test
  are never left to the booking gate by the one-second budget.

## The brief checklist, from the forty-fifth pass

The forty-fifth tranche (DECISIONS 84): twenty landings on three lanes
in 16 h 55 from the pass's seventh commit, 14.79 USD and 96.4 minutes
waiting a landing (4.64 / 60.1), fourteen battery games closed; no red
booking gate, no kill rule, fifteen `DEEP` answered, two `LANDED` after
the line, two merges backed out at `ccc merge` and re-merged. The pass
added these rows, each from the tranche's own receipts.

- **Read the gate's `Base moved:` line before the done line** (3341;
  the commander at 3335's and 3338's merges): two lanes announced
  landings on bases four and two landings behind, both merges backed
  out on the battery tables every promotion edits, and each cost an
  update, a resolve and a second gate after "landed"; the lane gate
  reads the base's movement and the paths both sides touched, and
  refuses the done line while any stands but the commander's paperwork
  (`COMMANDER_PAPERWORK`, Loop 3401: an overlap there merges clean).
- **A red on the Loop-line guard alone is forgiven on a lane, and the
  line is the commander's to file** (3328; 3314's three lane gates,
  3318's self-filed row, 3322's `none`): the guard asks that a journal's
  "for the Loop" line be filed in the parked file, which only the
  commander does, at the merge.
- **`check.sh`'s exit is cargo's; read the command's exit, never a
  pipe's** (3327; 3318's line): the wrapper returns the child's code,
  measured on a build error this pass (exit 101).
- **The click-free lane launches through `viadriver.sh`, never from the
  session's shell** (3334; 3329's line): `launchctl managername` reads
  `Aqua` through RonDriver.app and `Background` from a session's tree,
  and the tool refuses the second before it stages.
- **A road-node parting's first step is a whole-game log of ours' map
  around the tile** (3323; 3314's line), **and a blow larger than ours
  with `get_damage` agreeing is a packet scan for live `Ammo` first**
  (3328's note; 3322's method note).

## The brief checklist, from the forty-fourth pass

The forty-fourth tranche (DECISIONS 82): twenty landings on four lanes in
7 h 25 from the pass's third commit, 4.64 USD and 60.1 minutes waiting a
landing (3.70 / 52.0); no red booking gate, no kill rule, three `DEEP`
answered, two `STALLED` on one hung mutant, one `LANDED` after the line,
no merge backed out. The pass added these rows, each from the tranche's
own receipts.

- **A lane's cleanup is `tools/reap_agents.py --yes`, by ownership,
  never a glob** (3290; att-3281's done line): a `git worktree remove
  ../agent-*` loop from one lane took the three live lanes' subagent
  trees, a tranche after 3258's sentence said "only your own"; the tool
  removes the trees merged into the lane's branch and refuses the rest.
- **A sweep batch's subagents go out in waves of three, each wave one
  message** (3311, the pass's measurement): 3261 waited 170 minutes on
  six subagents in sequence and 3295 86, the subagent class 17.4 minutes
  a landing over the tranche — the frame said one subagent a function
  and nothing about when.
- **A red on the journal-quote guard alone is forgiven on a lane** (3277;
  the commander at four merges): the guard asks for lines that exist
  only once the gate has run, so fourteen of twenty first lane gates
  were red on it and nothing else; `COMMANDERS_LINES` carries it, the
  verdict names the remedy, and the booking gate holds the quote.
- **A batch's mutant is killed after five times the un-mutated run and
  counted caught; a mutant that does not compile is no verdict** (3276,
  3300; att-3261's `STALLED` exits and 3287's line): one mutant held a
  `cargo test` 100 minutes at 100 %, and a pin that did not type-check
  exited 101 and was read as a catch.
- A `--watch` of a field is `VA+OFF:LEN`, and a bare `+N` past 0x20 is
  refused (3280; 3269's line): `unit+0x58` watched 0x58 bytes from the
  unit and missed the field, which cost a run.
- **A lane gate widens only on the AI kind** (3313; DECISIONS 83, the
  forty-fourth steer): every lane of the tranche touched a sim path and
  every lane gate ran the whole timed suite at four threads, 6 to 26
  minutes, while fifteen of twenty landings moved no pin; the sweep and
  chapter kinds' gate lines carry `--no-widen` and the booking gate holds
  the pins.

## The brief checklist, from the forty-third pass

The forty-third tranche (DECISIONS 81): twenty landings on four lanes in
6 h 02 from the pass's commit, 3.70 USD and 52.0 minutes waiting a
landing (4.20 / 33.6); one red booking gate (a renumbered row's pin),
no kill rule, no `DEEP`, one `STALLED`, no merge backed out. The pass
added these rows, each from the tranche's own receipts; the sweep kind
took the two sentences, the chapter kind stands at 12,998 of 13,000.

- **A sweep lane's subagent has its own scratch, and the lane removes
  only the worktrees its subagents named** (3258; 3239's parked (c)
  and done line): two sibling subagents overwrote one `breaks.tsv`
  under a shared tmp, and a lane's closing loop over every `agent-*`
  worktree removed five of another lane's; the frame's sweep section
  says a subdirectory a function and a cleanup by the lane's own
  answers.
- A journal quotes its lane gate's `Lane verdict:` and `Gate minutes:`
  lines, or the paperwork guard fails the booking gate (Loop 3224 and
  969, the forty-third pass's guard, from item 3265 on): the four Opus
  journals of the forty-third tranche quoted them and none of the
  sixteen Sonnet ones did, and the pass that was to read the lane gates'
  minutes by step could read four. The frame's landing row stands as
  written; the row with teeth is `docs_guard`'s.
- A `--tests` filter names a live test (Loop 3227, the commander's rule,
  built in `brief.py`): a filter whose every matching `fn` is `#[ignore]`
  is refused with the names; three briefs named `battery_measure` and a
  lane gate exited 2 on it.

## The brief checklist, from the forty-second pass

The forty-second tranche (DECISIONS 80): twenty landings on four lanes in
4 h 52 from the pass's commit, 4.20 USD and 33.6 minutes waiting a
landing (4.25 / 34.0); one red booking gate (a stale memory row), no kill
rule, two `DEEP`, one `STALLED`, no merge backed out. The pass added
these rows, each from the tranche's own receipts; the chapter kind stood
at 12,999 of its 13,000 and the pass trimmed words, never a number, to
seat them.

- **The journal's gate section quotes `Gate minutes:` beside `Lane
  verdict:` and `Gate steps:`** (3224, the pass's measurement): the lane
  gate is 21.6 of the 33.6 minutes a landing waits and ran 11–51 minutes
  on a box under its cores at eleven of fifteen gate ends, and no
  receipt said which step took them; the gate prints the minutes by
  step now, and a pass reads them off the journals.
- **An `@FILE` line is a flag with its value, and a `--port-file`
  inside one expands** (3180, 3102; 3166's and 3078's lines): the
  reader split a line into one token and never expanded a file's own
  `--port-file`; it splits as the shell would and takes the same
  expansion for the file's tokens.
- A run number is the brief's `Reserved for you` or none (Loop 3176,
  the commander's rule, built in `brief.py`): a note that names a run
  the `--runs` list does not carry is refused, and so is a run another
  brief on the box names; two notes offered run 923 "if needed" and two
  captures landed under one number.

## The brief checklist, from the forty-first pass

The forty-first tranche (DECISIONS 79): twenty landings on four lanes in
4 h 44 from the pass's commit, 4.25 USD and 34.0 minutes waiting a
landing (6.22 / 41.9 on five); no red booking gate, no kill rule, one
`DEEP`, one `STALLED`, no merge backed out. The pass added these rows,
each from the tranche's own journals; the chapter kind stood at 12,991
of its 13,000 and the pass trimmed words, never a number, to seat them.

- **A windowed re-run over a gap starts at the last dumped block when
  it is within reach** (3155, 3140's line): the booking sized 1490..1746
  to the word, the walk named 1376–1378 — 126 blocks before it — and a
  second capture was owed; from run918's 1370 the same capture would
  have covered the walk-back.
- **A word at a lobby's last frame is scored on the long's closing
  block before a capture is booked** (3086's line): the close of a
  self-ending lobby is on disk in its own long (`cover=0` prints the
  closing block whole), and the peace pair's closing state was scored
  on a fresh capture where the long would have served.
- **A shape parked names the flag the generator could not spell, after
  a grep of `sweep_gen.py`'s `_stub` and `--alias` docs** (3133,
  3112's and 3120's lines): 3078, 3104 and 3112 each parked
  `num_aircraft_here`, `get_wonder_value`, `find_dock` and `new_danger`
  as unstarted, and 3120 built all four on flags the generator had.
- **The journal before the gate, and its "for the Loop" line is `none`
  or a finding the commander files** (3089, 2971's line; the forty-first
  pass's guard `a_journal_s_loop_line_is_filed_or_says_none`): a lane
  gate refuses a tree with no journal, and five of the tranche's
  fourteen findings were not filed at the merge.

## The brief checklist, from the fortieth pass

The fortieth tranche (DECISIONS 78): twenty landings on five lanes in
4 h 49 from the pass's commit, 6.22 USD and 41.9 minutes waiting a
landing (5.20 / 23.6 before); two red booking gates, one killed, two
`STALLED`, one merge backed out, no kill rule. The pass added these
rows, each from the tranche's own journals, and struck none — the
chapter kind stands at 12,965 of its 13,000 and the rows went to the
sweep kind, which has room.

- **A generator's verdict is its exit, never a grep of its stdout**
  (3022, 3004's line): `sweep_gen.py --regen` on a bad flag set exits
  2 and leaves the old files in place, and a `grep -iE error` over
  stdout read it as a pass twice in one batch.
- **A batch of breaks runs the un-mutated command first** (3037,
  3008's line): a port that did not compile exited 101 as a red walk
  does, and fifteen "held" breaks were a type error; `mutate.py
  --batch` refuses the tree now, and the row says why.
- **A two-line break writes `\n`** (3048, 3038's and 3058's lines): a
  break whose OLD spans a newline was refused line by line and one
  batch dropped it; the batch file decodes `\n` in OLD and NEW now.

## The brief checklist, from the thirty-ninth pass

The thirty-ninth tranche (DECISIONS 77): twenty landings on four lanes
in 3 h 53 min from the pass's commit, twenty-one with the drain's in
4 h 44, 5.20 USD and 23.6 minutes waiting a landing (5.42 / 35.2
before) under host admission; no kill rule, no red gate, no merge
backed out. The pass added these rows, each from the tranche's own
journals or the commander's note, and struck seven rows the tools
now enforce or the loop has outgrown: 875, 914, 958, 1060 (the
candidates list), 1176, 1126 and 1130 (fixture lore).

- **A `claimed` row is the other sweep lane's** (2926, the commander's):
  2877 re-swept 2863's function and 2910 re-swept 2900's two, a gate
  and a half each; `census.py --unbacked --claimed <worktree>` hides a
  function a sweep test on that tree cites, committed or not, and the
  sweep brief names the other lane's worktree.
- **Breaks go in one `mutate.py --batch FILE`** (2965, 2946's Loop line):
  a shell loop over `old|new` pairs split a Rust `||` and reported four
  breaks as passed; a tab-separated line each, on the restored tree.
- **`--regen` says what it kept from the old line, `--fresh` keeps
  nothing, and the `.rs` port wins** (2886, five reaches): `--rows=2400`
  left the kept `--rows 1200` merged in after it; the kept `--port`
  came back over a hand edit; a merged `--table` grew a second copy.
- **The generated header names the `p` windows in order** (2892, three
  reaches): sparse slices are `p` windows before the places, and a bias
  meant for the first place landed on a slice twice.
- **Units are read in one pass, comma-separated** (2896, 2881's line):
  6 s a unit on a 1.2 GB dump, over 100 s for a cast of eighteen.
- **`runwiring.py --chapter N` lists a chapter's places and its staged
  count against the walk's** (2914, three reaches): six files by hand
  since chapter eighty, the count and the words array's length each
  hand-counted and wrong once; the array is a slice now.
- **The walk prints each `add`'s landing and what each issuer line
  reached** (2970, 2654): the verb's coordinates snap to land by up to
  400 units, and a staged `@attack` refused for sight printed
  `skipped {}`.
- **The make list's head is `RON_DEBUG_MAKE=<frame>`** (2955, 2931's
  line): the probe every AI item re-added by hand, env-guarded now.
- **A windowed capture is filed by `filerun.py <run> <tag>`** (2933,
  2904's line): the copy into `Logs/` under the archive's names was a
  hand step in every journal of the AI word.
- **A value question the dump cannot answer takes a packet hooked at
  the caller's return before a reading** (2903, 2864's line): two
  hooks turned "theirs re-adds it, somehow" into "theirs never killed
  it" in one run.

## The brief checklist, from the thirty-eighth pass

The thirty-eighth tranche (DECISIONS 76): twenty-three landings on
four lanes in 5 h 31 min from the pass's commit, 5.42 USD and 35.2
minutes waiting a landing (4.22 / 36.5 before; 32.0 over the first
twenty) under host admission — the waiting held within a fifth, the
admission waited 2 to 41 s a gate, and both stay. The pass added these
rows, each from the tranche's own journals or its own measure, and
struck five rows the tools now enforce: 1743 (the brief composes the
item's seams itself), and 1081, 1086, 1089 and 2082 (the host budget
admits every test run; `memcap.sh` is `check.sh`'s now).

- **Your own test run is `zsh tools/check.sh test …`** (2763, Ramon's
  hog, measured): it joins the host budget Astra's branch brought; the
  heaviest coverage test fell from 200 s and 2.70 GB alone to 38 s and
  2.42 GB on the windowed reads.
- **Index-0 sites in different callers: theirs' first** (2847, 2825's
  Loop line): the booking named `1/193`, and the cause was a draw ours
  never makes, in theirs' index-0 caller; the unit ours names at that
  index is the first one after the missing draw. **A reach gap at equal
  distance is `max_range`'s terms in `tuning.rs`**, grepped before a
  gate is read (2847).
- **`seams.py --item` on `think_<type>` of a unit the draws name**
  (2827, 2808's Loop line): the seam that held 5985 was named in prose
  four days and reached only through the Transport Freighter's type.
- **A predicate refusing on one side against the reading: `--watch` its
  fields over the caller's call on a packet first** (2867, 2845's Loop
  line): a blind reading's `[bytes]` side effect sat unbuilt a month;
  the ring walk's predicate owned it.
- **A booking that carries a cast lists its callees' line counts before
  it is sized** (2824, 2812's Loop line): the carry alone reached
  `Unit::swap_team`'s 286 unread lines and the staging went to the next
  chapter.
- **The batch is re-read on the base at every `ccc update` before a
  fan-out** (2856, 2793's Loop line): six of nine built rows were on the
  base by the lane's update. `mutate.py`'s refusal already names the
  staged paths (`-uno`: an untracked file never counts).
- **A subagent's brief names its function in the body, never in the
  task line alone** (2875, 2857's Loop line): two subagents found a
  brief that ended at "Your function:".
- **A subagent never ends its turn on a background wait** (2820, the
  commander's): it is not re-invoked, and 2793 sat 1 h 50 on one that
  did; `lanewait.py` names a lane idle behind a child older than
  `--stalled-child-after` (90 minutes).
- The sweep rows name what the pass built for the lane: the `ui` mark
  (2819), the bare-citation list (2843), `@FILE` and `--port-file`
  (2796) and the regen that refuses to reset a hand port (2792).

## The brief checklist, from the thirty-seventh pass

The thirty-seventh tranche (DECISIONS 74): twenty-three landings on
four lanes in 6 h 11 min from the pass's commit, 4.22 USD and 36.5
minutes waiting a landing (3.92 / 27.0 before) — the waiting broke a
fifth, and the fourth lane goes by entry 61 §5. The pass added these
rows, each from the tranche's own journals or its own measure, and
struck eight rows the tools now enforce: 1150, 1180, 1217, 1234,
1513, 1543, 1699 and 1987.

- **A sweep's per-function check and its breaks run in debug, never
  `--release`** (2787, the pass's measure): a touched sweep test cycles
  in 4 s against 29 on an idle box; the harvest lane ran about forty
  release cycles a batch under four lanes and waited 40 to 49 minutes
  on them in items 2683, 2743 and 2774. The release build is the lane gate's; a
  red in debug alone is an overflow the port must spell.
- **A shape two batches parked is its own item, never a third batch's
  tail** (2732, 2719's Loop line): 2701, 2712 and 2720 parked the
  callee-run shape while each brief led with another build.
- **A fallback's parting asks first which of its gate's inputs
  differs** (2726, 2703's Loop line): draws 0..24 agreeing said the
  inputs were equal up to one, and the 195 in `danger` was the cause;
  2761 found its word by the same row in four commands.
- **A standing row on the object the parted orders target is walked
  back too** (2738, 2677's Loop line): 5407's cause sat two windows
  back in two standing rows on the capital that two widenings listed
  and nobody walked.
- **A one-field path parting lists the writers that copy an entry
  first** (2762, 2737's Loop line): `path[34]` agreed on `to` and
  parted on `tolerance` alone, a popped `goal_e` copied whole.
- **A standing `num_units`/`control` row on the parted queue's player
  is grepped in the parked file first** (2781, 2761's Loop line): a
  two-week-old parked gap, parked 1092 and 1328, said when it returns.
- **The staging is the landing; a build without it parks** (2756,
  2725's Loop line): chapter eighty was booked to stage an arm, built
  it from the listing and staged nothing, and was booked again.

## The brief checklist, from the thirty-sixth pass

The thirty-sixth tranche (DECISIONS 73): twenty-two landings on the
same three lanes in 5 h 52 min from the pass's commit, 3.92 USD and
27.0 minutes waiting a landing, no kill rule tripped, no red booking
gate. The pass added these rows, each from the tranche's own journals
or the commander's, and struck three the tools now enforce, rows 1839,
1885, 1931 and 2199.

- **A one-draw `Guy::inc_time` delta is placed by `Unit::inc_time`'s
  `+0x3e`/`+0x6e` callers in theirs' trace** (2639, 2609's Loop line):
  the word's unit in one read, no probe.
- **A word under a `Group::` caller takes `RON_DEBUG_ARMIES` beside
  `RON_STACKS`** (2610, 2554's Loop line): the dump prints no army
  record, and the probe named the army, its target and its leader.
- **A value the dump prints that no reading produces is grepped in the
  export** (2577, 2543's Loop line): a reading and a packet both agreed
  with this crate while `job_counter 500000` sat on every `finish`
  capture; the grep named `run_cmd` case 0x4f in a minute.
- **A value the original computes and ours does not runs on a packet,
  `step4-entered.txt` first** (2527, 2491's Loop line): the arm in 0.01
  s after an hour of reading the tail by hand.
- **The capture wait's `timeout` is the Bash tool's argument, never a
  command** (2599, 2572's Loop line): macOS has no `timeout`, and a
  prefixed wait exited at once with run871 still capturing.
- **An edit inside a `&&` chain is python or `sed -i ''`** (2628,
  2598's Loop line): a BSD `sed -i` failed first and the commit and the
  launch behind it never ran.
- **A control is the recipe with `--cmd-without <the arm's frame>` on
  the second reserved run** (2542, 2519's and 2530's Loop lines):
  run869's control was cut with `grep -v` into a job tmp, and its
  record carries a placeholder.
- **A float return's register is the listing's last write before
  `ret`, never the export's `in_ST0`** (2552, 2537's Loop line): an
  `st0` build was booked on the misprint and found wrong at the listing.
- **A sweep batch takes the functions a built shape holds first,
  shortest of the rest after** (2622, the commander's): six batches
  backed 10, 3, 10, 6, 7 and 1 as each session built one shape for one
  or two functions.
- **A sweep's `agent-*` worktrees go with its landing**: three stood a
  day after the thirty-fifth tranche, two with unmerged tips.

## The brief checklist, from the thirty-fifth pass

The thirty-fifth tranche (DECISIONS 72): twenty-two landings on three
lanes — Opus 5.5 medium on the AI word, Sonnet 5.5 medium on the rules
track and the sweep — in 6 h 17 min from the first booking, 4.18 USD and
33.6 minutes waiting a landing. The pass added these rows, each from the
tranche's own journals or the commander's.

- **The lane gate runs every timed test when the branch touches sim
  outside `sweep/` or a `rondata::diff` row list, and there is no
  whole-suite run before it** (2415, the commander's; 2409, the
  thirty-fourth steer's): both kill rules of the tranche tripped by one
  route — 2384's 144 `aggression` pins and 2400's floor 17379 → 14461
  were red only at the booking gate — and the suite run before the lane
  gate was the booking gate's work done twice, 5.9 minutes a landing.
  `release_gate.py --lane` reads the branch's paths since `--base`.
- **The harness cuts a worktree subagent from the integration tip; its
  first line is `git merge <the lane's branch>`** (2419, 2400's Loop
  line): a brief's "cut every worktree from the scaffold" had the
  subagent start without the lane's commits.
- **A break is `tools/mutate.py`, never a hand script** (2442, 2421's
  Loop line; 2480, 2469's): a hand restore by `shutil.move` left cargo a
  stale mutated binary, and a hand pass check keyed on `failed` matched
  cargo's `0 failed`.
- **The capture wait is on the viadriver log the launcher's `log:` line
  names** (2449, 2436's Loop line): `waitrun.sh` on a launcher log
  written with `>` holds no banner and exits 2 at once.
- **A golden cheat goes two frames clear of a birth it could race**
  (2479, 2448's Loop line): run859's first take put `age` on the
  Despot's birth frame, read before the birth by the original and after
  by the walk; a retake eight frames later closed.

## The brief checklist, from the thirty-fourth pass

The thirty-fourth tranche (DECISIONS 71): twenty-three landings on four
Opus 5.5 medium lanes in 6 h 04 min from the first booking, 7.07 USD and
36.3 minutes waiting a landing. The pass added these rows, each from the
tranche's own journals.

- **A leader row on the word's block: `diff` the dump's whole
  `LEADERDATA` across the block, unread fields included** (2292, 2259's
  Loop line): 4671's booked lead was the effect; the one-line diff named
  `wonder_mark` and `num_buildings`, both fields the coverage pin lists
  as unread.
- **A birth on the word's frame: `RON_DEBUG_BUILDS` over `<word−1>–<word>`
  names the queue that emptied, before guessing the building** (2319,
  2293's Loop line): 4946's booking guessed the Airbase from a standing
  row that was a sibling of the cause.
- **A packet for a decision first printed on block N is taken at
  `RON_STATE_FRAME=N−1`** (2303, 2265's Loop line): the brief's "at the
  word's own frame" cost run842 a second take; `docs/EMULATOR.md` §8
  had the rule.
- **No capture while a packet or widening of the game on disk answers the
  arm** (2330, 2304's Loop line): two packets answered chapter sixty-six
  without run 845, and a closed chapter's widening held 126 rows no word
  sees.
- **An arm the AI track built: grep its constant's every reader in the
  export, port or park each, before staging** (2356, 2331's Loop line):
  Monarchy's train arm had a cost twin one listing away, unported, found
  when the staged unit's price parted.
- **A sweep subagent's scratch goes under the job's tmp, named by
  function, never `/tmp`** (2373, 2341's Loop line): one subagent ran a
  sibling's break script against the sibling's worktree.
- **The census's backed column is reported, never re-pinned by a lane**
  (2367, the commander's): `census::SIMULATION_BACKED` is the two sweep
  lanes' shared pin and seven of thirteen merges backed out on it; the
  commander pins it at booking and the lane gate forgives its red.
- **`port` marks a consumer whose `Class::method` the crate spells;
  `name` a name match alone** (2286, 2221's and 2276's Loop lines):
  `BorderNodes::find` was marked a port consumer because something
  defines a `fn find`.
- **A backticked name is the compiler's and is dead weight** (2394,
  2379's Loop line): the forty shortest rows of `--unbacked` were
  initializers every lane read past by eye.

## The brief checklist, from the thirty-third pass

The thirty-third tranche (DECISIONS 70): twenty-three landings on four
Opus 5.5 medium lanes in 5 h 36 min, 7.60 USD and 23.8 minutes waiting
a landing. The pass added these rows, each from the tranche's own
journals.

- **A pair read as swapped says which swap it is** (2154, 2130's Loop
  line): run820's block 4247 read as a collision and was a job swap,
  shown by `unitat.py` on an earlier window in one command.
- **A standing row on a unit the block names is a candidate, scoring or
  not, and is walked back through every earlier widening of the pair
  before its mechanism is read** (2180, 2264; 2155's and 2229's Loop
  lines): `order:move.facing`, exempt from the score, stood on run820's
  first block for three items; 2229's parting was 1,665 frames before
  its word and each widening's standing rows led one window back.
- **A flag predicate standing in for a lineage is checked at each reader
  against the reader's own vslot** (2198, 2175's Loop line): `0x40 &&
  !0x20` was copied into two readers where the original asks
  `is_supply` alone, and the successor pair's word sat on it.
- **A mirror or `facing` parting lists every original `Unit::set_angle`
  caller this crate reaches without `unit_set_angle`** (2234, 2193's
  Loop line): a turn written through `Movement::set_facing` dropped the
  group arm in silence, and `seams.py --item` did not print the doc
  that listed it.
- **A break that passes is read against the draw before the code**
  (2222, 2200's Loop line); **a table the image builds at boot reads as
  zeros under the emulator** (2216, 2187's Loop line): two of ten breaks
  passed on a starved arm; `covers_tile`'s original answered 0 on every
  row through an unfilled `div_3_table`.
- ~~**A long with no `--detail` is refused before staging** (2199, 2107's
  Loop line): run830 staged `end:UNITS=3` on every frame, 37 minutes and
  717 MB the walker could not stand on.~~ **Struck by the thirty-sixth
  pass**: the runner says so itself, before staging.
- **`unitat.py` prints a `GroupMoveOrder`** (2145, 2131's Loop line):
  the dump spells it mixed-case and the printer's suffix match dropped
  it, so run827's 662 read as "the original queued nothing".
- **`repin.py --write` rewrites a golden `WANT_CH*` list** (2071, four
  chapters by hand): the want at the site is a local built from the
  const, followed to its `let … = CONST.iter()`.

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
~~- **A row three widenings pin is a rule, not a residue** (1176): run414,
  run419 and run420 each pinned a passenger's `avg_speed` apart after a
  landing, and it was one unbuilt exit rule.~~
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
- ~~**The lane says its own state** (1180): `ron_lane_state` prints
  `free`, `stale` or `held by`. A lock whose pids were dead read as a
  busy lane on eight landings, and one whose pid had been handed to
  another process refused a capture with no game running; the lock
  carries each pid's start time now. A stale lock is never removed by
  hand — one removal was refused by the permission classifier, rightly.~~ **Struck by the thirty-seventh pass**: the tool says so itself.
- ~~**A first take that dies before frame 0 is taken again once** before
  anything is read into it (1150): run416's and run420's first takes
  faulted inside Wine within four seconds of the launch.~~ **Struck by the thirty-seventh pass**: the tool says so itself.
- ~~**A release node is read with the `get_position` sweep on a packet**
  (1208): reached three times from one scratch file; the item that
  reaches it next commits it under `tools/recomp/`.~~ Struck by the
  twenty-third pass to pay for the done line's tool: one reach since the
  twentieth pass, and the sweep is still the scratch file's.
- **A script that stages nothing still exits `success: true`** (1170),
  and **a command to another player's units holds `be` across the pump**
  (1205): two of run422's three takes and run436's first were lost to
  the script's own lines.
- **Two sections appended at one anchor, or a count two lanes re-pinned,
  merge by hand with both kept** (1161, 1151), and the count is measured
  again on the merged tree. Six landings met a refused update and none
  was on code; the capture stanzas merge by union now, and a
  specification does not, because it is amended in place.
- ~~**A word that reaches its trace's end owes five things** (1217): the
  `ENDPOINTS` row and its `Endpoint` part, the `AI_WORDS` row closed,
  the compared pin's window dropped with its fields re-pinned, the
  coverage driver's word block a constant, a capture of the last blocks.~~
  **Struck by the thirty-seventh pass**: `tools/runwiring.py` says which (1729).
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

## The brief checklist, from the twenty-first pass

The rows continue, under a ceiling now: `tools/brief/frame.md` may not
grow past `brief.FRAME_CEILING` (parked 1226, the twenty-first pass,
2026-09-30), so a row is added when the frame has room and a Loop line
with one reach stays parked until it does. The twenty-first pass added
these from the three-lane tranche's "for the Loop" lines and two
failures of its own, and struck the frame's own story to pay for them.

- **Name each unit a booking names by its `TypeIndex` and domain before
  reading its chain** (1267, two reaches): "the scout" of 1281's booking
  was a Caravel, `num_queued[84]` was Pikemen and not the make list's
  Elite Javelineers, and the unit's domain decided the arm both times.
  A type number says its keying — `TypeIndex` for the dump's
  `num_queued` and the make list, the record's index for `num_units`.
- **When one side spent nothing on the word's frame, `seams.py` runs on
  the chain of the order the other side took** (1233): at 8907 ours
  spent nothing, `--item` read the original's chain, and the seam that
  named the cause sat on ours' own path.
- **A comment that says "not modelled" without the word is a seam, a
  plural `SEAMS:` block is one, and a list under a bold lead-in is its
  lead-in's paragraph** (1253, 1240; `tools/seams.py`): `find_friends`'
  enhancer arm decided the third map's 5376 from a plain comment,
  `do_cast`'s captain check sat in a `SEAMS:` block, and ROADS §9.4's
  `leech_codes` in the bullets under "**Not modelled**". The scan's
  widening found three more doors (14 → 17, parked 1224).
- **The first parted field's writers and readers are read for seams by
  name, before any reading** (1340, 1337, 1322; `tools/seams.py
  --field`): six landings of the twenty-first tranche found the answer
  in a `SEAM` on the field's pusher, converter, reader or gate — none on
  the word's chain — and five of them on a second, hand-named call. The
  verb names those functions itself, writers first, and a comment that
  says "a seam" without the word is a seam.
- **A chapter's `chNN f…` firsts pipe through `standing.py` as the AI
  track's `first …` do** (1250): item 1235 converted them with `sed`.
- ~~**The lane is held through the runner's restore and released after**
  (1234, three reaches; `unattended_capture.capture_lane`,
  `ron_lane_take <pid>`, `ron_lane_release`): run467's game exited, the
  lane read `stale`, and 1221's long trace launched into the restore.
  A waiter accepts `free` or `stale`; a `held by` is a live pid.~~ **Struck by the thirty-seventh pass**: the tool says so itself.
- **A take with `@` lines is read with `issuesmatch.py --none-refused`
  first** (966, 1256): two Loop lines of the tranche asked for a decoder
  of `INFO 17` that the seventeenth pass had built; the row had not
  reached the frame.
- ~~**A change on a shared path runs the widenings whose captures reach it
  before the gate** (1313): `land_passenger`'s snap moved nineteen
  widenings at the gate and seven more at the merge.~~ **Struck by the thirty-fifth pass**: the lane gate runs those widenings itself (2415).
- **Read your run's `docs/RUNS.md` section whole after `ccc update`**
  (1285): the union merge spliced run480's "Taken" into run476's, and
  run488's and run490's the same way.
- **A landing that moves another lane's word says so** (1289): 1264
  moved the third map's word under 1275, whose run491 and run494 became
  one window taken twice.
- **A command the worktree guard refuses is written to
  `$CLAUDE_JOB_DIR/tmp/` and run from there, and `no verdict (error)`
  is backed off, not stopped on** (1315, the pass's own — corrected the
  same day from the transcripts): item 1291 met five "server-side auto
  mode classifier gave no verdict (error)" answers in three minutes,
  ended its turn by the permission-shaped rule, and sat 163 minutes
  until a person typed "are we stuck?"; item 1293's `git merge`
  into its own branch was denied by the auto-mode classifier as "Git
  Destructive", `ccc update` backed out on the conflict, and it stopped
  as its commander asked and sat 86. The outage was the service's — 22 such
  answers in six sessions on 2026-09-30, none before the 27th — and the
  hours were the rule's. The pass itself was refused four times by the
  worktree guard on commands naming nothing of git, at a turn each.
- **The frame has a ceiling and may only fall** (1226, the pass's own):
  12,959 characters at the twentieth pass, 13,969 with this pass's rows,
  12,963 after the story left it and three one-reach rows stayed parked.

## The brief checklist, from the twenty-third pass

The first tranche on Sonnet 5.5 (DECISIONS 58, 59): eleven landings,
every word moved or chapter opened as booked, and the lanes' silences
were where the rows went. The pass added these and struck two rows to
pay: the chapter-script row that item 1094's test already guards, and
the release-node row below, one reach since the twentieth pass.

- **The done line names its tool and its address** (1412, 1406): three
  lanes wrote their landing in their own transcript and never loaded
  `SendMessage` — a deferred tool, 0 `ToolSearch` calls in each — and
  the commander slept three hours and thirty-four minutes until a
  person asked; a fourth sent "red only on your two lines" with neither
  `Lane verdict:` nor `Gate steps:` in it. The row reads
  `ToolSearch("select:SendMessage")`, then `SendMessage` to
  `attrition`, with the two lines quoted; and the chain subscribes to
  each lane's idle at the spawn, so the notice ends the next sleep.
- **A mutation runs through `tools/mutate.py`** (1391, with 784 and
  907's prose): it refuses a dirty tree, a mutation that took nothing
  and a tree that changed under the command — the mutated file itself
  included — and its exit is the command's. Seventy-eight scratch
  scripts in one tranche, one of which let the worker edit sources
  while it ran.
- **A re-pin runs through `tools/repin.py <cargo log> --write`** (1349,
  1411): it rewrites every literal `want` the `N pins moved:` report
  names and lists by hand the rest; 140, 8 and 85 pins were rewritten
  by scratch script before it. The delta in the constant's comment and
  the word's block in its widening stay yours (`CLAUDE.md`).
- **`seams.py --field` knows a struct literal from a declaration, says
  where a bare name occurs when it is no field, and `--unwritten
  <Struct>` lists the fields no non-test line writes** (1397, 1382,
  1400): four "writers none" on a cause in one tranche — two literals
  spanning lines, a variant, a gate in a callee's head — 1409 stays —
  and the standing rows of a window's first block held a field nothing
  wrote, twice.

## The brief checklist, from the twenty-fifth pass

The twenty-fifth tranche (DECISIONS 61, 62): twenty landings on a mixed
roster, and the rows are the lanes' own Loop lines. The pass added these
and paid by condensing four rows' wording, no number struck.

- **The compared pin's unread list is grepped for the parted field**
  (1475): seven journals of twenty named a key `coverage.rs` pins
  uncompared on the cause — `construct_hits` and `myhits` twice, a
  General's `mana_burn`, `leader_flags`, a field carried under another
  name — and each was dated to its first block by a grep after the fact.
  Quiet on such a key is a blind instrument, not agreement.
- **`tools/standing.py` reads the sibling widening's print** (1471): the
  third pair's and the coverage pair's `widen_on_siblings` write
  `standing <block> …` and `f<block> …`, and two landings fed them through
  `sed`; the tool reads both now.
- ~~**No `mkdir -p` before a launch** (1513)~~ (struck by the thirty-seventh pass: the runner refuses the directory and retakes), **and a traceback with no receipt is a
  dead take** (1503): ~~`captures.txt`'s runner refuses an output
  directory that exists, so the habit loses a launch~~; run659's first
  take died in `autostart_receipt.py` with no receipt and the lane's
  waiter sat on another lane's runner — `waitrun.sh` exits 2 on the
  traceback at once.

## The brief checklist, from the twenty-sixth pass

The twenty-sixth tranche (DECISIONS 62, 63): twenty landings on the mixed
roster, two lanes. The pass added these six rows, each a lane's own Loop
line, and paid by condensing some twenty-five rows' wording, no number
struck (`tools/brief/frame.md` at 12,989 of 13,000).

- **`ccc update` takes the lane's ref, never a SHA** (1557): a commander's
  "take `ccc update` onto <sha>" is one word off what the verb takes; the
  SHA is the base the update lands, and the brief says `att-<n>`.
- **On a packet, run the unit's whole `process` first and read its call
  trace** (1526): 1519 read three predicates both sides passed before the
  one that parted, and the trace named it in one run.
- **`--end-frame` is the game's last frame when the window reaches it,
  and a lobby that ends itself takes `--allow-early-end`** (1529, 1551):
  the receipt counts frames to `end + 1`, so run668's first take was lost
  to an endpoint past the game's end, and every coverage long (run652,
  run676) ended itself and read `success: false`; the runner's flag says
  `ended_early` and `last_frame` instead.
- **A detached suite is killed by its pid, never by `pkill -f` on its
  name** (1527): every worktree builds `rondata`'s test binary under one
  hashed name, and 1511's kill of its own suite matched 1524's.
- **`timeout` is absent on macOS** (1537): a brief habit of `timeout 3000
  cargo …` exits 127 with nothing useful.
- ~~**A one-string command is refused** (1543)~~, **the cap for the whole suite is 20,
  one filter a `cargo test`** (1551): ~~`tools/mutate.py -- $T` under
  zsh hands the command as one word and printed a traceback — it refuses
  now~~ (struck by the thirty-seventh pass: the tool says so itself); `tools/memcap.sh 10` is breached by the full run (10.2 GiB) where
  the gate caps at 20; `cargo test … run677 sahara_coverage` errors and
  prints nothing.
- **The lane gate names the word's tests; the slow ones are the
  booking gate's** (1567, DECISIONS 63 amended): the commander's brief
  passes the word's filters — the widening the item cites, the walk its
  map's floor is pinned by — as `--lane --tests <filters>`, and the gate
  `--skip`s every rondata test the timings file times over a second
  unless a filter names it (the two coverage pins first, 118 s and 80 s
  alone); a filter that matched no test is red, and a word that reads a
  new key adds `coverage::`. The worker runs the whole suite once on its
  own tree after its last `ccc update`; the booking gate runs it again
  and forgives nothing. Every landing had run it three times on one box.
- **No brief names a click-free lane** (1568, 1569, parked 1139): the
  runner takes the first free lane of the pool, grows one to
  `RON_LANES_MAX` and waits at the cap; the receipt names the lane, and a
  brief reserves only the run range. `RON_CAPTURE_LANE` is a test's. The
  queue lane and the pool exclude each other; `tools/gamelog/lanes.py
  --since <the pass's commit>` is the number the pass moves the cap on —
  peak lanes, minutes waited at the cap, frames per wall second.

## The brief checklist, from the twenty-seventh pass

The twenty-seventh tranche (DECISIONS 63, 64): twenty-two landings on
three lanes, a race and a sweep among them. The pass added these rows,
each a lane's own Loop line, a `sweep` section for the fanned batch's
brief, and paid by condensing eight rows' wording, no number struck; the
ceiling is each kind's own sections now (the chapter kind at 12,988 of
13,000), since the sweep's section reaches no word lane.

- **A projectile's landing reads the shooter's firsts, not the target's;
  two numbers that swap are read as births by `uid` across the gap**
  (1597): 1591's cause was 150 frames behind the nuke's landing and drew
  nothing — the attacker's `RON_FIRSTS` went straight to it; 1594's first
  parting was another unit's field list because two slots had swapped.
- **Grep the integration branch, `docs/AI.md` and the journals for a
  function before building it; a mechanism that makes a record kind
  appear in a window runs the `coverage` tests too** (1601): two lanes
  built `do_launch`'s silo arm in one hour, and a widening that first
  prints a kind moves the coverage pins.
- **When the booking is two destinations, print both sides' unit state
  and list the firsts of the unit the draws name, whatever the frame**
  (1622): "grep each value the booking names" found nothing for 1611; the
  cause parted 205 frames early with every draw agreeing.
- **Grep the `SEAM`s for the subsystem the event goes through, not the
  field** (1633): 1620's cause was a seam in `nuke.rs` that says "fog",
  which `seams.py --field ever_seen` could not reach.
- **The lane gate's filters always carry `coverage::`** (1623):
  `tools/brief.py --tests` adds it; 1611's widening put a window in the
  coverage driver and the two coverage pins were red on the booking gate
  alone.
- **A fanned batch's subagent worktrees are cut from the scaffold
  commit, and the shared target dir serves a stale binary** (1629):
  1619's ten started on the base and each reset before it could see
  `sweep/`; "0 tests" is the stale binary, cured by `touch`.
- **A sweep row's literal banks no constant** (1624): the constant guard
  skips `sweep/` by design, ratified by the pass.
- **At 300 k of context a lane writes its journal as a draft, commits,
  and arms its own clear with "continue item N from the journal"**
  (1637, decided with Ramon after the pass): one item runs 150 to 270
  requests and grows to half a million tokens of dump greps; about half
  a lane's price is spent above 300 k (the tranche reader's deep share,
  0.46 to 0.72 on the coverage lane), and four fifths of the bill is
  cache reads priced the same on Opus and Sonnet — so the context, not
  the model, is the cost. The commander's clear at ten is the same move.

## The brief checklist, from the twenty-eighth pass

The race on 1648 (DECISIONS 64, amended; 65): twenty landings on four
arms, one word, no other lane. The pass added these rows, each from the
race's own journals, and paid by condensing four rows' wording, no
number struck; the chapter kind stays under its ceiling.

- **A capture's wait is backgrounded with a `timeout` past its length**
  (1672): the harness kills a background command at thirty minutes by
  default, and `att-1648-sm`'s fourth landing lost that half hour to a
  `waitrun.sh` wait that died before the banner.
- **A landing ends the turn as done, never as needing input** (1671):
  five of the race's twenty landings — every one on Opus 5.5 — ended
  `tempo=blocked` asking for the commander's queue lines, and an armed
  clear fires only on idle; each cost a message and the minutes until
  it was read. The waiter prints the row's own detail on its `ENDED`
  line now, so the unblocking message is written in the turn that reads it.
- **A birth no queue holds is read off the building beside it** (1674):
  two of the four arms found the Terra Cotta Army's free soldier by a
  hand-written radius scan over `BUILDDATA`; `births.py` prints the
  owner's nearest building, with its `orig_type` and the distance, on
  every `+` line.

## The brief checklist, from the twenty-ninth pass

The twenty-ninth tranche (DECISIONS 65, 66): twenty-three landings on
four Sonnet 5.5 medium lanes in 4 h 11 min. The pass added these rows,
each from the tranche's own journals, and paid by condensing eleven
rows' wording, no number struck; the chapter kind stands at 12,990 of
13,000.

- **A probe goes in behind one env guard, in one commit reverted
  whole** (1727, 1717): 1716's regex that stripped its probes ate real
  code, and 1697's `git checkout <file>` to drop a print took the
  uncommitted work beside it — the rule was in `CLAUDE.md` and was
  broken anyway.
- **A gate prints `Gate pid:` first; kill it by that pid** (1713): two
  lanes `pkill -f`'d `release_gate.py` to stop their own suite and
  killed every gate on the box, the booking gate twice (exit 144).
- **A new window names its `GROUND_INEXACT` row first** (1715): the
  table's default is 0, and 1700's new window panicked on the height
  pin before the pins reported the count a worker re-pins.
- **A staging that names ids reads them off `RON_STAGE_ALL`'s print**
  (1730): 1707's `add 4 hoplites` made four squads, `0/7`..`0/29`.
- **A subagent's report is taken by its SHA** (1692): 1630's reported a
  commit on a branch it never committed on.
- **The sweep's batch is `tools/census.py --unbacked`'s, and
  `tools/sweep_gen.py` writes a shape's two files** (1703, 1725, 1706):
  four batches wrote the lister and three the generator as scratch
  scripts; the lister marks a port consumer, since 1724's ten had none.
- ~~**The lane gate takes `--item N` and refuses a tree with no journal**
  (1699): 1639's chain died on a `sed` and its done line went out
  without the journal.~~ **Struck by the thirty-seventh pass**: the tool says so itself.
- **At 300 k the waiter tells the commander, and the commander arms the
  clear** (1637, 1638): five sessions over 300 k in two tranches took
  the frame's self-arm none — a lane cannot read its own context, and
  the classifier refuses a session's arm of its own clear.
- **`tools/runwiring.py runNNN` prints where a run is wired** (1729):
  1719's five closing things and 1728's six pair edits were read off a
  diff by hand, the third reach.
- **A seam that says no capture holds its arm is dated by `seams.py
  --dated`** (1590): 1639 broke SYNC §3.8's "nothing on any capture has
  demolished a farm" silently; 624 such sentences stand, each with its
  blame day and the runs captured since.

## The brief checklist, from the thirtieth pass

The thirtieth tranche (DECISIONS 66, 67): twenty-three landings on four
Sonnet 5.5 medium lanes in 3 h 51 min, 2.78 USD and 20.9 minutes waiting
a landing. The pass added these rows, each from the tranche's own
journals, and paid by trimming the wording of twenty rows, no number
struck; the chapter kind stands at 12,979 of 13,000.

- ~~**The seams are read for the lane, in the brief** (1743): the frame's
  step 7 ran in one journal of nine word lanes and `standing.py` in none
  (lore's transcripts: `seams.py` once, `--callees` once, `report.py when`
  never), so `tools/brief.py` runs `seams.py --item N` and carries the
  print; the lane re-runs it with `--field` once it has one.~~
- **A probe is committed before any other edit; never `commit -a` across
  a probe and a build** (1864): 1829's revert of its probe commits took
  an uncommitted build that a `commit -a` had swallowed.
- **A chapter at its trace's end closes with `word == hi − 1` in its
  widening and `chN closed` on the `Golden:` line** (1781): 1740 found
  the form by its gate; `ENDPOINTS`/`AI_WORDS` are the long captures'.
- **The journal is written with the file tool, never in a backgrounded
  `&&` chain** (1781): a chain lost its heredoc once.
- **Parked rows go under the lane's park heading, never at the file's
  tail** (1812): 1744, 1768 and 1845 each appended `## Parked by item N`
  at the end of `docs/PARKED.md` and each merge backed out there; the
  pool is six headings deep, assigned by letter, held at two by
  `docs_guard`.
- **`RON_DEBUG_UNIT` prints `TI`, `TY` and the type's name** (1846): a
  brief that named a unit by `TY` read 272 as a `HUMMEL`; the Freighter
  was `TI` 322.
- ~~**The staging walk refuses a frame below its predecessor, as the
  capture does** (1839): run783's first launch died on a script
  `stage_walk` had walked clean.~~ **Struck by the thirty-sixth pass**:
  both refuse in their own words.
- **The sweep's shapes are `sweep_gen.py`'s six, its helpers
  `sweeplib.py`, a float argument `xmm_bits`; the dead weight is hidden
  from `--unbacked`** (1761, 1867, 1861): fourteen batches threw away
  the same generator, and every lane skipped destructors and `close`
  by hand ~300 bytes deep.
- **`step4.py --poke` and `--hook`** (1869): two scratch edits answered
  1845 whole, and 1738 hand-wrote the hook as `s4h.py`.
- **A `Pair`'s word is a const field `repin.py` rewrites** (1767): three
  re-pins of the successor pair's word were hand edits.

## The brief checklist, from the thirty-first pass

The thirty-first tranche (DECISIONS 68): twenty-three landings on four
Sonnet 5.5 medium lanes in 4 h 33 min, 3.56 USD and 24.9 minutes waiting
a landing, seven captures. The pass added these rows, each from the
tranche's own journals.

- **The lane gate carries every lobby's `first_parting`** (1910, 2000):
  1937's fix moved Great Sahara and broke the coverage pair's first
  parting 4730 → 4206, read by a capture; 1870's moved four coverage-pair
  widenings no filter named. `brief.py` adds the filter to a word's.
- **One unit out of a dump is `unitat.py`** (1981, 1739, 1601): four
  lanes wrote the parser; the type by name, the path's ends, the orders
  first issued first where the dump prints the top first, and a window's
  changes.
- **A unit's packet address is `unitaddr.py`** (1953): three lanes
  scanned for the masked pair by hand.
- ~~**The runner retakes a take that died before frame 0 once** (1987):
  run801's first take, 1988's lost take.~~ **Struck by the thirty-seventh pass**: the tool says so itself. ~~**It refuses a staged line
  past `--end-frame` in one line, and refuses `GROUPS` beside `DEATHS`
  before staging** (1885, 1931): run791's two takes.~~ **Struck by the
  thirty-sixth pass**: the runner says so itself.
- **`sweep_gen.py` has `obj` with windows, `void`, `vcall --own` and an
  answer that sets `ecx`** (1890, 1922): thirteen reaches of one Loop
  item, every batch of the tranche a hand-written script of one of
  these shapes; three generated scripts ran against the executable in
  the pass, 220 rows each, agreeing.
- **The coverage driver's windows are a slice** (1959): two lanes each
  added a tuple slot and the merged pattern was one short, a compile red
  no lane gate saw.

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

**Batch 24, 2026-10-05, the twenty-fourth pass
(`docs/audit/2026-10-05-fable-pass-24.md`): parked 1447's set.** Nine
blind readings and nine adjudications on Opus over the thirty landings
of 1426–1460; forty-two code-changing rows in twenty-two groups, the
lead row of each group re-read by the pass at the decompile or the
listing and the rest standing on the adjudicator's citation; two
marked rows settled (the avoid-land flag's writers, by the listing;
the coastal cell's second region, by seven dumps) and four left open
with their probes (A1 row 11, A5 #49, A7 row 19, A9 row 6). The builds
are queue item 1468. The adjudication files are kept outside the
repository, at `~/ghidra-projects/reading/pass24-2026-10-05/`.

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
