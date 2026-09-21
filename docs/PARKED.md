# Parked

The backlog that does not boot. `docs/QUEUE.md` holds what is in flight and
what is next; this holds everything else, and **a fresh session does not read
it** — the queue's opener is what starts a session, and this file is opened
when a *wave is composed*, which is a different and rarer moment (item 263,
decided 2026-09-07 with Ramon and lore).

Two rules ride with that, and they are the conditions the split was agreed
under:

- **Read this when composing a wave, not at boot.** Once per wave is cheap;
  once per session was the cost being removed.
- **A cross-item constraint does not park.** If an item says something a
  worker on *another* item must obey — that a file is about to be split,
  that a mechanism cannot move a scored word — it is stated in that
  worker's brief, or the item is promoted back to the queue. Two workers in
  one file is the failure this split would otherwise buy.

Numbers here are live: `tools/queueledger.py` reads this file alongside the
queue, so moving an item between the two is not a deletion and never reads
as one. Everything in `docs/QUEUE.md`'s "How to maintain this file" applies
here too, except the item cap — a parked item is not an open one.

## Loop — the steering pass's, never a worker's

Tooling, guards and the queue's own rules. The Fable pass takes these
(`CLAUDE.md`, "Fan-out rules"); a commander never spawns one. Item 279's
two ledger regexes closed in the third pass, 2026-09-17; the fourth pass,
2026-09-18, ruled six (`docs/audit/2026-09-18-fable-pass-4.md`); the sixth,
2026-09-19, ruled ten — 377, 398, 406, 404, 411, 397, 374, 343, 356, 321 —
each now a guard or a clause (`docs/audit/2026-09-19-fable-pass-6.md`,
DECISIONS 42); the seventh, 2026-09-21, ruled nine — 416, 417, 419, 420,
421, 424, 431, 433, 436 — two guards, six clauses and a DECISIONS
paragraph — and **built 367**, the AI's dump, the same day: run114 and
`docs/AI.md` §52 (`docs/audit/2026-09-21-fable-pass-7.md`, DECISIONS 43).
341 closed 2026-09-18 by item 363.

(449) **A widening pin, and a vacuity guard, can go stale by success.**
Measured twice on 2026-09-21 by item 442, which moved Great Lakes 9510 →
10161. `the_widening_behind_each_pinned_word_exists` checks that a named
widening test **exists**, never that it widens the word's *current*
frame — so when the word moved 651 frames past run100's window the row
kept passing and kept reading as pinned, which is the exact failure
DECISIONS 43 was written against. Repointed to `None, 448` by hand at
book time; the guard shape would be to compare the named test's frame
with the word it is pinned beside, which needs the frame to be declared
rather than buried in the test body. 442 reports the same class twice
more from the other side: a test keyed on the headline reports the
headline's *motion* as its own failure (both run100 tests did), and the
ladder's shared-extras floor fell 15 → 10 because the AI stopped buying
spurious units — a floor that only ratchets one way cannot tell a win
from a regression. One family, three instances, and the pass should rule
the family rather than the three.

(446) **The capture lane has no interlock but a conversation.** Raised by
Ramon, 2026-09-21: `astra`, a Codex session on this machine, borrows the
lane in ~30-second bursts and asks each time whether it is free. Today's
answer was `ps` plus the Logs directory's mtime, and the fence was two
`SendMessage`s to the lanes in flight — which worked only because astra
asked. The prefix, the window and the Logs directory are singletons, and
nothing stops one of ours launching the game into a running capture; a
capture that collides is also the kind of loss a branch check cannot see,
because the damage is a log file, not a commit. **Explicitly not a
priority** — the bursts are rare and always announced, and that is the
user's own framing. Parked for the pass to weigh shape against cost. Two
candidates: a lock file taken and released by `tools/gamelog/winelaunch.sh`
— the single line every capture script already sources, so there is exactly
one place to write it — carrying holder and start time and going stale
after N minutes; or nothing at all, on the grounds that two agents and a
question is cheaper than a mechanism nobody outside this repo honours. The
pass should decide it **together with (428)**: both are "two lanes, one
singleton, no interlock", and one answer may serve both.

(428) **Reserving a run number does not reserve the append point.**
Measured 2026-09-19: two lanes ran captures in parallel with run111 and
run112 properly reserved to each, and their merges still conflicted,
because both append a section to the **end** of `docs/RUNS.md` and neither
brief said where. The second merge refused and backed out — nothing lost,
but a landing stalled on a resolution the commander's brief could have
prevented. Two shapes to weigh: a brief that reserves the append order the
way it reserves the number, which costs a clause and fails silently when a
lane lands out of order; or one section per file under `docs/runs/`, the
way `docs/journal/` already works, which cannot conflict at all and is the
pattern this repo has already chosen once for exactly this reason. The
ledger's own section order would then be a generated index. The pass's,
because it changes a document's shape and the guards that read it.

(375) **A staged run is indistinguishable from an unstaged one, and the
borrow of a checksum trace is ungated.** Found by item 364, 2026-09-18,
while measuring its own first golden word as an artefact.
`borrow_from_siblings` gates `frame_seeds`/`frame_guys` on
`init.checksums.last().seed`, but borrows the **setup checksum trace
itself** ungated (`if init.checksums.is_empty()`), which makes that gate
compare a borrowed word with itself. Harmless where the donors are the
same game (run6/7/9); not harmless for the golden record, whose GAMEINFO
is **byte-identical to run11's** — map 14, seed 12345, size 2, the same 34
option fields — and whose setup word is the same `1003723497`, because the
script's first line runs at frame 0's `do_frame` entry, *after*
`Game::init`. **No gate on the map, the lobby or the setup word can tell a
staged run from an unstaged one; only the caller knows.**

364's runner does the right thing by hand — keeps the borrowed setup trace
after asserting its last word equals the trace's frame-0 entry word
(`0x3bd39ae9`), and refuses the per-frame records. The guard shape is to
make that the only possible behaviour: a golden run declares itself, and
the borrow refuses per-frame records from a run that did not. Written up
because the next golden capture will hit it and a silently-passing
comparison is what it looks like. Not urgent — see (376) for why.

(251) **`memcap.sh` has two doors left and half a fixture**: the refused
sample that read as zero is closed (exit 125, lab L25) and the mode-644
door closed 09-07. Left: it takes a **GiB cap as its first argument**; and
(281) it read `ps rss`, which over-counted the mapping 280 removed — ~1.3 GB
of 260's 8,816 was run58's clean text — and a 2 s poll under-reports a
sawtooth. Still wants the fixture with teeth: past the cap, dead in N
seconds, exit 137.

(335) **The commit trailer names the model the commander expected, not
the one that ran.** This repo reads the `Co-Authored-By` trailer to answer
"who wrote this tranche", while `CLAUDE.md`'s fan-out rules say to verify
the model **from the transcript, never from the spawn parameter**. Both of
the trailer's feeders are the forbidden source:

- **The commander dictates it in the brief** — eight of twelve transcript
  hits are worker prompts reading `Commit on loop-NNN with the trailer
  "…Claude Opus 5…"` (loops 219, 227, 228, 234, 235, 236, 7448,
  att-capture), written before the worker has made one request.
- **The harness's attribution reminder is not the session's model** —
  measured 2026-09-17, it said `Claude Fable 5.1` on a `claude-opus-5`
  session, and the day's first commit went out wrong.

**No realised wrong trailer is known here**: loop-329 ran `claude-opus-5`
over 201 requests and its `4fb593b` says Opus. Latent, and it bites the
first time a commander is wrong about what it spawned. Two smaller shapes
ride along: **one model, two strings** (356 commits `Claude Opus 5 (1M
context)` against 123 `Claude Opus 5`) and **149 of 772 commits untrailered**,
46 of them merges. A guard can compare a commit's trailer against what
served the session that wrote it — the first-request source `lore spawns`
already trusts. Guard, hook, or "a brief never dictates it": the pass's.

(341) closed 2026-09-18 by item 363: `live_session.stage()`'s window and
categories are caller-supplied, the golden record's first run used them, and
`docs/RUNS.md` run101–run105 is the evidence. The lane's real constraint was
never the cursor — it is that a launch from inside Claude Code's own process
tree gets no window at all (`nodrv_CreateWindow`, dead in 3.8 s, 0 frames),
so every launch goes through `viadriver.sh` (`docs/ORACLE.md`, "The
click-free lane needs a window").

(313) **The landing chain wants one verb.** Merge, gate, push and reap are
one chain by rule since 09-17; ccc has `merge`, `update` and `clear`, and
the reap is a separate command a commander typed after the chain twice and
forgot twice. Filed with ccc as `land <ref>` or `merge --reap`; until it
exists, the chain is one shell line in the commander's brief.

## Parked by the seventh Fable pass, 2026-09-21 — two words without a widening

(444) **East Indies' word has no widening on file.** `LONG_WORD_EAST_INDIES`
is 9,711, and the last capture on that map compared whole (run90) sits at a
word two thousand frames lower; `rondata::diff::testkit::WIDENINGS` says so
and the guard behind it reads this row. Parks rather than queues because the
map is not the headline — lower map first, DECISIONS 41 — and it is the
first item the AI track takes the day East Indies becomes the lower word:
every record the long capture's game dumps around 9,711, both directions,
before any mechanism.

(445) **Chapter one's word has no widening on file.** `GOLDEN_WORD_CHAPTER_ONE`
is 626 and none of the tests behind it is a whole-cast `compare` over the
word's own frame — they pin the seating, the reach, the hit and the
hand-off; `WIDENINGS` says so. Parks because chapter two's 616 is the lower
chapter and the rules headline; it is the first rules item the day the
chapter two word closes, and it is one probe over
`crate::diff::harness::compare`, the shape 441 used.

## Parked by the sixth Fable pass, 2026-09-19 — a guard's first run

(440) **9582, one `use_market+0x1ed` short.** The AI headline's next
parting, measured by item 432, 2026-09-19, under the probe that gives the
Scholar its two per-city values: ours spends **one** `Leader::use_market
+0x1ed` where the original spends **two** — the market's sell rotation,
the same site as 9382's agreeing pair. Everything else on the frame matches
entry for entry (`make_stuff+0x221` ×2, `+0x63d` ×2, three `set_anim`, two
`Ammo::init`, `Farms::inc_time`), and 9581 and 9583 agree whole.

Parks rather than queues because **it is not reachable on the real tree
until `create_units` computes per-city** (438). It is booked by its frame
and its draw delta with no mechanism named, and it replaces 9518's scholar
birth, which was probe 4's artefact through and through.

(429) **A positive i32 wrap is live wherever `create_units`' tail is
reached.** Measured by item 422, 2026-09-19, on frame 9382:
`out = wm(fac, wm(want, val) / divisor) / 256` with `val 42,000,000`,
`fac 256`, `want 20`, `divisor 25` gives `256 × 33,600,000 =
8,601,600,000`, which wraps as i32 to **11,665,408 — positive** — and
divides to exactly 45,568. The original's product stays inside i32
(`5,755,741 × 256 = 1,473,469,696`) and does not wrap. **The sign is the
defect**: §45 documents this same overflow in the arm next door and guards
it with `out < 0 → 9,999,999`, so a wrap that goes negative becomes the
ceiling and is caught, while one that goes positive becomes a plausible
small number that ranks below a Citizen and is not. 422 pins the wrap as
an assertion on its own frame; this parked the general case as a defect.

**Corrected in place, 2026-09-19, by 430's second probe — the wrap is
fidelity and this item was wrong.** Computing the tail in 64 bits so
nothing wraps costs **2,727 frames**: the word falls 9510 → 6783, because
the original is a 32-bit engine whose own `imul` wraps and this crate
tracks it that far *because* it wraps too. So the overflow is not a
defect; it is a wrong input carried faithfully into a wrong answer, and
the lever is entirely upstream in `val`, `want` or `divisor`.
`offer_value`'s doc comment now says so with the number beside it, and
`LONG_WORD_GREAT_LAKES` pinning 9510 means a well-meant "fix" of the
overflow **fails the floor** rather than passing quietly.

What survives is narrower and still true: **`out < 0 → 9,999,999` catches
only half of a wrap's range**, so a positive wrap is invisible where a
negative one is caught. That is an observability property of 32-bit
arithmetic the original shares, not a defect in the expression, and where
it matters is a question nobody has asked yet.

(427) **Twenty-four frames pass before chapter two's first arrow.**
Observed by item 415, 2026-09-19, and **free to measure from run112, which
is already on the disk** — no capture, no screen. Parks because it names no
score: it is a delay this crate and the original may well share, and
nobody has compared them. Worth an hour the day a combat word lands near
it, and worth nothing before that.

(425) **A name a pinned constant carries for an index has nothing to
disagree with.** Named by item 408 while settling 423, 2026-09-19, and it
is the reason that defect survived from run19 to today. Both sides of the
leader diff generate their key from the **same** `GOODS` array —
`rows()` at `leader.rs:87` and `theirs()` beside it both `format!` the name
in — and the dump prints `bucket` as six bare values, so the index is
positional on both sides and **the label never participates in matching**.
The pinned literals in `PARTS_ON_RUN19/107/111` then check only that the
string matches whatever `GOODS` currently says. So `GOODS` and its pins can
be wrong **together, consistently, forever**: the specifications got it
right, the decompile export got it right, and 858 tests never looked,
because a self-consistent label has nothing to disagree with.

The guard is the check nobody wrote, and it is deliberately **wider than
`GOODS`**: every index-to-name mapping the diff layer pins is compared
against the type record, `GOODS` being the first instance. If others exist
it finds them the day it lands; if none do it costs three lines and says
so. A worker's item, not the pass's — a correctness guard about the port
rather than loop machinery. Make it fail on purpose first.

**Its sibling already exists, which is the argument for building this
one.** Item 422 wrote `assert!(45_568 < 234_782 && 234_782 < 5_755_741)`
— three literals agreeing with each other, unable to fail — one commit
after landing 423, and clippy's `assertions_on_constants` killed it in
ninety seconds because the gate runs `-D warnings`. That class of
decorative check has a working guard. **The label class has none**: no
lint knows that `GOODS[2]` should read "wealth", which is why one sat
wrong from run19 until someone traced a number to the array that named
it. Nothing to build for the assertion case; this item is the gap.

**Measured 2026-09-19 by 423's own fail-on-purpose**, which is what tells
this apart from a load-bearing label: with one row left on the old name,
every value comparison passed — 33,536 field-frames — while the `parting`
equality failed in **exactly one position**, `income[2:wealth]` generated
against `income[2:metal]` pinned. The pin guards the literal against the
generated name; nothing guards the generated name against the truth.

**The same shape one level up, and 423 found it in its own writing**: its
`docs/AI.md` §48.4 said "three pinned constants across three items", an
estimate written into a specification without counting, and the count is
one pinned constant and one assertion key. A number asserted in prose has
nothing to disagree with it either — struck in place and pointed at the
measurement. Whether that generalises to a guard over counted claims in
`docs/` is the pass's to judge; it is named here so it is not lost.

(423) **`GOODS` is mislabelled in `diff/leader.rs`.** Found by item 414,
2026-09-19, while tracing what pays for a Scholar. Index 2 is named "metal"
and index 2 is what pays this Scholar's 40 **wealth**:
`sim::economy::Resource` has Wealth 2, Knowledge 3, Metal 4. **No verdict on
any capture is affected** — the comparisons are index against index, so
every number that has ever been checked was checked against its own
counterpart — but the *names* pinned at 2, 3 and 4 in `PARTS_ON_RUN19`,
`PARTS_ON_RUN107` and `PARTS_ON_RUN111` are the wrong resources, and a
reader who trusts them will reason about the wrong good. **Corrected in
place 2026-09-19, before the item ran**: the authority is the type record,
`rise.pdb`'s own `TypeIndex` — `0 FOOD 1 TIMBER 2 WEALTH 3 KNOWLEDGE
4 METAL 5 OIL` — which is `sim::economy::Resource` exactly; and the blast
radius is **six strings in one file, every one at index 2**, not three
constants across three items. There is no wrong row at 3 or 4, because
who=1's knowledge and metal never parted on those windows, and the
seventeen rows at 0, 1 and 5 are already right because those indices agree
between the two orderings. **No document changes**: `docs/AI.md:227` and
`docs/ECONOMY.md:861` already say wealth. Parks because it names no score;
a worker's to take, not the pass's.

(418) **Order coverage is a separate axis, and its unit is a native
issuer.** Found by item 365, 2026-09-19, while designing the golden
record's chapters. The cheat channel **stages state**; it does not order.
So seven of the eight new chapters add no order class the tree already
enters, chapter six adds one, and `bird` — console table case 82,
`Unit::add_air_patrol_order@005e4350` — is the only console command that
issues an order at all. "Every order class" is therefore not something the
chapters can deliver, and `docs/GOLDEN.md` §13 is the order-class-to-issuer
table DECISIONS 41 §5 was waiting for: coverage is counted in native
issuers reached, not chapters run. Parks because it names no score yet; it
returns when one names it.

(412) **Fourteen document-address pairs cite a dead-listed function.**
`docs_guard::a_dead_listed_address_is_cited_only_where_pinned` reads
`docs/EMULATOR.md` §4's fifteen and pins the citations standing on the day
in `DEAD_CITED`: `LeaderData::is_human@006ec170` in ARMY, COMBAT, GROUPS
and ORDERS; `find_wpath_army@00683730` in GROUPS and ORDERS;
`get_estimate@00688310` in PATHFINDER and RUNS; `locked_transport@006d5230`
and `find_dock@0065cfd0` in TRANSPORT; `set_domain@00633390` in COMBAT;
`add_attack_order@00622ce0` in ORDERS; `set_gathered_at@006b46b0` in AI;
`cos_table@00a469f0` in MOVEMENT. Each cites a standalone body the linker
kept while the live copy is inlined in a caller, so the claim rests on
whatever else backs it — run58's dump for `is_human`, a listing for the
rest — and the reconciliation is to say so at the citation and delete the
row. Names no score; a worker takes one document at a time, and the row's
deletion is the landing.

(413) **Seventy-seven constants the specifications name that no crate
carries.** `docs_guard::a_constant_a_document_names_is_built_or_pinned`
pins them by file in `UNBUILT` — AI 20, ORDERS 9, TECH 7, CITIES 6, GOODY 6,
ARMY 5, ECONOMY 5, PRODUCTION 4, GROUPS 3, and one or two in nine more —
and a file may only shrink. Three kinds, to be told apart one file at a
time: a field offset written without its `+` (spell it `+0x..` and the
guard stops reading it); a value named for context the code never needs
(say so beside it); and a constant read and never built, which is the
kind that cost a month at `0x66`. The third kind is an item on whichever
track its frame is on; the other two are a document edit and a lowered
pin. Names no score until a row does.

## Parked by the third Fable pass, 2026-09-17 — names no score

(310) **Nothing decrements the muster on death** — not `by_type`,
`by_group`, `control` or `active`. The original's `Unit::close@0060ee50:235`
undoes all three and this crate has no counterpart. No unit of players 0
or 1 dies in either window, so no diff reaches it, which is why 303 could
land `Sim::track_unit_type` correct in both directions of `set_type` and
still leave this open. Falsified by a window containing a death. AI §37.

(305) **The make list's five building values, and a factor of ten** — the
building producer's rows `MAKE[0]`..`MAKE[4]` still part, with `MAKE[2].cat`
beside them, and two are exactly ten times out: `MAKE[1]` 202500 against
2025000, `MAKE[2]` 165000 against 1012500. A different producer from 302's
`upgrade_units`, and the factor-of-ten shape is as strong an oracle as 302's
factor of two was. Value diff on disk, no capture needed. AI §36. Parked
because 302 and 303 closed make-list rows and moved no word.

(278) **`Unit::work@0060d180:440` is a second `set_new_location(…, 1, 1)`**
and nothing models it — two units of one type within `0x180` are pushed
apart by half their separation and **both snapped**, gated on
`field_0x82 < 0` and the order's `+0x30` vcall (read, never run). Ruled out
for 271's 6937; `Guy::last_pos` makes the signature searchable on every
dump: a unit that moved whose figure has `last_x == x`. run79's squad first.

(220) **TECH §13's twenty range blocks.** (209) **the once-per-game events a
dump install swallows** — a `set_*` whose **return value** drives an
irreversible record; **takes 224**. (224) **`mil_trainers`' other three
writers** (AI §29.4) — a trainer that changes city, upgrades in place or is
captured is filed by neither; referenced since 09-04, booked 09-17. (203)
`mark_behind_tiles`' `0x4` is a building *finishing*; (240) the merchant's
`gather_down`/`special` −1 at 6929, **takes 184**; (216) `create_buildings`
offers a gather building the original does not.

(242) **The crate's group id is not the original's** — `GroupData +0x4 =
64` against `group_id`'s stand-in, the army group's *slot* 1; the other
five agree on all 630 and 237 reports the row without scoring it. (243)
`UnitDump::group` is parsed and **nothing compares it**. (244) **1,428
grouped order records no test windows** — run79's 453, run31's 945. (245)
`focus.sh` matched a concurrent worker's shell on run84.

(255) **`go_to_unit`'s `0x480` and `go_to`'s `MOVE_TO` arm are read and
never run** — every joiner on disk is farther than `0x480` from its army's
`get_unit(0)`, so the falsifier is one born beside its army. (256)
`come_out`'s three `action_move_to` sites stay unreached; (254) its ring
has one sample (7284) and wants a second disembark. (167) run61's two (SYNC
§3.9). All four came off 253, which 271 closed.

(247) **An upgrade is an in-place guy-type change on the standing unit** —
run76's **6737**, three Archers going guy **170 → 177** keeping `(who, o)`
and `group 64`; East Indies' `1/32` does 340 → 341, the danger row moving
by `(110 − 100) / 2`. (181) CARAVAN §7.2–§7.3.

**The widening ledger** (87, DATALAYER §4, §4.1): **18** fields the harness
names nowhere, **46** one capture names — item 308 moved both on 2026-09-17,
`start_dist` off the uncompared list and `uid` onto the single-capture one; blind spot `avg_speed` (210) — it
counts *fields* and cannot see a record the parser never visits, which is
what hid 252's dumps. Uncounted: (88) the blind list — **802 cited, 650
entered, 152 never** since coverage came back 09-08 (ORACLE), not the 101
of 617 this line said for a week; (72)
every `+0xNN` a document pins vs its module; (89) the guard; (35) VISION §7.

(269) **The third axis: a field compared at the wrong width** (DATALAYER
§4.2, from 265). Ten `CityData` counters are `uchar` in the type record and
`i32` in `ai::CityAi` — `busy` and `gatherers` have bare byte writers of
their own — and `pop` is an eleventh on `sim::City`. None has been seen to
wrap; the falsifier is a capture where the sweep's count and the producers'
decrements cross zero. Same question one record up, for every
`char`/`short` of `LeaderData` held as an `i32`. A guard wants the PDB's
widths beside the sim's structs, not a grep.

(270) **The fourth axis: a word compared one bit at a time** (DATALAYER
§4.3, from 267). The ledger scans the differ for a field's *name*, so
`unit_masks` has been on neither list since the packed bit got a row while
one of at least eight modelled bits was actually compared — and the missing
one, `0x100000`, named Great Lakes' run-up cause on its first run. Nine
dumped masks want a per-**bit** census: `unit_masks`, `unit_masks2`,
`guy_flags`, `node_flags`, `city_flags`, `leader_flags`, `leader_flags2`,
`build_flags`, `role`. Same tool as 269.

(257) **Nineteen kept tests compare a torn block** (252): a closing dump is
frame n but for the one unit the quit caught mid-update, and every nested
archive's last `FRAME n` body *is* its closing dump. No live case found,
unaudited; fix is `compare_shutdown`'s n−1 allowance.

(175) **The uber chain past its birth**: `Objects::init_unit` threads
`uber_size` objects (CITIES §4.3); nothing else reads it. Takes (48)
COLLISION §3/§7, (73) `UnitData::group`'s back-pointer, (56) ARMY §13.

(161) **The make-list block is 2,500 frames behind East Indies' word**:
`create_buildings` first runs on 9982 (AI §25), so `building_value`,
`gather_value` and §24.4's arm wait on it. (195) `find_repair_spot`.

(211) **`get_speed`'s three remaining arms** (MOVEMENT): `unit_masks &
0x10`, set and cleared inside a frame; `has_general(0, 0x162)`'s siege
doubling; the group cap, gated on `action_type == 0` (219) — it wants a
grouped unit holding no action. (229) a figure in melee does not step its
clock — `unit_masks2 & 0x10` freezes `Guy::inc_time` (ANIM §5), and that
arm waits on a melee frame. **312 names this first** if the 7679 figure is
in melee.

(291) **The AI's caravans are not linked** — `vans.length` 0 against 1 and
`trade_val` 0 against 128 on **both** of player 1's cities, 246 of 246
blocks (287). That names two of 285's eighteen fields and gives them a
mechanism, and wealth is what a market buy spends, so it is a live
candidate for 290's shortfall. (288) the gull's `do_strafe`, unmodelled.
(274) run87's `1/26`. (268) `AnimalData::ox`/`whom`/`aid` carry nothing —
−1 on all forty animals on all 247 blocks. Closed as answered, not open.

(285) **The CITY record parts on every block and nothing asserts it** — 18
fields on 246 of 246 of run89's window, found by widening the whole record,
and **no window test on either map asserts `city_diverged`**. Two of the
eighteen are named now (291, the unlinked caravans). `1/3`'s `(+192, +192)`
`MOVEORDER` was booked here and closed itself when 284 landed the
air-physics fix.

(273) **`refresh_group_order` re-origins on the order's `form_id`, not the
member's list position** — `713ac3` reads `[eax+0x10]` off the `GroupOrder`
the `+0x94` vcall returns, where `Sim::group_refresh_order` uses
`g.list.iter().position(member)`. `do_group_move` step 2 rewrites `form_id`
every frame, so the two agree except where membership changed and the
follower arm has not run: wants a `GROUPS=1` window across a death or a
join in a marching formation. (275) `MoveOrder::facing` still does not
score; 267 split the two mechanisms, so re-read that. **304 is the nearest
live window** to this — a formation ending early on run76.

(315) **run90's `1/7` names no blocker on 7820** — `collide_o` and
`collide_who` read −1 where the original has 6 and 1, with `collide`
itself, the count, agreeing. New on 2026-09-17 with the suspend wiring
(items 301/304), and the window cannot price it: every position, every
angle, every order record and every draw count over those 111 blocks
agrees, and the two Merchant constants are the only parted units left.
Pinned in `run90_s_window_is_east_indies_shuffle` so it cannot move in
silence. Parked because it names no score — the falsifier is a capture
where a missing blocker identity changes a decision.

(316) **Sixty-two other callers of `clear_partial_path` are unchecked**
— item 304 fixed `kill_current_path`'s and no capture reaches the rest.
The `Group::action_*` and `think_carry*` families have **no counterpart
call in this crate at all**, which is the shape 304 turned out to be, one
level up. Parked because it names no score; it comes back the day a
window contains one of them.

(325) **`MAKE[*].city` is ours + 1 on every offer** — this crate's city
array puts the human's at index 0 and the AI's at 1 and 2 where the dump
reads 0 and 1. No offer in run19's window is chosen by the index, so
nothing scores it; the falsifier is an offer whose choice depends on the
city. Item 323, AI §38.

(326) **Three tech `val`s part from before run19's window** — Empire
2,100,000 against 1,800,000, Mercenaries 165,000 against 216,000,
Mathematics 82,500 against 63,000, all `research_techs`' arithmetic. A
value diff on disk, no capture needed; parked because the window that
prints them is not a scoring one. Item 323.

(327) **The Merchant is still not offered at slot 3** (`t 61`, `val
869,565`) — `civilian_value`'s merchant arm reads `known_rares`, which is
still a summed-region seam. Item 323, and a sibling of 291's unlinked
caravans.

(333) **`1/28`'s path stack is 41 where run19 says 42** — an off-by-one
already present at 8186 and belonging to `find_wpath`'s plan near the
goal, found by item 329 and deliberately not closed by it. The chase and
the delay are right either side of it, so nothing on the word depends on
it; the falsifier is a plan whose last leg the count decides. PATHFINDER
§21.

## Parked from the lab, 2026-09-17

(309) **`find_upath`'s pre-walk give-up exit targets the wrong label** —
`00683082` is push-and-return-length where this crate's `break` falls
through to the near test and the search. Unreachable today for a
transport-capable unit and no capture reaches it, so 301 recorded it in
PATHFINDER §18.4 rather than changing it. Parked because its falsifier does
not exist on disk.

(311) **The zero-pop predicate disagrees on paper and nowhere else** — the
original counts a `control_cost == 0` unit only when `is(0x134)` or
`is_gov_hero`; this crate's muster seams apply no test at all and its sweep
skips every zero-pop unit outright. Two different wrong answers that agree
on every capture, because nothing on disk separates them. AI §37. Parked
with 309: a reading-only disagreement with no falsifier.

(298) **The click-free capture lane starts 6 of 10 pairs** — a
`RON_AUTOSTART` tracer build replaces the two menu functions and clicks
Start from inside the modal loop; every success matched 19 logged bodies and
1,401 frame seeds against a hand-driven run. The failures are 180 s timeouts
and exit 40 before the menu; two faults map to the WoW64 transition RVA
0x1139, and moving the hooks off bulk restores (226's family) did not cure
it. The factor-isolation experiment is paused
(`docs/lab/2026-09-09-paused-runtime-experiment.md`). Falsifier: a ten-pair
cohort at 10/10. Lab rows L18, L19, L28, L29.

(299) **A single Gaia reseat correction hides a 556-frame heading
difference** — run69, Great Lakes: of seven actual reseat writes across
three intervals, removing tick 99's leaves this crate's Gaia heading off the
original's for 556 frames, only the control matches at 106, and all seven
converge by 3001 — invisible to the player comparator throughout. Lab rows
L53, L54, `docs/lab/2026-09-10-single-reseat-interventions.md`. Wants a
Gaia heading row in the differ; unscored today.

(300) **The original's command API is unused, and the replay adapter lowers
only `MoveTo`** — `CommandManager` has 68 `issue_*` entries;
`CommandManager::issue_move_to@00941720` builds the command and appends it
to `local_package` after `CommandManager::check_accept_issue@00940a70`,
while `tools/fuzz/scenario.py` teleports. On this side `input::Stream::one`
applies selection and `MoveTo` and skips production, repair and market. Lab
row L08: acceptance is not emission — package capacity can refuse after
selection changes — so an adapter needs acceptance, emission and
processed-frame witnesses. A prerequisite for manufactured falsifiers and
phase 5, not headline work.

## Parked from the queue, 2026-09-07

(277) **An age gained through a cascade is unmodelled** — `Sim::gain_tech`
reads the age gate off the type the call was made with, as the original does,
but the original **recurses into `gain_tech`** per cascaded grant where this
crate flattens the cascade into events, so a cascaded age would take 271's
snap arm there and not here. Parked to seat 290/291 under the cap, on the
ground that **no capture on this disk gains an age other than directly**, so
nothing can falsify it today. It is cheap to unpark: `BuildDump::max_age` is
parsed, so the falsifier is a grep rather than a reading — a block whose
`max_age` moves on research that is not one of the seven ages. Cross-item
constraint, and the reason this is not a silent drop: **271's snap arm is
live code**, so anyone touching `Sim::gain_tech` inherits this question.

(248) **ATTRITION's Territory section is 15,625 bytes of 16,000** and wants a
retelling pass before anything is added. Parked to seat 283; its remaining
terms are **closed** — handicap, temple and fort are unreachable in any game
that runs here — so nothing is waiting on it and the retelling is owed only
to whoever next adds to that section. Read it before re-booking any of the
three. (AI §2.1's `check_explore`.) No cross-item constraint: the size ceiling
is per-section and `docs_guard` enforces it, so a worker cannot trip over this
without being told by the guard itself.

(234) **Four rules of the turn/idle animation neither side has** (ANIM §9),
written and never landed — branch `rescue-234`. Parked by the commander to
seat 276–279 under the item cap, on the item's own words: the limbs move no
score, since §4.7's turning types are in neither scored game. **The two
static assertions are the cheap half** and are what to take first if it
comes back. It carries the ledger's nine `unverified` rows on the way, so
whoever unparks it inherits that audit. Nothing here is a cross-item
constraint: no live worker is in `crates/sim`'s animation code.

## Steering candidates, booked 2026-09-07 with Ramon

Neither is a mechanic and neither moves a word, which is why both park
rather than take a slot under a cap that stood at 18 of 18 the day they
were booked. Ruled by the second Fable pass the same day
(`docs/audit/2026-09-07-fable-pass-2.md`): 293 closed there, 292 stays,
and 283 came here from the queue to sit behind it.

(292) **Nothing in this tree has ever been profiled** — no bench target, no
criterion, no flamegraph, no samply, and no mention of Instruments anywhere
under `docs/`, `crates/` or `tools/`, grepped 2026-09-07. Every measurement
the gate has ever been given is **memory**: the 15,128 MiB peak at two
threads, the memcap in front of it, and the ratchet items 235, 260, 280 and
283 that came off it. Its **256 seconds** is a wall-clock fact with nothing
under it — no split between parsing a dump and comparing it, which is the
obvious first cut when run89 is 121 MB and run90 72 MB of text. That
ignorance is upstream of more than the gate's runtime: 283 proposes a
`#[global_allocator]` on reasoning about what macOS's allocator keeps, and a
profile is what would settle it rather than argue it. Cheap to start —
one bench target over the differ's parse and one over a sim tick — and it
earns its dependency the moment it prints a number nobody predicted.
**Ruled 2026-09-07**: stays parked with this first cut; the gate's five
minutes is not what the loop was losing time to (293), and the item is
taken by whichever worker next touches the gate's runtime, or with 283.

(283) **The ratchet wants a `#[global_allocator]`, not a mapping** — 280's
+1,563 MiB is *not* live data: a mapping is `munmap`ed at drop, while a
freed `String` of a capture's size is kept by macOS's allocator and cannot
serve the next capture's different size (260's own 5,332-MiB-with-nothing-
alive probe, again). An allocator that returns large blocks recovers most
of it with **no `unsafe` in this tree** — the crate carries it — and closes
235's ratchet for every large owned buffer, not the two that were mapped.
**Parked by the pass behind 292**: the retained memory is not live, not a
score, and not a hazard behind `memcap.sh`; a dependency is earned by a
measurement, and 292's bench is that measurement.

## Parked at the 289/290 merge, 2026-09-07

(296) **Great Lakes' endpoint is eight units short and nobody knows which**
— 290's `get_mod_resource_cap` fix halves the AI's commerce cap on this
lobby's Easiest, which is the right-hand side of every `income < cap` gate
in `create_buildings`, `create_units` and `research_techs`. It took 33
spurious units off the four endpoint rows and on Great Lakes went eight
past the mark: `(1,73)`-`(1,80)`, the last eight object numbers the AI ever
reaches. run53's endpoint is MISC-only, so their positions are known and
their types are not. **The cheapest falsifier is already on disk and was
not spent**: run80 is Great Lakes `LEADERS=9` over [23960, 24000), and that
record carries `num_units` and `num_queued`, which would name exactly which
types the AI is short of. Parked rather than booked because the queue stood
at 18 of 18 and this sits 16,400 frames past a word neither map moved, on
streams that are nobody's. What keeps the change that caused it is that its
own evidence is local and strong — twelve lines of decompile at 006d65b0,
and `rate[0]`, `rate[1]` and `best_good` going from wrong on all 80 blocks
of run84 to right on all 80. Expected in direction, unchased in size, and
290 declined to call it clean rather than papering it over.

**Two senses of one word, kept apart on purpose.** At the endpoint,
`unlinked` is a `(who, o)` the dump has and the simulation does not — a
unit the original built and we did not. In item 291 it is a caravan not
linked to a city. Same word, different counter, and blurring them would
make either number unreadable.

## Parked 2026-09-18, the two-lane session

(372) **The held-out map's 1 is comparable in kind, not in provenance.**
run106 ran on the click-free lane's lobby, not run33/run39's `-config
check.ini`, and its window opens at frame 1 rather than matching the score
runs' shape. Strict comparability wants a run33-shaped capture on map 9 —
a second capture and a second decision, which is why 363 left it. Park,
not bury: the 1 is the generalisation number and the first pass that
reads it should know exactly how it was taken.

(373) **The click-free lane's give-up truncates rather than stops.**
`--timeout` defaults to 180 s and the first held-out attempt returned 593
of 1,900 blocks in a **62 MB file that reads as completely ordinary** —
only `receipt.json` said `success: false`. The silent truncation is the
finding, not the timeout: every check that greps a dump would have passed
on it. In `docs/ORACLE.md` beside the new knobs. A guard shape: a capture
asserts its own block count against the window it asked for.
**Half-closed 2026-09-18 by item 369**: run107's stanza is the first in
`tools/gamelog/captures.txt` to carry that assertion (30 blocks,
9170..9199, no gap). Making it every capture's is still open.

(376) **The sibling borrow is measured harmless on every scored path, and
this is the record of it** (item 364, 2026-09-18, the negative the
commander asked for before pinning). Five Great Lakes captures — run33,
run53, run89, run97, run100 — take 14 `frame_seeds` and 14 `frame_guys`
from run12/run13 at frames 0–3 and 94–103. East Indies takes **nothing**:
setup word `793793043` never matches the donors' `1003723497`. run53 walked
all 24,000 frames both ways gives **word parts at 9362, sequence at 9182,
identical with the borrow and without**; run33's floor is unmoved between
the two columns. The installs are no-ops — our sim already produces the
original's word at each of those fourteen frames, all at frame ≤ 103 and
9,079 below the word. Structurally, `Built::tick` pushes the frame's
labels into `frame_sites` **before** installing anything, so an installed
frame's draw comparison is still the sim's own work; the install can only
re-anchor the next frame. **No pinned floor or word rests on installed
sibling frame data.**

(378) **Put the human back into `census_wars` and `census_strategy` too,
and the residue gets worse before it gets better.** 368 fixed
`census_territory`'s gate (the human is a leader: `plan_strategy@006b9620:159`
tests `leader_flags & 2`, `i != who`, `i >= 0` and nothing else, and on a
one-human-one-AI game the human is the only other leader there is);
`1/other_team_terr 0 theirs 266` is `0/my_team_terr 266`. The same defect
sits in the other two loops, but fixing them makes all five of
`weight_total`'s census facts agree and takes run19's leader residue from
**90 fields to 147** — the make list then parts on `t`, `cat`, `city` and
`val` across nearly every slot, because `active_wars != 0` reaches the
danger word and the region-strategy words as well.

The number is in the row on purpose: a successor that makes a residue
worse before better does not get taken unless its cost is visible.
Carried with it: the original runs a **full census for its human leader**
(`0/gatherers 5`, `0/peasants 5` in every dump) and this crate runs none,
so `census_strategy`'s `weaker` test reads the opponent's `attack` as
nought even once the gate is fixed. That is the part to build first.

(380) **A bare coordinate on the staged channel is a TILE**, `n × 0xc0 +
0x60` — measured by item 364, not read: `add hoplite who=0 4,40` put the
squad head at `(888, 7800)`. `docs/ORACLE.md`'s stored `arg × 768 + half a
footprint` is corrected for the staged channel, and 363's open "the
coordinate argument did not calibrate the way the runbook says" closes.

(383) **run107 opened two families never compared on Great Lakes.**
Player 1's ten `SITE` slots part as a **re-ordering of the same ten
sites** — our `SITE[3]` is their `SITE[1]` and so on, with `rank` in the
residue on every slot — and `tech_frame`/`tech_cat_frame[0..3]` sit at
nought against 8382/4976/8382/8182/6376. Both are inside
`run107_s_window_is_the_leader_record_at_the_word`'s 109 fields, so they
are floored and cannot regress silently; neither is on the word's frame,
which is why they park rather than queue. The re-ordering is the more
interesting: the same ten sites in a different order is a comparator or an
insertion order, not a missing mechanic.

(387) **Target selection should walk the world cell's object chain, and
ours is units-only.** Found by item 384 while closing 617's range half.
The crate already maintains the chain; switching the scan to it would
**drop buildings from target selection**, so it is a bigger item than it
looks and it would not have moved this frame. Parks on that second clause
— it names no score today — but it is the shape of a whole family, and
whoever takes it should expect the building half to come with it.

(388) **The golden record's `1/7` and `1/8` get their target from a group
path the sim does not have at all.** 384's other parting note: `near_o`
shows only a squad's captain searches (`1/7`, `1/8` and all of who=0 are
`-1` for 900 frames) and `Group::target_opportunity` spreads the captain's
find, which this crate does not model. Documented in `docs/GROUPS.md` §13.
Not on 617's frame — 386's tie-break is — so it parks until a word names it.

(393) **who=0's members take the captain's order a frame after it** —
`docs/GROUPS.md` §13's mirror of the thing item 391 closed on the other
side. 391 established that `Unit::target_opportunity@005fffc0`'s opening
`while` walks `o_up` to the victim's **captain**, and that the army rather
than the group moves a struck unit (`Armies::emergency` → `Army::process`
→ `Group::action_siege_attack_to`; `Group::target_opportunity`'s branch is
guarded by `type +0x2c8 & 0x10000 == 0` and never runs here). Dump-backed:
at the end of 616 the ATTACKORDER is on `0/6` alone, `0/7`/`0/8` a frame
later. Parks because 392's ring is what stands on the word's frame; this
is one frame below it and will likely fall out of the same model.

(400) **`0/8` plans its chase at 619 to the target's own point** where
the dump has (1224,8280) at 618 — item 395's second successor, dump
coordinates in `docs/COMBAT.md` §20.4. Below the golden word (624) rather
than on it, so it parks; likely to fall out of 399's `Army::process`
model, and worth re-measuring before it is taken.

(401) **Two reading-only residues from 395**, `docs/COMBAT.md` §20.3 and
§20.5: `find_melee_target@005ff9c0` carries its **own copy** of the
non-captain mirror, unimplemented here, whose only reachable caller is
`docs/GROUPS.md` §10's per-member call; and `think`'s shared epilogue at
`5f761a` clears the "could not reach" bit, which this crate never clears
at all. Neither is diff-backed — a run has reached neither — so they are
exactly the kind of claim the coverage section must list as reading-only,
and the kind a blind second reading is briefed with.

(403) **The launch table has one piece missing, and one capture closes
it.** Item 396 measured every release event `unit_graphics.xml` gives the
Longbowman — six rows reproducing all nine launch points to the unit
across three units and two facings — but run17's three Slinger families
are **measured and unusable**, because run17's `GUY` detail is 1 and
nothing there names the animation. A **`GUYS=4` re-run of any Slinger
fight** closes it, and the same shape closes every missile type in the
game. Cheap, and it generalises further than the item that found it.

(407) **A pool dump needs its INHERITED category set high enough, and
"DEATHS off" is the wrong rule.** Item 399 lost a `GROUPS` pool that came
back empty with `DEATHS` already off. `GuyData::log_data@005de6c0` sets
type `0x14` and then walks `set_detail(1,2,3,4)`, and **`set_detail` runs
whether or not the line is accepted**, so the pool inherits GUYS at detail
4 and `check_accept` drops it at `GUYS=2`. The rule is that the inherited
category must be set **at or above that dumper's highest `set_detail`**;
run31 and run92 satisfied it by accident with `GUYS=9`. In `docs/RUNS.md`
run110. Riding along: `tools/explore/golden_capture.sh` cds to
`tools/explore`, so `--cmd-file` must be absolute.

(409) **The ammo pool's order is not this crate's, and a two-landing frame
would spend the draws in the other order.** Found by item 402, 2026-09-19,
while widening the `AMMO` record. `Objects::add_ammo@00658b10` scans its
pool from slot 0 and takes the **lowest free slot**, so the original's
order is slot order with reuse — measured 0, 1, 2, 3, 4, 0, 1, 2, 3 over
run109's nine arrows. `Sim::projectiles` is a `Vec` that
`process_projectiles` `swap_remove`s from, so from the first landing it is
neither: at 9453 it holds a4, a3, a2 where the original's pool reads a2,
a3, a4. **Nothing in run109's window turns on it** — no two arrows land on
the same frame after 9451, and landing is what draws — which is why it
parks rather than queues. It becomes an item the moment a frame lands two.
A slot pool is the fix and it could move the word **in either direction**.
`docs/COMBAT.md` §24.4 names the rule and the consequence.

(410) **Three `AMMO` branches with no capture behind them.** Also 402. All
183 blocks print `traj 1`, so the parser's nested-`SplineData` skip — the
one written by indentation because the log writes no `END` — rests on a
hand-built fixture; **a catapult, or anything whose `ammo_path` is set**,
would make it evidence. Alongside it: `flags` bit 1, `rolling` and
`start_roll_angle` have no behaviour behind them, and nine allocations
from a cold pool cannot say whether `graph_index` wraps. Every row here is
one Longbowman shooting one farm on one trajectory, and each of the three
wants a different shooter rather than a different frame.

## Measured residues, none near a word

(371) **`market_speculation`'s two passes fire nowhere below the word.**
Read whole by item 362, both arms' predicates recorded in
`docs/journal/2026-09-18-item-362.md`. It was read because 362 was booked
as the market's and was not — `use_market` implements the decompile line
for line, and `MakeObject.num` was the cause. Parks because neither arm
fires below either map's word: it names no score, and the reading is
banked rather than lost.

(246) step 6's repath rests on run83's single event (239); (169)
`compute_site_stats`, 7,122 of run63's 27,000 site fields; (172) the
`bucket` pair on 5002/5061; (158) the human-leader sweep (AI §23.1); (159)
the `CITY` record's two seams; (146) `train_time`'s nine national arms;
(142) `World::tregion` (PATHFINDER §15–16); (122) 16 of 61 draws without a
`self.mark(`; (153) `TRIBE`; (20) the `Census` rows; (23) the formation
byte's sign, Echelon half (GROUPS §6.4), run52 the blocker; (103) a
woodcutter's clock, 445 v 480 (ORDERS §6.4); (105/45) Gaia's positions,
first bad 1658 (SYNC §4.2); (124) the loop flag is per animation file;
(116) the one `SITE` slot (AI §18); (166) `resource_cap` on five goods
(ECONOMY); (107) `epoch[0]`.

## Older backlog

(39) a 2D viewer over `Sim`; (41) `scenario.py`; a `find_target` block;
run7's orders; a mounted attacker; `calc_gather` non-flat;
`Leader::diplomacy`; (77) ANIM §3.2; (58) ROADS §7.4.
306. **The `city` column is one high on every make row that carries one** —
    ours 1/2 against theirs 0/1, on all 86 blocks of **both** windows, so an
    off-by-one in whatever city index `make_me` is passed. Unrelated to any
    value, which is what makes it separable from 305. AI §36.

307. **`age_p`'s zero-age arm is unexercised** — no type in the shipped tree
    reaches "a predecessor of age 0 leaves the walk looking" on either
    window, so that half of 302's fix rests on the PE listing and not on a
    diff. It is a reading-only claim in a document otherwise diff-backed,
    which is exactly what the coverage sections exist to flag. AI §36.4,
    §36.6. Falsified by a capture where a zero-age predecessor exists.

