# `tools/trace/` — the draw-site trace and function coverage of the original

An in-process instrument for `riseofnations.exe` under Wine. It answers
three questions the loggers cannot:

- **Which function drew?** Every step of the game's LCG, with the caller's
  return address and two more frames of the `ebp` chain, the RNG it hit, the
  seed before the step, and the sim-frame. The dumps show a draw's *outcome*;
  this shows its *site*, which is the only way to place the draws that leave
  no outcome (`docs/SYNC.md` §6).
- **Which functions ran?** Function-entry coverage of the whole executable —
  every one of the 48,233 functions in the Ghidra export — cumulative over
  the run, and per frame inside a chosen window. Against the addresses the
  documents cite, that is the list of **mechanics no run has ever
  exercised**, i.e. the claims that rest on the reading alone.
- **What did a function answer?** A chosen function's arguments *and its
  return value*, over a window of frames. The dumps print state, a draw
  record prints a seed, an `int 3` prints that something ran; a function
  that computes a number and hands it back leaves none of the three.
  `PathFinder::calc_cost` is the first, and the reason this exists: a
  per-step cost dump the `PATHFINDER` gamelog category has no line for
  (`docs/PATHFINDER.md` §10).

How it is established, how it works and what it found is in
`docs/ORACLE.md`, "The draw-site trace and function coverage"; this file is
the how-to.

## Pieces

| file | role |
| --- | --- |
| `tracer.c` | `rontrace.dll`: freestanding 32-bit, kernel32 only, no CRT, no floats. Trampolines `Random::get` (both), `MathUtilFuncSet::rand_real`, `Random::reseed` and `Game::do_frame`; plants `int 3` on every function entry and catches them in a vectored exception handler; **proxies** the `CALLS` sites so their arguments and answers are logged; runs `rontrace.cmd`'s cheat lines at the top of their frames through `ConsoleWin::parse_cmd`. |
| `kernel32.def` | the fourteen imports, stdcall-decorated for `llvm-dlltool -k` |
| `build.sh <install>` | clang (Homebrew LLVM) → `llvm-dlltool` → the pinned toolchain's `rust-lld -flavor link`; then `funcs.py` and `patch_exe.py`. Nothing to install. |
| `funcs.py` | `INDEX.tsv` → `rontrace.funcs`, the function entries as u32 RVAs |
| `patch_exe.py` | `riseofnations.exe` → `riseofnations_trace.exe`: a copy with one added section carrying a copy of the import table plus one descriptor for `rontrace.dll`. The install's own exe is never modified. |
| `wow64bop.c`, `.def`, `.sh` | **not part of a trace** — a 3.5 KB standalone that decides whether `cover=1`'s fault is Wine's or the host's 32-bit emulation. One vectored handler, one `int 3`, one syscall afterwards; no install, no graphics, no display. `docs/ORACLE.md`, "The falsifier for 226, costed". |
| `report.py` | the reader: `summary`, `draws`, `sites`, `coverage`, `functions`, `blind`. `draws` prints **the value each draw returned**: the record carries the seed *before* the step, so stepping the LCG once and applying `Random::get`'s scaling recovers an outcome no dump holds — a setup coin, a direction. |

Everything staged lands in the install directory (`/game`, gitignored):
`rontrace.dll`, `rontrace.funcs`, `rontrace.cfg`, `riseofnations_trace.exe`,
and the output `rontrace.log`. None of it enters the repo — a trace is a list
of the user's executable's addresses.

## Running a trace

```sh
tools/trace/build.sh /path/to/game            # once, or after editing tracer.c
printf 'window=0-3\ncover=1\n' > /path/to/game/rontrace.cfg
wine --bottle ron --workdir /path/to/game --wait-children \
     /path/to/game/riseofnations_trace.exe -config check.ini -automation
```

Then the usual drive (`docs/ORACLE.md`, "Running a check: the recipe in one
place"): Solo Game → Quick Battle → Start, play the frames wanted, quit
through the in-game menu. The log flushes at every frame boundary, so a kill
loses at most the current frame.

`rontrace.cfg`:

- `window=LO-HI` — sim-frames (inclusive) at whose start every function is
  re-armed, giving a per-frame entered set for each. Outside the window the
  arming is one-shot from attach, so the cumulative set is still complete;
  each function's record carries the frame it was first entered on.
- `cover=0` — draws only, no `int 3`s. Fast; use it when the question is only
  the stream. **On this machine it is not a choice**: under free Wine the
  int3 forest cannot run at all, because a thread that has been through the
  32-bit vectored-exception path takes its next 32→64 transition without
  switching mode (`docs/ORACLE.md`, "226: the fault is the bop, not the
  handler"). Captures are unaffected — the draw hooks are plain jumps — but
  `coverage`, `functions` and `blind` below have no new input until the
  oracle runs where 32-bit x86 is native.
- `callwin=LO-HI` — sim-frames over which the **proxied** sites log a record
  a call. Absent, nothing is patched and the run is byte-for-byte the
  instrument every capture up to run54 used, so leaving it out is how an
  earlier capture is reproduced.

## The call proxies

`tracer.c`'s `CALLS` table names functions whose *answer* is the question.
Each is replaced by a proxy that logs the arguments, calls the original
through the displaced-prologue trampoline, and logs `eax` — so the record is
`(arguments, return)` rather than "this ran". The arguments live in the
proxy's own frame, so recursion and re-entrancy cost nothing.

Eight are proxied today. Two are `docs/PATHFINDER.md`'s:
`PathFinder::astar_path@00683770`, whose entry and return **delimit one
search**, and `PathFinder::calc_cost@00684e50`, which is §5's per-step price.
Three are `docs/SYNC.md` §3.9's, and together they are **the record owner 9
never had**: `Unit::do_air_physics@005e86d0` brackets one flying unit's
frame and carries the patrol point it steers at, `Unit::air_turn_speed@
005ea390` answers the frame's turn rate (and so the bank angle nothing
dumps), and `Unit::set_new_location@005f8d20` is where the step landed.
`report.py … calls` prints them nested, with each world coordinate's cell
beside it; `rondata::trace::Call` is the Rust reader, so a `#[test]` can put
the original's price beside the simulation's for the same step.

Three more are `docs/ROADS.md` §7.2's, and they are the same trick with a
predicate where the mutator was: `PathFinder::astar_caravan_road@00685990`
brackets one road plan, `PathFinderData::valid_roadcoord@00688740` is the
gate — **the only record that carries a candidate's coordinate** — and
`PathFinder::calc_road_cost@00686300` is the price.
`rondata::trace::Trace::road_nodes` pairs each price with the gate that
returned just before it, because `calc_road_cost` is handed a pooled
`PathNode *` and its own arguments name no tile at all.

**The bracket is the identity.** `set_new_location` is taken by every unit
that moves, so what makes a record a *bird's* is that a `do_air_physics` on
the same `this` is still open when it returns — `Trace::air_frames` folds on
that, and `Trace::air_births` picks out the `(x, y, 1, 1)` call that put the
unit down before it ever flew.

Adding a site needs three things from the listing, and getting any of them
wrong corrupts the stack rather than failing loudly: the **prologue bytes**
(at least five, whole instructions, no rel-relative operand), the **argument
count**, and the fact that it is `__thiscall` and callee-clean — read the
`ret <imm>` at the end of the function and divide by four. The table carries
the prologue it expects and refuses on a mismatch, which catches a wrong
address but not a wrong arity.

A proxied entry carries no `int 3`, so a proxied function has **no HIT
record**; `report.py blind` counts a `CALL` record as its entry instead.

## Staging a scenario from a file: `rontrace.cmd`

The third instrument (`docs/ORACLE.md`, "The cheat channel"). One entry per
line, `<sim-frame> <text>`; at the entry of `Game::do_frame` for that frame
the text goes to `ConsoleWin::parse_cmd` exactly as the chat box would send
it with `cheat ` stripped — or, with a leading `!`, as a console command
(the console-only half of the table: `quit`, `ai off`, `pause`, `ffwd`).
`#` starts a comment. Lines run in file order; a frame lower than the
previous line's is clamped to it. ASCII only. Run16's scenario, which is
also the validation run (run16b):

```
300 peace who=1
330 add hoplite who=0 206,78
332 add supply who=0 208,80
334 add scout who=0 204,76
900 war who=1
1000 tech who=1 allegiance on
1300 die 9,0
1500 tech who=1 oath on
1700 add hoplite who=1 42,146
1900 tech who=0 allegiance on
2100 move 6 190,60
2400 !quit
```

A line runs before the frame's phases, so its effect is in the dump block
labelled `frame + 1` (the label is one ahead of the sim-frame). Each
executed line is an `INFO cmd` record — frame, line index, chat/console,
`parse_cmd`'s return — and `INFO cmds` at attach is the count parsed.
`no_mouse` is passed, so there is no cursor tile: always give `add`/`move`
their `x,y`. The line does not travel the order stream; a recording of the
run does not contain it. Nothing here is faster than the dump: at
`UNITS=3` the sim runs ~3 frames a second, so budget the frames.

Cost: an `int 3` is one exception per function per arming — a few thousand
per re-armed frame — which under Rosetta and Wine's WoW64 is a fraction of a
second. The draw hooks are plain jumps and cost nothing measurable.

## Reading it

```sh
tools/trace/report.py game/rontrace.log summary
tools/trace/report.py game/rontrace.log draws 0          # every draw of sim-frame 0, named
tools/trace/report.py game/rontrace.log sites setup 0 1  # folded by site, with counts
tools/trace/report.py game/rontrace.log coverage 0 1 2 3 # functions entered per frame
tools/trace/report.py game/rontrace.log calls 1477       # the proxied calls of one frame
tools/trace/report.py game/rontrace.log functions        # every function, first frame
tools/trace/report.py game/rontrace.log when Guy::init_real 2   # frames with >= 2 units born
tools/trace/report.py game/rontrace.log blind docs/ [more logs...]
```

There is a **second reader** since 2026-08-26, in Rust:
`rondata::trace` parses the same file, and `rondata --trace <log>` prints
the per-frame fold by site. It exists so a `#[test]` can assert a
mechanic's draw *sequence* against the original's without shelling out
(`docs/SYNC.md` §5, `docs/SCOUT.md` §12). It cannot name an address —
that needs `INDEX.tsv`, which is outside the repo — so `report.py` stays
the reader for anything a human is reading.

`when` is the verb for **dating an event across a whole run**, and it is
cheaper than any capture: a draw made from `Guy::init_real` is one unit
created, so `when Guy::init_real 2` lists every frame on which two or more
units were born. On Great Lakes that is three at 6612 — run76's Archer
squad — and running it over East Indies' own 24,000-frame trace named 15782
without taking anything. Grep the trace before booking a run
(`docs/ORACLE.md`, run77, which was booked off function coverage and missed
by five thousand frames).

`blind` collects every `name@00xxxxxx` the documents cite, and lists the
ones no given trace entered. A cited function that no run has driven is a
claim with no behavioural check behind it — the list is the queue for the
next checks, not a verdict on the reading.

Frames are the original's `Game::frame` read at `do_frame` entry — sim-frames
as `docs/SYNC.md` counts them, not the log's `FRAME n` labels (which are
one ahead; `docs/ORACLE.md`, "The frame label, settled"). Records before the
first frame are the setup path (`setup`, frame −1). Every FRAME record also
carries `game_random`'s word at that moment, which is how a trace is aligned
with a dump's checksum trace (`tools/gamelog/rngtrace.py`).

## Log format

32-byte header, then 32-byte records of eight little-endian u32:

| kind | a | b | c | d | e | f | g |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 HIT | function VA | thread id | | | | | frame |
| 7 CALL | site id | `this` | arg0 | arg1 | arg2 | arg3 | frame |
| 8 RET | site id | `eax` | arg4 | arg5 | arg6 | the byte behind arg7, or ~0 | frame |
| 1 `get()`, 3 `get(a,b)`, 4 `rand_real`, 6 `reseed` | caller (return address) | `Random*` | seed before | caller's caller | and its caller | arg0 (`a` for `get(a,b)`, the new seed for `reseed`) | frame |
| 2 FRAME | frame | `game_random` word | functions re-armed | `do_frame`'s caller | | | frame |
| 5 INFO | code | … | | | | | frame |

The header is `RONT`, version, image base, `.text` RVA and size, functions
listed, window lo, window hi. INFO codes: 1 attach, 2 hook-mismatch (the
prologue bytes were not the expected ones — the hook was refused), 3 hooked,
4 no function list, 5 VirtualProtect failed, 6 armed, 7 detach, 8 declined,
9 cmd, 10 cmd-noconsole, 11 cmds, 12 proxied (a call site was replaced:
rva, stub, argument count).

## Traps

- **The hooks are for this executable only.** Each carries the ten prologue
  bytes it expects and refuses on a mismatch (INFO 2). A different build
  needs its addresses re-read from its own PDB.
- **`Random::get` is not the only stepping site.** `MathUtilFuncSet::rand_real`
  inlines the LCG on `game_random`, and `NukeOut::init_shroom_fire` steps a
  local seed (graphics, not hooked). Four sites in `.text` carry the
  multiplier; a fifth would show up as a seed jump between consecutive draw
  records.
- **`int 3` on a function entry that is not one** would corrupt data. The
  list is Ghidra's function set with the PDB's names, all inside `.text`;
  nothing in it is data. Import thunks are code and are armed like anything
  else.
- The exception handler resumes at the function's first byte after restoring
  it, so a function that is entered by two threads at once is logged twice —
  harmless.
