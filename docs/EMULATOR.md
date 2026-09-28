# The emulator rung — the original's functions, called

*Established 2026-09-01 in the tools lane (Fable, from the lore session's
spike charter). One session; measured where it could be, counted where it
could not. Confidence: high on what was run, and section 6 lists the rest.*

The oracle ladder had static reading, the loggers' per-frame dumps, the
draw-site trace, and — since run55 — the call proxies, which log the
arguments and the answer of a named function **for the calls the game
happened to make**. This rung is the other half of that: enter a function
of the shipped executable with arguments *we* chose, outside the game, and
put its answer beside this crate's. It reaches what no capture's arguments
reach — the ends of a range, a sign, a width — and the first run found one.

## 1. What it is

`tools/emu/callfn.py`, run as `uv run tools/emu/callfn.py …` — the one
third-party Python package under `tools/`, `unicorn`, is declared inline in
the script (PEP 723) and resolved by `uv` (a wheel; six seconds, nothing
installed into the repository). It maps the executable's
sections at their preferred base — the bottle does not relocate it either;
`tools/trace` assumes `0x400000` throughout — resolves no import, builds a
stack with the arguments and a sentinel return address, enters the function,
and reads `eax` at the sentinel. An access to unmapped memory stops the run
and names the address, which is the price list for the next rung: the state
a function reads that this script did not lay out.

`sweep` prints one row per call, `<name> <args…> -> <eax>`, over a seeded
sweep; `crates/sim/src/path.rs`'s
`the_emulated_original_agrees_on_every_row` reads that table — from
`$RON_EMU_TABLE`, or by running the script against the install — and
asserts every row against this crate. The table stays outside the
repository, like the captures: it is derived from the user's own executable.

## 2. What it costs, measured

| function | state to synthesize | cost |
|---|---|---|
| `vector_dist@0046cff0` (ecx, edx) | none | 3,516 calls of it and `get_estimate` in 0.17 s, script load included |
| `get_estimate@00688310` (`ret 0x24`: two 16-byte `PathData` and a step) | none | as above |
| `GuyData::turn_speed@005de340` — read, not built | `units`' per-player band (stride `0x1c`, pointer array at `+0x10` indexed by `o`) → the unit → its type at `+0x18` (`+0x2c4` turn rate, `+0x304` guy count), the unit's masks at `+0x68`; `GameAccessConst::constantsc` (`+0x8`, `+0xc`); eight `GuyData` fields | about twelve fields, each an offset the type stream gives: an hour for one function, and `crates/sim`'s port already takes a flattened view of exactly these |
| `GroupData::get_stance_option@0070bab0` | the above, plus `malloc`/`free` and a vtable call | an import stub per DLL entry, and the object graph |

So the rung is free for a pure function, an hour per function for one
that reads a singleton chain, and a small PE loader's worth of import stubs
before anything that allocates. The Rust binding for unicorn was not
earned: the script is the tool, and the test shells out to it under the
same skip-if-absent rule the captures use.

## 3. The first target, and what it found

Two pure functions with ports in `crates/sim`: `vector_dist`, the integer
hypotenuse every distance in the simulation goes through, and the A*
heuristic `get_estimate`. The sweep is the fourteen edge values
`0, 1, 2, 3, 46340, 46341, 59999, 60000, 60001, 65535, 65536, 100000,
153600, 200000` crossed with themselves, plus 2,000 seeded random pairs at
three ranges, and 400 of those pairs through the estimate at each of the
three steps.

**The catch.** The original computes `hi + lo² / (2·hi)` in `unsigned`;
`world::vector_dist` squared `lo` in `i32`, which passes `i32::MAX` from
`lo = 46341` — a debug panic, or a wrapped, wrong distance in release —
where the original's own guard on that product only starts at 60,000. For
`(59999, 59999)` the original answers 89998 and this crate did not answer
at all. Fixed in `world.rs` the same day, in `u32`, as the original does
it. **No score moves**: the captured maps are 60 cells square (46,080
units), and the shortest leg of a diagonal on them never reaches 46,341.
A larger map reaches it; `docs/ATTRITION.md`'s paragraph on the guard was
rewritten, since it had called the branch unreachable "in tiles".

It is the recurring bug shape in miniature — the arithmetic was read
correctly and the *width* was not — and it is the kind a diff against a
capture cannot find, because no capture's arguments go there.

## 4. The blind list, re-read from the image

The charter's premise was that counter 2 — the cited functions no traced
run has entered, **97 of 659** on 2026-09-01 over 45 traces — shrinks under
emulation. Two scans over the image say what the list is made of:

- **15 are dead.** No `call`, no `jmp`, and no embedded copy of the address
  at any byte offset of any section — so no vtable, no dispatch table, no
  `push imm32`. Nothing can enter them, and no capture ever will:
  `Setup::init_wild_life@005aac70`, `Setup::build_leader@005abc70`,
  `UnitData::can_gather@00608850`, `UnitData::turn_speed@0060a600`,
  `Build::add_attack_order@00622ce0`, `Build::update_max_gatherers@00623310`,
  `BuildData::max_gatherers@00630590`, `BuildType::set_domain@00633390`,
  `ObjectsData::find_dock@0065cfd0`, `PathFinder::find_wpath_army@00683730`,
  `PathFinderData::get_estimate@00688310`, `World::set_gathered_at@006b46b0`,
  `LeaderData::locked_transport@006d5230`, `LeaderData::is_human@006ec170`,
  and `cos_table@00a469f0`, ~~which is data~~ which is code, a two-instruction
  thunk into `sin_table` (item 940's listing). `get_estimate` and `turn_speed`
  are the pattern: the body the document read is a standalone copy the
  linker kept, and the live copy is inlined in the caller — four times in
  `PathFinder::astar_path@00683770`, and as `GuyData::turn_speed@005de340`.
  A document that cites one of these cites code the game does not run;
  whether the inlined copy says the same thing is a separate reading, and
  `docs/ORDERS.md` already made that distinction for `MoveOrder::is_fleeing`.
- **12 are reached only through a pointer** — the order classes' virtual
  `clear`/`get_type`, `Build::finished`, `MapGrass::make_continents`, and
  the six `ScenarioFuncSet` script functions plus `rand_real`, which sit in
  the script interpreter's table and run when a script names them.
- **The other 70 are call-reachable**, and nearly all are the order and
  command handlers of things no capture has yet ordered — garrison, board,
  patrol, follow, repair, group attack, formation and stance changes. They
  come as families: one garrison order lights `process_*`, `add_*_order`,
  `do_*` and `Group::action_*` together. That is the targeted-capture plan
  entry 29 already names, and an emulator would need the unit, its order
  list, the group and the world synthesized to enter any of them.

So the emulator does not move counter 2 at all, and the counter has a
floor of 15 that no instrument moves: ~~it belongs in the enumerated,
deliberately accepted residue that `docs/DECISIONS.md` entry 29 describes.
The dead-scan is `tools/emu/`'s second product once it is reached again.~~
**It is there** (item 940, 2026-09-27): all fifteen, re-scanned dead, are
rows of `rondata::blind::RESIDUE`, along with the fourteen items 935 and 940
found (29). The dead-scan is `tools/trace/report.py <exe> refs`, not a
`tools/emu/` product. It also confirms each jump site against the
listing, and a test re-scans every "no reference" row against the install
(`docs/CENSUS.md`, "The blind list, 2026-09-27, item 940").

## 5. The verdict on the charter

**Reshaped, and kept small.** The interactive rung the charter asked for
mostly exists since run55: the proxies answer "what did this function
return" for anything a run reaches, in the game's own state. What
emulation adds is the choice of arguments, and that is worth exactly what
section 3 shows — a fixture rung for pure and near-pure functions, run as
a test — and no more: it does not read singletons for free, and it cannot
shrink a counter whose remaining entries are stateful handlers or dead
bodies. Not booked, with the reason: a PE loader with import stubs and a
Rust binding (nothing in the queue needs a function that allocates);
Ghidra's P-code emulator (unicorn under `uv` costs six seconds and needs
no project lock); an in-process Frida gadget (`tools/trace` already
patches the live process and proxies calls — a gadget would be a second
tool for the same job).

**What would earn the next reach:** a queue item whose open question is
the answer of a *pure* function over a range — a cost table, a damage
formula, a metric — where the reader's arithmetic is trusted and the width
or the sign is not. Add the function's address, convention and arity to
the script, sweep, assert.

## 6. What is not established

- `turn_speed`'s cost is a count of fields, not a build.
- The dead-scan finds a `call`/`jmp rel32` and any embedded 32-bit address.
  A target computed at run time (`base + index × size`) would escape it;
  MSVC does not emit function addresses that way, and every table-dispatched
  function on the list did show up as an embedded address.
- The sweep covers two functions and two calling shapes (register pair and
  `__thiscall` with a stack frame). A third shape graduates the encoding
  into a table like `tracer.c`'s `CALLS`.
- `dx = i32::MIN` is not swept: `i32::abs` panics there and the original
  returns arithmetic on `0x80000000`; no world coordinate reaches it.
- Whether the 60,000-unit guard is reachable on the largest map size the
  original ships — 78 cells on one leg — was not checked against the map
  tables.

## 7. Coverage

Diff-backed, by the emulated original itself: every row of the sweep, on
every `cargo test` run with the install present. Reading-only: the
classification of the 70 call-reachable functions into families (the names
say it; the traces of the next targeted capture will). The dead list is
neither: it is a fact about the image's bytes, re-derivable by
~~`emu-dead-scan` and pinned nowhere yet — pinning it as the floor of counter
2 is the queue item this document leaves for the main loop~~
`report.py … refs`, and pinned in `rondata::blind::RESIDUE` (item 940).

## 8. The packet rung, and the native twin (2026-09-23)

Two lab rungs merged on the eleventh Fable pass (`docs/DECISIONS.md` 47
§8; the evidence is `docs/lab/TYPED-STATE-REVIEW.md` and
`docs/lab/RECOMP-REVIEW.md`, and neither is re-derived here). They answer
the hour of synthesized state §2 prices: instead of building a singleton's
state by hand, take the original's.

**The packet.** A `RON_STATE_FRAME` build of the tracer
(`tools/trace/tracer.c`, mutually exclusive with the restore probe by
`#error`) copies the process's private data and main-image data at one
frame boundary — the logger's own `end_frame` and the `do_frame`
continuation after it — through `tools/explore/frame_snapshot.py`, which
demands the receipt, both hooks and the logger-return check.
`tools/explore/frame_state.py` decodes it through the PDB and RTTI;
`tools/recomp/snapshot.py` maps it as guest memory. Measured on the one
packet taken (Great Lakes, logger frame 11,186): 221 s launch to restore
on a narrow logging window, 843 MB, 177 ranges, 250 ms to copy, 5.5 s to
decode warm; 516,369 of 602,211 printed occurrences agree with the logger,
every observed `GUY` record whole, and the terrain's exact single bits.
Not established by it: atomicity across threads, logger parity, live
allocation coverage. **`Game::do_frame` is not enterable on a packet**
(items 680 and 1014, parked 685): the plan copies the process's private
data and not the game thread's stack, and the first call faults on a
renderer object outside the packet, so `step4.py`'s default entry is
unusable and a brief names the function it enters — `think_scout`,
`compare_target` — which both items did. **A packet of a pair past the
first carries its lobby field**: `unattended_capture.py --profile
KEY=N`, the queue lane's `profile:` key. ~~**A packet at frame N is after tick N−1's
decision**; the frame to take is the one before the divergence.~~ **The
frame to take is the word itself, read as a logger frame** (item 597,
parked 605, the twelfth pass): a packet at logger frame N is the state
after trace tick N−1, so the decision the word's tick makes is still
ahead of it. 597's Mine is first printed on block 10583, its placement is
tick 10582, and the packet that holds the state before it is
`RON_STATE_FRAME=10582` — not the 10581 "the frame before" named, which
would have left tick 10581 to emulate. The block where the placed thing
first appears settles it. Three packets since (run144, run147, run150):
76–221 s and 819–843 MB each. It is a second game on the single capture
lane, serialized with the loop's captures, and disk holds about a hundred
of them.

**A function on the packet.** `tools/recomp/step4.py` runs one function
on the packet under unicorn — the capturing thread's TEB as `fs:`, the
clock, heap and thread imports stubbed — and reports what it called,
`--watch` who writes a range and `--find` the instruction that first
produces a value. That is how item 327's formula was read: `create_units`
to its return, 438k instructions, the `idiv` and the `sar` named, forty
minutes from brief to formula. It is the move that earns the rung a
booking, inside a booked item (571 first), never a frame.

**The native twin.** `tools/recomp/lift.py` turns a function and its
direct callees into C — the integer and scalar-SSE subsets, `fs:` as the
guest TIB, imports through the IAT — and `difftest.py` diffs it row for
row against unicorn on `callfn.py`'s own sweep and on every object of a
packet. Verified: 3,516 sweep rows; `turn_speed` on 268 guys, asserted by
`crates/sim/src/movement.rs`'s `RON_TURN_TABLE` test (skips without the
table); `norm` bit-exact on 1,088 chosen vectors and `air_turn_speed`'s
truncation on 5,715 banks; `create_units` and 264 callees reproducing the
original's `LeaderData` byte for byte. 741 of 771 cited functions lift;
packed SSE, `lock` and x87 stop it. **Where the lift refuses, the float
oracle runs under unicorn on the packet instead** (`step4.py`; item 603,
parked 612): `get_position` and `set_attack` are packed-SSE, so the
pivot's 361 degrees were swept under unicorn on run147's packet and
agreed with `sim::pivot` at a 400× margin — slower per input, the same
answer, no lift to keep. Native is 5–9 ms against unicorn's
22–43 ms on the sweep, both behind `uv`'s six-second start — the twin
pays for a float residue on chosen inputs, not for speed.

**What they are not.** Not the logger (24,000 packets would be 20 TB),
not an answer to "why did it diverge" (the draw stream's), and not a
source: lifted C and packets live under `target/` and `~/ron-data/`,
generated from the user's install, and are read the way the decompiler
is read.
