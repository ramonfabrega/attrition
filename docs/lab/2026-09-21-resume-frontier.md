# Captured delegation reaches A* entry

The September-21 sidecar advances offline replay through `find_upath` to the
original A* entry. Its four observed call arguments match the native trace.
This is a **50-instruction prefix**, not a complete resumed search. The next
refusal is attempted instruction 73: a one-byte read at current unit offset
`+9`, PC `0x6837c8`. No new live capture or fidelity-score change occurred.

## Inputs and boundaries

`tools/explore/resume_frontier.py` first validates the whole context sidecar,
including table regeneration, graph/prefix/trace agreement and executable/tracer
binding. It then supplies captured delegation GPRs/arithmetic flags, stack,
registry header/indexed unit pointer, table allocation/center, saved graph,
mode words and the thread's exception-chain head. FS uses the explicit descriptor
mechanism from L55; the null page remains unmapped. Only four TIB bytes are
exposed because that is all this prefix consumes.

Seven four-byte pathfinder working fields are declared **write-before-read
scratch**, not invented zero-valued inputs. The original initializes every one
before it is read. The byte guards remain in force even where an uncaptured byte
shares a mapped page with a captured graph fragment. This is why the unit-header
read fails although other bytes of the same unit are available.

The direct listing of owned code establishes the write sites and the resumed
branch: `find_upath@0x682f30` enters its saving branch at `0x68332d`, then calls
`a_star@0x683770` from `0x683359`. No executable bytes or disassembly are retained
in git. Working outputs are checked against the captured unit identity, the two
coordinates transformed through the independently checked table, zeroed work
words and the resume flag. The native receipt checks the A* receiver, path-stack
pointer, step budget 48 and anti value 0. It does not check every internal field.

## Measured result and negative controls

On `/Users/rf-studio/ron-data/lab-captures/2026-09-21-restore-context`, frame 224,
owner 0 / id 16:

- A* entry reached in 50 instructions; all four recorded call arguments agree.
- 64 reused-engine resets and a fresh engine give identical observed state and
  write history. Removing the table, center, registry or indexed unit input fails.
- Removing each scratch output or the thread slot fails. Each of four altered
  native arguments and each of seven altered working words is rejected.
- Entering the original A* body reaches instruction 73 and refuses unit `+9`.
  The failure remains a byte-guard refusal; no memory is mapped on demand.

FXSAVE import is still absent. This prefix zeroes XMM0 before its two stores and
executes no x87 operation. A separate run perturbs all eight XMM registers, all
eight physical FP registers, MXCSR and the x87 control word and reproduces the
same observed prefix result. That is evidence for this prefix only, not permission
to omit extended state from a later search body. Unobserved control flags also
remain unproven. The following native A* result is not an emulated return.

## Next acquisition question

The missing byte belongs to the current unit's header. The collector inspected
some header fields when assembling its congestion roster, but did not retain
those bytes at this delegation event. An earlier roster observation is not a
same-frame memory snapshot and is not substituted here. Next to it, the owned
A* listing reads unit `+0xa`, `+0x6c` and `+0x18`; a coherent bounded header
capture is preferable to another single-byte addition. The existing saved graph
starts at unit `+0x104` and remains a structural snapshot, not all search state.

Prepare that acquisition contract offline; request a new slot before launching.
No conclusion here requires reclaiming Fable's capture lane now. Complete A*
resumption, extended-state import and any deeper world/terrain dependencies
remain open.

## Reproduction and validation

```sh
uv run --offline tools/explore/resume_frontier.py /path/to/game /path/to/capture
RON_RESUME_INSTALL=/path/to/game RON_RESUME_CAPTURE=/path/to/capture \
uv run --offline --with unicorn==2.1.4 python -m unittest discover \
  -s tools/explore -p test_resume_frontier.py
```

The four capture-dependent tests pass on the retained artifact. They explicitly
skip when the owned install/capture is absent. Eight thread-context and eleven
byte-guard regressions also pass. The full two-worker repository gate passed:
332 rondata, 855 sim, 13 fixed and three doctests; 782 fixture requests, none
missing. Reports: `/tmp/resume-frontier-gate` and corresponding `.log`.
The retained numerical result and input hashes are under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-21-resume-frontier`.
