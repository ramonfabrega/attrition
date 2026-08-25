# `tools/trace/` — the draw-site trace and function coverage of the original

An in-process instrument for `riseofnations.exe` under CrossOver. It answers
two questions the loggers cannot:

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

How it is established, how it works and what it found is in
`docs/ORACLE.md`, "The draw-site trace and function coverage"; this file is
the how-to.

## Pieces

| file | role |
| --- | --- |
| `tracer.c` | `rontrace.dll`: freestanding 32-bit, kernel32 only, no CRT, no floats. Trampolines `Random::get` (both), `MathUtilFuncSet::rand_real`, `Random::reseed` and `Game::do_frame`; plants `int 3` on every function entry and catches them in a vectored exception handler. |
| `kernel32.def` | the fourteen imports, stdcall-decorated for `llvm-dlltool -k` |
| `build.sh <install>` | clang (Homebrew LLVM) → `llvm-dlltool` → the pinned toolchain's `rust-lld -flavor link`; then `funcs.py` and `patch_exe.py`. Nothing to install. |
| `funcs.py` | `INDEX.tsv` → `rontrace.funcs`, the function entries as u32 RVAs |
| `patch_exe.py` | `riseofnations.exe` → `riseofnations_trace.exe`: a copy with one added section carrying a copy of the import table plus one descriptor for `rontrace.dll`. The install's own exe is never modified. |
| `report.py` | the reader: `summary`, `draws`, `sites`, `coverage`, `functions`, `blind` |

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
  the stream.

Cost: an `int 3` is one exception per function per arming — a few thousand
per re-armed frame — which under Rosetta and Wine's WoW64 is a fraction of a
second. The draw hooks are plain jumps and cost nothing measurable.

## Reading it

```sh
tools/trace/report.py game/rontrace.log summary
tools/trace/report.py game/rontrace.log draws 0          # every draw of sim-frame 0, named
tools/trace/report.py game/rontrace.log sites setup 0 1  # folded by site, with counts
tools/trace/report.py game/rontrace.log coverage 0 1 2 3 # functions entered per frame
tools/trace/report.py game/rontrace.log functions        # every function, first frame
tools/trace/report.py game/rontrace.log blind docs/ [more logs...]
```

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
| 1 `get()`, 3 `get(a,b)`, 4 `rand_real`, 6 `reseed` | caller (return address) | `Random*` | seed before | caller's caller | and its caller | arg0 (`a` for `get(a,b)`, the new seed for `reseed`) | frame |
| 2 FRAME | frame | `game_random` word | functions re-armed | `do_frame`'s caller | | | frame |
| 5 INFO | code | … | | | | | frame |

The header is `RONT`, version, image base, `.text` RVA and size, functions
listed, window lo, window hi. INFO codes: 1 attach, 2 hook-mismatch (the
prologue bytes were not the expected ones — the hook was refused), 3 hooked,
4 no function list, 5 VirtualProtect failed, 6 armed, 7 detach.

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
